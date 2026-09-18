use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

use pendon_core::{Event, NodeKind, Severity};
use pendon_renderer_solid::{ComponentTemplate, ImportEntry, SolidRenderHints};
use serde_json::{Map, Value};

// --- Public Types ---

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

/// Extra attributes parsed from [.class,#id]{key: val} syntax after cite args.
#[derive(Debug, Clone, Default)]
struct CiteExtraAttrs {
    classes: Vec<String>,
    id: Option<String>,
    data: Vec<(String, String)>,
    styles: Vec<(String, String)>,
}

/// Shared citation state that persists across multiple process_calls within
/// the same document. This allows cite to work correctly in sub-pipelines
/// (e.g., image captions) while maintaining a global index counter and
/// identity deduplication map.
pub struct CitationContext {
    references: Value,
    options: CiteOptions,
    cites: Vec<Value>,
    identities: HashMap<String, usize>,
    diagnostics: Vec<Event>,
}

impl CitationContext {
    /// Creates a new context from references and options.
    /// Call this once per document, then use process_events() for each
    /// event stream (main document, captions, etc.).
    pub fn new(references: Value, options: CiteOptions) -> Self {
        Self {
            references,
            options,
            cites: Vec::new(),
            identities: HashMap::new(),
            diagnostics: Vec::new(),
        }
    }

    /// Processes an event stream, transforming citation syntax into citation nodes.
    /// Mutates internal state (cites, identities, diagnostics) so subsequent calls
    /// continue numbering from where the previous call left off.
    pub fn process_events(&mut self, events: &[Event]) -> Vec<Event> {
        let mut out = Vec::with_capacity(events.len());
        let mut excluded = 0usize;
        let mut in_frontmatter = false;

        for event in events {
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
                Event::Text(text) if excluded == 0 && !in_frontmatter => {
                    emit_text(text, self, &mut out);
                }
                _ => out.push(event.clone()),
            }
        }

