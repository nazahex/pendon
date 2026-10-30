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
use pendon_extract::{ExtractOptions, ExtractOutput};
use pendon_number::NumberOptions;
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
    /// `[[task.<element>.number]]` + `[[task.number.custom]]` (ADR-0010).
    pub number: NumberOptions,
    /// `[[task.<element>.extract]]` — the per-element extraction blocks
    /// (`docs/archive/fulfilled/plugin-extract.md`).
    pub extract: ExtractOptions,
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
        // ADR-0010: `number` reads attributes, so the chain's order is part of
        // the contract. Fail before a document is parsed rather than silently
        // numbering nothing (or only half the elements).
        let ordered = plugin_names(plugin_list);
        validate_number_order(task_name, &ordered)?;
        // plugin-extract reads the attributes every element plugin (and
        // `number`) has emitted, so it must be the last of them: a block that
        // ran early would extract a node without its `__layer` or its number.
        validate_extract_order(task_name, &ordered)?;
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
            number: NumberOptions::default(),
            extract: ExtractOptions::default(),
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

/// §7.3 of `workflows/numbering-extract-type-export.md`: `number` must follow
/// every element plugin that emits attributes and precede `markdown`.
///
/// A misordered chain is rejected before a document is parsed, because the
/// alternative — numbering nothing, or only the elements a later plugin has
/// already converted — is invisible in the output (ADR-0010).
pub fn validate_number_order(task_name: &str, names: &[&str]) -> Result<(), String> {
    let Some(position) = names.iter().position(|name| *name == "number") else {
        return Ok(());
    };
    if let Some(markdown) = names.iter().position(|name| *name == "markdown") {
        if markdown < position {
            return Err(format!(
                "[{task_name}] plugin `number` comes after `markdown`; `number` must run before `markdown` so it sees the parsed elements (ADR-0010)"
            ));
        }
    }
    for (index, name) in names.iter().enumerate() {
        if index > position && crate::plugins::NUMBER_ELEMENTS.contains(name) {
            return Err(format!(
                "[{task_name}] plugin `number` comes before `{name}`; `number` must follow every element plugin that emits attributes (ADR-0010)"
            ));
        }
    }
    Ok(())
}

