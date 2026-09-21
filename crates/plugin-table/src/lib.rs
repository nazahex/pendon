use pendon_core::{Event, NodeKind, Pipeline};
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
                let html = render::render_table_html(&table_block, inline_pipeline);
                out.push(Event::StartNode(NodeKind::HtmlBlock));
                out.push(Event::Text(html));
                out.push(Event::EndNode(NodeKind::HtmlBlock));
            }
            i = table_block.end_index + 1;
            continue;
        }

        out.push(events[i].clone());
        i += 1;
    }

    out
}
