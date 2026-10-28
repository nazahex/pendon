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
use serde_json::Value;

use crate::fields::element_of_layer;

/// `inner_text` is a field like any other: `only` / `omit` / `recommended` can
/// name it next to the node's attributes.
pub const INNER_TEXT: &str = "inner_text";

/// Outline depth: a heading's own `depth` attribute, a section's
/// open-section count, `1` elsewhere.
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

    /// The fields this occurrence *carries*, in emission order — the base that
    /// `only` / `omit` / `recommended` narrow.
    pub fn available(&self) -> Vec<String> {
        let mut names: Vec<String> = self.attrs.iter().map(|(key, _)| key.clone()).collect();
        names.push(INNER_TEXT.to_string());
        // A heading carries `depth` as its own attribute (§7.4), so it is already
        // listed in emission order; only the derived case appends it.
        if !names.iter().any(|name| name == DEPTH) {
            names.push(DEPTH.to_string());
        }
        names
    }

    /// The JSON value of a field, or `None` when the occurrence does not carry it.
    ///
    /// A structured attribute (`JSON_ATTR_PREFIX` + JSON, e.g. `number` once
    /// `plugin-number` ran) is emitted structured; every other attribute stays the
    /// string it is in the stream.
    pub fn value(&self, name: &str) -> Option<Value> {
        match name {
            INNER_TEXT => {
                (!self.inner_text.is_empty()).then(|| Value::String(self.inner_text.clone()))
            }
            DEPTH => Some(Value::from(self.depth)),
            _ => self.get(name).map(attribute_value),
        }
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
                    "heading" => attrs
                        .iter()
                        .find(|(key, _)| key == "depth")
                        .and_then(|(_, value)| value.parse::<u8>().ok())
                        .map(|depth| depth.clamp(1, 6))
                        .unwrap_or(1),
                    "section" => sections.max(1),
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
