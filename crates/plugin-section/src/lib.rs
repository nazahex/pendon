//! §9.5: `plugin-section` — the section outline.
//!
//! `plugin-section` runs **before** `plugin-markdown`. It wraps every heading
//! (together with the content under it) in a nested `Section` node — or a
//! configured §11 component — driven by heading levels, by a decorator line
//! above the heading and by the in-stream level markers `<--->` / `>---<`.
//!
//! # Why *before* Markdown
//!
//! The level markers collide with Markdown: `>---<` is a blockquote and `<--->`
//! a plain paragraph, so once Markdown has run the raw marker is gone. Running
//! before Markdown lets the plugin see the raw heading text (the core parser
//! already emits `Heading` nodes), the raw marker lines and the decorator line.
//! Markdown then parses the interior of every `Section` node this plugin emits.
//!
//! # The section head
//!
//! A decorator line directly above a heading (a blank line is allowed) decorates
//! the **section**, not the heading — the heading already owns its extras on the
//! `#` run (§7.4), so a second head above it would be redundant. The decorator is
//! bound through [`pendon_extra::bind_decorators`] like the other block binders:
//!
//! ```text
//! @@sectionA{`slug-section`, #sectionID}
//! ###[slug-head]("Heading X")@@headingX{`slug-head-extras`, #headingID} Title
//! ```
//!
//! # ID priority
//!
//! The section id is the first available of `#sectionID` (the decorator's `#id`)
//! > `` `slug-section` `` (the decorator's slug) > `` `slug-head` `` (the
//! heading's `[slug]`) > `` `slug-head-extras` `` (the heading's extras slug) >
//! the slug of the heading title. When `plugin-section` owns the outline the
//! heading never owns an `id`: `plugin-heading` yields it
//! (`HeadingOptions::section_owns_id`). A heading's own extras `#id` is **not**
//! part of the chain (the RFC lists only the two above); it is ignored with a
//! warning.
//!
//! # Level markers
//!
//! A line whose trimmed content is exactly `<--->` deepens the outline by one
//! nested section; exactly `>---<` closes the innermost section. The deepen is
//! capped at heading depth 6 and the close is a no-op once the innermost open
//! node is the preface.

use std::collections::HashMap;

use pendon_core::{ensure_unique, slugify, Event, NodeKind, Severity};
use pendon_extra::{
    bind_decorators, scan_extras_chars, to_attributes, warning_message, Attrs, BoundDecorator,
    ExtrasAttr, ExtrasOptions,
};
use pendon_renderer_solid::{ComponentSet, ComponentTemplate, ImportEntry, SolidRenderHints};
use serde::{Deserialize, Serialize};

/// §9.5: the single layer of `plugin-section`, the `<section>` element it emits.
const SECTION_LAYER: &str = "section";

/// §9.5: the deepest section level `<--->` may reach (matching heading depth 6).
const MAX_LEVEL: usize = 6;

/// §11 component of the `section` layer. A typed decorator head routes to the
/// matching entry; an unclaimed type falls back to the layer default, else to
/// the built-in `Section` node.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SectionCustomNode {
    pub name: String,
    pub template: String,
    #[serde(default)]
    pub imports: Vec<ImportEntry>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SectionOptions {
    /// §11: the `section` layer, selected per instance by the decorator's
    /// `@@type{…}` marker.
    pub section: ComponentSet<SectionCustomNode>,
}

/// §9.5/§11: the layers this plugin owns, in declaration order.
pub fn layers() -> [&'static str; 1] {
    [SECTION_LAYER]
}

/// The layer key `[task.section.custom.section]` addresses (the primary layer).
pub fn primary_layer() -> &'static str {
    SECTION_LAYER
}

/// §11 rule 3: one template per entry of the `section` layer.
pub fn solid_hints(options: &SectionOptions) -> Option<SolidRenderHints> {
    if options.section.is_empty() {
        return None;
    }
    let mut hints = SolidRenderHints::default();
    for custom in options.section.components() {
        hints.templates.push(ComponentTemplate {
            node_type: custom.name.clone(),
            node_name: Some(custom.name.clone()),
            template: custom.template.clone(),
        });
        if !custom.imports.is_empty() {
            hints.template_imports.insert(
                (custom.name.clone(), Some(custom.name.clone())),
                custom.imports.clone(),
            );
        }
    }
    Some(hints)
}

