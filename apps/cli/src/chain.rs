//! The document's plugin chain, as one reusable function.
//!
//! A Pendon document is a stream of events plus a **chain** of plugins: construct
//! plugins first (marker, directive, blockquote, section, list), then the block
//! plugins (img, table, heading, cite, anchor), then `markdown` — which expands
//! raw text into nodes — and finally the stages that only make sense for a whole
//! document (`bind`'s `resolve`, the citation export).
//!
//! [`run`] is that chain. [`render_mdp`] runs the *same* chain over a nested
//! `{{{mdp[var] … }}}` payload, which is what makes a bound payload behave like
//! the surrounding document: wiki links, citations, figures, tables, math,
//! extras heads and custom components all work inside it, and the hints their
//! plugins need are collected into the host document's hint list.

use std::collections::{HashMap, HashSet};

use pendon_core::{ContextPipeline, Event, NodeKind};
use pendon_plugin_anchor::AnchorOptions;
use pendon_plugin_bind::{Fragment, MdpMode};
use pendon_plugin_blockquote::BlockquoteOptions;
use pendon_plugin_custom::PluginSpec;
use pendon_plugin_directive::DirectiveOptions;
use pendon_plugin_heading::HeadingOptions;
use pendon_plugin_image::ImageOptions;
use pendon_plugin_latex::LatexOptions;
use pendon_plugin_list::ListOptions;
use pendon_plugin_markdown::MarkdownOptions;
use pendon_plugin_marker::MarkerOptions;
use pendon_plugin_section::SectionOptions;
use pendon_plugin_table::TableOptions;
use pendon_plugin_wiki::WikiOptions;
use pendon_renderer_solid::SolidRenderHints;
use serde_json::Value;

use crate::plugins::{
    build_anchor_options, build_blockquote_options, build_context_inline_pipeline,
    build_directive_options, build_heading_options, build_image_options, build_list_options,
    build_marker_options, build_section_options, build_table_options, latex_target_for_format,
    load_custom_spec, plugin_names, process_stateless_plugin, track_used_spec, DocumentContext,
};

/// Everything the chain needs to know about a task: the plugin list and the
/// resolved options of every plugin in it.
///
/// [`ChainConfig::new`] fills it with defaults (the `pendon <file>` path); the
/// task path overrides the fields the `[task.<plugin>]` tables configure.
pub struct ChainConfig {
    pub task_name: String,
    /// `task.plugin` / `--plugin`, verbatim: the chain's order is the author's.
    pub plugin_list: Option<String>,
    pub enabled: HashSet<String>,
    pub markdown: MarkdownOptions,
    pub wiki: WikiOptions,
    pub latex: Option<LatexOptions>,
    pub image: ImageOptions,
    pub anchor: AnchorOptions,
    pub marker: MarkerOptions,
    pub directive: DirectiveOptions,
    pub blockquote: BlockquoteOptions,
    pub section: SectionOptions,
    pub list: ListOptions,
    pub table: TableOptions,
    pub heading: HeadingOptions,
    /// A `[task.plugin.vicado.renderer.solid]` override (§11), when configured.
    pub vicado_hints: Option<SolidRenderHints>,
}

impl ChainConfig {
    /// The default configuration for `plugin_list`: every plugin at its
    /// documented defaults.
    pub fn new(
        task_name: &str,
        plugin_list: Option<&str>,
        format: &str,
        markdown: MarkdownOptions,
        wiki: WikiOptions,
    ) -> Result<Self, String> {
        let enabled: HashSet<String> = plugin_names(plugin_list)
            .into_iter()
            .map(str::to_string)
            .collect();
        let latex = if enabled.contains("latex") {
            Some(LatexOptions {
                target: latex_target_for_format(format),
            })
        } else {
            None
        };
        // §9.5: when the section outline owns the ids, the heading yields its id
        // (it transfers to the section).
        let mut heading = build_heading_options(None)?;
        heading.section_owns_id = enabled.contains("section");

        Ok(Self {
            task_name: task_name.to_string(),
            plugin_list: plugin_list.map(str::to_string),
            enabled,
            markdown,
            wiki,
            latex,
            image: build_image_options(None)?,
            anchor: build_anchor_options(None)?,
            marker: build_marker_options(None)?,
            directive: build_directive_options(None)?,
            blockquote: build_blockquote_options(None)?,
            section: build_section_options(None)?,
            list: build_list_options(None)?,
            table: build_table_options(None)?,
            heading,
            vicado_hints: None,
        })
    }

