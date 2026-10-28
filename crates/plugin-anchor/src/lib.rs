use std::collections::BTreeMap;

use pendon_core::{Event, NodeKind, Severity};
use pendon_extra::{
    scan_extras_chars, to_attributes, ElementHead, ExtrasAttr, ExtrasHead, FieldName,
    NamesResolver, Preset, Slot, SlotContract, SlotKind, SlotPos, SlotTag,
};
use pendon_renderer_solid::{layer_templates, ComponentSet, ImportEntry, SolidRenderHints};

// --- Slot head (§7.2, §6.1) ---

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
/// Any `field.*` key in the matched `custom` entry voids the whole table for that
/// entry (D-7); the values then stay under their slot-qualified bare names.
const PRESET: &[Preset] = &[
    Preset::new(Slot::Bracket, SlotKind::Bare, FieldName::new("inner_text")),
    Preset::new(Slot::Paren, SlotKind::Bare, FieldName::new("href")),
    Preset::new(Slot::Paren, SlotKind::Quote, FieldName::new("title")),
];

/// The tag that claims anchor's head (D-11), the positions it claims, and where
/// each lands when the user configures nothing (§6.1, §6.5, D-7).
pub const HEAD: ElementHead = ElementHead::new(
    SlotContract {
        accepted: SLOTS,
        tag: SlotTag::Anchor,
    },
    PRESET,
);

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
    let chars: Vec<char> = text.chars().collect();
    let mut cursor = 0usize;
    let mut normal = String::new();

    while cursor < chars.len() {
        let is_image_syntax = chars[cursor] == '[' && cursor > 0 && chars[cursor - 1] == '!';

        if chars[cursor] == '[' && !is_image_syntax {
            if let Some(link) = parse_link(&chars, cursor) {
                let keys = |target: Option<&str>| options.custom.names_for(target);
                let built = build_attributes(&link.target, link.extras.as_ref(), &keys);
                flush_text(&mut normal, out);

                if let Some(message) = built.conflict.as_ref() {
                    out.push(Event::Diagnostic {
                        severity: Severity::Warning,
                        message: message.clone(),
                        span: None,
                    });
                }
                for message in &built.warnings {
                    out.push(Event::Diagnostic {
                        severity: Severity::Warning,
                        message: format!("[anchor] {message}"),
                        span: None,
                    });
                }

                emit_anchor(&link.label, built, out);
                cursor = link.end;
                continue;
            }
        }

        normal.push(chars[cursor]);
        cursor += 1;
    }

    flush_text(&mut normal, out);
}

/// A parsed `[label](target "title")` head plus the §11 extras head attached to
/// it (§7.2).
struct ParsedLink {
    end: usize,
    label: String,
    /// `url` and, after a `\0`, the optional head title.
    target: String,
    extras: Option<ExtrasHead>,
}

fn parse_link(chars: &[char], start: usize) -> Option<ParsedLink> {
    if chars.get(start) != Some(&'[') {
        return None;
    }

    let close_label = find_char(chars, start + 1, ']')?;
    if chars.get(close_label + 1) != Some(&'(') {
        return None;
    }

    let close_target = find_matching_paren(chars, close_label + 2)?;
    let label: String = chars[start + 1..close_label].iter().collect();
    let raw_input: String = chars[close_label + 2..close_target].iter().collect();

    let (raw_target, title) = split_target_title(&raw_input);
    let end = close_target + 1;

    // §7.2/§4.1: extras must touch the closing `)`.
    let (extras, end) = match scan_extras_chars(chars, end) {
        Some((head, next)) => (Some(head), next),
        None => (None, end),
    };

    let target = match title {
        Some(t) => format!("{}\u{0}{}", raw_target, t),
        None => raw_target,
    };

    Some(ParsedLink {
        end,
        label,
        target,
        extras,
    })
}

/// The attributes of one `<a>`: values in `BTreeMap` order (the emission order
/// the golden fixtures were built with), the bare flags of a §11 extras head,
/// the `^`/`~` conflict warning and the §13 warnings of the extras merge.
struct AnchorAttrs {
    values: BTreeMap<String, String>,
    flags: Vec<String>,
    conflict: Option<String>,
    warnings: Vec<String>,
}

