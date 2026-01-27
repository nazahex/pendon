use serde_json::{Map, Value};

use crate::imports::{ComponentTemplate, SolidRenderHints};

pub fn render_template(
    template: &ComponentTemplate,
    attrs: Option<&Map<String, Value>>,
    children: &str,
    text: Option<&str>,
) -> String {
    let mut out = String::new();
    let tpl = template.template.as_str();
    let bytes = tpl.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            if tpl[i..].starts_with("{children}") {
                out.push_str(children);
                i += "{children}".len();
                continue;
            }
            if tpl[i..].starts_with("{text}") {
                out.push_str(text.unwrap_or(""));
                i += "{text}".len();
                continue;
            }
            if tpl[i..].starts_with("{attrs.") {
                if let Some(end) = tpl[i + 7..].find('}') {
                    let key = &tpl[i + 7..i + 7 + end];
                    let val = get_attr_value(attrs, key);
                    out.push_str(&val);
                    i = i + 7 + end + 1;
                    continue;
                }
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

pub fn select_template<'a>(
    hints: Option<&'a SolidRenderHints>,
    node_type: &str,
    v: &Value,
) -> Option<&'a ComponentTemplate> {
    let Some(h) = hints else {
        return None;
    };
    let node_name = v
        .get("attrs")
        .and_then(|a: &Value| a.get("name"))
        .and_then(|n| n.as_str());

    let mut fallback: Option<&ComponentTemplate> = None;
    for tpl in &h.templates {
        if tpl.node_type != node_type {
            continue;
        }
        if let Some(expected) = tpl.node_name.as_deref() {
            if Some(expected) == node_name {
                return Some(tpl);
            }
        } else if fallback.is_none() {
            fallback = Some(tpl);
        }
    }
    fallback
}

fn get_attr_value(attrs: Option<&Map<String, Value>>, key: &str) -> String {
    let Some(map) = attrs else {
        return String::new();
    };
    match map.get(key) {
        Some(Value::String(s)) => s.clone(),
        Some(v) => v.to_string(),
        None => String::new(),
    }
}
