use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

use pendon_core::{Event, NodeKind, Severity};
use pendon_renderer_solid::{ComponentTemplate, ImportEntry, SolidRenderHints};
use serde_json::{Map, Value};

#[derive(Clone, Debug, Default)]
pub struct CiteCustomNode {
    pub name: String,
    pub template: String,
    pub imports: Vec<CiteImport>,
}

#[derive(Clone, Debug, Default)]
pub struct CiteSection {
    pub marker: String,
    pub name: String,
    pub template: String,
    pub imports: Vec<CiteImport>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CiteImport {
    pub module: String,
    pub default: Option<String>,
    pub names: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct CiteOptions {
    pub prefix: String,
    pub class_name: String,
    pub id_prefix: String,
    pub custom_node: Option<CiteCustomNode>,
    pub section: Option<CiteSection>,
    pub external_references: Option<Value>,
}

impl Default for CiteOptions {
    fn default() -> Self {
        Self {
            prefix: "citeref-".to_string(),
            class_name: "cite-ref".to_string(),
            id_prefix: "cra-".to_string(),
            custom_node: None,
            section: None,
            external_references: None,
        }
    }
}

pub fn load_references(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path).map_err(|e| {
        format!(
            "cannot read citation reference file '{}': {}",
            path.display(),
            e
        )
    })?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&text).map_err(|e| {
        format!(
            "invalid citation reference YAML '{}': {}",
            path.display(),
            e
        )
    })?;
    yaml_to_json(&yaml)
}

pub fn process(events: &[Event], options: &CiteOptions) -> Vec<Event> {
    let section_events = options
        .section
        .as_ref()
        .map(|section| replace_section_markers(events, section))
        .unwrap_or_else(|| events.to_vec());
    let mut frontmatter = extract_frontmatter(&section_events);
    let front_references = frontmatter
        .as_ref()
        .and_then(|data| data.get("references"))
        .cloned()
        .unwrap_or_else(|| Value::Object(Map::new()));
    let references = merge_references(&front_references, options.external_references.as_ref());

    let mut state = CitationState {
        references,
        options,
        cites: Vec::new(),
        identities: HashMap::new(),
        diagnostics: Vec::new(),
    };
    let mut out = Vec::with_capacity(events.len() + 8);
    let mut excluded = 0usize;
    let mut in_frontmatter = false;

    for event in &section_events {
        match event {
            Event::StartNode(kind) => {
                if matches!(
                    kind,
                    NodeKind::CodeFence
                        | NodeKind::InlineCode
                        | NodeKind::HtmlBlock
                        | NodeKind::HtmlInline
                ) {
                    excluded += 1;
                }
                if *kind == NodeKind::Frontmatter {
                    in_frontmatter = true;
                }
                out.push(event.clone());
            }
            Event::EndNode(kind) => {
                if *kind == NodeKind::Frontmatter {
                    in_frontmatter = false;
                }
                if matches!(
                    kind,
                    NodeKind::CodeFence
                        | NodeKind::InlineCode
                        | NodeKind::HtmlBlock
                        | NodeKind::HtmlInline
                ) {
                    excluded = excluded.saturating_sub(1);
                }
                out.push(event.clone());
            }
            Event::Attribute { name, value } if in_frontmatter && name == "data" => {
                out.push(Event::Attribute {
                    name: name.clone(),
                    value: value.clone(),
                });
            }
            Event::Text(text) if excluded == 0 && !in_frontmatter => {
                emit_text(text, &mut state, &mut out);
            }
            _ => out.push(event.clone()),
        }
    }

    let cites = std::mem::take(&mut state.cites);
    let empty_references = Value::Object(Map::new());
    let mut section_references = options
        .external_references
        .as_ref()
        .map(|external| references_for_cites(&empty_references, external, &cites))
        .unwrap_or_else(|| state.references.clone());
    if let Some(mut data) = frontmatter.take() {
        if let Value::Object(ref mut object) = data {
            let injected_references = options
                .external_references
                .as_ref()
                .map(|external| references_for_cites(&front_references, external, &cites));
            object.insert("cites".to_string(), Value::Array(cites));
            if let Some(references) = injected_references {
                section_references = references.clone();
                object.insert("references".to_string(), references);
            }
            let stored_cites = object
                .get("cites")
                .cloned()
                .unwrap_or(Value::Array(Vec::new()));
            annotate_section_nodes(
                &mut out,
                options.section.as_ref(),
                &stored_cites,
                &section_references,
            );
        }
        replace_frontmatter_data(&mut out, data);
    } else {
        let mut data = Map::new();
        data.insert("cites".to_string(), Value::Array(cites));
        if options.external_references.is_some() {
            data.insert("references".to_string(), section_references.clone());
        }
        let metadata = serde_json::to_string(&Value::Object(data.clone()))
            .unwrap_or_else(|_| "{}".to_string());
        let insert_at = out
            .iter()
            .position(|event| matches!(event, Event::StartNode(NodeKind::Document)))
            .map(|index| index + 1)
            .unwrap_or(0);
        out.splice(
            insert_at..insert_at,
            [
                Event::StartNode(NodeKind::Frontmatter),
                Event::Attribute {
                    name: "data".to_string(),
                    value: metadata,
                },
                Event::EndNode(NodeKind::Frontmatter),
            ],
        );
        let cites = data
            .get("cites")
            .cloned()
            .unwrap_or(Value::Array(Vec::new()));
        annotate_section_nodes(
            &mut out,
            options.section.as_ref(),
            &cites,
            &section_references,
        );
    }

    if !state.diagnostics.is_empty() {
        let insert_at = out
            .iter()
            .position(|event| matches!(event, Event::StartNode(NodeKind::Document)))
            .map(|idx| idx + 1)
            .unwrap_or(0);
        for (offset, diagnostic) in state.diagnostics.into_iter().enumerate() {
            out.insert(insert_at + offset, diagnostic);
        }
    }
    out
}

