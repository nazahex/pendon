//! §9.2: the blockquote layer — `> content`, configured by a decorator line.
//!
//! `>` stays plain Markdown; this plugin binds the *extras* a blockquote may
//! carry. A quote has **no head of its own**: the only way to configure one is
//! the decorator line above it (§9.1, bound by [`pendon_extra::bind_decorators`]).
//! Anything written inside the `>` — `> @@type{…} content` included — is content,
//! and `@@type{…}` there is literal text.
//!
//! The plugin runs **before** `plugin-markdown` (§12.1). It replaces the
//! paragraph the core parser wrapped the quote in with the quote node itself:
//!
//! * `NodeKind::Custom(name)` when `[[task.blockquote.custom]]` claims the
//!   `type` (§11 rule 3);
//! * `NodeKind::Element("blockquote")` otherwise — an unclaimed/default-less
//!   `type` renders the built-in element and keeps every extra (D8).
//!
//! `__plugin_kind = "block"` then tells `plugin-markdown` to re-lex the body as
//! block content, so headings, lists, tables, fences and nested quotes inside
//! the quote follow the normal rules.
//!
//! The layer key `[[task.blockquote.custom]]` is the plugin's primary layer.

use std::collections::HashMap;

use pendon_core::{Event, NodeKind, Severity};
use pendon_extra::{bind_decorators, emit_attrs, issue_event, BoundDecorator};
use pendon_renderer_solid::{layer_templates, ComponentSet, ImportEntry, SolidRenderHints};
use serde::{Deserialize, Serialize};

/// §11 primary layer of `plugin-blockquote`: the node a quote renders.
const BLOCKQUOTE_LAYER: &str = "blockquote";
/// The built-in fallback element of an unclaimed `type` (§9.2, D8).
const FALLBACK_ELEMENT: &str = "blockquote";

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BlockquoteCustomNode {
    pub name: String,
    pub template: String,
    #[serde(default)]
    pub imports: Vec<ImportEntry>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BlockquoteOptions {
    /// §11 component set of the `blockquote` layer: typed entries plus at most
    /// one default, selected by the head's `type` marker (§11 rule 3).
    pub custom: ComponentSet<BlockquoteCustomNode>,
}

/// §9.2: preprocess every blockquote so its extras are attributes of the quote
/// node.
pub fn process(events: &[Event], options: &BlockquoteOptions) -> Vec<Event> {
    // §9.1: a decorator line directly above the quote decorates it too.
    let specs = |target: Option<&str>| options.custom.specs_for(target);
    let bindings = bind_decorators(events, &specs, quote_target);
    let decorators: HashMap<usize, &BoundDecorator> = bindings
        .bound
        .iter()
        .map(|bound| (bound.target_index, bound))
        .collect();

    let mut out: Vec<Event> = Vec::with_capacity(bindings.events.len() + 8);
    // §9.1: a decorator that bound to nothing is dropped with a warning.
    for dropped in &bindings.dropped {
        out.push(diagnostic(&format!(
            "a decorator line bound to no block and was dropped ({:?})",
            dropped.reason
        )));
    }

    let mut i = 0;
    while i < bindings.events.len() {
        let is_quote = matches!(bindings.events[i], Event::StartNode(NodeKind::Paragraph))
            && quote_target(&bindings.events, i).is_some();
        if !is_quote {
            out.push(bindings.events[i].clone());
            i += 1;
            continue;
        }
        let end = paragraph_end(&bindings.events, i);
        emit_quote(
            &bindings.events[(i + 1)..end],
            decorators.get(&i).copied(),
            &mut out,
        );
        i = end + 1;
    }
    out
}

/// §11 rule 3: every entry of the layer answers the `type` markers it declares.
pub fn solid_hints(options: &BlockquoteOptions) -> Option<SolidRenderHints> {
    if options.custom.is_empty() {
        return None;
    }
    Some(SolidRenderHints {
        templates: layer_templates(BLOCKQUOTE_LAYER, &options.custom, |custom| {
            (custom.template.clone(), custom.imports.clone())
        }),
        ..SolidRenderHints::default()
    })
}

/// The layer key `[[task.blockquote.custom]]` addresses (always the primary).
pub fn primary_layer() -> &'static str {
    BLOCKQUOTE_LAYER
}

/// The indentation of the first line of the paragraph at `index` when that
/// paragraph opens a quote (`>`), `None` otherwise.
///
/// This is the `target` rule [`bind_decorators`] is given: §9.1 binds a
/// decorator line to the next block node, and for this plugin that block is a
/// paragraph that opens a quote.
pub(crate) fn quote_target(events: &[Event], index: usize) -> Option<usize> {
    if !matches!(
        events.get(index),
        Some(Event::StartNode(NodeKind::Paragraph))
    ) {
        return None;
    }
    let line = first_text(events, index + 1)?;
    let trimmed = line.trim_start_matches(' ');
    if !trimmed.starts_with('>') {
        return None;
    }
    Some(line.len() - trimmed.len())
}