fn build_attributes(
    encoded_target: &str,
    extras: Option<&ExtrasHead>,
    keys: NamesResolver,
) -> AnchorAttrs {
    let (encoded_url, title) = encoded_target
        .split_once('\u{0}')
        .map(|(url, title)| (url, Some(title)))
        .unwrap_or((encoded_target, None));

    let (url, suffix_modifiers) = strip_url_modifiers(encoded_url);
    let all_modifiers: Vec<char> = suffix_modifiers.chars().collect();

    let mut attrs = BTreeMap::new();
    attrs.insert("href".to_string(), url.clone());

    if let Some(title) = title {
        attrs.insert("title".to_string(), title.to_string());
    }

    let external = is_external(&url);
    let mut target = if external {
        Some("_blank".to_string())
    } else {
        None
    };

    let mut rel = Vec::new();
    if external {
        add_rel(&mut rel, "noopener");
    }

    let mut conflict = None;
    let mut explicit_target: Option<&str> = None;
    let mut i = 0usize;

    while i < all_modifiers.len() {
        let step = match all_modifiers[i] {
            '^' => {
                if explicit_target == Some("_self") {
                    conflict = Some(
                        "[anchor] both '^' and '~' were provided; last modifier wins".to_string(),
                    );
                }
                target = Some("_blank".to_string());
                explicit_target = Some("_blank");
                add_rel(&mut rel, "noopener");
                1
            }
            '~' => {
                if explicit_target == Some("_blank") {
                    conflict = Some(
                        "[anchor] both '^' and '~' were provided; last modifier wins".to_string(),
                    );
                }
                target = Some("_self".to_string());
                explicit_target = Some("_self");
                1
            }
            '!' => {
                add_rel(&mut rel, "nofollow");
                1
            }
            '$' => {
                add_rel(&mut rel, "sponsored");
                1
            }
            ';' if all_modifiers.get(i + 1) == Some(&';') => {
                add_rel(&mut rel, "ugc");
                2
            }
            '-' if all_modifiers.get(i + 1) == Some(&'-') => {
                add_rel(&mut rel, "noreferrer");
                2
            }
            // Unknown modifier (e.g. `?` section terminator): stop processing.
            _ => break,
        };
        i += step;
    }

    // §7.2 / §6.2: the extras head attaches to the `<a>`; the construct head
    // (URL modifiers and the `("title")` head) wins, `href` is never overridable.
    let mut flags = Vec::new();
    let mut warnings = Vec::new();
    if let Some(head) = extras {
        let resolved = keys(head.type_marker.as_deref());
        let parsed = to_attributes(head, &resolved);
        for warning in &parsed.warnings {
            warnings.push(pendon_extra::warning_message(warning));
        }
        let positional_id = parsed.value(&resolved.tick).map(|value| value.literal());
        for (key, value) in &parsed.items {
            if key == &resolved.tick {
                continue;
            }
            if key == "href" {
                warnings.push(
                    "`href` is owned by the link; the extras value was ignored (§10.4)".to_string(),
                );
                continue;
            }
            let text = match value {
                ExtrasAttr::Flag => {
                    if attrs.contains_key(key) {
                        warnings.push(format!(
                            "`{key}` was dropped because the construct head already sets it"
                        ));
                    } else {
                        flags.push(key.clone());
                    }
                    continue;
                }
                ExtrasAttr::Value(value) => value.literal(),
            };
            if key == "rel" {
                for token in text.split_whitespace() {
                    add_rel(&mut rel, token);
                }
                continue;
            }
            if key == "target" {
                // §7.2: an explicit modifier wins, the external `_blank` does not.
                if explicit_target.is_none() {
                    target = Some(text);
                } else {
                    warnings.push(
                        "`target` was dropped because a URL modifier already set it".to_string(),
                    );
                }
                continue;
            }
            if attrs.contains_key(key) {
                warnings.push(format!(
                    "`{key}` was dropped because the construct head already sets it"
                ));
                continue;
            }
            attrs.insert(key.clone(), text);
        }
        if let Some(id) = positional_id {
            if attrs.contains_key("id") {
                warnings.push(
                    "extras `slug` was dropped because the element already has an id".to_string(),
                );
            } else {
                attrs.insert("id".to_string(), id);
            }
        }
    }

    if let Some(target) = target {
        attrs.insert("target".to_string(), target);
    }

    if !rel.is_empty() {
        attrs.insert("rel".to_string(), rel.join(" "));
    }

    // §11 rule 3: the `@@type{…}` marker is the routing key of the instance. It
    // is carried to the node as a `type` attribute so a `{attrs.type}` template
    // can read it back; an explicit `type:` prop keeps its own value.
    let type_marker = extras.and_then(|head| head.type_marker.clone());
    if let Some(marker) = &type_marker {
        attrs
            .entry("type".to_string())
            .or_insert_with(|| marker.clone());
    }

    AnchorAttrs {
        values: attrs,
        flags,
        conflict,
        warnings,
    }
}

