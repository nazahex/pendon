use pendon_core::Event;
use serde_json;

mod builder;
use builder::build_ast_document;

pub fn render_ast_to_string(events: &[Event]) -> Result<String, serde_json::Error> {
    serde_json::to_string(&build_ast_document(events))
}

pub fn render_ast_to_string_pretty(events: &[Event]) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&build_ast_document(events))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pendon_core::{Event, NodeKind};
    use serde_json::Value;

    fn html_events(kind: NodeKind, text: &str) -> Vec<Event> {
        vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(kind.clone()),
            Event::Text(text.to_string()),
            Event::EndNode(kind),
            Event::EndNode(NodeKind::Document),
        ]
    }

    #[test]
    fn html_block_appears_in_ast() {
        let events = html_events(NodeKind::HtmlBlock, "<div>raw</div>");
        let output = render_ast_to_string(&events).unwrap();
        let parsed: Value = serde_json::from_str(&output).unwrap();
        let first_child = parsed["children"][0].clone();
        assert_eq!(first_child["type"], "HtmlBlock");
        assert_eq!(first_child["text"], "<div>raw</div>");
    }

    #[test]
    fn html_inline_roundtrips() {
        let mut events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Paragraph),
            Event::StartNode(NodeKind::HtmlInline),
            Event::Text("<span>ok</span>".to_string()),
            Event::EndNode(NodeKind::HtmlInline),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ];
        let pretty = render_ast_to_string_pretty(&events).unwrap();
        assert!(pretty.contains("HtmlInline"));

        events.push(Event::Text("ignored".to_string()));
        // ensure regular rendering still succeeds even with trailing text
        assert!(render_ast_to_string(&events).is_ok());
    }
}
