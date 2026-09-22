use pendon_core::Event;
use regex::Regex;
use std::collections::HashMap;

pub fn input_pattern_to_regex(pattern: &str) -> Result<Regex, String> {
    let mut re = String::from("^");
    let mut i = 0;
    let bytes: Vec<char> = pattern.chars().collect();
    while i < bytes.len() {
        if bytes[i] == '[' {
            if let Some(end) = bytes[i + 1..].iter().position(|&c| c == ']') {
                let end_idx = i + 1 + end;
                let name: String = bytes[i + 1..end_idx].iter().collect();
                if name.starts_with("...") {
                    re.push_str("(.+)");
                } else {
                    re.push_str("([^/]+)");
                }
                i = end_idx + 1;
                continue;
            } else {
                return Err("unclosed bracket".to_string());
            }
        }
        let ch = bytes[i];
        match ch {
            '.' | '+' | '*' | '?' | '^' | '$' | '(' | ')' | '|' | '{' | '}' | '\\' => {
                re.push('\\');
                re.push(ch);
            }
            _ => re.push(ch),
        }
        i += 1;
    }
    re.push('$');
    Regex::new(&re).map_err(|e| e.to_string())
}

pub fn captures_to_map(
    pattern: &str,
    caps: &regex::Captures,
) -> Result<HashMap<String, String>, String> {
    let mut map = HashMap::new();
    let mut i = 0;
    let chars: Vec<char> = pattern.chars().collect();
    let mut group = 1;
    while i < chars.len() {
        if chars[i] == '[' {
            let end = chars[i + 1..]
                .iter()
                .position(|&c| c == ']')
                .ok_or_else(|| "unclosed bracket".to_string())?
                + i
                + 1;
            let name: String = chars[i + 1..end].iter().collect();
            let val = caps
                .get(group)
                .ok_or_else(|| "missing capture".to_string())?
                .as_str()
                .to_string();
            map.insert(name, val);
            group += 1;
            i = end + 1;
        } else {
            i += 1;
        }
    }
    Ok(map)
}

pub fn substitute_output(pattern: &str, vars: &HashMap<String, String>) -> Result<String, String> {
    let mut out = String::new();
    let mut i = 0;
    let chars: Vec<char> = pattern.chars().collect();
    while i < chars.len() {
        if chars[i] == '[' {
            let end = chars[i + 1..]
                .iter()
                .position(|&c| c == ']')
                .ok_or_else(|| "unclosed bracket".to_string())?
                + i
                + 1;
            let name: String = chars[i + 1..end].iter().collect();
            let key = name.trim_start_matches("...").to_string();
            let val = vars
                .get(&name)
                .or_else(|| vars.get(&key))
                .ok_or_else(|| format!("missing var {}", name))?;
            out.push_str(val);
            i = end + 1;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    Ok(out)
}

pub fn extract_frontmatter_for_cite(events: &[Event]) -> Option<serde_json::Value> {
    let mut inside = false;
    for event in events {
        match event {
            pendon_core::Event::StartNode(pendon_core::NodeKind::Frontmatter) => inside = true,
            pendon_core::Event::EndNode(pendon_core::NodeKind::Frontmatter) => inside = false,
            pendon_core::Event::Attribute { name, value } if inside && name == "data" => {
                if let Ok(data) = serde_json::from_str(value) {
                    return Some(data);
                }
            }
            _ => {}
        }
    }
    None
}

pub fn merge_refs_for_context(
    front: &serde_json::Value,
    external: Option<&serde_json::Value>,
) -> serde_json::Value {
    let mut merged = serde_json::Map::new();
    if let Some(map) = external.and_then(serde_json::Value::as_object) {
        for (k, v) in map {
            merged.insert(k.clone(), v.clone());
        }
    }
    if let Some(map) = front.as_object() {
        for (k, v) in map {
            merged.insert(k.clone(), v.clone());
        }
    }
    serde_json::Value::Object(merged)
}

pub fn maybe_pretty(s: &str, pretty: bool) -> String {
    if !pretty {
        return s.to_string();
    }
    match serde_json::from_str::<serde_json::Value>(s) {
        Ok(v) => serde_json::to_string_pretty(&v).unwrap_or_else(|_| s.to_string()),
        Err(_) => s.to_string(),
    }
}
