use pendon_core::{ContextPipeline, Event};
use pendon_plugin_anchor::{AnchorCustomNode, AnchorOptions};
use pendon_plugin_blockquote::{BlockquoteCustomNode, BlockquoteOptions};
use pendon_plugin_cite::{CitationContext, CiteCustomNode, CiteOptions, ReferencesScope};
use pendon_plugin_custom::{load_index_from_path, load_spec_from_path, PluginSpec};
use pendon_plugin_directive::{DirectiveCustomNode, DirectiveOptions};
use pendon_plugin_extract::{BlockSpec, ExtractBlock, ExtractOptions};
use pendon_plugin_heading::{HeadingCustomNode, HeadingOptions};
use pendon_plugin_image::{ImageCustomNode, ImageOptions};
use pendon_plugin_list::{ListCustomNode, ListOptions};
use pendon_plugin_marker::{MarkerCustomNode, MarkerOptions};
use pendon_plugin_number::{NumberBlock, NumberOptions, NumberStyles};
use pendon_plugin_section::{SectionCustomNode, SectionOptions};
use pendon_renderer_solid::{
    ComponentSet, ComponentTemplate, ConfigEntry, DataExport, ImportEntry, SolidRenderHints,
    TypedComponent,
};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::config::{
    AnchorTaskConfig, BindTaskConfig, BlockquoteTaskConfig, CiteTaskConfig, ConfigTask,
    ExtractRule, HeadingTaskConfig, ImageTaskConfig, ListTaskConfig, NumberRule, OneOrMany,
    PluginCustomSection, SectionTaskConfig, TableTaskConfig,
};
use crate::utils::{substitute_output, validate_export_identifier};
use pendon_types::Selector;

#[derive(Clone)]
pub struct DocumentContext {
    citation: std::sync::Arc<std::sync::Mutex<CitationContext>>,
}

impl DocumentContext {
    pub fn new(citation: CitationContext) -> Self {
        Self {
            citation: std::sync::Arc::new(std::sync::Mutex::new(citation)),
        }
    }

    pub fn process_citations(&self, events: &[Event]) -> Vec<Event> {
        self.citation.lock().unwrap().process_events(events)
    }

    /// Finishes the cite layer: drains diagnostics and hands back the two data
    /// exports (citations, references) plus the resolved options so the caller
    /// can build `SolidRenderHints`. Frontmatter is left untouched —
    /// `cites`/`references` live in dedicated JS exports (§2.4). Which
    /// references the second export holds follows `references_scope` (§2.6).
    pub fn finalize_citations(&self) -> (Vec<Event>, Value, Value, CiteOptions) {
        let mut citation = self.citation.lock().unwrap();
        let citations = citation.get_citations();
        let references = citation.get_exported_references();
        let options = citation.options().clone();
        let diagnostics = citation.drain_diagnostics();
        (diagnostics, citations, references, options)
    }
}

pub fn build_context_inline_pipeline(
    image_options: ImageOptions,
    wiki_options: pendon_plugin_wiki::WikiOptions,
    anchor_options: AnchorOptions,
    latex_options: Option<pendon_plugin_latex::LatexOptions>,
    enabled_plugins: &HashSet<&str>,
) -> ContextPipeline<DocumentContext> {
    let mut pipeline = ContextPipeline::new();

    if enabled_plugins.contains("cite") {
        pipeline.add(|context: &mut DocumentContext, events: Vec<Event>| {
            context.process_citations(&events)
        });
    }
    if enabled_plugins.contains("wiki") {
        let options = wiki_options.clone();
        pipeline.add(move |_: &mut DocumentContext, events: Vec<Event>| {
            pendon_plugin_wiki::process_with_options(&events, options.clone())
        });
    }
    if enabled_plugins.contains("anchor") {
        let options = anchor_options.clone();
        pipeline.add(move |_: &mut DocumentContext, events: Vec<Event>| {
            pendon_plugin_anchor::process(&events, &options)
        });
    }
    if let Some(options) = latex_options {
        pipeline.add_after_markdown(move |_: &mut DocumentContext, events: Vec<Event>| {
            pendon_plugin_latex::process_with_options(&events, &options)
        });
    }

    if enabled_plugins.contains("image") {
        // §7.1 has no children any more: the caption that used to run through an
        // inline pipeline is retired (§14), so the image reads no pipeline — the
        // step stays here so an image inside a table cell is still claimed.
        let options = image_options.clone();
        pipeline.add(move |_: &mut DocumentContext, events: Vec<Event>| {
            pendon_plugin_image::process(&events, &options)
        });
    }
    if enabled_plugins.contains("cite") {
        pipeline.add(|context: &mut DocumentContext, events: Vec<Event>| {
            context.process_citations(&events)
        });
    }
    if enabled_plugins.contains("wiki") {
        let options = wiki_options.clone();
        pipeline.add(move |_: &mut DocumentContext, events: Vec<Event>| {
            pendon_plugin_wiki::process_with_options(&events, options.clone())
        });
    }
    if enabled_plugins.contains("anchor") {
        pipeline.add(move |_: &mut DocumentContext, events: Vec<Event>| {
            pendon_plugin_anchor::process(&events, &anchor_options)
        });
    }
    if let Some(options) = latex_options {
        pipeline.add_after_markdown(move |_: &mut DocumentContext, events: Vec<Event>| {
            pendon_plugin_latex::process_with_options(&events, &options)
        });
    }

    pipeline
}

pub fn process_stateless_plugin(
    name: &str,
    events: &[Event],
    wiki_options: &pendon_plugin_wiki::WikiOptions,
    latex_options: Option<&pendon_plugin_latex::LatexOptions>,
) -> Option<Vec<Event>> {
    Some(match name {
        "dialog" => pendon_plugin_dialog::process_with_options(
            events,
            &pendon_plugin_dialog::DialogOptions {
                latex: latex_options.copied(),
            },
        ),
        "latex" => match latex_options {
            Some(options) => pendon_plugin_latex::process_with_options(events, options),
            None => events.to_vec(),
        },
        "wiki" => pendon_plugin_wiki::process_with_options(events, wiki_options.clone()),
        "syntect" => pendon_plugin_codeblock::process(events),
        _ => return None,
    })
}

/// Maps a task output format to the math rendering target. Only `solid` needs
/// the JSX `innerHTML={...}` form; every other format gets plain HTML.
pub fn latex_target_for_format(format: &str) -> pendon_plugin_latex::LatexTarget {
    if format == "solid" {
        pendon_plugin_latex::LatexTarget::Solid
    } else {
        pendon_plugin_latex::LatexTarget::Html
    }
}

pub fn plugin_names(plugins: Option<&str>) -> Vec<&str> {
    plugins
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .collect()
}

pub fn has_plugin(plugins: Option<&str>, name: &str) -> bool {
    plugin_names(plugins).contains(&name)
}

pub fn load_custom_spec(
    path: &str,
    cache: &mut HashMap<String, PluginSpec>,
) -> Result<PluginSpec, String> {
    if let Some(spec) = cache.get(path) {
        return Ok(spec.clone());
    }

    let spec = load_spec_from_path(path)?;
    cache.insert(path.to_string(), spec.clone());
    Ok(spec)
}

