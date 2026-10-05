mod json;

pub use json::render_to_string;

#[cfg(test)]
mod tests {
    use super::*;
    use pendon_core::{Event, NodeKind};
    use serde_json::Value;

    /// §16 fixture 17/18 through the JSON renderer: a bare flag (§6.3) becomes
    /// JSON `true` and container attributes are not dropped.
    #[test]
    fn flags_and_container_attrs_reach_json() {
        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Blockquote),
            Event::Attribute {
                name: "class".to_string(),
                value: "x".to_string(),
            },
            Event::AttributeFlag {
                name: "isBar".to_string(),
            },
            Event::Text("quoted".to_string()),
            Event::EndNode(NodeKind::Blockquote),
            Event::EndNode(NodeKind::Document),
        ];

        let output = render_to_string(&events).unwrap();
        let parsed: Value = serde_json::from_str(&output).unwrap();
        let node = &parsed["children"][0];

        assert_eq!(node["type"], "Blockquote");
        assert_eq!(node["attrs"]["class"], "x", "{output}");
        assert_eq!(node["attrs"]["isBar"], true, "{output}");
    }
}
