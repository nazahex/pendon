use std::collections::BTreeMap;

use pendon_core::{Event, NodeKind};
use pendon_renderer_solid::{ComponentTemplate, ImportEntry, SolidRenderHints};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
struct VicadoSpec {
    language: String,
    class_name: Option<String>,
    id: Option<String>,
    props: BTreeMap<String, Value>,
}

pub fn process(events: &[Event]) -> Vec<Event> {
    let mut out: Vec<Event> = Vec::with_capacity(events.len());
    let mut i = 0usize;

    while i < events.len() {
        if matches!(events.get(i), Some(Event::StartNode(NodeKind::CodeFence))) {
            if let Some(end) = find_matching_end(events, i, NodeKind::CodeFence) {
                if let Some((spec, code)) = extract_vicado_spec_and_code(&events[i + 1..end]) {
                    emit_vicado_component(&mut out, &spec, &code);
                    i = end + 1;
                    continue;
                }
                out.extend(events[i..=end].iter().cloned());
                i = end + 1;
                continue;
            }
        }

        out.push(events[i].clone());
        i += 1;
    }

    out
}

pub fn solid_hints() -> SolidRenderHints {
    let mut hints = SolidRenderHints::default();
    let key = ("Vicado".to_string(), Some("Vicado".to_string()));
    hints.templates.push(ComponentTemplate {
        node_type: "Vicado".to_string(),
        node_name: Some("Vicado".to_string()),
        template: "<Vicado {attrs.jsx_props} />".to_string(),
    });
    hints
        .template_imports
        .entry(key)
        .or_default()
        .push(ImportEntry::Structured {
            module: "vicado".to_string(),
            default: None,
            names: vec!["Vicado".to_string()],
        });
    hints
}

fn emit_vicado_component(out: &mut Vec<Event>, spec: &VicadoSpec, code: &str) {
    let jsx_props = build_jsx_props(spec, code);
    out.push(Event::StartNode(NodeKind::Custom("Vicado".to_string())));
    out.push(Event::Attribute {
        name: "name".to_string(),
        value: "Vicado".to_string(),
    });
    out.push(Event::Attribute {
        name: "jsx_props".to_string(),
        value: jsx_props,
    });
    out.push(Event::EndNode(NodeKind::Custom("Vicado".to_string())));
}

fn build_jsx_props(spec: &VicadoSpec, code: &str) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.push(format!("language=\"{}\"", escape_jsx_attr(&spec.language)));
    parts.push(format!("code={{{}}}", json_string_expr(code)));

    if let Some(class_name) = spec.class_name.as_deref() {
        if !class_name.is_empty() {
            parts.push(format!("class=\"{}\"", escape_jsx_attr(class_name)));
        }
    }
    if let Some(id) = spec.id.as_deref() {
        if !id.is_empty() {
            parts.push(format!("id=\"{}\"", escape_jsx_attr(id)));
        }
    }

    for (key, value) in &spec.props {
        if !is_valid_prop_key(key) {
            continue;
        }
        let rendered = match value {
            Value::Bool(v) => format!("{}={{{}}}", key, if *v { "true" } else { "false" }),
            Value::Number(v) => format!("{}={{{}}}", key, v),
            Value::String(v) => format!("{}=\"{}\"", key, escape_jsx_attr(v)),
            _ => format!("{}={{{}}}", key, value),
        };
        parts.push(rendered);
    }

    parts.join(" ")
}

fn extract_vicado_spec_and_code(slice: &[Event]) -> Option<(VicadoSpec, String)> {
    let mut info: Option<&str> = None;
    let mut code = String::new();

    for ev in slice {
        match ev {
            Event::Attribute { name, value } if name == "lang" => {
                info = Some(value.as_str());
            }
            Event::Text(text) => code.push_str(text),
            _ => {}
        }
    }

    let info = info?;
    let spec = parse_vicado_info(info)?;
    Some((spec, trim_fence_boundary_newlines(&code).to_string()))
}

fn trim_fence_boundary_newlines(mut code: &str) -> &str {
    if let Some(rest) = code.strip_prefix('\n') {
        code = rest;
    }
    if let Some(rest) = code.strip_suffix('\n') {
        code = rest;
    }
    code
}

