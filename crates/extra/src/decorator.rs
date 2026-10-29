//! §9.1: the **decorator line** and its binder.
//!
//! A decorator line is a line whose entire content is `@@type{…}` / `@@{…}`
//! (optional leading indentation, nothing after the head, §4.3 for the literal
//! cases). The `@@` sigil is **mandatory** (D-10, ADR-0013): a bare `{…}` line is
//! literal text, never a decorator, because without `@@` a head is only ever
//! *adjacent* to its construct. The head itself keeps one order — `@@type` → slot
//! → `{specs}` → `%%numbering` (§5). A decorator decorates the **next block
//! node** at the same nesting level and is consumed: it MUST NOT be rendered.
//!
//! This module owns the rule, the event-level mechanics (finding the line,
//! dropping the paragraph the core parser wrapped it in, the
//! consecutive-decorators and no-following-block warnings) **and the claim
//! table**: [`Receiver`] names the five blocks a decorator may configure and
//! fixes their claim order (D-10 rule 2), and [`classify_block`] says which one
//! a given block node is. A plugin no longer decides for itself what a
//! decorator may attach to; it declares the one [`Receiver`] it owns in a
//! [`Claim`], and the binder works out the rest:
//!
//! * the block is this pass's receiver → the decorator binds here;
//! * the block is another (enabled) receiver → the line stays in the stream for
//!   that receiver's pass, whatever order the chain runs in;
//! * the block can never receive a decorator, or its receiver is not in the
//!   chain → the line is dropped with a `Severity::Warning` (D-10 rule 3).
//!
//! The binder returns the stream with every consumed decorator line removed,
//! each decorator bound to the target node it decorates, and the §9.1 warnings
//! for the decorators that were dropped ([`decorator_diagnostics`]).

use pendon_core::{Event, NodeKind, Severity};

use crate::typed::{parse_extras, to_attributes, Attrs, ExtrasMatch, Specs};
use crate::SpecsResolver;

/// Nodes whose text is copied verbatim (§4.3): a decorator-looking line inside
/// a code fence or raw HTML is literal text.
fn is_verbatim(kind: &NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::CodeFence | NodeKind::InlineCode | NodeKind::HtmlBlock | NodeKind::HtmlInline
    )
}

/// D-10 rule 2: the block a decorator line configures, in claim order.
///
/// `crates/extra` owns this set and its order, not the plugins: one decorator
/// has exactly one owner, and the owner is decided by **what the block is**,
/// never by which plugin happens to read the stream first. Most specific first —
/// a heading's outline outranks the table/list/quote/paragraph reading of the
/// same line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Receiver {
    /// A heading: `plugin-section`'s outline (§9.5, D-10 rule 5).
    Section,
    /// A table: its caption / header-run / body (§8).
    Table,
    /// A list container (§9.3).
    List,
    /// A blockquote (§9.2).
    Blockquote,
    /// The catch-all: a paragraph or a code fence (§9.1).
    Paragraph,
}

impl Receiver {
    /// The claim order of D-10 rule 2, most specific first.
    pub const ORDER: [Receiver; 5] = [
        Receiver::Section,
        Receiver::Table,
        Receiver::List,
        Receiver::Blockquote,
        Receiver::Paragraph,
    ];

    /// The plugin that owns this receiver, as `task.plugin` spells it.
    pub fn plugin(self) -> &'static str {
        match self {
            Receiver::Section => "section",
            Receiver::Table => "table",
            Receiver::List => "list",
            Receiver::Blockquote => "blockquote",
            Receiver::Paragraph => "markdown",
        }
    }

    /// The receiver a `task.plugin` name owns, when it owns one. `heading`,
    /// `marker`, `directive` and every inline plugin own none: they cannot
    /// receive a decorator (D-10 rule 3).
    pub fn from_plugin(plugin: &str) -> Option<Self> {
        Receiver::ORDER
            .into_iter()
            .find(|receiver| receiver.plugin() == plugin)
    }
}

/// D-10 rules 2–3: the receivers a **chain** has.
///
/// A decorator whose block belongs to a receiver that is not in the chain can
/// never be claimed, so the last binder drops it with a `Severity::Warning`
/// instead of leaving a stray `@@…` line in the document. The default is every
/// receiver, so a plugin used on its own never rejects another owner's
/// decorator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReceiverSet(u8);

impl ReceiverSet {
    /// No receiver at all.
    pub fn none() -> Self {
        Self(0)
    }