// --- Main processor ---

/// One open section, so the matching `EndNode` can be emitted at the right place.
struct Frame {
    level: usize,
    node: NodeKind,
}

/// A level marker read off its own paragraph line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Marker {
    /// `<--->`: deepen the outline by one nested section.
    Deepen,
    /// `>---<`: close the innermost section.
    Shallow,
}

pub fn process(events: &[Event], options: &SectionOptions) -> Vec<Event> {
    let extras = ExtrasOptions::default();
    let bindings = bind_decorators(events, &extras, section_target);
    let decorators: HashMap<usize, &BoundDecorator> = bindings
        .bound
        .iter()
        .map(|bound| (bound.target_index, bound))
        .collect();

    let mut out: Vec<Event> = Vec::with_capacity(bindings.events.len() + 16);
    // §9.1: a decorator that bound to nothing is dropped with a warning.
    for dropped in &bindings.dropped {
        out.push(diagnostic(&format!(
            "a decorator line bound to nothing and was dropped ({:?})",
            dropped.reason
        )));
    }

    let events = &bindings.events;
    let mut stack: Vec<Frame> = Vec::new();
    let mut used_ids: HashMap<String, usize> = HashMap::new();

    let mut i = 0;
    while i < events.len() {
        match &events[i] {
            Event::StartNode(NodeKind::Heading) => {
                let end = node_end(events, i, NodeKind::Heading);
                open_heading(
                    &mut stack,
                    &mut out,
                    &mut used_ids,
                    events,
                    i,
                    decorators.get(&i).copied(),
                    options,
                );
                out.extend(events[i..=end].iter().cloned());
                i = end + 1;
            }
            Event::StartNode(NodeKind::Paragraph) => {
                let end = node_end(events, i, NodeKind::Paragraph);
                if let Some(heading) = heading_within(events, i, end) {
                    // The core parser glues a decorator line and its heading into
                    // one paragraph. The section opens at the heading; the outer
                    // paragraph is dropped (its only content is the heading).
                    let heading_end = node_end(events, heading, NodeKind::Heading);
                    open_heading(
                        &mut stack,
                        &mut out,
                        &mut used_ids,
                        events,
                        heading,
                        decorators.get(&i).copied(),
                        options,
                    );
                    out.extend(events[heading..=heading_end].iter().cloned());
                } else if let Some(markers) = marker_paragraph(events, i, end) {
                    for marker in markers {
                        apply_marker(&mut stack, &mut out, marker);
                    }
                } else {
                    ensure_preface(&mut stack, &mut out);
                    out.extend(events[i..=end].iter().cloned());
                }
                i = end + 1;
            }
            Event::StartNode(NodeKind::Document) => {
                out.push(events[i].clone());
                i += 1;
            }
            Event::EndNode(NodeKind::Document) => {
                close_all(&mut stack, &mut out);
                out.push(events[i].clone());
                i += 1;
            }
            Event::StartNode(NodeKind::Frontmatter) => {
                let end = node_end(events, i, NodeKind::Frontmatter);
                out.extend(events[i..=end].iter().cloned());
                i = end + 1;
            }
            Event::StartNode(_) => {
                ensure_preface(&mut stack, &mut out);
                out.push(events[i].clone());
                i += 1;
            }
            ev => {
                out.push(ev.clone());
                i += 1;
            }
        }
    }
    close_all(&mut stack, &mut out);
    out
}

/// §9.5: binds a decorator line to the heading below it. Every other block is
/// declined so the line stays for `plugin-list` / `plugin-blockquote` /
/// `plugin-markdown` (§9.1 — a binder must never steal another's decorator).
fn section_target(events: &[Event], index: usize) -> Option<usize> {
    match &events[index] {
        Event::StartNode(NodeKind::Heading) => Some(0),
        // A decorator line that *touches* its heading shares one paragraph with
        // it, so the heading is the paragraph's first content.
        Event::StartNode(NodeKind::Paragraph) => {
            let end = node_end(events, index, NodeKind::Paragraph);
            heading_within(events, index, end).map(|_| 0)
        }
        _ => None,
    }
}

