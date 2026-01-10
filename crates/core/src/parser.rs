use crate::{tokenize, Event, NodeKind, Options, Severity};

// MVP parser: emit Document start/end and Text chunks split at newlines.
// Normalizes CRLF to LF while preserving content semantics.
pub fn parse(input: &str, options: &Options) -> Vec<Event> {
    let mut events = Vec::with_capacity(3 + input.len() / 8);
    events.push(Event::StartNode(NodeKind::Document));

    if !input.is_empty() {
        if let Some(limit) = options.max_doc_bytes {
            if input.len() > limit {
                events.push(Event::Diagnostic {
                    severity: if options.strict {
                        Severity::Error
                    } else {
                        Severity::Warning
                    },
                    message: format!("document bytes {} exceed limit {}", input.len(), limit),
                    span: None,
                });
            }
        }

        let tokens = tokenize(input);
        let mut in_paragraph = false;
        let mut in_heading = false;
        let mut in_code_fence = false;
        let mut blank_run = 0usize;
        let mut current_line_len = 0usize;

        let open_paragraph = |events: &mut Vec<Event>, in_paragraph: &mut bool| {
            if !*in_paragraph {
                events.push(Event::StartNode(NodeKind::Paragraph));
                *in_paragraph = true;
            }
        };
        let close_paragraph = |events: &mut Vec<Event>, in_paragraph: &mut bool| {
            if *in_paragraph {
                events.push(Event::EndNode(NodeKind::Paragraph));
                *in_paragraph = false;
            }
        };

        for tk in tokens {
            match tk {
                crate::Token::Text(s) => {
                    if s.is_empty() {
                        continue;
                    }
                    open_paragraph(&mut events, &mut in_paragraph);
                    current_line_len = current_line_len.saturating_add(s.len());
                    events.push(Event::Text(s.to_string()));
                    blank_run = 0;
                }
                crate::Token::Newline => {
                    // Always preserve newline text for fidelity
                    events.push(Event::Text("\n".to_string()));
                    if in_heading {
                        events.push(Event::EndNode(NodeKind::Heading));
                        in_heading = false;
                    }
                    if let Some(max) = options.max_line_len {
                        if current_line_len > max {
                            events.push(Event::Diagnostic {
                                severity: if options.strict {
                                    Severity::Error
                                } else {
                                    Severity::Warning
                                },
                                message: format!(
                                    "line length {} exceeds limit {}",
                                    current_line_len, max
                                ),
                                span: None,
                            });
                        }
                    }
                    current_line_len = 0;
                    blank_run += 1;
                    if let Some(max_blank) = options.max_blank_run {
                        if blank_run == max_blank + 1 {
                            events.push(Event::Diagnostic {
                                severity: if options.strict {
                                    Severity::Error
                                } else {
                                    Severity::Warning
                                },
                                message: format!(
                                    "blank run {} exceeds limit {}",
                                    blank_run, max_blank
                                ),
                                span: None,
                            });
                        }
                    }
                    if blank_run >= 2 {
                        close_paragraph(&mut events, &mut in_paragraph);
                    }
                }
                crate::Token::FenceBackticks(n) => {
                    // Treat as text for now to preserve concatenation
                    if n >= 3 && current_line_len == 0 {
                        if in_code_fence {
                            events.push(Event::EndNode(NodeKind::CodeFence));
                            in_code_fence = false;
                        } else {
                            events.push(Event::StartNode(NodeKind::CodeFence));
                            in_code_fence = true;
                        }
                    }
                    open_paragraph(&mut events, &mut in_paragraph);
                    events.push(Event::Text("`".repeat(n)));
                    blank_run = 0;
                }
                crate::Token::Hashes(n) => {
                    if current_line_len == 0 {
                        events.push(Event::StartNode(NodeKind::Heading));
                        in_heading = true;
                    }
                    open_paragraph(&mut events, &mut in_paragraph);
                    events.push(Event::Text("#".repeat(n)));
                    blank_run = 0;
                }
            }
        }
        close_paragraph(&mut events, &mut in_paragraph);
        if in_heading {
            events.push(Event::EndNode(NodeKind::Heading));
        }
        if in_code_fence {
            events.push(Event::EndNode(NodeKind::CodeFence));
        }
    }

    events.push(Event::EndNode(NodeKind::Document));
    events
}
