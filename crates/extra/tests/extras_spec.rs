//! Spec conformance tests for the typed extras head and directive heads.
//!
//! Every case cites the section of `docs/spec/SYNTAX.md` that it locks down.

use pendon_extra::{
    parse_directive_head, parse_extras, parse_type_marker, to_attributes, AttrValue, Attrs,
    DirectiveMatch, DirectiveSigil, ExtrasAttr, ExtrasError, ExtrasHead, ExtrasMatch,
    ExtrasOptions, ExtrasWarning, PositionalKeys,
};

/// Parses `input` as a complete head and maps it to attributes.
fn attrs_of(input: &str) -> (Vec<(String, ExtrasAttr)>, Vec<ExtrasWarning>) {
    let head = head_of(input);
    let attrs = to_attributes(&head, &ExtrasOptions::default());
    (attrs.items, attrs.warnings)
}

/// Parses `input` that must consist of a single head.
fn head_of(input: &str) -> ExtrasHead {
    match parse_extras(input) {
        ExtrasMatch::Head { head, rest } => {
            assert_eq!(rest, "", "head should consume the whole input");
            head
        }
        other => panic!("expected a head for {input:?}, got {other:?}"),
    }
}

/// Asserts the §4.3 fallback and returns the rejection reason.
fn literal_error(input: &str) -> ExtrasError {
    match parse_extras(input) {
        ExtrasMatch::Malformed { error, rest } => {
            assert_eq!(rest, input, "a malformed head consumes nothing (§4.3)");
            error
        }
        other => panic!("expected literal fallback for {input:?}, got {other:?}"),
    }
}

#[test]
fn head_must_be_adjacent() {
    assert!(matches!(
        parse_extras(" @@type{a}"),
        ExtrasMatch::Absent { .. }
    ));
    assert!(matches!(
        parse_extras("x@@type{a}"),
        ExtrasMatch::Absent { .. }
    ));
    assert!(matches!(
        parse_extras("@@type{a}"),
        ExtrasMatch::Head { .. }
    ));
}

#[test]
fn returns_the_trailing_text() {
    let ExtrasMatch::Head { head, rest } = parse_extras("@@note{.a} body") else {
        panic!("expected a head");
    };
    assert_eq!(head.type_marker.as_deref(), Some("note"));
    assert_eq!(rest, " body");
}

#[test]
fn malformed_heads_fall_back_to_literal_text() {
    use ExtrasError::*;

    assert_eq!(literal_error("@@type{"), UnterminatedHead);
    assert_eq!(literal_error("@@type{.hero"), UnterminatedHead);
    assert_eq!(literal_error("@@type{foo: }"), EmptyValue);
    assert_eq!(literal_error("@@type{1x: 2}"), InvalidItem);
    assert_eq!(literal_error("@@type{\"a\" junk}"), TrailingCharacters);
    assert_eq!(
        literal_error("@@type{foo: unquoted spaces}"),
        UnquotedWhitespace
    );
    assert_eq!(literal_error("@@type{`unclosed}"), UnterminatedHead);
    assert_eq!(
        pendon_extra::parse_extras_body("`a").unwrap_err(),
        UnterminatedValue
    );
    // §4.4: the type charset is ASCII alphanumeric.
    assert_eq!(literal_error("@@1x{a}"), InvalidHead);
    // §4.1: the `{` must touch the type.
    assert_eq!(literal_error("@@type {a}"), InvalidHead);
    // §4.3: a head may not span lines.
    assert_eq!(literal_error("@@type{a\nb}"), UnterminatedHead);
}

#[test]
fn a_type_only_head_carries_the_type_and_nothing_else() {
    // §4.1: the run after the type is literal text, the type is kept.
    let ExtrasMatch::Head { head, rest } = parse_extras("@@anchorA. tail") else {
        panic!("expected a type-only head");
    };
    assert_eq!(head.type_marker.as_deref(), Some("anchorA"));
    assert!(head.items.is_empty());
    assert_eq!(rest, ". tail");

    // `@@type-x{.x}` is the type `type` plus literal text.
    let ExtrasMatch::Head { head, rest } = parse_extras("@@type-x{.x}") else {
        panic!("expected a type-only head");
    };
    assert_eq!(head.type_marker.as_deref(), Some("type"));
    assert_eq!(rest, "-x{.x}");

    // …and a type at the very end of the text.
    let ExtrasMatch::Head { head, rest } = parse_extras("@@anchorA") else {
        panic!("expected a type-only head");
    };
    assert_eq!(head.type_marker.as_deref(), Some("anchorA"));
    assert_eq!(rest, "");
}

