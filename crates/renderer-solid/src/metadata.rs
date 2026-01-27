use serde_json::Value;

pub fn extract_frontmatter(v: &Value) -> Option<String> {
    if v.get("type")?.as_str()? != "Document" {
        return None;
    }
    let children = v.get("children")?.as_array()?;
    for ch in children {
        if ch.get("type").and_then(|t| t.as_str()) == Some("Frontmatter") {
            if let Some(attrs) = ch.get("attrs").and_then(|a| a.as_object()) {
                if let Some(data) = attrs.get("data").and_then(|d| d.as_str()) {
                    return Some(data.to_string());
                }
            }
        }
    }
    None
}

pub fn extract_headings(v: &Value) -> Option<String> {
    if v.get("type")?.as_str()? != "Document" {
        return None;
    }
    let children = v.get("children")?.as_array()?;
    for ch in children {
        if ch.get("type").and_then(|t| t.as_str()) == Some("Headings") {
            if let Some(attrs) = ch.get("attrs").and_then(|a| a.as_object()) {
                if let Some(data) = attrs.get("data").and_then(|d| d.as_str()) {
                    return Some(data.to_string());
                }
            }
        }
    }
    None
}
