//! §9.3/§9.4: the list layers.
//!
//! Three layers, mirroring `[[task.list.<layer>.custom]]` (§9.4):
//!
//! | Layer       | Element          | Matched by (§9.3)                                |
//! | ----------- | ---------------- | ------------------------------------------------ |
//! | `unordered` | `<ul>` container | a container decorator `type`, marker `-`/`*`/`+` |
//! | `ordered`   | `<ol>` container | a container decorator `type`, marker `1.`        |
//! | `list`      | `<li>` item      | an item decorator `type` (L2/L3)                 |
//!
//! The plugin runs **before** `plugin-markdown` (§12.1) and owns the *extras*
//! binding, never the Markdown itself.
//!
//! # The container merge (§9.4)
//!
//! The wrapper it emits carries `__plugin_kind = "unordered" | "ordered"`, which
//! `plugin-markdown` reads as a *container* placement: the body is re-lexed as
//! blocks and the list it decorates is built **inside** the wrapper instead of in
//! a `<ul>` / `<ol>` of its own. That is what makes the extras land on the
//! container (or on the component replacing it) rather than around it, keeps the
//! `start` offset of an ordered list, and still lets a nested list open its own
//! element. A wrapper whose body turns out not to be a list stays an ordinary
//! block node, so a decorator is never silently dropped.
//!
//! # Scope
//!
//! Implemented: **L1, the container decorator** — a decorator line directly above
//! a list (`@@type{…}` / `@@{…}`, §9.1) decorates the container. The node is
//! `NodeKind::Custom(name)` when the `type` is claimed (§11 rule 3),
//! `NodeKind::Element("ul"|"ol")` otherwise (D8); the layer `unordered` or
//! `ordered` follows the **marker** (§9.4), never the decorator's own type.
//!
//! Not implemented yet, and therefore documented rather than half-done:
//!
//! * **L2/L3, the item layer** (`list` / `<li>`): an item-decorator at the start
//!   of an item's content, and a decorator line as an item's second line.
//! * **L4** the `start` offset rule (the Markdown pass already reads it from the
//!   first item's number and the merge keeps it) and **L5**: nesting is parsed by
//!   `plugin-markdown`, and a nested list's own decorator would have to be
//!   indented to the nested list's level.

use std::collections::HashMap;

use pendon_core::{Event, NodeKind, Severity};
use pendon_extra::{bind_decorators, emit_attrs, warning_message, BoundDecorator, ExtrasOptions};
use pendon_renderer_solid::{ComponentSet, ComponentTemplate, ImportEntry, SolidRenderHints};
use serde::{Deserialize, Serialize};

/// §9.4: the layer named after the plugin owns the **item** element.
const LIST_LAYER: &str = "list";
/// §9.4: the layer owning the `<ul>` container.
const UNORDERED_LAYER: &str = "unordered";
/// §9.4: the layer owning the `<ol>` container.
const ORDERED_LAYER: &str = "ordered";

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ListCustomNode {
    pub name: String,
    pub template: String,
    #[serde(default)]
    pub imports: Vec<ImportEntry>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ListOptions {
    /// §9.4: `<li>` items, routed by an item-decorator `type` (L2/L3).
    pub list: ComponentSet<ListCustomNode>,
    /// §9.4: `<ul>` containers.
    pub unordered: ComponentSet<ListCustomNode>,
    /// §9.4: `<ol>` containers.
    pub ordered: ComponentSet<ListCustomNode>,
}

/// §9.3/§9.4: the layers this plugin owns, in declaration order.
pub fn layers() -> [&'static str; 3] {
    [LIST_LAYER, UNORDERED_LAYER, ORDERED_LAYER]
}

/// The layer key `[[task.list.list.custom]]` addresses (the primary layer).
pub fn primary_layer() -> &'static str {
    LIST_LAYER
}

