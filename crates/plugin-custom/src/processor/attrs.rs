use crate::specs::{AttrType, PluginSpec};
use pendon_core::{Event, Severity};
use regex::Captures;
use std::collections::BTreeMap;

pub fn collect_attrs(
    spec: &PluginSpec,
    caps: Option<&Captures>,
) -> (BTreeMap<String, String>, Vec<Event>) {
    let mut out: BTreeMap<String, String> = BTreeMap::new();
    let mut diags: Vec<Event> = Vec::new();

    let kv_blob = caps
        .and_then(|c| c.name("kv"))
        .map(|m| m.as_str().to_string())
        .unwrap_or_default();
    let kv_map = if kv_blob.is_empty() {
        BTreeMap::new()
    } else {
        parse_keyvals(&kv_blob)
    };

    for attr in &spec.attrs {
        let attr_type: AttrType = attr.r#type.as_str().into();
        let mut value: Option<String> = None;
        if let Some(caps) = caps {
            if let Some(m) = caps.name(&attr.name) {
                value = Some(m.as_str().to_string());
            }
        }
        if value.is_none() {
            if let Some(val) = kv_map.get(&attr.name) {
                value = Some(val.clone());
            }
        }
        if value.is_none() {
            if let Some(def) = &attr.default {
                value = Some(def.clone());
            }
        }
        match value {
            Some(v) => match attr_type.parse(&v) {
                Some(parsed) => {
                    out.insert(attr.name.clone(), parsed);
                }
                None => {
                    diags.push(Event::Diagnostic {
                        severity: Severity::Error,
                        message: format!(
                            "[plugin-custom:{}] attribute '{}' failed to parse as {}",
                            spec.name, attr.name, attr.r#type
                        ),
                        span: None,
                    });
                }
            },
            None => {
                if attr.required {
                    diags.push(Event::Diagnostic {
                        severity: Severity::Error,
                        message: format!(
                            "[plugin-custom:{}] missing required attribute '{}'",
                            spec.name, attr.name
                        ),
                        span: None,
                    });
                }
            }
        }
    }

    (out, diags)
}

fn parse_keyvals(raw: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    let trimmed = raw
        .trim()
        .trim_start_matches('{')
        .trim_end_matches('}')
        .trim();
    if trimmed.is_empty() {
        return map;
    }
    let mut buf = String::new();
    let mut in_quote = false;
    let mut escape = false;
    for ch in trimmed.chars() {
        if escape {
            buf.push(ch);
            escape = false;
            continue;
        }
        if ch == '\\' {
            escape = true;
            continue;
        }
        if ch == '"' {
            in_quote = !in_quote;
            buf.push(ch);
            continue;
        }
        if ch == ',' && !in_quote {
            ingest_kv(&mut map, &buf);
            buf.clear();
            continue;
        }
        buf.push(ch);
    }
    if !buf.is_empty() {
        ingest_kv(&mut map, &buf);
    }
    map
}

fn ingest_kv(map: &mut BTreeMap<String, String>, raw: &str) {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return;
    }
    let (key, val) = match trimmed.split_once(':') {
        Some(pair) => pair,
        None => return,
    };
    map.insert(key.trim().to_string(), val.trim().to_string());
}
