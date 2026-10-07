//! §9.1: the **decorator line** and its binder.
//!
//! A decorator line is a line whose entire content is `@@type{…}` / `@@{…}`
//! (optional leading indentation, nothing after the head, §4.3 for the literal
//! cases). It decorates the **next block node** at the same nesting level and is
//! consumed: it MUST NOT be rendered.
//!
//! This module owns the rule and the event-level mechanics (finding the line,
//! dropping the paragraph the core parser wrapped it in, the
//! consecutive-decorators and no-following-block warnings). The plugins own
//! *what* a decorator may attach to, through the `target` callback
//! [`bind_decorators`] takes:
//!
//! * `plugin-blockquote` accepts a `Paragraph` whose first line opens a quote
//!   (`> …`),
//! * `plugin-list` accepts a `Paragraph` whose first line opens a list item.
//!
//! The binder returns the stream with every consumed decorator line removed,
//! each decorator bound to the target node it decorates, and the §9.1 warnings
//! for the decorators that were dropped. A decorator whose following block the
//! `target` declines is left in the stream, so another plugin's binder can
//! still claim it (the plugins share one stream; §9.1–§9.4).

use pendon_core::{Event, NodeKind};

use crate::typed::{parse_extras, to_attributes, Attrs, ExtrasMatch, ExtrasOptions};

/// Nodes whose text is copied verbatim (§4.3): a decorator-looking line inside
/// a code fence or raw HTML is literal text.
fn is_verbatim(kind: &NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::CodeFence | NodeKind::InlineCode | NodeKind::HtmlBlock | NodeKind::HtmlInline
    )
}

/// One decorator line: the indentation it was written at and the head it
/// carried.
#[derive(Debug, Clone, PartialEq)]
pub struct DecoratorLine {
    /// Leading spaces before the head, the §9.1 indentation rule.
    pub indent: usize,
    /// The head's type marker, when it had one. It becomes the type marker of
    /// the decorated node and drives §11 component selection.
    pub type_marker: Option<String>,
    /// The head's attributes, in canonical order (§6.4).
    pub attrs: Attrs,
    /// Whether the line carried the explicit `@@` sigil.
    ///
    /// §9.1 makes the bare `@@`-less `{…}` spelling a decorator only when it can
    /// actually bind: `{…}` alone is ambiguous with literal text (a bare head at
    /// the start of a line), so a bare line with **no following block** stays
    /// literal instead of being dropped. The explicit `@@…` forms are decorators
    /// whatever follows, so they are still dropped with a warning (§9.1).
    pub explicit: bool,
}

/// A decorator bound to the block node at `target_index` of
/// [`Bindings::events`].
#[derive(Debug, Clone, PartialEq)]
pub struct BoundDecorator {
    pub target_index: usize,
    pub type_marker: Option<String>,
    pub attrs: Attrs,
}

/// Why a decorator line was dropped (§9.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecoratorDrop {
    /// No block node follows it: end of input, or the next sibling is not a
    /// block.
    NoFollowingBlock,
    /// A later decorator line of the same run applies instead. Only the
    /// **last** one applies; earlier ones are dropped.
    Overridden,
}

/// A decorator line that bound to nothing.
#[derive(Debug, Clone, PartialEq)]
pub struct DroppedDecorator {
    pub type_marker: Option<String>,
    pub reason: DecoratorDrop,
}

/// The result of binding the decorator lines of one event stream.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Bindings {
    /// The input stream with every consumed decorator line removed.
    pub events: Vec<Event>,
    /// One entry per decorated block, in document order.
    pub bound: Vec<BoundDecorator>,
    /// One entry per decorator line that was dropped, in document order.
    pub dropped: Vec<DroppedDecorator>,
}

/// §9.1: scans one line as a decorator.
///
/// Returns `None` for a blank line, for a line that is not a head at all and for
/// a malformed head (§4.3 keeps those literal). Trailing text after the head
/// also disqualifies the line: a decorator is the line's *entire* content.
pub fn parse_decorator_line(line: &str, options: &ExtrasOptions) -> Option<DecoratorLine> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    let ExtrasMatch::Head { head, rest } = parse_extras(trimmed) else {
        return None;
    };
    if !rest.trim().is_empty() {
        return None;
    }
    Some(DecoratorLine {
        indent: line.len() - line.trim_start_matches(' ').len(),
        type_marker: head.type_marker.clone(),
        attrs: to_attributes(&head, options),
        explicit: trimmed.starts_with("@@"),
    })
}

