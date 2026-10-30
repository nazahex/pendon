use std::collections::BTreeMap;

use pendon_core::{Event, NodeKind, Severity};
use pendon_extra::{
    append_class, issue_event, modifier_contributions, parse_extras, parse_slots, to_attributes,
    ElementHead, ExtrasAttr, ExtrasHead, ExtrasIssue, ExtrasMatch, FieldConfig, FieldName,
    ModifierConfig, Preset, Slot, SlotContract, SlotError, SlotKind, SlotMatch, SlotPair, SlotPos,
    SlotTag, SpecsKey, SpecsResolver,
};
use pendon_renderer_solid::{layer_templates, ComponentSet, ImportEntry, SolidRenderHints};

// --- Slot head (§7.2, §6.1) ---

/// The tag that claims this element's head (§5.3, D-11).
///
/// A slot that stands alone needs its tag, and the anchor is exactly the element
/// that used to own the untagged `[text](url)` (§14): the bracket and paren now
/// belong to whoever contracts them, so without `@` the text is not ours and
/// goes on to `plugin-markdown`.
const TAG: char = '@';

/// The one preset name the link does not write as an attribute: `bracket.bare`
/// is the text the `<a>` wraps (§6.5).
const INNER_TEXT: &str = "inner_text";

/// Every position anchor claims.
///
/// A head the element claims is read **whole**: a content the preset does not
/// name is not dropped, it becomes a bare field (`bracket.quote`, `paren.tick`,
/// …) when the preset is in play (D-7).
const SLOTS: &[SlotPos] = &[
    (Slot::Bracket, SlotKind::Bare),
    (Slot::Bracket, SlotKind::Quote),
    (Slot::Bracket, SlotKind::Tick),
    (Slot::Paren, SlotKind::Bare),
    (Slot::Paren, SlotKind::Quote),
    (Slot::Paren, SlotKind::Tick),
];

/// §7.2's preset assignment, from
/// `@[inner_text "bracket quote" `bracket tick`](href "title" `paren tick`)`.
///
/// Any `field.<slot>.<kind>` key in the matched `custom` entry voids the whole
/// table for that entry (D-7); the values then stay under their slot-qualified
/// bare names. A `field.specs.*` / `field.override.*` key does not void it.
const PRESET: &[Preset] = &[
    Preset::new(Slot::Bracket, SlotKind::Bare, FieldName::new("inner_text")),
    Preset::new(Slot::Paren, SlotKind::Bare, FieldName::new("href")),
    Preset::new(Slot::Paren, SlotKind::Quote, FieldName::new("title")),
];

/// D-12 rule 5: the fields `<a>` writes itself, so a head item that spells one
/// of them is an `Error` (`field.override.<k>` publishes it under another name).
const OWNED: &[FieldName] = &[FieldName::new("href")];

/// The tag that claims anchor's head (D-11), the positions it claims, and where
/// each lands when the user configures nothing (§6.1, §6.5, D-7).
pub const HEAD: ElementHead = ElementHead::new(
    SlotContract {
        accepted: SLOTS,
        tag: SlotTag::Anchor,
    },
    PRESET,
)
.with_owned(OWNED);

/// §11: the layer `[[task.anchor.custom]]` declares.
const ANCHOR_LAYER: &str = "anchor";

/// The layer key `[[task.anchor.custom]]` addresses.
pub fn primary_layer() -> &'static str {
    ANCHOR_LAYER
}

#[derive(Clone, Debug, Default)]
pub struct AnchorCustomNode {
    pub name: String,
    pub template: String,
    pub imports: Vec<ImportEntry>,
}

#[derive(Clone, Debug, Default)]
pub struct AnchorOptions {
    /// §11 component set of the `anchor` layer: typed entries plus at most one
    /// default, resolved for the whole document by the renderer against the
    /// `@@type{…}` markers it occurs (rule 3, ADR-0008 `D-3a`).
    pub custom: ComponentSet<AnchorCustomNode>,
}

pub fn process(events: &[Event], options: &AnchorOptions) -> Vec<Event> {
    let mut out = Vec::with_capacity(events.len());
    let mut excluded = 0usize;
    for event in events {
        match event {
            Event::StartNode(kind) => {
                if matches!(
                    kind,
                    NodeKind::CodeFence
                        | NodeKind::InlineCode
                        | NodeKind::HtmlBlock
                        | NodeKind::HtmlInline
                ) {
                    excluded += 1;
                }
                out.push(event.clone());
            }
            Event::EndNode(kind) => {
                if matches!(
                    kind,
                    NodeKind::CodeFence
                        | NodeKind::InlineCode
                        | NodeKind::HtmlBlock
                        | NodeKind::HtmlInline
                ) {
                    excluded = excluded.saturating_sub(1);
                }
                out.push(event.clone());
            }
            Event::Text(text) if excluded == 0 => emit_text(text, options, &mut out),
            _ => out.push(event.clone()),
        }
    }
    out
}

pub fn solid_hints(options: &AnchorOptions) -> Option<SolidRenderHints> {
    if options.custom.is_empty() {
        return None;
    }
    // §11 rule 3 / D-3a: one template per entry, tagged with its layer; the
    // renderer decides per document which entry answers which `type`.
    Some(SolidRenderHints {
        templates: layer_templates(ANCHOR_LAYER, &options.custom, |custom| {
            (custom.template.clone(), custom.imports.clone())
        }),
        ..SolidRenderHints::default()
    })
}

