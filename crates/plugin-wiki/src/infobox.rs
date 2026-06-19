use pendon_core::{parse, Event, NodeKind, Options};
use pendon_plugin_markdown::process as process_markdown;
use regex::Regex;

use crate::options::WikiOptions;
use crate::util::{collect_text, escape_html, find_matching_end, normalize_class_tokens};
use crate::wikilink::rewrite_wikilink_markdown;

pub(crate) fn process_infobox(events: &[Event], options: &WikiOptions) -> Vec<Event> {
    let mut out: Vec<Event> = Vec::with_capacity(events.len());
    let mut i = 0usize;

    while i < events.len() {
        if matches!(events.get(i), Some(Event::StartNode(NodeKind::Paragraph))) {
            if let Some(start_end) = find_matching_end(events, i, NodeKind::Paragraph) {
                let marker = collect_text(&events[i + 1..start_end]);
                if let Some(classes) = parse_infobox_start(marker.trim()) {
                    if let Some((close_start, close_end)) =
                        find_infobox_close(events, start_end + 1)
                    {
                        let inner = &events[start_end + 1..close_start];
                        let html = render_infobox(inner, classes.as_deref(), options);
                        out.push(Event::StartNode(NodeKind::HtmlBlock));
                        out.push(Event::Text(html));
                        out.push(Event::EndNode(NodeKind::HtmlBlock));
                        i = close_end + 1;
                        continue;
                    }
                }

                out.extend(events[i..=start_end].iter().cloned());
                i = start_end + 1;
                continue;
            }
        }

        out.push(events[i].clone());
        i += 1;
    }

    out
}

fn parse_infobox_start(s: &str) -> Option<Option<String>> {
    let re = Regex::new(r"^:::infobox(?:\[(?P<class>[^\]]+)\])?\s*$").ok()?;
    let caps = re.captures(s)?;
    let class = caps
        .name("class")
        .map(|m| normalize_class_tokens(m.as_str()));
    Some(class)
}

fn find_infobox_close(events: &[Event], from: usize) -> Option<(usize, usize)> {
    let mut i = from;
    while i < events.len() {
        if matches!(events.get(i), Some(Event::StartNode(NodeKind::Paragraph))) {
            let end = find_matching_end(events, i, NodeKind::Paragraph)?;
            let text = collect_text(&events[i + 1..end]);
            if text.trim() == ":::" {
                return Some((i, end));
            }
            i = end + 1;
            continue;
        }
        i += 1;
    }
    None
}

