//! End-to-end checks: core → `plugin-directive` → `plugin-markdown` → solid.
//!
//! These prove the two things the crate exists for: a directive becomes a
//! `Custom` node, and `plugin-markdown` re-lexes its content in the matching
//! mode (§10.2/§10.3).

use pendon_core::{parse, Event, NodeKind, Options};
use pendon_extra::PositionalKeys;
use pendon_plugin_directive::{process, solid_hints, DirectiveCustomNode, DirectiveOptions};
use pendon_renderer_solid::{render_solid_with_hints, ComponentSet, TypedComponent};

fn component(name: &str, template: &str) -> DirectiveCustomNode {
    DirectiveCustomNode {
        name: name.to_string(),
        template: template.to_string(),
        imports: Vec::new(),
    }
}

fn options() -> DirectiveOptions {
    DirectiveOptions {
        custom: ComponentSet::from_entries([
            TypedComponent::typed(
                vec!["note"],
                component("Note", "<Note {...attrs}>{children}</Note>"),
            ),
            TypedComponent::default_component(component(
                "DirectiveDefault",
                "<DirectiveDefault {...attrs}>{children}</DirectiveDefault>",
            )),
        ]),
    }
}

fn pipeline(src: &str) -> Vec<Event> {
    let events = parse(src, &Options::default());
    let mapped = process(&events, &options());
    pendon_plugin_markdown::process(&mapped)
}

fn has(events: &[Event], kind: NodeKind) -> bool {
    events
        .iter()
        .any(|event| matches!(event, Event::StartNode(k) if *k == kind))
}

/// All `Text` events concatenated (markdown may split a line into many events).
fn text_content(events: &[Event]) -> String {
    events
        .iter()
        .filter_map(|event| match event {
            Event::Text(t) => Some(t.as_str()),
            _ => None,
        })
        .collect()
}

/// The value of the first `name` attribute, `None` when nothing emitted it.
fn attr(events: &[Event], name: &str) -> Option<String> {
    events.iter().find_map(|event| match event {
        Event::Attribute { name: key, value } if key == name => Some(value.clone()),
        _ => None,
    })
}

/// §6.1 + §11 rule 5: a component entry renames all four positional slots
/// independently, and an entry that overrides none keeps `slug` / `title`.
#[test]
fn each_type_resolves_its_own_positional_keys() {
    let opts = DirectiveOptions {
        custom: ComponentSet::from_entries([
            TypedComponent::typed(
                vec!["note"],
                component("Note", "<Note {...attrs}>{children}</Note>"),
            )
            .with_positional(PositionalKeys {
                bracket_key: Some("label".into()),
                parentheses_key: Some("kind".into()),
                backtick_key: Some("author".into()),
                quote_key: Some("summary".into()),
            }),
            TypedComponent::default_component(component(
                "DirectiveDefault",
                "<DirectiveDefault {...attrs}>{children}</DirectiveDefault>",
            )),
        ]),
    };

    let run = |src: &str| {
        let events = parse(src, &Options::default());
        process(&events, &opts)
    };

    // The overridden entry: every slot lands on its configured key…
    let out = run("::note[A](\"B\"){`C`, \"D\"}Body::");
    assert!(has(&out, NodeKind::Custom("Note".to_string())), "{out:?}");
    assert_eq!(attr(&out, "label"), Some("A".to_string()));
    assert_eq!(attr(&out, "kind"), Some("B".to_string()));
    assert_eq!(attr(&out, "author"), Some("C".to_string()));
    assert_eq!(attr(&out, "summary"), Some("D".to_string()));
    // …and nothing spills onto the built-in slot the key was renamed away from.
    assert_eq!(attr(&out, "slug"), None, "{out:?}");
    assert_eq!(attr(&out, "title"), None, "{out:?}");

    // A type with no positional override falls through to the layer default,
    // which overrides nothing, so the §6.1 defaults apply (head > extras §6.2).
    let out = run("::other[X](\"Y\"){`Z`, \"W\"}Body::");
    assert_eq!(attr(&out, "slug"), Some("X".to_string()), "{out:?}");
    assert_eq!(attr(&out, "title"), Some("Y".to_string()), "{out:?}");
    assert_eq!(attr(&out, "label"), None, "{out:?}");
}

/// §10.3: the body is parsed as block content.
#[test]
fn block_body_is_relexed_as_blocks() {
    let out = pipeline("==note\n# Heading\n\n- one\n- two\n==\n");
    assert!(has(&out, NodeKind::Custom("Note".to_string())), "{out:?}");
    assert!(has(&out, NodeKind::Heading), "heading missing: {out:?}");
    assert!(has(&out, NodeKind::BulletList), "list missing: {out:?}");
    assert!(
        !text_content(&out).contains("=="),
        "fence leaked: {:?}",
        text_content(&out)
    );
}