/// §11 primary layer of `plugin-image`: the `<img>` element. A bare
/// §11 primary layer of `plugin-image` (§7.1): the `<img>` element.
const IMG_LAYER: &str = "img";
/// §11 primary layer of `plugin-heading`: the `<h*>` element.
const HEADING_LAYER: &str = "heading";
/// §11 primary layer of `plugin-cite`: the citation leaf.
const CITE_LAYER: &str = "cite";
/// Default template for the citation component: a self-closing leaf, so §11
/// rule 6 does not require a `{children}` token.
///
/// The reference *object* is bound from the module-level reference export, so
/// the binding follows a renamed `references_export` (§2.3 of RFC-0002); the
/// remaining props ride the `{...attrs}` spread, which keeps them typed.
fn cite_default_template(references_export: &str) -> String {
    format!(r#"<Citation reference={{{references_export}["{{attrs.ref_id}}"]}} {{...attrs}} />"#)
}
/// §11 primary layer of `plugin-marker`: the node `{{type}}` renders.
const MARKER_LAYER: &str = "marker";

/// Loads `task.<plugin>.custom` (§11) once per task, printing the loader's
/// non-fatal deprecation warnings (§11 rule 4, §13).
fn load_custom(
    plugin: &str,
    primary_layer: &str,
    custom: Option<&toml::Value>,
) -> Result<Option<crate::components::LoadedComponents>, String> {
    let Some(custom) = custom else {
        return Ok(None);
    };
    let loaded = crate::components::load(plugin, primary_layer, Some(custom))
        .map_err(|error| error.to_string())?;
    for warning in &loaded.warnings {
        eprintln!("Warning: {}", warning);
    }
    Ok(Some(loaded))
}

/// §14: `[task.<plugin>.custom_node]` was replaced by the §11 `custom.<layer>`
/// form. The removed key is still parsed so a stale config fails with a
/// migration message instead of silently losing the custom component.
fn check_custom_node_removed(plugin: &str, legacy: bool) -> Result<(), String> {
    if legacy {
        return Err(format!(
            "task.{plugin}.custom_node was removed; use task.{plugin}.custom.<layer> (§11, §14)"
        ));
    }
    Ok(())
}

/// Loads one §11 layer as a component set: every entry of the layer with its
/// `type` markers (§11 rule 3). An entry that names no component is carried as
/// a config-only entry (D-8), so its `field.*` and `bracket.modifier."…"` still
/// reach the element — a modifier never needs a custom component.
fn layer_set<C>(
    loaded: Option<&crate::components::LoadedComponents>,
    layer: &str,
    make: impl Fn(&crate::components::ComponentEntry) -> Result<C, String>,
) -> Result<ComponentSet<C>, String> {
    let Some(set) = loaded.and_then(|loaded| loaded.components.layer(layer)) else {
        return Ok(ComponentSet::new());
    };
    let mut entries = Vec::new();
    let mut configs = Vec::new();
    for entry in set.typed().iter().chain(set.default_entry()) {
        if entry.is_configured() {
            entries.push(TypedComponent {
                selectors: entry.selectors.clone(),
                component: make(entry)?,
                field: entry.field.clone(),
                modifier: entry.modifier.clone(),
            });
        } else if entry.carries_config() {
            // D-8: an entry that names no component still configures the
            // element — `field.*` and `bracket.modifier."…"` reach the layer the
            // element renders with, so a modifier needs no custom component.
            configs.push(ConfigEntry {
                selectors: entry.selectors.clone(),
                field: entry.field.clone(),
                modifier: entry.modifier.clone(),
            });
        }
    }
    Ok(ComponentSet::from_entries(entries).with_configs(configs))
}

/// Rejects §11 layers this plugin cannot carry yet (§14: no silent drops).
fn check_layers(
    plugin: &str,
    loaded: Option<&crate::components::LoadedComponents>,
    wired: &[&str],
) -> Result<(), String> {
    let Some(loaded) = loaded else {
        return Ok(());
    };
    crate::components::reject_unwired_layers(plugin, &loaded.components, wired)
        .map_err(|error| error.to_string())
}

pub fn build_cite_options(
    config: Option<&CiteTaskConfig>,
    input_pattern: &str,
    input_path: &str,
    captures: Option<&HashMap<String, String>>,
) -> Result<CiteOptions, String> {
    let Some(config) = config else {
        return Ok(CiteOptions::default());
    };
    let external_references = match config.reference_source.as_deref().unwrap_or("frontmatter") {
        "frontmatter" | "internal" => None,
        "external" => {
            let file = config.reference_file.as_deref().ok_or_else(|| {
                "cite.reference_file is required for external references".to_string()
            })?;
            let captures = captures
                .ok_or_else(|| format!("cannot resolve cite.reference_file '{}'", input_pattern))?;
            let mut values = captures.clone();
            if let Some(slug) = values.get("slug").cloned() {
                if let Some(chapter) = slug.split('/').next().filter(|part| !part.is_empty()) {
                    values.insert("chapter_id".to_string(), chapter.to_string());
                }
            }
            let path = substitute_output(file, &values).map_err(|error| {
                format!(
                    "cannot resolve cite.reference_file '{}' for '{}': {}",
                    file, input_path, error
                )
            })?;
            Some(pendon_plugin_cite::load_references(Path::new(&path))?)
        }
        source => {
            return Err(format!(
            "unsupported cite.reference_source '{}'; expected frontmatter, internal, or external",
            source
        ))
        }
    };
    check_custom_node_removed("cite", config.custom_node.is_some())?;
    // §2.6 of RFC-0002: only `cited` and `all` exist. Validate here so a typo
    // fails loudly instead of silently falling back to the source default.
    let references_scope = match config.references_scope.as_deref() {
        None => None,
        Some(value) => Some(ReferencesScope::parse(value).ok_or_else(|| {
            format!(
                "unsupported cite.references_scope '{value}'; expected {} or {}",
                ReferencesScope::Cited.as_str(),
                ReferencesScope::All.as_str()
            )
        })?),
    };
    // The export names are resolved before the component set: the fallback
    // citation template binds its `reference` prop to the reference export, so
    // it has to know the final name (§2.3 of RFC-0002).
    let defaults = CiteOptions::default();
    let references_export = validate_export_identifier(
        "cite",
        "references_export",
        config
            .references_export
            .as_deref()
            .unwrap_or(&defaults.references_export),
    )?;
    let citations_export = validate_export_identifier(
        "cite",
        "citations_export",
        config
            .citations_export
            .as_deref()
            .unwrap_or(&defaults.citations_export),
    )?;
    let default_template = cite_default_template(&references_export);
    let loaded = load_custom("cite", CITE_LAYER, config.custom.as_ref())?;
    check_layers("cite", loaded.as_ref(), &[CITE_LAYER])?;
    let custom = layer_set(loaded.as_ref(), CITE_LAYER, |entry| {
        Ok(CiteCustomNode {
            name: entry.name.clone().unwrap_or_else(|| "Citation".to_string()),
            template: entry
                .template
                .clone()
                .unwrap_or_else(|| default_template.clone()),
            imports: entry.imports.clone(),
        })
    })?;
    Ok(CiteOptions {
        references_export,
        citations_export,
        references_scope,
        custom,
        external_references,
    })
}

/// `[[task.bind.inject]]` (§6.1): every entry becomes one module-level
/// `export const`.
///
/// The source is resolved like `input`/`output` — relative to `pendon.toml`,
/// with the task's dynamic segments (`[foo]`, `[bar]`, …) substituted per page —
/// and its parsed value reuses the `{{{lang[var](path)}}}` converter, so an
/// injected `.yaml` and a `.yaml` block carry the same JSON shape. Every
/// resolved path is pushed onto `deps`, so editing a source re-renders the page
/// (the same rule a `(path)` block follows).
pub fn build_bind_injects(
    config: Option<&BindTaskConfig>,
    vars: &HashMap<String, String>,
    deps: &mut Vec<String>,
) -> Result<Vec<DataExport>, String> {
    let Some(config) = config else {
        return Ok(Vec::new());
    };
    let mut exports = Vec::with_capacity(config.inject.len());
    for entry in &config.inject {
        let source = substitute_output(&entry.source, vars)
            .map_err(|cause| format!("task.bind.inject source '{}': {cause}", entry.source))?;
        let name = entry
            .export
            .clone()
            .unwrap_or_else(|| default_export_name(&source));
        let name = validate_export_identifier("bind", "inject", &name)?;
        let value = pendon_plugin_bind::parse_data_file(Path::new(&source))
            .map_err(|cause| format!("task.bind.inject '{source}': {cause}"))?;
        deps.push(source);
        exports.push(DataExport::plugin("bind", name, value));
    }
    Ok(exports)
}

/// The name an inject entry falls back to when `export` is absent: its resolved
/// file's stem — so `./data/settings.toml` exports `settings` and
/// `../../dataX.toml` exports `dataX`. A stem that is not a valid identifier
/// (`2024.toml`) is reported by `validate_export_identifier`, which names the
/// value, rather than being silently rewritten.
fn default_export_name(source: &str) -> String {
    Path::new(source)
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_else(|| source.to_string())
}

pub fn build_anchor_options(config: Option<&AnchorTaskConfig>) -> Result<AnchorOptions, String> {
    let Some(config) = config else {
        return Ok(AnchorOptions::default());
    };
    check_custom_node_removed("anchor", config.custom_node.is_some())?;
    let loaded = load_custom("anchor", "anchor", config.custom.as_ref())?;
    check_layers("anchor", loaded.as_ref(), &["anchor"])?;
    let custom = layer_set(loaded.as_ref(), "anchor", |entry| {
        Ok(AnchorCustomNode {
            name: entry.name.clone().unwrap_or_else(|| "Anchor".to_string()),
            template: entry
                .template
                .clone()
                .unwrap_or_else(|| "<Anchor href=\"{attrs.href}\">{children}</Anchor>".to_string()),
            imports: entry.imports.clone(),
        })
    })?;
    Ok(AnchorOptions { custom })
}

pub fn build_table_options(
    config: Option<&TableTaskConfig>,
) -> Result<pendon_plugin_table::TableOptions, String> {
    let Some(config) = config else {
        return Ok(pendon_plugin_table::TableOptions::default());
    };

    check_custom_node_removed("table", config.custom_node.is_some())?;
    // §11: the plugin owns the list of layers it emits, so the CLI never keeps a
    // second copy of it (and a new layer cannot be silently dropped here).
    let wired = pendon_plugin_table::layers();
    let loaded = load_custom(
        "table",
        pendon_plugin_table::primary_layer(),
        config.custom.as_ref(),
    )?;
    check_layers("table", loaded.as_ref(), &wired)?;

    let pick = |layer: &str| {
        layer_set(loaded.as_ref(), layer, |entry| {
            Ok(pendon_plugin_table::CustomComponent {
                name: entry.name.clone().unwrap_or_default(),
                template: entry.template.clone().unwrap_or_default(),
                imports: entry.imports.clone(),
            })
        })
    };
    let table = pick("table")?;
    let caption = pick("caption")?;
    let thead = pick("thead")?;
    let tbody = pick("tbody")?;
    let tfoot = pick("tfoot")?;
    let row = pick("row")?;
    let cell = pick("cell")?;

    // No layer configured: keep the built-in element path rather than
    // entering the custom emit path with nothing to emit.
    if [&table, &caption, &thead, &tbody, &tfoot, &row, &cell]
        .iter()
        .all(|set| set.is_empty())
    {
        return Ok(pendon_plugin_table::TableOptions::default());
    }

    Ok(pendon_plugin_table::TableOptions {
        custom_node: Some(pendon_plugin_table::TableCustomNode {
            // §11 moves imports onto each entry, so there is no layer-wide
            // list to inherit any more.
            table,
            caption,
            thead,
            tbody,
            tfoot,
            row,
            cell,
        }),
    })
}

pub fn build_image_options(config: Option<&ImageTaskConfig>) -> Result<ImageOptions, String> {
    let Some(config) = config else {
        return Ok(ImageOptions::default());
    };
    check_custom_node_removed("image", config.custom_node.is_some())?;
    let wired = pendon_plugin_image::layers();
    let loaded = load_custom("image", IMG_LAYER, config.custom.as_ref())?;
    check_layers("image", loaded.as_ref(), &wired)?;
    // §7.1: `img` is the only layer there is — the `figure` container is retired
    // with the rest of the marker run (§14), so its block no longer loads.
    let img = layer_set(loaded.as_ref(), IMG_LAYER, |entry| {
        Ok::<_, String>(ImageCustomNode {
            name: require(entry.name.as_ref(), "image.custom.img.name")?,
            template: require(entry.template.as_ref(), "image.custom.img.template")?,
            imports: entry.imports.clone(),
        })
    })?;
    Ok(ImageOptions { img })
}

pub fn build_heading_options(config: Option<&HeadingTaskConfig>) -> Result<HeadingOptions, String> {
    let Some(config) = config else {
        return Ok(HeadingOptions::default());
    };
    check_custom_node_removed("heading", config.custom_node.is_some())?;
    // ADR-0010/§14: the heading plugin no longer numbers anything. A stale key
    // must fail loudly, not render an unnumbered document.
    if config.auto_number.is_some() || config.number_style.is_some() {
        return Err(concat!(
            "task.heading.auto_number / number_style were removed; ",
            "use [[task.heading.number]] with plugin-number (ADR-0010, §14)"
        )
        .to_string());
    }
    let loaded = load_custom("heading", HEADING_LAYER, config.custom.as_ref())?;
    check_layers("heading", loaded.as_ref(), &[HEADING_LAYER])?;
    let custom = layer_set(loaded.as_ref(), HEADING_LAYER, |entry| {
        Ok(HeadingCustomNode {
            name: require(entry.name.as_ref(), "heading.custom.heading.name")?,
            template: require(entry.template.as_ref(), "heading.custom.heading.template")?,
            imports: entry.imports.clone(),
        })
    })?;
    Ok(HeadingOptions {
        custom,
        // §9.5: the CLI flips this when `plugin-section` is enabled.
        section_owns_id: false,
    })
}

/// Reads a required field from a task component config.
fn require(value: Option<&String>, key: &str) -> Result<String, String> {
    value
        .cloned()
        .ok_or_else(|| format!("{key} is required when a custom node is configured"))
}

/// §10.1/§11: the marker layer is addressed by the plugin's own key, so
/// `[[task.marker.custom]]` and `[task.marker.custom.marker]` are equivalent.
pub fn build_marker_options(
    config: Option<&crate::config::MarkerTaskConfig>,
) -> Result<MarkerOptions, String> {
    let Some(config) = config else {
        return Ok(MarkerOptions::default());
    };
    check_custom_node_removed("marker", config.custom_node.is_some())?;
    let loaded = load_custom("marker", MARKER_LAYER, config.custom.as_ref())?;
    check_layers("marker", loaded.as_ref(), &[MARKER_LAYER])?;
    let custom = layer_set(loaded.as_ref(), MARKER_LAYER, |entry| {
        Ok(MarkerCustomNode {
            name: require(entry.name.as_ref(), "marker.custom.marker.name")?,
            template: require(entry.template.as_ref(), "marker.custom.marker.template")?,
            imports: entry.imports.clone(),
        })
    })?;
    Ok(MarkerOptions { custom })
}

/// §10.2/§10.3/§11: the directive layer is addressed by the plugin's own key, so
/// `[[task.directive.custom]]` and `[task.directive.custom.directive]` are
/// equivalent.
pub fn build_directive_options(
    config: Option<&crate::config::DirectiveTaskConfig>,
) -> Result<DirectiveOptions, String> {
    let Some(config) = config else {
        return Ok(DirectiveOptions::default());
    };
    let layer = pendon_plugin_directive::primary_layer();
    check_custom_node_removed("directive", config.custom_node.is_some())?;
    let loaded = load_custom("directive", layer, config.custom.as_ref())?;
    check_layers("directive", loaded.as_ref(), &[layer])?;
    let custom = layer_set(loaded.as_ref(), layer, |entry| {
        Ok(DirectiveCustomNode {
            name: require(entry.name.as_ref(), "directive.custom.directive.name")?,
            template: require(
                entry.template.as_ref(),
                "directive.custom.directive.template",
            )?,
            imports: entry.imports.clone(),
        })
    })?;
    Ok(DirectiveOptions { custom })
}

/// §9.3/§9.4/§11: the list layers. `list` is the primary layer (the `<li>`
/// item), `unordered` / `ordered` the `<ul>` / `<ol>` container layers.
pub fn build_list_options(config: Option<&ListTaskConfig>) -> Result<ListOptions, String> {
    let Some(config) = config else {
        return Ok(ListOptions::default());
    };
    let layers = pendon_plugin_list::layers();
    check_custom_node_removed("list", config.custom_node.is_some())?;
    let loaded = load_custom("list", layers[0], config.custom.as_ref())?;
    check_layers("list", loaded.as_ref(), &layers)?;

    let make = |layer: &'static str| {
        move |entry: &crate::components::ComponentEntry| {
            Ok::<_, String>(ListCustomNode {
                name: require(entry.name.as_ref(), &format!("list.custom.{layer}.name"))?,
                template: require(
                    entry.template.as_ref(),
                    &format!("list.custom.{layer}.template"),
                )?,
                imports: entry.imports.clone(),
            })
        }
    };
    let list = layer_set(loaded.as_ref(), layers[0], make(layers[0]))?;
    let unordered = layer_set(loaded.as_ref(), layers[1], make(layers[1]))?;
    let ordered = layer_set(loaded.as_ref(), layers[2], make(layers[2]))?;
    Ok(ListOptions {
        list,
        unordered,
        ordered,
    })
}

