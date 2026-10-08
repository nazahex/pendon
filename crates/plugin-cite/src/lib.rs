use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

use pendon_core::{Event, NodeKind, Severity};
use pendon_extra::{scan_extras_chars, to_attributes, ExtrasAttr, ExtrasHead};
use pendon_renderer_solid::{ComponentSet, ComponentTemplate, ImportEntry, SolidRenderHints};
use serde_json::{Map, Value};

// --- Public Types ---

#[derive(Clone, Debug, Default)]
pub struct CiteCustomNode {
    pub name: String,
    pub template: String,
    pub imports: Vec<ImportEntry>,
}

#[derive(Clone, Debug)]
pub struct CiteOptions {
    pub prefix: String,
    pub class_name: String,
    pub id_prefix: String,
    /// §11 component set of the `cite` layer: typed entries plus at most one
    /// default, selected per instance by the `@@type{…}` marker (rule 3).
    pub custom: ComponentSet<CiteCustomNode>,
    pub external_references: Option<Value>,
}

impl Default for CiteOptions {
    fn default() -> Self {
        Self {
            prefix: "citeref-".to_string(),
            class_name: "cite-ref".to_string(),
            id_prefix: "cra-".to_string(),
            custom: ComponentSet::new(),
            external_references: None,
        }
    }
}

/// Extra attributes parsed from the §7.3 `{…}` / `@@type{…}` extras head after
/// the cite args. The retired `[.class,#id]{key: val}` block is literal text
/// (§14).
#[derive(Debug, Clone, Default)]
struct CiteExtraAttrs {
    classes: Vec<String>,
    id: Option<String>,
    data: Vec<(String, String)>,
    /// The `style` value the extras head merged from `--var` items (§6.3).
    extras_style: Option<String>,
    /// Bare flags of the extras head (§6.3), emitted as bare attributes.
    flags: Vec<String>,
    /// §13 warnings raised while merging the head.
    warnings: Vec<String>,
    /// The `@@type{…}` marker, when present: the §11 routing key (rule 3).
    type_marker: Option<String>,
}

impl CiteExtraAttrs {
    /// Whether the construct side already sets `key` (extras then lose, §6.2).
    fn has_key(&self, key: &str) -> bool {
        match key {
            "id" => self.id.is_some(),
            "class" => !self.classes.is_empty(),
            "style" => self.extras_style.is_some(),
            _ => self.data.iter().any(|(name, _)| name == key),
        }
    }
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
    let mut add_node = |name: &str, template: &str, imports: &[ImportEntry]| {
        let key = (name.to_string(), Some(name.to_string()));
        hints.templates.push(ComponentTemplate {
            node_type: name.to_string(),
            node_name: Some(name.to_string()),
            template: template.to_string(),
        });
        hints.template_imports.insert(key, imports.to_vec());
    };
    if !options.custom.is_empty() {
        // §11 rule 3: every entry of the set answers instances of its own.
        for custom in options.custom.components() {
            add_node(&custom.name, &custom.template, &custom.imports);
        }
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
            if let Some((end, id, props, mut extra)) = parse_citation(&chars, cursor, ctx.options())
            {
                flush_text(&mut normal, out);

                // §7.3/§6.2: the cite args are the construct head and win over
                // a same-named extras prop.
                let head_keys: Vec<String> = props.keys().cloned().collect();
                let mut dropped: Vec<String> = Vec::new();
                extra.data.retain(|(key, _)| {
                    if head_keys.contains(key) {
                        dropped.push(key.clone());
                        false
                    } else {
                        true
                    }
                });
                for key in dropped {
                    extra.warnings.push(format!(
                        "`{key}` was dropped because the construct head already sets it"
                    ));
                }
                for message in &extra.warnings {
                    out.push(Event::Diagnostic {
                        severity: Severity::Warning,
                        message: format!("[cite] {message}"),
                        span: None,
                    });
                }

                if reference_exists(&ctx.references, &id) {
                    let mut citation = Map::new();
                    citation.insert("id".to_string(), Value::String(id.clone()));
                    for (key, value) in props {
                        citation.insert(key, Value::String(value));
                    }

                    // Calculate identity BEFORE inserting index to ensure proper deduplication.
                    let identity = canonical_identity(&citation);

                    let index = if let Some(&existing_index) = ctx.identities.get(&identity) {
                        existing_index
                    } else {
                        let new_index = ctx.cites.len() + 1;

                        // Push to the global cites array with the index included
                        let mut cite_for_array = citation.clone();
                        cite_for_array.insert("index".to_string(), Value::Number(new_index.into()));
                        ctx.cites.push(Value::Object(cite_for_array));

                        ctx.identities.insert(identity, new_index);
                        new_index
                    };

                    // CRITICAL: Always ensure index is present in the local citation map
                    // so it gets emitted as an attribute for custom nodes.
                    citation.insert("index".to_string(), Value::Number(index.into()));

                    emit_citation(out, &ctx.options, &citation, index, &extra);
                } else {
                    // Reference not found: emit raw syntax as plain text and log diagnostic
                    let raw: String = chars[cursor..end].iter().collect();
                    normal.push_str(&raw);
                    ctx.diagnostics.push(Event::Diagnostic {
                        severity: Severity::Error,
                        message: format!("[cite] reference '{}' was not found", id),
                        span: None,
                    });
                }

                // CRITICAL: Always advance cursor past the parsed citation,
                // regardless of whether the reference existed or not.
                cursor = end;
                continue;
            }
        }

