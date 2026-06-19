use crate::charmap::extract_charmap;
use crate::render::{render_dd, render_dt};
use pendon_core::{Event, NodeKind};
use std::collections::HashMap;

pub fn process(events: &[Event]) -> Vec<Event> {
    let charmap = extract_charmap(events);
    let mut out: Vec<Event> = Vec::with_capacity(events.len());
    let mut i = 0usize;

    while i < events.len() {
        if matches!(events.get(i), Some(Event::StartNode(NodeKind::Paragraph))) {
            if let Some(end) = find_matching_end(events, i, NodeKind::Paragraph) {
                let block = &events[i + 1..end];
                if let Some(html) = maybe_render_dialog_block(block, &charmap) {
                    out.push(Event::StartNode(NodeKind::HtmlBlock));
                    out.push(Event::Text(html));
                    out.push(Event::EndNode(NodeKind::HtmlBlock));
                    i = end + 1;
                    continue;
                }
                out.extend(events[i..=end].iter().cloned());
                i = end + 1;
                continue;
            }
        }

        out.push(events[i].clone());
        i += 1;
    }

    out
}

fn maybe_render_dialog_block(
    block_events: &[Event],
    charmap: &HashMap<String, String>,
) -> Option<String> {
    let raw = collect_text(block_events);
    if raw.trim().is_empty() {
        return None;
    }

    let mut rows: Vec<(String, String)> = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let (speaker, content) = trimmed.split_once(':')?;
        let speaker = speaker.trim();
        if speaker.is_empty() || !is_valid_speaker(speaker) {
            return None;
        }
        rows.push((speaker.to_string(), content.trim().to_string()));
    }

    if rows.is_empty() {
        return None;
    }

    let mut html = String::new();
    html.push_str("<dl>\n");
    for (speaker, content) in rows {
        let class = charmap.get(&speaker).cloned();
        html.push_str(&render_dt(&speaker, class.as_deref()));
        html.push(' ');
        html.push_str(&render_dd(&content, class.as_deref()));
        html.push('\n');
    }
    html.push_str("</dl>\n");
    Some(html)
}

fn is_valid_speaker(speaker: &str) -> bool {
    let mut has_alnum = false;
    for ch in speaker.chars() {
        if ch.is_alphanumeric() {
            has_alnum = true;
            continue;
        }
        if ch.is_whitespace() || matches!(ch, '\'' | '-' | '_' | '.') {
            continue;
        }
        return false;
    }
    has_alnum
}

fn collect_text(events: &[Event]) -> String {
    let mut out = String::new();
    for ev in events {
        if let Event::Text(t) = ev {
            out.push_str(t);
        }
    }
    out
}

fn find_matching_end(events: &[Event], start_idx: usize, kind: NodeKind) -> Option<usize> {
    let mut depth = 0isize;
    for (idx, ev) in events.iter().enumerate().skip(start_idx) {
        match ev {
            Event::StartNode(k) if *k == kind => depth += 1,
            Event::EndNode(k) if *k == kind => {
                depth -= 1;
                if depth == 0 {
                    return Some(idx);
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn does_not_parse_image_url_as_dialog() {
        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Paragraph),
            Event::Text("!![Alt](https://example.com/a.webp)".to_string()),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ];

        let out = process(&events);
        assert!(!out
            .iter()
            .any(|ev| matches!(ev, Event::StartNode(NodeKind::HtmlBlock))));
    }

    #[test]
    fn still_parses_normal_dialog_lines() {
        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Paragraph),
            Event::Text("Revan: \"Hi\"\nStevano: \"Hello\"".to_string()),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ];

        let out = process(&events);
        assert!(out
            .iter()
            .any(|ev| matches!(ev, Event::StartNode(NodeKind::HtmlBlock))));
    }
}