/// The first text chunk of a node, skipping its attributes.
fn first_text(events: &[Event], index: usize) -> Option<&str> {
    for event in events.iter().skip(index) {
        match event {
            Event::Text(text) => return Some(text),
            Event::Attribute { .. } | Event::AttributeFlag { .. } | Event::Diagnostic { .. } => {}
            _ => return None,
        }
    }
    None
}

/// The index of the `EndNode(Paragraph)` that closes the paragraph at `start`.
fn paragraph_end(events: &[Event], start: usize) -> usize {
    events
        .iter()
        .enumerate()
        .skip(start + 1)
        .find_map(|(i, event)| matches!(event, Event::EndNode(NodeKind::Paragraph)).then_some(i))
        .unwrap_or(events.len())
}

/// Removes one quote level from a line: the `>`, and the one space that
/// separates it from the content when present (§9.2). A lazy continuation line
/// (no `>`) keeps its text.
fn strip_one_level(line: &str) -> String {
    let spaces = line.len() - line.trim_start_matches(' ').len();
    let rest = &line[spaces..];
    match rest.strip_prefix('>') {
        Some(after) => {
            let after = after.strip_prefix(' ').unwrap_or(after);
            let mut out = String::with_capacity(line.len());
            out.push_str(&line[..spaces]);
            out.push_str(after);
            out
        }
        None => line.to_string(),
    }
}

/// A §13 `Warning` carrying the plugin's name.
fn diagnostic(message: &str) -> Event {
    Event::Diagnostic {
        severity: Severity::Warning,
        message: format!("[blockquote] {message}"),
        span: None,
    }
}

/// §9.2: turns one quote paragraph into the quote node.
///
/// The only head a blockquote reads is the decorator line above it (§9.1). The
/// content inside `>` is **content**, not a head: a `@@type{…}` written there is
/// literal text, and a slot written there belongs to whichever element claims
/// it.
fn emit_quote(paragraph: &[Event], decorator: Option<&BoundDecorator>, out: &mut Vec<Event>) {
    let mut body: Vec<Event> = Vec::with_capacity(paragraph.len());
    for event in paragraph {
        match event {
            // One quote level is stripped; lazy continuation lines keep their
            // text (§9.2).
            Event::Text(text) => body.push(Event::Text(strip_one_level(text))),
            _ => body.push(event.clone()),
        }
    }

    // §9.2: a quote has no construct head, no slot contract and no built-in
    // fields — the decorator is the whole of its extras.
    let attrs = decorator.map(|d| d.attrs.clone()).unwrap_or_default();
    let marker = decorator.and_then(|d| d.type_marker.clone());

    // §11 rule 3 / D-3a: the plugin does not pick the component; it tags the
    // node with its layer and the renderer decides once per document.
    let node = NodeKind::Element(FALLBACK_ELEMENT.to_string());

    out.push(Event::StartNode(node.clone()));
    // First attribute, so `plugin-markdown`'s placement look-ahead sees it.
    out.push(Event::Attribute {
        name: "__plugin_kind".to_string(),
        value: "block".to_string(),
    });
    out.push(Event::Attribute {
        name: "__layer".to_string(),
        value: BLOCKQUOTE_LAYER.to_string(),
    });
    out.push(Event::Attribute {
        name: "name".to_string(),
        value: FALLBACK_ELEMENT.to_string(),
    });
    if let Some(marker) = &marker {
        out.push(Event::Attribute {
            name: "type".to_string(),
            value: marker.clone(),
        });
    }
    for issue in &attrs.issues {
        out.push(issue_event("blockquote", issue));
    }
    emit_attrs(&attrs, out);
    out.extend(body);
    out.push(Event::EndNode(node));
}

#[cfg(test)]
mod tests {
    use super::*;
    use pendon_core::{parse, Options};
    use pendon_renderer_solid::TypedComponent;

    pub(crate) fn component(name: &str) -> BlockquoteCustomNode {
        BlockquoteCustomNode {
            name: name.to_string(),
            template: format!("<{name}>{{children}}</{name}>"),
            imports: Vec::new(),
        }
    }

    pub(crate) fn options() -> BlockquoteOptions {
        BlockquoteOptions {
            custom: ComponentSet::from_entries([
                TypedComponent::typed(vec!["bqA"], component("QuoteA")),
                TypedComponent::default_component(component("QuoteDefault")),
            ]),
        }
    }

    /// The same set without a layer default: an unclaimed `type` then falls back
    /// to the built-in element (§11 rule 3).
    fn typed_only() -> BlockquoteOptions {
        BlockquoteOptions {
            custom: ComponentSet::from_entries([TypedComponent::typed(
                vec!["bqA"],
                component("QuoteA"),
            )]),
        }
    }