fn strip_url_modifiers(url: &str) -> (String, String) {
    let mut base = url.to_string();
    let mut modifiers = String::new();
    loop {
        let (value, length) = if base.ends_with(";;") {
            (";;".to_string(), 2)
        } else if base.ends_with("--") {
            ("--".to_string(), 2)
        } else if base.ends_with('^')
            || base.ends_with('~')
            || base.ends_with('!')
            || base.ends_with('$')
        {
            (base[base.len() - 1..].to_string(), 1)
        } else if base.ends_with('?') {
            // `?` terminates the modifier section: strip it from the URL
            // and keep it in the modifier string so it acts as a
            // left-to-right processing terminator (§7.2).
            ("?".to_string(), 1)
        } else {
            break;
        };
        base.truncate(base.len() - length);
        modifiers.insert_str(0, &value);
    }
    (base, modifiers)
}

fn emit_anchor(label: &str, attrs: AnchorAttrs, out: &mut Vec<Event>) {
    // §11 rule 3 / D-3a: the plugin no longer picks the component — it tags the
    // node with its layer and keeps the `type` marker, and the renderer decides
    // once per document which entry answers it.
    let node = NodeKind::Link;
    out.push(Event::StartNode(node.clone()));
    out.push(Event::Attribute {
        name: "__layer".to_string(),
        value: ANCHOR_LAYER.to_string(),
    });
    for (name, value) in attrs.values {
        out.push(Event::Attribute { name, value });
    }
    // §6.3: a bare flag stays a bare attribute (`<a isFoo>`), it never becomes
    // `isFoo="isFoo"`.
    for name in attrs.flags {
        out.push(Event::AttributeFlag { name });
    }
    out.push(Event::Text(label.to_string()));
    out.push(Event::EndNode(node));
}

fn split_target_title(input: &str) -> (String, Option<String>) {
    let trimmed = input.trim();
    if let Some(quote) = trimmed.find('"') {
        let url = trimmed[..quote].trim();
        let title = trimmed[quote + 1..].trim_end_matches('"').to_string();
        (url.to_string(), Some(title))
    } else {
        (trimmed.to_string(), None)
    }
}

fn is_external(url: &str) -> bool {
    url.starts_with("http://")
        || url.starts_with("https://")
        || url.starts_with("//")
        || url.starts_with("www.")
        || (!url.starts_with('/') && url.contains('.'))
}

fn add_rel(rel: &mut Vec<String>, value: &str) {
    if !rel.iter().any(|item| item == value) {
        rel.push(value.to_string());
    }
}

fn find_char(chars: &[char], mut index: usize, wanted: char) -> Option<usize> {
    while index < chars.len() {
        if chars[index] == wanted {
            return Some(index);
        }
        index += 1;
    }
    None
}

fn find_matching_paren(chars: &[char], mut index: usize) -> Option<usize> {
    let mut depth = 0usize;
    while index < chars.len() {
        match chars[index] {
            '(' => depth += 1,
            ')' if depth == 0 => return Some(index),
            ')' => depth -= 1,
            _ => {}
        }
        index += 1;
    }
    None
}

