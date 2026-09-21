// src/attrs.rs
#[derive(Debug, Clone, Default)]
pub struct AttrSpec {
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub extra: Vec<(String, String)>,
}

pub fn parse_attr_block(input: &str) -> (AttrSpec, &str) {
    let s = input.trim_start();
    let mut spec = AttrSpec::default();
    let mut cursor = 0;
    let bytes = s.as_bytes();

    // Parse [.class,#id]
    if cursor < bytes.len() && bytes[cursor] == b'[' {
        if let Some(close_br) = s[cursor + 1..].find(']') {
            let class_block = &s[cursor + 1..cursor + 1 + close_br];
            for token in class_block
                .split(',')
                .map(|t| t.trim())
                .filter(|t| !t.is_empty())
            {
                if let Some(class_name) = token.strip_prefix('.') {
                    if !class_name.is_empty() {
                        spec.classes.push(class_name.to_string());
                    }
                } else if let Some(id) = token.strip_prefix('#') {
                    if !id.is_empty() {
                        spec.id = Some(id.to_string());
                    }
                }
            }
            cursor += 1 + close_br + 1;
        }
    }

    // Skip whitespace
    while cursor < bytes.len() && bytes[cursor] == b' ' {
        cursor += 1;
    }

    // Parse {key: "val"}
    if cursor < bytes.len() && bytes[cursor] == b'{' {
        if let Some(close_curly) = s[cursor + 1..].find('}') {
            let kv_block = &s[cursor + 1..cursor + 1 + close_curly];
            for pair in split_csv(kv_block) {
                let Some((k, v)) = pair.split_once(':') else {
                    continue;
                };
                let key = k.trim();
                let value = unquote(v.trim());
                if !key.is_empty() {
                    spec.extra.push((key.to_string(), value));
                }
            }
            cursor += 1 + close_curly + 1;
        }
    }

    let rest = s.get(cursor..).unwrap_or("");
    (spec, rest)
}

pub fn merge_attrs(base: &mut AttrSpec, overlay: &AttrSpec) {
    if let Some(id) = &overlay.id {
        base.id = Some(id.clone());
    }
    base.classes.extend(overlay.classes.clone());
    base.extra.extend(overlay.extra.clone());
}

fn split_csv(input: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut quote: Option<char> = None;

    for ch in input.chars() {
        if ch == '"' || ch == '\'' {
            if quote == Some(ch) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(ch);
            }
            buf.push(ch);
            continue;
        }

        if ch == ',' && quote.is_none() {
            if !buf.trim().is_empty() {
                out.push(buf.trim().to_string());
            }
            buf.clear();
            continue;
        }

        buf.push(ch);
    }

    if !buf.trim().is_empty() {
        out.push(buf.trim().to_string());
    }

    out
}

fn unquote(s: &str) -> String {
    let bytes = s.as_bytes();
    if bytes.len() >= 2
        && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\''))
    {
        return s[1..s.len() - 1].to_string();
    }
    s.to_string()
}