        out
    }

    /// Returns all accumulated cites as a JSON array.
    pub fn get_cites(&self) -> Value {
        Value::Array(self.cites.clone())
    }

    /// Returns only the references that were actually cited.
    pub fn get_used_references(&self) -> Value {
        let mut used = Map::new();
        if let Some(refs) = self.references.as_object() {
            for cite in &self.cites {
                if let Some(id) = cite.get("id").and_then(Value::as_str) {
                    if let Some(reference) = refs.get(id) {
                        used.insert(id.to_string(), reference.clone());
                    }
                }
            }
        }
        if let Some(ext) = self
            .options
            .external_references
            .as_ref()
            .and_then(Value::as_object)
        {
            for cite in &self.cites {
                if let Some(id) = cite.get("id").and_then(Value::as_str) {
                    if !used.contains_key(id) {
                        if let Some(reference) = ext.get(id) {
                            used.insert(id.to_string(), reference.clone());
                        }
                    }
                }
            }
        }
        Value::Object(used)
    }

    /// Drains accumulated diagnostics.
    pub fn drain_diagnostics(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.diagnostics)
    }

    /// Returns a reference to the options.
    pub fn options(&self) -> &CiteOptions {
        &self.options
    }

    /// Replaces paragraph nodes containing only the section marker text
    /// with a custom section node. Call this AFTER all process_events() calls
    /// are complete, during finalization.
    pub fn replace_section_markers(&self, events: &[Event]) -> Vec<Event> {
        let Some(section) = &self.options.section else {
            return events.to_vec();
        };

        let mut out = Vec::with_capacity(events.len());
        let mut i = 0usize;
        while i < events.len() {
            if matches!(events.get(i), Some(Event::StartNode(NodeKind::Paragraph))) {
                let mut end = i + 1;
                let mut text = String::new();
                while end < events.len()
                    && !matches!(events[end], Event::EndNode(NodeKind::Paragraph))
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
}

// --- Legacy API (backward compatible) ---

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

/// Legacy entry point. For new code, prefer CitationContext::new() + process_events().
pub fn process(events: &[Event], options: &CiteOptions) -> Vec<Event> {
    let frontmatter = extract_frontmatter(events);
    let front_references = frontmatter
        .as_ref()
        .and_then(|data| data.get("references"))
        .cloned()
        .unwrap_or_else(|| Value::Object(Map::new()));
    let references = merge_references(&front_references, options.external_references.as_ref());

    let mut ctx = CitationContext::new(references, options.clone());
    let processed = ctx.process_events(events);

    let diagnostics = ctx.drain_diagnostics();
    let mut out = processed;
    if !diagnostics.is_empty() {
        let insert_at = out
            .iter()
            .position(|event| matches!(event, Event::StartNode(NodeKind::Document)))
            .map(|idx| idx + 1)
            .unwrap_or(0);
        for (offset, diagnostic) in diagnostics.into_iter().enumerate() {
            out.insert(insert_at + offset, diagnostic);
        }
    }

    let cites = ctx.get_cites();
    let used_refs = ctx.get_used_references();
    update_frontmatter_in_events(&mut out, &cites, &used_refs, options);

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

// --- Internal Processing ---

fn emit_text(text: &str, ctx: &mut CitationContext, out: &mut Vec<Event>) {
    let chars: Vec<char> = text.chars().collect();
    let mut cursor = 0usize;
    let mut normal = String::new();
    while cursor < chars.len() {
        if chars[cursor..].starts_with(&['[', '^', '^', ']', '(']) {
            if let Some((end, id, props, extra)) = parse_citation(&chars, cursor) {
                flush_text(&mut normal, out);
                if reference_exists(&ctx.references, &id) {
                    let mut citation = Map::new();
                    citation.insert("id".to_string(), Value::String(id.clone()));
                    for (key, value) in props {
                        citation.insert(key, Value::String(value));
                    }
                    let identity = canonical_identity(&citation);
                    let index = if let Some(index) = ctx.identities.get(&identity) {
                        *index
                    } else {
                        let index = ctx.cites.len() + 1;
                        citation.insert("index".to_string(), Value::Number(index.into()));
                        ctx.cites.push(Value::Object(citation.clone()));
                        ctx.identities.insert(identity, index);
                        index
                    };
                    emit_citation(out, &ctx.options, &citation, index, &extra);
                } else {
                    let raw: String = chars[cursor..end].iter().collect();
                    normal.push_str(&raw);
                    ctx.diagnostics.push(Event::Diagnostic {
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
    extra: &CiteExtraAttrs,
) {
    if let Some(custom) = &options.custom_node {
        out.push(Event::StartNode(NodeKind::Custom(custom.name.clone())));
        out.push(Event::Attribute {
            name: "name".to_string(),
            value: custom.name.clone(),
        });
        // Emit core citation attributes
        for (key, value) in citation {
            out.push(Event::Attribute {
                name: key.clone(),
                value: value_to_string(value),
            });
        }
        // Emit extra attributes (class, id, data-*, style)
        emit_extra_attrs(out, extra);
        out.push(Event::EndNode(NodeKind::Custom(custom.name.clone())));
        return;
    }

    // Default HTML rendering with extra class support
    let id = citation
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let target = format!("{}{}-{}", options.prefix, index, id);
    let anchor = format!("{}{}", options.id_prefix, index);

    // Build class list: base class_name + extra classes
    let mut classes = vec![options.class_name.clone()];
    classes.extend(extra.classes.iter().cloned());
    let class_attr = classes.join(" ");

    let mut html = format!(
        "<sup class=\"{}\"><a href=\"#{}\" id=\"{}\"",
        escape_html(&class_attr),
        escape_html(&target),
        escape_html(&anchor),
    );

    // Add extra id if present (appended to anchor id)
    if let Some(extra_id) = &extra.id {
        html.push_str(&format!(" data-cite-id=\"{}\"", escape_html(extra_id)));
    }

    // Add data attributes
    for (k, v) in &extra.data {
        html.push_str(&format!(" data-{}=\"{}\"", escape_html(k), escape_html(v)));
    }

    // Add style attributes
    if !extra.styles.is_empty() {
        html.push_str(" style=\"");
        for (k, v) in &extra.styles {
            html.push_str(&escape_html(k));
            html.push(':');
            html.push_str(&escape_html(v));
            html.push(';');
        }
        html.push('"');
    }

    html.push_str(&format!(">[{}]</a></sup>", index));

    out.push(Event::StartNode(NodeKind::HtmlInline));
    out.push(Event::Text(html));
    out.push(Event::EndNode(NodeKind::HtmlInline));
}

fn emit_extra_attrs(out: &mut Vec<Event>, extra: &CiteExtraAttrs) {
    if let Some(id) = &extra.id {
        out.push(Event::Attribute {
            name: "id".to_string(),
            value: id.clone(),
        });
    }
    if !extra.classes.is_empty() {
        out.push(Event::Attribute {
            name: "class".to_string(),
            value: extra.classes.join(" "),
        });
    }
    for (k, v) in &extra.data {
        out.push(Event::Attribute {
            name: format!("data-{}", k),
            value: v.clone(),
        });
    }
    if !extra.styles.is_empty() {
        let style_str: String = extra
            .styles
            .iter()
            .map(|(k, v)| format!("{}:{};", k, v))
            .collect();
        out.push(Event::Attribute {
            name: "style".to_string(),
            value: style_str,
        });
    }
}

// --- Parsing Helpers ---

/// Parses [^^]("id", "loc")[.class,#id]{key: val} syntax.
/// Returns (total_end_position, cite_id, cite_props, extra_attrs).
fn parse_citation(
    chars: &[char],
    start: usize,
) -> Option<(usize, String, BTreeMap<String, String>, CiteExtraAttrs)> {
    // Parse the cite arguments: [^^]("id", "loc")
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

    // Move past the closing ')'
    let mut cursor = end + 1;

    // Parse optional extra attrs: [.class,#id]{key: val}
    let extra = parse_cite_extra_attrs(chars, &mut cursor);

    Some((cursor, id, props, extra))
}

/// Parses [.class,#id]{key: val} after cite arguments.
/// Advances cursor past consumed characters.
fn parse_cite_extra_attrs(chars: &[char], cursor: &mut usize) -> CiteExtraAttrs {
    let mut extra = CiteExtraAttrs::default();

    // Skip whitespace
    while *cursor < chars.len() && chars[*cursor] == ' ' {
        *cursor += 1;
    }

    // Parse optional class/id block: [.class,#id]
    if *cursor < chars.len() && chars[*cursor] == '[' {
        if let Some(close_br) = find_char(chars, *cursor + 1, ']') {
            let block: String = chars[*cursor + 1..close_br].iter().collect();
            for token in block.split(',').map(|t| t.trim()).filter(|t| !t.is_empty()) {
                if let Some(class_name) = token.strip_prefix('.') {
                    if !class_name.is_empty() {
                        extra.classes.push(class_name.to_string());
                    }
                } else if let Some(id) = token.strip_prefix('#') {
                    if !id.is_empty() {
                        extra.id = Some(id.to_string());
                    }
                }
            }
            *cursor = close_br + 1;
        }
    }

    // Skip whitespace between blocks
    while *cursor < chars.len() && chars[*cursor] == ' ' {
        *cursor += 1;
    }

    // Parse optional kv block: {key: val, ...}
    if *cursor < chars.len() && chars[*cursor] == '{' {
        if let Some(close_curly) = find_char(chars, *cursor + 1, '}') {
            let kv_block: String = chars[*cursor + 1..close_curly].iter().collect();
            for pair in split_csv_str(&kv_block) {
                let Some((k, v)) = pair.split_once(':') else {
                    continue;
                };
                let key = k.trim();
                let value = unquote_str(v.trim());
                if key.is_empty() {
                    continue;
                }
                if key.starts_with("--") {
                    extra.styles.push((key.to_string(), value));
                } else {
                    extra.data.push((key.to_string(), value));
                }
            }
            *cursor = close_curly + 1;
        }
    }

    extra
}

fn find_char(chars: &[char], mut index: usize, wanted: char) -> Option<usize> {
    while index < chars.len() {
        if chars[index] == wanted {
            return Some(index);
        }
        index += 1;
    }
    None
}

fn split_csv_str(input: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut quote: Option<char> = None;

    for ch in input.chars() {
        if ch == '"' || ch == '\'' {
            if quote == Some(ch) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(ch);
            }
            buf.push(ch);
            continue;
        }
        if ch == ',' && quote.is_none() {
            if !buf.trim().is_empty() {
                out.push(buf.trim().to_string());
            }
            buf.clear();
            continue;
        }
        buf.push(ch);
    }
    if !buf.trim().is_empty() {
        out.push(buf.trim().to_string());
    }
    out
}

fn unquote_str(s: &str) -> String {
    let bytes = s.as_bytes();
    if bytes.len() >= 2
        && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\''))
    {
        return s[1..s.len() - 1].to_string();
    }
    s.to_string()
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

// --- Frontmatter Helpers ---

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

pub fn update_frontmatter_in_events(
    events: &mut Vec<Event>,
    cites: &Value,
    references: &Value,
    options: &CiteOptions,
) {
    let mut inside = false;
    let mut found = false;
    for event in events.iter_mut() {
        match event {
            Event::StartNode(NodeKind::Frontmatter) => inside = true,
            Event::EndNode(NodeKind::Frontmatter) => inside = false,
            Event::Attribute { name, value } if inside && name == "data" => {
                if let Ok(mut data) = serde_json::from_str::<Value>(value) {
                    if let Value::Object(ref mut obj) = data {
                        obj.insert("cites".to_string(), cites.clone());
                        if options.external_references.is_some() {
                            obj.insert("references".to_string(), references.clone());
                        }
                    }
                    *value = serde_json::to_string(&data).unwrap_or_else(|_| "{}".to_string());
                    found = true;
                }
            }
            _ => {}
        }
    }

    if !found {
        let mut data = Map::new();
        data.insert("cites".to_string(), cites.clone());
        if options.external_references.is_some() {
            data.insert("references".to_string(), references.clone());
        }
        let metadata =
            serde_json::to_string(&Value::Object(data)).unwrap_or_else(|_| "{}".to_string());
        let insert_at = events
            .iter()
            .position(|event| matches!(event, Event::StartNode(NodeKind::Document)))
            .map(|index| index + 1)
            .unwrap_or(0);
        events.splice(
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
    fn shared_context_continues_indexing_across_calls() {
        let references: Value = serde_json::json!({"book": {"title": "Book"}});
        let options = CiteOptions::default();
        let mut ctx = CitationContext::new(references, options);

        let events_a = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Paragraph),
            Event::Text(r#"[^^]("book", "p. 1")"#.into()),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ];
        let _out_a = ctx.process_events(&events_a);

        let events_b = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Paragraph),
            Event::Text(r#"[^^]("book", "p. 2")"#.into()),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ];
        let _out_b = ctx.process_events(&events_b);

        let cites = ctx.get_cites();
        let arr = cites.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["index"], 1);
        assert_eq!(arr[1]["index"], 2);
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
        assert!(result.iter().any(|event| matches!(
            event,
            Event::StartNode(NodeKind::Custom(name)) if name == "Citation"
        )));
    }

    #[test]
    fn parses_extra_attrs_after_cite_args() {
        let chars: Vec<char> =
            r#"[^^]("book", "p. 1")[.highlight,.urgent,#my-cite]{ foo: "bar", --color: "red" }"#
                .chars()
                .collect();
        let (end, id, props, extra) = parse_citation(&chars, 0).unwrap();
        assert_eq!(id, "book");
        assert_eq!(props.get("loc").map(|s| s.as_str()), Some("p. 1"));
        assert_eq!(extra.classes, vec!["highlight", "urgent"]);
        assert_eq!(extra.id, Some("my-cite".to_string()));
        assert_eq!(extra.data, vec![("foo".to_string(), "bar".to_string())]);
        assert_eq!(
            extra.styles,
            vec![("--color".to_string(), "red".to_string())]
        );
        assert!(end > 0);
    }

    #[test]
    fn emits_extra_attrs_on_custom_node() {
        let mut options = CiteOptions::default();
        options.custom_node = Some(CiteCustomNode {
            name: "Citation".into(),
            template: "<Citation />".into(),
            imports: Vec::new(),
        });

        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Frontmatter),
            Event::Attribute {
                name: "data".into(),
                value: r#"{"references":{"book":{"title":"Book"}}}"#.into(),
            },
            Event::EndNode(NodeKind::Frontmatter),
            Event::StartNode(NodeKind::Paragraph),
            Event::Text(r#"[^^]("book")[.hero]{ foo: "bar" }"#.into()),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ];

        let result = process(&events, &options);
        assert!(result.iter().any(|e| matches!(
            e,
            Event::Attribute { name, value } if name == "class" && value == "hero"
        )));
        assert!(result.iter().any(|e| matches!(
            e,
            Event::Attribute { name, value } if name == "data-foo" && value == "bar"
        )));
    }
}
