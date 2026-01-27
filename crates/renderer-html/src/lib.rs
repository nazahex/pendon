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
}
