use pendon_core::{Event, NodeKind};
use std::collections::{HashMap, VecDeque};

struct SectionFrame {
    level: usize,
}

struct HeadingInfo {
    level: usize,
    id: Option<String>,
}

// --- Prefix Parser (ID extraction only, mirrors plugin-heading) ---

/// Extracts the custom ID from [id] prefix without modifying heading text.
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

fn ensure_unique(base: String, used: &mut HashMap<String, usize>) -> String {
    let counter = used.entry(base.clone()).or_insert(0);
    if *counter == 0 {
        *counter = 1;
        base
    } else {
        *counter += 1;
        format!("{}-{}", base, *counter)
    }
}

// --- Main Processor ---

pub fn process(events: &[Event]) -> Vec<Event> {
    let stripped = strip_frontmatter_block(events);
    let events = stripped.as_slice();

    let mut out: Vec<Event> = Vec::with_capacity(events.len() + 8);
    // ... sisa kode tetap sama
    let mut used_ids: HashMap<String, usize> = HashMap::new();
    let mut stack: Vec<SectionFrame> = Vec::new();
    let mut container_stack: Vec<NodeKind> = Vec::new();
    let mut forced_closed: VecDeque<NodeKind> = VecDeque::new();
    let mut in_code_fence = false;
    let mut in_frontmatter = false;
    let mut idx = 0usize;

    while idx < events.len() {
        match &events[idx] {
            Event::StartNode(NodeKind::Heading) if !in_code_fence && !in_frontmatter => {
                close_lists_for_heading(&mut container_stack, &mut forced_closed, &mut out);

                let (heading_block, heading_info, consumed) =
                    consume_heading(events, idx, &mut used_ids);

                close_sections_by_level(&mut stack, &mut out, heading_info.level);
                open_section(&mut stack, &mut out, heading_info.level, heading_info.id);

                out.extend(heading_block);
                idx = consumed;
                continue;
            }
            Event::StartNode(kind) => {
                container_stack.push(kind.clone());
                if *kind == NodeKind::CodeFence {
                    in_code_fence = true;
                }
                if *kind == NodeKind::Frontmatter {
                    in_frontmatter = true;
                }

                if !matches!(kind, NodeKind::Document) && !in_frontmatter {
                    ensure_preface_section(&mut stack, &mut out);
                }

                out.push(events[idx].clone());
            }
            Event::EndNode(kind) => {
                if forced_closed.front() == Some(kind) {
                    forced_closed.pop_front();
                    idx += 1;
                    continue;
                }
                pop_container(&mut container_stack, kind);
                out.push(events[idx].clone());

                if *kind == NodeKind::CodeFence {
                    in_code_fence = false;
                }
                if *kind == NodeKind::Frontmatter {
                    in_frontmatter = false;
                }
            }
            ev => {
                if !in_frontmatter {
                    ensure_preface_section(&mut stack, &mut out);
                }
                out.push(ev.clone());
            }
        }
        idx += 1;
    }

    // Close any remaining open sections
    while stack.pop().is_some() {
        out.push(Event::EndNode(NodeKind::Section));
    }

    out
}

/// Consumes a heading block and extracts its ID for section naming.
/// Does NOT modify heading text content — text cleanup is the responsibility
/// of plugin-heading which should run before this plugin.
fn consume_heading(
    events: &[Event],
    start_idx: usize,
    used_ids: &mut HashMap<String, usize>,
) -> (Vec<Event>, HeadingInfo, usize) {
    let mut out: Vec<Event> = Vec::new();
    let mut idx = start_idx;
    let mut heading_level: usize = 1;
    let mut heading_text = String::new();
    let mut heading_id_attr: Option<String> = None;

    while idx < events.len() {
        match &events[idx] {
            Event::StartNode(NodeKind::Heading) => {
                out.push(events[idx].clone());
                idx += 1;
            }
            Event::EndNode(NodeKind::Heading) => {
                out.push(events[idx].clone());
                idx += 1;
                break;
            }
            Event::Attribute { name, value } => {
                if name == "level" {
                    if let Ok(parsed) = value.parse::<usize>() {
                        heading_level = parsed;
                    }
                    out.push(events[idx].clone());
                } else if name == "id" {
                    // Capture id attribute set by plugin-heading, but do NOT
                    // copy it to output — ids belong on the Section node instead.
                    heading_id_attr = Some(value.clone());
                } else {
                    out.push(events[idx].clone());
                }
                idx += 1;
            }
            Event::Text(text) => {
                heading_text.push_str(text);
                out.push(events[idx].clone());
                idx += 1;
            }
            other => {
                out.push(other.clone());
                idx += 1;
            }
        }
    }

    // Extract ID using prefix parser as fallback when plugin-heading hasn't run
    let (prefix_id, _consumed) = parse_heading_prefix(&heading_text);

    // Priority: explicit id attr > prefix [id] > auto-slug from visible text
    // For slug generation, use text after prefix to avoid bracket/brace artifacts
    let clean_text = if _consumed > 0 {
        heading_text[_consumed..].trim().to_string()
    } else {
        heading_text.trim().to_string()
    };

    let need_id = heading_level >= 2 || prefix_id.is_some() || heading_id_attr.is_some();
    let final_id = if need_id {
        let base = heading_id_attr
            .or(prefix_id)
            .unwrap_or_else(|| slugify(&clean_text));
        Some(ensure_unique(base, used_ids))
    } else {
        None
    };

    (
        out,
        HeadingInfo {
            level: heading_level,
            id: final_id,
        },
        idx,
    )
}

