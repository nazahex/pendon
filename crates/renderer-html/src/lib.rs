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
    fn escapes_literal_braces_as_entities() {
        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Paragraph),
            Event::Text("a {foo} b".to_string()),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ];
        let output = render_html(&events);
        assert!(output.contains("&#123;foo&#125;"), "output = {output}");
        assert!(!output.contains("{foo}"), "output = {output}");
    }

    #[test]
    fn leaves_raw_html_braces_untouched() {
        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Paragraph),
            Event::StartNode(NodeKind::HtmlInline),
            Event::Text("<span innerHTML={`{literal}`}></span>".to_string()),
            Event::EndNode(NodeKind::HtmlInline),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ];
        let output = render_html(&events);
        assert!(
            output.contains("<span innerHTML={`{literal}`}></span>"),
            "output = {output}"
        );
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

    /// §6.3: flags are bare attributes and container attributes (`class`, …)
    /// survive into HTML in both modes.
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
            Event::EndNode(NodeKind::Document),
        ];
        let compact = render_html(&events);
        assert!(
            compact.contains("<p class=\"x\" isBar>"),
            "compact = {compact}"
        );
        let pretty = render_html_pretty(&events);
        assert!(
            pretty.contains("<p class=\"x\" isBar>"),
            "pretty = {pretty}"
        );
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

    /// A link's own attributes go through the same renderer as every other
    /// node's, so a fragment (`$fragment`, a bound `mdp` payload) is markup
    /// here too — escaped into the attribute value, never the raw AST JSON.
    fn link_with_fragment_events() -> Vec<Event> {
        let fragment = pendon_renderer_ast::fragment(vec![serde_json::json!({
            "type": "Paragraph",
            "children": [{ "type": "Text", "text": "hi" }],
        })]);
        let encoded = format!(
            "{}{}",
            pendon_core::JSON_ATTR_PREFIX,
            serde_json::to_string(&fragment).expect("fragment json")
        );
        vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Paragraph),
            Event::StartNode(NodeKind::Link),
            Event::Attribute {
                name: "href".to_string(),
                value: "/docs".to_string(),
            },
            Event::Attribute {
                name: "body".to_string(),
                value: encoded,
            },
            Event::Text("Card".to_string()),
            Event::EndNode(NodeKind::Link),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ]
    }

    #[test]
    fn link_fragment_attribute_is_rendered_as_escaped_markup() {
        for output in [
            render_html(&link_with_fragment_events()),
            render_html_pretty(&link_with_fragment_events()),
        ] {
            assert!(
                output.contains("body=\"&lt;p&gt;hi&lt;/p&gt;"),
                "output = {output}"
            );
            assert!(!output.contains("$fragment"), "output = {output}");
        }
    }

    /// ADR-0010/D-9: the `__number` transport attribute is stripped, and the
    /// numbering reaches HTML as its JSON text (the §6.3 structured fallback).
    #[test]
    fn the_internal_number_marker_never_reaches_the_html() {
        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Paragraph),
            Event::Attribute {
                name: "__number".to_string(),
                value: "{\"depth\":1,\"type\":\"numberB\"}".to_string(),
            },
            Event::Attribute {
                name: "number".to_string(),
                value: format!("{}{}", pendon_core::JSON_ATTR_PREFIX, "[\"1\",\"a\"]"),
            },
            Event::Text("Body".to_string()),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ];

        let output = render_html(&events);
        assert!(!output.contains("__number"), "output = {output}");
        assert!(output.contains("number="), "output = {output}");
    }

    /// §11 rule 3: `__layer` (and every other `__`-prefixed key) is an internal
    /// marker. `Link` writes its own attributes, so it needs the same skip as
    /// `render_attrs_except` — otherwise an anchor would leak its routing layer.
    #[test]
    fn a_links_internal_marker_never_reaches_the_html() {
        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Paragraph),
            Event::StartNode(NodeKind::Link),
            Event::Attribute {
                name: "href".to_string(),
                value: "/foo".to_string(),
            },
            Event::Attribute {
                name: "__layer".to_string(),
                value: "anchor".to_string(),
            },
            Event::Attribute {
                name: "rel".to_string(),
                value: "noopener".to_string(),
            },
            Event::Text("link".to_string()),
            Event::EndNode(NodeKind::Link),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ];

        for output in [render_html(&events), render_html_pretty(&events)] {
            assert!(!output.contains("__layer"), "output = {output}");
            assert!(output.contains("href=\"/foo\""), "output = {output}");
            assert!(output.contains("rel=\"noopener\""), "output = {output}");
        }
    }

    /// ADR-0014 rule 2: attributes come from the node's **projection**, which
    /// keeps a value's typed form. HTML stays stringly-typed by design (§6.3),
    /// so a bound number and a bound `null` print their literal text in both
    /// modes — the projection must not drop or rename them on the way.
    #[test]
    fn the_projection_keeps_every_typed_attribute() {
        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Paragraph),
            Event::Attribute {
                name: "count".to_string(),
                value: format!("{}{}", pendon_core::JSON_ATTR_PREFIX, "12"),
            },
            Event::Attribute {
                name: "missing".to_string(),
                value: format!("{}{}", pendon_core::JSON_ATTR_PREFIX, "null"),
            },
            Event::Text("Body".to_string()),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ];

        for output in [render_html(&events), render_html_pretty(&events)] {
            assert!(output.contains("count=\"12\""), "output = {output}");
            assert!(output.contains("missing=\"null\""), "output = {output}");
        }
    }
}
