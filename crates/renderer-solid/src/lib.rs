use pendon_core::Event;
use pendon_renderer_ast::render_ast_to_string;
use serde_json::Value;
use std::collections::BTreeSet;

mod components;
mod imports;
mod metadata;
mod node;
mod resolve;
mod template;

pub use components::{layer_templates, ComponentSet, TypedComponent};
pub use imports::{ComponentTemplate, DataExport, ExportSource, ImportEntry, SolidRenderHints};

pub fn render_solid(events: &[Event]) -> String {
    render_solid_with_hints(events, None)
}

pub fn render_solid_with_hints(events: &[Event], hints: Option<&SolidRenderHints>) -> String {
    let ast_json = render_ast_to_string(events).expect("AST serialization failed");
    let v: Value = serde_json::from_str(&ast_json).expect("Invalid AST JSON");
    // §11 / ADR-0008 (`D-3a`): which entry of each layer answers each occurring
    // type is decided **once per document**, before a single node is rendered.
    // The plugins only tag their nodes (`__layer` + `type`); `?` / `_` / `*`
    // therefore expand against the document's real type set, and the imports of
    // an entry that lost — or whose layer never occurs — are never emitted.
    let resolution = resolve::Resolution::build(hints, &v);
    let ctx = resolve::Ctx {
        hints,
        resolution: &resolution,
    };
    let mut used_markers: BTreeSet<String> = BTreeSet::new();
    collect_used_markers(&v, &mut used_markers, hints);
    let mut body = String::new();
    node::render_node(&v, &mut body, &ctx);

    let mut out = String::new();
    for line in imports::normalize_imports(hints, resolution.used(), &used_markers) {
        out.push_str(&line);
        out.push('\n');
    }
    if !out.is_empty() {
        out.push('\n');
    }
    // ADR-0009 / §20: one writer, one ordered list. `frontmatter` and
    // `headings` are ordinary `Automatic` registrations and come first, so a
    // plugin registering the same name later overrides them (last wins).
    for export in &collect_exports(&v, hints) {
        push_data_export(&mut out, export);
    }
    out.push_str("export default function PendonView() { return (<>");
    out.push('\n');
    out.push_str(&body);
    out.push_str("\n</>); }\n");
    out
}

/// The document's `export const` registrations, in emission order (ADR-0009).
///
/// `frontmatter` and `headings` are derived from the AST here — first, as
/// `Automatic` — and every plugin registration follows in the order the CLI
/// pushed it. Two registrations of one name collapse to the **last** one,
/// value and position alike, so only one line is ever emitted.
fn collect_exports(v: &Value, hints: Option<&SolidRenderHints>) -> Vec<DataExport> {
    let mut registrations: Vec<DataExport> = Vec::new();
    if let Some(data) = metadata::extract_frontmatter(v) {
        registrations.push(DataExport::automatic("frontmatter", export_value(&data)));
    }
    if let Some(data) = metadata::extract_headings(v) {
        registrations.push(DataExport::automatic("headings", data));
    }
    if let Some(hints) = hints {
        registrations.extend(hints.exports.iter().cloned());
    }

    let mut out: Vec<DataExport> = Vec::with_capacity(registrations.len());
    for export in registrations {
        out.retain(|existing| existing.name != export.name);
        out.push(export);
    }
    out
}

/// A metadata `data` attribute is JSON text (§20); a value that is not valid
/// JSON is exported as a string so nothing is silently lost.
fn export_value(data: &str) -> Value {
    serde_json::from_str(data).unwrap_or_else(|_| Value::String(data.to_string()))
}

