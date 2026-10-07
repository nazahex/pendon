use pendon_core::{Event, NodeKind};
use pendon_extra::{scan_extras_chars, to_attributes, ExtrasOptions};
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

/// Parses the heading head (§7.4 / §11) from the start of heading text:
/// `[slug]`, an optional `("title")` and the `{…}` / `@@type{…}` extras head.
///
/// §4.1: the head touches the `#` run, so nothing is trimmed from the front;
/// the retired `[.class]` / `{key: value}` forms are literal text (§14).
/// Returns (custom_id, consumed_character_count).
fn parse_heading_prefix(raw_text: &str) -> (Option<String>, usize) {
    let chars: Vec<char> = raw_text.chars().collect();
    let mut cursor = 0;
    let mut custom_id: Option<String> = None;

    // §7.4: an optional `[slug]` head, adjacent to the `#` run.
    if chars.get(cursor) == Some(&'[') {
        if let Some(close) = find_char(&chars, cursor + 1, ']') {
            let content: String = chars[cursor + 1..close].iter().collect();
            let trimmed = content.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('.') && !trimmed.starts_with('#') {
                custom_id = Some(trimmed.to_string());
                cursor = close + 1;
            }
        }
    }

    // §7.4: an optional `("title")` head, never part of the heading text.
    if chars.get(cursor) == Some(&'(') {
        if let Some(close) = find_matching_paren(&chars, cursor) {
            cursor = close + 1;
        }
    }

    // §7.4/§11: an adjacent `{…}` / `@@type{…}` head. `#id` beats the `[slug]`
    // head, which beats the extras slug (§6.2).
    if let Some((head, next)) = scan_extras_chars(&chars, cursor) {
        let parsed = to_attributes(&head, &ExtrasOptions::default());
        match parsed.value("id").map(|value| value.literal()) {
            Some(id) => custom_id = Some(id),
            None => {
                if custom_id.is_none() {
                    custom_id = parsed.value("slug").map(|value| value.literal());
                }
            }
        }
        cursor = next;
    }

    // §7.4: the whitespace between the head and the text is not part of it.
    while matches!(chars.get(cursor), Some(' ' | '\t')) {
        cursor += 1;
    }

    (custom_id, cursor)
}