fn parse_vicado_info(info: &str) -> Option<VicadoSpec> {
    let mut rest = info.trim();
    if rest.is_empty() {
        return None;
    }

    let (language, after_lang) = take_token(rest)?;
    rest = after_lang.trim_start();

    let (plugin_name, after_plugin) = take_token(rest)?;
    if !plugin_name.eq_ignore_ascii_case("vicado") {
        return None;
    }
    rest = after_plugin.trim_start();

    let mut class_name: Option<String> = None;
    let mut id: Option<String> = None;
    let mut props = BTreeMap::new();

    if let Some(after_open) = rest.strip_prefix('[') {
        let close = find_matching_bracket(after_open, '[', ']')?;
        let class_block = &after_open[..close];
        let (classes, parsed_id) = parse_class_block(class_block);
        if !classes.is_empty() {
            class_name = Some(classes.join(" "));
        }
        id = parsed_id;
        rest = after_open[close + 1..].trim_start();
    }

    if let Some(after_open) = rest.strip_prefix('{') {
        let close = find_matching_bracket(after_open, '{', '}')?;
        let props_block = &after_open[..close];
        props = parse_props_block(props_block);
        rest = after_open[close + 1..].trim_start();
    }

    if !rest.is_empty() {
        return None;
    }

    Some(VicadoSpec {
        language: language.to_string(),
        class_name,
        id,
        props,
    })
}

fn parse_class_block(input: &str) -> (Vec<String>, Option<String>) {
    let mut classes: Vec<String> = Vec::new();
    let mut id: Option<String> = None;

    for token in split_csv_like(input) {
        let t = token.trim();
        if t.is_empty() {
            continue;
        }
        if let Some(raw) = t.strip_prefix('#') {
            if !raw.trim().is_empty() {
                id = Some(raw.trim().to_string());
            }
            continue;
        }
        if let Some(raw) = t.strip_prefix('.') {
            if !raw.trim().is_empty() {
                classes.push(raw.trim().to_string());
            }
            continue;
        }
        classes.push(t.to_string());
    }

    (classes, id)
}

fn parse_props_block(input: &str) -> BTreeMap<String, Value> {
    let mut props = BTreeMap::new();

    for pair in split_csv_like(input) {
        let Some((raw_key, raw_value)) = pair.split_once(':') else {
            continue;
        };

        let key = raw_key.trim();
        if !is_valid_prop_key(key) {
            continue;
        }

        let value = parse_value(raw_value.trim());
        props.insert(key.to_string(), value);
    }

    props
}

fn parse_value(input: &str) -> Value {
    if input.eq_ignore_ascii_case("true") {
        return Value::Bool(true);
    }
    if input.eq_ignore_ascii_case("false") {
        return Value::Bool(false);
    }

    if (input.starts_with('"') && input.ends_with('"'))
        || (input.starts_with('\'') && input.ends_with('\''))
    {
        let inner = &input[1..input.len().saturating_sub(1)];
        return Value::String(unescape_basic(inner));
    }

    if let Ok(v) = input.parse::<i64>() {
        return Value::Number(v.into());
    }
    if let Ok(v) = input.parse::<f64>() {
        if let Some(n) = serde_json::Number::from_f64(v) {
            return Value::Number(n);
        }
    }

    Value::String(input.to_string())
}

fn split_csv_like(input: &str) -> Vec<&str> {
    let mut out: Vec<&str> = Vec::new();
    let mut start = 0usize;
    let mut quote: Option<char> = None;

    for (idx, ch) in input.char_indices() {
        match ch {
            '"' | '\'' => {
                if quote == Some(ch) {
                    quote = None;
                } else if quote.is_none() {
                    quote = Some(ch);
                }
            }
            ',' if quote.is_none() => {
                out.push(input[start..idx].trim());
                start = idx + 1;
            }
            _ => {}
        }
    }

    if start <= input.len() {
        out.push(input[start..].trim());
    }

    out
}

fn take_token(input: &str) -> Option<(&str, &str)> {
    let trimmed = input.trim_start();
    if trimmed.is_empty() {
        return None;
    }
    let split = trimmed
        .char_indices()
        .find(|(_, c)| c.is_whitespace())
        .map(|(i, _)| i)
        .unwrap_or(trimmed.len());

    Some((&trimmed[..split], &trimmed[split..]))
}

fn find_matching_end(events: &[Event], start_idx: usize, kind: NodeKind) -> Option<usize> {
    let mut depth = 0isize;
    for (idx, ev) in events.iter().enumerate().skip(start_idx) {
        match ev {
            Event::StartNode(k) if *k == kind => depth += 1,
            Event::EndNode(k) if *k == kind => {
                depth -= 1;
                if depth == 0 {
                    return Some(idx);
                }
            }
            _ => {}
        }
    }
    None
}

