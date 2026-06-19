use pendon_core::{Event, NodeKind, Severity};
use serde_json::{Number, Value};

pub fn process(events: &[Event]) -> Vec<Event> {
    if events.is_empty() {
        return Vec::new();
    }
    if !matches!(events.get(0), Some(Event::StartNode(NodeKind::Document))) {
        return events.to_vec();
    }

    match extract_frontmatter(events) {
        Ok(Some((json, resume_idx))) => {
            let mut out = Vec::with_capacity(events.len() + 4);
            out.push(Event::StartNode(NodeKind::Document));
            out.push(Event::StartNode(NodeKind::Frontmatter));
            out.push(Event::Attribute {
                name: "data".to_string(),
                value: json,
            });
            out.push(Event::EndNode(NodeKind::Frontmatter));
            out.extend(events[resume_idx..].iter().cloned());
            out
        }
        Ok(None) => events.to_vec(),
        Err(msg) => {
            let mut out = events.to_vec();
            let diag = Event::Diagnostic {
                severity: Severity::Error,
                message: format!("[micromatter] {}", msg),
                span: None,
            };
            let insert_at = if out.len() > 1 { 1 } else { 0 };
            out.insert(insert_at, diag);
            out
        }
    }
}

fn extract_frontmatter(events: &[Event]) -> Result<Option<(String, usize)>, String> {
    if events.len() < 4 {
        return Ok(None);
    }

    let mut idx = 1usize;
    // Expect opening --- rendered as a thematic break at the very start
    if !matches!(
        events.get(idx),
        Some(Event::StartNode(NodeKind::ThematicBreak))
    ) {
        return Ok(None);
    }
    if !matches!(events.get(idx + 1), Some(Event::Text(t)) if t.trim() == "---") {
        return Ok(None);
    }
    if !matches!(
        events.get(idx + 2),
        Some(Event::EndNode(NodeKind::ThematicBreak))
    ) {
        return Ok(None);
    }
    idx += 3;

    // Optional newline(s) immediately after opening fence
    while idx < events.len() {
        match &events[idx] {
            Event::Text(t) if t == "\n" => idx += 1,
            _ => break,
        }
    }

    let mut content = String::new();
    let mut close_idx: Option<usize> = None;
    while idx + 2 < events.len() {
        if matches!(
            events.get(idx),
            Some(Event::StartNode(NodeKind::ThematicBreak))
        ) && matches!(events.get(idx + 1), Some(Event::Text(t)) if t.trim() == "---")
            && matches!(
                events.get(idx + 2),
                Some(Event::EndNode(NodeKind::ThematicBreak))
            )
        {
            close_idx = Some(idx);
            break;
        }
        if let Event::Text(t) = &events[idx] {
            content.push_str(t);
        }
        idx += 1;
    }

    let close_start = match close_idx {
        Some(c) => c,
        None => return Err("missing closing ---".to_string()),
    };

    let json = parse_frontmatter(&content)?;

    let mut resume = close_start + 3;
    while resume < events.len() {
        match &events[resume] {
            Event::Text(t) if t == "\n" => resume += 1,
            Event::EndNode(NodeKind::Paragraph) | Event::StartNode(NodeKind::Paragraph) => {
                resume += 1;
                break;
            }
            _ => break,
        }
    }

    Ok(Some((json, resume)))
}

fn parse_frontmatter(body: &str) -> Result<String, String> {
    let yaml: serde_yaml::Value =
        serde_yaml::from_str(body).map_err(|e| format!("invalid YAML frontmatter: {}", e))?;
    let json = yaml_to_json(&yaml)?;
    serde_json::to_string(&json).map_err(|e| e.to_string())
}

fn yaml_to_json(value: &serde_yaml::Value) -> Result<Value, String> {
    match value {
        serde_yaml::Value::Null => Ok(Value::Null),
        serde_yaml::Value::Bool(v) => Ok(Value::Bool(*v)),
        serde_yaml::Value::Number(v) => {
            if let Some(i) = v.as_i64() {
                Ok(Value::Number(Number::from(i)))
            } else if let Some(u) = v.as_u64() {
                Ok(Value::Number(Number::from(u)))
            } else if let Some(f) = v.as_f64() {
                Number::from_f64(f)
                    .map(Value::Number)
                    .ok_or_else(|| "invalid floating-point value".to_string())
            } else {
                Err("unsupported numeric value".to_string())
            }
        }
        serde_yaml::Value::String(v) => Ok(Value::String(v.clone())),
        serde_yaml::Value::Sequence(values) => {
            let mut out = Vec::with_capacity(values.len());
            for item in values {
                out.push(yaml_to_json(item)?);
            }
            Ok(Value::Array(out))
        }
        serde_yaml::Value::Mapping(entries) => {
            let mut out = serde_json::Map::new();
            for (key, value) in entries {
                out.insert(yaml_key_to_string(key)?, yaml_to_json(value)?);
            }
            Ok(Value::Object(out))
        }
        serde_yaml::Value::Tagged(tagged) => yaml_to_json(&tagged.value),
    }
}

fn yaml_key_to_string(value: &serde_yaml::Value) -> Result<String, String> {
    match value {
        serde_yaml::Value::String(v) => Ok(v.clone()),
        serde_yaml::Value::Bool(v) => Ok(v.to_string()),
        serde_yaml::Value::Number(v) => Ok(v.to_string()),
        serde_yaml::Value::Null => Ok("null".to_string()),
        _ => Err("YAML mapping keys must be scalar values".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_micromatter(input: &str) -> Vec<Event> {
        let events = pendon_core::parse(input, &pendon_core::Options::default());
        process(&events)
    }

    fn frontmatter_json(events: &[Event]) -> String {
        events
            .iter()
            .find_map(|ev| match ev {
                Event::Attribute { name, value } if name == "data" => Some(value.clone()),
                _ => None,
            })
            .expect("frontmatter data attr")
    }

    #[test]
    fn parses_nested_yaml_frontmatter() {
        let events = run_micromatter(
            "---\ntitle: Demo\nmeta:\n  tags:\n    - a\n    - b\n  flags:\n    enabled: true\n    count: 3\n---\n\n# Hello\n",
        );

        let data: Value = serde_json::from_str(&frontmatter_json(&events)).expect("valid json");
        assert_eq!(data.get("title").and_then(|v| v.as_str()), Some("Demo"));
        assert_eq!(data["meta"]["tags"][0].as_str(), Some("a"));
        assert_eq!(data["meta"]["flags"]["enabled"].as_bool(), Some(true));
        assert_eq!(data["meta"]["flags"]["count"].as_i64(), Some(3));
    }

    #[test]
    fn parses_block_scalar_and_flow_sequence() {
        let events = run_micromatter(
            "---\nsummary: |\n  line one\n  line two\nitems: [one, two, three]\n---\n",
        );

        let data: Value = serde_json::from_str(&frontmatter_json(&events)).expect("valid json");
        assert_eq!(data["summary"].as_str(), Some("line one\nline two\n"));
        assert_eq!(data["items"][1].as_str(), Some("two"));
    }

    #[test]
    fn emits_error_for_invalid_yaml() {
        let events = run_micromatter("---\nfoo: [1, two\n---\n");
        assert!(events.iter().any(|ev| matches!(ev, Event::Diagnostic { severity: Severity::Error, message, .. } if message.contains("invalid YAML frontmatter"))));
    }
}