/// §9.2/§11: the blockquote layer. `blockquote` is the plugin's only (primary)
/// layer, so `[[task.blockquote.custom]]` and
/// `[task.blockquote.custom.blockquote]` are equivalent.
pub fn build_blockquote_options(
    config: Option<&BlockquoteTaskConfig>,
) -> Result<BlockquoteOptions, String> {
    let Some(config) = config else {
        return Ok(BlockquoteOptions::default());
    };
    let layer = pendon_plugin_blockquote::primary_layer();
    check_custom_node_removed("blockquote", config.custom_node.is_some())?;
    let loaded = load_custom("blockquote", layer, config.custom.as_ref())?;
    check_layers("blockquote", loaded.as_ref(), &[layer])?;
    let custom = layer_set(loaded.as_ref(), layer, |entry| {
        Ok(BlockquoteCustomNode {
            name: require(entry.name.as_ref(), "blockquote.custom.blockquote.name")?,
            template: require(
                entry.template.as_ref(),
                "blockquote.custom.blockquote.template",
            )?,
            imports: entry.imports.clone(),
        })
    })?;
    Ok(BlockquoteOptions { custom })
}

/// §9.5/§11: the `section` layer. `section` is the plugin's only (primary)
/// layer, so `[[task.section.custom]]` and `[task.section.custom.section]` are
/// equivalent.
pub fn build_section_options(config: Option<&SectionTaskConfig>) -> Result<SectionOptions, String> {
    let Some(config) = config else {
        return Ok(SectionOptions::default());
    };
    let layer = pendon_plugin_section::primary_layer();
    check_custom_node_removed("section", config.custom_node.is_some())?;
    let loaded = load_custom("section", layer, config.custom.as_ref())?;
    check_layers("section", loaded.as_ref(), &[layer])?;
    let custom = layer_set(loaded.as_ref(), layer, |entry| {
        Ok(SectionCustomNode {
            name: require(entry.name.as_ref(), "section.custom.section.name")?,
            template: require(entry.template.as_ref(), "section.custom.section.template")?,
            imports: entry.imports.clone(),
        })
    })?;
    Ok(SectionOptions { section: custom })
}

