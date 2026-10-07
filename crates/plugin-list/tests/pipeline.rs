//! End-to-end checks: core → `plugin-list` → `plugin-markdown` → solid.
//!
//! They prove what the crate exists for: the container decorator's extras land on
//! the `<ul>` / `<ol>` element — or on the component that replaces it — instead
//! of around it (§9.3 L1, §9.4), and `plugin-markdown` still parses the markers.

use pendon_core::{parse, validate_events, Event, NodeKind, Options};
use pendon_plugin_list::{process, solid_hints, ListCustomNode, ListOptions};
use pendon_renderer_solid::{render_solid_with_hints, ComponentSet, TypedComponent};

fn component(name: &str) -> ListCustomNode {
    ListCustomNode {
        name: name.to_string(),
        template: format!("<{name} {{...attrs}}>{{children}}</{name}>"),
        imports: Vec::new(),
    }
}

/// §9.4: a claimed `unordered` / `ordered` container plus an `unordered` default.
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

/// The full pipeline: core → `plugin-list` → `plugin-markdown` → solid.
fn jsx(src: &str, options: &ListOptions) -> String {
    let events = parse(src, &Options::default());
    let mapped = process(&events, options);
    let markdown = pendon_plugin_markdown::process(&mapped);
    render_solid_with_hints(&markdown, solid_hints(options).as_ref())
}

/// The same pipeline, stopping at the Markdown events.
fn markdown(src: &str, options: &ListOptions) -> Vec<Event> {
    let events = parse(src, &Options::default());
    pendon_plugin_markdown::process(&process(&events, options))
}

fn has_node(events: &[Event], kind: NodeKind) -> bool {
    events
        .iter()
        .any(|event| matches!(event, Event::StartNode(node) if *node == kind))
}

/// §9.3 L1 + §9.4: the container decorator's `type` selects the component of the
/// `unordered` layer, and the extras land on it.
#[test]
fn a_container_decorator_lands_on_the_component() {
    let jsx = jsx("@@unorderedA{.u}\n\n- one\n- two\n", &options());
    // Exactly one `<UL>`: the wrapper *is* the container, it does not nest.
    assert_eq!(jsx.matches("<UL").count(), 1, "{jsx}");
    assert!(jsx.contains(r#"class={"u"}"#), "{jsx}");
    assert!(jsx.contains(r#"type={"unorderedA"}"#), "{jsx}");
    assert!(jsx.contains("<li>one</li>"), "{jsx}");
    assert!(jsx.contains("<li>two</li>"), "{jsx}");
    // The built-in container is gone.
    assert!(!jsx.contains("<ul"), "{jsx}");
}

/// §11 rule 3: an untyped decorator routes to the layer default.
#[test]
fn an_untyped_container_uses_the_layer_default() {
    let jsx = jsx("@@{.u}\n\n- one\n", &options());
    assert_eq!(jsx.matches("<ULDefault").count(), 1, "{jsx}");
    assert!(jsx.contains(r#"class={"u"}"#), "{jsx}");
    assert!(jsx.contains("<li>one</li>"), "{jsx}");
}

/// §11 rule 3 + D8: without a component the extras land on the built-in `<ul>`.
#[test]
fn an_unclaimed_container_still_lands_on_the_element() {
    let jsx = jsx("@@{.u}\n\n- one\n", &ListOptions::default());
    assert!(jsx.contains("<ul"), "{jsx}");
    assert!(jsx.contains(r#"class="u""#), "{jsx}");
    assert!(jsx.contains("<li>one</li>"), "{jsx}");
    // One container, one item: nothing wrapped the list.
    assert_eq!(jsx.matches("<ul").count(), 1, "{jsx}");
    assert_eq!(jsx.matches("<li").count(), 1, "{jsx}");
}

/// §9.4: `start` comes from the first item's number and survives the merge.
#[test]
fn an_ordered_container_keeps_the_start_offset() {
    let jsx = jsx("@@orderedA{.o}\n\n6. Goo\n", &options());
    assert_eq!(jsx.matches("<OL").count(), 1, "{jsx}");
    assert!(jsx.contains(r#"class={"o"}"#), "{jsx}");
    assert!(jsx.contains("start={6}"), "{jsx}");
    assert!(jsx.contains("<li>Goo</li>"), "{jsx}");
    assert!(!jsx.contains("<ol"), "{jsx}");
}

/// The merge at event level: no `<ul>`/`<ol>` node of its own is built, the items
/// live directly in the wrapper, and the stream stays balanced.
#[test]
fn the_list_is_built_inside_the_wrapper() {
    let events = markdown("@@unorderedA{.u}\n\n- one\n- two\n", &options());
    assert!(
        !has_node(&events, NodeKind::BulletList),
        "the list opened a container of its own: {events:?}"
    );
    assert!(has_node(&events, NodeKind::ListItem), "{events:?}");
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::StartNode(NodeKind::Custom(name)) if name == "UL")),
        "{events:?}"
    );
    assert!(validate_events(&events).is_empty(), "{events:?}");
}

/// The wrapper carries the *next* list only: a list after it starts a fresh
/// container of its own.
#[test]
fn a_decorated_list_does_not_leak_into_the_next_list() {
    let jsx = jsx("@@{.u}\n\n- one\n\n- two\n", &ListOptions::default());
    assert_eq!(jsx.matches("<ul").count(), 2, "{jsx}");
    assert_eq!(jsx.matches(r#"class="u""#).count(), 1, "{jsx}");
    assert_eq!(jsx.matches("<li>").count(), 2, "{jsx}");
}

/// §9.3 L5: a nested list keeps its own container; the container extras stay on
/// the wrapper that decorated the outer list.
#[test]
fn a_nested_list_keeps_its_own_container() {
    let jsx = jsx("@@{.u}\n\n- a\n  - b\n- c\n", &ListOptions::default());
    assert_eq!(jsx.matches("<ul").count(), 2, "{jsx}");
    assert_eq!(jsx.matches(r#"class="u""#).count(), 1, "{jsx}");
    assert!(jsx.contains("<li>a<ul>"), "{jsx}");
    assert_eq!(jsx.matches("<li>").count(), 3, "{jsx}");
}