    /// `true` when `name` is in the task's plugin list.
    pub fn has(&self, name: &str) -> bool {
        self.enabled.contains(name)
    }

    /// The inline pipeline block plugins run over their own children (captions,
    /// cells, table bodies): the enabled inline plugins, in the document's own
    /// order.
    pub fn pipeline(&self) -> ContextPipeline<DocumentContext> {
        let enabled: HashSet<&str> = self.enabled.iter().map(String::as_str).collect();
        build_context_inline_pipeline(
            self.image.clone(),
            self.wiki.clone(),
            self.anchor.clone(),
            self.latex,
            &enabled,
        )
    }
}

/// The chain's accumulated state: one document context (citations are shared by
/// the whole page, including bound payloads), the custom-plugin cache, and the
/// render hints the plugins produced.
pub struct ChainState {
    pub context: DocumentContext,
    pub custom_registry: HashMap<String, PluginSpec>,
    pub custom_cache: HashMap<String, PluginSpec>,
    pub hints: Vec<SolidRenderHints>,
    pub used_specs: Vec<PluginSpec>,
    pub used_quiz: bool,
    pub used_vicado: bool,
    pub anchor_ran: bool,
    pub cite_ran: bool,
    /// §13: report each unknown plugin name once per task.
    reported: HashSet<String>,
    pipeline: ContextPipeline<DocumentContext>,
}

impl ChainState {
    pub fn new(
        context: DocumentContext,
        custom_registry: HashMap<String, PluginSpec>,
        pipeline: ContextPipeline<DocumentContext>,
    ) -> Self {
        Self {
            context,
            custom_registry,
            custom_cache: HashMap::new(),
            hints: Vec::new(),
            used_specs: Vec::new(),
            used_quiz: false,
            used_vicado: false,
            anchor_ran: false,
            cite_ran: false,
            reported: HashSet::new(),
            pipeline,
        }
    }
}

/// What the chain is running over.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Document {
    /// The page itself. [`prelude`] has already run its text stages, so the
    /// chain starts from the frontmatter node the citation context was built
    /// from.
    Page,
    /// A nested `mdp` payload: fresh text, no document-level exports.
    Payload,
}

/// Step 1 of the chain: the stages that must run **before** the citation
/// context is built.
///
/// `micromatter` rewrites the raw text, and it is what turns a leading `---`
/// block into the `Frontmatter` node the context reads its `references` from —
/// so the context cannot be constructed before it runs. Every other plugin
/// needs the context, so this is the only stage that comes first.
pub fn prelude(events: Vec<Event>, config: &ChainConfig) -> Vec<Event> {
    if config.has("micromatter") {
        pendon_plugin_micromatter::process(&events)
    } else {
        events
    }
}

/// Runs the whole chain over one document, after [`prelude`].
pub fn run(events: Vec<Event>, config: &ChainConfig, state: &mut ChainState) -> Vec<Event> {
    run_chain(events, config, state, Document::Page)
}

