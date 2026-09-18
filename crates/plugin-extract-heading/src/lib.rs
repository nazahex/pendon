use pendon_core::{Event, NodeKind};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PendonHeading {
    pub id: String,
    pub text: String,
    pub level: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub subheadings: Vec<PendonHeading>,
}

#[derive(Debug, Clone)]
struct HeadingCapture {
    text: String,
    level: usize,
    id: Option<String>,
}

// --- Extras Parser (mirrors plugin-heading) ---

/// Parses [id][.classes]{attrs} from the start of heading text.
/// Returns (custom_id, consumed_byte_length).
fn parse_heading_prefix(raw_text: &str) -> (Option<String>, usize) {
    let chars: Vec<char> = raw_text.chars().collect();
    let mut cursor = 0;
    let mut custom_id: Option<String> = None;

    // Skip leading whitespace
    while cursor < chars.len() && chars[cursor].is_whitespace() {
        cursor += 1;
    }

    // Parse first bracket group: [id] or [.class]
    if cursor < chars.len() && chars[cursor] == '[' {
        if let Some(close) = find_char(&chars, cursor + 1, ']') {
            let content: String = chars[cursor + 1..close].iter().collect();
            let trimmed = content.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('.') {
                custom_id = Some(trimmed.to_string());
            }
            cursor = close + 1;
        }
    }

    // Skip subsequent bracket groups: [.class,.extra]
    loop {
        while cursor < chars.len() && chars[cursor].is_whitespace() {
            cursor += 1;
        }
        if cursor >= chars.len() || chars[cursor] != '[' {
            break;
        }
        if let Some(close) = find_char(&chars, cursor + 1, ']') {
            cursor = close + 1;
        } else {
            break;
        }
    }

    // Skip curly brace attributes: { key: "val" }
    while cursor < chars.len() && chars[cursor].is_whitespace() {
        cursor += 1;
    }
    if cursor < chars.len() && chars[cursor] == '{' {
        if let Some(close) = find_matching_brace(&chars, cursor) {
            cursor = close + 1;
        }
    }

    (custom_id, cursor)
}

fn find_char(chars: &[char], mut index: usize, wanted: char) -> Option<usize> {
    while index < chars.len() {
        if chars[index] == wanted {
            return Some(index);
        }
        index += 1;
    }
    None
}

fn find_matching_brace(chars: &[char], start: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut i = start;
    while i < chars.len() {
        match chars[i] {
            '{' => depth += 1,
            '}' if depth == 1 => return Some(i),
            '}' => depth -= 1,
            _ => {}
        }
        i += 1;
    }
    None
}

/// Generates a URL-safe slug from heading text.
fn slugify(input: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in input.chars() {
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() {
            out.push(lower);
            last_dash = false;
        } else if matches!(lower, ' ' | '-' | '_' | '.') {
            if !last_dash && !out.is_empty() {
                out.push('-');
                last_dash = true;
            }
        }
    }
    if out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "section".to_string()
    } else {
        out
    }
}

// --- Main Processor ---

pub fn process(events: &[Event]) -> Vec<Event> {
    let headings = collect_headings(events);
    if headings.is_empty() {
        return events.to_vec();
    }

    let data = match serde_json::to_string(&headings) {
        Ok(s) => s,
        Err(_) => return events.to_vec(),
    };

    inject_headings_node(events, &data)
}

fn collect_headings(events: &[Event]) -> Vec<PendonHeading> {
    let mut roots: Vec<PendonHeading> = Vec::new();
    let mut path: Vec<usize> = Vec::new();
    let mut section_stack: Vec<Option<String>> = Vec::new();
    let mut idx = 0usize;

    while idx < events.len() {
        match &events[idx] {
            Event::StartNode(NodeKind::Section) => {
                section_stack.push(None);
                idx += 1;
            }
            Event::Attribute { name, value } if name == "id" && section_stack.last().is_some() => {
                if let Some(last) = section_stack.last_mut() {
                    *last = Some(value.clone());
                }
                idx += 1;
            }
            Event::EndNode(NodeKind::Section) => {
                section_stack.pop();
                idx += 1;
            }
            Event::StartNode(NodeKind::Heading) => {
                let (capture, consumed) = consume_heading(events, idx);
                let section_id = section_stack.last().and_then(|id| id.clone());
                let id = section_id
                    .or(capture.id)
                    .unwrap_or_else(|| slugify(&capture.text));
                let level = capture.level.max(1);
                let node = PendonHeading {
                    id,
                    text: capture.text,
                    level,
                    subheadings: Vec::new(),
                };
                insert_heading(node, &mut roots, &mut path);
                idx = consumed;
            }
            _ => {
                idx += 1;
            }
        }
    }

    roots
}

