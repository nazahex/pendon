//! §9.2: the blockquote layer — `> @@type{…} content`.
//!
//! `>` stays plain Markdown; this plugin only binds the *extras* a blockquote
//! may carry. Two spellings, the inner one winning (§9.2):
//!
//! * **inner head** — `> @@type{…} content`. The head sits immediately inside
//!   the quote, before its first content. A space between the marker, the head
//!   and the content is allowed (§4.1: blockquote is one of the two adjacency
//!   exceptions, with list).
//! * **decorator line** — `@@type{…}` on the line directly above the quote
//!   (§9.1, bound by [`pendon_extra::bind_decorators`]).
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
use pendon_extra::{
    bind_decorators, emit_attrs, parse_extras, to_attributes, warning_message, Attrs,
    BoundDecorator, ExtrasMatch, ExtrasOptions,
};
use pendon_renderer_solid::{ComponentSet, ComponentTemplate, ImportEntry, SolidRenderHints};
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
    let extras = ExtrasOptions::default();
    // §9.1: a decorator line directly above the quote decorates it too.
    let bindings = bind_decorators(events, &extras, quote_target);
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
            options,
            &extras,
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
    let mut hints = SolidRenderHints::default();
    for custom in options.custom.components() {
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
fn emit_quote(
    paragraph: &[Event],
    decorator: Option<&BoundDecorator>,
    options: &BlockquoteOptions,
    extras: &ExtrasOptions,
    out: &mut Vec<Event>,
) {
    let mut inner = Attrs::default();
    let mut type_marker: Option<String> = None;
    let mut body: Vec<Event> = Vec::with_capacity(paragraph.len());
    let mut first = true;

    for event in paragraph {
        let Event::Text(text) = event else {
            body.push(event.clone());
            continue;
        };
        if text.trim().is_empty() {
            body.push(event.clone());
            continue;
        }
        let line = strip_one_level(text);
        if first {
            first = false;
            match parse_extras(line.trim_start_matches(' ')) {
                // §9.2/§4.1: the head sits immediately inside the quote and the
                // space between the head and the content is allowed here.
                ExtrasMatch::Head { head, rest } => {
                    type_marker = head.type_marker.clone();
                    inner = to_attributes(&head, extras);
                    body.push(Event::Text(rest.trim_start_matches(' ').to_string()));
                }
                // §4.3: an absent or malformed head stays literal text.
                ExtrasMatch::Absent { .. } | ExtrasMatch::Malformed { .. } => {
                    body.push(Event::Text(line));
                }
            }
        } else {
            body.push(Event::Text(line));
        }
    }

    // §9.2: "when both exist the inner one wins", so the inner head merges
    // *over* the decorator's attributes.
    let decorator_attrs = decorator.map(|d| d.attrs.clone()).unwrap_or_default();
    let marker = type_marker.or_else(|| decorator.and_then(|d| d.type_marker.clone()));
    let attrs = inner.merge_with(decorator_attrs, &[]);

    let custom = options.custom.select(marker.as_deref());
    let node = match custom {
        Some(custom) => NodeKind::Custom(custom.name.clone()),
        None => NodeKind::Element(FALLBACK_ELEMENT.to_string()),
    };

    out.push(Event::StartNode(node.clone()));
    // First attribute, so `plugin-markdown`'s placement look-ahead sees it.
    out.push(Event::Attribute {
        name: "__plugin_kind".to_string(),
        value: "block".to_string(),
    });
    out.push(Event::Attribute {
        name: "name".to_string(),
        value: match custom {
            Some(custom) => custom.name.clone(),
            None => FALLBACK_ELEMENT.to_string(),
        },
    });
    if let Some(marker) = &marker {
        out.push(Event::Attribute {
            name: "type".to_string(),
            value: marker.clone(),
        });
    }
    for extras_warning in &attrs.warnings {
        out.push(diagnostic(&warning_message(extras_warning)));
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

    /// §9.2: the head immediately inside the quote decorates the quote itself.
    #[test]
    fn the_inner_head_decorates_the_quote() {
        let out = run("> @@bqA{.x} body\n");
        assert_eq!(custom(&out).as_deref(), Some("QuoteA"), "{out:?}");
        assert_eq!(attr(&out, "type").as_deref(), Some("bqA"));
        assert_eq!(attr(&out, "class").as_deref(), Some("x"));
        assert!(text(&out).contains("body"));
        // The `>` marker is gone and the paragraph wrapper with it.
        assert!(!text(&out).contains('>'), "{:?}", text(&out));
        assert!(
            !out.iter()
                .any(|event| matches!(event, Event::StartNode(NodeKind::Paragraph))),
            "{out:?}"
        );
    }

    /// §9.2: the marker and the head may be separated by a space.
    #[test]
    fn a_space_after_the_marker_is_allowed() {
        let out = run(">@@bqA{.x} body\n");
        assert_eq!(custom(&out).as_deref(), Some("QuoteA"), "{out:?}");
    }

    /// §9.1/§9.2: a decorator line directly above the quote decorates it too.
    #[test]
    fn a_decorator_line_above_the_quote_decorates_it() {
        let out = run("@@{.u}\n\n> quoted\n");
        assert_eq!(custom(&out).as_deref(), Some("QuoteDefault"), "{out:?}");
        assert_eq!(attr(&out, "class").as_deref(), Some("u"));
        assert!(!text(&out).contains("@@"));
        assert!(text(&out).contains("quoted"));
    }

    /// §9.2: "when both exist the inner one wins".
    #[test]
    fn the_inner_head_wins_over_the_decorator() {
        let out = run("@@bqB{.outer}\n\n> @@bqA{.inner} body\n");
        assert_eq!(custom(&out).as_deref(), Some("QuoteA"), "{out:?}");
        assert_eq!(attr(&out, "type").as_deref(), Some("bqA"));
        assert_eq!(attr(&out, "class").as_deref(), Some("inner"));
    }

    /// §11 rule 3 + D8: an unclaimed type renders the built-in element.
    #[test]
    fn an_unclaimed_type_falls_back_to_the_element() {
        let out = run_with("> @@bqZ{.z} body\n", &typed_only());
        assert!(custom(&out).is_none(), "{out:?}");
        assert_eq!(element(&out).as_deref(), Some("blockquote"));
        assert_eq!(attr(&out, "type").as_deref(), Some("bqZ"));
        assert_eq!(attr(&out, "class").as_deref(), Some("z"));
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
