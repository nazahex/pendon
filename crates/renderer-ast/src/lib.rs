use pendon_core::Event;
use serde_json::{json, Value};

mod builder;
use builder::build_ast_document;

/// Reserved key of an AST attribute value that carries a **rendered fragment**.
///
/// A fragment is a node list produced from a nested Pendon Markdown payload
/// (`{{{mdp[body] … }}}` in `plugin-bind`, §19): markup, not data. It rides an
/// attribute as `{"$fragment": [<node>, …]}`, so every renderer owns its own
/// format — `renderer-solid` emits `name={(<>…</>)}`, `renderer-html` inlines
/// the HTML into the attribute, and `renderer-ast` / `renderer-json` keep the
/// node list as structured JSON.
///
/// The key is reserved: an object with exactly this one key *is* a fragment.
pub const FRAGMENT_KEY: &str = "$fragment";

/// Wraps a node list as a fragment attribute value (see [`FRAGMENT_KEY`]).
pub fn fragment(nodes: Vec<Value>) -> Value {
    json!({ FRAGMENT_KEY: nodes })
}

/// The node list of a fragment attribute value, or `None` when `value` is an
/// ordinary object/array. Requires the [`FRAGMENT_KEY`] to be the only key, so
/// authored data (`{"$fragment": …, "other": …}`) is never mistaken for markup.
pub fn fragment_nodes(value: &Value) -> Option<&[Value]> {
    let map = value.as_object()?;
    if map.len() != 1 {
        return None;
    }
    map.get(FRAGMENT_KEY)?.as_array().map(Vec::as_slice)
}

pub fn render_ast_to_string(events: &[Event]) -> Result<String, serde_json::Error> {
    serde_json::to_string(&build_ast_document(events))
}

pub fn render_ast_to_string_pretty(events: &[Event]) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&build_ast_document(events))
}

/// The `children` of the AST a stream builds — the node list itself, without the
/// `Document` wrapper or the document-level `diagnostics`.
///
/// This is the seam a nested payload uses: the events are already a complete
/// sub-document, and only their nodes belong inside the host tree.
pub fn events_to_children(events: &[Event]) -> Result<Vec<Value>, serde_json::Error> {
    let document = serde_json::to_value(build_ast_document(events))?;
    Ok(match document.get("children").and_then(Value::as_array) {
        Some(children) => children.clone(),
        None => Vec::new(),
    })
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

    /// A data-binding transform transports a structured value as
    /// `JSON_ATTR_PREFIX` + compact JSON (`docs/rfc/plugin-bind.md`). The AST
    /// builder re-hydrates it into a real JSON value so the Solid spread path
    /// can emit `name={…}`; a plain string attribute is unchanged.
    #[test]
    fn json_attr_sentinel_rehydrates_to_a_real_value() {
        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Custom("Card".to_string())),
            Event::Attribute {
                name: "cfg".to_string(),
                value: format!("{}{{\"a\":1,\"b\":[2,3]}}", pendon_core::JSON_ATTR_PREFIX),
            },
            Event::Attribute {
                name: "label".to_string(),
                value: "plain".to_string(),
            },
            Event::EndNode(NodeKind::Custom("Card".to_string())),
            Event::EndNode(NodeKind::Document),
        ];

        let output = render_ast_to_string(&events).unwrap();
        let parsed: Value = serde_json::from_str(&output).unwrap();
        let node = &parsed["children"][0];

        assert_eq!(
            node["attrs"]["cfg"],
            serde_json::json!({"a": 1, "b": [2, 3]})
        );
        assert_eq!(node["attrs"]["label"], "plain");
    }

    /// A malformed JSON payload degrades to the raw string rather than dropping
    /// the attribute (§4.3-style resilience).
    #[test]
    fn a_malformed_json_attr_falls_back_to_string() {
        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Custom("Card".to_string())),
            Event::Attribute {
                name: "cfg".to_string(),
                value: format!("{}not-json", pendon_core::JSON_ATTR_PREFIX),
            },
            Event::EndNode(NodeKind::Custom("Card".to_string())),
            Event::EndNode(NodeKind::Document),
        ];

        let output = render_ast_to_string(&events).unwrap();
        let parsed: Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["children"][0]["attrs"]["cfg"], "not-json");
    }

    /// A `$fragment` value is a node list, not data: it survives into the AST as
    /// structured JSON (the host of every renderer-specific encoding).
    #[test]
    fn a_fragment_rehydrates_into_a_node_list() {
        let fragment = fragment(vec![serde_json::json!({
            "type": "Paragraph",
            "children": [{ "type": "Text", "text": "hi" }]
        })]);
        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Custom("Card".to_string())),
            Event::Attribute {
                name: "body".to_string(),
                value: format!("{}{}", pendon_core::JSON_ATTR_PREFIX, fragment),
            },
            Event::EndNode(NodeKind::Custom("Card".to_string())),
            Event::EndNode(NodeKind::Document),
        ];

        let output = render_ast_to_string(&events).unwrap();
        let parsed: Value = serde_json::from_str(&output).unwrap();
        let body = &parsed["children"][0]["attrs"]["body"];

        assert_eq!(body[FRAGMENT_KEY][0]["type"], "Paragraph");
        let nodes = fragment_nodes(body).expect("fragment nodes");
        assert_eq!(nodes.len(), 1);
    }

    /// Only an object whose *single* key is `$fragment` is markup; authored data
    /// that happens to carry the key stays data.
    #[test]
    fn a_fragment_requires_the_reserved_key_to_be_alone() {
        assert!(fragment_nodes(&serde_json::json!({ "$fragment": [] })).is_some());
        assert!(fragment_nodes(&serde_json::json!({ "$fragment": [], "n": 1 })).is_none());
        assert!(fragment_nodes(&serde_json::json!({ "n": 1 })).is_none());
        assert!(fragment_nodes(&serde_json::json!([])).is_none());
    }

    /// The seam a nested payload uses: the `Document` wrapper and the
    /// diagnostics stay behind, only the nodes are embedded.
    #[test]
    fn events_to_children_drops_the_document_wrapper_and_diagnostics() {
        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::Diagnostic {
                severity: pendon_core::Severity::Warning,
                message: "careful".to_string(),
                span: None,
            },
            Event::StartNode(NodeKind::Paragraph),
            Event::Text("body".to_string()),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ];

        let children = events_to_children(&events).unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0]["type"], "Paragraph");
    }

    /// ADR-0010/D-9: `__number` is an internal transport attribute. The AST
    /// builder drops it, so neither the AST nor the JSON renderer (which shares
    /// this builder) can leak it; `number` itself stays a real JS array.
    #[test]
    fn the_internal_number_marker_never_reaches_the_ast() {
        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Heading),
            Event::Attribute {
                name: "level".to_string(),
                value: "2".to_string(),
            },
            Event::Attribute {
                name: "__number".to_string(),
                value: "{\"depth\":1,\"type\":\"numberB\"}".to_string(),
            },
            Event::Attribute {
                name: "number".to_string(),
                value: format!("{}{}", pendon_core::JSON_ATTR_PREFIX, "[\"1\",\"a\"]"),
            },
            Event::Text("Title".to_string()),
            Event::EndNode(NodeKind::Heading),
            Event::EndNode(NodeKind::Document),
        ];

        let output = render_ast_to_string(&events).unwrap();
        assert!(!output.contains("__number"), "{output}");
        let parsed: Value = serde_json::from_str(&output).unwrap();
        assert_eq!(
            parsed["children"][0]["attrs"]["number"],
            serde_json::json!(["1", "a"])
        );
    }
}