/// The span of the `Paragraph` that holds nothing but the decorator line at
/// `line` (the core parser wraps every decorator line in its own paragraph).
///
/// `None` when the line shares its paragraph with other content: §9.1 then
/// treats it as ordinary text, never as a decorator.
fn decorator_paragraph(events: &[Event], line: usize) -> Option<(usize, usize)> {
    let mut start = None;
    let mut i = line;
    while i > 0 {
        i -= 1;
        match &events[i] {
            Event::StartNode(NodeKind::Paragraph) => {
                start = Some(i);
                break;
            }
            Event::Text(_) => {}
            _ => return None,
        }
    }
    let start = start?;

    let mut end = None;
    for (j, event) in events.iter().enumerate().skip(line + 1) {
        match event {
            Event::EndNode(NodeKind::Paragraph) => {
                end = Some(j);
                break;
            }
            Event::Text(_) => {}
            _ => return None,
        }
    }
    Some((start, end?))
}

/// The decorator lines of `events`, each with the paragraph it owns.
fn decorator_lines(
    events: &[Event],
    options: &ExtrasOptions,
) -> Vec<(usize, DecoratorLine, (usize, usize))> {
    let mut excluded = 0usize;
    let mut raw: Vec<(usize, DecoratorLine)> = Vec::new();
    for (i, event) in events.iter().enumerate() {
        match event {
            Event::StartNode(kind) if is_verbatim(kind) => excluded += 1,
            Event::EndNode(kind) if is_verbatim(kind) => excluded = excluded.saturating_sub(1),
            Event::Text(text) if excluded == 0 => {
                if let Some(line) = parse_decorator_line(text, options) {
                    raw.push((i, line));
                }
            }
            _ => {}
        }
    }

    // A decorator owns its line, but consecutive decorators share one paragraph:
    // the paragraph may hold nothing but decorator lines and whitespace (§9.1).
    let indices: Vec<usize> = raw.iter().map(|(index, _)| *index).collect();
    let mut lines = Vec::new();
    for (index, line) in raw {
        let Some((start, end)) = decorator_paragraph(events, index) else {
            continue;
        };
        let only_decorators = ((start + 1)..end).all(|j| match &events[j] {
            Event::Text(text) if text.trim().is_empty() => true,
            Event::Text(_) => indices.binary_search(&j).is_ok(),
            _ => false,
        });
        if only_decorators {
            lines.push((index, line, (start, end)));
        }
    }
    lines
}

fn mark_removed(removed: &mut [bool], (start, end): (usize, usize)) {
    for flag in &mut removed[start..=end] {
        *flag = true;
    }
}

/// §9.1: splits a paragraph whose *leading* lines are decorator lines into a
/// decorator-only paragraph followed by the block it decorates.
///
/// The core parser (`pendon-core`) is a blank-line paragraph splitter, so the
/// canonical spelling where a decorator line touches its block — `@@{.u}`
/// immediately above `- one`, or `@@aside{…}` immediately above `> quote` —
/// arrives as a single `Paragraph` holding both. The binder needs the decorator
/// in a paragraph of its own (as in the blank-line spelling), so this rewrites
/// such a paragraph into two. A paragraph that is *only* decorator lines (the
/// blank-line spelling) or that has no non-blank content after the run is left
/// untouched, so the transform is idempotent.
fn split_leading_decorator_paragraphs(events: &[Event], options: &ExtrasOptions) -> Vec<Event> {
    let mut out: Vec<Event> = Vec::with_capacity(events.len() + 4);
    let mut i = 0;
    while i < events.len() {
        if !matches!(events[i], Event::StartNode(NodeKind::Paragraph)) {
            out.push(events[i].clone());
            i += 1;
            continue;
        }
        let Some(end) = (i + 1..events.len())
            .find(|&j| matches!(events[j], Event::EndNode(NodeKind::Paragraph)))
        else {
            out.push(events[i].clone());
            i += 1;
            continue;
        };
        match leading_decorator_split(&events[i + 1..end], options) {
            Some(content_start) => {
                out.push(Event::StartNode(NodeKind::Paragraph));
                out.extend(events[i + 1..i + 1 + content_start].iter().cloned());
                out.push(Event::EndNode(NodeKind::Paragraph));
                out.push(Event::StartNode(NodeKind::Paragraph));
                out.extend(events[i + 1 + content_start..end].iter().cloned());
                out.push(Event::EndNode(NodeKind::Paragraph));
            }
            None => out.extend(events[i..=end].iter().cloned()),
        }
        i = end + 1;
    }
    out
}

