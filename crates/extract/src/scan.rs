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
    /// Two things become structure: a dotted attribute name (`soo.baz` nests as
    /// `soo` → `baz`, exactly the path a config has to write) and a structured
    /// value that is an object (`price` → `details` → `amount`). An array is a
    /// leaf: paths address fields, never indices. `inner_text` and `depth` are
    /// fields like any other, so a selection names them next to an attribute.
    pub fn fields(&self) -> Value {
        let mut root = Map::new();
        for (name, raw) in &self.attrs {
            // `depth` is the *derived* outline depth (clamped for a heading, the
            // open-section count for a section), so the attribute it was read
            // from never reaches the row as a bare string.
            if name == DEPTH {
                continue;
            }
            insert_path(&mut root, name, attribute_value(raw));
        }
        if !self.inner_text.is_empty() {
            root.insert(
                INNER_TEXT.to_string(),
                Value::String(self.inner_text.clone()),
            );
        }
        root.insert(DEPTH.to_string(), Value::from(self.depth));
        Value::Object(root)
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
                let Some(element) = element_at(events, index, kind) else {
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

/// The element the node starting at `index` belongs to.
///
/// `__layer` is authoritative (an `img` is `image`);
/// a bare `Heading` node with no layer is a heading too, because the heading
/// plugin owns that node kind outright.
fn element_at(events: &[Event], index: usize, kind: &NodeKind) -> Option<&'static str> {
    for event in &events[index + 1..] {
        match event {
            Event::Attribute { name, value } if name == "__layer" => {
                return element_of_layer(value);
            }
            Event::Attribute { .. } | Event::AttributeFlag { .. } | Event::Diagnostic { .. } => {}
            _ => break,
        }
    }
    matches!(kind, NodeKind::Heading).then_some("heading")
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