/// The index of a `StartNode(Heading)` that is the first content of the
/// paragraph that spans `start..end`. `None` for an ordinary paragraph.
fn heading_within(events: &[Event], start: usize, end: usize) -> Option<usize> {
    for index in (start + 1)..end {
        match &events[index] {
            Event::Text(text) if text.trim().is_empty() => {}
            Event::Attribute { .. } | Event::AttributeFlag { .. } | Event::Diagnostic { .. } => {}
            Event::StartNode(NodeKind::Heading) => return Some(index),
            _ => return None,
        }
    }
    None
}

/// Opens the section of the heading that starts at `heading`, reading its level
/// and id chain and applying the bound decorator.
fn open_heading(
    stack: &mut Vec<Frame>,
    out: &mut Vec<Event>,
    used_ids: &mut HashMap<String, usize>,
    events: &[Event],
    heading: usize,
    decorator: Option<&BoundDecorator>,
    options: &SectionOptions,
) {
    let level = heading_level(events, heading);
    let end = node_end(events, heading, NodeKind::Heading);
    let chain = heading_chain(&heading_text(events, heading, end), level);
    if chain.extras_id.is_some() {
        out.push(diagnostic(
            "the heading's extras `#id` is ignored; the id transfers to the section",
        ));
    }
    close_sections(stack, out, level);
    open_section(stack, out, used_ids, level, decorator, &chain, options);
}

/// Closes every open section that cannot contain a heading of `level`: the
/// preface (level 0) and every section at `level` or deeper.
fn close_sections(stack: &mut Vec<Frame>, out: &mut Vec<Event>, level: usize) {
    while let Some(frame) = stack.last() {
        if frame.level == 0 || frame.level >= level {
            out.push(Event::EndNode(frame.node.clone()));
            stack.pop();
        } else {
            break;
        }
    }
}

/// Closes every open section, deepest first.
fn close_all(stack: &mut Vec<Frame>, out: &mut Vec<Event>) {
    while let Some(frame) = stack.pop() {
        out.push(Event::EndNode(frame.node));
    }
}

/// §9.5: opens a preface `Section` (level 0, no id) for content that precedes
/// the first heading.
fn ensure_preface(stack: &mut Vec<Frame>, out: &mut Vec<Event>) {
    if stack.is_empty() {
        out.push(Event::StartNode(NodeKind::Section));
        stack.push(Frame {
            level: 0,
            node: NodeKind::Section,
        });
    }
}

/// Applies a level marker to the outline.
fn apply_marker(stack: &mut Vec<Frame>, out: &mut Vec<Event>, marker: Marker) {
    match marker {
        Marker::Deepen => {
            let current = stack.last().map(|frame| frame.level).unwrap_or(0);
            if current >= MAX_LEVEL {
                return;
            }
            out.push(Event::StartNode(NodeKind::Section));
            stack.push(Frame {
                level: current + 1,
                node: NodeKind::Section,
            });
        }
        Marker::Shallow => match stack.last() {
            // The preface is the shallowest node; there is nothing to close.
            Some(frame) if frame.level == 0 => {}
            Some(_) => {
                let frame = stack.pop().expect("frame checked above");
                out.push(Event::EndNode(frame.node));
            }
            None => {}
        },
    }
}

