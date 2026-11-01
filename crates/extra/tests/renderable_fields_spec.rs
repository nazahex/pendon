//! `;;…;;` / `;;;…;;;` renderable fields at the grammar layer.
//!
//! `docs/spec/SYNTAX.md` §6.7, `docs/decisions/0021-renderable-fields.md`. These
//! tests pin what the **head lexers** must do and nothing more: detect a whole
//! span, keep its interior opaque, and transport it as [`AttrValue::Mdp`]. The
//! resolve stage and the renderers are covered elsewhere — this crate never
//! renders and never looks at a plugin.

use pendon_core::{mdp_attr_payload, Event, MdpMode};
use pendon_extra::{
    emit_attrs, parse_extras, parse_slot_parts, parse_slots, to_attributes, AttrValue, ExtrasAttr,
    ExtrasHead, ExtrasMatch, Slot, SlotKind, SlotMatch, SlotParts, Specs,
};

/// A renderable attribute value.
fn mdp(mode: MdpMode, source: &str) -> ExtrasAttr {
    ExtrasAttr::Value(AttrValue::Mdp {
        mode,
        source: source.to_string(),
    })
}

fn text(value: &str) -> ExtrasAttr {
    ExtrasAttr::Value(AttrValue::Str(value.to_string()))
}

/// Parses `input` as one complete head.
fn head_of(input: &str) -> ExtrasHead {
    match parse_extras(input) {
        ExtrasMatch::Head { head, rest } => {
            assert_eq!(rest, "", "head should consume the whole input");
            head
        }
        other => panic!("expected a head for {input:?}, got {other:?}"),
    }
}

/// The attributes of a head, in §6.4 order.
fn attrs_of(input: &str) -> Vec<(String, ExtrasAttr)> {
    to_attributes(&head_of(input), &Specs::default()).items
}

fn parts_of(content: &str) -> SlotParts {
    parse_slot_parts(content).expect("expected valid slot content")
}

/// The `(mode, source)` of one slot content, when it is renderable.
fn render_of(parts: &SlotParts, kind: SlotKind) -> Option<(MdpMode, &str)> {
    parts
        .render(kind)
        .map(|render| (render.mode, render.source.as_str()))
}

#[test]
fn all_six_slot_positions_accept_a_renderable() {
    // The same span in every position of the two groups: what a slot *is*
    // differs, what it *holds* never does (§5.2).
    let head = head_of("@@t[;;b;; \";;q;;\" `;;t;;`](;;pb;; \";;pq;;\" `;;pt;;`)");
    for (slot, kind, source) in [
        (Slot::Bracket, SlotKind::Bare, "b"),
        (Slot::Bracket, SlotKind::Quote, "q"),
        (Slot::Bracket, SlotKind::Tick, "t"),
        (Slot::Paren, SlotKind::Bare, "pb"),
        (Slot::Paren, SlotKind::Quote, "pq"),
        (Slot::Paren, SlotKind::Tick, "pt"),
    ] {
        let render = head
            .slots
            .render(slot, kind)
            .unwrap_or_else(|| panic!("{slot:?}/{kind:?} should be renderable"));
        assert_eq!(render.mode, MdpMode::Inline, "{slot:?}/{kind:?}");
        assert_eq!(render.source, source, "{slot:?}/{kind:?}");
        // The text a literal consumer reads is the authored spelling.
        assert_eq!(
            head.slots.get(slot, kind),
            Some(match slot {
                Slot::Bracket => match kind {
                    SlotKind::Bare => ";;b;;",
                    SlotKind::Quote => ";;q;;",
                    SlotKind::Tick => ";;t;;",
                },
                Slot::Paren => match kind {
                    SlotKind::Bare => ";;pb;;",
                    SlotKind::Quote => ";;pq;;",
                    SlotKind::Tick => ";;pt;;",
                },
            }),
            "{slot:?}/{kind:?}"
        );
    }
}