fn emit_text(text: &str, options: &AnchorOptions, out: &mut Vec<Event>) {
    let mut cursor = 0usize;
    let mut normal = String::new();

    while cursor < text.len() {
        let rest = &text[cursor..];
        match read_head(rest, options) {
            Read::Anchor { parts, consumed } => {
                flush_text(&mut normal, out);
                for issue in &parts.issues {
                    out.push(issue_event("anchor", issue));
                }
                emit_anchor(parts, out);
                cursor += consumed;
            }
            // §4.3 / D-2: slot-shaped but illegal — an `Error`, and the text stays
            // exactly as it was written.
            Read::Malformed(error) => {
                out.push(Event::Diagnostic {
                    severity: Severity::Error,
                    message: format!("[anchor] {error}"),
                    span: None,
                });
                normal.push(TAG);
                cursor += TAG.len_utf8();
            }
            Read::Literal => {
                let ch = rest.chars().next().expect("cursor is in bounds");
                normal.push(ch);
                cursor += ch.len_utf8();
            }
        }
    }

    flush_text(&mut normal, out);
}

/// What one `@` in the text turned out to be (§7.2).
enum Read {
    /// Not an anchor head — `mail@example.com`, or a head another tag claims.
    Literal,
    /// Slot-shaped but illegal (§4.3, D-2): `Error` plus literal text.
    Malformed(SlotError),
    /// A complete link: its attributes and the bytes it consumed.
    Anchor {
        /// What the `<a>` carries.
        parts: AnchorParts,
        /// How many bytes of the text the head occupied, extras included.
        consumed: usize,
    },
}

/// Reads an anchor head at the start of `rest` — the tag, the two slots (§5.2–
/// §5.3) and the §11 extras head attached to the closing `)`.
fn read_head(rest: &str, options: &AnchorOptions) -> Read {
    let pair = match parse_slots(rest) {
        SlotMatch::Slots { pair, rest: after } if pair.tag == SlotTag::Anchor => {
            (pair, rest.len() - after.len())
        }
        // Only a **tagged** head is ours to complain about: an untagged malformed
        // bracket (`[foo "bar](/x)`) belongs to whoever else wants it (§4.3).
        SlotMatch::Malformed { error, .. } if rest.starts_with(TAG) => {
            return Read::Malformed(error)
        }
        // Untagged `[x](y)` is not ours (§14): the anchor claims no head it was
        // not tagged into, so `plugin-markdown` gets the text untouched.
        _ => return Read::Literal,
    };
    let (pair, mut consumed) = pair;

    // §4.1: the extras head must touch the `)`.
    let extras = match parse_extras(&rest[consumed..]) {
        ExtrasMatch::Head { head, rest: after } => {
            consumed = rest.len() - after.len();
            Some(head)
        }
        ExtrasMatch::Absent { .. } | ExtrasMatch::Malformed { .. } => None,
    };

    let specs = |target: Option<&str>| options.custom.specs_for(target);
    // §6.5 / D-7: the preset is voided by the **matched** entry, never by another
    // entry's `field.*`.
    let marker = extras.as_ref().and_then(|head| head.type_marker.as_deref());
    let matched = extras
        .as_ref()
        .and_then(|head| options.custom.field_for(head.type_marker.as_deref()));

    Read::Anchor {
        parts: build_attributes(
            &pair,
            extras.as_ref(),
            &specs,
            matched,
            options.custom.modifier_for(marker),
        ),
        consumed,
    }
}

/// What one `<a>` carries: the text it wraps, its attributes in `BTreeMap`
/// order (the emission order the golden fixtures were built with) and the bare
/// flags of a §11 extras head.
///
/// There is no `target` / `rel` field any more: automatic external-link
/// attributes and the URL-suffix modifiers that set them are gone (§7.2, §14),
/// so both are ordinary extras keys like any other.
#[derive(Debug, Default)]
struct AnchorParts {
    text: String,
    values: BTreeMap<String, String>,
    flags: Vec<String>,
    issues: Vec<ExtrasIssue>,
}