    /// Every receiver (the standalone-plugin default).
    pub fn all() -> Self {
        Receiver::ORDER
            .iter()
            .fold(Self::none(), |set, r| set.with(*r))
    }

    /// The receivers the given `task.plugin` names own, in claim order.
    pub fn from_plugins<'a>(plugins: impl IntoIterator<Item = &'a str>) -> Self {
        plugins
            .into_iter()
            .filter_map(Receiver::from_plugin)
            .fold(Self::none(), |set, receiver| set.with(receiver))
    }

    /// This set plus `receiver`.
    pub fn with(self, receiver: Receiver) -> Self {
        Self(self.0 | 1 << receiver as u8)
    }

    pub fn contains(self, receiver: Receiver) -> bool {
        self.0 & (1 << receiver as u8) != 0
    }

    /// The members, in claim order.
    pub fn iter(self) -> impl Iterator<Item = Receiver> {
        Receiver::ORDER
            .into_iter()
            .filter(move |receiver| self.contains(*receiver))
    }
}

impl Default for ReceiverSet {
    fn default() -> Self {
        Self::all()
    }
}

/// What the block node that follows a decorator is, from the decorator's point
/// of view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// A block that receives decorators: which receiver owns it and the
    /// indentation of its first line (the §9.1 indentation rule).
    Receives(Receiver, usize),
    /// A block that can never receive a decorator; the string names it for the
    /// warning (D-10 rule 3).
    Rejects(&'static str),
}

/// D-10 rules 2–3: one binder's view of the claim table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Claim {
    /// The one receiver this pass owns.
    pub receiver: Receiver,
    /// The receivers the chain has. A pass that meets a decorator for a
    /// receiver *outside* this set drops it: no pass will ever claim it.
    pub enabled: ReceiverSet,
}

impl Claim {
    /// A pass that claims its own receiver and leaves every other block to its
    /// owner — the behaviour of every pass except the final one.
    pub fn only(receiver: Receiver) -> Self {
        Self {
            receiver,
            enabled: ReceiverSet::all(),
        }
    }

    /// The final pass (the Markdown pass): besides claiming `receiver` it also
    /// drops what no enabled receiver will claim (D-10 rule 3).
    pub fn sweeping(receiver: Receiver, enabled: ReceiverSet) -> Self {
        Self { receiver, enabled }
    }
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
    /// The specs the head's positions were resolved with (§6.1, D-5): the
    /// consumer projects `specs.*` paths through it (§6.3).
    pub specs: Specs,
    /// Literal text that followed the head on the same line, when the head was
    /// separated from it by whitespace. D-10 rule 3: that text is **dropped**
    /// with a `Severity::Warning`; the head still decorates.
    pub trailing: String,
}

/// A decorator bound to the block node at `target_index` of
/// [`Bindings::events`].
#[derive(Debug, Clone, PartialEq)]
pub struct BoundDecorator {
    pub target_index: usize,
    pub type_marker: Option<String>,
    pub attrs: Attrs,
    /// The specs the decorator's head was mapped with (§6.3).
    pub specs: Specs,
}

/// Why a decorator line was dropped (§9.1, D-10 rule 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecoratorDrop {
    /// No block node follows it: end of input, or the next sibling is not a
    /// block.
    NoFollowingBlock,
    /// A later decorator line of the same run applies instead. Only the
    /// **last** one applies; earlier ones are dropped.
    Overridden,
    /// The following block is a construct that never receives decorators — a
    /// marker, a directive, a frontmatter block, a data block, an
    /// already-emitted element. The string names the construct for the warning.
    NotAReceiver(&'static str),
    /// The following block belongs to a receiver this chain does not have, so no
    /// pass will ever claim the decorator (a heading without `plugin-section`,
    /// a table without `plugin-table`, …).
    ReceiverAbsent(Receiver),
}

/// A decorator line that bound to nothing.
#[derive(Debug, Clone, PartialEq)]
pub struct DroppedDecorator {
    pub type_marker: Option<String>,
    pub reason: DecoratorDrop,
}

/// Literal text that sat after a decorator's head and was dropped, because a
/// decorator line is a *head*, not content (D-10 rule 3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DroppedText {
    pub type_marker: Option<String>,
    pub text: String,
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
    /// One entry per bound decorator whose line carried trailing literal text.
    pub trailing: Vec<DroppedText>,
}

