//! §10.2: the inline `::type…::` directive.
//!
//! The colon sigil is shared with the block form's markup but has its own rules:
//! the opening run is 2..=7 colons, the **closing run must be at least as long**
//! as the opening run, and the content between them is inline content. An inner
//! directive with fewer colons closes implicitly at the outer's close (§10.2).
//!
//! Everything that does not parse as a directive (`::`, `:: text ::`, an
//! unterminated `::note x`) stays literal text (§4.3).

use pendon_core::Event;
use pendon_extra::{parse_directive_head, parse_extras, DirectiveMatch};

use crate::{emit_directive, is_verbatim, resolve_head, DirectiveOptions};

/// One matched inline directive: its parsed head, its content and the byte index
/// just past the closing colon run.
struct Found {
    parsed: crate::ParsedDirective,
    content: String,
    end: usize,
}

pub fn process(events: &[Event], options: &DirectiveOptions) -> Vec<Event> {
    let mut out = Vec::with_capacity(events.len());
    let mut excluded = 0usize;

    for event in events {
        match event {
            Event::StartNode(kind) => {
                if is_verbatim(kind) {
                    excluded += 1;
                }
                out.push(event.clone());
            }
            Event::EndNode(kind) => {
                if is_verbatim(kind) {
                    excluded = excluded.saturating_sub(1);
                }
                out.push(event.clone());
            }
            Event::Text(text) if excluded == 0 => process_line(text, options, &mut out),
            other => out.push(other.clone()),
        }
    }

    out
}

/// Splits one source line into plain text and inline directive nodes.
///
/// The core parser keeps every source line in its own `Text` event, so a
/// directive never spans two events. Adjacent text events (a heading marker, an
/// indentation run, …) are handled independently, exactly like `plugin-marker`.
fn process_line(text: &str, options: &DirectiveOptions, out: &mut Vec<Event>) {
    let bytes = text.as_bytes();
    let mut plain_start = 0usize;
    let mut cursor = 0usize;

    while cursor < bytes.len() {
        if bytes[cursor] == b':' {
            if let Some(found) = scan(text, cursor) {
                if cursor > plain_start {
                    out.push(Event::Text(text[plain_start..cursor].to_string()));
                }
                let mut children = Vec::new();
                if !found.content.is_empty() {
                    // Nested inline directives inside the content bind as well.
                    process_line(&found.content, options, &mut children);
                }
                emit_directive(&found.parsed, false, options, &children, out);
                cursor = found.end;
                plain_start = cursor;
                continue;
            }
        }
        cursor += 1;
    }

    if plain_start < text.len() {
        out.push(Event::Text(text[plain_start..].to_string()));
    }
}

/// Scans an inline directive at `start` (the first colon of a run).
///
/// Returns `None` when the text is not a directive, which keeps `::`, a bare
/// `::`-plus-whitespace and an unterminated opener as literal text (§4.3).
fn scan(text: &str, start: usize) -> Option<Found> {
    let bytes = text.as_bytes();
    let mut count = 0usize;
    while bytes.get(start + count) == Some(&b':') {
        count += 1;
    }
    if !(2..=7).contains(&count) {
        return None;
    }

    let DirectiveMatch::Head { head, rest } = parse_directive_head(&text[start..]) else {
        return None;
    };
    // §10.2/§10.3: the type is mandatory; only a head with one can open.
    head.type_marker.as_ref()?;

    let (parsed, consumed_extras) = resolve_head(&head, rest);
    let content_start = text.len() - rest.len() + consumed_extras;

    let (content_end, close_len) = closing_run(text, content_start, count)?;
    Some(Found {
        parsed,
        content: text[content_start..content_end].to_string(),
        end: content_end + close_len,
    })
}

/// Finds the closing colon run of an inline directive.
///
/// Returns the byte index of the run and its length. A colon run opens a nested
/// directive when it is followed by a legal head, so the stack counts the nesting
/// depth; a run that is not long enough to close the innermost directive is
/// literal content (§10.2).
fn closing_run(text: &str, content_start: usize, opening: usize) -> Option<(usize, usize)> {
    let bytes = text.as_bytes();
    let mut stack = vec![opening];
    let mut cursor = content_start;

    while cursor < bytes.len() {
        let offset = text[cursor..].find(':')?;
        let run = cursor + offset;
        let mut len = 0usize;
        while bytes.get(run + len) == Some(&b':') {
            len += 1;
        }

        if !(2..=7).contains(&len) {
            cursor = run + len.max(1);
            continue;
        }

        if let DirectiveMatch::Head { head, rest } = parse_directive_head(&text[run..]) {
            if head.type_marker.is_some() {
                // A nested opener: skip its head (and extras) and keep scanning.
                let mut nested_end = text.len() - rest.len();
                if let pendon_extra::ExtrasMatch::Head { rest: after, .. } = parse_extras(rest) {
                    nested_end = text.len() - after.len();
                }
                stack.push(len);
                cursor = nested_end;
                continue;
            }
        }

        if len >= *stack.last().unwrap_or(&opening) {
            stack.pop();
            if stack.is_empty() {
                return Some((run, len));
            }
        }
        cursor = run + len;
    }

    None
}
