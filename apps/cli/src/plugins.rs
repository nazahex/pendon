use pendon_core::{ContextPipeline, Event};
use pendon_extra::SolidImportSpec;
use pendon_plugin_anchor::{AnchorCustomNode, AnchorImport, AnchorOptions};
use pendon_plugin_cite::{CitationContext, CiteCustomNode, CiteImport, CiteOptions, CiteSection};
use pendon_plugin_custom::{load_index_from_path, load_spec_from_path, PluginSpec};
use pendon_plugin_img::ImgOptions;
use pendon_renderer_solid::{ComponentTemplate, ImportEntry, SolidRenderHints};
use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::config::{
    AnchorTaskConfig, CiteTaskConfig, PluginCustomSection, PluginVicadoSection,
    TableComponentConfig, TableTaskConfig,
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
        let events = citation.replace_section_markers(events);
        drop(citation);

        let mut events = events;
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
        "sectionize" => pendon_plugin_sectionize::process(events),
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
    let custom_node = config.custom_node.as_ref().map(|node| CiteCustomNode {
        name: node.name.clone().unwrap_or_else(|| "Citation".to_string()),
        template: node.template.clone().unwrap_or_else(|| {
            "<Citation index={attrs.index} id={attrs.id} loc={attrs.loc} />".to_string()
        }),
        imports: node
            .imports
            .as_deref()
            .map(parse_cite_imports)
            .unwrap_or_default(),
    });
    let section = config.section.as_ref().map(|section| {
        let name = section
            .node
            .clone()
            .unwrap_or_else(|| "CitationSection".to_string());
        CiteSection {
            marker: section
                .marker
                .clone()
                .unwrap_or_else(|| "{{ footnote }}".to_string()),
            name: name.clone(),
            template: section.template.clone().unwrap_or_else(|| {
                format!(
                    "<{name} cites={{frontmatter.cites}} references={{frontmatter.references}} />"
                )
            }),
            imports: section
                .imports
                .as_deref()
                .map(parse_cite_imports)
                .unwrap_or_default(),
        }
    });
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
        custom_node,
        section,
        external_references,
    })
}

pub fn build_anchor_options(config: Option<&AnchorTaskConfig>) -> AnchorOptions {
    let custom_node = config
        .and_then(|config| config.custom_node.as_ref())
        .map(|node| AnchorCustomNode {
            name: node.name.clone().unwrap_or_else(|| "Anchor".to_string()),
            template: node
                .template
                .clone()
                .unwrap_or_else(|| "<Anchor href=\"{attrs.href}\">{children}</Anchor>".to_string()),
            imports: node
                .imports
                .as_deref()
                .map(parse_anchor_imports)
                .unwrap_or_default(),
        });
    AnchorOptions { custom_node }
}

fn parse_anchor_imports(values: &[toml::Value]) -> Vec<AnchorImport> {
    parse_solid_imports(values)
        .into_iter()
        .map(|import| AnchorImport {
            module: import.module,
            default: import.default,
            names: import.names,
        })
        .collect()
}

fn parse_cite_imports(values: &[toml::Value]) -> Vec<CiteImport> {
    parse_solid_imports(values)
        .into_iter()
        .map(|import| CiteImport {
            module: import.module,
            default: import.default,
            names: import.names,
        })
        .collect()
}

pub fn build_table_options(config: Option<&TableTaskConfig>) -> pendon_plugin_table::TableOptions {
    let Some(config) = config else {
        return pendon_plugin_table::TableOptions::default();
    };

    let custom_node =
        config
            .custom_node
            .as_ref()
            .map(|node| pendon_plugin_table::TableCustomNode {
                table: build_table_component(node.table.as_ref()),
                caption: build_table_component(node.caption.as_ref()),
                row: build_table_component(node.row.as_ref()),
                cell: build_table_component(node.cell.as_ref()),
            });

    pendon_plugin_table::TableOptions { custom_node }
}

fn build_table_component(
    cfg: Option<&TableComponentConfig>,
) -> Option<pendon_plugin_table::CustomComponent> {
    let Some(c) = cfg else {
        return None;
    };
    Some(pendon_plugin_table::CustomComponent {
        name: c.name.clone().unwrap_or_default(),
        template: c.template.clone().unwrap_or_default(),
        imports: c
            .imports
            .as_deref()
            .map(parse_table_imports)
            .unwrap_or_default(),
    })
}

fn parse_table_imports(values: &[toml::Value]) -> Vec<pendon_plugin_table::CustomImport> {
    parse_solid_imports(values)
        .into_iter()
        .map(|import| pendon_plugin_table::CustomImport {
            module: import.module,
            default: import.default,
            names: import.names,
        })
        .collect()
}

fn parse_solid_imports(values: &[toml::Value]) -> Vec<SolidImportSpec> {
    values
        .iter()
        .filter_map(|value| {
            let table = value.as_table()?;
            let module = table.get("module")?.as_str()?.to_string();
            let default = table
                .get("default")
                .and_then(toml::Value::as_str)
                .map(str::to_string);
            let names = table
                .get("names")
                .and_then(toml::Value::as_array)
                .map(|values| {
                    values
                        .iter()
                        .filter_map(|value| value.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default();
            Some(SolidImportSpec {
                module,
                default,
                names,
            })
        })
        .collect()
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

fn parse_import_entries(imports: &[toml::Value]) -> Vec<ImportEntry> {
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
