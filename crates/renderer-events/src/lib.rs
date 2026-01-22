use pendon_core::{Event, NodeKind, Severity};
use serde::Serialize;

#[derive(Serialize)]
#[serde(tag = "type")]
enum Ev<'a> {
    Start { node: &'a str },
    End { node: &'a str },
    Text { text: &'a str },
    Attribute { name: &'a str, value: &'a str },
    Diagnostic { severity: &'a str, message: &'a str },
}

pub fn render_events_to_string(events: &[Event]) -> Result<String, serde_json::Error> {
    let mut out: Vec<Ev> = Vec::with_capacity(events.len());
    for ev in events {
        match ev {
            Event::StartNode(kind) => out.push(Ev::Start {
                node: node_str(kind),
            }),
            Event::EndNode(kind) => out.push(Ev::End {
                node: node_str(kind),
            }),
            Event::Text(s) => out.push(Ev::Text { text: s }),
            Event::Attribute { name, value } => out.push(Ev::Attribute { name, value }),
            Event::Diagnostic {
                severity, message, ..
            } => out.push(Ev::Diagnostic {
                severity: match severity {
                    Severity::Warning => "Warning",
                    Severity::Error => "Error",
                },
                message,
            }),
        }
    }
    serde_json::to_string(&out)
}

fn node_str(k: &NodeKind) -> &str {
    match k {
        NodeKind::Document => "Document",
        NodeKind::Frontmatter => "Frontmatter",
        NodeKind::Paragraph => "Paragraph",
        NodeKind::Blockquote => "Blockquote",
        NodeKind::CodeFence => "CodeFence",
        NodeKind::Heading => "Heading",
        NodeKind::ThematicBreak => "ThematicBreak",
        NodeKind::BulletList => "BulletList",
        NodeKind::OrderedList => "OrderedList",
        NodeKind::ListItem => "ListItem",
        NodeKind::Table => "Table",
        NodeKind::TableHead => "TableHead",
        NodeKind::TableBody => "TableBody",
        NodeKind::TableRow => "TableRow",
        NodeKind::TableCell => "TableCell",
        NodeKind::Emphasis => "Emphasis",
        NodeKind::Strong => "Strong",
        NodeKind::InlineCode => "InlineCode",
        NodeKind::Link => "Link",
        NodeKind::Bold => "Bold",
        NodeKind::Italic => "Italic",
    }
}