pub fn solid_hints(options: &CiteOptions) -> Option<SolidRenderHints> {
    let mut hints = SolidRenderHints::default();
    let mut add_node = |name: &str, template: &str, imports: &[CiteImport]| {
        let key = (name.to_string(), Some(name.to_string()));
        hints.templates.push(ComponentTemplate {
            node_type: name.to_string(),
            node_name: Some(name.to_string()),
            template: template.to_string(),
        });
        hints.template_imports.insert(
            key,
            imports
                .iter()
                .map(|import| ImportEntry::Structured {
                    module: import.module.clone(),
                    default: import.default.clone(),
                    names: import.names.clone(),
                })
                .collect(),
        );
    };
    if let Some(custom) = options.custom_node.as_ref() {
        add_node(&custom.name, &custom.template, &custom.imports);
    }
    if let Some(section) = options.section.as_ref() {
        add_node(&section.name, &section.template, &section.imports);
    }
    if hints.templates.is_empty() {
        None
    } else {
        Some(hints)
    }
}

fn replace_section_markers(events: &[Event], section: &CiteSection) -> Vec<Event> {
    let mut out = Vec::with_capacity(events.len());
    let mut i = 0usize;
    while i < events.len() {
        if matches!(events.get(i), Some(Event::StartNode(NodeKind::Paragraph))) {
            let mut end = i + 1;
            let mut text = String::new();
            while end < events.len() && !matches!(events[end], Event::EndNode(NodeKind::Paragraph))
            {
                if let Event::Text(value) = &events[end] {
                    text.push_str(value);
                }
                end += 1;
            }
            if end < events.len() && text.trim() == section.marker {
                out.push(Event::StartNode(NodeKind::Custom(section.name.clone())));
                out.push(Event::Attribute {
                    name: "name".to_string(),
                    value: section.name.clone(),
                });
                out.push(Event::EndNode(NodeKind::Custom(section.name.clone())));
                i = end + 1;
                continue;
            }
        }
        out.push(events[i].clone());
        i += 1;
    }
    out
}