fn render_infobox(inner: &[Event], class_name: Option<&str>, options: &WikiOptions) -> String {
    let mut html = String::new();
    html.push_str("<aside class=\"infobox");
    if let Some(cls) = class_name {
        if !cls.is_empty() {
            html.push(' ');
            html.push_str(&escape_html(cls));
        }
    }
    html.push_str("\">\n  <dl>");

    let mut i = 0usize;
    while i < inner.len() {
        if !matches!(inner.get(i), Some(Event::StartNode(NodeKind::Paragraph))) {
            i += 1;
            continue;
        }
        let para_end = match find_matching_end(inner, i, NodeKind::Paragraph) {
            Some(v) => v,
            None => break,
        };
        let raw = collect_text(&inner[i + 1..para_end]);
        let trimmed = raw.trim();

        if trimmed.is_empty() {
            i = para_end + 1;
            continue;
        }

        if let Some((div_classes, inline_body)) = parse_inline_div_block(&raw) {
            let rendered = render_fragment(&rewrite_wikilink_markdown(&inline_body, options));
            html.push_str("\n    <dd");
            html.push_str(&full_class_attr(Some(&div_classes)));
            html.push('>');
            html.push_str(&strip_single_paragraph_wrapper(rendered.trim()));
            html.push_str("</dd>");
            i = para_end + 1;
            continue;
        }

        if let Some((div_classes, block_end)) = parse_div_block(inner, i) {
            let body = collect_div_block_text(inner, para_end + 1, block_end.0);
            let rendered = render_fragment(&rewrite_wikilink_markdown(&body, options));
            html.push_str("\n    <dd");
            html.push_str(&full_class_attr(Some(&div_classes)));
            html.push('>');
            html.push_str(&strip_single_paragraph_wrapper(rendered.trim()));
            html.push_str("</dd>");
            i = block_end.1 + 1;
            continue;
        }

        if let Some((level, heading)) = parse_heading(trimmed) {
            html.push_str("\n    <dt");
            html.push_str(&full_class_attr(Some(&format!("h{}", level))));
            html.push('>');
            html.push_str(&strip_single_paragraph_wrapper(
                render_fragment(&rewrite_wikilink_markdown(heading, options)).trim(),
            ));
            html.push_str("</dt>");
            i = para_end + 1;
            continue;
        }

        let mut handled_pairs = false;
        for line in raw.lines() {
            let l = line.trim();
            if l.is_empty() {
                continue;
            }
            if let Some((left, right)) = l.split_once('=') {
                handled_pairs = true;
                let key = left.trim();
                let (classes, value) = parse_class_prefix(right.trim());

                html.push_str("\n    <dt");
                if !classes.is_empty() {
                    html.push_str(" class=\"");
                    html.push_str(&escape_html(&classes));
                    html.push('"');
                }
                html.push('>');
                html.push_str(&strip_single_paragraph_wrapper(
                    render_fragment(&rewrite_wikilink_markdown(key, options)).trim(),
                ));
                html.push_str("</dt>");

                html.push_str("\n    <dd");
                if !classes.is_empty() {
                    html.push_str(" class=\"");
                    html.push_str(&escape_html(&classes));
                    html.push('"');
                }
                html.push('>');
                html.push_str(&strip_single_paragraph_wrapper(
                    render_fragment(&rewrite_wikilink_markdown(value, options)).trim(),
                ));
                html.push_str("</dd>");
            }
        }

        if !handled_pairs {
            html.push_str("\n    <dd class=\"full\">");
            html.push_str(&strip_single_paragraph_wrapper(
                render_fragment(&rewrite_wikilink_markdown(trimmed, options)).trim(),
            ));
            html.push_str("</dd>");
        }

        i = para_end + 1;
    }

    html.push_str("\n  </dl>\n</aside>");
    html
}

fn full_class_attr(extra: Option<&str>) -> String {
    let mut classes: Vec<&str> = vec!["full"];
    if let Some(extra) = extra {
        for token in extra.split_whitespace() {
            if !token.is_empty() {
                classes.push(token);
            }
        }
    }

    format!(" class=\"{}\"", escape_html(&classes.join(" ")))
}

fn parse_div_block(inner: &[Event], start_para: usize) -> Option<(String, (usize, usize))> {
    let start_end = find_matching_end(inner, start_para, NodeKind::Paragraph)?;
    let marker = collect_text(&inner[start_para + 1..start_end]);
    let marker = marker.trim();
    if !marker.starts_with("::[") || !marker.ends_with(']') {
        return None;
    }

    let class_raw = &marker[3..marker.len() - 1];
    let classes = normalize_class_tokens(class_raw);

    let mut i = start_end + 1;
    while i < inner.len() {
        if matches!(inner.get(i), Some(Event::StartNode(NodeKind::Paragraph))) {
            let end = find_matching_end(inner, i, NodeKind::Paragraph)?;
            let t = collect_text(&inner[i + 1..end]);
            if t.trim() == "::" {
                return Some((classes, (i, end)));
            }
            i = end + 1;
            continue;
        }
        i += 1;
    }

    None
}

