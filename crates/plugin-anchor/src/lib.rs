use std::collections::BTreeMap;

use pendon_core::{Event, NodeKind, Severity};
use pendon_renderer_solid::{ComponentTemplate, ImportEntry, SolidRenderHints};

#[derive(Clone, Debug, Default)]
pub struct AnchorCustomNode {
    pub name: String,
    pub template: String,
    pub imports: Vec<AnchorImport>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnchorImport {
    pub module: String,
    pub default: Option<String>,
    pub names: Vec<String>,
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
    hints.template_imports.insert(
        key,
        custom
            .imports
            .iter()
            .map(|import| ImportEntry::Structured {
                module: import.module.clone(),
                default: import.default.clone(),
                names: import.names.clone(),
            })
            .collect(),
    );
    Some(hints)
}

fn emit_text(text: &str, options: &AnchorOptions, out: &mut Vec<Event>) {
    let chars: Vec<char> = text.chars().collect();
    let mut cursor = 0usize;
    let mut normal = String::new();
    while cursor < chars.len() {
        let start = modifier_start(&chars, cursor).unwrap_or(cursor);
        if start == cursor || start < chars.len() && chars[start] == '[' {
            if let Some((end, label, raw_target, extra)) = parse_link(&chars, start) {
                let modifiers = &chars[cursor..start];
                let (attrs, warning) = build_attributes(modifiers, &raw_target, extra);
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

fn modifier_start(chars: &[char], cursor: usize) -> Option<usize> {
    let mut i = cursor;
    while i < chars.len() && matches!(chars[i], '^' | '~' | '!' | '$' | '-' | ';') {
        i += 1;
    }
    if i > cursor {
        Some(i)
    } else if chars.get(cursor) == Some(&'[') {
        Some(cursor)
    } else {
        None
    }
}

fn parse_link(chars: &[char], start: usize) -> Option<(usize, String, String, Option<String>)> {
    if chars.get(start) != Some(&'[') {
        return None;
    }
    let close_label = find_char(chars, start + 1, ']')?;
    if chars.get(close_label + 1) != Some(&'(') {
        return None;
    }
    let close_target = find_matching_paren(chars, close_label + 2)?;
    let label = chars[start + 1..close_label].iter().collect::<String>();
    let target = chars[close_label + 2..close_target]
        .iter()
        .collect::<String>();
    let (raw_target, title) = split_target_title(&target);
    let mut end = close_target + 1;
    let extra = if chars.get(end) == Some(&'{') {
        let close_extra = find_char(chars, end + 1, '}')?;
        let value = chars[end + 1..close_extra].iter().collect::<String>();
        end = close_extra + 1;
        Some(value)
    } else {
        None
    };
    let target = if let Some(title) = title {
        format!("{}\u{0}{}", raw_target, title)
    } else {
        raw_target
    };
    Some((end, label, target, extra))
}

fn build_attributes(
    modifiers: &[char],
    encoded_target: &str,
    extra: Option<String>,
) -> (BTreeMap<String, String>, Option<String>) {
    let (encoded_url, title) = encoded_target
        .split_once('\u{0}')
        .map(|(url, title)| (url, Some(title)))
        .unwrap_or((encoded_target, None));
    let (url, suffix_modifiers) = strip_url_modifiers(encoded_url);
    let mut all_modifiers = modifiers.to_vec();
    all_modifiers.extend(suffix_modifiers.chars());
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
        for (key, value) in parse_extra_attrs(&extra) {
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

fn parse_extra_attrs(input: &str) -> BTreeMap<String, String> {
    let mut attrs = BTreeMap::new();
    for part in input.split(',') {
        let Some((key, value)) = part.split_once(':') else {
            continue;
        };
        let key = key.trim().to_string();
        let value = value
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .to_string();
        if !key.is_empty() {
            attrs.insert(key, value);
        }
    }
    attrs
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
    fn adds_external_defaults_and_modifiers() {
        let mut out = Vec::new();
        emit_text(
            "^--$![foo](https://example.com/path)",
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
        assert!(attrs.contains(&("rel", "noopener noreferrer sponsored nofollow")));
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