fn flush_text(normal: &mut String, out: &mut Vec<Event>) {
    if !normal.is_empty() {
        out.push(Event::Text(std::mem::take(normal)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn adds_external_defaults_without_modifiers() {
        let mut out = Vec::new();
        emit_text(
            "[foo](https://example.com)",
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
        assert!(attrs.contains(&("target", "_blank")));
        assert!(attrs.contains(&("rel", "noopener")));
    }

    #[test]
    fn parses_an_adjacent_extras_head() {
        let mut out = Vec::new();
        emit_text(
            "[foo](/docs){zo: \"kong\", .bax}",
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
        emit_text("[a](/a)[b](/b)", &AnchorOptions::default(), &mut out);
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
        emit_text("[foo](/docs)", &options, &mut out);
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
        emit_text("[a](/a)@@anchorA{.hero}", &options, &mut out);
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
        emit_text("[a](/a)@@undeclared{}", &options, &mut out);
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
        emit_text("[a](/a)@@anchorB{}", &options, &mut out);
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

    /// Collects the attributes, bare flags and warnings of one emitted text.
    struct Emitted {
        attrs: Vec<(String, String)>,
        flags: Vec<String>,
        warnings: Vec<String>,
        text: String,
    }

    fn emit(text: &str) -> Emitted {
        let mut out = Vec::new();
        emit_text(text, &AnchorOptions::default(), &mut out);
        let mut emitted = Emitted {
            attrs: Vec::new(),
            flags: Vec::new(),
            warnings: Vec::new(),
            text: String::new(),
        };
        for event in out {
            match event {
                Event::Attribute { name, value } => emitted.attrs.push((name, value)),
                Event::AttributeFlag { name } => emitted.flags.push(name),
                Event::Text(text) => emitted.text.push_str(&text),
                Event::Diagnostic { message, .. } => emitted.warnings.push(message),
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

    #[test]
    fn extras_attach_to_the_anchor_element() {
        let emitted = emit("[foo](/docs)@@anchor{.hero,#main,rel: \"ugc\",isFoo}");
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
        let emitted = emit("[foo](/docs \"Head title\")@anchor{\"Extras title\"}");
        // `@anchor{…}` is not a head (single `@`), so it stays literal text.
        assert_eq!(value(&emitted, "title"), Some("Head title"));
        assert!(emitted.text.contains("@anchor"));

        let emitted = emit("[foo](/docs \"Head title\")@@anchor{\"Extras title\"}");
        assert_eq!(value(&emitted, "title"), Some("Head title"));
        assert!(
            emitted.warnings.iter().any(|w| w.contains("`title`")),
            "{:?}",
            emitted.warnings
        );
    }

    #[test]
    fn extras_slug_becomes_the_element_id() {
        let emitted = emit("[foo](/docs)@@anchor{`slug-a`}");
        assert_eq!(value(&emitted, "id"), Some("slug-a"));
        assert!(value(&emitted, "slug").is_none());

        // §6.2: `#id` > head slug > extras slug.
        let emitted = emit("[foo](/docs)@@anchor{`slug-a`, #explicit}");
        assert_eq!(value(&emitted, "id"), Some("explicit"));
        assert!(
            emitted.warnings.iter().any(|w| w.contains("extras `slug`")),
            "{:?}",
            emitted.warnings
        );
    }

    #[test]
    fn href_stays_construct_owned() {
        let emitted = emit("[foo](/docs)@@anchor{href: \"/evil\"}");
        assert_eq!(value(&emitted, "href"), Some("/docs"));
        assert!(
            emitted.warnings.iter().any(|w| w.contains("`href`")),
            "{:?}",
            emitted.warnings
        );
    }

    #[test]
    fn target_extras_do_not_replace_an_explicit_modifier() {
        // `^` is an explicit modifier: it wins.
        let emitted = emit("[foo](/docs^)@@anchor{target: \"_self\"}");
        assert_eq!(value(&emitted, "target"), Some("_blank"));
        assert!(emitted.warnings.iter().any(|w| w.contains("`target`")));

        // An external link only defaults to `_blank`, so extras may retarget it.
        let emitted = emit("[foo](https://example.com)@@anchor{target: \"_self\"}");
        assert_eq!(value(&emitted, "target"), Some("_self"));
    }

    #[test]
    fn malformed_extras_stay_literal_text() {
        let emitted = emit("[foo](/docs)@@anchor{`unterminated}");
        assert_eq!(value(&emitted, "href"), Some("/docs"));
        assert!(emitted.text.contains("@@anchor{"), "{}", emitted.text);
        assert!(emitted.warnings.is_empty(), "{:?}", emitted.warnings);
    }

    /// §14/§D3: the retired `[.class,#id]{key: value}` form is plain text.
    #[test]
    fn the_retired_legacy_form_is_literal_text() {
        let emitted = emit("[foo](/docs)[.bax,#rew]{zo: \"kong\"}");
        assert_eq!(value(&emitted, "href"), Some("/docs"));
        assert!(value(&emitted, "class").is_none());
        assert!(value(&emitted, "id").is_none());
        assert!(emitted.warnings.is_empty(), "{:?}", emitted.warnings);
        assert!(
            emitted.text.contains("[.bax,#rew]{zo: \"kong\"}"),
            "{}",
            emitted.text
        );
    }

    /// §4.1: extras touch the `)`, and a type may stand alone.
    #[test]
    fn a_type_only_head_stops_before_the_punctuation() {
        let emitted = emit("[t](/a)@@anchorA.");
        assert_eq!(value(&emitted, "href"), Some("/a"));
        assert_eq!(value(&emitted, "type"), Some("anchorA"));
        assert_eq!(emitted.text, "t.");

        // A bare head is the same as the `@@` spelling (§3).
        let emitted = emit("[t](/a){.hero}");
        assert_eq!(value(&emitted, "href"), Some("/a"));
        assert_eq!(value(&emitted, "class"), Some("hero"));
        assert_eq!(emitted.text, "t");

        // `@@type {…}` is not a head (§4.1).
        let emitted = emit("[t](/a)@@anchorA {.hero}");
        assert!(value(&emitted, "class").is_none());
        assert!(
            emitted.text.contains("@@anchorA {.hero}"),
            "{}",
            emitted.text
        );
    }
}
