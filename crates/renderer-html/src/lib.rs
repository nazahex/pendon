use pendon_core::Event;
use pendon_renderer_ast::render_ast_to_string;
use serde_json::Value;

mod compact;
mod pretty;
mod utils;

pub use compact::render_html;
pub use pretty::render_html_pretty;

pub(crate) fn events_to_ast_value(events: &[Event]) -> Value {
    let ast_json = render_ast_to_string(events).expect("AST serialization failed");
    serde_json::from_str(&ast_json).expect("Invalid AST JSON")
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

    fn link_with_title_events() -> Vec<Event> {
        vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Paragraph),
            Event::StartNode(NodeKind::Link),
            Event::Attribute {
                name: "href".to_string(),
                value: "https://example.com".to_string(),
            },
            Event::Attribute {
                name: "title".to_string(),
                value: "Baz Wax".to_string(),
            },
            Event::Text("foo".to_string()),
            Event::EndNode(NodeKind::Link),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ]
    }

    #[test]
    fn html_block_passes_through_in_compact_mode() {
        let output = render_html(&html_block_events());
        assert!(output.contains("<div>raw</div>"), "output = {output}");
    }

    #[test]
    fn html_inline_passes_through_in_pretty_mode() {
        let output = render_html_pretty(&html_inline_events());
        assert!(output.contains("<span>inline</span>"), "output = {output}");
    }

    #[test]
    fn link_title_is_rendered_in_html_modes() {
        let compact = render_html(&link_with_title_events());
        assert!(
            compact.contains("<a href=\"https://example.com\" title=\"Baz Wax\">"),
            "compact output = {compact}"
        );

        let pretty = render_html_pretty(&link_with_title_events());
        assert!(
            pretty.contains("<a href=\"https://example.com\" title=\"Baz Wax\">"),
            "pretty output = {pretty}"
        );
    }
}