fn annotate_section_nodes(
    events: &mut Vec<Event>,
    section: Option<&CiteSection>,
    cites: &Value,
    references: &Value,
) {
    let Some(section_name) = section.map(|section| section.name.as_str()) else {
        return;
    };
    let cites = serde_json::to_string(cites).unwrap_or_else(|_| "[]".to_string());
    let references = serde_json::to_string(references).unwrap_or_else(|_| "{}".to_string());
    let mut i = 0usize;
    while i < events.len() {
        let is_section_start = match events.get(i) {
            Some(Event::StartNode(NodeKind::Custom(name))) => name == section_name,
            _ => false,
        };
        if is_section_start {
            if let Some(Event::Attribute { name, value }) = events.get(i + 1) {
                if name == "name" && value == section_name {
                    events.insert(
                        i + 2,
                        Event::Attribute {
                            name: "cites".to_string(),
                            value: cites.clone(),
                        },
                    );
                    events.insert(
                        i + 3,
                        Event::Attribute {
                            name: "references".to_string(),
                            value: references.clone(),
                        },
                    );
                    i += 2;
                }
            }
        }
        i += 1;
    }
}

struct CitationState<'a> {
    references: Value,
    options: &'a CiteOptions,
    cites: Vec<Value>,
    identities: HashMap<String, usize>,
    diagnostics: Vec<Event>,
}

fn emit_text(text: &str, state: &mut CitationState<'_>, out: &mut Vec<Event>) {
    let chars: Vec<char> = text.chars().collect();
    let mut cursor = 0usize;
    let mut normal = String::new();
    while cursor < chars.len() {
        if chars[cursor..].starts_with(&['[', '^', '^', ']', '(']) {
            if let Some((end, id, props)) = parse_citation(&chars, cursor) {
                flush_text(&mut normal, out);
                if reference_exists(&state.references, &id) {
                    let mut citation = Map::new();
                    citation.insert("id".to_string(), Value::String(id.clone()));
                    for (key, value) in props {
                        citation.insert(key, Value::String(value));
                    }
                    let identity = canonical_identity(&citation);
                    let index = if let Some(index) = state.identities.get(&identity) {
                        *index
                    } else {
                        let index = state.cites.len() + 1;
                        citation.insert("index".to_string(), Value::Number(index.into()));
                        state.cites.push(Value::Object(citation.clone()));
                        state.identities.insert(identity, index);
                        index
                    };
                    emit_citation(out, state.options, &citation, index);
                } else {
                    let raw: String = chars[cursor..end].iter().collect();
                    normal.push_str(&raw);
                    state.diagnostics.push(Event::Diagnostic {
                        severity: Severity::Error,
                        message: format!("[cite] reference '{}' was not found", id),
                        span: None,
                    });
                }
                cursor = end;
                continue;
            }
        }
        normal.push(chars[cursor]);
        cursor += 1;
    }
    flush_text(&mut normal, out);
}

fn emit_citation(
    out: &mut Vec<Event>,
    options: &CiteOptions,
    citation: &Map<String, Value>,
    index: usize,
) {
    if let Some(custom) = &options.custom_node {
        out.push(Event::StartNode(NodeKind::Custom(custom.name.clone())));
        out.push(Event::Attribute {
            name: "name".to_string(),
            value: custom.name.clone(),
        });
        for (key, value) in citation {
            if key != "index" || value.is_number() {
                out.push(Event::Attribute {
                    name: key.clone(),
                    value: value_to_string(value),
                });
            }
        }
        out.push(Event::EndNode(NodeKind::Custom(custom.name.clone())));
        return;
    }

    let id = citation
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let target = format!("{}{}-{}", options.prefix, index, id);
    let anchor = format!("{}{}", options.id_prefix, index);
    let html = format!(
        "<sup class=\"{}\"><a href=\"#{}\" id=\"{}\">[{}]</a></sup>",
        escape_html(&options.class_name),
        escape_html(&target),
        escape_html(&anchor),
        index
    );
    out.push(Event::StartNode(NodeKind::HtmlInline));
    out.push(Event::Text(html));
    out.push(Event::EndNode(NodeKind::HtmlInline));
}

