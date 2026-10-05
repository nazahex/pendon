use pendon_core::{Event, NodeKind};
use pendon_extra::{
    emit_attr_warnings, parse_extras, to_attributes, warning_event, ExtrasAttr, ExtrasHead,
    ExtrasMatch, ExtrasOptions, ExtrasWarning,
};

use crate::options::WikiOptions;
use crate::util::capitalize_first;

/// Keys the plugin produces itself; extras must not override them (§7.5).
const WIKI_OWNED_KEYS: [&str; 2] = ["href", "title"];

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
        if let Some(end_rel) = text[after_open..].find("]]") {
            let end = after_open + end_rel;
            let raw = text[after_open..end].trim();
            if let Some(link) = parse_wikilink(raw, options) {
                // §7.5: an adjacent `@@type{…}` head attaches to the `<a>`.
                let (extras, consumed) = scan_extras(&text[end + 2..]);
                out.push(Event::StartNode(NodeKind::Link));
                out.push(Event::Attribute {
                    name: "href".to_string(),
                    value: link.href,
                });
                out.push(Event::Attribute {
                    name: "title".to_string(),
                    value: link.title,
                });
                emit_wikilink_extras(extras.as_ref(), out);
                out.push(Event::Text(link.label));
                out.push(Event::EndNode(NodeKind::Link));
                cursor = end + 2 + consumed;
            } else {
                out.push(Event::Text(text[start..end + 2].to_string()));
                cursor = end + 2;
            }
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

/// Scans the `@@type{…}` head that may follow `]]` (§7.5).
///
/// Returns the parsed head (when the text really is a head) and how many bytes
/// of `tail` it consumed, so a malformed head stays literal text (§4.3).
fn scan_extras(tail: &str) -> (Option<ExtrasHead>, usize) {
    match parse_extras(tail) {
        ExtrasMatch::Head { head, rest } => (Some(head), tail.len() - rest.len()),
        ExtrasMatch::Absent { .. } | ExtrasMatch::Malformed { .. } => (None, 0),
    }
}

/// Emits the extra attributes of a wikilink. `href` and `title` are produced by
/// the plugin and are reported when extras try to override them (§7.5).
fn emit_wikilink_extras(head: Option<&ExtrasHead>, out: &mut Vec<Event>) {
    let Some(head) = head else {
        return;
    };
    let attrs = to_attributes(head, &ExtrasOptions::default());
    emit_attr_warnings("wiki", &attrs, out);
    for (key, value) in &attrs.items {
        if WIKI_OWNED_KEYS.contains(&key.as_str()) {
            out.push(warning_event(
                "wiki",
                &ExtrasWarning::OwnedKeyIgnored { key: key.clone() },
            ));
            continue;
        }
        match value {
            ExtrasAttr::Flag => out.push(Event::AttributeFlag { name: key.clone() }),
            ExtrasAttr::Value(value) => out.push(Event::Attribute {
                name: key.clone(),
                value: value.literal(),
            }),
        }
    }
}

pub(crate) fn rewrite_wikilink_markdown(input: &str, options: &WikiOptions) -> String {
    let mut out = String::new();
    let mut cursor = 0usize;
    while let Some(start_rel) = input[cursor..].find("[[") {
        let start = cursor + start_rel;
        out.push_str(&input[cursor..start]);
        let after_open = start + 2;
        if let Some(end_rel) = input[after_open..].find("]]") {
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
            ..Default::default()
        };
        let link = parse_wikilink("Anim Esta", &opts).unwrap();
        assert_eq!(link.href, "/id/wiki/Anim_Esta");
    }

    // --- §7.5 extras head -------------------------------------------------

    fn events_of(text: &str) -> Vec<Event> {
        let mut out = Vec::new();
        emit_wikilink_text(text, &mut out, &WikiOptions::default());
        out
    }

    fn attr(events: &[Event], name: &str) -> Option<String> {
        events.iter().find_map(|event| match event {
            Event::Attribute { name: n, value } if n == name => Some(value.clone()),
            _ => None,
        })
    }

    fn text_of(events: &[Event]) -> String {
        events
            .iter()
            .filter_map(|event| match event {
                Event::Text(text) => Some(text.as_str()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn extras_attach_to_the_wikilink() {
        let out = events_of("[[Anim Esta|Anim]]@@anchor{.wiki,#w,isFoo} trailing");
        assert_eq!(attr(&out, "href").as_deref(), Some("/Anim_Esta"));
        assert_eq!(attr(&out, "class").as_deref(), Some("wiki"));
        assert_eq!(attr(&out, "id").as_deref(), Some("w"));
        assert!(out
            .iter()
            .any(|event| matches!(event, Event::AttributeFlag { name } if name == "isFoo")));
        // The head is consumed and the label survives.
        assert_eq!(text_of(&out), "Anim trailing");
    }

    #[test]
    fn href_is_not_overridable_by_extras() {
        let out = events_of("[[Anim Esta]]@@anchor{href: \"/evil\"}");
        assert_eq!(attr(&out, "href").as_deref(), Some("/Anim_Esta"));
        assert!(out.iter().any(|event| matches!(
            event,
            Event::Diagnostic { message, .. } if message.contains("`href`")
        )));
    }

    #[test]
    fn malformed_extras_stay_literal_text() {
        let out = events_of("[[Anim Esta]]@@anchor{`unterminated}");
        assert_eq!(attr(&out, "href").as_deref(), Some("/Anim_Esta"));
        assert!(text_of(&out).contains("@@anchor{"), "{}", text_of(&out));
    }

    #[test]
    fn extras_do_not_break_a_following_wikilink() {
        let out = events_of("[[A]]@@anchor{.x} [[B]]");
        assert_eq!(
            out.iter()
                .filter(|event| matches!(event, Event::StartNode(NodeKind::Link)))
                .count(),
            2
        );
        assert_eq!(text_of(&out), "A B");
    }
}