/// §9.3 L1: binds a container decorator line to the list below it.
pub fn process(events: &[Event], options: &ListOptions) -> Vec<Event> {
    let extras = ExtrasOptions::default();
    let bindings = bind_decorators(events, &extras, list_target);
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
        let target = match decorators.get(&i) {
            Some(decorator) => list_kind(&bindings.events, i).map(|kind| (kind, *decorator)),
            None => None,
        };
        let Some((kind, decorator)) = target else {
            out.push(bindings.events[i].clone());
            i += 1;
            continue;
        };

        let end = paragraph_end(&bindings.events, i);
        let node = container_node(&kind, decorator, options);
        out.push(Event::StartNode(node.clone()));
        // First attribute: the layer tells `plugin-markdown` which element the
        // wrapper stands for (§9.4).
        out.push(Event::Attribute {
            name: "__plugin_kind".to_string(),
            value: layer_of(&kind).to_string(),
        });
        out.push(Event::Attribute {
            name: "name".to_string(),
            value: node_name(&node, &kind),
        });
        if let Some(marker) = &decorator.type_marker {
            out.push(Event::Attribute {
                name: "type".to_string(),
                value: marker.clone(),
            });
        }
        for extras_warning in &decorator.attrs.warnings {
            out.push(diagnostic(&warning_message(extras_warning)));
        }
        emit_attrs(&decorator.attrs, &mut out);
        // The marker lines stay untouched: `plugin-markdown` parses them.
        out.extend(bindings.events[(i + 1)..end].iter().cloned());
        out.push(Event::EndNode(node));
        i = end + 1;
    }
    out
}

/// §11 rule 3: one template per entry of every layer the plugin owns.
pub fn solid_hints(options: &ListOptions) -> Option<SolidRenderHints> {
    let sets: [&ComponentSet<ListCustomNode>; 3] =
        [&options.list, &options.unordered, &options.ordered];
    if sets.iter().all(|set| set.is_empty()) {
        return None;
    }
    let mut hints = SolidRenderHints::default();
    for set in sets {
        for custom in set.components() {
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
    }
    Some(hints)
}

/// The `target` rule of [`bind_decorators`]: a paragraph that opens a list item.
///
/// The returned indentation is the block's own, which §9.1 compares a
/// decorator's indentation against.
pub(crate) fn list_target(events: &[Event], index: usize) -> Option<usize> {
    let line = first_line(events, index)?;
    item_marker(line).map(|(_, indent)| indent)
}

/// The list container a paragraph opens, if it opens one.
fn list_kind(events: &[Event], index: usize) -> Option<NodeKind> {
    item_marker(first_line(events, index)?).map(|(kind, _)| kind)
}

/// The first line of the paragraph at `index`, when the paragraph is the node at
/// `index`.
fn first_line<'a>(events: &'a [Event], index: usize) -> Option<&'a str> {
    if !matches!(
        events.get(index),
        Some(Event::StartNode(NodeKind::Paragraph))
    ) {
        return None;
    }
    for event in events.iter().skip(index + 1) {
        match event {
            Event::Text(text) if !text.trim().is_empty() => return Some(text),
            Event::Text(_) => {}
            Event::Attribute { .. } | Event::AttributeFlag { .. } | Event::Diagnostic { .. } => {}
            _ => return None,
        }
    }
    None
}

/// §9.3: the marker of a list item line — its kind and its indentation.
fn item_marker(line: &str) -> Option<(NodeKind, usize)> {
    let indent = line.len() - line.trim_start_matches(' ').len();
    let rest = &line[indent..];
    if rest.starts_with("- ") || rest.starts_with("* ") || rest.starts_with("+ ") {
        return Some((NodeKind::BulletList, indent));
    }
    let digits = rest.chars().take_while(|c| c.is_ascii_digit()).count();
    if digits > 0 && rest[digits..].starts_with(". ") {
        return Some((NodeKind::OrderedList, indent));
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

/// §9.4: the layer owning `kind`, the value the wrapper declares.
fn layer_of(kind: &NodeKind) -> &'static str {
    match kind {
        NodeKind::OrderedList => ORDERED_LAYER,
        _ => UNORDERED_LAYER,
    }
}

/// The component set of the container layer for `kind` (§9.4).
fn container_set<'a>(
    options: &'a ListOptions,
    kind: &NodeKind,
) -> &'a ComponentSet<ListCustomNode> {
    match kind {
        NodeKind::OrderedList => &options.ordered,
        _ => &options.unordered,
    }
}

/// §11 rule 3 + D8: the wrapper node of a decorated container.
fn container_node(kind: &NodeKind, decorator: &BoundDecorator, options: &ListOptions) -> NodeKind {
    match container_set(options, kind).select(decorator.type_marker.as_deref()) {
        Some(custom) => NodeKind::Custom(custom.name.clone()),
        None => NodeKind::Element(fallback_element(kind).to_string()),
    }
}

/// The built-in element of an unclaimed container `type` (§9.4, D8).
fn fallback_element(kind: &NodeKind) -> &'static str {
    match kind {
        NodeKind::OrderedList => "ol",
        _ => "ul",
    }
}