/// Finds the `)` matching the `(` at `start`, counting nested pairs.
fn find_matching_paren(chars: &[char], start: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut i = start;
    while i < chars.len() {
        match chars[i] {
            '(' => depth += 1,
            ')' if depth == 1 => return Some(i),
            ')' => depth -= 1,
            _ => {}
        }
        i += 1;
    }
    None
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

fn strip_heading_marker(input: &str, level: usize) -> &str {
    let mut marker_end = 0;
    for (index, character) in input.char_indices() {
        if character == '#' {
            marker_end = index + character.len_utf8();
        } else {
            break;
        }
    }
    if marker_end != level {
        return input;
    }
    // §4.1: the head must touch the marker, so the separating space is kept here
    // and consumed by the head parser (or by the text trim).
    &input[marker_end..]
}

fn strip_number_prefix(input: &str) -> &str {
    let bytes = input.as_bytes();
    let mut index = 0;
    let mut saw_digit = false;
    while index < bytes.len() {
        if bytes[index].is_ascii_digit() {
            saw_digit = true;
            index += 1;
        } else if bytes[index] == b'.' && saw_digit {
            saw_digit = false;
            index += 1;
        } else if bytes[index].is_ascii_whitespace() {
            return if index > 0 && !saw_digit {
                input[index..].trim_start()
            } else {
                input
            };
        } else {
            return input;
        }
    }
    input
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
            // §9.5/§11 rule 3: plugin-heading swaps `Heading` for
            // `Custom(name)` when a custom template is configured, so those
            // components are headings too (see `is_heading_component`).
            Event::StartNode(kind)
                if matches!(kind, NodeKind::Heading)
                    || (matches!(kind, NodeKind::Custom(_))
                        && is_heading_component(events, idx)) =>
            {
                let (capture, consumed) = consume_heading(events, idx, kind);
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

/// Returns true when the node opening at `start_idx` is a heading component.
///
/// `plugin-heading` replaces `NodeKind::Heading` with `NodeKind::Custom(name)`
/// when a custom template is configured (§11 rule 3), so the metadata pass has
/// to recognise those components as headings as well. `level` is heading's
/// signature: no other plugin emits it on a `Custom` node — `plugin-markdown`
/// only ever writes it on `Heading` — so its presence marks the component as a
/// heading.
///
/// §13 warnings are pushed as `Diagnostic` events immediately after
/// `StartNode` and *before* the attribute run, so they are skipped alongside
/// the attributes; the first content event ends the look-ahead.
fn is_heading_component(events: &[Event], start_idx: usize) -> bool {
    for event in &events[start_idx + 1..] {
        match event {
            Event::Attribute { name, value } if name == "level" => {
                return value.parse::<usize>().is_ok();
            }
            Event::Attribute { .. } | Event::AttributeFlag { .. } | Event::Diagnostic { .. } => {
                continue
            }
            _ => return false,
        }
    }
    false
}

/// Consumes the events of one heading node opening at `start_idx`.
///
/// Only nodes of the *same* kind as `start` are counted: a `Heading` closes at
/// `EndNode(Heading)` exactly as before, and a `Custom(name)` component closes
/// at its own `EndNode(Custom(name))` even when inline children (`Strong`,
/// `Code`, …) are re-lexed inside it. Ignoring unrelated `StartNode`s keeps the
/// scan from overrunning when markdown leaves an unbalanced non-heading node
/// inside the heading.
fn consume_heading(
    events: &[Event],
    start_idx: usize,
    start: &NodeKind,
) -> (HeadingCapture, usize) {
    let mut idx = start_idx + 1;
    let mut depth = 1usize;
    let mut level: usize = 1;
    let mut text = String::new();
    let mut explicit_id: Option<String> = None;
    let mut auto_slug: Option<String> = None;

    while idx < events.len() {
        match &events[idx] {
            Event::Attribute { name, value } => {
                if name == "level" {
                    if let Ok(parsed) = value.parse::<usize>() {
                        level = parsed;
                    }
                } else if name == "id" {
                    explicit_id = Some(value.clone());
                } else if name == "slug" {
                    auto_slug = Some(value.clone());
                }
                idx += 1;
            }
            Event::Text(t) => {
                text.push_str(t);
                idx += 1;
            }
            Event::StartNode(k) if k == start => {
                depth += 1;
                idx += 1;
            }
            Event::EndNode(k) if k == start => {
                depth = depth.saturating_sub(1);
                idx += 1;
                if depth == 0 {
                    break;
                }
            }
            _ => {
                idx += 1;
            }
        }
    }

    let visible_text = strip_number_prefix(strip_heading_marker(&text, level));

    // Use the same prefix parser as plugin-heading to strip the head.
    let (prefix_id, consumed_len) = parse_heading_prefix(visible_text);
    let clean_text = visible_text[consumed_len..].trim().to_string();

    let final_text = if clean_text.is_empty() {
        visible_text.trim().to_string()
    } else {
        clean_text
    };

    (
        HeadingCapture {
            text: final_text,
            level,
            // §6.2/README: the explicit `id` beats the inline head, which beats
            // heading's auto-slug; the caller falls back to the slugified text.
            id: explicit_id.or(prefix_id).or(auto_slug),
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

#[cfg(test)]
mod tests {
    use super::*;

    /// §14: the retired `[.class]` / `{key: value}` forms are literal text.
    #[test]
    fn the_retired_legacy_head_is_literal_text() {
        let text = "[foo-bar][.extra]{ qux: \"anu\" } Foo Bar Barosa";
        let (id, consumed) = parse_heading_prefix(text);
        assert_eq!(id.as_deref(), Some("foo-bar"));
        assert_eq!(&text[consumed..], "[.extra]{ qux: \"anu\" } Foo Bar Barosa");
    }

    /// §7.4: the whitespace after the head belongs to the head.
    #[test]
    fn strips_the_spaced_head() {
        let text = "[slug] Title";
        let (id, consumed) = parse_heading_prefix(text);
        assert_eq!(id.as_deref(), Some("slug"));
        assert_eq!(&text[consumed..], "Title");
    }

    /// §4.1/§3: a bare extras head is a head; a spaced one is not.
    #[test]
    fn handles_bare_and_spaced_extras_heads() {
        let text = "{.extra} Body";
        let (id, consumed) = parse_heading_prefix(text);
        assert_eq!(id, None);
        assert_eq!(&text[consumed..], "Body");

        let text = "@@heading {.extra} Body";
        let (_, consumed) = parse_heading_prefix(text);
        assert_eq!(consumed, 0);
    }

    #[test]
    fn strips_the_extras_head() {
        let text = "[foo-bar]@@heading{.extra, qux: \"anu\"} Foo Bar Barosa";
        let (id, consumed) = parse_heading_prefix(text);
        assert_eq!(id.as_deref(), Some("foo-bar"));
        assert_eq!(&text[consumed..], "Foo Bar Barosa");
    }

    #[test]
    fn strips_the_title_head() {
        let text = "[slug](\"Boom\") Body";
        let (id, consumed) = parse_heading_prefix(text);
        assert_eq!(id.as_deref(), Some("slug"));
        assert_eq!(&text[consumed..], "Body");
    }

    #[test]
    fn extras_id_beats_the_slug_head() {
        let text = "[slug]@@heading{#explicit} Body";
        let (id, _) = parse_heading_prefix(text);
        assert_eq!(id.as_deref(), Some("explicit"));

        let text = "@@heading{`extras-slug`} Body";
        let (id, _) = parse_heading_prefix(text);
        assert_eq!(id.as_deref(), Some("extras-slug"));
    }

    #[test]
    fn malformed_extras_are_kept_in_the_text() {
        let text = "@@heading{`unterminated} Body";
        let (id, consumed) = parse_heading_prefix(text);
        assert_eq!(id, None);
        assert_eq!(consumed, 0);
    }

    fn attr(name: &str, value: &str) -> Event {
        Event::Attribute {
            name: name.to_string(),
            value: value.to_string(),
        }
    }

    /// §11 rule 3/§9.5: `plugin-heading` swaps `NodeKind::Heading` for
    /// `NodeKind::Custom(name)` when a custom template is configured (the
    /// heading demo renders `<DocHeading>`), so the metadata pass must still
    /// extract it — otherwise `export const headings` disappears entirely.
    #[test]
    fn extracts_a_custom_heading_component() {
        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Section),
            attr("id", "foo-bar"),
            Event::StartNode(NodeKind::Custom("DocHeading".to_string())),
            attr("name", "DocHeading"),
            attr("level", "2"),
            attr("raw_title", "Foo Bar Barosa"),
            Event::Text("Foo Bar Barosa\n".to_string()),
            Event::EndNode(NodeKind::Custom("DocHeading".to_string())),
            Event::EndNode(NodeKind::Section),
            Event::EndNode(NodeKind::Document),
        ];

        let headings = collect_headings(&events);
        assert_eq!(headings.len(), 1);
        // §9.5: the surrounding section owns the id.
        assert_eq!(headings[0].id, "foo-bar");
        assert_eq!(headings[0].text, "Foo Bar Barosa");
        assert_eq!(headings[0].level, 2);
    }

    /// Without a section the component's own auto-slug is the id (§6.2).
    #[test]
    fn custom_heading_without_a_section_falls_back_to_the_slug() {
        let events = vec![
            Event::StartNode(NodeKind::Custom("DocHeading".to_string())),
            attr("level", "3"),
            attr("slug", "deep-title"),
            Event::Text("Deep Title".to_string()),
            Event::EndNode(NodeKind::Custom("DocHeading".to_string())),
        ];

        let headings = collect_headings(&events);
        assert_eq!(headings.len(), 1);
        assert_eq!(headings[0].id, "deep-title");
        assert_eq!(headings[0].text, "Deep Title");
        assert_eq!(headings[0].level, 3);
    }

    /// §13 warnings are pushed as `Diagnostic` events immediately after
    /// `StartNode` and *before* the attribute run, so the look-ahead that
    /// recognises the component must skip them.
    #[test]
    fn detects_a_custom_heading_component_after_diagnostics() {
        let events = vec![
            Event::StartNode(NodeKind::Custom("DocHeading".to_string())),
            Event::Diagnostic {
                severity: pendon_core::Severity::Warning,
                message: "[heading] extras `#id` replaced the `[slug]` head (§6.2)".to_string(),
                span: None,
            },
            attr("level", "4"),
            Event::Text("Warned".to_string()),
            Event::EndNode(NodeKind::Custom("DocHeading".to_string())),
        ];

        let headings = collect_headings(&events);
        assert_eq!(headings.len(), 1);
        assert_eq!(headings[0].level, 4);
    }

    /// A custom component without a `level` (a directive, table cell, …) is not
    /// a heading and must never be extracted.
    #[test]
    fn ignores_non_heading_components() {
        let events = vec![
            Event::StartNode(NodeKind::Custom("Note".to_string())),
            attr("name", "Note"),
            Event::Text("hello".to_string()),
            Event::EndNode(NodeKind::Custom("Note".to_string())),
        ];

        assert!(collect_headings(&events).is_empty());
    }

    /// The scan is depth-tracked, so inline children re-lexed inside the
    /// component are traversed without ending the heading early.
    #[test]
    fn custom_heading_collects_text_across_inline_children() {
        let events = vec![
            Event::StartNode(NodeKind::Custom("DocHeading".to_string())),
            attr("level", "2"),
            Event::Text("Hello ".to_string()),
            Event::StartNode(NodeKind::Strong),
            Event::Text("World".to_string()),
            Event::EndNode(NodeKind::Strong),
            Event::EndNode(NodeKind::Custom("DocHeading".to_string())),
        ];

        let headings = collect_headings(&events);
        assert_eq!(headings.len(), 1);
        assert_eq!(headings[0].text, "Hello World");
    }
}
