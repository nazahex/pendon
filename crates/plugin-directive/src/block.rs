//! §10.3: the block `==type` … `==` directive.
//!
//! A fence **with** a type opens a directive; a fence **alone** closes the
//! innermost open directive (LIFO; the widths need not match, they SHOULD). A
//! bare fence with nothing open is literal text, and the type is mandatory — a
//! typed fallback (`==note` → `<div>`) is the only way to get an anonymous
//! container (§10.3, OPEN-B1).
//!
//! The body is kept as the raw events between the fences, so `plugin-markdown`
//! re-lexes it as block content (headings, lists, paragraphs) once the
//! `__plugin_kind = block` node reaches it.
//!
//! The core parser wraps a fence line in a paragraph and may even split one
//! directive's body across several paragraphs, so the fence lines are recognised
//! on the emitted `Text` events and the surrounding paragraph events are trimmed
//! the way `plugin-custom` does for `:::` blocks.

use pendon_core::{Event, NodeKind};
use pendon_extra::{parse_directive_head, DirectiveHead, DirectiveMatch, DirectiveSigil};

use crate::{emit_directive, is_verbatim, resolve_head, DirectiveOptions, ParsedDirective};

/// One open block directive: its parsed head plus the raw events collected so
/// far.
struct Open {
    parsed: ParsedDirective,
    inner: Vec<Event>,
}

pub fn process(events: &[Event], options: &DirectiveOptions) -> Vec<Event> {
    let mut out: Vec<Event> = Vec::with_capacity(events.len() + 4);
    let mut stack: Vec<Open> = Vec::new();
    let mut excluded = 0usize;
    let mut index = 0usize;

    while index < events.len() {
        let event = &events[index];

        // A fence only counts at the block level: inside a code fence or raw HTML
        // a `==` run is literal text (§4.3).
        if excluded == 0 {
            if let Event::Text(line) = event {
                let trimmed = line.trim();
                if is_bare_fence(trimmed) {
                    if let Some(open) = stack.pop() {
                        close(open, options, &mut stack, &mut out);
                        index = skip_closing_line(events, index);
                        continue;
                    }
                    // Nothing open: a bare fence is literal text (§10.3).
                } else if let DirectiveMatch::Head { head, rest } = parse_directive_head(trimmed) {
                    if head.sigil == DirectiveSigil::Equal && head.type_marker.is_some() {
                        open(head, rest, options, &mut stack, &mut out);
                        index += 1;
                        continue;
                    }
                }
            }
        }

        match event {
            Event::StartNode(kind) if is_verbatim(kind) => excluded += 1,
            Event::EndNode(kind) if is_verbatim(kind) => excluded = excluded.saturating_sub(1),
            _ => {}
        }

        match stack.last_mut() {
            Some(parent) => parent.inner.push(event.clone()),
            None => out.push(event.clone()),
        }
        index += 1;
    }

    // §10.3: an unclosed directive is closed implicitly, with a warning.
    while let Some(mut open) = stack.pop() {
        open.parsed.warnings.push(
            "unclosed block directive at the end of the input; closed implicitly (§10.3)"
                .to_string(),
        );
        let mut events = Vec::new();
        emit_directive(&open.parsed, true, options, &open.inner, &mut events);
        match stack.last_mut() {
            Some(parent) => parent.inner.extend(events),
            None => out.extend(events),
        }
    }

    out
}

/// Opens a directive for a typed fence line.
fn open(
    head: DirectiveHead,
    rest: &str,
    options: &DirectiveOptions,
    stack: &mut Vec<Open>,
    out: &mut Vec<Event>,
) {
    // Drop the `<p>` the core opened for the fence line so the directive is a
    // block node rather than a paragraph (mirrors `plugin-custom`).
    match stack.last_mut() {
        Some(parent) => pop_trailing_paragraph(&mut parent.inner),
        None => pop_trailing_paragraph(out),
    }

    let (parsed, consumed) = resolve_head(&head, rest, options);
    let mut inner = Vec::new();
    // §10.3 grammar: the opening fence line may carry body text.
    let trailing = rest[consumed..].trim();
    if !trailing.is_empty() {
        inner.push(Event::Text(trailing.to_string()));
        inner.push(Event::Text("\n".to_string()));
    }
    stack.push(Open { parsed, inner });
}

/// Closes the innermost directive into its parent buffer (or the output).
fn close(mut open: Open, options: &DirectiveOptions, stack: &mut [Open], out: &mut Vec<Event>) {
    pop_trailing_paragraph(&mut open.inner);
    let mut events = Vec::new();
    emit_directive(&open.parsed, true, options, &open.inner, &mut events);
    match stack.last_mut() {
        Some(parent) => parent.inner.extend(events),
        None => out.extend(events),
    }
}

/// Skips the newline that ends the closing fence line plus the `EndNode` of the
/// paragraph the fence line lived in.
fn skip_closing_line(events: &[Event], index: usize) -> usize {
    let mut next = index + 1;
    if matches!(events.get(next), Some(Event::Text(text)) if text == "\n") {
        next += 1;
    }
    while matches!(events.get(next), Some(Event::EndNode(NodeKind::Paragraph))) {
        next += 1;
    }
    next
}

/// Removes a dangling paragraph start, so an empty fence paragraph never wraps
/// the directive in a `<p>`.
fn pop_trailing_paragraph(buffer: &mut Vec<Event>) {
    while matches!(buffer.last(), Some(Event::StartNode(NodeKind::Paragraph))) {
        buffer.pop();
    }
}

/// `true` when the whole line is a `==` run of 2..=7 (the close fence, §10.3).
fn is_bare_fence(trimmed: &str) -> bool {
    let count = trimmed.bytes().take_while(|byte| *byte == b'=').count();
    count == trimmed.len() && (2..=7).contains(&count)
}