/// §9.1: scans one line as a decorator.
///
/// The `@@` sigil is mandatory (D-10, ADR-0013): a line that opens with a bare
/// `{…}` head is literal text — an adjacent head needs a construct to touch, and
/// a line of its own has none — so it never configures the block below it.
/// Returns `None` for a blank line, for a line without `@@`, for a line that is
/// not a head at all and for a malformed head (§4.3 keeps those literal).
///
/// Text after the head is judged by adjacency (§4.1). Text that **touches** the
/// head belongs to the head's own spelling — `@@{.a}aside` puts the type after
/// the body, which is not a head — so the whole line stays literal. Text after a
/// space is stray prose on a decorator line: the head still decorates and the
/// text comes back as [`DecoratorLine::trailing`] for the binder to drop with a
/// `Severity::Warning` (D-10 rule 3).
pub fn parse_decorator_line(line: &str, specs: SpecsResolver) -> Option<DecoratorLine> {
    let trimmed = line.trim();
    if !trimmed.starts_with("@@") {
        return None;
    }
    let ExtrasMatch::Head { head, rest } = parse_extras(trimmed) else {
        return None;
    };
    let trailing = if rest.trim().is_empty() {
        String::new()
    } else if rest.starts_with(' ') || rest.starts_with('\t') {
        rest.trim().to_string()
    } else {
        return None;
    };
    let specs = specs(head.type_marker.as_deref());
    Some(DecoratorLine {
        indent: line.len() - line.trim_start_matches(' ').len(),
        type_marker: head.type_marker.clone(),
        attrs: to_attributes(&head, &specs),
        specs,
        trailing,
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
    specs: SpecsResolver,
) -> Vec<(usize, DecoratorLine, (usize, usize))> {
    let mut excluded = 0usize;
    let mut raw: Vec<(usize, DecoratorLine)> = Vec::new();
    for (i, event) in events.iter().enumerate() {
        match event {
            Event::StartNode(kind) if is_verbatim(kind) => excluded += 1,
            Event::EndNode(kind) if is_verbatim(kind) => excluded = excluded.saturating_sub(1),
            Event::Text(text) if excluded == 0 => {
                if let Some(line) = parse_decorator_line(text, specs) {
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
fn split_leading_decorator_paragraphs(events: &[Event], specs: SpecsResolver) -> Vec<Event> {
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
        match leading_decorator_split(&events[i + 1..end], specs) {
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
fn leading_decorator_split(inner: &[Event], specs: SpecsResolver) -> Option<usize> {
    let mut k = 0;
    let mut run_end = None;
    while k < inner.len() {
        match &inner[k] {
            Event::Text(line) if parse_decorator_line(line, specs).is_some() => {
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

/// D-10 rule 2: which receiver owns the block node that starts at `index`,
/// together with the indentation of its first line (the §9.1 rule compares each
/// decorator's own indentation against it).
///
/// `None` when `index` is not a `StartNode` at all. The classification is by
/// **what the block is**, so it does not depend on which plugin already ran: a
/// `Paragraph` starting with `|` is a table whether or not `plugin-table` has
/// transformed it yet.
pub fn classify_block(events: &[Event], index: usize) -> Option<Target> {
    let Event::StartNode(kind) = events.get(index)? else {
        return None;
    };
    Some(match kind {
        NodeKind::Paragraph => classify_paragraph(events, index),
        NodeKind::Heading => Target::Receives(Receiver::Section, 0),
        NodeKind::CodeFence => {
            Target::Receives(Receiver::Paragraph, first_line_indent(events, index))
        }
        NodeKind::Element(name) => match name.as_str() {
            "ul" | "ol" => Target::Receives(Receiver::List, first_line_indent(events, index)),
            "blockquote" => {
                Target::Receives(Receiver::Blockquote, first_line_indent(events, index))
            }
            "table" => Target::Receives(Receiver::Table, first_line_indent(events, index)),
            _ => Target::Rejects("an element another plugin emitted"),
        },
        NodeKind::Custom(_) => Target::Rejects("a marker, directive or custom component"),
        NodeKind::Frontmatter => Target::Rejects("frontmatter"),
        NodeKind::HtmlBlock | NodeKind::HtmlInline => Target::Rejects("raw HTML"),
        NodeKind::ThematicBreak => Target::Rejects("a thematic break"),
        NodeKind::Document => Target::Rejects("the document"),
        _ => Target::Rejects("a node another plugin emitted"),
    })
}

/// The receiver of a `Paragraph` node, read off its first content.
fn classify_paragraph(events: &[Event], index: usize) -> Target {
    for event in events.iter().skip(index + 1) {
        match event {
            Event::Attribute { .. } | Event::AttributeFlag { .. } | Event::Diagnostic { .. } => {}
            Event::Text(text) if text.trim().is_empty() => {}
            Event::Text(text) => {
                let indent = text.len() - text.trim_start_matches(' ').len();
                return Target::Receives(receiver_of_line(text), indent);
            }
            // A decorator line that *touches* its heading or code fence shares
            // the paragraph with it, so the block is the nested node.
            Event::StartNode(NodeKind::Heading) => return Target::Receives(Receiver::Section, 0),
            Event::StartNode(NodeKind::CodeFence) => {
                return Target::Receives(Receiver::Paragraph, 0)
            }
            Event::StartNode(_) => return Target::Rejects("a node another plugin emitted"),
            Event::EndNode(_) => break,
        }
    }
    Target::Rejects("an empty paragraph")
}

/// D-10 rule 2: the receiver a block's first line opens, by its marker alone.
fn receiver_of_line(line: &str) -> Receiver {
    let trimmed = line.trim_start_matches(' ');
    if trimmed.starts_with('>') {
        Receiver::Blockquote
    } else if list_marker(trimmed) {
        Receiver::List
    } else if trimmed.starts_with('|') {
        Receiver::Table
    } else {
        Receiver::Paragraph
    }
}

/// §9.3: whether `line` opens a list item (`- `, `* `, `+ `, `1. `, `1) `).
fn list_marker(line: &str) -> bool {
    if line.starts_with("- ") || line.starts_with("* ") || line.starts_with("+ ") {
        return true;
    }
    let digits = line.chars().take_while(|c| c.is_ascii_digit()).count();
    if digits == 0 {
        return false;
    }
    let rest = &line[digits..];
    rest.starts_with(". ") || rest.starts_with(") ")
}

/// The leading spaces of the first text line of the node at `index`.
fn first_line_indent(events: &[Event], index: usize) -> usize {
    for event in events.iter().skip(index + 1) {
        match event {
            Event::Attribute { .. } | Event::AttributeFlag { .. } | Event::Diagnostic { .. } => {}
            Event::Text(text) => return text.len() - text.trim_start_matches(' ').len(),
            _ => break,
        }
    }
    0
}

/// §9.1: the `Warning` diagnostics a binder raises for what it had to drop,
/// tagged with the plugin that raised them (the same `[ctx]` prefix
/// [`issue_event`](crate::issue_event) uses).
///
/// One implementation for every plugin, so the four binders cannot drift apart.
pub fn decorator_diagnostics(bindings: &Bindings, context: &str) -> Vec<Event> {
    let mut out = Vec::with_capacity(bindings.dropped.len() + bindings.trailing.len());
    for dropped in &bindings.dropped {
        let message = match dropped.reason {
            DecoratorDrop::NoFollowingBlock => {
                "a decorator line bound to no block and was dropped".to_string()
            }
            DecoratorDrop::Overridden => {
                "a decorator line was overridden by a later decorator line and was dropped"
                    .to_string()
            }
            DecoratorDrop::NotAReceiver(reason) => format!(
                "a decorator line was dropped: the block below it is {reason} and cannot receive decorators"
            ),
            DecoratorDrop::ReceiverAbsent(receiver) => format!(
                "a decorator line was dropped: the block below it belongs to `{}`, which is not in this chain",
                receiver.plugin()
            ),
        };
        out.push(warning(context, message));
    }
    for text in &bindings.trailing {
        out.push(warning(
            context,
            format!(
                "literal text after a decorator's head was dropped ({:?})",
                text.text
            ),
        ));
    }
    out
}

/// A §9.1 `Warning` on the decorator rules, prefixed with its plugin.
fn warning(context: &str, message: String) -> Event {
    Event::Diagnostic {
        severity: Severity::Warning,
        message: format!("[{context}] {message}"),
        span: None,
    }
}

///
/// §9.1: binds every decorator line of `events` to the block node it owns.
///
/// The owner is decided by the claim table of D-10 rule 2, not by the caller:
/// [`classify_block`] names the receiver of the following block and `claim` says
/// which receiver this pass owns. A decorator for another receiver the chain
/// **has** stays in the stream for that receiver's pass, whatever order the
/// chain runs in; one for a receiver the chain does not have, or for a block
/// that can never receive a decorator, is dropped with a warning (D-10 rule 3).
/// A decorator with no following block at all is dropped the same way; every
/// decorator line carries `@@`, so there is no ambiguous spelling left in the
/// stream.
///
/// A run of decorator lines binds its **last** line; the earlier ones are
/// dropped as [`DecoratorDrop::Overridden`]. When any line of the run is
/// indented deeper than the block, the whole run stays literal text (§9.1) and
/// nothing is bound.
pub fn bind_decorators(events: &[Event], specs: SpecsResolver, claim: Claim) -> Bindings {
    // §9.1: the canonical spelling puts the decorator line *touching* its block
    // (`@@{.u}` immediately above `- one`), and the core wraps that whole run in
    // ONE `Paragraph`. Split the paragraph's leading decorator lines off so the
    // binder sees a decorator-only paragraph followed by the block, exactly as
    // it does for the blank-line spelling. Without this, a touching decorator
    // shared its paragraph with the block and was silently treated as text.
    let normalized = split_leading_decorator_paragraphs(events, specs);
    let events: &[Event] = &normalized;
    let candidates = decorator_lines(events, specs);
    let inside = |index: usize| {
        candidates
            .iter()
            .any(|(_, _, (start, end))| (*start..=*end).contains(&index))
    };

    let mut removed = vec![false; events.len()];
    let mut bound: Vec<(usize, BoundDecorator)> = Vec::new();
    let mut dropped: Vec<DroppedDecorator> = Vec::new();
    let mut trailing: Vec<DroppedText> = Vec::new();
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

        let target = match event {
            Event::StartNode(_) => classify_block(events, i),
            Event::Text(text) if text.trim().is_empty() => continue,
            Event::Attribute { .. } | Event::AttributeFlag { .. } | Event::Diagnostic { .. } => {
                continue
            }
            // Any other event (a non-blank text, the end of the document) means
            // no block node follows.
            _ => None,
        };

        match target {
            // §9.1: a decorator indented deeper than the block it would attach
            // to is literal text, so the whole run is left alone.
            Some(Target::Receives(_, indent))
                if pending.iter().any(|(line, _)| line.indent > indent) =>
            {
                pending.clear()
            }
            // D-10 rule 2: the block is this pass's receiver — bind here.
            Some(Target::Receives(owner, _)) if owner == claim.receiver => {
                let (line, span) = pending.pop().expect("pending is not empty");
                for (overridden, overridden_span) in pending.drain(..) {
                    dropped.push(DroppedDecorator {
                        type_marker: overridden.type_marker,
                        reason: DecoratorDrop::Overridden,
                    });
                    mark_removed(&mut removed, overridden_span);
                }
                mark_removed(&mut removed, span);
                if !line.trailing.is_empty() {
                    trailing.push(DroppedText {
                        type_marker: line.type_marker.clone(),
                        text: line.trailing.clone(),
                    });
                }
                bound.push((
                    i,
                    BoundDecorator {
                        target_index: 0,
                        type_marker: line.type_marker,
                        attrs: line.attrs,
                        specs: line.specs,
                    },
                ));
            }
            // D-10 rule 2: the block belongs to another receiver the chain has,
            // so the run stays in place for that receiver's pass — whichever
            // order the chain runs the two plugins in.
            Some(Target::Receives(owner, _)) if claim.enabled.contains(owner) => pending.clear(),
            // D-10 rule 3: the block can never receive a decorator, or its
            // receiver is not in this chain. Nothing will ever claim it, so it
            // is dropped with a warning instead of rendering as a stray line.
            Some(Target::Receives(owner, _)) => {
                for (line, span) in pending.drain(..) {
                    dropped.push(DroppedDecorator {
                        type_marker: line.type_marker,
                        reason: DecoratorDrop::ReceiverAbsent(owner),
                    });
                    mark_removed(&mut removed, span);
                }
            }
            Some(Target::Rejects(reason)) => {
                for (line, span) in pending.drain(..) {
                    dropped.push(DroppedDecorator {
                        type_marker: line.type_marker,
                        reason: DecoratorDrop::NotAReceiver(reason),
                    });
                    mark_removed(&mut removed, span);
                }
            }
            // No block node follows at all: the decorator is dropped with a
            // warning (§9.1).
            None => {
                for (line, span) in pending.drain(..) {
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
        trailing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Specs;
    use pendon_core::{parse, Options};

    fn document(src: &str) -> Vec<Event> {
        parse(src, &Options::default())
    }

    /// A `Claim` shaped like a list pass: it owns `Receiver::List` and the chain
    /// has every receiver.
    fn list_claim() -> Claim {
        Claim::only(Receiver::List)
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
        bind_with(src, list_claim())
    }

    fn bind_with(src: &str, claim: Claim) -> Bindings {
        let events = document(src);
        bind_decorators(&events, &|_: Option<&str>| Specs::default(), claim)
    }

    /// The type marker the claim-order fixture writes above `receiver`'s block.
    fn claim_marker(receiver: Receiver) -> &'static str {
        match receiver {
            Receiver::Section => "sectionA",
            Receiver::Table => "tableA",
            Receiver::List => "listA",
            Receiver::Blockquote => "quoteA",
            Receiver::Paragraph => "paraA",
        }
    }

    /// Runs one pass per receiver in `order`, feeding each pass the stream the
    /// previous one produced, and checks that every pass claimed exactly its own
    /// decorator — no more, no less.
    fn bind_chain(src: &str, order: &[Receiver]) {
        let mut events = document(src);
        for receiver in order {
            let bindings = bind_decorators(
                &events,
                &|_: Option<&str>| Specs::default(),
                Claim::only(*receiver),
            );
            assert!(
                bindings.dropped.is_empty(),
                "{}: {:?}",
                receiver.plugin(),
                bindings.dropped
            );
            assert_eq!(
                bindings.bound.len(),
                1,
                "{}: {:?}",
                receiver.plugin(),
                bindings.bound
            );
            assert_eq!(
                bindings.bound[0].type_marker.as_deref(),
                Some(claim_marker(*receiver)),
                "{} claimed the wrong decorator",
                receiver.plugin()
            );
            events = bindings.events;
        }
        // Five receivers, five decorator lines: the chain consumed all of them.
        assert!(!texts(&events).contains("@@"), "{events:?}");
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

    /// §3/§9.1/D-10: `@@` is mandatory, so the sigil alone is what makes a
    /// line a decorator; a type-only head still decorates.
    #[test]
    fn a_type_only_decorator_is_recognised() {
        let bindings = bind("@@unorderedA\n\n- one\n");
        assert_eq!(bindings.bound.len(), 1);
        assert_eq!(bindings.bound[0].type_marker.as_deref(), Some("unorderedA"));
    }

    /// §9.1/D-10 (S2-5.1): a bare head line is literal text, never a decorator —
    /// it does not configure the block below it, raises no diagnostic and stays
    /// in the stream exactly as written.
    #[test]
    fn a_bare_head_line_is_literal_text() {
        let bindings = bind("{.u}\n\n- one\n");
        assert!(bindings.bound.is_empty(), "{:?}", bindings.bound);
        assert!(bindings.dropped.is_empty(), "{:?}", bindings.dropped);
        assert!(
            texts(&bindings.events).contains("{.u}"),
            "{:?}",
            bindings.events
        );
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

    /// §9.1/D-10: the head order is `@@type` → slot → `{specs}` → `%%numbering`.
    /// A type written after the body is not a head at all, so the whole line is
    /// literal text and decorates nothing.
    #[test]
    fn a_type_after_the_body_is_literal_text() {
        let bindings = bind("@@{.a}aside\n\n- one\n");
        assert!(bindings.bound.is_empty(), "{:?}", bindings.bound);
        assert!(bindings.dropped.is_empty(), "{:?}", bindings.dropped);
        assert!(
            texts(&bindings.events).contains("@@{.a}aside"),
            "{:?}",
            bindings.events
        );
    }

    /// D-10 rule 2: a block that belongs to another receiver the chain **has**
    /// is left alone, so that receiver's own pass can claim it — whatever order
    /// the chain runs the plugins in.
    #[test]
    fn a_block_of_another_enabled_receiver_keeps_the_decorator() {
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

    /// D-10 rule 3: a block whose receiver is **not** in the chain is a receiver
    /// no pass will ever reach, so the decorator is dropped with a warning
    /// rather than rendered as a stray line.
    #[test]
    fn a_block_of_an_absent_receiver_drops_the_decorator() {
        let claim = Claim::sweeping(Receiver::List, ReceiverSet::none().with(Receiver::List));
        let bindings = bind_with("@@{.u}\n\nJust a paragraph.\n", claim);
        assert!(bindings.bound.is_empty(), "{:?}", bindings.bound);
        assert_eq!(
            bindings.dropped,
            vec![DroppedDecorator {
                type_marker: None,
                reason: DecoratorDrop::ReceiverAbsent(Receiver::Paragraph),
            }]
        );
        assert!(texts(&bindings.events).contains("Just a paragraph."));
        assert!(!texts(&bindings.events).contains("@@"));
    }

    /// D-10 rule 2: the claim order is by **what the block is**, not by which
    /// plugin ran first, so a list pass leaves a table's decorator alone.
    #[test]
    fn a_table_block_is_not_the_list_receivers() {
        let bindings = bind("@@{.u}\n\n| a | b |\n| - | - |\n| 1 | 2 |\n");
        assert!(bindings.bound.is_empty(), "{:?}", bindings.bound);
        assert!(bindings.dropped.is_empty(), "{:?}", bindings.dropped);
        assert!(texts(&bindings.events).contains("@@{.u}"));
    }

    /// D-10 rule 2 (S2-5.2): the claim table has one owner per block, and it is
    /// **order independent** — the same document bound by the five passes in
    /// claim order and in reverse order binds the same decorator to the same
    /// block, so a plugin's position in `task.plugin` can never change the
    /// output. Each of the five receivers claims its own line and only its own.
    #[test]
    fn each_receiver_claims_its_own_decorator_whatever_the_order() {
        let src = "@@sectionA{.s}\n\n## A heading\n\n\
                   @@tableA{.t}\n\n| a | b |\n| - | - |\n| 1 | 2 |\n\n\
                   @@listA{.l}\n\n- one\n\n\
                   @@quoteA{.q}\n\n> a quote\n\n\
                   @@paraA{.p}\n\nA paragraph.\n";
        bind_chain(src, &Receiver::ORDER);

        let mut reverse = Receiver::ORDER;
        reverse.reverse();
        bind_chain(src, &reverse);
    }

    /// D-10 rule 3: a block that can never receive a decorator drops it with a
    /// warning instead of leaving a stray `@@…` line in the output.
    #[test]
    fn a_block_that_is_not_a_receiver_drops_the_decorator() {
        let bindings = bind("@@{.u}\n\n---\n");
        assert!(bindings.bound.is_empty(), "{:?}", bindings.bound);
        assert_eq!(
            bindings.dropped,
            vec![DroppedDecorator {
                type_marker: None,
                reason: DecoratorDrop::NotAReceiver("a thematic break"),
            }]
        );
        assert!(!texts(&bindings.events).contains("@@"));
    }

    /// §9.1: a decorator indented deeper than the block stays literal text.
    #[test]
    fn a_decorator_deeper_than_the_block_stays_literal() {
        let bindings = bind("  @@{.u}\n\n- one\n");
        assert!(bindings.bound.is_empty(), "{:?}", bindings.bound);
        assert!(bindings.dropped.is_empty());
        assert!(texts(&bindings.events).contains("@@"));
    }

    /// §4.1/D-10 rule 3: text after the head is judged by adjacency. Text that
    /// **touches** the head is part of its spelling (`@@{.a}aside` is a
    /// type-after-body head), so the whole line stays literal text.
    #[test]
    fn touching_text_keeps_the_decorator_line_literal() {
        let bindings = bind("@@{.u}trailing\n\n- one\n");
        assert!(bindings.bound.is_empty(), "{:?}", bindings.bound);
        assert!(bindings.dropped.is_empty(), "{:?}", bindings.dropped);
        assert!(bindings.trailing.is_empty(), "{:?}", bindings.trailing);
        assert!(texts(&bindings.events).contains("@@{.u}trailing"));
    }

    /// D-10 rule 3: text after **whitespace** is stray prose on a head line. The
    /// head still decorates its block and the text is dropped with a warning.
    #[test]
    fn whitespace_separated_text_is_dropped_and_the_head_still_binds() {
        let bindings = bind("@@{.u} trailing\n\n- one\n");
        assert_eq!(bindings.bound.len(), 1, "{:?}", bindings.bound);
        assert!(bindings.dropped.is_empty(), "{:?}", bindings.dropped);
        assert_eq!(
            bindings.trailing,
            vec![DroppedText {
                type_marker: None,
                text: "trailing".to_string(),
            }]
        );
        assert!(!texts(&bindings.events).contains("@@"));
        assert!(!texts(&bindings.events).contains("trailing"));

        // The warning is raised from the same entry, for every binder.
        let diagnostics = decorator_diagnostics(&bindings, "list");
        assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
        assert!(matches!(
            &diagnostics[0],
            Event::Diagnostic {
                severity: Severity::Warning,
                ..
            }
        ));
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
        let specs = |_: Option<&str>| Specs::default();
        assert!(parse_decorator_line("", &specs).is_none());
        assert!(parse_decorator_line("   ", &specs).is_none());
        assert!(parse_decorator_line("plain text", &specs).is_none());
        assert!(parse_decorator_line("@@{.a", &specs).is_none());
        let line = parse_decorator_line("@@{.a} tail", &specs).expect("a decorator line");
        assert_eq!(line.trailing, "tail");
        // §9.1/D-10: `@@` is mandatory — a bare head is not a decorator line.
        assert!(parse_decorator_line("{.a}", &specs).is_none());
        assert!(parse_decorator_line("  {\"Title\"}", &specs).is_none());
        // Nor is a head whose type is not first (`type`-last is literal text).
        assert!(parse_decorator_line("@@{.a}aside", &specs).is_none());

        let line = parse_decorator_line("  @@note{.a}", &specs).expect("decorator line");
        assert_eq!(line.indent, 2);
        assert_eq!(line.type_marker.as_deref(), Some("note"));
        assert!(line.attrs.has("specs.class"));
    }

    /// §5/§9.1: a decorator reads the whole head in the one canonical order —
    /// `@@type` → slot → `{specs}` → `%%numbering` — and every part survives.
    #[test]
    fn a_decorator_reads_the_head_in_canonical_order() {
        let specs = |_: Option<&str>| Specs::default();
        let line = parse_decorator_line("@@aside[intro](\"T\"){.box}%%%numberA", &specs)
            .expect("decorator line");
        assert_eq!(line.type_marker.as_deref(), Some("aside"));
        assert!(line.attrs.has("bracket.bare"));
        assert!(line.attrs.has("paren.bare"));
        assert!(line.attrs.has("specs.class"));
        let number = line.attrs.value("__number").expect("a numbering marker");
        assert_eq!(
            number.literal(),
            "{\"depth\":2,\"type\":\"numberA\"}".to_string()
        );
    }

    /// §9.1/§6.1: `@@type[…](…){…}` and the untyped `@@[…]` are decorator
    /// lines; a plain `[x](y)` line is not (omitting `@@` means link, §4.1).
    #[test]
    fn positional_groups_make_a_decorator_line() {
        let specs = |_: Option<&str>| Specs::default();

        let line =
            parse_decorator_line("@@aside[intro](\"T\"){.box}", &specs).expect("decorator line");
        assert_eq!(line.type_marker.as_deref(), Some("aside"));
        assert_eq!(
            line.attrs
                .value("bracket.bare")
                .map(|value| value.literal()),
            Some("intro".to_string())
        );
        assert_eq!(
            line.attrs.value("paren.bare").map(|value| value.literal()),
            Some("T".to_string())
        );
        assert!(line.attrs.has("specs.class"));

        // Groups without a body and the untyped `@@[…]` form.
        let line = parse_decorator_line("@@aside[intro]", &specs).expect("decorator line");
        assert_eq!(line.type_marker.as_deref(), Some("aside"));
        assert!(line.attrs.has("bracket.bare"));

        let line = parse_decorator_line("@@[intro](\"T\")", &specs).expect("decorator line");
        assert_eq!(line.type_marker, None);
        assert!(line.attrs.has("bracket.bare"));
        assert!(line.attrs.has("paren.bare"));

        // A link paragraph is never a decorator line (§4.1: no `@@`, no head).
        assert!(parse_decorator_line("[intro](url)", &specs).is_none());
        assert!(parse_decorator_line("[]{.box}", &specs).is_none());
        // Trailing text after a space is stray prose, not part of the head: the
        // head still decorates and the text is dropped with a warning.
        let line = parse_decorator_line("@@aside[intro] tail", &specs).expect("a decorator line");
        assert_eq!(line.trailing, "tail");
        // A malformed group keeps the whole line literal (§4.3).
        assert!(parse_decorator_line("@@aside[intro{.box}", &specs).is_none());
    }

    /// End to end: a group-carrying decorator binds its attributes to the
    /// decorated block.
    #[test]
    fn a_group_decorator_binds_its_attributes() {
        let bindings = bind("@@aside[intro](\"T\"){.u}\n\n- one\n");
        assert_eq!(bindings.bound.len(), 1, "{:?}", bindings.bound);
        let bound = &bindings.bound[0];
        assert_eq!(bound.type_marker.as_deref(), Some("aside"));
        assert!(bound.attrs.has("specs.class"));
        assert!(bound.attrs.has("bracket.bare"));
        assert!(bound.attrs.has("paren.bare"));
        assert!(!texts(&bindings.events).contains("@@aside"));
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

    /// §9.1/D-10 rule 2: a touching decorator on a block that belongs to another
    /// enabled receiver is left in the stream for that receiver's pass.
    #[test]
    fn a_touching_decorator_on_a_foreign_block_is_kept() {
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