fn consume_heading(events: &[Event], start_idx: usize) -> (HeadingCapture, usize) {
    let mut idx = start_idx + 1;
    let mut level: usize = 1;
    let mut text = String::new();
    let mut heading_id: Option<String> = None;

    while idx < events.len() {
        match &events[idx] {
            Event::Attribute { name, value } => {
                if name == "level" {
                    if let Ok(parsed) = value.parse::<usize>() {
                        level = parsed;
                    }
                } else if name == "id" {
                    heading_id = Some(value.clone());
                }
                idx += 1;
            }
            Event::Text(t) => {
                text.push_str(t);
                idx += 1;
            }
            Event::EndNode(NodeKind::Heading) => {
                idx += 1;
                break;
            }
            _ => {
                idx += 1;
            }
        }
    }

    // Use the same prefix parser as plugin-heading to strip [id][.class]{attrs}
    let (prefix_id, consumed_len) = parse_heading_prefix(&text);
    let clean_text = text[consumed_len..].trim().to_string();

    let final_text = if clean_text.is_empty() {
        text.trim().to_string()
    } else {
        clean_text
    };

    (
        HeadingCapture {
            text: final_text,
            level,
            id: heading_id.or(prefix_id),
        },
        idx,
    )
}

fn insert_heading(node: PendonHeading, roots: &mut Vec<PendonHeading>, path: &mut Vec<usize>) {
    while let Some(level) = current_level(roots, path) {
        if level >= node.level {
            path.pop();
        } else {
            break;
        }
    }

    let target = resolve_children_mut(roots, path);
    target.push(node);
    let new_idx = target.len().saturating_sub(1);
    path.push(new_idx);
}

fn current_level(roots: &[PendonHeading], path: &[usize]) -> Option<usize> {
    let mut cursor = roots;
    let mut node: Option<&PendonHeading> = None;
    for &idx in path {
        node = cursor.get(idx);
        cursor = match node {
            Some(n) => &n.subheadings,
            None => return None,
        };
    }
    node.map(|n| n.level)
}

fn resolve_children_mut<'a>(
    roots: &'a mut Vec<PendonHeading>,
    path: &[usize],
) -> &'a mut Vec<PendonHeading> {
    let mut cursor = roots;
    for &idx in path {
        cursor = &mut cursor[idx].subheadings;
    }
    cursor
}

fn inject_headings_node(events: &[Event], data: &str) -> Vec<Event> {
    let mut out: Vec<Event> = Vec::with_capacity(events.len() + 4);
    let mut in_document = false;
    let mut pending_insert = false;
    let mut inserted = false;
    let mut frontmatter_depth = 0usize;

    for ev in events.iter() {
        match ev {
            Event::StartNode(NodeKind::Document) => {
                in_document = true;
                pending_insert = true;
                out.push(ev.clone());
            }
            Event::StartNode(NodeKind::Frontmatter) if in_document => {
                frontmatter_depth += 1;
                out.push(ev.clone());
            }
            Event::EndNode(NodeKind::Frontmatter) if in_document => {
                out.push(ev.clone());
                frontmatter_depth = frontmatter_depth.saturating_sub(1);
                if frontmatter_depth == 0 && pending_insert && !inserted {
                    push_headings_block(&mut out, data);
                    inserted = true;
                    pending_insert = false;
                }
            }
            Event::EndNode(NodeKind::Document) if in_document => {
                if pending_insert && !inserted {
                    push_headings_block(&mut out, data);
                    inserted = true;
                    pending_insert = false;
                }
                out.push(ev.clone());
                in_document = false;
            }
            _ => {
                if in_document && pending_insert && frontmatter_depth == 0 && !inserted {
                    push_headings_block(&mut out, data);
                    inserted = true;
                    pending_insert = false;
                }
                out.push(ev.clone());
            }
        }
    }

    if !inserted && in_document {
        push_headings_block(&mut out, data);
    }

    out
}

fn push_headings_block(out: &mut Vec<Event>, data: &str) {
    let kind = headings_node_kind();
    out.push(Event::StartNode(kind.clone()));
    out.push(Event::Attribute {
        name: "data".to_string(),
        value: data.to_string(),
    });
    out.push(Event::EndNode(kind));
}

fn headings_node_kind() -> NodeKind {
    NodeKind::Custom("Headings".to_string())
}