/// Emit `export const <name> = <json>;` — the only writer of a module-level
/// data binding.
///
/// `Value` always serialises, so the fallback is only a defensive guard.
fn push_data_export(out: &mut String, export: &DataExport) {
    out.push_str("export const ");
    out.push_str(&export.name);
    out.push_str(" = ");
    out.push_str(&serde_json::to_string(&export.value).unwrap_or_else(|_| "null".to_string()));
    out.push_str(";\n");
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

    /// A paragraph carrying one extra attribute, so a §6.3 mapping can be checked
    /// one position at a time.
    fn paragraph_with_attr(name: &str, value: &str) -> Vec<Event> {
        vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Paragraph),
            Event::Attribute {
                name: name.to_string(),
                value: value.to_string(),
            },
            Event::Text("body".to_string()),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ]
    }

    /// S2-3.4/§6.3: the universal specs positions are **DOM** attributes in the
    /// JSX output — `specs.class` is written `class`, never the dotted path.
    #[test]
    fn specs_class_renders_as_the_dom_class() {
        let output = render_solid(&paragraph_with_attr("specs.class", "lead big"));
        assert!(
            output.contains("<p class=\"lead big\">"),
            "output = {output}"
        );
    }

    /// S2-3.4/§6.3: `specs.id` is the DOM `id`.
    #[test]
    fn specs_id_renders_as_the_dom_id() {
        let output = render_solid(&paragraph_with_attr("specs.id", "p1"));
        assert!(output.contains("<p id=\"p1\">"), "output = {output}");
    }

    /// S2-3.4/§6.3: `specs.style` is the DOM `style`.
    #[test]
    fn specs_style_renders_as_the_dom_style() {
        let output = render_solid(&paragraph_with_attr("specs.style", "--tone: red"));
        assert!(
            output.contains("<p style=\"--tone: red\">"),
            "output = {output}"
        );
    }

    /// S2-3.4: `specs.tick` is **not** universal, so the field map's path is what
    /// reaches the DOM. `specs.tick="x"` is not JSX (a `.` is a member
    /// expression), so it is emitted as a one-key spread that keeps the name.
    #[test]
    fn specs_tick_keeps_its_dotted_name() {
        let output = render_solid(&paragraph_with_attr("specs.tick", "listing-1"));
        assert!(
            output.contains("<p {...{ \"specs.tick\": \"listing-1\" }}>"),
            "output = {output}"
        );
        assert!(!output.contains("specs.tick=\""), "output = {output}");
    }

    /// S2-3.4: `specs.quote` keeps its dotted path too, and a neighbouring
    /// universal position is still written as its DOM name.
    #[test]
    fn specs_quote_keeps_its_dotted_name() {
        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Paragraph),
            Event::Attribute {
                name: "specs.quote".to_string(),
                value: "T".to_string(),
            },
            Event::Attribute {
                name: "specs.class".to_string(),
                value: "lead".to_string(),
            },
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ];
        let output = render_solid(&events);
        assert!(
            output.contains("<p class=\"lead\" {...{ \"specs.quote\": \"T\" }}>"),
            "output = {output}"
        );
    }

    /// §6.3 / S2-3.6: a **component** receives a dotted field exactly the way a
    /// built-in element does. The `{...attrs}` spread used to write the path
    /// verbatim — `<NoteAside specs.tick="…">` — which is not JSX at all, because
    /// `a.b=` parses as a member expression.
    #[test]
    fn a_component_receives_a_dotted_field_as_a_one_key_spread() {
        let mut events = vec![Event::StartNode(NodeKind::Document)];
        events.extend(pendon_core::element_open("span"));
        events.push(Event::Attribute {
            name: "__layer".to_string(),
            value: "marker".to_string(),
        });
        events.push(Event::Attribute {
            name: "type".to_string(),
            value: "aside".to_string(),
        });
        events.push(Event::Attribute {
            name: "specs.tick".to_string(),
            value: "listing-1".to_string(),
        });
        events.push(pendon_core::element_close("span"));
        events.push(Event::EndNode(NodeKind::Document));

        let hints = SolidRenderHints {
            templates: vec![ComponentTemplate::new_layer(
                "marker",
                Vec::new(),
                "<NoteAside {...attrs} />",
                Vec::new(),
            )],
            ..SolidRenderHints::default()
        };
        let output = render_solid_with_hints(&events, Some(&hints));
        assert!(
            output.contains("<NoteAside {...{ \"specs.tick\": \"listing-1\" }} type=\"aside\" />"),
            "output = {output}"
        );
        assert!(!output.contains("specs.tick=\""), "output = {output}");
    }

    /// §6.1 / D-4: `field.override.<path>` renames an attribute **as it is
    /// written**, so a template reads the public name (`{attrs.ref_id}`) while the
    /// node keeps the internal one the processor produced. Before S2-4.4 the
    /// projection was always called with `None` on this path, which made an
    /// override a documented no-op.
    #[test]
    fn an_override_renames_a_component_prop() {
        let mut events = vec![Event::StartNode(NodeKind::Document)];
        events.extend(pendon_core::element_open("span"));
        events.push(Event::Attribute {
            name: "__layer".to_string(),
            value: "cite".to_string(),
        });
        events.push(Event::Attribute {
            name: "reference_id".to_string(),
            value: "smith".to_string(),
        });
        events.push(pendon_core::element_close("span"));
        events.push(Event::EndNode(NodeKind::Document));

        let mut field = pendon_extra::FieldConfig::new();
        field
            .add_override(
                pendon_extra::Path::from_dotted("reference_id").expect("valid path"),
                pendon_extra::Path::from_dotted("ref_id").expect("valid path"),
            )
            .expect("one override");
        let hints = SolidRenderHints {
            templates: vec![ComponentTemplate::new_layer(
                "cite",
                Vec::new(),
                "<Cite {...attrs} />",
                Vec::new(),
            )
            .with_field(field)],
            ..SolidRenderHints::default()
        };
        let output = render_solid_with_hints(&events, Some(&hints));
        assert!(
            output.contains(r#"<Cite ref_id="smith" />"#),
            "output = {output}"
        );
        assert!(!output.contains("reference_id"), "output = {output}");
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

    /// §2.3 of RFC-0002 + ADR-0009: reference and citation payloads ride as
    /// module-level `export const` bindings, both names are configurable, and
    /// references come first because a citation names one.
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
            exports: vec![
                DataExport::plugin(
                    "cite",
                    "bibliographyData",
                    serde_json::json!({"book": {"title": "Book"}}),
                ),
                DataExport::plugin(
                    "cite",
                    "citationList",
                    serde_json::json!([
                        {"reference_id": "book", "citation_number": 1}
                    ]),
                ),
            ],
            ..Default::default()
        };
        let output = render_solid_with_hints(&events, Some(&hints));
        assert!(
            output.contains(r#"export const bibliographyData = {"book":{"title":"Book"}};"#),
            "output = {output}"
        );
        assert!(
            output.contains(
                r#"export const citationList = [{"citation_number":1,"reference_id":"book"}];"#
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

    /// A document with a `Frontmatter` node, so the automatic registrations run.
    fn document_with_frontmatter(data: &str) -> Vec<Event> {
        vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Frontmatter),
            Event::Attribute {
                name: "data".to_string(),
                value: data.to_string(),
            },
            Event::EndNode(NodeKind::Frontmatter),
            Event::EndNode(NodeKind::Document),
        ]
    }

    /// ADR-0009 rule 3: `frontmatter`/`headings` are ordinary `Automatic`
    /// registrations, so they precede every plugin registration.
    #[test]
    fn automatic_exports_come_before_plugin_registrations() {
        let events = document_with_frontmatter(r#"{"title":"Demo"}"#);
        let hints = SolidRenderHints {
            exports: vec![DataExport::plugin(
                "bind",
                "rows",
                serde_json::json!([1, 2]),
            )],
            ..Default::default()
        };
        let output = render_solid_with_hints(&events, Some(&hints));
        let frontmatter_at = output
            .find("export const frontmatter")
            .expect("frontmatter export");
        let rows_at = output.find("export const rows").expect("rows export");
        assert!(frontmatter_at < rows_at, "output = {output}");
        assert!(
            output.contains(r#"export const frontmatter = {"title":"Demo"};"#),
            "output = {output}"
        );
    }

    /// ADR-0009 rule 4: the last registration of a name wins entirely — value
    /// and position — and only one line is emitted.
    #[test]
    fn the_last_registration_of_a_name_wins() {
        let events = document_with_frontmatter(r#"{"title":"Automatic"}"#);
        let hints = SolidRenderHints {
            exports: vec![
                DataExport::plugin("custom", "frontmatter", serde_json::json!({"a": 1})),
                DataExport::plugin("bind", "other", serde_json::json!(true)),
                DataExport::plugin("custom", "frontmatter", serde_json::json!({"b": 2})),
            ],
            ..Default::default()
        };
        let output = render_solid_with_hints(&events, Some(&hints));
        assert_eq!(
            output.matches("export const frontmatter").count(),
            1,
            "output = {output}"
        );
        assert!(
            output.contains(r#"export const frontmatter = {"b":2};"#),
            "output = {output}"
        );
        let other_at = output.find("export const other").expect("other export");
        let frontmatter_at = output
            .find("export const frontmatter")
            .expect("frontmatter export");
        assert!(
            other_at < frontmatter_at,
            "the last registration keeps its own position: {output}"
        );
        assert!(
            frontmatter_at < output.find("export default function").expect("body"),
            "output = {output}"
        );
    }

    /// ADR-0010/D-9: the `__number` transport attribute is stripped (every
    /// `__`-prefixed key is), while `attrs.number` reaches JSX as a real array.
    #[test]
    fn the_internal_number_marker_never_reaches_the_jsx() {
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

        let output = render_solid(&events);
        assert!(!output.contains("__number"), "output = {output}");
        assert!(
            output.contains("number={[\"1\", \"a\"]}"),
            "output = {output}"
        );
    }

    /// §6.3/ADR-0014 rule 2: `render_attrs` emits the node's **projection**, and
    /// the projection keeps a value's typed form — a `plugin-bind` value that
    /// resolved to `null` still renders `k={null}` and a number stays a number
    /// instead of being re-quoted.
    #[test]
    fn the_projection_keeps_attribute_types() {
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

        let output = render_solid(&events);
        assert!(output.contains("count={12}"), "output = {output}");
        assert!(output.contains("missing={null}"), "output = {output}");
    }

    /// The built-in `<h*>` fallback prints `attrs.number` like any other
    /// attribute: numbering is no longer heading-template-only, so a heading
    /// without a custom component would otherwise lose its number (ADR-0010).
    #[test]
    fn the_builtin_heading_fallback_keeps_the_number() {
        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Heading),
            Event::Attribute {
                name: "depth".to_string(),
                value: "2".to_string(),
            },
            Event::Attribute {
                name: "number".to_string(),
                value: format!("{}{}", pendon_core::JSON_ATTR_PREFIX, "[\"1\",\"a\"]"),
            },
            Event::Attribute {
                name: "slug".to_string(),
                value: "title".to_string(),
            },
            Event::Text("Title".to_string()),
            Event::EndNode(NodeKind::Heading),
            Event::EndNode(NodeKind::Document),
        ];

        let output = render_solid(&events);
        assert!(
            output.contains("<h2 number={[\"1\", \"a\"]}>"),
            "output = {output}"
        );
        assert!(!output.contains("slug=\"title\""), "output = {output}");
    }
}