/// §10.2: the content is parsed as inline content.
#[test]
fn inline_body_is_relexed_inline() {
    let out = pipeline("::note *em* and `code`::\n");
    assert!(has(&out, NodeKind::Custom("Note".to_string())), "{out:?}");
    assert!(has(&out, NodeKind::Emphasis), "emphasis missing: {out:?}");
    assert!(has(&out, NodeKind::InlineCode), "code missing: {out:?}");
}

/// The full solid path: the component renders with its extras, never leaking the
/// internal `__plugin_kind` marker (§11 rule 3).
#[test]
fn solid_rendering_of_a_block_directive() {
    let events = pipeline("==note[slug-1]\nbody text\n==\n");
    let jsx = render_solid_with_hints(&events, solid_hints(&options()).as_ref());
    assert!(jsx.contains("<Note"), "{jsx}");
    assert!(jsx.contains("slug-1"), "{jsx}");
    assert!(jsx.contains("body text"), "{jsx}");
    assert!(!jsx.contains("__plugin_kind"), "{jsx}");
    assert!(!jsx.contains("=="), "{jsx}");
}

/// An inline directive keeps its paragraph and its surrounding text.
#[test]
fn solid_rendering_of_an_inline_directive() {
    let events = pipeline("Before ::note hi:: after\n");
    let jsx = render_solid_with_hints(&events, solid_hints(&options()).as_ref());
    assert!(jsx.contains("<Note"), "{jsx}");
    assert!(jsx.contains("Before"), "{jsx}");
    assert!(jsx.contains("after"), "{jsx}");
}

/// §10.2: empty inline content still opens and closes the directive.
#[test]
fn inline_empty_content_still_directive() {
    let out = pipeline("::note::\n");
    assert!(has(&out, NodeKind::Custom("Note".to_string())), "{out:?}");
    assert!(
        !text_content(&out).contains("::"),
        "fence leaked: {:?}",
        text_content(&out)
    );
}

/// §10.3: an empty block body still opens and closes the directive.
#[test]
fn block_empty_content_still_directive() {
    let out = pipeline("==note\n==\n");
    assert!(has(&out, NodeKind::Custom("Note".to_string())), "{out:?}");
    assert!(
        !text_content(&out).contains("=="),
        "fence leaked: {:?}",
        text_content(&out)
    );
}

/// A colon run with no type marker stays literal text (§4.3).
#[test]
fn inline_without_type_is_literal() {
    let out = pipeline("Before ::: after\n");
    assert!(
        !out.iter()
            .any(|event| matches!(event, Event::StartNode(NodeKind::Custom(_)))),
        "unexpected directive: {out:?}"
    );
    assert!(
        text_content(&out).contains(":::"),
        "literal lost: {:?}",
        text_content(&out)
    );
}

/// A bare `=` fence with nothing open stays literal text (§10.3).
#[test]
fn bare_block_fence_with_nothing_open_is_literal() {
    let out = pipeline("Before\n\n====\n");
    assert!(
        !out.iter()
            .any(|event| matches!(event, Event::StartNode(NodeKind::Custom(_)))),
        "unexpected directive: {out:?}"
    );
    assert!(
        text_content(&out).contains("===="),
        "literal lost: {:?}",
        text_content(&out)
    );
}

/// Delimiter collision: a `=` underline after a paragraph line is not a block
/// fence. (Pendon has no setext headings — §6 is ATX-only — so the line must
/// stay literal paragraph text, never a directive.)
#[test]
fn paragraph_underline_is_not_a_block_fence() {
    let out = pipeline("Title\n===\n");
    assert!(
        !out.iter()
            .any(|event| matches!(event, Event::StartNode(NodeKind::Custom(_)))),
        "unexpected directive: {out:?}"
    );
    assert!(
        text_content(&out).contains("==="),
        "literal lost: {:?}",
        text_content(&out)
    );
}

/// Delimiter collision: `std::vector` must not open an inline directive.
#[test]
fn cpp_scope_resolution_stays_literal() {
    let out = pipeline("Use std::vector here\n");
    assert!(
        !out.iter()
            .any(|event| matches!(event, Event::StartNode(NodeKind::Custom(_)))),
        "unexpected directive: {out:?}"
    );
    assert!(
        text_content(&out).contains("std::vector"),
        "literal lost: {:?}",
        text_content(&out)
    );
}