/// `plugin-extract` reads the `__layer` tag and the extras every element plugin
/// emitted, so it must come **after** them (and after `number`, whose
/// `citation_number`/`--number` fields a block can select).
///
/// A misordered chain is rejected before a document is parsed: extraction that
/// runs early writes rows missing the very fields the block asked for, which is
/// invisible in the output file.
pub fn validate_extract_order(task_name: &str, names: &[&str]) -> Result<(), String> {
    let Some(position) = names.iter().position(|name| *name == "extract") else {
        return Ok(());
    };
    for (index, name) in names.iter().enumerate() {
        if index > position
            && (crate::plugins::NUMBER_ELEMENTS.contains(name)
                || *name == "number"
                // D-14: `micromatter` is what turns the `---` block into the
                // frontmatter node a `[[task.frontmatter.extract]]` reads.
                || *name == "micromatter")
        {
            return Err(format!(
                "[{task_name}] plugin `extract` comes before `{name}`; `extract` must follow every plugin whose attributes it reads (docs/archive/fulfilled/plugin-extract.md)"
            ));
        }
    }
    Ok(())
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
    pub anchor_ran: bool,
    pub cite_ran: bool,
    /// What this document's `[[task.<element>.extract]]` blocks produced: the
    /// side files to write and the exports the solid renderer registers. Filled
    /// by the `extract` stage, consumed once the page has rendered.
    pub extract: ExtractOutput,
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
            anchor_ran: false,
            cite_ran: false,
            extract: ExtractOutput::default(),
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
        pendon_plugin_frontmatter::process(&events)
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
/// A [`Document::Payload`] is a sub-document: it runs every plugin for its
/// **in-document** effects (so a citation inside a payload joins the page's list,
/// ADR-0007), but only a [`Document::Page`] writes **document-level outputs** —
/// side files and `export const` registrations. `extract` is the one remaining
/// stage that produces those, so it is the one stage a payload skips.
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

    for name in plugin_names(config.plugin_list.as_deref()) {
        // `micromatter` already ran in `prelude`, `bind` is a pre-parse +
        // resolve stage with no work of its own inside the loop (ADR-0004).
        if name == "micromatter" || name == "bind" {
            continue;
        }
        // Only a `Page` writes document-level outputs: `extract`'s blocks describe
        // the page (they write side files and register `export const`), so a
        // payload does not run them. Every other plugin keeps its in-document
        // effect inside a payload.
        if fragment && name == "extract" {
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
            "image" => {
                let image_opts = &config.image;
                let result = pendon_plugin_image::process(&events, image_opts);
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

            "number" => {
                // ADR-0010: numbering runs after every element plugin has emitted
                // its attributes and before `markdown` re-lexes the rest. The
                // position is validated up front (`validate_number_order`), so a
                // misordered chain is a config error, not a silent difference.
                pendon_number::process(&events, &config.number)
            }
            "extract" => {
                // P.2 of `docs/archive/fulfilled/plugin-extract.md`: extraction only *reads*
                // the stream the chain has built so far, so it returns the events
                // untouched and leaves the rows for `process` to write once the
                // page itself rendered. The position is validated up front
                // (`validate_extract_order`): a block that ran before its
                // element plugin would extract a node without its attributes.
                state.extract = pendon_extract::run(&events, &config.extract);
                events
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
            "markdown" => {
                // D-10 rules 2–3: the Markdown pass is the chain's last binder,
                // so it also needs to know which receivers the chain has — a
                // decorator for a construct whose plugin is missing can never be
                // claimed and must be dropped instead of rendering as text.
                let receivers = pendon_extra::ReceiverSet::from_plugins(
                    config.enabled.iter().map(String::as_str),
                );
                pendon_plugin_markdown::process_with_receivers(&events, config.markdown, receivers)
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

    events
}

/// The hints of the plugins that produce them once, for the whole document.
///
/// Called after `bind`'s `resolve` (which may render payloads through the chain)
/// inside a bound payload still imports what it needs.
pub fn finish(config: &ChainConfig, state: &mut ChainState) {
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

#[cfg(test)]
mod tests {
    use super::{
        run_chain, validate_extract_order, validate_number_order, ChainConfig, ChainState, Document,
    };
    use pendon_extract::{BlockSpec, ExtractBlock, ExtractOptions};

    /// ADR-0010: the only accepted shape is "every element plugin, then
    /// `number`, then `markdown`".
    #[test]
    fn number_after_the_elements_and_before_markdown_is_accepted() {
        assert!(validate_number_order(
            "T",
            &["micromatter", "heading", "anchor", "number", "markdown"]
        )
        .is_ok());
        // `markdown` is optional (a chain may end with `number`).
        assert!(validate_number_order("T", &["heading", "number"]).is_ok());
    }

    /// A chain without `number` numbers nothing but is still a valid chain.
    #[test]
    fn a_chain_without_number_is_accepted() {
        assert!(validate_number_order("T", &["heading", "markdown"]).is_ok());
        assert!(validate_number_order("T", &[]).is_ok());
    }

    /// `number` reads attributes, so it must not run after `markdown` has
    /// consumed them into components.
    #[test]
    fn number_after_markdown_is_rejected() {
        let err = validate_number_order("T", &["heading", "markdown", "number"])
            .expect_err("misordered chain");
        assert!(
            err.contains("[T] plugin `number` comes after `markdown`"),
            "{err}"
        );
    }

    /// An element plugin that runs *after* `number` would emit its attributes
    /// too late — silently unnumbered output is the failure mode this guard
    /// exists for.
    #[test]
    fn number_before_an_element_plugin_is_rejected() {
        let err = validate_number_order("T", &["number", "heading", "markdown"])
            .expect_err("misordered chain");
        assert!(
            err.contains("[T] plugin `number` comes before `heading`"),
            "{err}"
        );
    }

    /// plugin-extract reads the attributes the element plugins (and `number`)
    /// emitted, so it must be the last of them.
    #[test]
    fn extract_after_every_attribute_plugin_is_accepted() {
        assert!(
            validate_extract_order("T", &["heading", "cite", "number", "extract", "markdown"])
                .is_ok()
        );
        // A chain without `extract` extracts nothing but stays valid.
        assert!(validate_extract_order("T", &["heading", "markdown"]).is_ok());
        assert!(validate_extract_order("T", &[]).is_ok());
    }

    /// A block that ran before its element plugin would write rows missing the
    /// very attributes it asked for — invisible in the output file.
    #[test]
    fn extract_before_an_element_plugin_is_rejected() {
        let err = validate_extract_order("T", &["extract", "cite", "number"])
            .expect_err("misordered chain");
        assert!(
            err.contains("[T] plugin `extract` comes before `cite`"),
            "{err}"
        );
    }

    /// `--number` is a field a block may select, so `number` must run first too.
    #[test]
    fn extract_before_number_is_rejected() {
        let err = validate_extract_order("T", &["heading", "extract", "number"])
            .expect_err("misordered chain");
        assert!(
            err.contains("[T] plugin `extract` comes before `number`"),
            "{err}"
        );
    }

    fn extract_heading_chain() -> (ChainConfig, ChainState) {
        let mut config = ChainConfig::new(
            "T",
            Some("heading,extract"),
            "solid",
            Default::default(),
            Default::default(),
        )
        .expect("valid chain");
        let block = ExtractBlock::build(
            BlockSpec {
                element: "heading".to_string(),
                output: vec!["out/h.json".to_string()],
                format: vec!["json".to_string()],
                ..BlockSpec::default()
            },
            0,
        )
        .expect("valid block");
        config.extract = ExtractOptions {
            blocks: vec![block],
            ..ExtractOptions::default()
        };
        let context =
            crate::plugins::DocumentContext::new(pendon_plugin_cite::CitationContext::new(
                serde_json::Value::Object(serde_json::Map::new()),
                Default::default(),
            ));
        let state = ChainState::new(context, std::collections::HashMap::new(), config.pipeline());
        (config, state)
    }

    /// T4.4: extraction writes nothing for a `Payload` document — the blocks
    /// describe the page, not a bound payload (only a `Page` writes
    /// document-level output).
    #[test]
    fn a_payload_document_extracts_nothing() {
        let (config, mut state) = extract_heading_chain();
        let events = pendon_core::parse(
            "## Intro
",
            &pendon_core::Options::default(),
        );

        let _ = run_chain(events.clone(), &config, &mut state, Document::Payload);
        assert!(
            state.extract.is_empty(),
            "a payload wrote {:?}",
            state.extract
        );

        let _ = run_chain(events, &config, &mut state, Document::Page);
        assert_eq!(state.extract.files.len(), 1, "the page still extracts");
    }
}
