use pendon_core::{ContextPipeline, Event};
use pendon_plugin_anchor::{AnchorCustomNode, AnchorOptions};
use pendon_plugin_blockquote::{BlockquoteCustomNode, BlockquoteOptions};
use pendon_plugin_cite::{CitationContext, CiteCustomNode, CiteOptions};
use pendon_plugin_custom::{load_index_from_path, load_spec_from_path, PluginSpec};
use pendon_plugin_directive::{DirectiveCustomNode, DirectiveOptions};
use pendon_plugin_heading::{HeadingCustomNode, HeadingOptions};
use pendon_plugin_img::{ImgCustomNode, ImgOptions};
use pendon_plugin_list::{ListCustomNode, ListOptions};
use pendon_plugin_marker::{MarkerCustomNode, MarkerOptions};
use pendon_plugin_section::{SectionCustomNode, SectionOptions};
use pendon_renderer_solid::{
    ComponentSet, ComponentTemplate, ImportEntry, SolidRenderHints, TypedComponent,
};
use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::config::{
    AnchorTaskConfig, BlockquoteTaskConfig, CiteTaskConfig, HeadingTaskConfig, ImgTaskConfig,
    ListTaskConfig, PluginCustomSection, PluginVicadoSection, SectionTaskConfig, TableTaskConfig,
};
use crate::utils::substitute_output;

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

    pub fn finalize_citations(&self, events: &[Event]) -> (Vec<Event>, Vec<Event>) {
        let mut citation = self.citation.lock().unwrap();
        let cites = citation.get_cites();
        let references = citation.get_used_references();
        let options = citation.options().clone();
        let diagnostics = citation.drain_diagnostics();
        drop(citation);

        let mut events = events.to_vec();
        pendon_plugin_cite::update_frontmatter_in_events(
            &mut events,
            &cites,
            &references,
            &options,
        );
        (events, diagnostics)
    }
}

