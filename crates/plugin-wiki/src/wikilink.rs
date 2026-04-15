use pendon_core::{Event, NodeKind};

use crate::options::WikiOptions;
use crate::util::capitalize_first;

#[derive(Debug)]
pub(crate) struct WikiLink {
    pub(crate) href: String,
    pub(crate) title: String,
    pub(crate) label: String,
}

pub(crate) fn process_wikilinks(events: &[Event], options: &WikiOptions) -> Vec<Event> {
    let mut out: Vec<Event> = Vec::with_capacity(events.len());
    let mut html_depth = 0usize;

    for ev in events {
        match ev {
            Event::StartNode(NodeKind::HtmlBlock) | Event::StartNode(NodeKind::HtmlInline) => {
                html_depth += 1;
                out.push(ev.clone());
            }
            Event::EndNode(NodeKind::HtmlBlock) | Event::EndNode(NodeKind::HtmlInline) => {
                html_depth = html_depth.saturating_sub(1);
                out.push(ev.clone());
            }
            Event::Text(text) if html_depth == 0 => emit_wikilink_text(text, &mut out, options),
            _ => out.push(ev.clone()),
        }
    }

    out
}

pub(crate) fn emit_wikilink_text(text: &str, out: &mut Vec<Event>, options: &WikiOptions) {
    let mut cursor = 0usize;
    while let Some(start_rel) = text[cursor..].find("[[") {
        let start = cursor + start_rel;
        if start > cursor {
            out.push(Event::Text(text[cursor..start].to_string()));
        }
        let after_open = start + 2;
        if let Some(end_rel) = text[after_open..].find("]]" ) {
            let end = after_open + end_rel;
            let raw = text[after_open..end].trim();
            if let Some(link) = parse_wikilink(raw, options) {
                out.push(Event::StartNode(NodeKind::Link));
                out.push(Event::Attribute {
                    name: "href".to_string(),
                    value: link.href,
                });
                out.push(Event::Attribute {
                    name: "title".to_string(),
                    value: link.title,
                });
                out.push(Event::Text(link.label));
                out.push(Event::EndNode(NodeKind::Link));
            } else {
                out.push(Event::Text(text[start..end + 2].to_string()));
            }
            cursor = end + 2;
        } else {
            out.push(Event::Text(text[start..].to_string()));
            cursor = text.len();
            break;
        }
    }

    if cursor < text.len() {
        out.push(Event::Text(text[cursor..].to_string()));
    }
}

pub(crate) fn rewrite_wikilink_markdown(input: &str, options: &WikiOptions) -> String {
    let mut out = String::new();
    let mut cursor = 0usize;
    while let Some(start_rel) = input[cursor..].find("[[") {
        let start = cursor + start_rel;
        out.push_str(&input[cursor..start]);
        let after_open = start + 2;
        if let Some(end_rel) = input[after_open..].find("]]" ) {
            let end = after_open + end_rel;
            let raw = input[after_open..end].trim();
            if let Some(link) = parse_wikilink(raw, options) {
                out.push('[');
                out.push_str(&link.label);
                out.push_str("](");
                out.push_str(&link.href);
                out.push_str(" \"");
                out.push_str(&link.title);
                out.push_str("\")");
            } else {
                out.push_str(&input[start..end + 2]);
            }
            cursor = end + 2;
        } else {
            out.push_str(&input[start..]);
            cursor = input.len();
            break;
        }
    }
    if cursor < input.len() {
        out.push_str(&input[cursor..]);
    }
    out
}

pub(crate) fn parse_wikilink(raw: &str, options: &WikiOptions) -> Option<WikiLink> {
    if raw.is_empty() {
        return None;
    }
    let (target_raw, label_raw) = match raw.split_once('|') {
        Some((a, b)) => (a.trim(), Some(b.trim())),
        None => (raw.trim(), None),
    };

    if target_raw.is_empty() {
        return None;
    }

    let canonical = capitalize_first(target_raw);
    let slug = canonical.replace(' ', "_");

    let label = match label_raw {
        Some(label) if !label.is_empty() => label.to_string(),
        _ => target_raw.to_string(),
    };

    Some(WikiLink {
        href: build_wiki_href(&slug, options),
        title: canonical,
        label,
    })
}

fn build_wiki_href(slug: &str, options: &WikiOptions) -> String {
    let prefix = options
        .link_prefix
        .as_deref()
        .map(str::trim)
        .unwrap_or("")
        .trim_end_matches('/');

    if prefix.is_empty() {
        return format!("/{}", slug);
    }

    let normalized = if prefix.starts_with('/') {
        prefix.to_string()
    } else {
        format!("/{}", prefix)
    };
    format!("{}/{}", normalized, slug)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_wikilink_basic() {
        let link = parse_wikilink("Anim Esta", &WikiOptions::default()).unwrap();
        assert_eq!(link.href, "/Anim_Esta");
        assert_eq!(link.title, "Anim Esta");
        assert_eq!(link.label, "Anim Esta");
    }

    #[test]
    fn parses_wikilink_with_alias_and_parenthetical() {
        let link = parse_wikilink("Anim Esta (Officia) | Anim", &WikiOptions::default()).unwrap();
        assert_eq!(link.href, "/Anim_Esta_(Officia)");
        assert_eq!(link.title, "Anim Esta (Officia)");
        assert_eq!(link.label, "Anim");
    }

    #[test]
    fn rewrites_text_with_wikilink_to_link_events() {
        let mut out = Vec::new();
        emit_wikilink_text("Nisi [[Anim Esta]] id", &mut out, &WikiOptions::default());
        assert!(out
            .iter()
            .any(|ev| matches!(ev, Event::StartNode(NodeKind::Link))));
    }

    #[test]
    fn applies_wiki_link_prefix() {
        let opts = WikiOptions {
            link_prefix: Some("/id/wiki".to_string()),
        };
        let link = parse_wikilink("Anim Esta", &opts).unwrap();
        assert_eq!(link.href, "/id/wiki/Anim_Esta");
    }
}
