use pendon_core::{Event, NodeKind};

pub(crate) fn collect_text(events: &[Event]) -> String {
    let mut out = String::new();
    for ev in events {
        if let Event::Text(t) = ev {
            out.push_str(t);
        }
    }
    out
}

pub(crate) fn find_matching_end(
    events: &[Event],
    start_idx: usize,
    kind: NodeKind,
) -> Option<usize> {
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

pub(crate) fn escape_html(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

pub(crate) fn normalize_class_tokens(raw: &str) -> String {
    raw.split(',')
        .map(|t| t.trim().trim_start_matches('.'))
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}