        // Fallback: character is not part of a valid citation syntax
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
    // §11 rule 3: the marker picks the component, an unmatched marker falls
    // back to the layer default, no default to the built-in `<sup>` markup.
    if let Some(custom) = options.custom.select(extra.type_marker.as_deref()) {
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
        // Emit extra attributes (class, id, props, style)
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
    let style = cite_style(extra);
    if !style.is_empty() {
        html.push_str(&format!(" style=\"{}\"", escape_html(&style)));
    }

    // §6.3: a bare flag stays a bare attribute in the markup too.
    for name in &extra.flags {
        html.push_str(&format!(" {}", escape_html(name)));
    }

    html.push_str(&format!(">[{}]</a></sup>", index));

    out.push(Event::StartNode(NodeKind::HtmlInline));
    out.push(Event::Text(html));
    out.push(Event::EndNode(NodeKind::HtmlInline));
}

/// Emits extra attributes on a custom component node.
///
/// Unlike the default HTML path (which renders arbitrary props as `data-*`
/// attributes), a custom Solid component receives plain props, so the keys are
/// passed through verbatim. The extra `#id` is emitted as `cite-id` to keep it
/// distinguishable from the citation reference `id`.
fn emit_extra_attrs(out: &mut Vec<Event>, extra: &CiteExtraAttrs) {
    if let Some(id) = &extra.id {
        out.push(Event::Attribute {
            name: "cite-id".to_string(),
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
            name: k.clone(),
            value: v.clone(),
        });
    }
    let style = cite_style(extra);
    if !style.is_empty() {
        out.push(Event::Attribute {
            name: "style".to_string(),
            value: style,
        });
    }
    // §6.3: a bare flag stays a bare attribute on custom components too.
    for name in &extra.flags {
        out.push(Event::AttributeFlag { name: name.clone() });
    }
}

/// The `style` value of a citation: the `--var` items and the `style:` prop the
/// extras head merged (§6.3).
fn cite_style(extra: &CiteExtraAttrs) -> String {
    let mut styles = String::new();
    if let Some(value) = &extra.extras_style {
        styles.push_str(value);
        if !styles.ends_with(';') {
            styles.push(';');
        }
    }
    styles
}

// --- Parsing Helpers ---

/// Parses `[^^](ref "loc")` (§7.3) plus an adjacent extras head.
///
/// The reference is an **unquoted** token and the location is an optional
/// **quoted** string. Everything else stays literal text: a quoted reference
/// (`[^^](book)`), the retired `("ref", "loc")` form, a `loc=` prop and an
/// empty `[^^]()` are not citations.
///
/// Returns (total_end_position, cite_id, cite_props, extra_attrs).
fn parse_citation(
    chars: &[char],
    start: usize,
    options: &CiteOptions,
) -> Option<(usize, String, BTreeMap<String, String>, CiteExtraAttrs)> {
    // `[^^](` is five characters.
    let mut cursor = start + 5;

    while matches!(chars.get(cursor), Some(ch) if ch.is_whitespace()) {
        cursor += 1;
    }
    let reference_start = cursor;
    while let Some(&ch) = chars.get(cursor) {
        if ch == ')' || ch.is_whitespace() {
            break;
        }
        if ch == '"' || ch == ',' {
            return None;
        }
        cursor += 1;
    }
    if cursor == reference_start {
        return None;
    }
    let id: String = chars[reference_start..cursor].iter().collect();
    let mut props = BTreeMap::new();

    // An optional quoted location, separated from the reference by whitespace.
    let mut after_space = cursor;
    while matches!(chars.get(after_space), Some(ch) if ch.is_whitespace()) {
        after_space += 1;
    }
    if after_space > cursor {
        if chars.get(after_space) != Some(&'"') {
            return None;
        }
        let mut quote_end = after_space + 1;
        while let Some(&ch) = chars.get(quote_end) {
            if ch == '"' {
                break;
            }
            quote_end += 1;
        }
        if chars.get(quote_end) != Some(&'"') {
            return None;
        }
        let loc: String = chars[after_space + 1..quote_end].iter().collect();
        props.insert("loc".to_string(), loc);
        cursor = quote_end + 1;
        while matches!(chars.get(cursor), Some(ch) if ch.is_whitespace()) {
            cursor += 1;
        }
    }

    if chars.get(cursor) != Some(&')') {
        return None;
    }
    cursor += 1;

    let extra = parse_cite_extra_attrs(chars, &mut cursor, options);
    Some((cursor, id, props, extra))
}

/// Parses the blocks attached to a citation: the adjacent §7.3 `{…}` /
/// `@@type{…}` extras head. It attaches to the citation node; the construct head
/// (the citation args) wins (§6.2). The retired `[.class,#id]{key: val}` block is
/// literal text (§14).
fn parse_cite_extra_attrs(
    chars: &[char],
    cursor: &mut usize,
    options: &CiteOptions,
) -> CiteExtraAttrs {
    let mut extra = CiteExtraAttrs::default();

    // §7.3/§4.1: the extras head must be adjacent (`[^^](book)@@cite{…}`).
    let rest: Vec<char> = chars[*cursor..].to_vec();
    if let Some((head, next)) = scan_extras_chars(&rest, 0) {
        *cursor += next;
        merge_cite_extras(&mut extra, &head, options);
    }

    extra
}

/// Merges a §7.3 extras head into the citation's extra attributes.
///
/// The construct side wins (§6.2): a slot the head or the deprecated block
/// already set is kept and the extras value is reported as dropped. `class`
/// accumulates (§6.4) and bare flags stay bare attributes (§6.3).
fn merge_cite_extras(extra: &mut CiteExtraAttrs, head: &ExtrasHead, options: &CiteOptions) {
    // §6.1/§11 rule 5: the entry answering this marker names the extras keys.
    let keys = options.custom.keys_for(head.type_marker.as_deref());
    let parsed = to_attributes(head, &keys);
    for warning in &parsed.warnings {
        extra.warnings.push(pendon_extra::warning_message(warning));
    }

    // §6.2: `#id` > extras `slug`; both feed the `cite-id` slot.
    let extras_id = parsed.value("id").map(|value| value.literal());
    let extras_slug = parsed
        .value(&keys.backtick_key)
        .map(|value| value.literal());

    for (key, value) in &parsed.items {
        if key == "id" || key == &keys.backtick_key {
            continue;
        }
        match value {
            ExtrasAttr::Flag => {
                if extra.has_key(key) {
                    extra.warnings.push(format!(
                        "`{key}` was dropped because the construct head already sets it"
                    ));
                } else {
                    extra.flags.push(key.clone());
                }
            }
            ExtrasAttr::Value(value) => {
                let text = value.literal();
                if key == "class" {
                    for token in text.split_whitespace() {
                        extra.classes.push(token.to_string());
                    }
                    continue;
                }
                if key == "style" {
                    if extra.has_key("style") {
                        extra.warnings.push(
                            "`style` was dropped because the construct head already sets it"
                                .to_string(),
                        );
                    } else {
                        extra.extras_style = Some(text);
                    }
                    continue;
                }
                if extra.has_key(key) {
                    extra.warnings.push(format!(
                        "`{key}` was dropped because the construct head already sets it"
                    ));
                    continue;
                }
                extra.data.push((key.clone(), text));
            }
        }
    }

    match extras_id.or(extras_slug) {
        Some(id) if extra.id.is_none() => extra.id = Some(id),
        Some(_) => extra.warnings.push(
            "extras `id` was dropped because the citation already has an id (§6.2)".to_string(),
        ),
        None => {}
    }

    // §11 rule 3: the type marker is the routing key of the citation; it is
    // carried as a `type` attribute so a `{attrs.type}` template can read it
    // back. An explicit `type:` prop keeps its own value.
    extra.type_marker = head.type_marker.clone();
    if let Some(marker) = &head.type_marker {
        if !extra.has_key("type") {
            extra.data.push(("type".to_string(), marker.clone()));
        }
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
    use pendon_renderer_solid::TypedComponent;

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
            Event::Text(r#"A [^^](book "p. 1") B [^^](book "p. 1") C [^^](book "p. 2")."#.into()),
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
            Event::Text(r#"[^^](book "p. 1")"#.into()),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ];
        let _out_a = ctx.process_events(&events_a);

        let events_b = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Paragraph),
            Event::Text(r#"[^^](book "p. 2")"#.into()),
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
        options.custom =
            ComponentSet::from_entries([TypedComponent::default_component(CiteCustomNode {
                name: "Citation".into(),
                template: "<Citation>{children}</Citation>".into(),
                imports: Vec::new(),
            })]);
        let result = process(&events_with_refs(), &options);
        assert!(result.iter().any(|event| matches!(
            event,
            Event::StartNode(NodeKind::Custom(name)) if name == "Citation"
        )));
    }

    #[test]
    fn parses_extra_attrs_after_cite_args() {
        let chars: Vec<char> =
            r#"[^^](book "p. 1"){.highlight, .urgent, #my-cite, foo: "bar", --color: "red"}"#
                .chars()
                .collect();
        let (end, id, props, extra) = parse_citation(&chars, 0, &CiteOptions::default()).unwrap();
        assert_eq!(id, "book");
        assert_eq!(props.get("loc").map(|s| s.as_str()), Some("p. 1"));
        assert_eq!(extra.classes, vec!["highlight", "urgent"]);
        assert_eq!(extra.id, Some("my-cite".to_string()));
        assert_eq!(extra.data, vec![("foo".to_string(), "bar".to_string())]);
        assert_eq!(extra.extras_style.as_deref(), Some("--color: red"));
        assert!(end > 0);
    }

    /// §7.3/D5: the only accepted forms are an unquoted ref and an optional
    /// quoted loc. Everything else is literal text.
    #[test]
    fn the_retired_cite_forms_are_literal_text() {
        for source in [
            concat!("[^^](", "\"book\"", ")"),
            concat!("[^^](book, ", "\"p. 1\"", ")"),
            concat!("[^^](book, loc=", "\"p. 1\"", ")"),
            "[^^]()",
        ] {
            let chars: Vec<char> = source.chars().collect();
            assert!(
                parse_citation(&chars, 0, &CiteOptions::default()).is_none(),
                "{source} must stay literal"
            );
        }

        // …and none of them renders a citation.
        let result = process(
            &events_with_text(concat!(
                "[^^](",
                "\"book\"",
                ") and [^^](book, ",
                "\"p. 1\"",
                ") and [^^]()"
            )),
            &CiteOptions::default(),
        );
        assert!(html_text(&result).is_empty(), "{}", html_text(&result));
        assert!(result.iter().any(|event| matches!(
            event,
            Event::Text(text) if text.contains("book")
        )));
    }

    #[test]
    fn emits_extra_attrs_on_custom_node() {
        let mut options = CiteOptions::default();
        options.custom =
            ComponentSet::from_entries([TypedComponent::default_component(CiteCustomNode {
                name: "Citation".into(),
                template: "<Citation />".into(),
                imports: Vec::new(),
            })]);

        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Frontmatter),
            Event::Attribute {
                name: "data".into(),
                value: r#"{"references":{"book":{"title":"Book"}}}"#.into(),
            },
            Event::EndNode(NodeKind::Frontmatter),
            Event::StartNode(NodeKind::Paragraph),
            Event::Text(r#"[^^](book){.hero, foo: "bar"}"#.into()),
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
            Event::Attribute { name, value } if name == "foo" && value == "bar"
        )));
    }