/// The offset inside a paragraph's inner events where a leading run of decorator
/// lines ends and the decorated block begins. `None` when the paragraph does not
/// start with a decorator run or has no non-blank content after it.
fn leading_decorator_split(inner: &[Event], options: &ExtrasOptions) -> Option<usize> {
    let mut k = 0;
    let mut run_end = None;
    while k < inner.len() {
        match &inner[k] {
            Event::Text(line) if parse_decorator_line(line, options).is_some() => {
                k += 1;
                // The separator newline belongs to the decorator line.
                if matches!(inner.get(k), Some(Event::Text(text)) if text == "\n") {
                    k += 1;
                }
                run_end = Some(k);
            }
            _ => break,
        }
    }
    let run_end = run_end?;
    // Something has to follow the run for it to decorate a block. Usually that is
    // a non-blank text line; but the core parser also nests a block node — the
    // `StartNode` of a heading or a code fence — directly after the decorator
    // inside the same paragraph, with no text of its own (the fence's/heading's
    // content comes later). Both count as the decorated block.
    inner[run_end..]
        .iter()
        .position(|event| match event {
            Event::Text(text) => !text.trim().is_empty(),
            Event::StartNode(_) => true,
            _ => false,
        })
        .map(|offset| run_end + offset)
}

/// §9.1: binds every decorator line of `events` to the next block node.
///
/// `target` answers, for the `StartNode` at `index`, the indentation of that
/// block's first line when a decorator may attach to it, and `None` when the
/// node is not a decoratable block. The indentation is what the §9.1 rule
/// compares each decorator's own indentation against.
///
/// A decorator whose following block `target` **declines** is left in the
/// stream: several plugins bind decorators over the same stream (§9.1–§9.4:
/// `plugin-list`, `plugin-blockquote`, `plugin-markdown`), so a binder must
/// never steal a line that belongs to another plugin's binder. Only an
/// **explicit** `@@`-prefixed decorator with no following block at all is
/// dropped; a bare `@@`-less `{…}` head is ambiguous with literal text, so with
/// no block to bind it stays in the stream (see [`DecoratorLine::explicit`]).
///
/// A run of decorator lines binds its **last** line; the earlier ones are
/// dropped as [`DecoratorDrop::Overridden`]. When any line of the run is
/// indented deeper than the block, the whole run stays literal text (§9.1) and
/// nothing is bound.
pub fn bind_decorators(
    events: &[Event],
    options: &ExtrasOptions,
    target: impl Fn(&[Event], usize) -> Option<usize>,
) -> Bindings {
    // §9.1: the canonical spelling puts the decorator line *touching* its block
    // (`@@{.u}` immediately above `- one`), and the core wraps that whole run in
    // ONE `Paragraph`. Split the paragraph's leading decorator lines off so the
    // binder sees a decorator-only paragraph followed by the block, exactly as
    // it does for the blank-line spelling. Without this, a touching decorator
    // shared its paragraph with the block and was silently treated as text.
    let normalized = split_leading_decorator_paragraphs(events, options);
    let events: &[Event] = &normalized;
    let candidates = decorator_lines(events, options);
    let inside = |index: usize| {
        candidates
            .iter()
            .any(|(_, _, (start, end))| (*start..=*end).contains(&index))
    };

    let mut removed = vec![false; events.len()];
    let mut bound: Vec<(usize, BoundDecorator)> = Vec::new();
    let mut dropped: Vec<DroppedDecorator> = Vec::new();
    let mut pending: Vec<(DecoratorLine, (usize, usize))> = Vec::new();

    for (i, event) in events.iter().enumerate() {
        if inside(i) {
            // The decorator line and the blank leftovers of its paragraph: it
            // neither binds nor terminates the run.
            if let Some((_, line, span)) = candidates.iter().find(|(index, ..)| *index == i) {
                pending.push((line.clone(), *span));
            }
            continue;
        }
        if pending.is_empty() {
            continue;
        }

        let target_indent = match event {
            Event::StartNode(_) => Some(target(events, i)),
            Event::Text(text) if text.trim().is_empty() => continue,
            Event::Attribute { .. } | Event::AttributeFlag { .. } | Event::Diagnostic { .. } => {
                continue
            }
            // Any other event (a non-blank text, the end of the document) means
            // no block node follows.
            _ => None,
        };

        match target_indent {
            // §9.1: a decorator indented deeper than the block it would attach
            // to is literal text, so the whole run is left alone.
            Some(Some(indent)) if pending.iter().any(|(line, _)| line.indent > indent) => {
                pending.clear()
            }
            Some(Some(_)) => {
                let (line, span) = pending.pop().expect("pending is not empty");
                for (overridden, overridden_span) in pending.drain(..) {
                    dropped.push(DroppedDecorator {
                        type_marker: overridden.type_marker,
                        reason: DecoratorDrop::Overridden,
                    });
                    mark_removed(&mut removed, overridden_span);
                }
                mark_removed(&mut removed, span);
                bound.push((
                    i,
                    BoundDecorator {
                        target_index: 0,
                        type_marker: line.type_marker,
                        attrs: line.attrs,
                    },
                ));
            }
            // §9.1: the following block node is not one this plugin decorates,
            // so the run is left in place for another plugin's binder to claim.
            // Dropping it here would steal the decorator from its real owner.
            Some(None) => pending.clear(),
            // No block node follows at all. An explicit `@@…` decorator is
            // dropped with a warning; a bare `{…}` head is only a decorator when
            // it can bind (§9.1), so it is left in the stream as literal text.
            None => {
                for (line, span) in pending.drain(..) {
                    if !line.explicit {
                        continue;
                    }
                    dropped.push(DroppedDecorator {
                        type_marker: line.type_marker,
                        reason: DecoratorDrop::NoFollowingBlock,
                    });
                    mark_removed(&mut removed, span);
                }
            }
        }
    }
    for (line, span) in pending.drain(..) {
        if !line.explicit {
            continue;
        }
        dropped.push(DroppedDecorator {
            type_marker: line.type_marker,
            reason: DecoratorDrop::NoFollowingBlock,
        });
        mark_removed(&mut removed, span);
    }

    // Emit the stream without the consumed lines and remap the bound targets.
    let mut output: Vec<Event> = Vec::with_capacity(events.len());
    let mut positions: Vec<Option<usize>> = vec![None; events.len()];
    for (i, event) in events.iter().enumerate() {
        if removed[i] {
            continue;
        }
        positions[i] = Some(output.len());
        output.push(event.clone());
    }
    let bound = bound
        .into_iter()
        .filter_map(|(index, mut binding)| {
            binding.target_index = positions[index]?;
            Some(binding)
        })
        .collect();

    Bindings {
        events: output,
        bound,
        dropped,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pendon_core::{parse, Options};

    fn document(src: &str) -> Vec<Event> {
        parse(src, &Options::default())
    }

    /// The first line of the paragraph that opens at or after `index`.
    fn first_line(events: &[Event], index: usize) -> Option<&str> {
        for event in events.iter().skip(index) {
            match event {
                Event::Text(text) => return Some(text),
                Event::Attribute { .. } | Event::AttributeFlag { .. } => continue,
                _ => return None,
            }
        }
        None
    }

    /// A `target` rule shaped like `plugin-list`'s: a paragraph whose first
    /// line opens a bullet item. The returned indentation is the block's own.
    fn list_target(events: &[Event], index: usize) -> Option<usize> {
        if !matches!(events[index], Event::StartNode(NodeKind::Paragraph)) {
            return None;
        }
        let line = first_line(events, index + 1)?;
        let marker = line.trim_start();
        if !(marker.starts_with("- ") || marker.starts_with("* ") || marker.starts_with("+ ")) {
            return None;
        }
        Some(line.len() - marker.len())
    }

    fn texts(events: &[Event]) -> String {
        events
            .iter()
            .filter_map(|event| match event {
                Event::Text(text) => Some(text.as_str()),
                _ => None,
            })
            .collect()
    }

    fn bind(src: &str) -> Bindings {
        let events = document(src);
        bind_decorators(&events, &ExtrasOptions::default(), list_target)
    }

    /// §9.1: a decorator line decorates the next block and is consumed.
    #[test]
    fn a_decorator_binds_to_the_next_list() {
        let bindings = bind("@@{.u, #l03}\n\n- one\n- two\n");
        assert!(bindings.dropped.is_empty(), "{:?}", bindings.dropped);
        assert_eq!(bindings.bound.len(), 1, "{:?}", bindings.bound);

        let bound = &bindings.bound[0];
        assert_eq!(bound.type_marker, None);
        let target = &bindings.events[bound.target_index];
        assert!(
            matches!(target, Event::StartNode(NodeKind::Paragraph)),
            "{target:?}"
        );
        // The lines of the list survive; the decorator line does not.
        assert!(texts(&bindings.events).contains("- one\n"));
        assert!(
            !texts(&bindings.events).contains("@@"),
            "{:?}",
            bindings.events
        );
    }

    /// §3/§9.1: the `@@` prefix is optional and a type-only head is a decorator.
    #[test]
    fn bare_and_type_only_decorators_are_recognised() {
        let bindings = bind("{.u}\n\n- one\n");
        assert_eq!(bindings.bound.len(), 1);
        assert_eq!(bindings.bound[0].type_marker, None);

        let bindings = bind("@@unorderedA\n\n- one\n");
        assert_eq!(bindings.bound.len(), 1);
        assert_eq!(bindings.bound[0].type_marker.as_deref(), Some("unorderedA"));
    }

    /// §9.1: only the **last** decorator of a run applies.
    #[test]
    fn consecutive_decorators_keep_only_the_last() {
        let bindings = bind("@@first{.a}\n@@second{.b}\n\n- one\n");
        assert_eq!(bindings.bound.len(), 1, "{:?}", bindings.bound);
        assert_eq!(bindings.bound[0].type_marker.as_deref(), Some("second"));
        assert_eq!(
            bindings.dropped,
            vec![DroppedDecorator {
                type_marker: Some("first".to_string()),
                reason: DecoratorDrop::Overridden,
            }]
        );
        // Both lines are consumed.
        assert!(!texts(&bindings.events).contains("@@"));
    }

    /// §9.1: a decorator with no following block is dropped with a warning.
    #[test]
    fn a_decorator_without_a_following_block_is_dropped() {
        let bindings = bind("@@{.u}\n");
        assert!(bindings.bound.is_empty());
        assert_eq!(
            bindings.dropped,
            vec![DroppedDecorator {
                type_marker: None,
                reason: DecoratorDrop::NoFollowingBlock,
            }]
        );
        assert!(!texts(&bindings.events).contains("@@"));
    }

    /// §9.1: a bare `{…}` head with no following block is ambiguous with literal
    /// text (a bare head at the start of a line), so it stays literal instead of
    /// being dropped. Only the explicit `@@…` forms are unambiguous decorators.
    #[test]
    fn a_bare_head_without_a_following_block_stays_literal() {
        let bindings = bind("{foo}\n");
        assert!(bindings.bound.is_empty());
        assert!(bindings.dropped.is_empty(), "{:?}", bindings.dropped);
        assert!(
            texts(&bindings.events).contains("{foo}"),
            "{:?}",
            bindings.events
        );
    }

    /// §9.1/§9.4: a decorator whose following block `target` declines is *left
    /// in place*, so another plugin's binder (`plugin-blockquote`,
    /// `plugin-markdown`) can still claim it. Consuming it here would steal the
    /// decorator from its real owner.
    #[test]
    fn a_declined_block_leaves_the_decorator_for_another_plugin() {
        let bindings = bind("@@{.u}\n\nJust a paragraph.\n");
        assert!(bindings.bound.is_empty());
        assert!(bindings.dropped.is_empty(), "{:?}", bindings.dropped);
        // Both the paragraph and the decorator line are untouched.
        assert!(texts(&bindings.events).contains("Just a paragraph."));
        assert!(
            texts(&bindings.events).contains("@@{.u}"),
            "{:?}",
            bindings.events
        );
    }

    /// §9.1: a decorator indented deeper than the block stays literal text.
    #[test]
    fn a_decorator_deeper_than_the_block_stays_literal() {
        let bindings = bind("  @@{.u}\n\n- one\n");
        assert!(bindings.bound.is_empty(), "{:?}", bindings.bound);
        assert!(bindings.dropped.is_empty());
        assert!(texts(&bindings.events).contains("@@"));
    }

    /// §9.1: trailing text after the head is not a decorator line at all.
    #[test]
    fn trailing_text_disqualifies_a_decorator_line() {
        let bindings = bind("@@{.u} trailing\n\n- one\n");
        assert!(bindings.bound.is_empty());
        assert!(bindings.dropped.is_empty());
        assert!(texts(&bindings.events).contains("@@{.u} trailing"));
    }

    /// §4.3/§9.1: a decorator-looking line inside a code fence is literal.
    #[test]
    fn a_decorator_inside_a_code_fence_is_literal() {
        let bindings = bind("```\n@@{.u}\n```\n\n- one\n");
        assert!(bindings.bound.is_empty());
        assert!(bindings.dropped.is_empty());
        assert!(texts(&bindings.events).contains("@@{.u}"));
    }

    /// §9.1: a decorator owns its line, so the paragraph that held it — and the
    /// empty `<p>` it would have rendered — is consumed with it.
    #[test]
    fn the_decorator_paragraph_is_removed() {
        let bindings = bind("@@{.u}\n\n- one\n");
        let paragraphs = bindings
            .events
            .iter()
            .filter(|event| matches!(event, Event::StartNode(NodeKind::Paragraph)))
            .count();
        assert_eq!(paragraphs, 1, "{:?}", bindings.events);
    }

    #[test]
    fn parse_decorator_line_rejects_non_heads() {
        let options = ExtrasOptions::default();
        assert!(parse_decorator_line("", &options).is_none());
        assert!(parse_decorator_line("   ", &options).is_none());
        assert!(parse_decorator_line("plain text", &options).is_none());
        assert!(parse_decorator_line("@@{.a", &options).is_none());
        assert!(parse_decorator_line("@@{.a} tail", &options).is_none());

        let line = parse_decorator_line("  @@note{.a}", &options).expect("decorator line");
        assert_eq!(line.indent, 2);
        assert_eq!(line.type_marker.as_deref(), Some("note"));
        assert!(line.attrs.has("class"));
    }

    /// §9.1 canonical spelling: the decorator line touches its block, so the
    /// core wraps both in ONE paragraph. The binder must still bind it (it
    /// normalizes the paragraph first).
    #[test]
    fn a_touching_decorator_binds_without_a_blank_line() {
        let bindings = bind("@@{.u, #l03}\n- one\n- two\n");
        assert!(bindings.dropped.is_empty(), "{:?}", bindings.dropped);
        assert_eq!(bindings.bound.len(), 1, "{:?}", bindings.bound);
        let target = &bindings.events[bindings.bound[0].target_index];
        assert!(
            matches!(target, Event::StartNode(NodeKind::Paragraph)),
            "{target:?}"
        );
        // The marker lines survive; the decorator line does not.
        assert!(texts(&bindings.events).contains("- one\n"));
        assert!(
            !texts(&bindings.events).contains("@@"),
            "{:?}",
            bindings.events
        );
    }

    /// §9.1: consecutive decorator lines touching the block keep only the last.
    #[test]
    fn consecutive_touching_decorators_keep_only_the_last() {
        let bindings = bind("@@first{.a}\n@@second{.b}\n- one\n");
        assert_eq!(bindings.bound.len(), 1, "{:?}", bindings.bound);
        assert_eq!(bindings.bound[0].type_marker.as_deref(), Some("second"));
        assert_eq!(
            bindings.dropped,
            vec![DroppedDecorator {
                type_marker: Some("first".to_string()),
                reason: DecoratorDrop::Overridden,
            }]
        );
        assert!(!texts(&bindings.events).contains("@@"));
    }

    /// §9.1: a touching decorator in front of a non-decoratable block is left in
    /// the stream (another plugin's binder may claim it), never dropped.
    #[test]
    fn a_touching_decorator_on_a_declined_block_stays_literal() {
        let bindings = bind("@@{.u}\nJust a paragraph.\n");
        assert!(bindings.bound.is_empty());
        assert!(bindings.dropped.is_empty(), "{:?}", bindings.dropped);
        assert!(texts(&bindings.events).contains("@@{.u}"));
        assert!(texts(&bindings.events).contains("Just a paragraph."));
    }

    /// §9.1: a decorator run with no non-blank content after it (the blank-line
    /// spelling, or end of input) is not split into an empty paragraph.
    #[test]
    fn a_decorator_only_paragraph_is_left_untouched() {
        let bindings = bind("@@{.u}\n\n- one\n");
        let paragraphs = bindings
            .events
            .iter()
            .filter(|event| matches!(event, Event::StartNode(NodeKind::Paragraph)))
            .count();
        assert_eq!(paragraphs, 1, "{:?}", bindings.events);
    }
}
