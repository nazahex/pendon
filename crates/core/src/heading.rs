use std::collections::HashMap;

pub fn slugify(input: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in input.chars() {
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() {
            out.push(lower);
            last_dash = false;
        } else if matches!(lower, ' ' | '-' | '_' | '.') {
            if !last_dash && !out.is_empty() {
                out.push('-');
                last_dash = true;
            }
        }
    }
    if out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "section".to_string()
    } else {
        out
    }
}

pub fn extract_id(text: &str) -> (String, Option<String>) {
    let trimmed = text.trim_end();
    if trimmed.ends_with('}') {
        if let Some(start) = trimmed.rfind("{#") {
            if start + 2 < trimmed.len() - 1 {
                let candidate = &trimmed[start + 2..trimmed.len() - 1];
                if !candidate.is_empty()
                    && candidate
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
                {
                    let before = trimmed[..start].trim_end();
                    return (before.to_string(), Some(candidate.to_string()));
                }
            }
        }
    }
    (trimmed.to_string(), None)
}

pub fn strip_trailing_id(text: &str) -> (String, bool) {
    let (clean, id) = extract_id(text);
    (clean, id.is_some())
}

pub fn ensure_unique(base: String, used: &mut HashMap<String, usize>) -> String {
    let counter = used.entry(base.clone()).or_insert(0);
    if *counter == 0 {
        *counter = 1;
        base
    } else {
        *counter += 1;
        format!("{}-{}", base, *counter)
    }
}
