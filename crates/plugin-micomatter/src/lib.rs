use pendon_core::{Event, NodeKind, Severity};
use serde_json::{Map, Number, Value};

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
                message: format!("[micomatter] {}", msg),
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
    if !matches!(events.get(idx), Some(Event::StartNode(NodeKind::ThematicBreak))) {
        return Ok(None);
    }
    if !matches!(events.get(idx + 1), Some(Event::Text(t)) if t.trim() == "---") {
        return Ok(None);
    }
    if !matches!(events.get(idx + 2), Some(Event::EndNode(NodeKind::ThematicBreak))) {
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
        if matches!(events.get(idx), Some(Event::StartNode(NodeKind::ThematicBreak)))
            && matches!(events.get(idx + 1), Some(Event::Text(t)) if t.trim() == "---")
            && matches!(events.get(idx + 2), Some(Event::EndNode(NodeKind::ThematicBreak)))
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

    let mut block = String::from("---\n");
    block.push_str(&content);
    if !block.ends_with('\n') {
        block.push('\n');
    }
    block.push_str("---");
    let json = parse_frontmatter(&block)?;

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

fn parse_frontmatter(block: &str) -> Result<String, String> {
    let mut lines = block.lines();
    let first = lines.next().unwrap_or("").trim();
    if first != "---" {
        return Err("frontmatter must start with ---".to_string());
    }

    let mut body: Vec<&str> = Vec::new();
    let mut closed = false;
    for line in lines.by_ref() {
        if line.trim() == "---" {
            closed = true;
            break;
        }
        body.push(line);
    }

    if !closed {
        return Err("missing closing ---".to_string());
    }

    for rest in lines {
        if !rest.trim().is_empty() {
            return Err("content found after closing ---".to_string());
        }
    }

    let mut map = Map::new();
    for (idx, raw_line) in body.iter().enumerate() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let (key, raw_value) = trimmed
            .split_once(':')
            .ok_or_else(|| format!("missing ':' on frontmatter line {}", idx + 1))?;
        validate_key(key)?;
        let value_str = strip_comment(raw_value).trim().to_string();
        let value = parse_value(&value_str)
            .map_err(|e| format!("{} at frontmatter line {}", e, idx + 1))?;
        map.insert(key.trim().to_string(), value);
    }

    serde_json::to_string(&Value::Object(map)).map_err(|e| e.to_string())
}

fn validate_key(key: &str) -> Result<(), String> {
    let mut chars = key.chars();
    match chars.next() {
        Some(ch) if ch.is_ascii_alphabetic() || ch == '_' => {}
        _ => return Err(format!("invalid key '{}': must start with [A-Za-z_]", key.trim())),
    }
    for ch in chars {
        if !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '-') {
            return Err(format!(
                "invalid key '{}': only [A-Za-z0-9_-] allowed",
                key.trim()
            ));
        }
    }
    Ok(())
}

fn strip_comment(raw: &str) -> String {
    let mut out = String::new();
    let mut in_quote = false;
    let mut escape = false;
    for ch in raw.chars() {
        if escape {
            out.push(ch);
            escape = false;
            continue;
        }
        if ch == '\\' && in_quote {
            escape = true;
            out.push(ch);
            continue;
        }
        if ch == '"' {
            in_quote = !in_quote;
            out.push(ch);
            continue;
        }
        if ch == '#' && !in_quote {
            break;
        }
        out.push(ch);
    }
    out
}

fn parse_value(raw: &str) -> Result<Value, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("missing value".to_string());
    }
    if trimmed.starts_with('[') {
        return parse_array(trimmed);
    }
    parse_scalar(trimmed)
}

fn parse_array(raw: &str) -> Result<Value, String> {
    let inner = raw
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .ok_or_else(|| "unterminated array".to_string())?;
    let content = inner.trim();
    if content.is_empty() {
        return Ok(Value::Array(Vec::new()));
    }
    let items = split_array_items(content)?;
    let mut out: Vec<Value> = Vec::with_capacity(items.len());
    let mut kind: Option<ArrayKind> = None;
    for item in items {
        let value = parse_scalar(item.trim())?;
        let v_kind = classify_value(&value);
        if let Some(existing) = kind {
            if existing != v_kind {
                return Err("mixed array types".to_string());
            }
        } else {
            kind = Some(v_kind);
        }
        out.push(value);
    }
    Ok(Value::Array(out))
}

fn parse_scalar(raw: &str) -> Result<Value, String> {
    if raw.is_empty() {
        return Err("missing value".to_string());
    }
    if raw.starts_with('"') {
        return Ok(Value::String(parse_quoted(raw)?));
    }
    if raw == "true" {
        return Ok(Value::Bool(true));
    }
    if raw == "false" {
        return Ok(Value::Bool(false));
    }
    if let Ok(i) = raw.parse::<i64>() {
        return Ok(Value::Number(Number::from(i)));
    }
    if let Ok(f) = raw.parse::<f64>() {
        if let Some(num) = Number::from_f64(f) {
            return Ok(Value::Number(num));
        }
    }
    if raw.contains('[') || raw.contains(']') || raw.contains(',') {
        return Err("bare strings cannot contain '[', ']', or ','".to_string());
    }
    Ok(Value::String(raw.to_string()))
}

fn parse_quoted(raw: &str) -> Result<String, String> {
    if !(raw.starts_with('"') && raw.ends_with('"')) {
        return Err("unterminated quoted string".to_string());
    }
    let inner = &raw[1..raw.len() - 1];
    let mut out = String::new();
    let mut escape = false;
    for ch in inner.chars() {
        if escape {
            match ch {
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                'n' => out.push('\n'),
                other => out.push(other),
            }
            escape = false;
            continue;
        }
        if ch == '\\' {
            escape = true;
            continue;
        }
        out.push(ch);
    }
    if escape {
        return Err("unterminated quoted string".to_string());
    }
    Ok(out)
}

fn split_array_items(content: &str) -> Result<Vec<String>, String> {
    let mut items: Vec<String> = Vec::new();
    let mut buf = String::new();
    let mut in_quote = false;
    let mut escape = false;
    for ch in content.chars() {
        if escape {
            buf.push(ch);
            escape = false;
            continue;
        }
        if in_quote {
            if ch == '\\' {
                escape = true;
                buf.push(ch);
                continue;
            }
            if ch == '"' {
                in_quote = false;
            }
            buf.push(ch);
            continue;
        }
        if ch == '"' {
            in_quote = true;
            buf.push(ch);
            continue;
        }
        if ch == ',' {
            items.push(buf.trim().to_string());
            buf.clear();
            continue;
        }
        buf.push(ch);
    }
    if in_quote {
        return Err("unterminated quoted string in array".to_string());
    }
    if !buf.trim().is_empty() {
        items.push(buf.trim().to_string());
    }
    Ok(items)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ArrayKind {
    Bool,
    Int,
    Float,
    String,
}

fn classify_value(v: &Value) -> ArrayKind {
    match v {
        Value::Bool(_) => ArrayKind::Bool,
        Value::Number(n) => {
            if n.is_i64() {
                ArrayKind::Int
            } else {
                ArrayKind::Float
            }
        }
        _ => ArrayKind::String,
    }
}
