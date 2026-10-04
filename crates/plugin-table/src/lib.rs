use pendon_core::InlinePipeline;
use pendon_core::{Event, Pipeline};
use serde::{Deserialize, Serialize};

mod attrs;
mod custom;
mod grid;
mod parser;
mod render;

pub use custom::solid_hints;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TableOptions {
    pub custom_node: Option<TableCustomNode>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TableCustomNode {
    pub table: Option<CustomComponent>,
    pub caption: Option<CustomComponent>,
    pub row: Option<CustomComponent>,
    pub cell: Option<CustomComponent>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CustomComponent {
    pub name: String,
    pub template: String,
    #[serde(default)]
    pub imports: Vec<CustomImport>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomImport {
    pub module: String,
    pub default: Option<String>,
    #[serde(default)]
    pub names: Vec<String>,
}

pub fn process(events: &[Event], options: &TableOptions, inline_pipeline: &Pipeline) -> Vec<Event> {
    let mut out: Vec<Event> = Vec::with_capacity(events.len());
    let mut i = 0usize;

    while i < events.len() {
        if let Some(table_block) = parser::try_parse_table_block(events, i) {
            if options.custom_node.is_some() {
                custom::emit_custom_table(&table_block, options, inline_pipeline, &mut out);
            } else {
                render::emit_table_elements(&table_block, inline_pipeline, &mut (), &mut out);
            }
            i = table_block.end_index + 1;
            continue;
        }

        out.push(events[i].clone());
        i += 1;
    }

    out
}

pub fn process_with_context<C, P>(
    events: &[Event],
    options: &TableOptions,
    inline_pipeline: &P,
    context: &mut C,
) -> Vec<Event>
where
    P: InlinePipeline<C>,
{
    let mut out = Vec::with_capacity(events.len());
    let mut i = 0usize;

    while i < events.len() {
        if let Some(table_block) = parser::try_parse_table_block(events, i) {
            if options.custom_node.is_some() {
                custom::emit_custom_table_with_context(
                    &table_block,
                    options,
                    inline_pipeline,
                    context,
                    &mut out,
                );
                i = table_block.end_index + 1;
                continue;
            }
            render::emit_table_elements(&table_block, inline_pipeline, context, &mut out);
            i = table_block.end_index + 1;
            continue;
        }
        out.push(events[i].clone());
        i += 1;
    }
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    use pendon_core::{ContextPipeline, NodeKind};

    fn paragraph_events(text: &str) -> Vec<Event> {
        vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Paragraph),
            Event::Text(text.to_string()),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ]
    }

    /// Concatenated text of a subtree; the markdown plugin emits many small
    /// chunks, so searching single `Text` events is not reliable.
    fn text_of(events: &[Event]) -> String {
        let mut text = String::new();
        for event in events {
            if let Event::Text(chunk) = event {
                text.push_str(chunk);
            }
        }
        text
    }

    /// Collects the children of every `tag` element in the stream.
    fn all_element_children(events: &[Event], tag: &str) -> Vec<Vec<Event>> {
        let mut result = Vec::new();
        let mut index = 0;

        while index < events.len() {
            let matches_tag = matches!(
                &events[index],
                Event::StartNode(NodeKind::Element(name)) if name == tag
            );
            if !matches_tag {
                index += 1;
                continue;
            }

            let mut depth = 0usize;
            let mut children = Vec::new();
            let mut cursor = index + 1;
            while cursor < events.len() {
                match &events[cursor] {
                    Event::StartNode(NodeKind::Element(_)) => {
                        depth += 1;
                        children.push(events[cursor].clone());
                    }
                    Event::EndNode(NodeKind::Element(_)) if depth == 0 => break,
                    Event::EndNode(NodeKind::Element(_)) => {
                        depth -= 1;
                        children.push(events[cursor].clone());
                    }
                    other => children.push(other.clone()),
                }
                cursor += 1;
            }

            result.push(children);
            index = cursor + 1;
        }

        result
    }

    fn has_custom_component(events: &[Event], name: &str) -> bool {
        events
            .iter()
            .any(|event| matches!(event, Event::StartNode(NodeKind::Custom(node)) if node == name))
    }

    fn has_attribute(events: &[Event], name: &str, value: &str) -> bool {
        events.iter().any(|event| {
            matches!(event, Event::Attribute { name: n, value: v } if n == name && v == value)
        })
    }

    /// Stand-in for an inline custom plugin (like `cite`): rewrites `{{cite}}`
    /// text into a custom component node.
    fn custom_component_pipeline() -> ContextPipeline<()> {
        let mut pipeline = ContextPipeline::new();
        pipeline.add(|_: &mut (), events: Vec<Event>| {
            let mut out = Vec::new();
            for event in events {
                match event {
                    Event::Text(text) => {
                        for (index, part) in text.split("{{cite}}").enumerate() {
                            if index > 0 {
                                out.push(Event::StartNode(NodeKind::Custom("Cite".into())));
                                out.push(Event::Attribute {
                                    name: "id".into(),
                                    value: "book".into(),
                                });
                                out.push(Event::EndNode(NodeKind::Custom("Cite".into())));
                            }
                            if !part.is_empty() {
                                out.push(Event::Text(part.to_string()));
                            }
                        }
                    }
                    other => out.push(other),
                }
            }
            out
        });
        pipeline
    }

    #[test]
    fn emits_structured_elements_instead_of_html_strings() {
        let pipeline = Pipeline::default();
        let events = paragraph_events("| Produk | Stok |\n| --- | --- |\n| Laptop | 15 |\n");
        let out = process(&events, &TableOptions::default(), &pipeline);

        for tag in ["table", "thead", "tbody", "tr", "th", "td"] {
            assert!(
                out.iter().any(
                    |event| matches!(event, Event::StartNode(NodeKind::Element(name)) if name == tag)
                ),
                "missing <{tag}> element"
            );
        }

        // The table is no longer emitted as one raw HTML string.
        assert!(!text_of(&out).contains("<table"));
        assert!(!text_of(&out).contains("</td>"));

        let cells = all_element_children(&out, "td");
        assert_eq!(cells.len(), 2);
        assert!(text_of(&cells[0]).contains("Laptop"));
    }

    #[test]
    fn nests_custom_components_inside_table_cells() {
        let pipeline = custom_component_pipeline();
        let events =
            paragraph_events("| Head {{cite}} | B |\n| --- | --- |\n| {{cite}} cell | x |\n");
        let out = process_with_context(&events, &TableOptions::default(), &pipeline, &mut ());

        let headers = all_element_children(&out, "th");
        assert_eq!(headers.len(), 2);
        assert!(
            has_custom_component(&headers[0], "Cite"),
            "custom component must stay inside <th>"
        );

        let cells = all_element_children(&out, "td");
        assert_eq!(cells.len(), 2);
        assert!(
            has_custom_component(&cells[0], "Cite"),
            "custom component must stay inside <td>"
        );
        assert!(has_attribute(&cells[0], "id", "book"));
        assert!(text_of(&cells[0]).contains("cell"));
    }

    fn list_items(children: &[Event]) -> usize {
        children
            .iter()
            .filter(|event| matches!(event, Event::StartNode(NodeKind::ListItem)))
            .count()
    }

    /// The `\n` escape makes block content such as a multi-item list expressible
    /// inside a single table cell.
    #[test]
    fn escaped_newlines_in_cells_render_lists() {
        let pipeline = Pipeline::default();
        let events = paragraph_events(
            "| Bullet | Ordered |\n| --- | --- |\n| - Alpha\\n- Beta | 1. One\\n2. Two |\n",
        );
        let out = process(&events, &TableOptions::default(), &pipeline);

        let cells = all_element_children(&out, "td");
        assert_eq!(cells.len(), 2);
        assert!(
            cells[0]
                .iter()
                .any(|event| matches!(event, Event::StartNode(NodeKind::BulletList))),
            "bullet list expected in the first cell: {:?}",
            cells[0]
        );
        assert!(
            cells[1]
                .iter()
                .any(|event| matches!(event, Event::StartNode(NodeKind::OrderedList))),
            "ordered list expected in the second cell: {:?}",
            cells[1]
        );
        assert_eq!(list_items(&cells[0]), 2);
        assert_eq!(list_items(&cells[1]), 2);
        // The escape itself must never reach the output.
        assert!(!text_of(&cells[0]).contains("\\n"));
    }
}
