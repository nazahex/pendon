use pendon_core::{Event, NodeKind};

pub fn process(events: &[Event]) -> Vec<Event> {
    // 1. Merge adjacent text events to handle multiline formula blocks cleanly
    let merged = merge_adjacent_text(events.to_vec());

    // 2. Process math parsing on non-excluded text events
    let mut out = Vec::with_capacity(merged.len());
    let mut exclude_depth: usize = 0;

    for ev in merged {
        match &ev {
            Event::StartNode(NodeKind::CodeFence)
            | Event::StartNode(NodeKind::InlineCode)
            | Event::StartNode(NodeKind::HtmlBlock)
            | Event::StartNode(NodeKind::HtmlInline) => {
                exclude_depth += 1;
                out.push(ev);
            }
            Event::EndNode(NodeKind::CodeFence)
            | Event::EndNode(NodeKind::InlineCode)
            | Event::EndNode(NodeKind::HtmlBlock)
            | Event::EndNode(NodeKind::HtmlInline) => {
                exclude_depth = exclude_depth.saturating_sub(1);
                out.push(ev);
            }
            Event::Text(text) if exclude_depth == 0 => {
                process_text(text, &mut out);
            }
            _ => {
                out.push(ev);
            }
        }
    }

    out
}

fn merge_adjacent_text(events: Vec<Event>) -> Vec<Event> {
    let mut out: Vec<Event> = Vec::with_capacity(events.len());
    for ev in events {
        match ev {
            Event::Text(t) => {
                if let Some(Event::Text(prev)) = out.last_mut() {
                    prev.push_str(&t);
                } else {
                    out.push(Event::Text(t));
                }
            }
            other => out.push(other),
        }
    }
    out
}

fn process_text(text: &str, out: &mut Vec<Event>) {
    let mut cursor = 0;
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut normal_text = String::new();

    let flush_normal = |normal: &mut String, out: &mut Vec<Event>| {
        if !normal.is_empty() {
            out.push(Event::Text(normal.clone()));
            normal.clear();
        }
    };

    while cursor < len {
        // Look for escaped dollar sign "\$"
        if chars[cursor] == '\\' && cursor + 1 < len && chars[cursor + 1] == '$' {
            normal_text.push('$');
            cursor += 2;
            continue;
        }

        // Look for block math "$$"
        if cursor + 1 < len && chars[cursor] == '$' && chars[cursor + 1] == '$' {
            let start = cursor;
            let mut end = cursor + 2;
            let mut found_close = false;
            while end + 1 < len {
                if chars[end] == '$' && chars[end + 1] == '$' {
                    found_close = true;
                    break;
                }
                end += 1;
            }

            if found_close {
                flush_normal(&mut normal_text, out);
                let formula: String = chars[start + 2..end].iter().collect();

                let opts = katex::Opts::builder()
                    .display_mode(true)
                    .throw_on_error(false)
                    .build()
                    .unwrap();

                match katex::render_with_opts(&formula, &opts) {
                    Ok(html) => {
                        out.push(Event::StartNode(NodeKind::HtmlBlock));
                        out.push(Event::Text(html));
                        out.push(Event::EndNode(NodeKind::HtmlBlock));
                    }
                    Err(err) => {
                        out.push(Event::StartNode(NodeKind::HtmlBlock));
                        out.push(Event::Text(format!(
                            "<span class=\"katex-error\">{}</span>",
                            err
                        )));
                        out.push(Event::EndNode(NodeKind::HtmlBlock));
                    }
                }
                cursor = end + 2;
                continue;
            }
        }

        // Look for inline math "$"
        if chars[cursor] == '$' {
            let start = cursor;
            let mut end = cursor + 1;
            let mut found_close = false;

            if end < len && chars[end] != ' ' && chars[end] != '\n' {
                while end < len {
                    if chars[end] == '$' {
                        if end > start + 1 && chars[end - 1] != ' ' && chars[end - 1] != '\n' {
                            found_close = true;
                            break;
                        }
                    }
                    end += 1;
                }
            }

            if found_close {
                flush_normal(&mut normal_text, out);
                let formula: String = chars[start + 1..end].iter().collect();

                let opts = katex::Opts::builder()
                    .display_mode(false)
                    .throw_on_error(false)
                    .build()
                    .unwrap();

                match katex::render_with_opts(&formula, &opts) {
                    Ok(html) => {
                        out.push(Event::StartNode(NodeKind::HtmlInline));
                        out.push(Event::Text(html));
                        out.push(Event::EndNode(NodeKind::HtmlInline));
                    }
                    Err(err) => {
                        out.push(Event::StartNode(NodeKind::HtmlInline));
                        out.push(Event::Text(format!(
                            "<span class=\"katex-error\">{}</span>",
                            err
                        )));
                        out.push(Event::EndNode(NodeKind::HtmlInline));
                    }
                }
                cursor = end + 1;
                continue;
            }
        }

        // Otherwise, consume one character as normal text
        normal_text.push(chars[cursor]);
        cursor += 1;
    }

    flush_normal(&mut normal_text, out);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_math() {
        let events = vec![Event::Text("Einstein: $E = mc^2$.".to_string())];
        let res = process(&events);
        assert_eq!(res.len(), 5);
        assert_eq!(res[0], Event::Text("Einstein: ".to_string()));
        assert_eq!(res[1], Event::StartNode(NodeKind::HtmlInline));
        assert!(matches!(&res[2], Event::Text(h) if h.contains("class=\"katex\"")));
        assert_eq!(res[3], Event::EndNode(NodeKind::HtmlInline));
        assert_eq!(res[4], Event::Text(".".to_string()));
    }

    #[test]
    fn test_block_math() {
        let events = vec![
            Event::Text("$$".to_string()),
            Event::Text("\n".to_string()),
            Event::Text("x = y".to_string()),
            Event::Text("\n".to_string()),
            Event::Text("$$".to_string()),
        ];
        let res = process(&events);
        assert_eq!(res.len(), 3);
        assert_eq!(res[0], Event::StartNode(NodeKind::HtmlBlock));
        assert!(matches!(&res[1], Event::Text(h) if h.contains("class=\"katex-display\"")));
        assert_eq!(res[2], Event::EndNode(NodeKind::HtmlBlock));
    }

    #[test]
    fn test_escaped_dollar() {
        let events = vec![Event::Text("I have \\$5 and \\$10.".to_string())];
        let res = process(&events);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0], Event::Text("I have $5 and $10.".to_string()));
    }
}
