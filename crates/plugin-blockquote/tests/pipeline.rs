//! End-to-end checks: core → `plugin-blockquote` → `plugin-markdown` → solid.
//!
//! They prove the two things the crate exists for: the quote's extras become
//! attributes of the quote node, and `plugin-markdown` still parses the body
//! (a list inside the quote is a list, not raw text).

use pendon_core::{parse, Event, NodeKind, Options};
use pendon_plugin_blockquote::{process, solid_hints, BlockquoteCustomNode, BlockquoteOptions};
use pendon_renderer_solid::{render_solid_with_hints, ComponentSet, TypedComponent};

fn component(name: &str, template: &str) -> BlockquoteCustomNode {
    BlockquoteCustomNode {
        name: name.to_string(),
        template: template.to_string(),
        imports: Vec::new(),
    }
}

fn options() -> BlockquoteOptions {
    BlockquoteOptions {
        custom: ComponentSet::from_entries([
            TypedComponent::typed(
                vec!["quoteA"],
                component("QuoteA", "<QuoteA {...attrs}>{children}</QuoteA>"),
            ),
            TypedComponent::default_component(component(
                "QuoteDefault",
                "<QuoteDefault {...attrs}>{children}</QuoteDefault>",
            )),
        ]),
    }
}

/// The same set without a layer default (an unclaimed `type` then renders the
/// built-in `<blockquote>`, §11 rule 3 + D8).
fn typed_only() -> BlockquoteOptions {
    BlockquoteOptions {
        custom: ComponentSet::from_entries([TypedComponent::typed(
            vec!["quoteA"],
            component("QuoteA", "<QuoteA {...attrs}>{children}</QuoteA>"),
        )]),
    }
}

fn pipeline(src: &str) -> Vec<Event> {
    let events = parse(src, &Options::default());
    let mapped = process(&events, &options());
    pendon_plugin_markdown::process(&mapped)
}

/// §9.2: the body of the quote is still block content for `plugin-markdown`.
#[test]
fn the_quote_body_is_relexed_as_blocks() {
    let out = pipeline("> @@quoteA{.x}\n>\n> - one\n> - two\n");
    assert!(
        out.iter().any(
            |event| matches!(event, Event::StartNode(NodeKind::Custom(name)) if name == "QuoteA")
        ),
        "{out:?}"
    );
    assert!(
        out.iter()
            .any(|event| matches!(event, Event::StartNode(NodeKind::BulletList))),
        "the list inside the quote was not parsed: {out:?}"
    );
}

/// The full solid path: the component renders with its extras, and the internal
/// `__plugin_kind` marker never reaches the output (§11 rule 3).
#[test]
fn solid_rendering_of_a_typed_quote() {
    let events = pipeline("> @@quoteA{.x} quoted *text*\n");
    let jsx = render_solid_with_hints(&events, solid_hints(&options()).as_ref());
    assert!(jsx.contains("<QuoteA"), "{jsx}");
    assert!(jsx.contains("class={\"x\"}"), "{jsx}");
    assert!(jsx.contains("<em>text</em>"), "{jsx}");
    assert!(!jsx.contains("__plugin_kind"), "{jsx}");
}

/// §11 rule 3: an untyped quote routes to the layer default.
#[test]
fn solid_rendering_of_an_untyped_quote_uses_the_default() {
    let events = pipeline("> plain quote\n");
    let jsx = render_solid_with_hints(&events, solid_hints(&options()).as_ref());
    assert!(jsx.contains("<QuoteDefault"), "{jsx}");
}

/// §11 rule 3 + D8: without a default, an unclaimed quote renders the built-in
/// `<blockquote>` element and keeps the extras as plain attributes.
#[test]
fn solid_rendering_of_a_fallback_quote() {
    let events = {
        let events = parse("> @@quoteZ{.x} body\n", &Options::default());
        pendon_plugin_markdown::process(&process(&events, &typed_only()))
    };
    let jsx = render_solid_with_hints(&events, solid_hints(&typed_only()).as_ref());
    assert!(jsx.contains("<blockquote"), "{jsx}");
    assert!(jsx.contains("class=\"x\""), "{jsx}");
    assert!(jsx.contains("body"), "{jsx}");
}
