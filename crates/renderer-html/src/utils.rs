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
///
/// A fragment (`$fragment`, a nested `mdp` payload) is markup, not data: its node
/// list is rendered to HTML and escaped into the attribute value, so a bound
/// `data: $mdp` reads as `<a data="<p>…</p>">` instead of leaking the AST JSON.
pub(crate) fn render_attrs_except(v: &Value, out: &mut String, skip: &[&str]) {
    let Some(attrs) = v.get("attrs").and_then(|attrs| attrs.as_object()) else {
        return;
    };
    for (name, value) in attrs {
        // A `__`-prefixed key is an internal marker (`__plugin_kind`, §11 rule 3)
        // and must never reach the output, matching the JSX renderer.
        if name.starts_with("__") || skip.contains(&name.as_str()) {
            continue;
        }
        render_attr(name, value, out);
    }
}

/// Emits one attribute: `name="value"`, or a bare `name` for a flag (§6.3).
///
/// The value goes through [`fragment_aware_text`], so a fragment is markup here
/// too — including in the nodes that render their own attributes (`Link`).
pub(crate) fn render_attr(name: &str, value: &Value, out: &mut String) {
    out.push(' ');
    out.push_str(name);
    if matches!(value, Value::Bool(true)) {
        return;
    }
    // A `JSX_ATTR_PREFIX` payload (an extras `( … )` expression) is a
    // Solid-only concept; in HTML it is emitted as an escaped string so
    // the sentinel never leaks into markup.
    if let Value::String(text) = value {
        if let Some(payload) = pendon_core::jsx_attr_payload(text) {
            out.push_str("=\"");
            escape_html(payload, out);
            out.push('"');
            return;
        }
    }
    out.push_str("=\"");
    escape_html(&fragment_aware_text(value), out);
    out.push('"');
}

/// The text of an attribute value, with every fragment rendered to HTML.
///
/// A whole-attribute fragment renders as its markup; a fragment nested inside a
/// structured value (`{body: $mdp}`) renders in place, so the surrounding JSON
/// stays readable.
pub(crate) fn fragment_aware_text(value: &Value) -> String {
    if let Some(nodes) = pendon_renderer_ast::fragment_nodes(value) {
        let mut out = String::new();
        crate::compact::render_fragment_nodes(nodes, &mut out);
        return out;
    }
    match value {
        Value::String(text) => text.clone(),
        Value::Array(items) => {
            let rendered: Vec<String> = items.iter().map(fragment_aware_text).collect();
            format!("[{}]", rendered.join(","))
        }
        Value::Object(map) => {
            let rendered: Vec<String> = map
                .iter()
                .map(|(key, item)| {
                    format!(
                        "{}:{}",
                        serde_json::Value::String(key.clone()),
                        fragment_aware_text(item)
                    )
                })
                .collect();
            format!("{{{}}}", rendered.join(","))
        }
        other => other.to_string(),
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