/// The element plugins whose nodes can be numbered, in the order their blocks
/// are registered. A block's `element` is the plugin token, so
/// `[[task.image.number]]` numbers the `img` layer (§7.1 has one layer).
pub const NUMBER_ELEMENTS: [&str; 10] = [
    "heading",
    "anchor",
    "image",
    "table",
    "marker",
    "directive",
    "list",
    "blockquote",
    "section",
    "cite",
];

/// Every numbering block of a task: one per `[[task.<element>.number]]` entry,
/// then the `[[task.number.custom]]` group (ADR-0010).
///
/// A block is validated here rather than in the plugin so a typo names the
/// config key (`task.heading.number[0] '??'`), exactly like a `custom` selector.
pub fn build_number_options(task: &ConfigTask) -> Result<NumberOptions, String> {
    let mut blocks = Vec::new();
    for element in NUMBER_ELEMENTS {
        let rules: Option<&Vec<NumberRule>> = match element {
            "heading" => task.heading.as_ref().map(|config| &config.number),
            "anchor" => task.anchor.as_ref().map(|config| &config.number),
            "image" => task.image.as_ref().map(|config| &config.number),
            "table" => task.table.as_ref().map(|config| &config.number),
            "marker" => task.marker.as_ref().map(|config| &config.number),
            "directive" => task.directive.as_ref().map(|config| &config.number),
            "list" => task.list.as_ref().map(|config| &config.number),
            "blockquote" => task.blockquote.as_ref().map(|config| &config.number),
            "section" => task.section.as_ref().map(|config| &config.number),
            "cite" => task.cite.as_ref().map(|config| &config.number),
            _ => None,
        };
        let Some(rules) = rules else {
            continue;
        };
        for (index, rule) in rules.iter().enumerate() {
            let key = format!("task.{element}.number[{index}]");
            blocks.push(number_block(Some(element), rule, &key)?);
        }
    }
    if let Some(custom) = task.number.as_ref() {
        for (index, rule) in custom.custom.iter().enumerate() {
            let key = format!("task.number.custom[{index}]");
            blocks.push(number_block(None, rule, &key)?);
        }
    }
    Ok(NumberOptions { blocks })
}

/// One `[[…number]]` entry as a [`NumberBlock`], naming its own key in errors.
fn number_block(
    element: Option<&str>,
    rule: &NumberRule,
    key: &str,
) -> Result<NumberBlock, String> {
    let selectors = match &rule.types {
        Some(types) => {
            let tokens = types.tokens();
            Selector::parse_all(tokens).map_err(|error| format!("{key}.type {error}"))?
        }
        None => Vec::new(),
    };
    let styles = match &rule.style {
        Some(style) => match style {
            OneOrMany::One(system) => NumberStyles::One(system.clone()),
            OneOrMany::Many(systems) => NumberStyles::PerDepth(systems.clone()),
        },
        None => NumberStyles::default(),
    };
    Ok(NumberBlock::new(element, selectors, styles))
}
/// Every element a `[[task.<element>.extract]]` section can name
/// (`docs/spec/FIELDS-INVENTORY.md`).
///
/// The five elements §1 lists as in scope but without a layer tag (`wiki`,
/// `codeblock`, `latex`, `bind`, `frontmatter`) are absent on purpose: naming one
/// is a config error (`build_extract_options`) instead of a block that silently
/// matches nothing.
pub const EXTRACT_ELEMENTS: [&str; 10] = [
    "anchor",
    "blockquote",
    "cite",
    "directive",
    "heading",
    "image",
    "list",
    "marker",
    "section",
    "table",
];

/// §1 of `docs/spec/FIELDS-INVENTORY.md`: in scope, but nothing can be extracted
/// from them yet (no `__layer`, no attributes, or the data already has an
/// export). A block under one of these names is a config *error*, not a silent
/// no-op — the section is unknown to the schema, so it has to be caught here
/// before §13's generic unknown-key warning.
pub const UNEXTRACTABLE_ELEMENTS: [&str; 5] = ["wiki", "codeblock", "latex", "bind", "frontmatter"];

/// Every extraction block of a task, in the order the sections are read.
///
/// Like `build_number_options`, the blocks are validated for *every* task: a
/// typo in `format`, `output` or `type` fails even before `extract` is added to
/// `task.plugin`, so the config error does not depend on the plugin list. The
/// pass itself only runs when `extract` is enabled (`chain.rs`).
pub fn build_extract_options(task: &ConfigTask) -> Result<ExtractOptions, String> {
    // §1 of the inventory: an element that is in scope but cannot be extracted
    // fails loudly, so a block never looks accepted while extracting nothing.
    for element in UNEXTRACTABLE_ELEMENTS {
        let Some(toml::Value::Table(section)) = task.unknown.get(element) else {
            continue;
        };
        if section.contains_key("extract") {
            return Err(format!(
                "task.{element}.extract: `{element}` cannot be extracted yet (§1 of docs/spec/FIELDS-INVENTORY.md)"
            ));
        }
    }
    let mut blocks = Vec::new();
    for element in EXTRACT_ELEMENTS {
        let rules: Option<&Vec<ExtractRule>> = match element {
            "anchor" => task.anchor.as_ref().map(|config| &config.extract),
            "blockquote" => task.blockquote.as_ref().map(|config| &config.extract),
            "cite" => task.cite.as_ref().map(|config| &config.extract),
            "directive" => task.directive.as_ref().map(|config| &config.extract),
            "heading" => task.heading.as_ref().map(|config| &config.extract),
            "image" => task.image.as_ref().map(|config| &config.extract),
            "list" => task.list.as_ref().map(|config| &config.extract),
            "marker" => task.marker.as_ref().map(|config| &config.extract),
            "section" => task.section.as_ref().map(|config| &config.extract),
            "table" => task.table.as_ref().map(|config| &config.extract),
            _ => None,
        };
        let Some(rules) = rules else {
            continue;
        };
        for (index, rule) in rules.iter().enumerate() {
            blocks.push(extract_block(element, rule, index)?);
        }
    }
    Ok(ExtractOptions { blocks })
}

/// One `[[task.<element>.extract]]` entry as a validated [`ExtractBlock`].
///
/// The block owns the `task.<element>.extract[i].<key>` prefix in its errors, so
/// this only validates the one value the crate does not know about: the export
/// name, which has to be a JavaScript identifier (§20).
fn extract_block(element: &str, rule: &ExtractRule, index: usize) -> Result<ExtractBlock, String> {
    let tokens = |value: &Option<OneOrMany>| -> Vec<String> {
        value
            .as_ref()
            .map(|tokens| {
                tokens
                    .tokens()
                    .iter()
                    .map(|token| token.to_string())
                    .collect()
            })
            .unwrap_or_default()
    };
    let export = match &rule.export {
        Some(name) => Some(validate_export_identifier(
            &format!("task.{element}.extract[{index}]"),
            "export",
            name,
        )?),
        None => None,
    };
    ExtractBlock::build(
        BlockSpec {
            element: element.to_string(),
            types: tokens(&rule.types),
            output: tokens(&rule.output),
            format: tokens(&rule.format),
            only: tokens(&rule.only),
            omit: tokens(&rule.omit),
            recommended: rule.recommended,
            export,
        },
        index,
    )
    .map_err(|error| error.to_string())
}

pub fn track_used_spec(list: &mut Vec<PluginSpec>, spec: PluginSpec) {
    if !list.iter().any(|s| s.name == spec.name) {
        list.push(spec);
    }
}

pub fn build_solid_hints(specs: &[PluginSpec]) -> SolidRenderHints {
    let mut hints = SolidRenderHints::default();
    let mut seen_templates: HashMap<(String, Option<String>), usize> = HashMap::new();

    for spec in specs {
        if let Some(renderer) = spec.renderer.as_ref().and_then(|r| r.solid.as_ref()) {
            let node_type = spec
                .ast
                .as_ref()
                .and_then(|a| a.node.clone())
                .unwrap_or_else(|| spec.name.clone());
            let node_name = spec.ast.as_ref().and_then(|a| a.node_name.clone());
            let mut keys: Vec<(String, Option<String>)> =
                vec![(node_type.clone(), node_name.clone())];
            if node_type == "Component" {
                if let Some(name) = node_name.clone() {
                    keys.push((name.clone(), Some(name)));
                }
            }

            let parsed_imports = parse_import_entries(&renderer.imports[..]);

            if let Some(tpl) = &renderer.component_template {
                for key in &keys {
                    match seen_templates.get(key) {
                        Some(&index) => {
                            hints.templates[index]
                                .imports
                                .extend(parsed_imports.clone());
                        }
                        None => {
                            seen_templates.insert(key.clone(), hints.templates.len());
                            hints.templates.push(ComponentTemplate::new_identity(
                                key.0.clone(),
                                key.1.clone(),
                                tpl.clone(),
                                parsed_imports.clone(),
                            ));
                        }
                    }
                }
            } else if spec.matcher.start.is_some() {
                if !parsed_imports.is_empty() {
                    if let Some(marker) = spec.matcher.start.clone() {
                        hints.text_imports.push((marker, parsed_imports));
                    } else {
                        hints.global_imports.extend(parsed_imports);
                    }
                }
            } else {
                hints.global_imports.extend(parsed_imports);
            }
        }
    }

    hints
}