/// The `name` attribute: the component name, or the fallback element.
fn node_name(node: &NodeKind, kind: &NodeKind) -> String {
    match node {
        NodeKind::Custom(name) => name.clone(),
        _ => fallback_element(kind).to_string(),
    }
}

/// A §13 `Warning` carrying the plugin's name.
fn diagnostic(message: &str) -> Event {
    Event::Diagnostic {
        severity: Severity::Warning,
        message: format!("[list] {message}"),
        span: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pendon_core::{parse, Options};
    use pendon_renderer_solid::TypedComponent;

    fn component(name: &str) -> ListCustomNode {
        ListCustomNode {
            name: name.to_string(),
            template: format!("<{name}>{{children}}</{name}>"),
            imports: Vec::new(),
        }
    }

    fn options() -> ListOptions {
        ListOptions {
            list: ComponentSet::default(),
            unordered: ComponentSet::from_entries([
                TypedComponent::typed(vec!["unorderedA"], component("UL")),
                TypedComponent::default_component(component("ULDefault")),
            ]),
            ordered: ComponentSet::from_entries([TypedComponent::typed(
                vec!["orderedA"],
                component("OL"),
            )]),
        }
    }

    fn run(src: &str) -> Vec<Event> {
        run_with(src, &options())
    }

    fn run_with(src: &str, options: &ListOptions) -> Vec<Event> {
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

    /// §9.3 L1: the decorator line above the list decorates the container.
    #[test]
    fn a_container_decorator_binds_to_the_list() {
        let out = run("@@unorderedA{.u, #l03}\n\n- one\n- two\n");
        assert_eq!(custom(&out).as_deref(), Some("UL"), "{out:?}");
        assert_eq!(attr(&out, "__plugin_kind").as_deref(), Some("unordered"));
        assert_eq!(attr(&out, "type").as_deref(), Some("unorderedA"));
        assert_eq!(attr(&out, "class").as_deref(), Some("u"));
        assert_eq!(attr(&out, "id").as_deref(), Some("l03"));
        // The marker lines stay untouched for `plugin-markdown` (§9.3).
        assert!(text(&out).contains("- one"), "{:?}", text(&out));
        assert!(!text(&out).contains("@@"));
    }

    /// §11 rule 3: an untyped container decorator routes to the layer default.
    #[test]
    fn an_untyped_container_decorator_uses_the_layer_default() {
        let out = run("@@{.u}\n\n- one\n");
        assert_eq!(custom(&out).as_deref(), Some("ULDefault"), "{out:?}");
        assert_eq!(attr(&out, "type"), None);
        assert_eq!(attr(&out, "class").as_deref(), Some("u"));
    }

    /// §9.4: the marker decides the layer, not the decorator's type.
    #[test]
    fn an_ordered_list_uses_the_ordered_layer() {
        let out = run("@@orderedA{.o}\n\n1. one\n");
        assert_eq!(custom(&out).as_deref(), Some("OL"), "{out:?}");
        assert_eq!(attr(&out, "__plugin_kind").as_deref(), Some("ordered"));
    }

    /// §11 rule 3 + D8: without a component the container is the built-in
    /// element of its layer.
    #[test]
    fn without_a_component_the_container_is_the_built_in_element() {
        let empty = ListOptions::default();
        let out = run_with("@@{.u}\n\n- one\n", &empty);
        assert_eq!(element(&out).as_deref(), Some("ul"), "{out:?}");

        let out = run_with("@@{.o}\n\n1. one\n", &empty);
        assert_eq!(element(&out).as_deref(), Some("ol"), "{out:?}");
    }

    /// §9.3: a list without a decorator is left for `plugin-markdown`.
    #[test]
    fn an_undecorated_list_is_left_alone() {
        let out = run("- one\n- two\n");
        assert!(custom(&out).is_none(), "{out:?}");
        assert!(element(&out).is_none(), "{out:?}");
        assert!(text(&out).contains("- one"));
    }

    /// §9.4: the layer inventory and the hints (§11 rule 3).
    #[test]
    fn layers_and_hints() {
        assert_eq!(layers(), ["list", "unordered", "ordered"]);
        assert_eq!(primary_layer(), LIST_LAYER);
        assert!(solid_hints(&ListOptions::default()).is_none());
        assert_eq!(solid_hints(&options()).expect("hints").templates.len(), 3);
    }
}
