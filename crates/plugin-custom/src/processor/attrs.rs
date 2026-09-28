use crate::specs::{strip_quotes, AttrType, PluginSpec};
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

    // Pass through any remaining key-value pairs from the {...} block
    // that were not explicitly declared in [[attrs]]. This enables the
    // dynamic spread operator in the Solid renderer.
    for (k, v) in kv_map {
        if !out.contains_key(&k) {
            out.insert(k, strip_quotes(&v));
        }
    }

    (out, diags)
}

// Pastikan fungsi parse_keyvals, split_respecting_quotes, dan ingest_kv
// yang kita perbarui sebelumnya tetap ada di bawah ini.
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

    // Split by comma, but respect quoted strings to prevent breaking
    // values that contain commas or special characters.
    let pairs = split_respecting_quotes(trimmed);

    for pair in pairs {
        ingest_kv(&mut map, &pair);
    }

    map
}

/// Splits a string by commas while ignoring commas inside single or double quotes.
fn split_respecting_quotes(input: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_quote: Option<char> = None;
    let mut escape_next = false;

    for ch in input.chars() {
        if escape_next {
            current.push(ch);
            escape_next = false;
            continue;
        }

        match ch {
            '\\' => {
                escape_next = true;
                current.push(ch);
            }
            '"' | '\'' => {
                if in_quote == Some(ch) {
                    in_quote = None;
                } else if in_quote.is_none() {
                    in_quote = Some(ch);
                }
                current.push(ch);
            }
            ',' if in_quote.is_none() => {
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    result.push(trimmed);
                }
                current.clear();
            }
            _ => {
                current.push(ch);
            }
        }
    }

    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        result.push(trimmed);
    }

    result
}

fn ingest_kv(map: &mut BTreeMap<String, String>, raw: &str) {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return;
    }

    // Only split on the FIRST colon to allow colons inside quoted values
    let Some((key, val)) = trimmed.split_once(':') else {
        return;
    };

    let clean_key = key.trim().to_string();
    let clean_val = val.trim().to_string();

    if !clean_key.is_empty() {
        map.insert(clean_key, clean_val);
    }
}