/// §9.5/§11: opens the section for a heading — the configured component of the
/// decorator's type, else the built-in `Section` node — carrying the id from the
/// priority chain and the decorator's remaining attributes.
fn open_section(
    stack: &mut Vec<Frame>,
    out: &mut Vec<Event>,
    used_ids: &mut HashMap<String, usize>,
    level: usize,
    decorator: Option<&BoundDecorator>,
    chain: &HeadInfo,
    options: &SectionOptions,
) {
    let custom = decorator.and_then(|d| options.section.select(d.type_marker.as_deref()));
    let node = custom
        .map(|custom| NodeKind::Custom(custom.name.clone()))
        .unwrap_or(NodeKind::Section);

    out.push(Event::StartNode(node.clone()));
    // §11 rule 3: a `Custom` node is inline by default; `block` makes
    // `plugin-markdown` re-lex the section body as blocks.
    if let NodeKind::Custom(name) = &node {
        out.push(Event::Attribute {
            name: "__plugin_kind".to_string(),
            value: "block".to_string(),
        });
        out.push(Event::Attribute {
            name: "name".to_string(),
            value: name.clone(),
        });
    }
    if let Some(marker) = decorator.and_then(|d| d.type_marker.clone()) {
        out.push(Event::Attribute {
            name: "type".to_string(),
            value: marker,
        });
    }
    let id = ensure_unique(section_id(decorator, chain), used_ids);
    out.push(Event::Attribute {
        name: "id".to_string(),
        value: id,
    });
    if let Some(decorator) = decorator {
        for warning in &decorator.attrs.warnings {
            out.push(diagnostic(&warning_message(warning)));
        }
        emit_decorator_attrs(&decorator.attrs, out);
    }
    stack.push(Frame { level, node });
}

/// §9.5: the section id — `#sectionID` > decorator slug > `[slug]` > heading
/// extras slug > the slug of the heading title.
fn section_id(decorator: Option<&BoundDecorator>, chain: &HeadInfo) -> String {
    let deco_id = decorator.and_then(|d| d.attrs.value("id").map(|value| value.literal()));
    let deco_slug = decorator.and_then(|d| d.attrs.value("slug").map(|value| value.literal()));
    deco_id
        .or(deco_slug)
        .or_else(|| chain.slug_head.clone())
        .or_else(|| chain.extras_slug.clone())
        .unwrap_or_else(|| slugify(&chain.title))
}

/// Pushes a decorator's attributes, minus `id` and `slug` (both were consumed by
/// the id chain), in canonical order (§6.4).
fn emit_decorator_attrs(attrs: &Attrs, out: &mut Vec<Event>) {
    for (name, value) in &attrs.items {
        if name == "id" || name == "slug" {
            continue;
        }
        match value {
            ExtrasAttr::Flag => out.push(Event::AttributeFlag { name: name.clone() }),
            ExtrasAttr::Value(value) => out.push(Event::Attribute {
                name: name.clone(),
                value: value.literal(),
            }),
        }
    }
}

/// A §13 `Warning` carrying the plugin's name.
fn diagnostic(message: &str) -> Event {
    Event::Diagnostic {
        severity: Severity::Warning,
        message: format!("[section] {message}"),
        span: None,
    }
}

// --- Heading head + line helpers ---

/// §9.5: the id-relevant parts of a heading's own head (§7.4).
struct HeadInfo {
    /// `[slug]` — the `` `slug-head` `` of the priority chain.
    slug_head: Option<String>,
    /// The extras `` `slug` `` — the `` `slug-head-extras` `` of the chain.
    extras_slug: Option<String>,
    /// The extras `#id`; ignored by the section (reported).
    extras_id: Option<String>,
    /// The heading text after the head; slugified as the last fallback.
    title: String,
}

/// Parses the head of a raw core heading line (`##[slug]("t")@@type{…} Title`)
/// into the pieces the id chain needs. Mirrors `plugin-heading`'s own parser, so
/// the section and the heading agree on what the head is.
fn heading_chain(raw: &str, level: usize) -> HeadInfo {
    let chars: Vec<char> = raw.chars().collect();
    let mut cursor = 0;
    while chars.get(cursor) == Some(&'#') {
        cursor += 1;
    }
    if cursor == level && chars.get(cursor) == Some(&' ') {
        cursor += 1;
    }

    let mut slug_head = None;
    if chars.get(cursor) == Some(&'[') {
        if let Some(close) = find_char(&chars, cursor + 1, ']') {
            let content: String = chars[cursor + 1..close].iter().collect();
            let trimmed = content.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('.') && !trimmed.starts_with('#') {
                slug_head = Some(trimmed.to_string());
                cursor = close + 1;
            }
        }
    }
    if chars.get(cursor) == Some(&'(') {
        if let Some(close) = find_matching_paren(&chars, cursor) {
            cursor = close + 1;
        }
    }

    let mut extras_slug = None;
    let mut extras_id = None;
    if let Some((head, next)) = scan_extras_chars(&chars, cursor) {
        let parsed = to_attributes(&head, &ExtrasOptions::default());
        extras_id = parsed.value("id").map(|value| value.literal());
        extras_slug = parsed.value("slug").map(|value| value.literal());
        cursor = next;
    }

    let title: String = chars[cursor..].iter().collect();
    HeadInfo {
        slug_head,
        extras_slug,
        extras_id,
        title: title.trim().to_string(),
    }
}