fn open_section(
    stack: &mut Vec<SectionFrame>,
    out: &mut Vec<Event>,
    level: usize,
    id: Option<String>,
) {
    out.push(Event::StartNode(NodeKind::Section));
    if let Some(id_val) = id {
        out.push(Event::Attribute {
            name: "id".to_string(),
            value: id_val,
        });
    }
    stack.push(SectionFrame { level });
}

fn ensure_preface_section(stack: &mut Vec<SectionFrame>, out: &mut Vec<Event>) {
    if stack.is_empty() {
        // Preface carries level 0 so it stays open until a heading explicitly closes it.
        open_section(stack, out, 0, None);
    }
}

fn close_sections_by_level(stack: &mut Vec<SectionFrame>, out: &mut Vec<Event>, new_level: usize) {
    while let Some(frame) = stack.last() {
        if frame.level == 0 {
            out.push(Event::EndNode(NodeKind::Section));
            stack.pop();
            continue;
        }

        if (frame.level == 1 && new_level >= 2) || (frame.level >= new_level) {
            out.push(Event::EndNode(NodeKind::Section));
            stack.pop();
        } else {
            break;
        }
    }
}

fn is_list_kind(kind: &NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::BulletList | NodeKind::OrderedList | NodeKind::ListItem
    )
}

fn close_lists_for_heading(
    container_stack: &mut Vec<NodeKind>,
    forced_closed: &mut VecDeque<NodeKind>,
    out: &mut Vec<Event>,
) {
    while let Some(top) = container_stack.last() {
        if is_list_kind(top) {
            let kind = container_stack.pop().unwrap();
            out.push(Event::EndNode(kind.clone()));
            forced_closed.push_back(kind);
        } else {
            break;
        }
    }
}

fn pop_container(container_stack: &mut Vec<NodeKind>, kind: &NodeKind) {
    if container_stack.last() == Some(kind) {
        container_stack.pop();
    }
}

fn strip_frontmatter_block(events: &[Event]) -> Vec<Event> {
    let mut out: Vec<Event> = Vec::with_capacity(events.len());
    let mut idx = 0usize;

    // Preserve leading Document start if present
    if matches!(events.get(idx), Some(Event::StartNode(NodeKind::Document))) {
        out.push(events[idx].clone());
        idx += 1;
    }

    if let Some(consumed) = frontmatter_span(events, idx) {
        idx = consumed;
    }

    out.extend_from_slice(&events[idx..]);
    out
}

fn frontmatter_span(events: &[Event], start: usize) -> Option<usize> {
    let mut idx = start;

    // Opening ---
    if !matches!(
        events.get(idx),
        Some(Event::StartNode(NodeKind::ThematicBreak))
    ) {
        return None;
    }
    idx += 1;
    if matches!(
        events.get(idx),
        Some(Event::EndNode(NodeKind::ThematicBreak))
    ) {
        idx += 1;
    }

    // Paragraph payload
    if !matches!(events.get(idx), Some(Event::StartNode(NodeKind::Paragraph))) {
        return None;
    }
    idx += 1;
    while idx < events.len() {
        match events.get(idx) {
            Some(Event::EndNode(NodeKind::Paragraph)) => {
                idx += 1;
                break;
            }
            Some(_) => idx += 1,
            None => return None,
        }
    }

    // Closing ---
    if !matches!(
        events.get(idx),
        Some(Event::StartNode(NodeKind::ThematicBreak))
    ) {
        return None;
    }
    idx += 1;
    if matches!(
        events.get(idx),
        Some(Event::EndNode(NodeKind::ThematicBreak))
    ) {
        idx += 1;
    }

    Some(idx)
}
