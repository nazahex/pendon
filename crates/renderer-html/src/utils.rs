use serde_json::Value;

pub(crate) fn children(v: &Value) -> Option<&[Value]> {
    v.get("children")
        .and_then(|c| c.as_array())
        .map(|arr| arr.as_slice())
}

pub(crate) fn attr_str<'a>(v: &'a Value, key: &str) -> Option<&'a str> {
    v.get("attrs")
        .and_then(|a| a.get(key))
        .and_then(|val| val.as_str())
}

pub(crate) fn attr_bool(v: &Value, key: &str) -> bool {
    attr_str(v, key).map(|raw| raw == "1").unwrap_or(false)
}

/// Emits every attribute except `skip` as HTML: `name="value"`, or a bare `name`
/// for a flag (§6.3), matching what the JSX renderer emits.
pub(crate) fn render_attrs_except(v: &Value, out: &mut String, skip: &[&str]) {
    let Some(attrs) = v.get("attrs").and_then(|attrs| attrs.as_object()) else {
        return;
    };
    for (name, value) in attrs {
        if skip.contains(&name.as_str()) {
            continue;
        }
        out.push(' ');
        out.push_str(name);
        if matches!(value, Value::Bool(true)) {
            continue;
        }
        out.push_str("=\"");
        match value {
            Value::String(value) => escape_html(value, out),
            other => escape_html(&other.to_string(), out),
        }
        out.push('"');
    }
}

pub(crate) fn escape_html(input: &str, out: &mut String) {
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            // Escape literal braces so they survive JSX embedding and render as
            // text (`&#123;`/`&#125;` are valid HTML entities and render as `{`/`}`).
            '{' => out.push_str("&#123;"),
            '}' => out.push_str("&#125;"),
            _ => out.push(ch),
        }
    }
}
