use pendon_core::Event;
use pendon_renderer_ast::render_ast_to_string;
use serde_json::Value;
use std::collections::BTreeSet;

mod components;
mod imports;
mod metadata;
mod node;
mod template;

pub use components::{ComponentSet, TypedComponent};
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
    // RFC-0002 §2.3: reference/citation payloads ride as module-level exports,
    // never through frontmatter. References come first so a reader can resolve
    // a citation's `reference` prop without forward references.
    if let Some(h) = hints {
        if let Some(v) = &h.references_export {
            let name = h.references_export_name.as_deref().unwrap_or("references");
            push_data_export(&mut out, name, v);
        }
        if let Some(v) = &h.citations_export {
            let name = h.citations_export_name.as_deref().unwrap_or("citations");
            push_data_export(&mut out, name, v);
        }
    }
    out.push_str("export default function PendonView() { return (<>");
    out.push('\n');
    out.push_str(&body);
    out.push_str("\n</>); }\n");
    out
}

/// Emit `export const <name> = <json>;` — one module-level data binding.
///
/// `Value` always serialises, so the fallback is only a defensive guard.
fn push_data_export(out: &mut String, name: &str, value: &Value) {
    out.push_str("export const ");
    out.push_str(name);
    out.push_str(" = ");
    out.push_str(&serde_json::to_string(value).unwrap_or_else(|_| "null".to_string()));
    out.push_str(";\n");
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

    // A fragment (`$fragment`, a nested `mdp` payload) is a node list stored in
    // an attribute value. Its components need their imports just like the
    // document's own children.
    if let Some(attrs) = v.get("attrs").and_then(|a| a.as_object()) {
        for value in attrs.values() {
            collect_fragment_nodes(value, out);
        }
    }
}

fn collect_fragment_nodes(v: &Value, out: &mut BTreeSet<(String, Option<String>)>) {
    if let Some(nodes) = pendon_renderer_ast::fragment_nodes(v) {
        for node in nodes {
            collect_used_nodes(node, out);
        }
        return;
    }
    match v {
        Value::Array(items) => {
            for item in items {
                collect_fragment_nodes(item, out);
            }
        }
        Value::Object(map) => {
            for item in map.values() {
                collect_fragment_nodes(item, out);
            }
        }
        _ => {}
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

    fn raw_codefence_events() -> Vec<Event> {
        vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::CodeFence),
            Event::Attribute {
                name: "raw_html".to_string(),
                value: "1".to_string(),
            },
            Event::Text("<p>  <b>x</b></p>".to_string()),
            Event::EndNode(NodeKind::CodeFence),
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

    #[test]
    fn html_comments_use_jsx_comment_syntax() {
        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::HtmlBlock),
            Event::Text("<!-- comment -->".to_string()),
            Event::EndNode(NodeKind::HtmlBlock),
            Event::EndNode(NodeKind::Document),
        ];
        let output = render_solid(&events);
        assert!(output.contains("{/* comment */}"));
        assert!(!output.contains("<!-- comment -->"));
    }

    #[test]
    fn codefence_raw_html_uses_innerhtml_binding() {
        let output = render_solid(&raw_codefence_events());
        assert!(output.contains("<pre><code innerHTML={\"<p>  <b>x</b></p>\"} /></pre>"));
    }

    /// §6.3: a flag is a bare attribute, and a container that used to drop its
    /// attributes (`Paragraph`, `Blockquote`, `BulletList`, `OrderedList`) keeps
    /// them.
    #[test]
    fn attribute_flag_and_container_attrs_are_rendered() {
        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Paragraph),
            Event::Attribute {
                name: "class".to_string(),
                value: "x".to_string(),
            },
            Event::AttributeFlag {
                name: "isBar".to_string(),
            },
            Event::EndNode(NodeKind::Paragraph),
            Event::StartNode(NodeKind::Document),
        ];
        let output = render_solid(&events);
        assert!(
            output.contains("<p class=\"x\" isBar>"),
            "output = {output}"
        );
    }

    /// §11 rule 3: a `__`-prefixed key is an internal marker (`__plugin_kind`),
    /// never a real attribute. It must not reach the output even on a plain
    /// `Element` node, where the template path's `{...attrs}` filter does not
    /// apply.
    #[test]
    fn internal_marker_attributes_are_not_rendered() {
        let mut events = vec![Event::StartNode(NodeKind::Document)];
        events.extend(pendon_core::element_open("blockquote"));
        events.push(Event::Attribute {
            name: "__plugin_kind".to_string(),
            value: "element".to_string(),
        });
        events.push(Event::Attribute {
            name: "class".to_string(),
            value: "outer".to_string(),
        });
        events.push(Event::Text("body".to_string()));
        events.push(pendon_core::element_close("blockquote"));
        events.push(Event::EndNode(NodeKind::Document));
        let output = render_solid(&events);
        assert!(!output.contains("__plugin_kind"), "output = {output}");
        assert!(
            output.contains("<blockquote class=\"outer\">"),
            "output = {output}"
        );
    }

    /// §2.3 of RFC-0002: reference and citation payloads ride as module-level
    /// `export const` bindings, and both names are configurable.
    #[test]
    fn citation_data_is_exported_as_js_bindings() {
        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Paragraph),
            Event::Text("body".to_string()),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ];
        let hints = SolidRenderHints {
            references_export: Some(serde_json::json!({"book": {"title": "Book"}})),
            citations_export: Some(serde_json::json!([
                {"ref_id": "book", "citation_number": 1, "instance_index": 1}
            ])),
            references_export_name: Some("bibliographyData".to_string()),
            citations_export_name: Some("citationList".to_string()),
            ..Default::default()
        };
        let output = render_solid_with_hints(&events, Some(&hints));
        assert!(
            output.contains(r#"export const bibliographyData = {"book":{"title":"Book"}};"#),
            "output = {output}"
        );
        assert!(
            output.contains(
                r#"export const citationList = [{"citation_number":1,"instance_index":1,"ref_id":"book"}];"#
            ),
            "output = {output}"
        );
        let refs_at = output
            .find("export const bibliographyData")
            .expect("reference export");
        let citations_at = output
            .find("export const citationList")
            .expect("citation export");
        assert!(refs_at < citations_at, "output = {output}");
    }
}