fn parse_citation(
    chars: &[char],
    start: usize,
) -> Option<(usize, String, BTreeMap<String, String>)> {
    let mut end = start + 5;
    let mut quote = false;
    while end < chars.len() {
        match chars[end] {
            '"' => quote = !quote,
            ')' if !quote => break,
            _ => {}
        }
        end += 1;
    }
    if end >= chars.len() {
        return None;
    }
    let args = split_args(&chars[start + 5..end]);
    if args.is_empty() {
        return None;
    }
    let id = parse_value(args[0].trim())?;
    let mut positional = 0usize;
    let mut props = BTreeMap::new();
    for raw in args.into_iter().skip(1) {
        let part = raw.trim();
        if let Some(eq) = part.find('=') {
            let key = part[..eq].trim();
            if key.is_empty() {
                return None;
            }
            props.insert(key.to_string(), parse_value(part[eq + 1..].trim())?);
        } else {
            if positional == 0 {
                props.insert("loc".to_string(), parse_value(part)?);
                positional += 1;
            } else {
                return None;
            }
        }
    }
    Some((end + 1, id, props))
}

fn split_args(chars: &[char]) -> Vec<String> {
    let mut result = Vec::new();
    let mut start = 0usize;
    let mut quote = false;
    for (index, ch) in chars.iter().enumerate() {
        match ch {
            '"' => quote = !quote,
            ',' if !quote => {
                result.push(chars[start..index].iter().collect());
                start = index + 1;
            }
            _ => {}
        }
    }
    result.push(chars[start..].iter().collect());
    result
}

fn parse_value(value: &str) -> Option<String> {
    let value = value.trim();
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        serde_json::from_str::<String>(value).ok()
    } else if !value.is_empty() {
        Some(value.to_string())
    } else {
        None
    }
}

fn extract_frontmatter(events: &[Event]) -> Option<Value> {
    let mut inside = false;
    for event in events {
        match event {
            Event::StartNode(NodeKind::Frontmatter) => inside = true,
            Event::EndNode(NodeKind::Frontmatter) => inside = false,
            Event::Attribute { name, value } if inside && name == "data" => {
                if let Ok(data) = serde_json::from_str(value) {
                    return Some(data);
                }
            }
            _ => {}
        }
    }
    None
}

fn replace_frontmatter_data(events: &mut [Event], data: Value) {
    let serialized = serde_json::to_string(&data).unwrap_or_else(|_| "{}".to_string());
    let mut inside = false;
    for event in events {
        match event {
            Event::StartNode(NodeKind::Frontmatter) => inside = true,
            Event::EndNode(NodeKind::Frontmatter) => inside = false,
            Event::Attribute { name, value } if inside && name == "data" => {
                *value = serialized.clone()
            }
            _ => {}
        }
    }
}

fn merge_references(front: &Value, external: Option<&Value>) -> Value {
    let mut merged = Map::new();
    if let Some(map) = external.and_then(Value::as_object) {
        for (key, value) in map {
            merged.insert(key.clone(), value.clone());
        }
    }
    if let Some(map) = front.as_object() {
        for (key, value) in map {
            merged.insert(key.clone(), value.clone());
        }
    }
    Value::Object(merged)
}

fn references_for_cites(front: &Value, external: &Value, cites: &[Value]) -> Value {
    let mut merged = front.as_object().cloned().unwrap_or_default();
    let Some(external) = external.as_object() else {
        return Value::Object(merged);
    };
    for cite in cites {
        let Some(id) = cite.get("id").and_then(Value::as_str) else {
            continue;
        };
        if !merged.contains_key(id) {
            if let Some(reference) = external.get(id) {
                merged.insert(id.to_string(), reference.clone());
            }
        }
    }
    Value::Object(merged)
}

fn reference_exists(references: &Value, id: &str) -> bool {
    references
        .as_object()
        .is_some_and(|map| map.contains_key(id))
}