#[test]
fn block_and_inline_are_different_modes() {
    assert_eq!(
        render_of(&parts_of(";;;a;;;"), SlotKind::Bare),
        Some((MdpMode::Block, "a"))
    );
    assert_eq!(
        render_of(&parts_of(";;a;;"), SlotKind::Bare),
        Some((MdpMode::Inline, "a"))
    );
}

#[test]
fn the_interior_is_opaque_to_the_slot_lexer() {
    // Every character that would otherwise end a content, split a content or
    // close the bracket.
    let parts = parts_of(r#";;a, b "c" [d] {e} (f): g;;"#);
    assert_eq!(
        render_of(&parts, SlotKind::Bare),
        Some((MdpMode::Inline, r#"a, b "c" [d] {e} (f): g"#))
    );
    // The space inside the payload does not start a second content either.
    assert!(parts.quote.is_none());
    assert!(parts.tick.is_none());
}

#[test]
fn a_bracket_still_closes_after_a_renderable() {
    // A `]` inside the payload must not close the slot: the head continues to
    // the paren.
    let head = head_of("@@t[;;a ] b;;](;;c ) d;;)");
    assert_eq!(
        head.slots
            .render(Slot::Bracket, SlotKind::Bare)
            .map(|r| r.source.as_str()),
        Some("a ] b")
    );
    assert_eq!(
        head.slots
            .render(Slot::Paren, SlotKind::Bare)
            .map(|r| r.source.as_str()),
        Some("c ) d")
    );
}

#[test]
fn a_modifier_run_after_a_renderable_still_applies() {
    let head = head_of("@@t[;;a;; |x]");
    assert_eq!(
        head.slots
            .render(Slot::Bracket, SlotKind::Bare)
            .map(|r| r.source.as_str()),
        Some("a")
    );
    assert!(head.slots.has_modifier('x'));
}

#[test]
fn a_slot_scan_never_splits_a_renderable() {
    // The tag path (`parse_slots`) shares the content scanner, so a `;;`
    // renderable in a document head behaves exactly like an attached group.
    match parse_slots("@[;;a b;;](/x)") {
        SlotMatch::Slots { pair, rest } => {
            assert_eq!(rest, "");
            assert_eq!(
                pair.render(Slot::Bracket, SlotKind::Bare)
                    .map(|r| r.source.as_str()),
                Some("a b")
            );
            assert_eq!(pair.get(Slot::Paren, SlotKind::Bare), Some("/x"));
        }
        other => panic!("expected a slot head, got {other:?}"),
    }
}

#[test]
fn specs_values_accept_a_renderable() {
    // The `:` `,` `"` `[` `]` `{` `}` inside the payload are payload.
    let items = attrs_of(r#"@@t{note: ;;a: b, "c" [d] {e};;, d: 1}"#);
    assert_eq!(
        items,
        vec![
            (
                "note".to_string(),
                mdp(MdpMode::Inline, r#"a: b, "c" [d] {e}"#)
            ),
            ("d".to_string(), ExtrasAttr::Value(AttrValue::Int(1))),
        ]
    );
}

#[test]
fn a_block_span_is_a_block_value() {
    // A spec value is one line, so a block payload writes its line breaks as
    // `\n`; materialisation turns them into real newlines (§6.7).
    let items = attrs_of("@@t{body: ;;;## A heading\\n\\n- one;;;}");
    assert_eq!(
        items,
        vec![(
            "body".to_string(),
            mdp(MdpMode::Block, "## A heading\n\n- one")
        )]
    );
}

#[test]
fn a_quoted_scalar_may_carry_one_span() {
    // The quotes are an escape hatch, not a downgrade: their body is still one
    // span, so the value is renderable (§6.7).
    let items = attrs_of(r#"@@t{note: ";;x y;;"}"#);
    assert_eq!(
        items,
        vec![("note".to_string(), mdp(MdpMode::Inline, "x y"))]
    );
    // Quotes around *more* than a span are a string, as before.
    let items = attrs_of(r#"@@t{note: "pre ;;x;; post"}"#);
    assert_eq!(items, vec![("note".to_string(), text("pre ;;x;; post"))]);
}

#[test]
fn a_renderable_nests_inside_an_object_and_an_array() {
    let inline = AttrValue::Mdp {
        mode: MdpMode::Inline,
        source: "a, b".to_string(),
    };
    let items = attrs_of("@@t{meta: {body: ;;a, b;;, n: 1}}");
    assert_eq!(
        items,
        vec![(
            "meta".to_string(),
            ExtrasAttr::Value(AttrValue::Object(vec![
                ("body".to_string(), inline.clone()),
                ("n".to_string(), AttrValue::Int(1)),
            ]))
        )]
    );

    let items = attrs_of("@@t{list: [;;a, b;;, 2]}");
    assert_eq!(
        items,
        vec![(
            "list".to_string(),
            ExtrasAttr::Value(AttrValue::Array(vec![inline, AttrValue::Int(2)]))
        )]
    );
}

#[test]
fn an_unclosed_span_stays_literal_text() {
    // §4.3: nothing is consumed, so the whole head is literal — an unclosed
    // renderable never half-parses.
    match parse_extras("@@t{note: ;;a b}") {
        ExtrasMatch::Malformed { .. } => {}
        other => panic!("expected a malformed head, got {other:?}"),
    }
    // `;;a b` as a bare slot content has no closer, so it is ordinary text.
    let parts = parts_of(";;a b");
    assert_eq!(parts.get(SlotKind::Bare), Some(";;a b"));
    assert!(parts.render(SlotKind::Bare).is_none());
}

#[test]
fn a_span_that_is_not_whole_is_not_a_renderable() {
    let parts = parts_of(";;a;; tail");
    assert_eq!(parts.get(SlotKind::Bare), Some(";;a;; tail"));
    assert!(parts.render(SlotKind::Bare).is_none());
}

#[test]
fn literal_consumers_keep_the_authored_span_text() {
    // `.class`, `#id`, `style:` and `--var` are not field values, so they see
    // the text — nothing disappears, and nothing becomes a fragment where a
    // string is required (D-RF-6).
    let items = attrs_of("@@t{.c, class: ;;x;;}");
    assert_eq!(
        items,
        vec![("class".to_string(), text("c ;;x;;"))],
        "a class list accumulates as text"
    );

    let items = attrs_of("@@t{#i}");
    assert_eq!(items, vec![("id".to_string(), text("i"))]);
    // `.class` and `#id` have a *name* grammar, so a span is not spellable
    // there at all — the head is rejected, not silently degraded.
    assert!(matches!(
        parse_extras("@@t{#;;i;;}"),
        ExtrasMatch::Malformed { .. }
    ));

    let items = attrs_of("@@t{style: ;;y;;}");
    assert_eq!(items, vec![("style".to_string(), text(";;y;;"))]);

    let items = attrs_of("@@t{--tone: ;;z;;}");
    assert_eq!(items, vec![("style".to_string(), text("--tone: ;;z;;"))]);

    let items = attrs_of("@@t{isFoo}");
    assert_eq!(items, vec![("isFoo".to_string(), ExtrasAttr::Flag)]);
}

#[test]
fn the_transport_keeps_the_sentinel() {
    // The latent bug this feature needs fixed: `emit_attrs` must transport, not
    // literalise, or the value would reach the resolver as text.
    let head = head_of("@@t{note: ;;a;;}");
    let attrs = to_attributes(&head, &Specs::default());
    let mut events = Vec::new();
    emit_attrs(&attrs, &mut events);
    let value = events
        .iter()
        .find_map(|event| match event {
            Event::Attribute { name, value } if name == "note" => Some(value),
            _ => None,
        })
        .expect("note attribute");
    assert_eq!(mdp_attr_payload(value), Some((MdpMode::Inline, "a")));
}