/// The single parser for task-level `imports` arrays. Every entry is either a
/// raw import line (`"import X from 'y'"`) or a structured
/// `{ module, default, names }` table — one syntax for every plugin.
pub(crate) fn parse_import_entries(imports: &[toml::Value]) -> Vec<ImportEntry> {
    let mut parsed_imports: Vec<ImportEntry> = Vec::new();
    for val in imports {
        match val {
            toml::Value::String(s) => parsed_imports.push(ImportEntry::Raw(s.clone())),
            toml::Value::Table(tbl) => {
                if let Some(module) = tbl.get("module").and_then(|v| v.as_str()) {
                    let default = tbl
                        .get("default")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let names = tbl
                        .get("names")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect::<Vec<String>>()
                        })
                        .unwrap_or_default();
                    parsed_imports.push(ImportEntry::Structured {
                        module: module.to_string(),
                        default,
                        names,
                    });
                }
            }
            _ => {}
        }
    }
    parsed_imports
}

pub fn merge_solid_hints(
    custom_specs: &[PluginSpec],
    builtin_hints: &[SolidRenderHints],
) -> Option<SolidRenderHints> {
    let mut merged = SolidRenderHints::default();
    // A `[[plugin]]` spec overrides the built-in component of the same node
    // identity: its template *and* its imports win wholesale, so the built-in
    // one is dropped before merging (imports now live on the template).
    let custom = if custom_specs.is_empty() {
        None
    } else {
        Some(build_solid_hints(custom_specs))
    };
    let overridden: HashSet<(String, Option<String>)> = custom
        .as_ref()
        .map(|custom| {
            custom
                .templates
                .iter()
                .map(|tpl| (tpl.node_type.clone(), tpl.node_name.clone()))
                .collect()
        })
        .unwrap_or_default();
    if let Some(custom) = &custom {
        extend_hints(&mut merged, custom);
    }
    for hint in builtin_hints {
        let mut filtered = hint.clone();
        filtered
            .templates
            .retain(|tpl| !overridden.contains(&(tpl.node_type.clone(), tpl.node_name.clone())));
        extend_hints(&mut merged, &filtered);
    }
    if hints_empty(&merged) {
        None
    } else {
        Some(merged)
    }
}

fn extend_hints(target: &mut SolidRenderHints, extra: &SolidRenderHints) {
    target.global_imports.extend(extra.global_imports.clone());
    target.text_imports.extend(extra.text_imports.clone());

    for tpl in &extra.templates {
        let same = |t: &ComponentTemplate| {
            t.node_type == tpl.node_type
                && t.node_name == tpl.node_name
                && t.template == tpl.template
        };
        if let Some(existing) = target.templates.iter_mut().find(|t| same(t)) {
            existing.imports.extend(tpl.imports.iter().cloned());
        } else {
            target.templates.push(tpl.clone());
        }
    }

    // Data exports (§2.3 of RFC-0002, ADR-0009) are ordinary registrations:
    // they keep the producer's order and the renderer applies "last wins" for a
    // repeated name.
    target.exports.extend(extra.exports.iter().cloned());
}

fn hints_empty(hints: &SolidRenderHints) -> bool {
    hints.global_imports.is_empty()
        && hints.text_imports.is_empty()
        && hints.templates.is_empty()
        && hints.exports.is_empty()
}

