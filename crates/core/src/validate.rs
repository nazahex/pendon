use crate::{Event, NodeKind, Severity};

/// Validates the structural invariants required by event-based renderers.
///
/// The validator is intentionally non-mutating so callers can decide whether
/// diagnostics should be warnings, errors, or ignored in production mode.
pub fn validate_events(events: &[Event]) -> Vec<Event> {
    let mut diagnostics = Vec::new();
    let mut stack: Vec<&NodeKind> = Vec::new();

    for event in events {
        match event {
            Event::StartNode(kind) => stack.push(kind),
            Event::EndNode(kind) => match stack.pop() {
                Some(open) if open == kind => {}
                Some(open) => {
                    diagnostics.push(diagnostic(format!(
                        "event node mismatch: closed '{}' while '{}' is open",
                        kind.name(),
                        open.name()
                    )));
                }
                None => diagnostics.push(diagnostic(format!(
                    "event node '{}' closed without an open node",
                    kind.name()
                ))),
            },
            Event::Attribute { name, .. } if stack.is_empty() => {
                diagnostics.push(diagnostic(format!(
                    "attribute '{}' appears outside a node",
                    name
                )));
            }
            _ => {}
        }
    }

    while let Some(open) = stack.pop() {
        diagnostics.push(diagnostic(format!(
            "event node '{}' was not closed",
            open.name()
        )));
    }

    diagnostics
}

fn diagnostic(message: String) -> Event {
    Event::Diagnostic {
        severity: Severity::Error,
        message,
        span: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_balanced_nodes_and_nested_attributes() {
        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Paragraph),
            Event::Attribute {
                name: "class".into(),
                value: "lead".into(),
            },
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ];

        assert!(validate_events(&events).is_empty());
    }

    #[test]
    fn reports_mismatched_and_unclosed_nodes() {
        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Heading),
        ];

        let diagnostics = validate_events(&events);
        assert_eq!(diagnostics.len(), 2);
        assert!(matches!(
            &diagnostics[0],
            Event::Diagnostic {
                severity: Severity::Error,
                ..
            }
        ));
    }
}
