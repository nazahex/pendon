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
    bind_decorators, decorator_diagnostics, issue_event, scan_extras_chars, to_attributes, Attrs,
    BoundDecorator, Claim, ExtrasAttr, Receiver, Specs, SpecsKey,
};
use pendon_renderer_solid::{layer_templates, ComponentSet, ImportEntry, SolidRenderHints};
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
    /// §11: the `section` layer, resolved for the whole document by the
    /// renderer against the decorators' `@@type{…}` markers (ADR-0008 `D-3a`).
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
    Some(SolidRenderHints {
        templates: layer_templates(SECTION_LAYER, &options.section, |custom| {
            (custom.template.clone(), custom.imports.clone())
        }),
        ..SolidRenderHints::default()
    })
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
    let specs = |target: Option<&str>| options.section.specs_for(target);
    // D-10 rule 2: a heading is the one receiver this pass owns, so a
    // decorator above a list, quote, table or plain paragraph is left in the
    // stream for the plugin that owns it.
    let bindings = bind_decorators(events, &specs, Claim::only(Receiver::Section));
    let decorators: HashMap<usize, &BoundDecorator> = bindings
        .bound
        .iter()
        .map(|bound| (bound.target_index, bound))
        .collect();

    let mut out: Vec<Event> = Vec::with_capacity(bindings.events.len() + 16);
    // §9.1: a decorator that bound to nothing is dropped with a warning.
    out.extend(decorator_diagnostics(&bindings, "section"));

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
    let chain = heading_chain(&heading_text(events, heading, end), level, options);
    if chain.extras_id.is_some() {
        out.push(diagnostic(
            "the heading's extras `#id` is ignored; the id transfers to the section",
        ));
    }
    close_sections(stack, out, level);
    open_section(stack, out, used_ids, level, decorator, &chain);
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
///
/// It is a section like any other — same layer tag, same `depth` field — so an
/// extractor sees the whole outline instead of only its headed part.
fn ensure_preface(stack: &mut Vec<Frame>, out: &mut Vec<Event>) {
    if stack.is_empty() {
        push_section_open(out, 0);
        stack.push(Frame {
            level: 0,
            node: NodeKind::Section,
        });
    }
}

/// Opens one `<section>` node: its §11 layer tag (§11 rule 3) and its `depth`
/// field (§9.5 — the outline level it stands for, `0` for the preface).
fn push_section_open(out: &mut Vec<Event>, depth: usize) {
    out.push(Event::StartNode(NodeKind::Section));
    out.push(Event::Attribute {
        name: "__layer".to_string(),
        value: SECTION_LAYER.to_string(),
    });
    out.push(Event::Attribute {
        name: "depth".to_string(),
        value: depth.to_string(),
    });
}