pub fn build_context_inline_pipeline(
    img_options: ImgOptions,
    wiki_options: pendon_plugin_wiki::WikiOptions,
    anchor_options: AnchorOptions,
    latex_options: Option<pendon_plugin_latex::LatexOptions>,
    enabled_plugins: &HashSet<&str>,
) -> ContextPipeline<DocumentContext> {
    let mut pipeline = ContextPipeline::new();
    let mut image_pipeline = ContextPipeline::new();

    if enabled_plugins.contains("cite") {
        image_pipeline.add(|context: &mut DocumentContext, events: Vec<Event>| {
            context.process_citations(&events)
        });
    }
    if enabled_plugins.contains("wiki") {
        let options = wiki_options.clone();
        image_pipeline.add(move |_: &mut DocumentContext, events: Vec<Event>| {
            pendon_plugin_wiki::process_with_options(&events, options.clone())
        });
    }
    if enabled_plugins.contains("anchor") {
        let options = anchor_options.clone();
        image_pipeline.add(move |_: &mut DocumentContext, events: Vec<Event>| {
            pendon_plugin_anchor::process(&events, &options)
        });
    }
    if let Some(options) = latex_options {
        image_pipeline.add_after_markdown(move |_: &mut DocumentContext, events: Vec<Event>| {
            pendon_plugin_latex::process_with_options(&events, &options)
        });
    }

    if enabled_plugins.contains("img") {
        pipeline.add(move |context: &mut DocumentContext, events: Vec<Event>| {
            pendon_plugin_img::process_with_context(&events, &img_options, &image_pipeline, context)
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
        "extract-heading" => pendon_plugin_extract_heading::process(events),
        "syntect" => pendon_plugin_codeblock_syntect::process(events),
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

pub fn plugin_names<'a>(plugins: Option<&'a str>) -> Vec<&'a str> {
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

/// §11 primary layer of `plugin-anchor`: the `<a>` it emits.
const ANCHOR_LAYER: &str = "anchor";
/// §11 primary layer of `plugin-img`: the `<figure>` container. A bare
/// `[task.img.custom]` table addresses this layer.
const IMG_LAYER: &str = "figure";
/// §11 layers of `plugin-img` (§7.1): the `<img>` element and its `<figure>`
/// container.
const IMG_LAYERS: [&str; 2] = ["img", "figure"];
/// §11 primary layer of `plugin-heading`: the `<h*>` element.
const HEADING_LAYER: &str = "heading";
/// §11 primary layer of `plugin-cite`: the citation leaf.
const CITE_LAYER: &str = "cite";
/// Default template for the citation component: a self-closing leaf, so §11
/// rule 6 does not require a `{children}` token.
const CITE_DEFAULT_TEMPLATE: &str =
    "<Citation index={attrs.index} id={attrs.id} loc={attrs.loc} />";
/// §11 primary layer of `plugin-marker`: the node `{{type}}` renders.
const MARKER_LAYER: &str = "marker";
/// §11 layers of `plugin-table`, one per element it emits.
const TABLE_LAYERS: [&str; 7] = ["table", "caption", "thead", "tbody", "tfoot", "row", "cell"];

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

/// A layer entry with no name, template or imports carries no intent (an empty
/// `[custom]` table, or `custom = []`).
fn entry_is_configured(entry: &crate::components::ComponentEntry) -> bool {
    entry.name.is_some() || entry.template.is_some() || !entry.imports.is_empty()
}

/// Loads one §11 layer as a component set: every entry of the layer with its
/// `type` markers (§11 rule 3). Entries with no name, template or imports are
/// dropped, so an empty `[custom]` table stays a no-op.
fn layer_set<C>(
    loaded: Option<&crate::components::LoadedComponents>,
    layer: &str,
    make: impl Fn(&crate::components::ComponentEntry) -> Result<C, String>,
) -> Result<ComponentSet<C>, String> {
    let Some(set) = loaded.and_then(|loaded| loaded.components.layer(layer)) else {
        return Ok(ComponentSet::new());
    };
    let mut entries = Vec::new();
    for entry in set
        .typed()
        .iter()
        .chain(set.default_entry().into_iter())
        .filter(|entry| entry_is_configured(entry))
    {
        entries.push(TypedComponent {
            types: entry.types.clone(),
            component: make(entry)?,
        });
    }
    Ok(ComponentSet::from_entries(entries))
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
    let loaded = load_custom("cite", CITE_LAYER, config.custom.as_ref())?;
    check_layers("cite", loaded.as_ref(), &[CITE_LAYER])?;
    let custom = layer_set(loaded.as_ref(), CITE_LAYER, |entry| {
        Ok(CiteCustomNode {
            name: entry.name.clone().unwrap_or_else(|| "Citation".to_string()),
            template: entry
                .template
                .clone()
                .unwrap_or_else(|| CITE_DEFAULT_TEMPLATE.to_string()),
            imports: entry.imports.clone(),
        })
    })?;
    Ok(CiteOptions {
        prefix: config
            .prefix
            .clone()
            .unwrap_or_else(|| "citeref-".to_string()),
        class_name: config
            .class
            .clone()
            .unwrap_or_else(|| "cite-ref".to_string()),
        id_prefix: config
            .id_prefix
            .clone()
            .unwrap_or_else(|| "cra-".to_string()),
        custom,
        external_references,
    })
}

pub fn build_anchor_options(config: Option<&AnchorTaskConfig>) -> Result<AnchorOptions, String> {
    let Some(config) = config else {
        return Ok(AnchorOptions::default());
    };
    check_custom_node_removed("anchor", config.custom_node.is_some())?;
    let loaded = load_custom("anchor", ANCHOR_LAYER, config.custom.as_ref())?;
    check_layers("anchor", loaded.as_ref(), &[ANCHOR_LAYER])?;
    let custom = layer_set(loaded.as_ref(), ANCHOR_LAYER, |entry| {
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
    let loaded = load_custom("table", TABLE_LAYERS[0], config.custom.as_ref())?;
    check_layers("table", loaded.as_ref(), &TABLE_LAYERS)?;

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
            imports: Vec::new(),
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

pub fn build_img_options(config: Option<&ImgTaskConfig>) -> Result<ImgOptions, String> {
    let Some(config) = config else {
        return Ok(ImgOptions::default());
    };
    check_custom_node_removed("img", config.custom_node.is_some())?;
    let loaded = load_custom("img", IMG_LAYER, config.custom.as_ref())?;
    check_layers("img", loaded.as_ref(), &IMG_LAYERS)?;
    let make = |layer: &'static str| {
        move |entry: &crate::components::ComponentEntry| {
            Ok::<_, String>(ImgCustomNode {
                name: require(entry.name.as_ref(), &format!("img.custom.{layer}.name"))?,
                template: require(
                    entry.template.as_ref(),
                    &format!("img.custom.{layer}.template"),
                )?,
                imports: entry.imports.clone(),
            })
        }
    };
    let img = layer_set(loaded.as_ref(), "img", make("img"))?;
    let figure = layer_set(loaded.as_ref(), "figure", make("figure"))?;
    Ok(ImgOptions { img, figure })
}

pub fn build_heading_options(config: Option<&HeadingTaskConfig>) -> Result<HeadingOptions, String> {
    let Some(config) = config else {
        return Ok(HeadingOptions::default());
    };
    check_custom_node_removed("heading", config.custom_node.is_some())?;
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
        auto_number: config.auto_number.unwrap_or_default(),
        number_style: config.number_style.clone().unwrap_or_default(),
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
/// `[task.marker.custom]` and `[task.marker.custom.marker]` are equivalent.
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
/// `[task.directive.custom]` and `[task.directive.custom.directive]` are
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
/// layer, so `[task.blockquote.custom]` and
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
/// layer, so `[task.section.custom]` and `[task.section.custom.section]` are
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

pub fn track_used_spec(list: &mut Vec<PluginSpec>, spec: PluginSpec) {
    if !list.iter().any(|s| s.name == spec.name) {
        list.push(spec);
    }
}

pub fn build_solid_hints(specs: &[PluginSpec]) -> SolidRenderHints {
    let mut hints = SolidRenderHints::default();
    let mut seen_templates: HashSet<(String, Option<String>)> = HashSet::new();

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

            let parsed_imports = parse_import_entries(&renderer.imports);

            if let Some(tpl) = &renderer.component_template {
                for key in &keys {
                    if seen_templates.insert(key.clone()) {
                        hints.templates.push(ComponentTemplate {
                            node_type: key.0.clone(),
                            node_name: key.1.clone(),
                            template: tpl.clone(),
                        });
                    }
                    hints
                        .template_imports
                        .entry(key.clone())
                        .or_default()
                        .extend(parsed_imports.clone());
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

pub fn build_vicado_hints_override(cfg: Option<&PluginVicadoSection>) -> Option<SolidRenderHints> {
    let imports = cfg
        .and_then(|c| c.renderer.as_ref())
        .and_then(|r| r.solid.as_ref())
        .and_then(|s| s.imports.as_ref())
        .map(|vals| parse_import_entries(vals))
        .unwrap_or_default();

    if imports.is_empty() {
        return None;
    }

    let mut hints = pendon_plugin_vicado::solid_hints();
    hints
        .template_imports
        .insert(("Vicado".to_string(), Some("Vicado".to_string())), imports);
    Some(hints)
}

pub fn merge_solid_hints(
    custom_specs: &[PluginSpec],
    builtin_hints: &[SolidRenderHints],
) -> Option<SolidRenderHints> {
    let mut merged = SolidRenderHints::default();
    for hint in builtin_hints {
        extend_hints(&mut merged, hint);
    }
    if !custom_specs.is_empty() {
        let custom = build_solid_hints(custom_specs);
        if !custom.template_imports.is_empty() {
            let custom_import_keys: HashSet<(String, Option<String>)> =
                custom.template_imports.keys().cloned().collect();
            merged
                .template_imports
                .retain(|k, _| !custom_import_keys.contains(k));
        }
        extend_hints(&mut merged, &custom);
    }
    if hints_empty(&merged) {
        None
    } else {
        Some(merged)
    }
}

fn extend_hints(target: &mut SolidRenderHints, extra: &SolidRenderHints) {
    target.global_imports.extend(extra.global_imports.clone());
    for (key, imports) in &extra.template_imports {
        target
            .template_imports
            .entry(key.clone())
            .or_default()
            .extend(imports.clone());
    }
    target.text_imports.extend(extra.text_imports.clone());

    for tpl in &extra.templates {
        let key = (&tpl.node_type, &tpl.node_name, &tpl.template);
        if !target
            .templates
            .iter()
            .any(|t| (&t.node_type, &t.node_name, &t.template) == key)
        {
            target.templates.push(tpl.clone());
        }
    }
}

fn hints_empty(hints: &SolidRenderHints) -> bool {
    hints.global_imports.is_empty()
        && hints.template_imports.is_empty()
        && hints.text_imports.is_empty()
        && hints.templates.is_empty()
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
    use pendon_plugin_heading::NumberStyle;

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
        // §11 rule 4: imports moved onto the entries, there is no layer-wide
        // list to inherit any more.
        assert!(node.imports.is_empty());

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

    /// `img` and `heading` gained task-level config, so a half-specified
    /// component must be reported instead of emitting a broken import.
    #[test]
    fn img_and_heading_require_name_and_template() {
        let cfg: ImgTaskConfig =
            toml::from_str("[custom.figure]\nname = \"Figure\"\n").expect("valid img config");
        let err = build_img_options(Some(&cfg)).expect_err("template is required");
        assert!(err.contains("img.custom.figure.template"), "{err}");

        let cfg: HeadingTaskConfig =
            toml::from_str("[custom.heading]\ntemplate = \"<H>{children}</H>\"\n")
                .expect("valid heading config");
        let err = build_heading_options(Some(&cfg)).expect_err("name is required");
        assert!(err.contains("heading.custom.heading.name"), "{err}");
    }

    #[test]
    fn img_and_heading_share_the_import_syntax() {
        let cfg: ImgTaskConfig = toml::from_str(
            r#"
[custom.figure]
name = "Figure"
template = "<Figure>{children}</Figure>"
imports = ["import Figure from '@/components/Figure';"]
"#,
        )
        .expect("valid img config");
        let options = build_img_options(Some(&cfg)).expect("valid img options");
        let node = options
            .figure
            .default_component()
            .expect("figure component");
        assert_eq!(node.name, "Figure");
        assert!(matches!(&node.imports[0], ImportEntry::Raw(line) if line.contains("Figure")));

        let cfg: HeadingTaskConfig = toml::from_str(
            r#"
auto_number = true

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
        assert!(options.auto_number);
        assert_eq!(options.number_style, NumberStyle::NestedNumber);
        let node = options
            .custom
            .default_component()
            .expect("heading component");
        assert!(matches!(
            &node.imports[0],
            ImportEntry::Structured { module, .. } if module == "@/components/DocHeading"
        ));
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
        // §11 rule 2: the `type` markers travel with the entry, so the plugin can
        // route per instance (§11 rule 3, OPEN-C3).
        let entry = options.custom.entries().first().expect("anchor entry");
        assert_eq!(entry.types, vec!["anchorA", "anchorB"]);
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
        assert!(node.imports.is_empty());
        assert!(node.caption.is_empty());
        assert!(node.cell.is_empty());
    }

    /// A `custom` key with nothing in it keeps the built-in element path.
    #[test]
    fn empty_custom_key_changes_nothing() {
        assert!(table_options("[custom]\n").custom_node.is_none());

        let cfg: AnchorTaskConfig = toml::from_str("custom = []\n").expect("valid anchor config");
        assert!(build_anchor_options(Some(&cfg))
            .expect("valid anchor options")
            .custom
            .is_empty());
    }

    /// §11 rules 2/3: typed entries and the layer default all reach the plugin,
    /// which routes per instance (OPEN-C3) instead of the CLI refusing them.
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
        let typed = options.custom.select(Some("anchorA")).expect("typed entry");
        assert_eq!(typed.name, "AnchorAB");
        let unclaimed = options
            .custom
            .select(Some("anchorZ"))
            .expect("default entry");
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

        let img: ImgTaskConfig =
            toml::from_str("[custom_node]\nname = \"Figure\"\n").expect("valid img config");
        let err = build_img_options(Some(&img)).expect_err("removed key");
        assert!(err.contains("task.img.custom_node was removed"), "{err}");

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

    /// `img` reads the `figure` layer (the outermost node) and the `img` layer
    /// (the element it contains), and reports unknown layers.
    #[test]
    fn img_reads_the_figure_layer() {
        let cfg: ImgTaskConfig = toml::from_str(
            r#"
[custom.figure]
name = "Figure"
template = "<Figure>{children}</Figure>"
"#,
        )
        .expect("valid img config");
        let options = build_img_options(Some(&cfg)).expect("valid img options");
        assert_eq!(
            options
                .figure
                .default_component()
                .expect("figure component")
                .name,
            "Figure"
        );

        let cfg: ImgTaskConfig =
            toml::from_str("[custom.figure]\nname = \"Figure\"\n").expect("valid img config");
        let err = build_img_options(Some(&cfg)).expect_err("template required");
        assert!(err.contains("img.custom.figure.template"), "{err}");

        // The `img` layer is wired: it renders the `<img>` element of a figure
        // or the whole component of a plain image (§7.1).
        let cfg: ImgTaskConfig =
            toml::from_str("[custom.img]\nname = \"Thumb\"\ntemplate = \"<img />\"\n")
                .expect("valid img config");
        let options = build_img_options(Some(&cfg)).expect("img layer wired");
        assert_eq!(
            options.img.default_component().expect("img component").name,
            "Thumb"
        );
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
            toml::from_str("[custom]\nname = \"Cite\"\n").expect("valid cite config");
        let options = build_cite_options(Some(&cfg), "./src/[slug].md", "./src/a.md", None)
            .expect("valid cite options");
        let node = options.custom.default_component().expect("cite component");
        assert_eq!(node.name, "Cite");
        assert_eq!(node.template, CITE_DEFAULT_TEMPLATE);
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
            options
                .ordered
                .select(Some("orderedA"))
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
            options
                .custom
                .select(Some("bqA"))
                .expect("typed quote component")
                .name,
            "QuoteA"
        );

        let cfg: BlockquoteTaskConfig =
            toml::from_str("[custom.figure]\nname = \"X\"\n").expect("toml");
        let err = build_blockquote_options(Some(&cfg)).expect_err("unsupported layer");
        assert!(err.contains("unsupported layer for `blockquote`"), "{err}");
    }
}
