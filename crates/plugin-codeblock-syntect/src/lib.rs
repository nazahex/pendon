use pendon_core::{Event, NodeKind};

mod highlight;
mod info;
mod inline;
mod mappers;
mod wrap;

#[cfg(test)]
mod tests {
    use super::highlight::highlight_output;
    use super::info::parse_info_string;

    #[test]
    fn line_wrappers_ignore_pre_classes() {
        let info = parse_info_string("js .wrap {1} {3-5}");
        let code = "import { log } from \"console\";\nconst data = [10, 20, null];\nasync function* hitung(a, b = 5) {\n  for (let x of data) {\n    if (x?.val ?? true) yield (a + b) * x;\n  }\n}\n";
        let out = highlight_output(code, &info, None);

        assert!(out.contains("<p class=\"mark\">"));
        assert!(out.contains("<p>const data"));
        assert!(!out.contains("<p class=\"wrap"));
    }
}

use highlight::highlight_output;
use info::parse_info_string;

pub fn process(events: &[Event]) -> Vec<Event> {
    let mut out: Vec<Event> = Vec::with_capacity(events.len());
    let mut i = 0usize;
    while i < events.len() {
        match &events[i] {
            Event::StartNode(NodeKind::CodeFence) => {
                // Collect attrs + inner text until EndNode(CodeFence)
                let mut j = i + 1;
                let mut raw_info: Option<String> = None;
                let mut debug: Option<String> = None;
                let mut inner = String::new();
                while j < events.len() {
                    match &events[j] {
                        Event::Attribute { name, value } => {
                            if name == "lang" {
                                raw_info = Some(value.clone());
                            } else if name == "syntect_debug" {
                                debug = Some(value.clone());
                            }
                        }
                        Event::Text(t) => inner.push_str(t),
                        Event::EndNode(NodeKind::CodeFence) => break,
                        _ => {}
                    }
                    j += 1;
                }
                // Default: pass-through if no end found
                if j >= events.len() {
                    out.push(events[i].clone());
                    i += 1;
                    continue;
                }
                // Resolve debug mode: attribute overrides env var
                let env_debug = std::env::var("PENDON_SYNTECT_DEBUG").ok();
                let debug_mode = debug
                    .as_deref()
                    .map(|s| s.to_string())
                    .or(env_debug)
                    .map(|s| s.to_lowercase());
                let parsed_info = raw_info
                    .as_deref()
                    .map(parse_info_string)
                    .unwrap_or_default();
                let highlighted = highlight_output(&inner, &parsed_info, debug_mode.as_deref());
                out.push(Event::StartNode(NodeKind::CodeFence));
                if let Some(l) = parsed_info
                    .lang
                    .clone()
                    .or_else(|| raw_info.as_ref().cloned())
                {
                    out.push(Event::Attribute {
                        name: "lang".to_string(),
                        value: l,
                    });
                }
                if !parsed_info.pre_classes.is_empty() {
                    out.push(Event::Attribute {
                        name: "class".to_string(),
                        value: parsed_info.pre_classes.join(" "),
                    });
                }
                // Mark as raw HTML payload for downstream renderer
                out.push(Event::Attribute {
                    name: "raw_html".to_string(),
                    value: "1".to_string(),
                });
                out.push(Event::Text(highlighted));
                out.push(Event::EndNode(NodeKind::CodeFence));
                i = j + 1;
            }
            _ => {
                out.push(events[i].clone());
                i += 1;
            }
        }
    }
    out
}