/// Applies a level marker to the outline.
fn apply_marker(stack: &mut Vec<Frame>, out: &mut Vec<Event>, marker: Marker) {
    match marker {
        Marker::Deepen => {
            let current = stack.last().map(|frame| frame.level).unwrap_or(0);
            if current >= MAX_LEVEL {
                return;
            }
            push_section_open(out, current + 1);
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
) {
    // §9.5 / D-3a: the section keeps its own kind and names its layer; the entry
    // that answers the decorator's `type` is chosen by the renderer, once per
    // document, so one plugin run can still serve differently configured docs.
    let node = NodeKind::Section;

    out.push(Event::StartNode(node.clone()));
    out.push(Event::Attribute {
        name: "__layer".to_string(),
        value: SECTION_LAYER.to_string(),
    });
    // §9.5's preset assignment: `depth` is the outline level the section stands
    // for — the depth of the heading it wraps — so a section and its heading
    // agree, and a `<--->` child is one deeper than its parent.
    out.push(Event::Attribute {
        name: "depth".to_string(),
        value: level.to_string(),
    });
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
        for issue in &decorator.attrs.issues {
            out.push(issue_event("section", issue));
        }
        emit_decorator_attrs(&decorator.attrs, &decorator.specs, out);
    }
    stack.push(Frame { level, node });
}

/// §9.5: the section id — `#sectionID` > decorator slug > `[slug]` > heading
/// extras slug > the slug of the heading title.
///
/// The decorator's `#id` and slug contribute **only at their default paths**
/// (§6.3): a `field.specs.*` assignment makes them fields of the author's own,
/// published by [`emit_decorator_attrs`] rather than consumed here.
fn section_id(decorator: Option<&BoundDecorator>, chain: &HeadInfo) -> String {
    let deco_id = decorator.and_then(|d| {
        d.specs
            .element_path(SpecsKey::Id)
            .and_then(|path| d.attrs.value(path))
            .map(|value| value.literal())
    });
    let deco_slug = decorator.and_then(|d| {
        d.specs
            .element_path(SpecsKey::Tick)
            .and_then(|path| d.attrs.value(path))
            .map(|value| value.literal())
    });
    deco_id
        .or(deco_slug)
        .or_else(|| chain.slug_head.clone())
        .or_else(|| chain.extras_slug.clone())
        .unwrap_or_else(|| slugify(&chain.title))
}

/// Pushes a decorator's attributes, minus `#id` and the extras slug (both were
/// consumed by the id chain), in canonical order (§6.4) with its `specs.*` paths
/// projected (§6.3).
fn emit_decorator_attrs(attrs: &Attrs, specs: &Specs, out: &mut Vec<Event>) {
    let tick = specs.element_path(SpecsKey::Tick);
    for (name, value) in specs.dom_fields(&attrs.items, &[]) {
        if name == "id" || tick == Some(name) {
            continue;
        }
        match value {
            ExtrasAttr::Flag => out.push(Event::AttributeFlag {
                name: name.to_string(),
            }),
            ExtrasAttr::Value(value) => out.push(Event::Attribute {
                name: name.to_string(),
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
fn heading_chain(raw: &str, level: usize, options: &SectionOptions) -> HeadInfo {
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
        let specs = options.section.specs_for(head.type_marker.as_deref());
        let parsed = to_attributes(&head, &specs);
        // §6.2/§6.3: both feed the id chain at their **default** paths only.
        extras_id = specs
            .element_path(SpecsKey::Id)
            .and_then(|path| parsed.value(path))
            .map(|value| value.literal());
        extras_slug = specs
            .element_path(SpecsKey::Tick)
            .and_then(|path| parsed.value(path))
            .map(|value| value.literal());
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

/// The `depth` attribute of the heading that starts at `start` (§7.4). Defaults
/// to 1 when the attribute is missing.
fn heading_level(events: &[Event], start: usize) -> usize {
    for event in events.iter().skip(start + 1) {
        match event {
            Event::Attribute { name, value } if name == "depth" => {
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

    /// §9.5's preset assignment: every section publishes `depth` — the outline
    /// level it stands for — and the heading's own depth is what a headed section
    /// carries, so section and heading agree.
    #[test]
    fn every_section_publishes_its_depth() {
        let out = run("Preface text\n\n## A\n\nbody\n\n### B\n\n<--->\n\ndeep\n");
        let depths: Vec<String> = out
            .iter()
            .enumerate()
            .filter_map(|(index, event)| match event {
                Event::StartNode(NodeKind::Section) => Some(index),
                _ => None,
            })
            .map(|index| {
                out[index + 1..]
                    .iter()
                    .find_map(|event| match event {
                        Event::Attribute { name, value } if name == "depth" => Some(value.clone()),
                        Event::Attribute { .. } => None,
                        _ => None,
                    })
                    .expect("every section carries depth")
            })
            .collect();
        // prehead 0, `## A` 2, `### B` 3, `<--->` under B 4.
        assert_eq!(depths, ["0", "2", "3", "4"], "{out:?}");
        // Every one of them names the layer that owns it (§11 rule 3), the
        // preface included.
        let layers = out
            .iter()
            .filter(|event| {
                matches!(event, Event::Attribute { name, value }
                if name == "__layer" && value == SECTION_LAYER)
            })
            .count();
        assert_eq!(layers, 4, "{out:?}");
    }

    /// §9.5 / D-3a: the section keeps its kind and its layer tag plus the
    /// decorator's marker and class; the renderer picks the component.
    #[test]
    fn a_typed_section_is_tagged_with_its_layer() {
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
            .any(|e| matches!(e, Event::StartNode(NodeKind::Section))));
        assert_eq!(attr(&out, "__layer").as_deref(), Some("section"));
        assert_eq!(attr(&out, "type").as_deref(), Some("secA"));
        assert_eq!(attr(&out, "class").as_deref(), Some("lead"));
        // The layer's one entry is registered with its own selectors.
        let hints = solid_hints(&options).expect("hints");
        assert_eq!(hints.templates.len(), 1);
        assert_eq!(hints.templates[0].layer.as_deref(), Some("section"));
        assert_eq!(hints.templates[0].selectors.len(), 1);
        assert!(solid_hints(&SectionOptions::default()).is_none());
    }
}
