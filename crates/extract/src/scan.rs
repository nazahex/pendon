//! Reading element occurrences out of the event stream.
//!
//! Extraction is a *reader*: it never rewrites the stream (P.2), so the document
//! keeps rendering exactly as it would without the plugin. A node belongs to an
//! element when its `__layer` attribute says so (the same contract `plugin-number`
//! and the §11 component sets use) — the events are the one place that knows every
//! extras field, including the ones no renderer ever sees.

use pendon_core::{
    is_json_attr, is_jsx_attr, json_attr_payload, jsx_attr_payload, Event, NodeKind,
};
use pendon_extra::FieldConfig;
use serde_json::{Map, Value};

use crate::fields::element_of_layer;

/// `inner_text` is a field like any other: `only` / `omit` / `preset` can
/// name it next to the node's attributes.
pub const INNER_TEXT: &str = "inner_text";

/// Outline depth: a heading's or a section's own `depth` attribute, the
/// open-section count for a section without one, `1` elsewhere.
pub const DEPTH: &str = "depth";

/// One element occurrence, as the extractor sees it.
#[derive(Clone, Debug)]
pub struct ElementNode {
    pub element: &'static str,
    /// The `__layer` value the node carries (`img` for an image, `unordered` for
    /// a bullet list) — what a `field.*` lookup is keyed by (D-7).
    pub layer: String,
    /// Attributes in emission order, `__`-prefixed internal keys removed.
    pub attrs: Vec<(String, String)>,
    /// The node's text, descendants included, whitespace collapsed.
    pub inner_text: String,
    pub depth: u8,
}

impl ElementNode {
    /// The value of an attribute, internal keys excluded.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    /// The occurrence's fields as one nested object — the tree `include` /
    /// `only` / `omit` walk (§21 rule 4).
    ///
    /// Every field goes through **the projection** ([`pendon_extra::project`]),
    /// the same function the renderers call, so extraction can never disagree
    /// with the page about what a public field is called: `field.override.*`
    /// renames apply (D-4), `specs.*` positions land under their DOM names, and
    /// a `__`-prefixed internal key never reaches the row (S2-8.4).
    ///
    /// What this adds to the node before projecting: the **derived** `depth`
    /// (the attribute it was read from is a string, and a heading's depth is
    /// clamped), and `inner_text`, which is not an attribute at all. Both go in
    /// as attributes so an override can rename them like any other field.
    ///
    /// Nesting is the projection's: a dotted path (`soo.baz`, `specs.tick`) is
    /// two segments, an object value keeps its shape, and an array stays a leaf.
    pub fn fields(&self, config: Option<&FieldConfig>) -> Value {
        let mut attrs = Map::new();
        for (name, raw) in &self.attrs {
            // Replaced below: `depth` ships the derived value, never the string
            // the emitter wrote.
            if name == DEPTH {
                continue;
            }
            insert_path(&mut attrs, name, attribute_value(raw));
        }
        insert_path(&mut attrs, DEPTH, Value::from(self.depth));
        if !self.inner_text.is_empty() {
            insert_path(
                &mut attrs,
                INNER_TEXT,
                Value::String(self.inner_text.clone()),
            );
        }
        let node = Value::Object(Map::from_iter([(
            "attrs".to_string(),
            Value::Object(attrs),
        )]));
        pendon_extra::project(&node, config).to_value().to_json()
    }
}