fn canonical_identity(citation: &Map<String, Value>) -> String {
    serde_json::to_string(citation).unwrap_or_default()
}

fn value_to_string(value: &Value) -> String {
    value
        .as_str()
        .map(str::to_string)
        .unwrap_or_else(|| value.to_string())
}

fn flush_text(text: &mut String, out: &mut Vec<Event>) {
    if !text.is_empty() {
        out.push(Event::Text(std::mem::take(text)));
    }
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn yaml_to_json(value: &serde_yaml::Value) -> Result<Value, String> {
    match value {
        serde_yaml::Value::Null => Ok(Value::Null),
        serde_yaml::Value::Bool(value) => Ok(Value::Bool(*value)),
        serde_yaml::Value::Number(value) => serde_json::to_value(value).map_err(|e| e.to_string()),
        serde_yaml::Value::String(value) => Ok(Value::String(value.clone())),
        serde_yaml::Value::Sequence(values) => values
            .iter()
            .map(yaml_to_json)
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array),
        serde_yaml::Value::Mapping(values) => {
            let mut object = Map::new();
            for (key, value) in values {
                let key = match key {
                    serde_yaml::Value::String(key) => key.clone(),
                    _ => return Err("citation reference mapping keys must be strings".to_string()),
                };
                object.insert(key, yaml_to_json(value)?);
            }
            Ok(Value::Object(object))
        }
        serde_yaml::Value::Tagged(tagged) => yaml_to_json(&tagged.value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn events_with_refs() -> Vec<Event> {
        vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Frontmatter),
            Event::Attribute {
                name: "data".into(),
                value: r#"{"references":{"book":{"title":"Book"}}}"#.into(),
            },
            Event::EndNode(NodeKind::Frontmatter),
            Event::StartNode(NodeKind::Paragraph),
            Event::Text(
                r#"A [^^]("book", "p. 1") B [^^]("book", "p. 1") C [^^]("book", "p. 2")."#.into(),
            ),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ]
    }

    #[test]
    fn numbers_duplicate_details_once_and_different_details_separately() {
        let result = process(&events_with_refs(), &CiteOptions::default());
        let data = result
            .iter()
            .find_map(|event| match event {
                Event::Attribute { name, value } if name == "data" => {
                    serde_json::from_str::<Value>(value).ok()
                }
                _ => None,
            })
            .unwrap();
        assert_eq!(data["cites"].as_array().unwrap().len(), 2);
        assert_eq!(data["cites"][0]["index"], 1);
        assert_eq!(data["cites"][1]["index"], 2);
    }

    #[test]
    fn renders_custom_node() {
        let mut options = CiteOptions::default();
        options.custom_node = Some(CiteCustomNode {
            name: "Citation".into(),
            template: "<Citation>{children}</Citation>".into(),
            imports: Vec::new(),
        });
        let result = process(&events_with_refs(), &options);
        assert!(result.iter().any(
            |event| matches!(event, Event::StartNode(NodeKind::Custom(name)) if name == "Citation")
        ));
    }

    #[test]
    fn replaces_section_marker_and_exposes_citation_data() {
        let mut events = events_with_refs();
        let document_end = events.len() - 1;
        events.splice(
            document_end..document_end,
            [
                Event::StartNode(NodeKind::Paragraph),
                Event::Text("{{ footnote }}".into()),
                Event::EndNode(NodeKind::Paragraph),
            ],
        );
        let mut options = CiteOptions::default();
        options.section = Some(CiteSection {
            marker: "{{ footnote }}".into(),
            name: "Bibliography".into(),
            template: "<Bibliography cites={attrs.cites} references={attrs.references} />".into(),
            imports: Vec::new(),
        });
        let result = process(&events, &options);
        assert!(result.iter().any(|event| matches!(
            event,
            Event::StartNode(NodeKind::Custom(name)) if name == "Bibliography"
        )));
        assert!(result.iter().any(|event| matches!(
            event,
            Event::Attribute { name, value } if name == "cites" && value.contains("book")
        )));
    }
}
