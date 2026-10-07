use pendon_core::InlinePipeline;
use pendon_core::{Event, Pipeline};
use pendon_renderer_solid::{ComponentSet, ImportEntry};
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
    /// Imports applied to every configured component below. Each component's
    /// own `imports` list is added on top; the renderer deduplicates.
    #[serde(default)]
    pub imports: Vec<ImportEntry>,
    /// §11 component sets, one per layer (`§8` layer mapping).
    #[serde(default)]
    pub table: ComponentSet<CustomComponent>,
    #[serde(default)]
    pub caption: ComponentSet<CustomComponent>,
    #[serde(default)]
    pub thead: ComponentSet<CustomComponent>,
    #[serde(default)]
    pub tbody: ComponentSet<CustomComponent>,
    #[serde(default)]
    pub tfoot: ComponentSet<CustomComponent>,
    #[serde(default)]
    pub row: ComponentSet<CustomComponent>,
    #[serde(default)]
    pub cell: ComponentSet<CustomComponent>,
}

impl TableCustomNode {
    /// Whether any layer has a component to render with.
    pub fn is_configured(&self) -> bool {
        [
            &self.table,
            &self.caption,
            &self.thead,
            &self.tbody,
            &self.tfoot,
            &self.row,
            &self.cell,
        ]
        .iter()
        .any(|set| !set.is_empty())
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CustomComponent {
    pub name: String,
    pub template: String,
    #[serde(default)]
    pub imports: Vec<ImportEntry>,
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
    use pendon_renderer_solid::TypedComponent;

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

    /// §8: the declaration line, the `|| caption ||` line and the cell/row/section
    /// extras each land on their own layer; bare `===` is retired.
    #[test]
    fn section_eight_layers_land_on_their_elements() {
        let pipeline = Pipeline::default();
        let events = paragraph_events(
            "|-[sales](\"Laporan\")@@tableX{.striped}-|\n\
             ||@@captionX{.cap} Caption||\n\
             | A | B |\n\
             | :---@@cellA{.v-top} | :--- |\n\
             |@@cellB{.lead} 1 | 2 |@@rowB{.info}\n\
             |===|@@tfootX{.total}\n\
             | Total | > |\n",
        );
        let out = process(&events, &TableOptions::default(), &pipeline);

        // Declaration line → `<table>`; caption line → `<caption>`.
        assert!(has_attribute(&out, "id", "sales"));
        assert!(has_attribute(&out, "title", "Laporan"));
        assert!(has_attribute(&out, "type", "tableX"));
        assert!(has_attribute(&out, "class", "striped"));
        let caption = all_element_children(&out, "caption");
        assert!(has_attribute(&caption[0], "class", "cap"), "{caption:?}");
        assert!(has_attribute(&caption[0], "type", "captionX"));

        // Cell-front extras → the `<th>` of that column (and its `<td>`s), row
        // end-of-line extras → the `<tr>`, `|===|` extras → the `<tfoot>`.
        assert!(has_attribute(&out, "type", "cellA"));
        assert!(has_attribute(&out, "class", "v-top"));
        assert!(has_attribute(&out, "type", "cellB"));
        // §6.4: the column class and the cell class accumulate.
        assert!(has_attribute(&out, "class", "v-top lead"));
        assert!(has_attribute(&out, "type", "rowB"));
        assert!(has_attribute(&out, "class", "info"));
        let tfoot = all_element_children(&out, "tfoot");
        assert!(has_attribute(&tfoot[0], "class", "total"), "{tfoot:?}");
        assert!(has_attribute(&tfoot[0], "type", "tfootX"));

        // A bare `===` is no longer a footer separator (§8).
        let events = paragraph_events("| A |\n| --- |\n| 1 |\n===\n| 2 |\n");
        let out = process(&events, &TableOptions::default(), &pipeline);
        assert!(all_element_children(&out, "tfoot").is_empty());
    }

    /// §8: extras in front of a `>`/`^` marker decorate the cell they merge into.
    #[test]
    fn cell_extras_before_a_span_marker_merge_into_the_spanned_cell() {
        let pipeline = Pipeline::default();
        let events = paragraph_events("| A | B |\n| --- | --- |\n| 1 |@@cellA{.wide} >|\n");
        let out = process(&events, &TableOptions::default(), &pipeline);

        assert!(has_attribute(&out, "colspan", "2"));
        assert!(has_attribute(&out, "type", "cellA"));
        assert!(has_attribute(&out, "class", "wide"));
    }

    /// §8: a taxonomy that the lexer turned into a `Link` — the declaration head
    /// `[slug]("A longer title")` — is rebuilt before the table is parsed.
    #[test]
    fn declaration_head_lexed_as_a_link_still_parses() {
        let pipeline = Pipeline::default();
        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Paragraph),
            Event::Text("|-".to_string()),
            Event::StartNode(NodeKind::Link),
            Event::Attribute {
                name: "href".to_string(),
                value: "\"Laporan Penjualan 2026\"".to_string(),
            },
            Event::Text("sales".to_string()),
            Event::EndNode(NodeKind::Link),
            Event::Text("@@tableX{.striped}-|\n| A |\n| --- |\n| 1 |\n".to_string()),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ];
        let out = process(&events, &TableOptions::default(), &pipeline);

        assert!(has_attribute(&out, "id", "sales"));
        assert!(has_attribute(&out, "title", "Laporan Penjualan 2026"));
        assert!(text_of(&out).contains('1'));
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

    // --- custom node: layers, fallbacks and imports ---

    /// A component template that only wraps its children.
    fn component(name: &str) -> CustomComponent {
        CustomComponent {
            name: name.to_string(),
            template: format!("<{name}>{{children}}</{name}>"),
            imports: Vec::new(),
        }
    }

    /// A one-component layer set (its default entry).
    fn layer(name: &str) -> ComponentSet<CustomComponent> {
        ComponentSet::from_entries([TypedComponent::default_component(component(name))])
    }

    /// Every layer of the table is customised.
    fn fully_custom_options() -> TableOptions {
        TableOptions {
            custom_node: Some(TableCustomNode {
                imports: Vec::new(),
                table: layer("CustomTable"),
                caption: layer("TableCaption"),
                thead: layer("TableHead"),
                tbody: layer("TableBody"),
                tfoot: layer("TableFoot"),
                row: layer("TableRow"),
                cell: layer("TableCell"),
            }),
        }
    }

    /// Caption, header, body and footer all present, so every configured layer
    /// is exercised by a single input.
    const CUSTOM_TABLE_INPUT: &str =
        "|| Judul Tabel ||\n| Produk | Stok |\n| --- | --- |\n| Laptop | 15 |\n|===|\n| Total | 15 |\n";

    #[test]
    fn custom_node_replaces_every_table_layer() {
        let pipeline = Pipeline::default();
        let events = paragraph_events(CUSTOM_TABLE_INPUT);
        let out = process(&events, &fully_custom_options(), &pipeline);

        for name in [
            "CustomTable",
            "TableCaption",
            "TableHead",
            "TableBody",
            "TableFoot",
            "TableRow",
            "TableCell",
        ] {
            assert!(has_custom_component(&out, name), "missing <{name}>");
        }

        for tag in [
            "table", "caption", "thead", "tbody", "tfoot", "tr", "th", "td",
        ] {
            assert!(
                !out.iter().any(
                    |event| matches!(event, Event::StartNode(NodeKind::Element(name)) if name == tag)
                ),
                "plain <{tag}> must not be emitted when a component is configured"
            );
        }
    }

    /// Configuring a single layer must not disable the rest: every remaining
    /// layer falls back to its plain element.
    #[test]
    fn partially_configured_custom_node_falls_back_to_plain_elements() {
        let pipeline = Pipeline::default();
        let events = paragraph_events("| A | B |\n| --- | --- |\n| 1 | 2 |\n");
        let options = TableOptions {
            custom_node: Some(TableCustomNode {
                table: layer("CustomTable"),
                ..Default::default()
            }),
        };
        let out = process(&events, &options, &pipeline);

        assert!(has_custom_component(&out, "CustomTable"));
        for tag in ["thead", "tbody", "tr", "th", "td"] {
            assert!(
                out.iter().any(
                    |event| matches!(event, Event::StartNode(NodeKind::Element(name)) if name == tag)
                ),
                "missing fallback <{tag}>"
            );
        }
        assert!(!has_custom_component(&out, "TableRow"));
    }

    /// Section components cover `<thead>`/`<tbody>`/`<tfoot>` as well.
    #[test]
    fn custom_sections_replace_thead_tbody_tfoot() {
        let pipeline = Pipeline::default();
        let events = paragraph_events("| A |\n| --- |\n| 1 |\n|===|\n| 2 |\n");
        let out = process(&events, &fully_custom_options(), &pipeline);

        // `<TableBody>`/`<TableFoot>` only appear when those rows exist.
        for name in ["TableHead", "TableBody", "TableFoot"] {
            assert!(has_custom_component(&out, name), "missing <{name}>");
        }
        assert!(!has_custom_component(&out, "thead"));
    }

    /// Cells are pre-rendered by this plugin, so the Markdown pass has to know
    /// they must pass through verbatim — a re-lex dropped the whitespace-only
    /// chunks and glued the words of every multi-word cell together.
    #[test]
    fn custom_components_are_marked_as_pre_rendered_elements() {
        let pipeline = Pipeline::default();
        let events = paragraph_events("| Laptop Pro |\n| --- |\n| Mouse Wireless |\n");
        let out = process(&events, &fully_custom_options(), &pipeline);

        assert!(has_attribute(&out, "__plugin_kind", "element"));
        assert!(text_of(&out).contains("Laptop Pro"));
        assert!(text_of(&out).contains("Mouse Wireless"));
    }

    /// Imports declared once under `custom_node` reach every component; a
    /// component's own list is appended on top of them.
    #[test]
    fn shared_imports_apply_to_every_component() {
        let mut options = fully_custom_options();
        let node = options.custom_node.as_mut().expect("custom node");
        node.imports = vec![ImportEntry::Raw(
            "import { TableCaption } from '@comp/table';".to_string(),
        )];
        let thead = node.thead.entries_mut().first_mut().expect("thead entry");
        thead.component.imports = vec![ImportEntry::Structured {
            module: "@comp/table".to_string(),
            default: None,
            names: vec!["TableHead".to_string()],
        }];

        let hints = solid_hints(&options).expect("hints");
        assert_eq!(hints.templates.len(), 7);

        let key = ("TableHead".to_string(), Some("TableHead".to_string()));
        let imports = hints.template_imports.get(&key).expect("thead imports");
        assert_eq!(imports.len(), 2, "shared + own imports: {imports:?}");
        assert!(matches!(&imports[0], ImportEntry::Raw(line) if line.contains("TableCaption")));
        assert!(matches!(
            &imports[1],
            ImportEntry::Structured { module, names, .. }
                if module == "@comp/table" && names == &vec!["TableHead".to_string()]
        ));

        let cell_key = ("TableCell".to_string(), Some("TableCell".to_string()));
        let cell_imports = hints.template_imports.get(&cell_key).expect("cell imports");
        assert_eq!(cell_imports.len(), 1, "shared import only");
        assert!(matches!(&cell_imports[0], ImportEntry::Raw(_)));
    }
}