/// The level attribute of the heading that starts at `start` (§7.4). Defaults to
/// 1 when the attribute is missing.
fn heading_level(events: &[Event], start: usize) -> usize {
    for event in events.iter().skip(start + 1) {
        match event {
            Event::Attribute { name, value } if name == "level" => {
                return value.parse().unwrap_or(1);
            }
            Event::Attribute { .. } | Event::AttributeFlag { .. } => continue,
            _ => break,
        }
    }
    1
}

/// The text of the node between `start` (its `StartNode`) and `end` (its
/// `EndNode`), concatenated.
fn heading_text(events: &[Event], start: usize, end: usize) -> String {
    let mut text = String::new();
    for event in &events[start + 1..end] {
        if let Event::Text(chunk) = event {
            text.push_str(chunk);
        }
    }
    text
}

/// §9.5: reads a paragraph that consists only of level-marker lines. Returns
/// `None` when any line is ordinary text, so a marker never eats real content.
fn marker_paragraph(events: &[Event], start: usize, end: usize) -> Option<Vec<Marker>> {
    let mut text = String::new();
    for event in &events[start + 1..end] {
        match event {
            Event::Text(chunk) => text.push_str(chunk),
            Event::Attribute { .. } | Event::AttributeFlag { .. } | Event::Diagnostic { .. } => {}
            _ => return None,
        }
    }
    let mut markers = Vec::new();
    for line in text.lines() {
        match line.trim() {
            "" => {}
            "<--->" => markers.push(Marker::Deepen),
            ">---<" => markers.push(Marker::Shallow),
            _ => return None,
        }
    }
    if markers.is_empty() {
        None
    } else {
        Some(markers)
    }
}

/// The index of the `EndNode` that closes the `start`-th `kind` node.
fn node_end(events: &[Event], start: usize, kind: NodeKind) -> usize {
    let mut depth = 0usize;
    for (index, event) in events.iter().enumerate().skip(start) {
        match event {
            Event::StartNode(k) if *k == kind => depth += 1,
            Event::EndNode(k) if *k == kind => {
                depth -= 1;
                if depth == 0 {
                    return index;
                }
            }
            _ => {}
        }
    }
    events.len().saturating_sub(1)
}

/// Finds `wanted` in `chars` at or after `index`.
fn find_char(chars: &[char], mut index: usize, wanted: char) -> Option<usize> {
    while index < chars.len() {
        if chars[index] == wanted {
            return Some(index);
        }
        index += 1;
    }
    None
}

