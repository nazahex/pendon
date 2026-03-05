use pendon_core::Event;
use pendon_renderer_ast::render_ast_to_string;
use serde_json::Value;
use std::collections::BTreeSet;

mod imports;
mod metadata;
mod node;
mod template;

pub use imports::{ComponentTemplate, ImportEntry, SolidRenderHints};

pub fn render_solid(events: &[Event]) -> String {
    render_solid_with_hints(events, None)
}

pub fn render_solid_with_hints(events: &[Event], hints: Option<&SolidRenderHints>) -> String {
    let ast_json = render_ast_to_string(events).expect("AST serialization failed");
    let v: Value = serde_json::from_str(&ast_json).expect("Invalid AST JSON");
    let frontmatter = metadata::extract_frontmatter(&v);
    let headings = metadata::extract_headings(&v);
    let mut used_nodes: BTreeSet<(String, Option<String>)> = BTreeSet::new();
    let mut used_markers: BTreeSet<String> = BTreeSet::new();
    collect_used_nodes(&v, &mut used_nodes);
    collect_used_markers(&v, &mut used_markers, hints);
    let mut body = String::new();
    node::render_node(&v, &mut body, hints);

    let mut out = String::new();
    for line in imports::normalize_imports(hints, &used_nodes, &used_markers) {
        out.push_str(&line);
        out.push('\n');
    }
    if !out.is_empty() {
        out.push('\n');
    }
    if let Some(fm) = frontmatter {
        out.push_str("export const frontmatter = ");
        out.push_str(&fm);
        out.push_str(";\n");
    }
    if let Some(h) = headings {
        out.push_str("export const headings = ");
        out.push_str(&h);
        out.push_str(";\n");
    }
    out.push_str("export default function PendonView() { return (<>");
    out.push('\n');
    out.push_str(&body);
    out.push_str("\n</>); }\n");
    out
}

fn collect_used_nodes(v: &Value, out: &mut BTreeSet<(String, Option<String>)>) {
    if let Some(kind) = v.get("type").and_then(|t| t.as_str()) {
        let name = v
            .get("attrs")
            .and_then(|a| a.get("name"))
            .and_then(|n| n.as_str())
            .map(|s| s.to_string());
        out.insert((kind.to_string(), name.clone()));
        if name.is_some() {
            out.insert((kind.to_string(), None));
        }
    }

    if let Some(children) = v.get("children").and_then(|c| c.as_array()) {
        for ch in children {
            collect_used_nodes(ch, out);
        }
    }
}

fn collect_used_markers(
    v: &Value,
    out: &mut BTreeSet<String>,
    hints: Option<&imports::SolidRenderHints>,
) {
    let Some(hints) = hints else {
        return;
    };
    if hints.text_imports.is_empty() {
        return;
    }

    let mut maybe_match = |text: &str| {
        for (marker, _) in &hints.text_imports {
            if text.contains(marker) {
                out.insert(marker.clone());
            }
        }
    };

    if let Some(text) = v.get("text").and_then(|t| t.as_str()) {
        maybe_match(text);
    }
    if let Some(children) = v.get("children").and_then(|c| c.as_array()) {
        for ch in children {
            collect_used_markers(ch, out, Some(hints));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pendon_core::{Event, NodeKind};

    fn html_block_events() -> Vec<Event> {
        vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::HtmlBlock),
            Event::Text("<div>raw</div>".to_string()),
            Event::EndNode(NodeKind::HtmlBlock),
            Event::EndNode(NodeKind::Document),
        ]
    }

    fn html_inline_events() -> Vec<Event> {
        vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Paragraph),
            Event::StartNode(NodeKind::HtmlInline),
            Event::Text("<span>inline</span>".to_string()),
            Event::EndNode(NodeKind::HtmlInline),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ]
    }

    #[test]
    fn html_block_passes_raw_html() {
        let output = render_solid(&html_block_events());
        assert!(output.contains("<div>raw</div>"));
    }

    #[test]
    fn html_inline_passes_raw_fragment() {
        let output = render_solid(&html_inline_events());
        assert!(output.contains("<span>inline</span>"));
    }
}