#[test]
fn empty_heads_are_valid() {
    // §3: `{}` and `@@{}` are the same empty head, no warning.
    for source in ["{}", "@@{}", "@@type{}"] {
        let ExtrasMatch::Head { head, rest } = parse_extras(source) else {
            panic!("expected a head for {source:?}");
        };
        assert!(head.items.is_empty(), "{source:?}");
        assert_eq!(rest, "", "{source:?}");
    }
}

#[test]
fn the_bare_spelling_is_a_head() {
    // §3: `{…}` and `@@{…}` are equivalent, type marker aside.
    let ExtrasMatch::Head { head, rest } = parse_extras("{.hero, #id} tail") else {
        panic!("expected a bare head");
    };
    assert_eq!(head.type_marker, None);
    assert_eq!(head.items.len(), 2);
    assert_eq!(rest, " tail");

    let ExtrasMatch::Head { head, .. } = parse_extras("@@{.hero}") else {
        panic!("expected a prefixed head");
    };
    assert_eq!(head.type_marker, None);
    assert_eq!(head.items.len(), 1);
}

#[test]
fn a_head_without_a_type_needs_items() {
    let head = head_of("@@{.hero}");
    assert_eq!(head.type_marker, None);
    assert_eq!(head.items.len(), 1);
}

#[test]
fn values_are_typed_per_section_5() {
    let (items, warnings) = attrs_of(r#"@@type{a: "12", b: 12, c: 6.0, d: true, e: red}"#);
    assert_eq!(warnings, vec![]);
    assert_eq!(
        items,
        vec![
            (
                "a".to_string(),
                ExtrasAttr::Value(AttrValue::Str("12".into()))
            ),
            ("b".to_string(), ExtrasAttr::Value(AttrValue::Int(12))),
            ("c".to_string(), ExtrasAttr::Value(AttrValue::Float(6.0))),
            ("d".to_string(), ExtrasAttr::Value(AttrValue::Bool(true))),
            (
                "e".to_string(),
                ExtrasAttr::Value(AttrValue::Raw("red".into()))
            ),
        ]
    );
}

#[test]
fn classes_accumulate_and_other_items_are_last_wins() {
    let (items, warnings) = attrs_of("@@type{.a, .b, #first, #second, k: \"1\", k: \"2\"}");
    assert_eq!(
        items,
        vec![
            (
                "class".to_string(),
                ExtrasAttr::Value(AttrValue::Str("a b".into()))
            ),
            (
                "id".to_string(),
                ExtrasAttr::Value(AttrValue::Str("second".into()))
            ),
            (
                "k".to_string(),
                ExtrasAttr::Value(AttrValue::Str("2".into()))
            ),
        ]
    );
    assert_eq!(warnings, vec![ExtrasWarning::DuplicateId]);
}

#[test]
fn empty_items_are_ignored() {
    let (items, _) = attrs_of("@@type{a: \"1\",,b: \"2\",}");
    assert_eq!(items.len(), 2);
}

#[test]
fn positional_keys_are_emitted_slug_then_title() {
    let head = head_of("@@type{\"T\", .c, `s`}");
    let keys: Vec<String> = to_attributes(&head, &ExtrasOptions::default())
        .items
        .into_iter()
        .map(|(key, _)| key)
        .collect();
    assert_eq!(keys, vec!["class", "slug", "title"]);
}

#[test]
fn positional_keys_follow_component_config() {
    let head = head_of("@@type{`a`, \"L\"}");
    let items = to_attributes(&head, &ExtrasOptions::new("anchorId", "label")).items;
    assert_eq!(items[0].0, "anchorId");
    assert_eq!(items[1].0, "label");
}

/// §11 rule 5: the config-side overrides are all optional, so an entry that
/// overrides nothing keeps every built-in §6.1 key, and a partial override
/// fills only the gaps it left.
#[test]
fn positional_key_overrides_resolve_unset_keys_to_the_defaults() {
    let empty = PositionalKeys::default();
    assert!(empty.is_empty());
    assert_eq!(empty.resolve(), ExtrasOptions::default());

    let partial = PositionalKeys {
        parentheses_key: Some("level".into()),
        ..PositionalKeys::default()
    };
    assert!(!partial.is_empty());
    assert_eq!(
        partial.resolve(),
        ExtrasOptions {
            parentheses_key: "level".into(),
            ..ExtrasOptions::default()
        }
    );

    // The four slots stay independent: overriding one never touches the others.
    let all = PositionalKeys {
        backtick_key: Some("ref".into()),
        quote_key: Some("blurb".into()),
        bracket_key: Some("label".into()),
        parentheses_key: Some("kind".into()),
    };
    assert!(!all.is_empty());
    assert_eq!(
        all.resolve(),
        ExtrasOptions {
            backtick_key: "ref".into(),
            quote_key: "blurb".into(),
            bracket_key: "label".into(),
            parentheses_key: "kind".into(),
        }
    );
}

#[test]
fn css_vars_and_style_props_merge_into_one_style_attribute() {
    let (items, _) = attrs_of(r#"@@type{.a, --tone: red, style: "color: blue", --size: 2rem}"#);
    assert_eq!(
        items,
        vec![
            (
                "class".to_string(),
                ExtrasAttr::Value(AttrValue::Str("a".into()))
            ),
            (
                "style".to_string(),
                ExtrasAttr::Value(AttrValue::Str(
                    "--tone: red; color: blue; --size: 2rem".into()
                ))
            ),
        ]
    );
}

#[test]
fn duplicate_keys_collapse_to_one_attribute() {
    // §6.4: the positional slot keeps its position, the later value wins.
    let (items, warnings) = attrs_of("@@type{`s`, slug: \"other\"}");
    assert_eq!(
        items,
        vec![(
            "slug".to_string(),
            ExtrasAttr::Value(AttrValue::Str("other".into()))
        )]
    );
    assert_eq!(
        warnings,
        vec![ExtrasWarning::Overridden {
            key: "slug".to_string()
        }]
    );
}

#[test]
fn class_props_accumulate_with_class_items() {
    let (items, warnings) = attrs_of("@@type{.a, class: \"text-lg\", .b}");
    assert_eq!(
        items,
        vec![(
            "class".to_string(),
            ExtrasAttr::Value(AttrValue::Str("a text-lg b".into()))
        )]
    );
    assert_eq!(warnings, vec![]);
}

#[test]
fn flags_stay_bare_in_the_dom_fallback() {
    let head = head_of("@@type{isBar, role: tab}");
    let attrs = to_attributes(&head, &ExtrasOptions::default());
    assert_eq!(
        attrs.to_dom(),
        vec![
            ("isBar".to_string(), "isBar".to_string()),
            ("role".to_string(), "tab".to_string()),
        ]
    );
}

#[test]
fn escapes_are_consumed() {
    let (items, _) = attrs_of(r#"@@type{title: "a\,b", note: 'c\'d'}"#);
    assert_eq!(
        items,
        vec![
            (
                "title".to_string(),
                ExtrasAttr::Value(AttrValue::Str("a,b".into()))
            ),
            (
                "note".to_string(),
                ExtrasAttr::Value(AttrValue::Str("c'd".into()))
            ),
        ]
    );
}

#[test]
fn the_head_wins_over_extras() {
    let extras = {
        let head = head_of("@@type{\"Extras\", href: \"evil\"}");
        to_attributes(&head, &ExtrasOptions::default())
    };

    let mut construct = Attrs::default();
    construct.push("title", ExtrasAttr::Value(AttrValue::Str("Head".into())));

    let merged = construct.merge_with(extras, &["href"]);
    assert_eq!(merged.value("title"), Some(&AttrValue::Str("Head".into())));
    assert_eq!(
        merged.warnings,
        vec![
            ExtrasWarning::Overridden {
                key: "title".to_string()
            },
            ExtrasWarning::OwnedKeyIgnored {
                key: "href".to_string()
            },
        ]
    );
}

/// The worked example of §5.1, in the §6.4 emission order.
#[test]
fn spec_worked_example() {
    let source = concat!(
        r#"@@type{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", "#,
        r#"bar: 12, isFoo: true, --style-var: "2rem", isBar}"#
    );
    let (items, warnings) = attrs_of(source);

    assert_eq!(warnings, vec![]);
    assert_eq!(
        items,
        vec![
            (
                "class".to_string(),
                ExtrasAttr::Value(AttrValue::Str("extra class".into()))
            ),
            (
                "id".to_string(),
                ExtrasAttr::Value(AttrValue::Str("id".into()))
            ),
            (
                "slug".to_string(),
                ExtrasAttr::Value(AttrValue::Str("slug-foo".into()))
            ),
            (
                "title".to_string(),
                ExtrasAttr::Value(AttrValue::Str("Title Foo".into()))
            ),
            (
                "foo".to_string(),
                ExtrasAttr::Value(AttrValue::Str("bar".into()))
            ),
            ("bar".to_string(), ExtrasAttr::Value(AttrValue::Int(12))),
            (
                "isFoo".to_string(),
                ExtrasAttr::Value(AttrValue::Bool(true))
            ),
            (
                "style".to_string(),
                ExtrasAttr::Value(AttrValue::Str("--style-var: 2rem".into()))
            ),
            ("isBar".to_string(), ExtrasAttr::Flag),
        ]
    );
}

#[test]
fn parses_directive_heads() {
    let DirectiveMatch::Head { head, rest } =
        parse_directive_head("::note[slug-a](\"Title\") body")
    else {
        panic!("expected a directive head");
    };
    assert_eq!(head.sigil, DirectiveSigil::Colon);
    assert_eq!(head.count, 2);
    assert_eq!(head.type_marker.as_deref(), Some("note"));
    assert_eq!(head.bracket.as_deref(), Some("slug-a"));
    assert_eq!(head.parentheses.as_deref(), Some("Title"));
    assert_eq!(rest, " body");

    // A bare fence is the close form (§10.3).
    let DirectiveMatch::Head { head, rest } = parse_directive_head("===") else {
        panic!("expected a directive head");
    };
    assert_eq!(head.sigil, DirectiveSigil::Equal);
    assert_eq!(head.count, 3);
    assert_eq!(head.type_marker, None);
    assert_eq!(rest, "");

    // A single sigil is prose, eight is literal text.
    assert!(matches!(
        parse_directive_head(": not a head"),
        DirectiveMatch::Absent { .. }
    ));
    assert!(matches!(
        parse_directive_head("plain"),
        DirectiveMatch::Absent { .. }
    ));
    assert!(matches!(
        parse_directive_head("========"),
        DirectiveMatch::Absent { .. }
    ));
    // An opening fence needs a type name; brackets need a closing `]`.
    assert!(matches!(
        parse_directive_head("==5"),
        DirectiveMatch::Malformed { .. }
    ));
    assert!(matches!(
        parse_directive_head("::note[unclosed"),
        DirectiveMatch::Malformed { .. }
    ));
}

#[test]
fn scans_the_type_marker_without_parsing_the_body() {
    assert_eq!(
        parse_type_marker("@@headingX{`a`} tail").map(|(name, rest)| (name, rest.to_string())),
        Some(("headingX".to_string(), "{`a`} tail".to_string()))
    );
    assert_eq!(parse_type_marker("@@{.a}"), None);
    assert_eq!(parse_type_marker("@@123"), None);
}

// ------------------------------------------------- positional groups (§6.1)

/// §9.1/§6.1: `@@type[…](…){…}` — the groups sit between the type and the
/// body, each part touching the previous one.
#[test]
fn heads_carry_positional_groups() {
    let head = head_of("@@aside[intro](\"A title\"){.box, #a1}");
    assert_eq!(head.type_marker.as_deref(), Some("aside"));
    assert_eq!(head.bracket.as_deref(), Some("intro"));
    assert_eq!(head.parentheses.as_deref(), Some("A title"));
    assert_eq!(head.items.len(), 2);

    // Each group is optional, in either combination, with or without a body.
    let head = head_of("@@aside[intro]{.box}");
    assert_eq!(head.bracket.as_deref(), Some("intro"));
    assert_eq!(head.parentheses, None);

    let head = head_of("@@aside(\"T\")");
    assert_eq!(head.bracket, None);
    assert_eq!(head.parentheses.as_deref(), Some("T"));
    assert!(head.items.is_empty());

    // §9.1: the untyped decorator `@@[…]` — the groups alone carry the head.
    let head = head_of("@@[intro](\"T\"){.box}");
    assert_eq!(head.type_marker, None);
    assert_eq!(head.bracket.as_deref(), Some("intro"));
    assert_eq!(head.parentheses.as_deref(), Some("T"));

    // Groups without a type or body are still a head (`@@[…]` / `@@(…)`).
    let head = head_of("@@[intro]");
    assert_eq!(head.bracket.as_deref(), Some("intro"));
    assert!(head.items.is_empty());
}

/// §4.1 adjacency applies to the groups too.
#[test]
fn groups_must_be_adjacent() {
    // A space before `[` ends the head; the rest stays literal.
    let ExtrasMatch::Head { head, rest } = parse_extras("@@aside [intro]") else {
        panic!("expected a type-only head");
    };
    assert_eq!(head.type_marker.as_deref(), Some("aside"));
    assert_eq!(head.bracket, None);
    assert_eq!(rest, " [intro]");

    // `@@type {…}` is still not a head, groups or not.
    use ExtrasError::*;
    assert_eq!(literal_error("@@aside[intro] {.box}"), InvalidHead);
    // An unterminated group falls back to literal text (§4.3).
    assert_eq!(literal_error("@@aside[unclosed{.box}"), UnterminatedHead);
    assert_eq!(literal_error("@@aside(\"unclosed{.box}"), UnterminatedHead);
    // A bare `@@` is still not a head.
    assert_eq!(literal_error("@@ plain"), InvalidHead);
}

/// §6.1/§6.4: the groups map through `bracket_key` / `parentheses_key`, sit
/// with the positional keys, and win a collision with a body item (§6.2).
#[test]
fn groups_map_through_the_positional_keys() {
    let head = head_of("@@aside[intro](\"T\"){.c, `body-slug`, slug: \"p\"}");
    let attrs = to_attributes(&head, &ExtrasOptions::default());
    let keys: Vec<String> = attrs.items.iter().map(|(key, _)| key.clone()).collect();
    // class, then the group keys, and the colliding body items are dropped.
    assert_eq!(keys, vec!["class", "slug", "title"]);
    assert_eq!(
        attrs.value("slug").map(AttrValue::literal),
        Some("intro".to_string())
    );
    assert_eq!(
        attrs.value("title").map(AttrValue::literal),
        Some("T".to_string())
    );
    assert_eq!(
        attrs
            .warnings
            .iter()
            .filter(|warning| matches!(warning, ExtrasWarning::Overridden { .. }))
            .count(),
        2,
        "{:?}",
        attrs.warnings
    );

    // The keys follow the component config like every other positional.
    let head = head_of("@@note[n](\"T\")");
    let options = ExtrasOptions {
        bracket_key: "level".into(),
        parentheses_key: "summary".into(),
        ..ExtrasOptions::default()
    };
    let attrs = to_attributes(&head, &options);
    assert_eq!(
        attrs.value("level").map(AttrValue::literal),
        Some("n".to_string())
    );
    assert_eq!(
        attrs.value("summary").map(AttrValue::literal),
        Some("T".to_string())
    );

    // Without a body the groups still emit, in §6.4 order.
    let head = head_of("@@[s](\"t\")");
    let attrs = to_attributes(&head, &ExtrasOptions::default());
    let keys: Vec<String> = attrs.items.iter().map(|(key, _)| key.clone()).collect();
    assert_eq!(keys, vec!["slug", "title"]);
}
