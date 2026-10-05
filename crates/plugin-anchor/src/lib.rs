use std::collections::BTreeMap;

use pendon_core::{Event, NodeKind, Severity};
use pendon_extra::{parse_attrs, ExtraAttrs};
use pendon_renderer_solid::{ComponentTemplate, ImportEntry, SolidRenderHints};

#[derive(Clone, Debug, Default)]
pub struct AnchorCustomNode {
    pub name: String,
    pub template: String,
    pub imports: Vec<ImportEntry>,
}

#[derive(Clone, Debug)]
pub struct AnchorOptions {
    pub custom_node: Option<AnchorCustomNode>,
}

impl Default for AnchorOptions {
    fn default() -> Self {
        Self { custom_node: None }
    }
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
    let custom = options.custom_node.as_ref()?;
    let key = (custom.name.clone(), Some(custom.name.clone()));
    let mut hints = SolidRenderHints::default();
    hints.templates.push(ComponentTemplate {
        node_type: custom.name.clone(),
        node_name: Some(custom.name.clone()),
        template: custom.template.clone(),
    });
    hints.template_imports.insert(key, custom.imports.clone());
    Some(hints)
}

fn emit_text(text: &str, options: &AnchorOptions, out: &mut Vec<Event>) {
    let chars: Vec<char> = text.chars().collect();
    let mut cursor = 0usize;
    let mut normal = String::new();

    while cursor < chars.len() {
        let is_image_syntax = chars[cursor] == '[' && cursor > 0 && chars[cursor - 1] == '!';

        if chars[cursor] == '[' && !is_image_syntax {
            if let Some((end, label, raw_target, extra)) = parse_link(&chars, cursor) {
                let (attrs, warning) = build_attributes(&raw_target, extra);
                flush_text(&mut normal, out);

                if let Some(message) = warning {
                    out.push(Event::Diagnostic {
                        severity: Severity::Warning,
                        message,
                        span: None,
                    });
                }

                emit_anchor(&label, attrs, options, out);
                cursor = end;
                continue;
            }
        }

        normal.push(chars[cursor]);
        cursor += 1;
    }

    flush_text(&mut normal, out);
}

fn parse_link(chars: &[char], start: usize) -> Option<(usize, String, String, Option<ExtraAttrs>)> {
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
    let extra = parse_link_extra_attrs(chars, &mut end);

    let target = match title {
        Some(t) => format!("{}\u{0}{}", raw_target, t),
        None => raw_target,
    };

    Some((end, label, target, extra))
}

fn build_attributes(
    encoded_target: &str,
    extra: Option<ExtraAttrs>,
) -> (BTreeMap<String, String>, Option<String>) {
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

    if let Some(extra) = extra {
        if let Some(id) = extra.id {
            attrs.insert("id".to_string(), id);
        }
        if !extra.classes.is_empty() {
            attrs.insert("class".to_string(), extra.classes.join(" "));
        }
        for (key, value) in extra.properties {
            if key == "rel" {
                for token in value.split_whitespace() {
                    add_rel(&mut rel, token);
                }
            } else if key == "target" {
                target = Some(value);
            } else {
                attrs.insert(key, value);
            }
        }
    }

    if let Some(target) = target {
        attrs.insert("target".to_string(), target);
    }

    if !rel.is_empty() {
        attrs.insert("rel".to_string(), rel.join(" "));
    }

    (attrs, conflict)
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

fn emit_anchor(
    label: &str,
    attrs: BTreeMap<String, String>,
    options: &AnchorOptions,
    out: &mut Vec<Event>,
) {
    let node = options
        .custom_node
        .as_ref()
        .map(|custom| NodeKind::Custom(custom.name.clone()))
        .unwrap_or(NodeKind::Link);
    out.push(Event::StartNode(node.clone()));
    if let Some(custom) = options.custom_node.as_ref() {
        out.push(Event::Attribute {
            name: "name".to_string(),
            value: custom.name.clone(),
        });
    }
    for (name, value) in attrs {
        out.push(Event::Attribute { name, value });
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
            custom_node: Some(AnchorCustomNode {
                name: "Anchor".into(),
                template: "<Anchor>{children}</Anchor>".into(),
                imports: Vec::new(),
            }),
        };
        let mut out = Vec::new();
        emit_text("[foo](/docs)", &options, &mut out);
        assert!(out.iter().any(
            |event| matches!(event, Event::StartNode(NodeKind::Custom(name)) if name == "Anchor")
        ));
    }
}