    /// The raw HTML the default (non-custom) citation path emits.
    fn html_text(events: &[Event]) -> String {
        events
            .iter()
            .filter_map(|event| match event {
                Event::Text(text) if text.starts_with("<sup") => Some(text.clone()),
                _ => None,
            })
            .collect()
    }

    fn events_with_text(text: &str) -> Vec<Event> {
        vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Frontmatter),
            Event::Attribute {
                name: "data".into(),
                value: r#"{"references":{"book":{"title":"Book"}}}"#.into(),
            },
            Event::EndNode(NodeKind::Frontmatter),
            Event::StartNode(NodeKind::Paragraph),
            Event::Text(text.to_string()),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ]
    }

    #[test]
    fn parses_the_extras_head_after_cite_args() {
        let chars: Vec<char> =
            r#"[^^](book)@@cite{.highlight, #short, note: "x", --color: "red"} tail"#
                .chars()
                .collect();
        let (end, id, props, extra) = parse_citation(&chars, 0, &CiteOptions::default()).unwrap();
        assert_eq!(id, "book");
        assert!(props.is_empty());
        assert_eq!(extra.classes, vec!["highlight"]);
        assert_eq!(extra.id.as_deref(), Some("short"));
        // §11 rule 3: the marker rides along as the `type` attribute.
        assert_eq!(extra.type_marker.as_deref(), Some("cite"));
        assert_eq!(
            extra.data,
            vec![
                ("note".to_string(), "x".to_string()),
                ("type".to_string(), "cite".to_string()),
            ]
        );
        assert_eq!(extra.extras_style.as_deref(), Some("--color: red"));

        // Only the citation and its head are consumed, the trailing text stays.
        let consumed: String = chars[..end].iter().collect();
        assert_eq!(
            consumed,
            r#"[^^](book)@@cite{.highlight, #short, note: "x", --color: "red"}"#
        );
    }

    #[test]
    fn extras_merge_into_the_custom_citation_node() {
        let mut options = CiteOptions::default();
        options.custom =
            ComponentSet::from_entries([TypedComponent::default_component(CiteCustomNode {
                name: "Citation".into(),
                template: "<Citation />".into(),
                imports: Vec::new(),
            })]);

        let result = process(
            &events_with_text(r#"[^^](book)@@cite{.hero, citeId: "short", isFoo}"#),
            &options,
        );

        assert!(result.iter().any(|e| matches!(
            e,
            Event::Attribute { name, value } if name == "class" && value == "hero"
        )));
        assert!(result.iter().any(|e| matches!(
            e,
            Event::Attribute { name, value } if name == "citeId" && value == "short"
        )));
        // §6.3: bare flags stay bare attributes.
        assert!(result
            .iter()
            .any(|e| matches!(e, Event::AttributeFlag { name } if name == "isFoo")));
        // §7.3: the reference id of the citation is untouched.
        assert!(result.iter().any(|e| matches!(
            e,
            Event::Attribute { name, value } if name == "id" && value == "book"
        )));
    }

    #[test]
    fn cite_args_win_over_extras_props() {
        let result = process(
            &events_with_text(r#"[^^](book "p. 1")@@cite{loc: "p. 9", note: "n"}"#),
            &CiteOptions::default(),
        );

        let html = html_text(&result);
        assert!(html.contains("data-note=\"n\""), "{html}");
        assert!(!html.contains("loc"), "{html}");
        assert!(result.iter().any(|event| matches!(
            event,
            Event::Diagnostic { message, .. }
                if message.contains("[cite]") && message.contains("`loc`")
        )));
    }

    #[test]
    fn extras_style_and_flags_reach_the_markup() {
        let result = process(
            &events_with_text(r#"[^^](book)@@cite{.hero, --color: "red", isFoo}"#),
            &CiteOptions::default(),
        );

        let html = html_text(&result);
        assert!(html.contains("class=\"cite-ref hero\""), "{html}");
        assert!(html.contains("style=\"--color: red;\""), "{html}");
        assert!(html.contains(" isFoo>"), "{html}");
    }

    #[test]
    fn cites_extras_id_lands_in_the_cite_id_slot() {
        let result = process(
            &events_with_text(r#"[^^](book)@@cite{`short-slug`}"#),
            &CiteOptions::default(),
        );
        // §6.2: `#id` > extras `slug`.
        assert!(html_text(&result).contains("data-cite-id=\"short-slug\""));
    }

    /// §14/D3: the retired `[.class,#id]{k:v}` block is literal text.
    #[test]
    fn the_retired_block_is_literal_text() {
        let result = process(
            &events_with_text(r#"[^^](book)[#short]"#),
            &CiteOptions::default(),
        );
        assert!(
            html_text(&result).starts_with("<sup"),
            "{:?}",
            html_text(&result)
        );
        assert!(!html_text(&result).contains("[#short]"));
        assert!(result.iter().any(|event| matches!(
            event,
            Event::Text(text) if text.contains("[#short]")
        )));
    }

    #[test]
    fn extra_id_does_not_replace_citation_id_on_custom_node() {
        let mut options = CiteOptions::default();
        options.custom =
            ComponentSet::from_entries([TypedComponent::default_component(CiteCustomNode {
                name: "Citation".into(),
                template: "<Citation />".into(),
                imports: Vec::new(),
            })]);

        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Frontmatter),
            Event::Attribute {
                name: "data".into(),
                value: r#"{"references":{"book":{"title":"Book"}}}"#.into(),
            },
            Event::EndNode(NodeKind::Frontmatter),
            Event::StartNode(NodeKind::Paragraph),
            Event::Text(r#"[^^](book){#short}"#.into()),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ];

        let result = process(&events, &options);
        assert!(result.iter().any(|event| matches!(
            event,
            Event::Attribute { name, value } if name == "id" && value == "book"
        )));
        assert!(result.iter().any(|event| matches!(
            event,
            Event::Attribute { name, value } if name == "cite-id" && value == "short"
        )));
    }
}