fn find_matching_bracket(input: &str, open: char, close: char) -> Option<usize> {
    let mut depth = 1usize;
    let mut quote: Option<char> = None;

    for (idx, ch) in input.char_indices() {
        match ch {
            '"' | '\'' => {
                if quote == Some(ch) {
                    quote = None;
                } else if quote.is_none() {
                    quote = Some(ch);
                }
            }
            c if c == open && quote.is_none() => depth += 1,
            c if c == close && quote.is_none() => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(idx);
                }
            }
            _ => {}
        }
    }

    None
}

fn is_valid_prop_key(key: &str) -> bool {
    let mut chars = key.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }
    chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
}

fn json_string_expr(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string())
}

fn unescape_basic(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(next) = chars.next() {
                match next {
                    'n' => out.push('\n'),
                    'r' => out.push('\r'),
                    't' => out.push('\t'),
                    '\\' => out.push('\\'),
                    '\'' => out.push('\''),
                    '"' => out.push('"'),
                    other => {
                        out.push('\\');
                        out.push(other);
                    }
                }
                continue;
            }
        }
        out.push(ch);
    }

    out
}

fn escape_jsx_attr(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_info_with_class_id_and_props() {
        let spec = parse_vicado_info(
            "typescript vicado [.class, class2, #root] {mount: \"visible\", foo: \"baz\", baz: 89}",
        )
        .unwrap();

        assert_eq!(spec.language, "typescript");
        assert_eq!(spec.class_name.as_deref(), Some("class class2"));
        assert_eq!(spec.id.as_deref(), Some("root"));
        assert_eq!(
            spec.props.get("mount"),
            Some(&Value::String("visible".to_string()))
        );
        assert_eq!(
            spec.props.get("foo"),
            Some(&Value::String("baz".to_string()))
        );
        assert_eq!(spec.props.get("baz"), Some(&Value::Number(89.into())));
    }

    #[test]
    fn ignores_non_vicado_info_string() {
        assert!(parse_vicado_info("typescript").is_none());
        assert!(parse_vicado_info("typescript img").is_none());
    }

    #[test]
    fn transforms_vicado_codefence_into_custom_node() {
        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::CodeFence),
            Event::Attribute {
                name: "lang".to_string(),
                value: "typescript vicado {mount: \"visible\"}".to_string(),
            },
            Event::Text("function tsCodeHere()".to_string()),
            Event::EndNode(NodeKind::CodeFence),
            Event::EndNode(NodeKind::Document),
        ];

        let out = process(&events);
        assert!(out.iter().any(|e| {
            matches!(e, Event::StartNode(NodeKind::Custom(name)) if name == "Vicado")
        }));
        assert!(out.iter().any(|e| {
            matches!(e, Event::Attribute { name, value } if name == "jsx_props" && value.contains("code={\"function tsCodeHere()\"}"))
        }));
    }

    #[test]
    fn provides_solid_template_hint() {
        let hints = solid_hints();
        assert!(hints
            .templates
            .iter()
            .any(|tpl| tpl.node_type == "Vicado" && tpl.node_name.as_deref() == Some("Vicado")));
    }

    #[test]
    fn trims_single_boundary_newline_from_code_body() {
        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::CodeFence),
            Event::Attribute {
                name: "lang".to_string(),
                value: "html vicado".to_string(),
            },
            Event::Text("\n<div>ok</div>\n".to_string()),
            Event::EndNode(NodeKind::CodeFence),
            Event::EndNode(NodeKind::Document),
        ];

        let out = process(&events);
        assert!(out.iter().any(|e| {
            matches!(e, Event::Attribute { name, value }
                if name == "jsx_props" && value.contains("code={\"<div>ok</div>\"}"))
        }));
    }

    #[test]
    fn keeps_intentional_blank_line_inside_code_body() {
        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::CodeFence),
            Event::Attribute {
                name: "lang".to_string(),
                value: "html vicado".to_string(),
            },
            Event::Text("\n\n<div>ok</div>\n\n".to_string()),
            Event::EndNode(NodeKind::CodeFence),
            Event::EndNode(NodeKind::Document),
        ];

        let out = process(&events);
        assert!(out.iter().any(|e| {
            matches!(e, Event::Attribute { name, value }
                if name == "jsx_props" && value.contains("code={\"\\n<div>ok</div>\\n\"}"))
        }));
    }
}
