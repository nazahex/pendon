use pendon_core::{Event, NodeKind};
use std::collections::HashMap;

pub fn extract_charmap(events: &[Event]) -> HashMap<String, String> {
    let mut in_frontmatter = false;
    for ev in events {
        match ev {
            Event::StartNode(NodeKind::Frontmatter) => in_frontmatter = true,
            Event::EndNode(NodeKind::Frontmatter) => break,
            Event::Attribute { name, value } if in_frontmatter && name == "data" => {
                return parse_charmap_json(value);
            }
            _ => {}
        }
    }
    HashMap::new()
}

pub fn parse_charmap_json(data: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let Ok(v) = serde_json::from_str::<serde_json::Value>(data) else {
        return map;
    };
    let Some(arr) = v.get("charmap").and_then(|c| c.as_array()) else {
        return map;
    };

    let mut i = 0usize;
    while i + 1 < arr.len() {
        let Some(name) = arr[i].as_str() else {
            i += 2;
            continue;
        };
        let Some(class_name) = arr[i + 1].as_str() else {
            i += 2;
            continue;
        };
        map.insert(name.to_string(), class_name.to_string());
        i += 2;
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_charmap_pairs() {
        let m = parse_charmap_json(r#"{"charmap":["A","x","B","y"]}"#);
        assert_eq!(m.get("A").map(String::as_str), Some("x"));
        assert_eq!(m.get("B").map(String::as_str), Some("y"));
    }
}