fn parse_inline_div_block(raw: &str) -> Option<(String, String)> {
    let trimmed = raw.trim();
    if trimmed == "::" || trimmed == ":::" || trimmed.starts_with(":::infobox") {
        return None;
    }

    if let Some(rest) = trimmed.strip_prefix("::[") {
        let close = rest.find(']')?;
        let class_raw = &rest[..close];
        let body_with_close = rest[close + 1..].trim();
        if let Some(body) = body_with_close.strip_suffix("::") {
            return Some((normalize_class_tokens(class_raw), body.trim().to_string()));
        }
    }

    if let Some(body) = trimmed
        .strip_prefix("::")
        .and_then(|s| s.strip_suffix("::"))
    {
        return Some((String::new(), body.trim().to_string()));
    }

    None
}

fn collect_div_block_text(inner: &[Event], from: usize, until_para_start: usize) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut i = from;
    while i < until_para_start {
        if matches!(inner.get(i), Some(Event::StartNode(NodeKind::Paragraph))) {
            if let Some(end) = find_matching_end(inner, i, NodeKind::Paragraph) {
                let t = collect_text(&inner[i + 1..end]);
                if !t.trim().is_empty() {
                    parts.push(t.trim().to_string());
                }
                i = end + 1;
                continue;
            }
        }
        i += 1;
    }
    parts.join("\n\n")
}

fn parse_heading(s: &str) -> Option<(usize, &str)> {
    let hashes = s.chars().take_while(|c| *c == '#').count();
    if (2..=6).contains(&hashes) {
        let rest = s.get(hashes..)?.trim_start();
        if !rest.is_empty() {
            return Some((hashes, rest));
        }
    }
    None
}

fn parse_class_prefix(s: &str) -> (String, &str) {
    let t = s.trim_start();
    if !(t.starts_with("[.") || t.starts_with("[#")) {
        return (String::new(), t);
    }
    if let Some(close) = t.find(']') {
        let cls = normalize_class_tokens(&t[1..close]);
        let rest = t[close + 1..].trim_start();
        return (cls, rest);
    }
    (String::new(), t)
}

fn render_fragment(input: &str) -> String {
    if input.trim().is_empty() {
        return String::new();
    }
    let events = parse(input, &Options::default());
    let markdown = process_markdown(&events);
    pendon_renderer_html::render_html(&markdown)
}

fn strip_single_paragraph_wrapper(html: &str) -> String {
    if let Some(inner) = html
        .strip_prefix("<p>")
        .and_then(|s| s.strip_suffix("</p>"))
    {
        return inner.trim().to_string();
    }
    html.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_inline_div_block_with_classes() {
        let raw = "::[.img]\n![Some Image](https://example.com/a.png)\n::";
        let (classes, body) = parse_inline_div_block(raw).unwrap();
        assert_eq!(classes, "img");
        assert_eq!(body, "![Some Image](https://example.com/a.png)");
    }

    #[test]
    fn parses_inline_div_block_without_classes() {
        let raw = "::\nSome **text**\n::";
        let (classes, body) = parse_inline_div_block(raw).unwrap();
        assert!(classes.is_empty());
        assert_eq!(body, "Some **text**");
    }

    #[test]
    fn parses_inline_div_block_when_newlines_are_collapsed() {
        let raw = "::[.img]![Some Image](https://example.com/a.png)::";
        let (classes, body) = parse_inline_div_block(raw).unwrap();
        assert_eq!(classes, "img");
        assert_eq!(body, "![Some Image](https://example.com/a.png)");
    }

    #[test]
    fn class_prefix_is_detected_for_dot_notation() {
        let (classes, value) = parse_class_prefix("[.foo,.bar] Baz");
        assert_eq!(classes, "foo bar");
        assert_eq!(value, "Baz");
    }

    #[test]
    fn wikilink_value_is_not_treated_as_class_prefix() {
        let (classes, value) = parse_class_prefix("[[Baz]]");
        assert!(classes.is_empty());
        assert_eq!(value, "[[Baz]]");
    }
}
