use pendon_core::Event;
use pendon_renderer_ast::render_ast_to_string;

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
    let v: serde_json::Value = serde_json::from_str(&ast_json).expect("Invalid AST JSON");
    let frontmatter = metadata::extract_frontmatter(&v);
    let headings = metadata::extract_headings(&v);
    let mut body = String::new();
    node::render_node(&v, &mut body, hints);

    let mut out = String::new();
    for line in imports::normalize_imports(hints) {
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