/// Runs the chain over `events`.
///
/// A [`Document::Payload`] is a sub-document, so the stages that export
/// document-level data (`extract-heading`'s `headings`) are skipped. Every other
/// plugin runs exactly as it does for the page.
fn run_chain(
    mut events: Vec<Event>,
    config: &ChainConfig,
    state: &mut ChainState,
    document: Document,
) -> Vec<Event> {
    let fragment = document == Document::Payload;
    // A payload is a sub-document parsed from its own text, so its text stages
    // have not run yet; the page's already have, in `prelude`.
    if document == Document::Payload {
        events = prelude(events, config);
    }

    let mut markdown_ran = false;
    let mut quiz_pending = false;

    for name in plugin_names(config.plugin_list.as_deref()) {
        // `micromatter` already ran in `prelude`, `bind` is a pre-parse +
        // resolve stage with no work of its own inside the loop (ADR-0004).
        if name == "micromatter" || name == "bind" {
            continue;
        }
        // `extract-heading` exports `headings` for the *document*; a payload has
        // no such export to contribute to.
        if fragment && name == "extract-heading" {
            continue;
        }

        if let Some(path) = name.strip_prefix("toml:") {
            let spec = match load_custom_spec(path, &mut state.custom_cache) {
                Ok(spec) => spec,
                Err(msg) => {
                    eprintln!("Error: {}", msg);
                    continue;
                }
            };
            track_used_spec(&mut state.used_specs, spec.clone());
            events = pendon_plugin_custom::process_with_context(
                &events,
                &spec,
                &state.pipeline,
                &mut state.context,
            );
            continue;
        }

        if let Some(processed) =
            process_stateless_plugin(name, &events, &config.wiki, config.latex.as_ref())
        {
            events = processed;
            continue;
        }

        events = match name {
            "quiz" => {
                state.used_quiz = true;
                if let Some(spec) = state.custom_registry.get("quiz") {
                    track_used_spec(&mut state.used_specs, spec.clone());
                }
                if markdown_ran {
                    pendon_plugin_quiz::process(&events)
                } else {
                    quiz_pending = true;
                    events
                }
            }
            "image" => {
                let image_opts = &config.image;
                let result = pendon_plugin_image::process_with_context(
                    &events,
                    image_opts,
                    &state.pipeline,
                    &mut state.context,
                );
                if let Some(hints) = pendon_plugin_image::solid_hints(image_opts) {
                    state.hints.push(hints);
                }
                result
            }
            "table" => {
                let table_opts = &config.table;
                let result = pendon_plugin_table::process_with_context(
                    &events,
                    table_opts,
                    &state.pipeline,
                    &mut state.context,
                );
                if let Some(hints) = pendon_plugin_table::solid_hints(table_opts) {
                    state.hints.push(hints);
                }
                result
            }

            "heading" => {
                let result = pendon_plugin_heading::process(&events, &config.heading);
                if let Some(hints) = pendon_plugin_heading::solid_hints(&config.heading) {
                    state.hints.push(hints);
                }
                result
            }
            "anchor" => {
                state.anchor_ran = true;
                pendon_plugin_anchor::process(&events, &config.anchor)
            }
            "cite" => {
                state.cite_ran = true;
                state.context.process_citations(&events)
            }
            "marker" => {
                // §10.1: markers are plain text patterns, so this plugin runs
                // before markdown like the other construct plugins.
                let result = pendon_plugin_marker::process(&events, &config.marker);
                if let Some(hints) = pendon_plugin_marker::solid_hints(&config.marker) {
                    state.hints.push(hints);
                }
                result
            }
            "directive" => {
                // §10.2/§10.3: directives are text constructs, so this plugin
                // runs before markdown like the other construct plugins.
                let result = pendon_plugin_directive::process(&events, &config.directive);
                if let Some(hints) = pendon_plugin_directive::solid_hints(&config.directive) {
                    state.hints.push(hints);
                }
                result
            }
            "blockquote" => {
                // §9.2: `>` stays plain Markdown; this plugin only binds the
                // extras a quote may carry, so it runs before markdown re-lexes
                // the stripped body as blocks.
                let result = pendon_plugin_blockquote::process(&events, &config.blockquote);
                if let Some(hints) = pendon_plugin_blockquote::solid_hints(&config.blockquote) {
                    state.hints.push(hints);
                }
                result
            }
            "section" => {
                // §9.5: the section outline runs before markdown so it can see
                // the raw heading text, the section decorator and the level
                // markers (`>---<` would otherwise be a blockquote). Markdown
                // then parses the interior of every `Section` it emits.
                let result = pendon_plugin_section::process(&events, &config.section);
                if let Some(hints) = pendon_plugin_section::solid_hints(&config.section) {
                    state.hints.push(hints);
                }
                result
            }
            "list" => {
                // §9.3: the list markers stay plain Markdown; this plugin only
                // binds a container decorator to the list below it, so it runs
                // before markdown parses the list.
                let result = pendon_plugin_list::process(&events, &config.list);
                if let Some(hints) = pendon_plugin_list::solid_hints(&config.list) {
                    state.hints.push(hints);
                }
                result
            }
            "vicado" => {
                state.used_vicado = true;
                pendon_plugin_vicado::process(&events)
            }
            "markdown" => {
                let result = pendon_plugin_markdown::process_with_options(&events, config.markdown);
                markdown_ran = true;
                if quiz_pending {
                    quiz_pending = false;
                    pendon_plugin_quiz::process(&result)
                } else {
                    result
                }
            }
            other => {
                if let Some(spec) = state.custom_registry.get(other) {
                    let spec = spec.clone();
                    track_used_spec(&mut state.used_specs, spec.clone());
                    pendon_plugin_custom::process_with_context(
                        &events,
                        &spec,
                        &state.pipeline,
                        &mut state.context,
                    )
                } else {
                    // §13: an unknown plugin name used to be ignored without a
                    // word. Report it once per name; the build still succeeds.
                    if state.reported.insert(other.to_string()) {
                        eprintln!(
                            "Warning: [{}] unknown plugin '{}' in task.plugin; \
                             it is ignored (§13)",
                            config.task_name, other
                        );
                    }
                    events
                }
            }
        };
    }

    if quiz_pending {
        events = pendon_plugin_quiz::process(&events);
    }
    events
}