/// Turns the two slots and the extras head into the fields the `<a>` exposes
/// (§6.5 preset, §6.2 priority, §6.3 projection).
///
/// `field` is the **matched** entry's `field.*` (D-7): when it configures one,
/// every preset row is void for this element and each position keeps its
/// slot-qualified name, which the link writes as an attribute rather than
/// dropping.
fn build_attributes(
    pair: &SlotPair,
    extras: Option<&ExtrasHead>,
    specs: SpecsResolver,
    field: Option<&FieldConfig>,
    modifier: Option<&ModifierConfig>,
) -> AnchorParts {
    let mut parts = AnchorParts::default();
    let mut issues = Vec::new();

    // §6.5: `bracket.bare → inner_text`, `paren.bare → href`,
    // `paren.quote → title`; every other position is a bare field (D-7).
    let fields = HEAD.assign(pair, field).unwrap_or_else(|error| {
        // The entry's assignments contradict each other (one path is a prefix of
        // another), so they cannot all be published. A config clash is the
        // author's bug: say so and keep the preset instead of silently emitting
        // half a link. An empty config cannot clash — every path is a unit name.
        issues.push(ExtrasIssue::error(format!(
            "the matched entry's `field.*` assignments clash ({error}); the preset is kept"
        )));
        HEAD.assign(pair, None)
            .expect("an untouched preset cannot clash")
    });
    for (path, value) in fields.iter() {
        let name = path.dotted();
        let text = value.literal();
        if name == INNER_TEXT {
            parts.text = text;
        } else {
            parts.values.insert(name, text);
        }
    }

    if let Some(head) = extras {
        let specs = specs(head.type_marker.as_deref());
        let parsed = to_attributes(head, &specs);
        issues.extend(parsed.issues.iter().cloned());
        // §6.2/§6.3: the tick position feeds the id fallback **while it keeps its
        // default path**; `field.specs.tick = "x"` moves it to a field of the
        // author's own, which the anchor then does not read.
        let tick = specs.element_path(SpecsKey::Tick);
        let specs_id = tick
            .and_then(|path| parsed.value(path))
            .map(|value| value.literal());
        // §6.3: the anchor's own positions — an extras `"title"` is the link's
        // `title` (at the default path), and `class` / `id` / `style` are the
        // universal DOM fields. The extras slug is the id fallback below, never an
        // attribute of its own.
        for (key, value) in specs.dom_fields(&parsed.items, &[(SpecsKey::Quote, "title")]) {
            if tick == Some(key) {
                continue;
            }
            if HEAD.owns(key) {
                // D-12's built-in collision: `href` is written by the link, so a
                // `href:` extra is an `Error`, and `field.override.href` (D-4) is
                // the way to publish it under another name.
                issues.push(ExtrasIssue::owned_key(key));
                continue;
            }
            if parts.values.contains_key(key) {
                issues.push(ExtrasIssue::overridden(key));
                continue;
            }
            match value {
                // §6.3: a bare flag stays a bare attribute (`<a isFoo>`).
                ExtrasAttr::Flag => parts.flags.push(key.to_string()),
                ExtrasAttr::Value(value) => {
                    parts
                        .values
                        .insert(key.to_string(), value.transport_for(key));
                }
            }
        }
        if let Some(id) = specs_id {
            if parts.values.contains_key("id") {
                issues.push(ExtrasIssue::warning(
                    "extras `slug` was dropped because the element already has an id",
                ));
            } else {
                parts.values.insert("id".to_string(), id);
            }
        }
        // §11 rule 3: the `@@type{…}` marker is the routing key of the instance. It
        // is carried to the node as a `type` attribute so a `{attrs.type}` template
        // can read it back; an explicit `type:` prop keeps its own value.
        if let Some(marker) = head.type_marker.as_deref() {
            parts
                .values
                .entry("type".to_string())
                .or_insert_with(|| marker.to_string());
        }
    }

    // §5.4 / D-9: the bracket's modifier run. It contributes last, so every field
    // the head or the extras already produced wins — a modifier is configuration,
    // the head is authored text.
    merge_modifiers(&mut parts, pair, modifier, &mut issues);

    parts.issues = issues;
    parts
}

/// Merges `pair`'s modifier run into the link's fields (§5.4, D-9).
///
/// `class` accumulates (§6.4); every other contributed field is dropped with a
/// warning when the element already carries it, and an unusable merge (D-9's
/// `Severity::Error` tier) drops the whole run.
fn merge_modifiers(
    parts: &mut AnchorParts,
    pair: &SlotPair,
    config: Option<&ModifierConfig>,
    issues: &mut Vec<ExtrasIssue>,
) {
    let fields = match modifier_contributions(config, pair) {
        Ok(fields) => fields,
        Err(error) => {
            issues.push(ExtrasIssue::error(error.message().to_string()));
            return;
        }
    };
    for field in fields {
        if field.is_flag() {
            if parts.values.contains_key(&field.name) {
                issues.push(ExtrasIssue::modifier_dropped(&field.name));
            } else {
                parts.flags.push(field.name);
            }
            continue;
        }
        let text = field.text();
        if field.name == "class" {
            let classes = append_class(parts.values.get("class").map(String::as_str), &text);
            parts.values.insert("class".to_string(), classes);
        } else if parts.values.contains_key(&field.name) {
            issues.push(ExtrasIssue::modifier_dropped(&field.name));
        } else {
            parts.values.insert(field.name, text);
        }
    }
}

fn emit_anchor(parts: AnchorParts, out: &mut Vec<Event>) {
    // §11 rule 3 / D-3a: the plugin no longer picks the component — it tags the
    // node with its layer and keeps the `type` marker, and the renderer decides
    // once per document which entry answers it.
    let node = NodeKind::Link;
    out.push(Event::StartNode(node.clone()));
    out.push(Event::Attribute {
        name: "__layer".to_string(),
        value: ANCHOR_LAYER.to_string(),
    });
    for (name, value) in parts.values {
        out.push(Event::Attribute { name, value });
    }
    // §6.3: a bare flag stays a bare attribute (`<a isFoo>`), it never becomes
    // `isFoo="isFoo"`.
    for name in parts.flags {
        out.push(Event::AttributeFlag { name });
    }
    out.push(Event::Text(parts.text));
    out.push(Event::EndNode(node));
}

