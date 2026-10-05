use std::collections::BTreeMap;

use pendon_core::{Event, NodeKind, Severity};
use pendon_extra::{
    legacy_extras_warning, parse_attrs, scan_extras_chars, to_attributes, ExtraAttrs, ExtrasAttr,
    ExtrasHead, ExtrasOptions,
};
use pendon_renderer_solid::{ComponentSet, ComponentTemplate, ImportEntry, SolidRenderHints};

#[derive(Clone, Debug, Default)]
pub struct AnchorCustomNode {
    pub name: String,
    pub template: String,
    pub imports: Vec<ImportEntry>,
}

#[derive(Clone, Debug, Default)]
pub struct AnchorOptions {
    /// §11 component set of the `anchor` layer: typed entries plus at most one
    /// default, selected per instance by the `@@type{…}` marker (rule 3).
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
    let mut hints = SolidRenderHints::default();
    // §11 rule 3: every entry of the set answers instances of its own, so every
    // entry needs a template (typed entries are not shadowed by the default).
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

fn emit_text(text: &str, options: &AnchorOptions, out: &mut Vec<Event>) {
    let chars: Vec<char> = text.chars().collect();
    let mut cursor = 0usize;
    let mut normal = String::new();

    while cursor < chars.len() {
        let is_image_syntax = chars[cursor] == '[' && cursor > 0 && chars[cursor - 1] == '!';

        if chars[cursor] == '[' && !is_image_syntax {
            if let Some(link) = parse_link(&chars, cursor) {
                let built =
                    build_attributes(&link.target, link.legacy.as_ref(), link.extras.as_ref());
                flush_text(&mut normal, out);

                // §14: the pre-§11 `[.class,#id]{key: value}` form still works but
                // is reported, so no config keeps it by accident.
                if link.legacy.is_some() {
                    out.push(legacy_extras_warning("anchor"));
                }
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

                emit_anchor(&link.label, built, options, out);
                cursor = link.end;
                continue;
            }
        }

        normal.push(chars[cursor]);
        cursor += 1;
    }

    flush_text(&mut normal, out);
}

/// A parsed `[label](target "title")` head plus whatever attributes are attached
/// to it: the §11 extras head and the deprecated `[.class,#id]{key: value}` form
/// (§7.2).
struct ParsedLink {
    end: usize,
    label: String,
    /// `url` and, after a `\0`, the optional head title.
    target: String,
    extras: Option<ExtrasHead>,
    legacy: Option<ExtraAttrs>,
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
    let mut end = close_target + 1;

    // Extra attributes must be directly attached without spaces. Accepts both
    // the `{key: val}` form and the `[.class,#id]{key: val}` form.
    let legacy = parse_link_extra_attrs(chars, &mut end);

    // §7.2: the `@@type{…}` extras head follows the legacy block when present.
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
        legacy,
    })
}

/// The attributes of one `<a>`: values in `BTreeMap` order (the emission order
/// the golden fixtures were built with), the bare flags of a §11 extras head,
/// the `^`/`~` conflict warning and the §13 warnings of the extras merge.
struct AnchorAttrs {
    values: BTreeMap<String, String>,
    flags: Vec<String>,
    /// The `@@type{…}` marker of the extras head, when present: the §11 routing
    /// key of the instance (§11 rule 3).
    type_marker: Option<String>,
    conflict: Option<String>,
    warnings: Vec<String>,
}

fn build_attributes(
    encoded_target: &str,
    legacy: Option<&ExtraAttrs>,
    extras: Option<&ExtrasHead>,
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
        if all_modifiers[i] == '^' {
            if explicit_target == Some("_self") {
                conflict =
                    Some("[anchor] both '^' and '~' were provided; last modifier wins".to_string());
            }
            target = Some("_blank".to_string());
            explicit_target = Some("_blank");
            add_rel(&mut rel, "noopener");
        } else if all_modifiers[i] == '~' {
            if explicit_target == Some("_blank") {
                conflict =
                    Some("[anchor] both '^' and '~' were provided; last modifier wins".to_string());
            }
            target = Some("_self".to_string());
            explicit_target = Some("_self");
        } else if all_modifiers[i] == '!' {
            add_rel(&mut rel, "nofollow");
        } else if all_modifiers[i] == '$' {
            add_rel(&mut rel, "sponsored");
        } else if all_modifiers[i] == ';' && all_modifiers.get(i + 1) == Some(&';') {
            add_rel(&mut rel, "ugc");
            i += 1;
        } else if all_modifiers[i] == '-' && all_modifiers.get(i + 1) == Some(&'-') {
            add_rel(&mut rel, "noreferrer");
            i += 1;
        }
        i += 1;
    }

    if let Some(extra) = legacy {
        if let Some(id) = extra.id.as_ref() {
            attrs.insert("id".to_string(), id.clone());
        }
        if !extra.classes.is_empty() {
            attrs.insert("class".to_string(), extra.classes.join(" "));
        }
        for (key, value) in &extra.properties {
            if key == "rel" {
                for token in value.split_whitespace() {
                    add_rel(&mut rel, token);
                }
            } else if key == "target" {
                target = Some(value.clone());
            } else {
                attrs.insert(key.clone(), value.clone());
            }
        }
    }

    // §7.2 / §6.2: the extras head attaches to the `<a>`; the construct head
    // (URL modifiers and the `("title")` head) wins, `href` is never overridable.
    let mut flags = Vec::new();
    let mut warnings = Vec::new();
    if let Some(head) = extras {
        let parsed = to_attributes(head, &ExtrasOptions::default());
        for warning in &parsed.warnings {
            warnings.push(pendon_extra::warning_message(warning));
        }
        let positional_id = parsed.value("slug").map(|value| value.literal());
        for (key, value) in &parsed.items {
            if key == "slug" {
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
        type_marker,
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
        } else {
            break;
        };
        base.truncate(base.len() - length);
        modifiers.insert_str(0, &value);
    }
    (base, modifiers)
}

