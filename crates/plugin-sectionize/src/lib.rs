use pendon_core::{Event, NodeKind};
use std::collections::{HashMap, VecDeque};

struct SectionFrame {
    level: usize,
}

struct HeadingInfo {
    level: usize,
    id: Option<String>,
}

pub fn process(events: &[Event]) -> Vec<Event> {
    let events = strip_frontmatter_block(events);

    let mut out: Vec<Event> = Vec::with_capacity(events.len() + 8);
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

                let (mut heading_block, heading_info, consumed) =
                    consume_heading(&events, idx, &mut used_ids);

                close_sections_by_level(&mut stack, &mut out, heading_info.level);

                open_section(&mut stack, &mut out, heading_info.level, heading_info.id);

                out.append(&mut heading_block);

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
    let mut last_text_idx: Option<usize> = None;

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
                    // do not copy level attribute here; it will be reattached via original event
                    out.push(events[idx].clone());
                } else if name == "id" {
                    heading_id_attr = Some(value.clone());
                    // skip copying heading id attribute; ids move to section
                } else {
                    out.push(events[idx].clone());
                }
                idx += 1;
            }
            Event::Text(text) => {
                heading_text.push_str(text);
                last_text_idx = Some(out.len());
                out.push(events[idx].clone());
                idx += 1;
            }
            other => {
                out.push(other.clone());
                idx += 1;
            }
        }
    }

    if let Some(tidx) = last_text_idx {
        if let Some(Event::Text(text)) = out.get(tidx) {
            let (stripped, removed) = strip_trailing_id(text);
            if removed {
                if let Some(Event::Text(slot)) = out.get_mut(tidx) {
                    *slot = stripped;
                }
            }
        }
    }

    let (clean_title, custom_id) = extract_id(&heading_text);
    let need_id = heading_level >= 2 || custom_id.is_some() || heading_id_attr.is_some();
    let final_id = if need_id {
        let base = heading_id_attr
            .or(custom_id)
            .unwrap_or_else(|| slugify(&clean_title));
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
    if let Some(id_val) = id.clone() {
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
    // if the kind was already force-closed, the container stack will already be clean
}

fn extract_id(text: &str) -> (String, Option<String>) {
    let trimmed = text.trim_end();
    if trimmed.ends_with('}') {
        if let Some(start) = trimmed.rfind("{#") {
            if start + 2 < trimmed.len() - 1 {
                let candidate = &trimmed[start + 2..trimmed.len() - 1];
                if !candidate.is_empty()
                    && candidate
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
                {
                    let before = trimmed[..start].trim_end();
                    return (before.to_string(), Some(candidate.to_string()));
                }
            }
        }
    }
    (trimmed.to_string(), None)
}

fn strip_trailing_id(text: &str) -> (String, bool) {
    let (clean, id) = extract_id(text);
    (clean, id.is_some())
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

    // ---
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

    // paragraph payload
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

    // closing ---
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
