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

/// Words a `export const <name>` may not use, because the emitted module would
/// not parse (ADR-0009, §20 of `spec/SYNTAX.md`).
///
/// A Solid module is strict-mode code, so this is the ES2015+ reserved set plus
/// `await` (module goal), the strict-mode reserved words (`let`, `static`, …) and
/// the two bindings strict mode forbids (`eval`, `arguments`). `of` is **not**
/// here: it is contextual, only special inside `for (… of …)`.
const RESERVED_IDENTIFIERS: [&str; 48] = [
    "arguments",
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "eval",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "implements",
    "import",
    "in",
    "instanceof",
    "interface",
    "let",
    "new",
    "null",
    "package",
    "private",
    "protected",
    "public",
    "return",
    "static",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "yield",
];

/// Validates a name that becomes `export const <name> = …` in the Solid output.
///
/// One site for every producer (`cite`, `bind`'s `[[task.bind.inject]]`,
/// Phase 4's `extract.export`): the message names the config key **and** the
/// offending value, so a typo never renders with a broken module behind it.
pub fn validate_export_identifier(task: &str, key: &str, value: &str) -> Result<String, String> {
    let mut chars = value.chars();
    let starts_ok = chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_' || c == '$');
    let rest_ok = chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$');
    let reserved = RESERVED_IDENTIFIERS.contains(&value);
    if starts_ok && rest_ok && !reserved {
        return Ok(value.to_string());
    }
    let mut message = format!("{task}.{key} '{value}' is not a valid JavaScript identifier");
    if reserved {
        message.push_str(" (reserved word)");
    }
    Err(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// §20: `[A-Za-z_$][A-Za-z0-9_$]*` is accepted unchanged.
    #[test]
    fn export_identifier_accepts_valid_names() {
        for name in [
            "citations",
            "bibliographyData",
            "_private",
            "$dollar",
            "a1",
            "dataX",
        ] {
            assert_eq!(
                validate_export_identifier("cite", "citations_export", name),
                Ok(name.to_string()),
                "{name}"
            );
        }
    }

    /// A leading digit, a dash or an empty name is rejected, naming the key and
    /// the value (T2.4).
    #[test]
    fn export_identifier_rejects_malformed_names() {
        for name in ["1data", "citations-list", "", "with space", "ünicode"] {
            let err = validate_export_identifier("cite", "citations_export", name)
                .expect_err("malformed identifier");
            assert!(err.contains("not a valid JavaScript identifier"), "{err}");
            assert!(err.contains(&format!("'{name}'")), "{err}");
            assert!(!err.contains("reserved word"), "{err}");
        }
    }

    /// §20: a reserved word is syntactically fine but cannot be an exported
    /// binding, so it is rejected as well.
    #[test]
    fn export_identifier_rejects_reserved_words() {
        for name in [
            "export",
            "default",
            "class",
            "await",
            "null",
            "true",
            "in",
            "yield",
            "static",
            "arguments",
            "eval",
        ] {
            let err =
                validate_export_identifier("bind", "inject", name).expect_err("reserved word");
            assert!(err.starts_with(&format!("bind.inject '{name}'")), "{err}");
            assert!(
                err.contains("is not a valid JavaScript identifier"),
                "{err}"
            );
            assert!(err.contains("reserved word"), "{err}");
        }
    }

    /// `of` is contextual, not reserved: `const of = 1` parses, so a data file
    /// named `of.toml` must not be rejected.
    #[test]
    fn export_identifier_accepts_contextual_keywords() {
        assert_eq!(
            validate_export_identifier("bind", "inject", "of"),
            Ok("of".to_string())
        );
    }

    /// The message carries the producing task's own name, so `bind` never
    /// reports a `cite.` prefix.
    #[test]
    fn export_identifier_prefixes_the_producing_task() {
        let err = validate_export_identifier("bind", "inject", "data-x").expect_err("bad name");
        assert_eq!(
            err,
            "bind.inject 'data-x' is not a valid JavaScript identifier"
        );
    }
}