    fn run(src: &str) -> Vec<Event> {
        run_with(src, &options())
    }

    fn run_with(src: &str, options: &BlockquoteOptions) -> Vec<Event> {
        let events = parse(src, &Options::default());
        process(&events, options)
    }

    fn custom(events: &[Event]) -> Option<String> {
        events.iter().find_map(|event| match event {
            Event::StartNode(NodeKind::Custom(name)) => Some(name.clone()),
            _ => None,
        })
    }

    fn element(events: &[Event]) -> Option<String> {
        events.iter().find_map(|event| match event {
            Event::StartNode(NodeKind::Element(name)) => Some(name.clone()),
            _ => None,
        })
    }

    fn attr(events: &[Event], key: &str) -> Option<String> {
        events.iter().find_map(|event| match event {
            Event::Attribute { name, value } if name == key => Some(value.clone()),
            _ => None,
        })
    }

    fn text(events: &[Event]) -> String {
        events
            .iter()
            .filter_map(|event| match event {
                Event::Text(chunk) => Some(chunk.as_str()),
                _ => None,
            })
            .collect()
    }

    /// §9.2: an inner `@@type{…}` is **content**, not a head — the quote keeps it
    /// literally, reads no `type` and no extras from it.
    #[test]
    fn an_inner_head_is_literal_text() {
        let out = run("> @@bqA{.x} body\n");
        assert_eq!(element(&out).as_deref(), Some("blockquote"), "{out:?}");
        assert_eq!(attr(&out, "__layer").as_deref(), Some("blockquote"));
        assert_eq!(attr(&out, "type"), None, "{out:?}");
        assert_eq!(attr(&out, "specs.class"), None, "{out:?}");
        assert!(text(&out).contains("@@bqA{.x} body"), "{:?}", text(&out));
        // The `>` marker is still gone and the paragraph wrapper with it.
        assert!(!text(&out).contains('>'), "{:?}", text(&out));
        assert!(
            !out.iter()
                .any(|event| matches!(event, Event::StartNode(NodeKind::Paragraph))),
            "{out:?}"
        );
    }

    /// §9.2: the body is content, so a slot inside it belongs to the element that
    /// claims it — the quote itself reads none of it.
    #[test]
    fn the_body_keeps_its_own_slots() {
        let out = run("> @[the docs](/docs) and [plain](/x)\n");
        assert_eq!(attr(&out, "type"), None, "{out:?}");
        assert_eq!(attr(&out, "href"), None, "{out:?}");
        assert!(
            text(&out).contains("@[the docs](/docs) and [plain](/x)"),
            "{:?}",
            text(&out)
        );
    }

    /// §9.1/§9.2: a decorator line directly above the quote decorates it too.
    #[test]
    fn a_decorator_line_above_the_quote_decorates_it() {
        let out = run("@@{.u}\n\n> quoted\n");
        // D-3a: the quote stays one layer-tagged node; the default entry is the
        // renderer's to apply (it answers every instance of the layer).
        assert_eq!(element(&out).as_deref(), Some("blockquote"), "{out:?}");
        assert_eq!(attr(&out, "__layer").as_deref(), Some("blockquote"));
        assert_eq!(attr(&out, "specs.class").as_deref(), Some("u"));
        assert!(!text(&out).contains("@@"));
        assert!(text(&out).contains("quoted"));
    }

    /// §11 rule 3 + D8: an unclaimed decorator `type` renders the built-in
    /// element and keeps every extra.
    #[test]
    fn an_unclaimed_type_falls_back_to_the_element() {
        let out = run_with("@@bqZ{.z}\n\n> body\n", &typed_only());
        assert!(custom(&out).is_none(), "{out:?}");
        assert_eq!(element(&out).as_deref(), Some("blockquote"));
        assert_eq!(attr(&out, "type").as_deref(), Some("bqZ"));
        assert_eq!(attr(&out, "specs.class").as_deref(), Some("z"));
    }

    /// A plain quote keeps its text and gains no `type`.
    #[test]
    fn a_plain_quote_has_no_type() {
        let out = run_with("> plain *quote*\n", &typed_only());
        assert_eq!(element(&out).as_deref(), Some("blockquote"));
        assert_eq!(attr(&out, "type"), None);
        assert!(text(&out).contains("plain *quote*"));
    }

    /// §9.2: a nested quote keeps one level for the Markdown pass to re-lex.
    #[test]
    fn a_nested_quote_keeps_one_level() {
        let out = run(">> inner\n");
        assert!(text(&out).contains("> inner"), "{:?}", text(&out));
    }

    #[test]
    fn primary_layer_matches_the_config_key() {
        assert_eq!(primary_layer(), BLOCKQUOTE_LAYER);
        assert!(solid_hints(&BlockquoteOptions::default()).is_none());
        assert_eq!(solid_hints(&options()).expect("hints").templates.len(), 2);
    }
}