fn flush_text(normal: &mut String, out: &mut Vec<Event>) {
    if !normal.is_empty() {
        out.push(Event::Text(std::mem::take(normal)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pendon_extra::{AttrValue, FieldConfig, Path};
    use pendon_renderer_solid::TypedComponent;

    /// §7.2's preset, position by position (S2-2.5). This is the element's
    /// contract as data: a change here changes the syntax, not the plumbing.
    #[test]
    fn the_head_declares_the_spec_preset() {
        assert_eq!(HEAD.contract.tag, SlotTag::Anchor);
        assert!(
            HEAD.is_consistent(),
            "a preset must sit on a claimed position"
        );
        // Every position of the head is claimed; the preset names three of them.
        for slot in Slot::ALL {
            for kind in SlotKind::ALL {
                assert!(
                    HEAD.contract.accepts(slot, kind),
                    "anchor drops {}.{}",
                    slot.name(),
                    kind.name()
                );
            }
        }
        let got: Vec<String> = HEAD
            .presets()
            .iter()
            .map(|preset| {
                format!(
                    "{}.{} -> {}",
                    preset.slot.name(),
                    preset.kind.name(),
                    preset.name
                )
            })
            .collect();
        assert_eq!(
            got,
            [
                "bracket.bare -> inner_text",
                "paren.bare -> href",
                "paren.quote -> title",
            ]
        );
        // Everything else is a bare field, never a preset (D-7).
        assert_eq!(HEAD.preset_for(Slot::Bracket, SlotKind::Quote), None);
        assert_eq!(HEAD.preset_for(Slot::Paren, SlotKind::Tick), None);
    }

    /// §7.2 / §14 (D-16): external-link detection is **gone**. An absolute URL is
    /// an ordinary `href` — no `target`, no `rel`, no classification — and the old
    /// suffix modifiers are just characters in the URL now.
    #[test]
    fn external_urls_get_no_automatic_target_or_rel() {
        for url in [
            "https://example.com",
            "//cdn.example.com/x",
            "www.example.com",
            "/docs/bar",
        ] {
            let emitted = emit(&format!("@[foo]({url})"));
            assert_eq!(value(&emitted, "href"), Some(url));
            assert!(
                value(&emitted, "target").is_none(),
                "{url}: {:?}",
                emitted.attrs
            );
            assert!(
                value(&emitted, "rel").is_none(),
                "{url}: {:?}",
                emitted.attrs
            );
            assert!(
                emitted.diagnostics.is_empty(),
                "{url}: {:?}",
                emitted.diagnostics
            );
        }

        // `^`, `~`, `!`, `$`, `;;`, `--` are no longer syntax: they stay in the URL
        // (a `rel:` / `target:` extra is the only way to set those attributes).
        let emitted = emit("@[foo](https://example.com/bar!?~$)");
        assert_eq!(value(&emitted, "href"), Some("https://example.com/bar!?~$"));
        assert!(value(&emitted, "rel").is_none());
        assert!(value(&emitted, "target").is_none());
        assert!(emitted.diagnostics.is_empty(), "{:?}", emitted.diagnostics);
    }

    /// §5.3 / §14: the slot tag is what claims a head, so the untagged
    /// `[text](url)` this element used to own is not an anchor any more — it stays
    /// text for `plugin-markdown`, with no diagnostic.
    #[test]
    fn an_untagged_slot_is_not_an_anchor() {
        let emitted = emit("[foo](/docs)@@anchor{.hero}");
        assert!(emitted.attrs.is_empty(), "{:?}", emitted.attrs);
        assert_eq!(emitted.text, "[foo](/docs)@@anchor{.hero}");
        assert!(emitted.diagnostics.is_empty(), "{:?}", emitted.diagnostics);
    }

    #[test]
    fn parses_an_adjacent_extras_head() {
        let mut out = Vec::new();
        emit_text(
            "@[foo](/docs){zo: \"kong\", .bax}",
            &AnchorOptions::default(),
            &mut out,
        );
        let attrs: Vec<_> = out
            .iter()
            .filter_map(|event| match event {
                Event::Attribute { name, value } => Some((name.as_str(), value.as_str())),
                _ => None,
            })
            .collect();
        assert!(attrs.contains(&("class", "bax")));
        assert!(attrs.contains(&("zo", "kong")));

        let leftover: String = out
            .iter()
            .filter_map(|event| match event {
                Event::Text(text) => Some(text.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(leftover, "foo");
    }

    #[test]
    fn does_not_swallow_following_link() {
        let mut out = Vec::new();
        emit_text("@[a](/a)@[b](/b)", &AnchorOptions::default(), &mut out);
        let links = out
            .iter()
            .filter(|event| matches!(event, Event::StartNode(NodeKind::Link)))
            .count();
        assert_eq!(links, 2);
    }

    #[test]
    fn creates_custom_node() {
        let options = AnchorOptions {
            custom: ComponentSet::from_entries([TypedComponent::default_component(
                AnchorCustomNode {
                    name: "Anchor".into(),
                    template: "<Anchor>{children}</Anchor>".into(),
                    imports: Vec::new(),
                },
            )]),
        };
        let mut out = Vec::new();
        emit_text("@[foo](/docs)", &options, &mut out);
        // D-3a: the node keeps the built-in kind and names its layer; the
        // renderer, not the plugin, picks the component.
        assert!(out
            .iter()
            .any(|event| matches!(event, Event::StartNode(NodeKind::Link))));
        assert!(out.iter().any(
            |event| matches!(event, Event::Attribute { name, value } if name == "__layer" && value == "anchor")
        ));
    }

    /// §11 rule 3: the marker routes to its own component, an unclaimed marker
    /// to the layer default, and a marker without either to the `<a>` element —
    /// which still carries the marker as its `type` attribute.
    #[test]
    fn type_marker_routes_between_typed_entries_and_the_default() {
        let options = AnchorOptions {
            custom: ComponentSet::from_entries([
                TypedComponent::typed(
                    ["anchorA", "anchorB"],
                    AnchorCustomNode {
                        name: "AnchorAB".into(),
                        template: "<AnchorAB>{children}</AnchorAB>".into(),
                        imports: Vec::new(),
                    },
                ),
                TypedComponent::default_component(AnchorCustomNode {
                    name: "AnchorDefault".into(),
                    template: "<AnchorDefault>{children}</AnchorDefault>".into(),
                    imports: Vec::new(),
                }),
            ]),
        };

        let mut out = Vec::new();
        emit_text("@[a](/a)@@anchorA{.hero}", &options, &mut out);
        assert!(out.iter().any(
            |event| matches!(event, Event::Attribute { name, value } if name == "type" && value == "anchorA")
        ));
        assert!(out.iter().any(
            |event| matches!(event, Event::Attribute { name, value } if name == "__layer" && value == "anchor")
        ));

        // An undeclared marker is still just a marker on the same layer: which
        // entry answers it (the default here, `None` with no default) is the
        // renderer's decision, taken once per document.
        let mut out = Vec::new();
        emit_text("@[a](/a)@@undeclared{}", &options, &mut out);
        assert!(out.iter().any(
            |event| matches!(event, Event::Attribute { name, value } if name == "type" && value == "undeclared")
        ));
    }

    /// Without a matching entry the node stays the `<a>` element, but the marker
    /// is not lost (§6.4): it is emitted as a `type` attribute.
    #[test]
    fn unclaimed_marker_keeps_the_builtin_element_and_its_type() {
        let options = AnchorOptions {
            custom: ComponentSet::from_entries([TypedComponent::typed(
                ["anchorA"],
                AnchorCustomNode {
                    name: "AnchorA".into(),
                    template: "<AnchorA>{children}</AnchorA>".into(),
                    imports: Vec::new(),
                },
            )]),
        };
        let mut out = Vec::new();
        emit_text("@[a](/a)@@anchorB{}", &options, &mut out);
        assert!(out
            .iter()
            .any(|event| matches!(event, Event::StartNode(NodeKind::Link))));
        assert!(out.iter().any(
            |event| matches!(event, Event::Attribute { name, value } if name == "type" && value == "anchorB")
        ));
    }

    /// Every entry of the set needs a template, typed or not.
    #[test]
    fn hints_cover_typed_entries_and_the_default() {
        let options = AnchorOptions {
            custom: ComponentSet::from_entries([
                TypedComponent::typed(
                    ["anchorA"],
                    AnchorCustomNode {
                        name: "AnchorA".into(),
                        template: "<AnchorA>{children}</AnchorA>".into(),
                        imports: Vec::new(),
                    },
                ),
                TypedComponent::default_component(AnchorCustomNode {
                    name: "AnchorDefault".into(),
                    template: "<AnchorDefault>{children}</AnchorDefault>".into(),
                    imports: Vec::new(),
                }),
            ]),
        };
        let hints = solid_hints(&options).expect("hints");
        assert_eq!(hints.templates.len(), 2);
        // Every template belongs to the `anchor` layer and keeps the selectors
        // its entry was declared with (empty = the layer default).
        assert!(hints
            .templates
            .iter()
            .all(|template| template.layer.as_deref() == Some("anchor")));
        assert!(hints.templates[0]
            .selectors
            .iter()
            .any(|selector| selector.literal() == Some("anchorA")));
        assert!(hints.templates[1].selectors.is_empty());
    }

    // --- §7.2 extras head -------------------------------------------------

    /// Collects the attributes, bare flags and §13 diagnostics of one emitted text.
    struct Emitted {
        attrs: Vec<(String, String)>,
        flags: Vec<String>,
        /// Every `Event::Diagnostic`, as `(severity, message)`.
        diagnostics: Vec<(Severity, String)>,
        text: String,
    }

    impl Emitted {
        /// True when some diagnostic's message contains `needle`.
        fn mentions(&self, needle: &str) -> bool {
            self.diagnostics
                .iter()
                .any(|(_, message)| message.contains(needle))
        }

        /// True when some diagnostic is an `Error` whose message contains `needle`.
        fn errors_on(&self, needle: &str) -> bool {
            self.diagnostics
                .iter()
                .any(|(severity, message)| *severity == Severity::Error && message.contains(needle))
        }
    }

    fn emit(text: &str) -> Emitted {
        let mut out = Vec::new();
        emit_text(text, &AnchorOptions::default(), &mut out);
        collect(out)
    }

    /// Collects the attributes, bare flags, diagnostics and text of one event run.
    fn collect(out: Vec<Event>) -> Emitted {
        let mut emitted = Emitted {
            attrs: Vec::new(),
            flags: Vec::new(),
            diagnostics: Vec::new(),
            text: String::new(),
        };
        for event in out {
            match event {
                Event::Attribute { name, value } => emitted.attrs.push((name, value)),
                Event::AttributeFlag { name } => emitted.flags.push(name),
                Event::Text(text) => emitted.text.push_str(&text),
                Event::Diagnostic {
                    severity, message, ..
                } => emitted.diagnostics.push((severity, message)),
                _ => {}
            }
        }
        emitted
    }

    fn value<'a>(emitted: &'a Emitted, name: &str) -> Option<&'a str> {
        emitted
            .attrs
            .iter()
            .find_map(|(key, value)| (key == name).then_some(value.as_str()))
    }

    // --- §5.4 slot modifiers (S2-6.2, D-9) --------------------------------

    /// Runs `text` through the emitter with `options`.
    fn emit_with(text: &str, options: &AnchorOptions) -> Emitted {
        let mut out = Vec::new();
        emit_text(text, options, &mut out);
        collect(out)
    }

    /// An anchor layer whose **default** entry carries `config`: every anchor
    /// without a `type` marker resolves to it, exactly as a real config would.
    fn modifier_options(config: ModifierConfig) -> AnchorOptions {
        AnchorOptions {
            custom: ComponentSet::from_entries([TypedComponent::default_component(
                AnchorCustomNode {
                    name: "Anchor".into(),
                    template: "<Anchor>{children}</Anchor>".into(),
                    imports: Vec::new(),
                },
            )
            .with_modifier(config)]),
        }
    }

    /// One contribution, as config writes it.
    fn field(name: &str, value: AttrValue) -> (Path, AttrValue) {
        (Path::from_dotted(name).expect("a valid path"), value)
    }

    /// The proposal's own symbol pair (D-9): `?` and `!` both write `rel` and
    /// `soo.baz`; `!` also adds a bare flag.
    fn proposal_modifiers() -> ModifierConfig {
        let mut config = ModifierConfig::new();
        config.define(
            '?',
            vec![
                field("rel", AttrValue::Str("nofollow".into())),
                field("soo.baz", AttrValue::Str("wax".into())),
                field("class", AttrValue::Str("external".into())),
            ],
        );
        config.define(
            '!',
            vec![
                field("rel", AttrValue::Str("noopener".into())),
                field("soo.baz", AttrValue::Str("wax".into())),
                field("isNew", AttrValue::Bool(true)),
            ],
        );
        config
    }

    /// The prefix a structured value travels with (§6.3 transport): the array is
    /// real JSON in the event stream, so `plugin-extract` reads an array.
    fn structured(json: &str) -> String {
        format!("{}{}", pendon_core::JSON_ATTR_PREFIX, json)
    }

    /// D-9's worked example, end to end: `|?!` writes both symbols' fields, and
    /// two contributions to one path become an array in written order.
    #[test]
    fn a_modifier_run_contributes_the_fields_its_symbols_define() {
        let options = modifier_options(proposal_modifiers());
        let emitted = emit_with("@[docs |?!](https://example.com)", &options);
        assert_eq!(value(&emitted, "href"), Some("https://example.com"));
        assert_eq!(
            value(&emitted, "rel"),
            Some(structured("[\"nofollow\",\"noopener\"]").as_str())
        );
        assert_eq!(
            value(&emitted, "soo.baz"),
            Some(structured("[\"wax\",\"wax\"]").as_str())
        );
        assert_eq!(value(&emitted, "class"), Some("external"));
        // A `true` contribution is a bare flag, never `isNew="isNew"` (§6.3).
        assert!(
            emitted.flags.contains(&"isNew".to_string()),
            "{:?}",
            emitted.attrs
        );
        assert!(emitted.diagnostics.is_empty(), "{:?}", emitted.diagnostics);
        // The run is consumed: it is configuration, not link text.
        assert_eq!(emitted.text, "docs");
    }

    /// A symbol with no config is legal and silent (S2-6.1): Pendon ships no
    /// modifier of its own, so `|?` means nothing until an entry defines it.
    #[test]
    fn an_unconfigured_symbol_contributes_nothing() {
        let emitted = emit_with(
            "@[docs |?xyz](https://example.com)",
            &modifier_options(ModifierConfig::new()),
        );
        assert_eq!(value(&emitted, "href"), Some("https://example.com"));
        assert!(value(&emitted, "rel").is_none(), "{:?}", emitted.attrs);
        assert!(emitted.flags.is_empty(), "{:?}", emitted.flags);
        assert!(emitted.diagnostics.is_empty(), "{:?}", emitted.diagnostics);
        assert_eq!(emitted.text, "docs");

        // Same text, no config at all — the shipped default.
        let emitted = emit("@[docs |?!](https://example.com)");
        assert!(value(&emitted, "rel").is_none(), "{:?}", emitted.attrs);
        assert!(emitted.diagnostics.is_empty(), "{:?}", emitted.diagnostics);
    }

    /// A modifier is configuration, the head is authored text: a field the
    /// element already carries wins, and the drop is reported.
    #[test]
    fn the_heads_own_field_wins_over_the_modifier() {
        let options = modifier_options(proposal_modifiers());
        let emitted = emit_with(
            "@[docs |?!](https://example.com)@@anchor{rel: \"ugc\"}",
            &options,
        );
        assert_eq!(value(&emitted, "rel"), Some("ugc"));
        assert!(
            emitted.mentions("from the modifier run"),
            "{:?}",
            emitted.diagnostics
        );
        // Only the losing field is dropped — the rest of the run still lands.
        assert_eq!(value(&emitted, "class"), Some("external"));
    }

    /// §6.4 / D-12: `class` is the one field that accumulates instead of
    /// clashing, so a modifier adds tokens to the head's classes.
    #[test]
    fn a_modifier_class_accumulates_on_the_heads_classes() {
        let options = modifier_options(proposal_modifiers());
        let emitted = emit_with(
            "@[docs |?](https://example.com)@@anchor{.hero, .external}",
            &options,
        );
        assert_eq!(value(&emitted, "class"), Some("hero external"));
        assert!(emitted.diagnostics.is_empty(), "{:?}", emitted.diagnostics);
    }

    /// D-9's `Severity::Error` tier: two contributions that cannot be nested
    /// (`rel` beside `rel.x`) drop the **whole** run — no partial projection.
    #[test]
    fn an_irreconcilable_merge_drops_the_whole_run() {
        let mut config = ModifierConfig::new();
        config.define('?', vec![field("rel", AttrValue::Str("nofollow".into()))]);
        config.define('!', vec![field("rel.x", AttrValue::Str("noopener".into()))]);
        let emitted = emit_with(
            "@[docs |?!](https://example.com)",
            &modifier_options(config),
        );
        assert!(value(&emitted, "rel").is_none(), "{:?}", emitted.attrs);
        assert!(value(&emitted, "rel.x").is_none(), "{:?}", emitted.attrs);
        assert!(
            emitted.errors_on("cannot contribute"),
            "{:?}",
            emitted.diagnostics
        );
        // The link itself is untouched — only the run is void.
        assert_eq!(value(&emitted, "href"), Some("https://example.com"));
        assert_eq!(emitted.text, "docs");
    }

    #[test]
    fn extras_attach_to_the_anchor_element() {
        let emitted = emit("@[foo](/docs)@@anchor{.hero,#main,rel: \"ugc\",isFoo}");
        assert_eq!(value(&emitted, "href"), Some("/docs"));
        assert_eq!(value(&emitted, "class"), Some("hero"));
        assert_eq!(value(&emitted, "id"), Some("main"));
        assert!(emitted.flags.contains(&"isFoo".to_string()));
        // §7.2: `rel:` extras merge with the modifier result.
        assert_eq!(value(&emitted, "rel"), Some("ugc"));
        // The head is consumed, no fragment of it is left in the label.
        assert_eq!(emitted.text, "foo");
    }

    #[test]
    fn head_title_wins_over_an_extras_title() {
        let emitted = emit("@[foo](/docs \"Head title\")@anchor{\"Extras title\"}");
        // `@anchor{…}` is not a head (single `@`), so it stays literal text.
        assert_eq!(value(&emitted, "title"), Some("Head title"));
        assert!(emitted.text.contains("@anchor"));

        let emitted = emit("@[foo](/docs \"Head title\")@@anchor{\"Extras title\"}");
        assert_eq!(value(&emitted, "title"), Some("Head title"));
        assert!(emitted.mentions("`title`"), "{:?}", emitted.diagnostics);
    }

    #[test]
    fn extras_slug_becomes_the_element_id() {
        let emitted = emit("@[foo](/docs)@@anchor{`slug-a`}");
        assert_eq!(value(&emitted, "id"), Some("slug-a"));
        assert!(value(&emitted, "slug").is_none());

        // §6.2: `#id` > head slug > extras slug.
        let emitted = emit("@[foo](/docs)@@anchor{`slug-a`, #explicit}");
        assert_eq!(value(&emitted, "id"), Some("explicit"));
        assert!(
            emitted.mentions("extras `slug`"),
            "{:?}",
            emitted.diagnostics
        );
    }

    /// §6.3 — the proposal's "Assign specs fields": an assigned specs position is
    /// a **public field of the author's own**. The element neither renames it to
    /// `class` / `id` / `style` / `title` nor reads it as its slug (§6.2), so all
    /// five positions land exactly where the entry put them, and the default
    /// paths they left stay empty.
    #[test]
    fn an_assigned_specs_path_is_a_field_of_its_own() {
        let mut field = FieldConfig::new();
        for (position, path) in [
            (SpecsKey::Tick, "slug"),
            (SpecsKey::Quote, "title"),
            (SpecsKey::Class, "foo.class"),
            (SpecsKey::Id, "bar.id"),
            (SpecsKey::Style, "foo.style"),
        ] {
            field
                .assign_specs(
                    position.key(),
                    Some(Path::from_dotted(path).expect("a dotted path")),
                )
                .expect("a known specs key");
        }
        let options = AnchorOptions {
            custom: ComponentSet::from_entries([TypedComponent::typed(
                ["reference"],
                AnchorCustomNode {
                    name: "Reference".into(),
                    template: "<Reference>{children}</Reference>".into(),
                    imports: Vec::new(),
                },
            )
            .with_field(field)]),
        };
        let mut out = Vec::new();
        emit_text(
            "@[foo](/x)@@reference{`john-foo`, \"Price Title\", .cta, #main, style: \"color: red\"}",
            &options,
            &mut out,
        );
        let emitted = collect(out);
        assert_eq!(value(&emitted, "href"), Some("/x"));
        assert_eq!(value(&emitted, "slug"), Some("john-foo"));
        assert_eq!(value(&emitted, "title"), Some("Price Title"));
        assert_eq!(value(&emitted, "foo.class"), Some("cta"));
        assert_eq!(value(&emitted, "bar.id"), Some("main"));
        assert_eq!(value(&emitted, "foo.style"), Some("color: red"));
        // Nothing was captured by the element: no DOM rename, no id fallback, and
        // no second copy under a default name (§6.2, §6.3 rule 2).
        for key in ["class", "id", "style", "specs.tick", "specs.quote"] {
            assert!(value(&emitted, key).is_none(), "{key}: {:?}", emitted.attrs);
        }

        // A move to a name the element has no use for is published just the same,
        // and the position's default path stays idle.
        let mut field = FieldConfig::new();
        field
            .assign_specs("quote", Some(Path::from_dotted("meta.title").unwrap()))
            .unwrap();
        let options = AnchorOptions {
            custom: ComponentSet::from_entries([TypedComponent::typed(
                ["reference"],
                AnchorCustomNode {
                    name: "Reference".into(),
                    template: "<Reference>{children}</Reference>".into(),
                    imports: Vec::new(),
                },
            )
            .with_field(field)]),
        };
        let mut out = Vec::new();
        emit_text("@[foo](/x)@@reference{\"Price Title\"}", &options, &mut out);
        let emitted = collect(out);
        assert_eq!(value(&emitted, "meta.title"), Some("Price Title"));
        assert!(value(&emitted, "title").is_none(), "{:?}", emitted.attrs);
    }

    /// D-12's built-in collision: an extras `k: v` naming a field the element
    /// writes itself is an **`Error`** that points at `field.override` (D-4), not
    /// a §6.2 `Warning`.
    #[test]
    fn href_stays_construct_owned() {
        let emitted = emit("@[foo](/docs)@@anchor{href: \"/evil\"}");
        assert_eq!(value(&emitted, "href"), Some("/docs"));
        assert!(emitted.errors_on("`href`"), "{:?}", emitted.diagnostics);
        assert!(
            emitted.mentions("field.override.href"),
            "{:?}",
            emitted.diagnostics
        );
    }

    /// §6.5: the preset assignment is `inner_text` / `href` / `title`, and every
    /// other position of the same head is a bare field under its slot-qualified
    /// name — nothing is dropped (D-7).
    #[test]
    fn all_six_positions_land_where_the_preset_says() {
        let emitted =
            emit("@[Inner \"Head quote\" `Head tick`](/docs \"Head title\" `Paren tick`)");
        assert_eq!(emitted.text, "Inner");
        assert_eq!(value(&emitted, "href"), Some("/docs"));
        assert_eq!(value(&emitted, "title"), Some("Head title"));
        assert_eq!(value(&emitted, "bracket.quote"), Some("Head quote"));
        assert_eq!(value(&emitted, "bracket.tick"), Some("Head tick"));
        assert_eq!(value(&emitted, "paren.tick"), Some("Paren tick"));
        assert!(
            value(&emitted, "inner_text").is_none(),
            "{:?}",
            emitted.attrs
        );
        assert!(emitted.diagnostics.is_empty(), "{:?}", emitted.diagnostics);
    }

    /// §6.1 / D-7: an assigned position is a field of the author's own. The link
    /// reads only the three names its preset declares, so a position that moved
    /// away keeps its value as an attribute and stops feeding `href` / the text.
    #[test]
    fn an_assigned_position_is_a_field_of_the_author() {
        let mut field = FieldConfig::new();
        field.assign_slot(
            Slot::Bracket,
            SlotKind::Bare,
            Some(Path::from_dotted("label").unwrap()),
        );
        field.assign_slot(
            Slot::Paren,
            SlotKind::Bare,
            Some(Path::from_dotted("url").unwrap()),
        );
        let options = AnchorOptions {
            custom: ComponentSet::from_entries([TypedComponent::typed(
                ["reference"],
                AnchorCustomNode {
                    name: "Reference".into(),
                    template: "<Reference>{children}</Reference>".into(),
                    imports: Vec::new(),
                },
            )
            .with_field(field)]),
        };
        let mut out = Vec::new();
        emit_text("@[Inner](/docs)@@reference{}", &options, &mut out);
        let emitted = collect(out);
        assert_eq!(value(&emitted, "label"), Some("Inner"));
        assert_eq!(value(&emitted, "url"), Some("/docs"));
        // The preset is void as a whole (D-7): neither `inner_text` nor `href` is
        // written, and the text the link wraps is empty.
        assert!(value(&emitted, "href").is_none(), "{:?}", emitted.attrs);
        assert_eq!(emitted.text, "");
    }

    /// A `@` that is not a head stays text, and a slot-shaped head that does not
    /// parse is an `Error` + literal text (§4.3, D-2).
    #[test]
    fn a_malformed_slot_is_an_error_and_stays_literal() {
        let emitted = emit("mail@example.com and @[foo \"bar](/x)");
        assert!(emitted.attrs.is_empty(), "{:?}", emitted.attrs);
        assert!(
            emitted.text.contains("mail@example.com"),
            "{}",
            emitted.text
        );
        assert!(
            emitted.text.contains("@[foo \"bar](/x)"),
            "{}",
            emitted.text
        );
        assert!(emitted.errors_on("slot"), "{:?}", emitted.diagnostics);
    }

    /// The tag is read wherever it starts a head — an anchor is inline by nature
    /// (§7.2), so it is not tied to the line head.
    #[test]
    fn the_tag_is_claimed_mid_text() {
        let emitted = emit("see @[the docs](/docs) for details");
        assert_eq!(value(&emitted, "href"), Some("/docs"));
        assert_eq!(emitted.text, "see the docs for details");
    }

    #[test]
    fn malformed_extras_stay_literal_text() {
        let emitted = emit("@[foo](/docs)@@anchor{`unterminated}");
        assert_eq!(value(&emitted, "href"), Some("/docs"));
        assert!(emitted.text.contains("@@anchor{"), "{}", emitted.text);
        assert!(emitted.diagnostics.is_empty(), "{:?}", emitted.diagnostics);
    }

    /// §14/§D3: the retired `[.class,#id]{key: value}` form is plain text.
    #[test]
    fn the_retired_legacy_form_is_literal_text() {
        let emitted = emit("@[foo](/docs)[.bax,#rew]{zo: \"kong\"}");
        assert_eq!(value(&emitted, "href"), Some("/docs"));
        assert!(value(&emitted, "class").is_none());
        assert!(value(&emitted, "id").is_none());
        assert!(emitted.diagnostics.is_empty(), "{:?}", emitted.diagnostics);
        assert!(
            emitted.text.contains("[.bax,#rew]{zo: \"kong\"}"),
            "{}",
            emitted.text
        );
    }

    /// §4.1: extras touch the `)`, and a type may stand alone.
    #[test]
    fn a_type_only_head_stops_before_the_punctuation() {
        let emitted = emit("@[t](/a)@@anchorA.");
        assert_eq!(value(&emitted, "href"), Some("/a"));
        assert_eq!(value(&emitted, "type"), Some("anchorA"));
        assert_eq!(emitted.text, "t.");

        // A bare head is the same as the `@@` spelling (§3).
        let emitted = emit("@[t](/a){.hero}");
        assert_eq!(value(&emitted, "href"), Some("/a"));
        assert_eq!(value(&emitted, "class"), Some("hero"));
        assert_eq!(emitted.text, "t");

        // `@@type {…}` is not a head (§4.1).
        let emitted = emit("@[t](/a)@@anchorA {.hero}");
        assert!(value(&emitted, "class").is_none());
        assert!(
            emitted.text.contains("@@anchorA {.hero}"),
            "{}",
            emitted.text
        );
    }
}
