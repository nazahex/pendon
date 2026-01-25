use pendon_core::{Event, Severity};
use serde::Serialize;

#[derive(Serialize)]
#[serde(tag = "type")]
enum Ev<'a> {
    Start { node: String },
    End { node: String },
    Text { text: &'a str },
    Attribute { name: &'a str, value: &'a str },
    Diagnostic { severity: &'a str, message: &'a str },
}

pub fn render_events_to_string(events: &[Event]) -> Result<String, serde_json::Error> {
    let mut out: Vec<Ev> = Vec::with_capacity(events.len());
    for ev in events {
        match ev {
            Event::StartNode(kind) => out.push(Ev::Start {
                node: kind.name().into_owned(),
            }),
            Event::EndNode(kind) => out.push(Ev::End {
                node: kind.name().into_owned(),
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
