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

pub(crate) fn escape_html(input: &str, out: &mut String) {
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
}