fn emit_anchor(label: &str, attrs: AnchorAttrs, options: &AnchorOptions, out: &mut Vec<Event>) {
    // §11 rule 3: the marker picks the component; an unmatched marker falls back
    // to the layer default, and a layer without a default to the `<a>` element.
    let custom = options.custom.select(attrs.type_marker.as_deref());
    let node = custom
        .map(|custom| NodeKind::Custom(custom.name.clone()))
        .unwrap_or(NodeKind::Link);
    out.push(Event::StartNode(node.clone()));
    if let Some(custom) = custom {
        out.push(Event::Attribute {
            name: "name".to_string(),
            value: custom.name.clone(),
        });
    }
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

/// Parses an adjacent `[.class,#id]{key: val}` (or `{key: val}`) block after a
/// link target. Returns `None` when nothing valid is attached, preserving
/// surrounding text (e.g. a following markdown link).
fn parse_link_extra_attrs(chars: &[char], end: &mut usize) -> Option<ExtraAttrs> {
    let source: String = chars[*end..].iter().collect();
    let starts_with_block = source.starts_with('{')
        || (source.starts_with('[')
            && source
                .chars()
                .nth(1)
                .is_some_and(|character| character == '.' || character == '#'));
    if !starts_with_block {
        return None;
    }

    let parsed = parse_attrs(&source);
    let has_content = parsed.attrs.id.is_some()
        || !parsed.attrs.classes.is_empty()
        || !parsed.attrs.properties.is_empty();
    if !parsed.had_attrs || !has_content {
        return None;
    }

    let consumed = source.len().saturating_sub(parsed.rest.len());
    *end += source[..consumed].chars().count();
    Some(parsed.attrs)
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
    fn parses_bracket_and_brace_extra_attrs() {
        let mut out = Vec::new();
        emit_text(
            "[foo](/docs)[.bax,#rew]{zo: \"kong\"}",
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
        assert!(attrs.contains(&("id", "rew")));
        assert!(attrs.contains(&("zo", "kong")));

        let leftover: String = out
            .iter()
            .filter_map(|event| match event {
                Event::Text(text) => Some(text.as_str()),
                _ => None,
            })
            .collect();
        assert!(!leftover.contains("[.bax]"));
        assert!(!leftover.contains("kong"));
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
        assert!(out.iter().any(
            |event| matches!(event, Event::StartNode(NodeKind::Custom(name)) if name == "Anchor")
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
            |event| matches!(event, Event::StartNode(NodeKind::Custom(name)) if name == "AnchorAB")
        ));
        assert!(out.iter().any(
            |event| matches!(event, Event::Attribute { name, value } if name == "type" && value == "anchorA")
        ));

        let mut out = Vec::new();
        emit_text("[a](/a)@@undeclared{}", &options, &mut out);
        assert!(out.iter().any(
            |event| matches!(event, Event::StartNode(NodeKind::Custom(name)) if name == "AnchorDefault")
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
        assert!(hints
            .templates
            .iter()
            .any(|template| template.node_type == "AnchorA"));
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

    #[test]
    fn legacy_extras_form_is_reported() {
        let emitted = emit("[foo](/docs)[.bax,#rew]{zo: \"kong\"}");
        assert_eq!(value(&emitted, "class"), Some("bax"));
        assert_eq!(value(&emitted, "id"), Some("rew"));
        assert_eq!(value(&emitted, "zo"), Some("kong"));
        assert!(
            emitted
                .warnings
                .iter()
                .any(|w| w.contains("deprecated") && w.contains("@@type")),
            "{:?}",
            emitted.warnings
        );
    }
}
