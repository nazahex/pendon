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

    #[test]
    fn list_item_keeps_nested_list_after_inline_component() {
        let component = NodeKind::Custom("Epis".to_string());
        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::BulletList),
            Event::StartNode(NodeKind::ListItem),
            Event::StartNode(component.clone()),
            Event::Attribute {
                name: "__plugin_kind".to_string(),
                value: "inline".to_string(),
            },
            Event::Text("Aliquip commodo commodo.".to_string()),
            Event::EndNode(component),
            Event::StartNode(NodeKind::BulletList),
            Event::StartNode(NodeKind::ListItem),
            Event::Text("Nested satu".to_string()),
            Event::EndNode(NodeKind::ListItem),
            Event::EndNode(NodeKind::BulletList),
            Event::EndNode(NodeKind::ListItem),
            Event::EndNode(NodeKind::BulletList),
            Event::EndNode(NodeKind::Document),
        ];

        let output = render_ast_to_string(&events).unwrap();
        let parsed: Value = serde_json::from_str(&output).unwrap();
        let outer_item = &parsed["children"][0]["children"][0];

        assert_eq!(outer_item["type"], "ListItem");
        // The item must keep its children instead of collapsing them into `text`.
        assert!(outer_item.get("text").is_none(), "{outer_item}");
        let kinds: Vec<&str> = outer_item["children"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["type"].as_str().unwrap())
            .collect();
        assert_eq!(kinds, vec!["Epis", "BulletList"], "{outer_item}");
        assert_eq!(
            outer_item["children"][1]["children"][0]["text"],
            "Nested satu"
        );
    }

    #[test]
    fn block_custom_component_still_flattens_into_item_text() {
        let component = NodeKind::Custom("Parego".to_string());
        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::BulletList),
            Event::StartNode(NodeKind::ListItem),
            Event::StartNode(component.clone()),
            Event::Attribute {
                name: "__plugin_kind".to_string(),
                value: "block".to_string(),
            },
            Event::Text("Block body".to_string()),
            Event::EndNode(component),
            Event::EndNode(NodeKind::ListItem),
            Event::EndNode(NodeKind::BulletList),
            Event::EndNode(NodeKind::Document),
        ];

        let output = render_ast_to_string(&events).unwrap();
        let parsed: Value = serde_json::from_str(&output).unwrap();
        let outer_item = &parsed["children"][0]["children"][0];

        // Block components keep the historical plain-text item shape.
        assert_eq!(outer_item["type"], "ListItem");
        assert_eq!(outer_item["text"], "Block body");
    }

    #[test]
    fn list_item_keeps_html_inline_in_order() {
        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::OrderedList),
            Event::StartNode(NodeKind::ListItem),
            Event::Text("Menekan tombol ".to_string()),
            Event::StartNode(NodeKind::HtmlInline),
            Event::Text("<kbd>".to_string()),
            Event::EndNode(NodeKind::HtmlInline),
            Event::Text("Tab".to_string()),
            Event::StartNode(NodeKind::HtmlInline),
            Event::Text("</kbd>".to_string()),
            Event::EndNode(NodeKind::HtmlInline),
            Event::Text(" pada keyboard.".to_string()),
            Event::EndNode(NodeKind::ListItem),
            Event::EndNode(NodeKind::OrderedList),
            Event::EndNode(NodeKind::Document),
        ];

        let output = render_ast_to_string(&events).unwrap();
        let parsed: Value = serde_json::from_str(&output).unwrap();
        let item = &parsed["children"][0]["children"][0];

        assert!(item.get("text").is_none());
        assert_eq!(item["children"][0]["type"], "Text");
        assert_eq!(item["children"][0]["text"], "Menekan tombol ");
        assert_eq!(item["children"][1]["type"], "HtmlInline");
        assert_eq!(item["children"][1]["text"], "<kbd>");
        assert_eq!(item["children"][2]["type"], "Text");
        assert_eq!(item["children"][2]["text"], "Tab");
        assert_eq!(item["children"][3]["type"], "HtmlInline");
        assert_eq!(item["children"][3]["text"], "</kbd>");
        assert_eq!(item["children"][4]["type"], "Text");
        assert_eq!(item["children"][4]["text"], " pada keyboard.");
    }

    /// §16 fixture 17/18: a bare flag (§6.3) is JSON `true`, and the containers
    /// that historically dropped attributes (`Paragraph`, `Blockquote`,
    /// `BulletList`, `OrderedList`) keep them in the AST that the HTML and JSON
    /// renderers consume.
    #[test]
    fn flags_and_container_attrs_survive_into_ast() {
        for kind in [
            NodeKind::Paragraph,
            NodeKind::Blockquote,
            NodeKind::BulletList,
            NodeKind::OrderedList,
        ] {
            let events = vec![
                Event::StartNode(NodeKind::Document),
                Event::StartNode(kind.clone()),
                Event::Attribute {
                    name: "data-role".to_string(),
                    value: "note".to_string(),
                },
                Event::AttributeFlag {
                    name: "isBar".to_string(),
                },
                Event::EndNode(kind.clone()),
                Event::EndNode(NodeKind::Document),
            ];

            let output = render_ast_to_string(&events).unwrap();
            let parsed: Value = serde_json::from_str(&output).unwrap();
            let node = &parsed["children"][0];
            let label = format!("{kind:?}");

            assert_eq!(node["type"], label, "node type: {output}");
            assert_eq!(node["attrs"]["data-role"], "note", "{label}: {output}");
            assert_eq!(node["attrs"]["isBar"], true, "{label}: {output}");
        }
    }
}