/// The hints of the plugins that produce them once, for the whole document.
///
/// Called after `bind`'s `resolve` (which may render payloads through the chain)
/// and after the citation export, so a quiz or a `Vicado` that only appears
/// inside a bound payload still imports what it needs.
pub fn finish(config: &ChainConfig, state: &mut ChainState) {
    if state.used_quiz {
        state.hints.push(pendon_plugin_quiz::solid_hints());
    }
    if state.used_vicado {
        let hints = match &config.vicado_hints {
            Some(override_hints) => override_hints.clone(),
            None => pendon_plugin_vicado::solid_hints(),
        };
        state.hints.push(hints);
    }
    if state.anchor_ran {
        if let Some(hints) = pendon_plugin_anchor::solid_hints(&config.anchor) {
            state.hints.push(hints);
        }
    }
}

/// Renders an `mdp` payload through the document's chain — the renderer
/// `plugin-bind::resolve_with` calls (§19, §2.3 of RFC `plugin-bind`).
///
/// The payload is parsed as its own document, run through the chain, and turned
/// into a fragment: the node list every renderer consumes. In the inline form the
/// payload's own paragraph wrapper is dropped, so text written on the head line
/// does not gain a block wrapper it never had.
pub fn render_mdp(
    markdown: &str,
    mode: MdpMode,
    config: &ChainConfig,
    state: &mut ChainState,
) -> Fragment {
    let parsed = pendon_core::parse(markdown, &pendon_core::Options::default());
    let processed = run_chain(parsed, config, state, Document::Payload);
    let (content, diagnostics) = pendon_core::split_diagnostics(processed);
    let content = match mode {
        MdpMode::Inline => unwrap_document(content),
        MdpMode::Block => content,
    };

    match pendon_renderer_ast::events_to_children(&content) {
        Ok(nodes) => Fragment {
            value: pendon_renderer_ast::fragment(nodes),
            diagnostics,
        },
        // A payload that cannot be turned into a tree still binds: it degrades to
        // text rather than dropping the attribute (§4.3-style resilience).
        Err(_) => Fragment {
            value: Value::String(markdown.to_string()),
            diagnostics,
        },
    }
}

/// Strips the document wrapper — and, when the payload is one paragraph, that
/// paragraph too — so the inline form contributes inline nodes.
fn unwrap_document(mut events: Vec<Event>) -> Vec<Event> {
    if matches!(events.first(), Some(Event::StartNode(NodeKind::Document))) {
        events.remove(0);
    }
    if matches!(events.last(), Some(Event::EndNode(NodeKind::Document))) {
        events.pop();
    }
    let one_paragraph = matches!(events.first(), Some(Event::StartNode(NodeKind::Paragraph)))
        && matches!(events.last(), Some(Event::EndNode(NodeKind::Paragraph)))
        && single_top_level(&events);
    if one_paragraph {
        events.remove(0);
        events.pop();
    }
    events
}

/// `true` when the stream holds exactly one top-level node.
fn single_top_level(events: &[Event]) -> bool {
    let mut depth = 0usize;
    for (index, event) in events.iter().enumerate() {
        match event {
            Event::StartNode(_) => depth += 1,
            Event::EndNode(_) => {
                depth = depth.saturating_sub(1);
                if depth == 0 && index + 1 != events.len() {
                    return false;
                }
            }
            _ => {}
        }
    }
    depth == 0
}
