use pendon_core::{Event, NodeKind};
use pendon_plugin_markdown::process;

fn block_custom_body() -> Vec<Event> {
    // Simulate what plugin-directive will emit for ==note[a]\n# Title\npara 1\n==\n
    vec![
        Event::StartNode(NodeKind::Document),
        Event::StartNode(NodeKind::Custom("Note".to_string())),
        Event::Attribute {
            name: "__plugin_kind".into(),
            value: "block".into(),
        },
        Event::Attribute {
            name: "type".into(),
            value: "note".into(),
        },
        Event::StartNode(NodeKind::Paragraph),
        Event::Text("\n".into()),
        Event::StartNode(NodeKind::Heading),
        Event::Attribute {
            name: "level".into(),
            value: "1".into(),
        },
        Event::Text("Title".into()),
        Event::Text("\n".into()),
        Event::EndNode(NodeKind::Heading),
        Event::Text("para 1".into()),
        Event::Text("\n".into()),
        Event::EndNode(NodeKind::Paragraph),
        Event::EndNode(NodeKind::Custom("Note".to_string())),
        Event::EndNode(NodeKind::Document),
    ]
}

fn inline_custom_span() -> Vec<Event> {
    vec![
        Event::StartNode(NodeKind::Document),
        Event::StartNode(NodeKind::Paragraph),
        Event::Text("before ".into()),
        Event::StartNode(NodeKind::Custom("Tip".to_string())),
        Event::Attribute {
            name: "__plugin_kind".into(),
            value: "inline".into(),
        },
        Event::Attribute {
            name: "type".into(),
            value: "tip".into(),
        },
        Event::Text("hello **bold** and `code`".into()),
        Event::EndNode(NodeKind::Custom("Tip".to_string())),
        Event::Text(" after".into()),
        Event::EndNode(NodeKind::Paragraph),
        Event::EndNode(NodeKind::Document),
    ]
}

fn main() {
    let out = process(&block_custom_body());
    println!("======= BLOCK Custom placement=block =======");
    for ev in &out {
        println!("{:?}", ev);
    }
    let out = process(&inline_custom_span());
    println!("======= INLINE Custom placement=inline =======");
    for ev in &out {
        println!("{:?}", ev);
    }
}