/// Stores `value` at the dotted path `name`, creating intermediate objects.
///
/// A name that is a plain field lands at the root. When a path collides with a
/// scalar the nested name wins (a dotted name is an explicit path), and a
/// repeated exact name keeps the later attribute — both are impossible for the
/// names the pipeline produces today, so the rule only has to be predictable.
fn insert_path(object: &mut Map<String, Value>, name: &str, value: Value) {
    let Some((head, rest)) = name.split_once('.') else {
        object.insert(name.to_string(), value);
        return;
    };
    let entry = object
        .entry(head.to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    if !entry.is_object() {
        *entry = Value::Object(Map::new());
    }
    if let Value::Object(child) = entry {
        insert_path(child, rest, value);
    }
}

/// A node attribute as a JSON value: decoded when the stream marked it as JSON,
/// stripped when it is a raw JSX fragment, kept verbatim otherwise.
fn attribute_value(raw: &str) -> Value {
    if is_json_attr(raw) {
        if let Some(payload) = json_attr_payload(raw) {
            if let Ok(value) = serde_json::from_str::<Value>(payload) {
                return value;
            }
        }
    }
    if is_jsx_attr(raw) {
        if let Some(payload) = jsx_attr_payload(raw) {
            return Value::String(payload.to_string());
        }
    }
    Value::String(raw.to_string())
}

/// Every occurrence of every extractable element, in document order.
pub fn scan(events: &[Event]) -> Vec<ElementNode> {
    let mut nodes = Vec::new();
    let mut sections: u8 = 0;
    for (index, event) in events.iter().enumerate() {
        match event {
            Event::StartNode(kind) => {
                if matches!(kind, NodeKind::Section) {
                    sections = sections.saturating_add(1);
                }
                let Some((element, layer)) = element_at(events, index, kind) else {
                    continue;
                };
                let (attrs, inner_text) = consume(events, index, kind);
                let depth = match element {
                    // §9.5/§7.4: both publish their own `depth`, so the attribute is
                    // the answer. The section count stays as the fallback for a node
                    // an older/foreign emitter left without one.
                    "section" => attrs
                        .iter()
                        .find(|(key, _)| key == "depth")
                        .and_then(|(_, value)| value.parse::<u8>().ok())
                        .unwrap_or_else(|| sections.max(1)),
                    "heading" => attrs
                        .iter()
                        .find(|(key, _)| key == "depth")
                        .and_then(|(_, value)| value.parse::<u8>().ok())
                        .map(|depth| depth.clamp(1, 6))
                        .unwrap_or(1),
                    _ => 1,
                };
                nodes.push(ElementNode {
                    element,
                    layer: layer.to_string(),
                    attrs,
                    inner_text,
                    depth,
                });
            }
            Event::EndNode(NodeKind::Section) => {
                sections = sections.saturating_sub(1);
            }
            _ => {}
        }
    }
    nodes
}

/// The attribute `micromatter` carries the parsed frontmatter in (§20): one
/// `Frontmatter` node with a `data` attribute holding compact JSON.
pub const FRONTMATTER_DATA: &str = "data";

/// The document's frontmatter as one object, when the chain produced a
/// `Frontmatter` node (D-14).
///
/// Frontmatter is not an element occurrence — it has no `__layer` and no
/// `inner_text` — so it is read straight out of the node `micromatter` writes
/// instead of going through [`scan`]. Its keys are the fields, and a nested key
/// is a path, exactly like an attribute path.
pub fn frontmatter(events: &[Event]) -> Option<Value> {
    let mut inside = false;
    for event in events {
        match event {
            Event::StartNode(NodeKind::Frontmatter) => inside = true,
            Event::EndNode(NodeKind::Frontmatter) => inside = false,
            Event::Attribute { name, value } if inside && name == FRONTMATTER_DATA => {
                return serde_json::from_str(value).ok();
            }
            _ => {}
        }
    }
    None
}

/// The element the node starting at `index` belongs to.
///
/// `__layer` is authoritative (an `img` is `image`);
/// a bare `Heading` node with no layer is a heading too, because the heading
/// plugin owns that node kind outright.
fn element_at<'a>(
    events: &'a [Event],
    index: usize,
    kind: &NodeKind,
) -> Option<(&'static str, &'a str)> {
    for event in &events[index + 1..] {
        match event {
            Event::Attribute { name, value } if name == "__layer" => {
                return element_of_layer(value).map(|element| (element, value.as_str()));
            }
            Event::Attribute { .. } | Event::AttributeFlag { .. } | Event::Diagnostic { .. } => {}
            _ => break,
        }
    }
    matches!(kind, NodeKind::Heading).then_some(("heading", "heading"))
}

/// The attribute run and the text of the node starting at `start`.
///
/// The run ends at the first content event; the text runs to the node's own
/// `EndNode`, so nested inline nodes are flattened into `inner_text`.
fn consume(
    events: &[Event],
    start: usize,
    start_kind: &NodeKind,
) -> (Vec<(String, String)>, String) {
    let mut attrs: Vec<(String, String)> = Vec::new();
    let mut text = String::new();
    let mut in_attributes = true;
    let mut inline = 0usize;
    for event in &events[start + 1..] {
        if in_attributes {
            match event {
                Event::Attribute { name, value } => {
                    if !name.starts_with("__") {
                        attrs.push((name.clone(), value.clone()));
                    }
                    continue;
                }
                Event::AttributeFlag { name } => {
                    // §6.3: a bare flag is `true`, never `flag="flag"`.
                    if !name.starts_with("__") {
                        attrs.push((name.clone(), "true".to_string()));
                    }
                    continue;
                }
                Event::Diagnostic { .. } => continue,
                _ => in_attributes = false,
            }
        }
        match event {
            Event::Text(chunk) => text.push_str(chunk),
            Event::StartNode(_) => inline += 1,
            Event::EndNode(kind) => {
                if inline == 0 && kind == start_kind {
                    break;
                }
                inline = inline.saturating_sub(1);
            }
            _ => {}
        }
    }
    (attrs, collapse(&text))
}

/// Collapses whitespace runs so a row is stable across line wrapping.
fn collapse(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
