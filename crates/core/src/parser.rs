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
        let mut in_html_comment = false;
        let mut in_bind_block = false;
        let mut blank_run = 0usize;
        let mut current_line_len = 0usize;
        let mut just_opened_code_fence = false;

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
                // Treat immediate text after an opening code fence as info string (lang)
                crate::Token::Text(s) if just_opened_code_fence && current_line_len == 0 => {
                    let info = s.trim();
                    if !info.is_empty() {
                        events.push(Event::Attribute {
                            name: "lang".to_string(),
                            value: info.to_string(),
                        });
                    }
                    // do not emit as content, and do not open paragraph
                    blank_run = 0;
                }
                crate::Token::Text(s) => {
                    if s.is_empty() {
                        continue;
                    }
                    // Thematic break hint: line composed of 3+ hyphens at start
                    if current_line_len == 0 {
                        let trimmed = s.trim();
                        if trimmed.len() >= 3 && trimmed.chars().all(|c| c == '-') {
                            events.push(Event::StartNode(NodeKind::ThematicBreak));
                            events.push(Event::Text(s.to_string()));
                            events.push(Event::EndNode(NodeKind::ThematicBreak));
                            blank_run = 0;
                            current_line_len = current_line_len.saturating_add(s.len());
                            continue;
                        }
                    }
                    // HTML comment detection: <!-- ... -->
                    if current_line_len == 0 {
                        if s.trim_start().starts_with("<!--") {
                            // One-line comments (`<!-- x -->`) open and close at
                            // once, matching the scanner; only a line that lacks
                            // `-->` opens a multi-line comment.
                            if !s.contains("-->") {
                                in_html_comment = true;
                            }
                            // Still emit the text for fidelity
                            open_paragraph(&mut events, &mut in_paragraph);
                            current_line_len = current_line_len.saturating_add(s.len());
                            events.push(Event::Text(s.to_string()));
                            blank_run = 0;
                            continue;
                        }
                        if s.trim_start().ends_with("-->") && in_html_comment {
                            in_html_comment = false;
                            // Still emit the text for fidelity
                            open_paragraph(&mut events, &mut in_paragraph);
                            current_line_len = current_line_len.saturating_add(s.len());
                            events.push(Event::Text(s.to_string()));
                            blank_run = 0;
                            continue;
                        }
                    }
                    // Bind block detection: {{{ ... }}}
                    if current_line_len == 0 {
                        if s.trim_start().starts_with("{{{") {
                            // A head line that already carries its `}}}` (a
                            // `(path)` payload or a one-line body) is a whole
                            // block that opens and closes at once; only a line
                            // lacking `}}}` opens a multi-line block.
                            if !s.contains("}}}") {
                                in_bind_block = true;
                            }
                            // Still emit the text for fidelity
                            open_paragraph(&mut events, &mut in_paragraph);
                            current_line_len = current_line_len.saturating_add(s.len());
                            events.push(Event::Text(s.to_string()));
                            blank_run = 0;
                            continue;
                        }
                        if s.trim_start().ends_with("}}}") && in_bind_block {
                            in_bind_block = false;
                            // Still emit the text for fidelity
                            open_paragraph(&mut events, &mut in_paragraph);
                            current_line_len = current_line_len.saturating_add(s.len());
                            events.push(Event::Text(s.to_string()));
                            blank_run = 0;
                            continue;
                        }
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
                    // reset any one-line state
                    just_opened_code_fence = false;
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
                    // Code fence markers: toggle node, do not emit backtick text
                    if n >= 3 && current_line_len == 0 {
                        if in_code_fence {
                            events.push(Event::EndNode(NodeKind::CodeFence));
                            in_code_fence = false;
                        } else {
                            events.push(Event::StartNode(NodeKind::CodeFence));
                            in_code_fence = true;
                            just_opened_code_fence = true;
                        }
                        // fence line should not open/close paragraph nor add text
                        blank_run = 0;
                    } else {
                        // fallback: treat as literal backticks
                        open_paragraph(&mut events, &mut in_paragraph);
                        events.push(Event::Text("`".repeat(n)));
                        blank_run = 0;
                    }
                }
                crate::Token::Hashes(n) => {
                    if current_line_len == 0 && !in_code_fence && !in_html_comment && !in_bind_block
                    {
                        events.push(Event::StartNode(NodeKind::Heading));
                        // attach heading level attribute
                        events.push(Event::Attribute {
                            name: "level".to_string(),
                            value: n.to_string(),
                        });
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