pub fn load_custom_registry(
    cfg: Option<&PluginCustomSection>,
) -> Result<HashMap<String, PluginSpec>, String> {
    let mut map: HashMap<String, PluginSpec> = HashMap::new();
    let Some(cfg) = cfg else {
        return Ok(map);
    };
    if let Some(sources) = &cfg.source {
        for src in sources {
            let plugins = load_index_from_path(src)?;
            for plugin in plugins {
                map.insert(plugin.id, plugin.spec);
            }
        }
    }
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pendon_extra::SpecsKey;

    /// The component a task config wires into one layer for a `type` marker: a
    /// literal entry beats the layer default (§11 rule 3). The real decision is
    /// taken once per document by the renderer (ADR-0008 `D-3a`); this helper
    /// only asserts what the config put into the plugin's layer set.
    fn component_for<'a, C>(
        set: &'a pendon_renderer_solid::ComponentSet<C>,
        marker: &str,
    ) -> Option<&'a C> {
        set.entries()
            .iter()
            .find(|entry| entry.matches_type(marker))
            .or_else(|| set.default_entry())
            .map(|entry| &entry.component)
    }

    fn table_options(toml_src: &str) -> pendon_plugin_table::TableOptions {
        let cfg: TableTaskConfig = toml::from_str(toml_src).expect("valid table config");
        build_table_options(Some(&cfg)).expect("valid table options")
    }

    /// §11: raw import lines and structured `{ module, names }` tables are one
    /// and the same list, and each layer owns its own list.
    #[test]
    fn table_options_accept_both_import_syntaxes() {
        let options = table_options(
            r#"
[custom.table]
name = "CustomTable"
template = "<CustomTable>{children}</CustomTable>"
imports = ["import { TableCaption } from '@comp/table';", { module = "@comp/table", default = "CustomTable" }]

[custom.thead]
name = "TableHead"
template = "<TableHead>{children}</TableHead>"

[[custom.thead.imports]]
module = "@comp/table"
names = ["TableHead"]
"#,
        );

        let node = options.custom_node.expect("custom node");
        // §11 rule 4: imports live on the entries, so there is no layer-wide
        // list to inherit any more.

        let table = node.table.default_component().expect("table component");
        assert_eq!(table.name, "CustomTable");
        assert!(
            matches!(&table.imports[0], ImportEntry::Raw(line) if line.contains("TableCaption"))
        );
        assert!(matches!(
            &table.imports[1],
            ImportEntry::Structured { module, default, .. }
                if module == "@comp/table" && default.as_deref() == Some("CustomTable")
        ));

        let thead = node.thead.default_component().expect("thead component");
        assert_eq!(thead.template, "<TableHead>{children}</TableHead>");
        assert!(matches!(
            &thead.imports[0],
            ImportEntry::Structured { names, .. } if names == &vec!["TableHead".to_string()]
        ));

        // Unconfigured layers stay empty and fall back to plain elements.
        assert!(node.caption.is_empty());
        assert!(node.tbody.is_empty());
        assert!(node.tfoot.is_empty());
        assert!(node.row.is_empty());
        assert!(node.cell.is_empty());
    }

    #[test]
    fn table_options_without_custom_node_change_nothing() {
        assert!(build_table_options(None)
            .expect("no config")
            .custom_node
            .is_none());
        let cfg: TableTaskConfig = toml::from_str("").expect("empty table config");
        assert!(build_table_options(Some(&cfg))
            .expect("empty config")
            .custom_node
            .is_none());
    }

    /// `image` and `heading` gained task-level config, so a half-specified
    /// component must be reported instead of emitting a broken import.
    #[test]
    fn img_and_heading_require_name_and_template() {
        let cfg: ImageTaskConfig =
            toml::from_str("[custom.img]\nname = \"Thumb\"\n").expect("valid img config");
        let err = build_image_options(Some(&cfg)).expect_err("template is required");
        assert!(err.contains("image.custom.img.template"), "{err}");

        let cfg: HeadingTaskConfig =
            toml::from_str("[custom.heading]\ntemplate = \"<H>{children}</H>\"\n")
                .expect("valid heading config");
        let err = build_heading_options(Some(&cfg)).expect_err("name is required");
        assert!(err.contains("heading.custom.heading.name"), "{err}");
    }

    #[test]
    fn img_and_heading_share_the_import_syntax() {
        let cfg: ImageTaskConfig = toml::from_str(
            r#"
[custom.img]
name = "Figure"
template = "<Figure>{children}</Figure>"
imports = ["import Figure from '@/components/Figure';"]
"#,
        )
        .expect("valid img config");
        let options = build_image_options(Some(&cfg)).expect("valid img options");
        let node = options.img.default_component().expect("img component");
        assert_eq!(node.name, "Figure");
        assert!(matches!(&node.imports[0], ImportEntry::Raw(line) if line.contains("Figure")));

        let cfg: HeadingTaskConfig = toml::from_str(
            r#"
[custom.heading]
name = "DocHeading"
template = "<DocHeading level={{attrs.level}}>{children}</DocHeading>"

[[custom.heading.imports]]
module = "@/components/DocHeading"
default = "DocHeading"
"#,
        )
        .expect("valid heading config");
        let options = build_heading_options(Some(&cfg)).expect("valid heading options");
        let node = options
            .custom
            .default_component()
            .expect("heading component");
        assert!(matches!(
            &node.imports[0],
            ImportEntry::Structured { module, .. } if module == "@/components/DocHeading"
        ));
    }

    /// §6.3 (S2-3.3): an entry's `field.specs.*` block travels with it into the
    /// plugin's layer set, so `specs_for(marker)` answers the entry that owns the
    /// marker — and the entry's assignment, not the built-in default.
    #[test]
    fn anchor_options_carry_the_entry_specs() {
        let cfg: AnchorTaskConfig = toml::from_str(
            r#"
[[custom]]
type = ["anchorA"]
name = "AnchorDefault"
template = "<AnchorDefault {...attrs}>{children}</AnchorDefault>"
field.specs.tick = "slug"
field.specs.quote = "title"
field.specs.class = "foo.class"
field.specs.id = "bar.id"
field.specs.style = "foo.style"
"#,
        )
        .expect("valid anchor config");
        let options = build_anchor_options(Some(&cfg)).expect("valid anchor options");
        let specs = options.custom.specs_for(Some("anchorA"));
        let paths: Vec<&str> = [
            SpecsKey::Tick,
            SpecsKey::Quote,
            SpecsKey::Class,
            SpecsKey::Id,
            SpecsKey::Style,
        ]
        .into_iter()
        .map(|key| specs.path(key))
        .collect();
        assert_eq!(paths, ["slug", "title", "foo.class", "bar.id", "foo.style"]);
    }

    /// §11: `custom` is the canonical key, `[[custom]]` addresses the primary
    /// layer, and imports live on the entry.
    #[test]
    fn anchor_options_read_the_layered_custom_key() {
        let cfg: AnchorTaskConfig = toml::from_str(
            r#"
[[custom]]
type = ["anchorA", "anchorB"]
name = "AnchorAB"
template = "<AnchorAB {...attrs}>{children}</AnchorAB>"

[[custom.imports]]
module = "@comp/shared/Anchor"
names = ["AnchorAB"]
"#,
        )
        .expect("valid anchor config");

        let options = build_anchor_options(Some(&cfg)).expect("valid anchor options");
        // §11 rule 2: the `type` selectors travel with the entry, so the renderer
        // can resolve the layer once per document (rule 3, ADR-0008 `D-3a`).
        let entry = options.custom.entries().first().expect("anchor entry");
        assert_eq!(
            entry
                .selectors
                .iter()
                .map(|selector| selector.to_string())
                .collect::<Vec<_>>(),
            vec!["anchorA", "anchorB"]
        );
        let node = &entry.component;
        assert_eq!(node.name, "AnchorAB");
        assert_eq!(node.template, "<AnchorAB {...attrs}>{children}</AnchorAB>");
        assert!(matches!(
            &node.imports[0],
            ImportEntry::Structured { module, .. } if module == "@comp/shared/Anchor"
        ));
    }

    /// §11 layered table config: one key per layer, imports per entry, and no
    /// layer-wide import list any more.
    #[test]
    fn table_options_read_the_layers_of_custom() {
        let options = table_options(
            r#"
[custom.table]
name = "CustomTable"
template = "<CustomTable>{children}</CustomTable>"

[custom.thead]
name = "TableHead"
template = "<TableHead>{children}</TableHead>"

[[custom.thead.imports]]
module = "@comp/table"
names = ["TableHead"]
"#,
        );

        let node = options.custom_node.expect("custom node");
        assert_eq!(
            node.table.default_component().expect("table layer").name,
            "CustomTable"
        );
        let thead = node.thead.default_component().expect("thead layer");
        assert_eq!(thead.template, "<TableHead>{children}</TableHead>");
        assert!(matches!(
            &thead.imports[0],
            ImportEntry::Structured { module, names, .. }
                if module == "@comp/table" && names == &vec!["TableHead".to_string()]
        ));
        assert!(node.caption.is_empty());
        assert!(node.cell.is_empty());
    }

    /// §5.4 / D-9 + D-8: a custom entry that names no component is **config**,
    /// not a component — its `bracket.modifier."…"` tables reach the layer the
    /// element renders with, and it adds no template, so the built-in element
    /// still renders.
    #[test]
    fn a_config_only_entry_reaches_the_layer_without_a_template() {
        let cfg: AnchorTaskConfig = toml::from_str(
            r#"
[[custom]]
bracket.modifier."?" = { rel = "nofollow" }
"#,
        )
        .expect("valid anchor config");
        let options = build_anchor_options(Some(&cfg)).expect("valid anchor options");

        assert!(options.custom.is_empty(), "no component, so no template");
        assert!(
            pendon_plugin_anchor::solid_hints(&options).is_none(),
            "a config-only layer must not switch the emit path"
        );
        let modifier = options.custom.modifier_for(None).expect("the config");
        assert_eq!(modifier.for_symbol('?').len(), 1);
        assert!(modifier.for_symbol('!').is_empty());
    }

    /// An empty `custom` says nothing: D-8 makes the single-table form a load
    /// error, and `custom = []` stays a no-op.
    #[test]
    fn an_empty_custom_is_an_error_not_a_no_op() {
        let cfg: TableTaskConfig = toml::from_str("[custom]\n").expect("valid table config");
        let error = build_table_options(Some(&cfg)).expect_err("single `[custom]` table");
        assert!(
            error.contains("must be an array of tables"),
            "unexpected error: {error}"
        );

        let cfg: AnchorTaskConfig = toml::from_str("custom = []\n").expect("valid anchor config");
        assert!(build_anchor_options(Some(&cfg))
            .expect("valid anchor options")
            .custom
            .is_empty());
    }

    /// §11 rules 2/3: typed entries and the layer default all reach the plugin,
    /// so the renderer can resolve them per document (`D-3a`) instead of the CLI
    /// refusing a layer with several entries.
    #[test]
    fn typed_entries_and_the_default_build_a_routable_set() {
        let cfg: AnchorTaskConfig = toml::from_str(
            r#"
[[custom]]
type = ["anchorA", "anchorB"]
name = "AnchorAB"
template = "<AnchorAB {...attrs}>{children}</AnchorAB>"

[[custom]]
name = "AnchorDefault"
template = "<AnchorDefault {...attrs}>{children}</AnchorDefault>"
"#,
        )
        .expect("valid anchor config");
        let options = build_anchor_options(Some(&cfg)).expect("routable layer");

        assert_eq!(options.custom.entries().len(), 2);
        let typed = component_for(&options.custom, "anchorA").expect("typed entry");
        assert_eq!(typed.name, "AnchorAB");
        let unclaimed = component_for(&options.custom, "anchorZ").expect("default entry");
        assert_eq!(unclaimed.name, "AnchorDefault");
    }

    /// Unknown layers are typos and the removed `custom_node` key is a
    /// migration; both are reported (§11 rule 1, §14).
    #[test]
    fn unknown_layers_and_removed_custom_node_are_rejected() {
        let cfg: AnchorTaskConfig =
            toml::from_str("[custom.legend]\nname = \"Legend\"\n").expect("valid anchor config");
        let err = build_anchor_options(Some(&cfg)).expect_err("unknown layer");
        assert!(err.contains("unsupported layer"), "{err}");

        let cfg: AnchorTaskConfig = toml::from_str(
            "[custom]\nname = \"A\"\ntemplate = \"<A>{children}</A>\"\n\n[custom_node]\nname = \"B\"\n",
        )
        .expect("valid anchor config");
        let err = build_anchor_options(Some(&cfg)).expect_err("removed key");
        assert!(err.contains("custom_node was removed"), "{err}");
    }

    /// Every plugin that used to read `[task.<plugin>.custom_node]` fails the
    /// build with a migration message instead of ignoring the table (§14).
    #[test]
    fn every_plugin_rejects_the_removed_custom_node_key() {
        let table: TableTaskConfig =
            toml::from_str("[custom_node.cell]\nname = \"Cell\"\n").expect("valid table config");
        let err = build_table_options(Some(&table)).expect_err("removed key");
        assert!(err.contains("task.table.custom_node was removed"), "{err}");

        let img: ImageTaskConfig =
            toml::from_str("[custom_node]\nname = \"Figure\"\n").expect("valid img config");
        let err = build_image_options(Some(&img)).expect_err("removed key");
        assert!(err.contains("task.image.custom_node was removed"), "{err}");

        let heading: HeadingTaskConfig =
            toml::from_str("[custom_node]\nname = \"Heading\"\n").expect("valid heading config");
        let err = build_heading_options(Some(&heading)).expect_err("removed key");
        assert!(
            err.contains("task.heading.custom_node was removed"),
            "{err}"
        );

        let cite: CiteTaskConfig =
            toml::from_str("[custom_node]\nname = \"Cite\"\n").expect("valid cite config");
        let err = build_cite_options(Some(&cite), "./src/[slug].md", "./src/a.md", None)
            .expect_err("removed key");
        assert!(err.contains("task.cite.custom_node was removed"), "{err}");
    }

    /// ADR-0010/§14: `plugin-heading` no longer numbers anything, so a stale
    /// `auto_number` / `number_style` must fail the build with a message naming
    /// their replacement instead of rendering an unnumbered document.
    #[test]
    fn heading_rejects_the_removed_numbering_keys() {
        for source in ["auto_number = true\n", "number_style = \"nested-number\"\n"] {
            let heading: HeadingTaskConfig = toml::from_str(source).expect("valid heading config");
            let err = build_heading_options(Some(&heading)).expect_err("removed key");
            assert!(
                err.contains("task.heading.auto_number / number_style were removed"),
                "{err}"
            );
            assert!(err.contains("[[task.heading.number]]"), "{err}");
        }
    }

    /// RFC-0002 §2.3: a malformed export name must fail loudly instead of
    /// silently rendering with defaults.
    #[test]
    fn cite_rejects_bad_export_names() {
        let cite: CiteTaskConfig =
            toml::from_str("citations_export = \"cites-list\"\n").expect("valid cite config");
        let err = build_cite_options(Some(&cite), "./src/[slug].md", "./src/a.md", None)
            .expect_err("bad identifier");
        assert!(err.contains("not a valid JavaScript identifier"), "{err}");

        let cite: CiteTaskConfig = toml::from_str(
            "references_export = \"bibliographyData\"\ncitations_export = \"citationList\"\n",
        )
        .expect("valid cite config");
        let options = build_cite_options(Some(&cite), "./src/[slug].md", "./src/a.md", None)
            .expect("valid cite options");
        assert_eq!(options.references_export, "bibliographyData");
        assert_eq!(options.citations_export, "citationList");
    }

    /// §2.6 of RFC-0002: `references_scope` accepts only `cited` / `all`, and
    /// leaving it unset keeps the source-dependent default (resolved by the
    /// plugin, not the CLI).
    #[test]
    fn cite_reads_and_validates_the_references_scope() {
        for (value, expected) in [
            ("cited", ReferencesScope::Cited),
            ("all", ReferencesScope::All),
        ] {
            let cite: CiteTaskConfig = toml::from_str(&format!("references_scope = \"{value}\"\n"))
                .expect("valid cite config");
            let options = build_cite_options(Some(&cite), "./src/[slug].md", "./src/a.md", None)
                .expect("valid cite options");
            assert_eq!(options.references_scope, Some(expected), "{value}");
        }

        let cite: CiteTaskConfig =
            toml::from_str("reference_source = \"internal\"\n").expect("valid cite config");
        let options = build_cite_options(Some(&cite), "./src/[slug].md", "./src/a.md", None)
            .expect("valid cite options");
        assert_eq!(
            options.references_scope, None,
            "unset must keep the source-dependent default"
        );

        let cite: CiteTaskConfig =
            toml::from_str("references_scope = \"everything\"\n").expect("valid cite config");
        let err = build_cite_options(Some(&cite), "./src/[slug].md", "./src/a.md", None)
            .expect_err("unknown scope");
        assert!(
            err.contains("unsupported cite.references_scope 'everything'"),
            "{err}"
        );
        assert!(err.contains("cited") && err.contains("all"), "{err}");
    }

    /// `image` reads its one `img` layer and rejects the retired `figure` layer
    /// as an unknown one (§7.1, §14).
    #[test]
    fn img_reads_the_img_layer() {
        let cfg: ImageTaskConfig = toml::from_str(
            r#"
[custom.img]
name = "Thumb"
template = "<img />"
"#,
        )
        .expect("valid img config");
        let options = build_image_options(Some(&cfg)).expect("valid img options");
        assert_eq!(
            options.img.default_component().expect("img component").name,
            "Thumb"
        );

        let cfg: ImageTaskConfig =
            toml::from_str("[custom.img]\nname = \"Thumb\"\n").expect("valid img config");
        let err = build_image_options(Some(&cfg)).expect_err("template required");
        assert!(err.contains("image.custom.img.template"), "{err}");

        // The `figure` container is retired with the marker run, so a config that
        // still declares it names a layer no plugin wires (§13: load error).
        let cfg: ImageTaskConfig = toml::from_str(
            r#"
[custom.figure]
name = "Figure"
template = "<Figure>{children}</Figure>"
"#,
        )
        .expect("valid img config");
        let err = build_image_options(Some(&cfg)).expect_err("figure is not a layer");
        assert!(err.contains("figure"), "{err}");
    }

    /// `heading` and `cite` read their primary layer; the citation leaf keeps its
    /// default (self-closing) template when the entry omits one.
    #[test]
    fn heading_and_cite_read_their_primary_layer() {
        let cfg: HeadingTaskConfig = toml::from_str(
            r#"
[custom.heading]
name = "DocHeading"
template = "<DocHeading>{children}</DocHeading>"
"#,
        )
        .expect("valid heading config");
        let options = build_heading_options(Some(&cfg)).expect("valid heading options");
        assert_eq!(
            options
                .custom
                .default_component()
                .expect("heading component")
                .name,
            "DocHeading"
        );

        let cfg: CiteTaskConfig =
            toml::from_str("custom = [{ name = \"Cite\" }]").expect("valid cite config");
        let options = build_cite_options(Some(&cfg), "./src/[slug].md", "./src/a.md", None)
            .expect("valid cite options");
        let node = options.custom.default_component().expect("cite component");
        assert_eq!(node.name, "Cite");
        assert_eq!(node.template, cite_default_template("references"));
    }

    /// §2.3 of RFC-0002: a renamed `references_export` also renames the binding
    /// inside the fallback template, so the rendered JSX never points at an
    /// undefined module binding.
    #[test]
    fn the_default_cite_template_follows_a_renamed_reference_export() {
        let cfg: CiteTaskConfig = toml::from_str(
            "references_export = \"bibliographyData\"\ncustom = [{ name = \"Citation\" }]",
        )
        .expect("valid cite config");
        let options = build_cite_options(Some(&cfg), "./src/[slug].md", "./src/a.md", None)
            .expect("valid cite options");
        let node = options
            .custom
            .default_component()
            .expect("cite component")
            .clone();
        assert_eq!(
            node.template,
            r#"<Citation reference={bibliographyData["{attrs.ref_id}"]} {...attrs} />"#
        );
        assert!(
            !node.template.contains("references["),
            "template = {}",
            node.template
        );
    }

    /// `list` reads its three layers (§9.4) and `blockquote` its single layer
    /// (§9.2); both reject an unwired layer (§14: no silent drops).
    #[test]
    fn list_and_blockquote_read_their_layers() {
        let cfg: ListTaskConfig = toml::from_str(
            r#"
[custom.unordered]
name = "UnorderedBox"
template = "<ul {...attrs}>{children}</ul>"

[[custom.ordered]]
type = ["orderedA"]
name = "OrderedA"
template = "<OrderedA {...attrs}>{children}</OrderedA>"
"#,
        )
        .expect("valid list config");
        let options = build_list_options(Some(&cfg)).expect("valid list options");
        assert_eq!(
            options
                .unordered
                .default_component()
                .expect("unordered component")
                .name,
            "UnorderedBox"
        );
        assert_eq!(
            component_for(&options.ordered, "orderedA")
                .expect("typed ordered component")
                .name,
            "OrderedA"
        );
        assert!(options.list.is_empty());

        let cfg: ListTaskConfig = toml::from_str("[custom.figure]\nname = \"X\"\n").expect("toml");
        let err = build_list_options(Some(&cfg)).expect_err("unsupported layer");
        assert!(err.contains("unsupported layer for `list`"), "{err}");

        let cfg: BlockquoteTaskConfig = toml::from_str(
            r#"
[[custom.blockquote]]
type = ["bqA"]
name = "QuoteA"
template = "<QuoteA {...attrs}>{children}</QuoteA>"
"#,
        )
        .expect("valid blockquote config");
        let options = build_blockquote_options(Some(&cfg)).expect("valid blockquote options");
        assert_eq!(
            component_for(&options.custom, "bqA")
                .expect("typed quote component")
                .name,
            "QuoteA"
        );

        let cfg: BlockquoteTaskConfig =
            toml::from_str("[custom.figure]\nname = \"X\"\n").expect("toml");
        let err = build_blockquote_options(Some(&cfg)).expect_err("unsupported layer");
        assert!(err.contains("unsupported layer for `blockquote`"), "{err}");
    }

    /// §6.1: an inject entry names its binding explicitly, or the file stem names
    /// it, and the payload is parsed by the same converter a `{{{lang[var] …}}}`
    /// block uses. The resolved path is a cache dependency.
    #[test]
    fn bind_injects_register_named_and_stem_exports() {
        let dir = tempfile::tempdir().expect("temp dir");
        std::fs::create_dir_all(dir.path().join("alpha")).expect("dir");
        std::fs::write(dir.path().join("alpha/rows.json"), r#"[{"name":"Alpha"}]"#).expect("rows");
        std::fs::write(dir.path().join("settings.toml"), "count = 2\n").expect("settings");

        // `source` is CWD-relative, so the test uses absolute paths; TOML literal
        // strings keep them out of the escape rules.
        let rows = dir.path().join("alpha/rows.json");
        let settings = dir.path().join("settings.toml");
        let config: BindTaskConfig = toml::from_str(&format!(
            "inject = [\n  {{ source = '{}', export = \"fooData\" }},\n  {{ source = '{}' }},\n]\n",
            rows.display(),
            settings.display()
        ))
        .expect("valid bind config");
        let vars = HashMap::from([("foo".to_string(), "alpha".to_string())]);
        let mut deps = Vec::new();
        let exports =
            build_bind_injects(Some(&config), &vars, &mut deps).expect("valid inject entries");

        assert_eq!(exports.len(), 2);
        assert_eq!(exports[0].name, "fooData");
        assert_eq!(exports[0].value, serde_json::json!([{"name": "Alpha"}]));
        // No `export`: the file stem names the binding.
        assert_eq!(exports[1].name, "settings");
        assert_eq!(exports[1].value, serde_json::json!({ "count": 2 }));
        assert_eq!(
            deps,
            vec![rows.display().to_string(), settings.display().to_string()]
        );

        // A dynamic segment is substituted before the stem is taken.
        let config: BindTaskConfig = toml::from_str(&format!(
            "inject = [{{ source = '{}' }}]\n",
            dir.path().join("[foo]/rows.json").display()
        ))
        .expect("valid bind config");
        let mut deps = Vec::new();
        let exports =
            build_bind_injects(Some(&config), &vars, &mut deps).expect("valid inject entries");
        assert_eq!(exports[0].name, "rows");
        assert_eq!(deps, vec![rows.display().to_string()]);
    }

    /// A missing source, an unknown segment and a bad identifier are task
    /// errors that name the entry — never a silently skipped export.
    #[test]
    fn bind_injects_report_the_offending_entry() {
        let mut deps = Vec::new();
        let config: BindTaskConfig =
            toml::from_str("inject = [{ source = \"./nope.toml\", export = \"data-x\" }]\n")
                .expect("valid bind config");
        let err = build_bind_injects(Some(&config), &HashMap::new(), &mut deps)
            .expect_err("bad identifier");
        assert_eq!(
            err,
            "bind.inject 'data-x' is not a valid JavaScript identifier"
        );

        let config: BindTaskConfig =
            toml::from_str("inject = [{ source = \"./[foo].toml\" }]\n").expect("valid config");
        let err =
            build_bind_injects(Some(&config), &HashMap::new(), &mut deps).expect_err("missing var");
        assert!(
            err.starts_with("task.bind.inject source './[foo].toml'"),
            "{err}"
        );

        let config: BindTaskConfig =
            toml::from_str("inject = [{ source = \"./nope.toml\" }]\n").expect("valid config");
        let err = build_bind_injects(Some(&config), &HashMap::new(), &mut deps)
            .expect_err("unreadable source");
        assert!(err.starts_with("task.bind.inject './nope.toml'"), "{err}");
        assert!(deps.is_empty(), "a failed entry records no dependency");
    }

    /// No `[task.bind]` section is not an error: the task simply has no injects.
    #[test]
    fn bind_injects_are_empty_without_a_section() {
        let mut deps = Vec::new();
        let exports = build_bind_injects(None, &HashMap::new(), &mut deps).expect("no section");
        assert!(exports.is_empty());
        assert!(deps.is_empty());
    }

    /// P.1.a/b/c: the three shapes a `[[task.<element>.extract]]` block uses —
    /// arrays for several destinations, a bare string for one, and `export`
    /// without an `output`.
    fn extract_task(toml_src: &str) -> ConfigTask {
        toml::from_str(&format!(
            "input = \"./src/[slug].mdp\"\noutput = \"./out/[slug].jsx\"\nformat = \"solid\"\n{toml_src}"
        ))
        .expect("valid task config")
    }

    #[test]
    fn extract_blocks_keep_the_authored_order_and_values() {
        let task = extract_task(
            r#"
[[cite.extract]]
type = ["CiteA", "CiteB"]
output = ["./out/[slug].cite.json", "../../project-2/[category].yaml"]
format = ["json", "yaml"]
only = ["citation_number", "instance_id"]

[[cite.extract]]
export = "citationsC"
omit = ["instance_id"]

[[cite.extract]]
output = "./out/[category]-cite-c.yaml"
format = "yaml"
export = "citationsC"
omit = ["instance_id", "extraAttr1"]
"#,
        );
        let options = build_extract_options(&task).expect("valid blocks");
        assert_eq!(options.blocks.len(), 3);

        let first = &options.blocks[0];
        assert_eq!(first.element, "cite");
        assert_eq!(first.selectors.len(), 2);
        assert_eq!(first.outputs.len(), 2);
        assert_eq!(first.outputs[0].0, "./out/[slug].cite.json");
        assert_eq!(first.outputs[1].1, pendon_plugin_extract::Format::Yaml);
        assert_eq!(first.only, ["citation_number", "instance_id"]);
        assert!(first.export.is_none());

        // A block with no `type` covers the whole element and writes no file.
        let second = &options.blocks[1];
        assert!(second.selectors.is_empty());
        assert!(second.outputs.is_empty());
        assert_eq!(second.export.as_deref(), Some("citationsC"));

        // A single `output`/`format` string is one pair, not an error.
        let third = &options.blocks[2];
        assert_eq!(third.outputs.len(), 1);
        assert_eq!(third.destination_count(), 2);
    }

    #[test]
    fn extract_blocks_are_grouped_by_their_element_section() {
        let task = extract_task(
            r#"
[[heading.extract]]
output = "./out/h.json"
format = "json"

[[table.extract]]
output = "./out/t.toml"
format = "toml"
"#,
        );
        let options = build_extract_options(&task).expect("valid blocks");
        let elements: Vec<&str> = options.blocks.iter().map(|block| block.element).collect();
        // The order is the element list's, not the config's.
        assert_eq!(elements, ["heading", "table"]);
    }

    /// The crate owns the `task.<element>.extract[i].<key>` prefix; a bad format
    /// is reported against the key the user wrote, with the block's index.
    #[test]
    fn extract_errors_name_the_key() {
        let task = extract_task(
            r#"
[[cite.extract]]
output = ["./a.json", "./b.json"]
format = "json"
"#,
        );
        let err = build_extract_options(&task).expect_err("mismatched counts");
        assert!(err.starts_with("task.cite.extract[0].format: "), "{err}");
        assert!(err.contains("1 format(s) for 2 output(s)"), "{err}");
    }

    /// `export` goes through the same identifier validator as `bind.inject`
    /// (§20), so an unexportable name fails at config time.
    #[test]
    fn extract_export_names_are_validated() {
        let task = extract_task(
            r#"
[[cite.extract]]
export = "not a name"
"#,
        );
        let err = build_extract_options(&task).expect_err("invalid identifier");
        assert!(
            err.starts_with("task.cite.extract[0].export 'not a name'"),
            "{err}"
        );
    }

    /// No section is not an error: the task simply extracts nothing.
    #[test]
    fn extract_is_empty_without_a_section() {
        let task = extract_task("");
        assert!(build_extract_options(&task).expect("no section").is_empty());
    }

    /// §1 of `docs/spec/FIELDS-INVENTORY.md`: an in-scope element with no
    /// extractable fields fails by name instead of being silently ignored — the
    /// section is unknown to the schema, so it lands in `task.unknown`.
    #[test]
    fn an_unextractable_element_is_rejected_by_name() {
        let task = extract_task(
            r#"
[[wiki.extract]]
output = "./out/[slug].wiki.json"
format = "json"
"#,
        );
        let err = build_extract_options(&task).expect_err("wiki is not extractable");
        assert!(err.starts_with("task.wiki.extract: "), "{err}");
        assert!(err.contains("cannot be extracted yet"), "{err}");
    }

    /// A known-but-empty `[task.wiki]` table is *not* an extraction error: only
    /// an `extract` block under it is.
    #[test]
    fn other_unknown_sections_are_not_extraction_errors() {
        let task = extract_task("[wiki]\nlink_prefix = \"/wiki/\"\n");
        assert!(build_extract_options(&task)
            .expect("not an extract block")
            .is_empty());
    }
}