/// Finds the `)` matching the `(` at `start`, counting nested pairs.
fn find_matching_paren(chars: &[char], start: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut index = start;
    while index < chars.len() {
        match chars[index] {
            '(' => depth += 1,
            ')' if depth == 1 => return Some(index),
            ')' => depth -= 1,
            _ => {}
        }
        index += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use pendon_core::{parse, Options};
    use pendon_renderer_solid::TypedComponent;

    fn run(src: &str) -> Vec<Event> {
        let events = parse(src, &Options::default());
        process(&events, &SectionOptions::default())
    }

    fn section_starts(events: &[Event]) -> Vec<NodeKind> {
        events
            .iter()
            .filter_map(|event| match event {
                Event::StartNode(kind) if is_section_node(kind) => Some(kind.clone()),
                _ => None,
            })
            .collect()
    }

    fn is_section_node(kind: &NodeKind) -> bool {
        matches!(kind, NodeKind::Section) || matches!(kind, NodeKind::Custom(_))
    }

    fn attr(events: &[Event], key: &str) -> Option<String> {
        events.iter().find_map(|event| match event {
            Event::Attribute { name, value } if name == key => Some(value.clone()),
            _ => None,
        })
    }

    /// §9.5: each heading opens a section at its level; a deeper heading nests.
    #[test]
    fn wraps_headings_in_nested_sections() {
        let out = run("## A\n\nbody\n\n### B\n\nmore\n");
        let starts = section_starts(&out);
        assert_eq!(starts.len(), 2, "{out:?}");
        assert_eq!(attr(&out, "id").as_deref(), Some("a"));
    }

    /// §9.5 id priority: decorator `#id` > decorator slug > `[slug]` >
    /// extras slug > title.
    #[test]
    fn id_priority_chain() {
        let out = run(
            "@@secA{`slug-section`, #sectionID}\n###[slug-head](\"T\")@@h{`slug-extras`} Title\n",
        );
        assert_eq!(attr(&out, "id").as_deref(), Some("sectionID"));
        assert_eq!(attr(&out, "type").as_deref(), Some("secA"));

        let out = run("@@secA{`slug-section`}\n###[slug-head] Title\n");
        assert_eq!(attr(&out, "id").as_deref(), Some("slug-section"));

        let out = run("@@secA\n###[slug-head] Title\n");
        assert_eq!(attr(&out, "id").as_deref(), Some("slug-head"));

        let out = run("###@@h{`slug-extras`} Title\n");
        assert_eq!(attr(&out, "id").as_deref(), Some("slug-extras"));

        let out = run("### Hello World\n");
        assert_eq!(attr(&out, "id").as_deref(), Some("hello-world"));
    }

    /// §9.5: the heading's own extras `#id` is ignored with a warning.
    #[test]
    fn heading_extras_id_is_ignored() {
        let out = run("##[slug-head]@@h{#headingID} Title\n");
        assert_eq!(attr(&out, "id").as_deref(), Some("slug-head"));
        assert!(
            out.iter()
                .any(|e| matches!(e, Event::Diagnostic { message, .. }
                if message.contains("the id transfers to the section"))),
            "{out:?}"
        );
    }

    /// §9.5: `<--->` deepens, `>---<` shallows; the preface absorbs stray
    /// shallows.
    #[test]
    fn level_markers_deepen_and_shallow() {
        let out = run("## A\n\n<--->\n\nx\n\n>---<\n\ny\n");
        assert_eq!(section_starts(&out).len(), 2, "{out:?}");

        // Four consecutive deepens on one paragraph, capped at level 6.
        let out = run("## A\n\n<--->\n<--->\n<--->\n<--->\n\nx\n");
        assert_eq!(section_starts(&out).len(), 5, "{out:?}");
    }

    /// §9.5/§11: a configured component replaces the built-in `Section` node and
    /// keeps the decorator's class.
    #[test]
    fn configured_component_replaces_the_builtin() {
        let options = SectionOptions {
            section: ComponentSet::from_entries([TypedComponent::typed(
                vec!["secA"],
                SectionCustomNode {
                    name: "SectionA".to_string(),
                    template: "<SectionA {...attrs}>{children}</SectionA>".to_string(),
                    imports: Vec::new(),
                },
            )]),
        };
        let events = parse("@@secA{.lead}\n## A\n", &Options::default());
        let out = process(&events, &options);
        assert!(out
            .iter()
            .any(|e| matches!(e, Event::StartNode(NodeKind::Custom(name)) if name == "SectionA")));
        assert_eq!(attr(&out, "__plugin_kind").as_deref(), Some("block"));
        assert_eq!(attr(&out, "class").as_deref(), Some("lead"));
        assert_eq!(solid_hints(&options).expect("hints").templates.len(), 1);
        assert!(solid_hints(&SectionOptions::default()).is_none());
    }
}
