use pendon_core::{parse, Event, NodeKind, Options};
use pendon_plugin_markdown::process as process_markdown;

pub fn process(events: &[Event]) -> Vec<Event> {
    let mut out: Vec<Event> = Vec::with_capacity(events.len());
    let mut i = 0usize;

    while i < events.len() {
        if matches!(events.get(i), Some(Event::StartNode(NodeKind::Paragraph))) {
            if let Some(end) = find_matching_end(events, i, NodeKind::Paragraph) {
                let block = &events[i + 1..end];
                if let Some(html) = maybe_render_advanced_image(block) {
                    out.push(Event::StartNode(NodeKind::HtmlBlock));
                    out.push(Event::Text(html));
                    out.push(Event::EndNode(NodeKind::HtmlBlock));
                    i = end + 1;
                    continue;
                }
                out.extend(events[i..=end].iter().cloned());
                i = end + 1;
                continue;
            }
        }

        out.push(events[i].clone());
        i += 1;
    }

    out
}

fn maybe_render_advanced_image(block_events: &[Event]) -> Option<String> {
    let raw = collect_text_only(block_events)?;
    let line = raw.trim();
    if line.is_empty() || line.contains('\n') {
        return None;
    }

    if let Some(html) = render_figure_syntax(line) {
        return Some(html);
    }

    if let Some(html) = render_decorated_image_syntax(line) {
        return Some(html);
    }

    None
}

fn render_figure_syntax(line: &str) -> Option<String> {
    let core = parse_image_core(line)?;
    if core.marker.container != Some(ContainerKind::Figure) {
        return None;
    }
    let (attrs, rest, _had_attrs) = parse_optional_attrs(core.rest);
    let caption = rest.trim();

    let mut out = String::new();
    out.push_str("<figure");
    push_common_attrs(&mut out, &attrs);
    out.push('>');
    out.push_str("<img");
    push_image_marker_attrs(&mut out, &core.marker);
    out.push_str(" alt=\"");
    escape_html(&core.alt, &mut out);
    out.push_str("\" src=\"");
    escape_html(&core.src, &mut out);
    out.push_str("\" />");

    if !caption.is_empty() {
        let caption_html = render_inline_fragment(caption);
        if !caption_html.is_empty() {
            out.push_str("<figcaption>");
            out.push_str(&caption_html);
            out.push_str("</figcaption>");
        }
    }

    out.push_str("</figure>");
    Some(out)
}

fn render_decorated_image_syntax(line: &str) -> Option<String> {
    let core = parse_image_core(line)?;
    if core.marker.container == Some(ContainerKind::Figure) {
        return None;
    }

    if let Some(container) = core.marker.container {
        let (attrs, rest, _had_attrs) = parse_optional_attrs(core.rest);
        if !rest.trim().is_empty() {
            return None;
        }

        let tag = match container {
            ContainerKind::Paragraph => "p",
            ContainerKind::Division => "div",
            ContainerKind::Figure => return None,
        };

        let mut out = String::new();
        out.push('<');
        out.push_str(tag);
        push_common_attrs(&mut out, &attrs);
        out.push('>');
        out.push_str("<img");
        push_image_marker_attrs(&mut out, &core.marker);
        out.push_str(" alt=\"");
        escape_html(&core.alt, &mut out);
        out.push_str("\" src=\"");
        escape_html(&core.src, &mut out);
        out.push_str("\" />");
        out.push_str("</");
        out.push_str(tag);
        out.push('>');
        return Some(out);
    }

    let (attrs, rest, had_attrs) = parse_optional_attrs(core.rest);
    let has_marker_mod = core.marker.has_modifiers();
    if !has_marker_mod && (!had_attrs || !rest.trim().is_empty()) {
        return None;
    }
    if has_marker_mod && !rest.trim().is_empty() {
        return None;
    }

    let mut out = String::new();
    out.push_str("<img");
    push_image_marker_attrs(&mut out, &core.marker);
    out.push_str(" alt=\"");
    escape_html(&core.alt, &mut out);
    out.push_str("\"");
    push_common_attrs(&mut out, &attrs);
    out.push_str(" src=\"");
    escape_html(&core.src, &mut out);
    out.push_str("\" />");
    Some(out)
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct AttrSpec {
    id: Option<String>,
    classes: Vec<String>,
    data: Vec<(String, String)>,
    styles: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
struct ImageCore<'a> {
    alt: String,
    src: String,
    rest: &'a str,
    marker: ImageMarker,
}

#[derive(Debug, Clone, Copy, Default)]
struct ImageMarker {
    container: Option<ContainerKind>,
    lazy: bool,
    async_decoding: bool,
    width: Option<usize>,
    height: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContainerKind {
    Figure,
    Paragraph,
    Division,
}

impl ImageMarker {
    fn has_modifiers(self) -> bool {
        self.lazy || self.async_decoding || self.width.is_some() || self.height.is_some()
    }
}

fn parse_image_core<'a>(line: &'a str) -> Option<ImageCore<'a>> {
    let open_br = line.find('[')?;
    let marker_raw = line[..open_br].trim();
    let marker = parse_marker(marker_raw)?;

    let mut idx = open_br;
    if line.get(idx..=idx)? != "[" {
        return None;
    }
    let close_br_rel = line[idx + 1..].find(']')?;
    let close_br = idx + 1 + close_br_rel;
    let alt = line[idx + 1..close_br].to_string();

    idx = close_br + 1;
    if line.get(idx..=idx)? != "(" {
        return None;
    }
    let close_par_rel = line[idx + 1..].find(')')?;
    let close_par = idx + 1 + close_par_rel;
    let src = line[idx + 1..close_par].to_string();

    let rest = line.get(close_par + 1..).unwrap_or("");
    Some(ImageCore {
        alt,
        src,
        rest,
        marker,
    })
}

fn parse_marker(raw: &str) -> Option<ImageMarker> {
    let (explicit_container, marker_raw) = if let Some(rest) = raw.strip_prefix('p') {
        (Some(ContainerKind::Paragraph), rest)
    } else if let Some(rest) = raw.strip_prefix('d') {
        (Some(ContainerKind::Division), rest)
    } else {
        (None, raw)
    };

    if marker_raw.is_empty() || !marker_raw.contains('!') {
        return None;
    }

    let is_figure = marker_raw.contains("!!");
    if explicit_container.is_some() && is_figure {
        return None;
    }

    let mut marker = ImageMarker {
        container: if is_figure {
            Some(ContainerKind::Figure)
        } else {
            explicit_container
        },
        lazy: marker_raw.contains('?'),
        async_decoding: marker_raw.contains('~'),
        width: None,
        height: None,
    };

    let bytes = marker_raw.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] as char {
            '!' | '?' | '~' => {
                i += 1;
            }
            'w' | 'h' => {
                let key = bytes[i] as char;
                i += 1;
                let start = i;
                while i < bytes.len() && (bytes[i] as char).is_ascii_digit() {
                    i += 1;
                }
                if i == start {
                    return None;
                }
                let num = marker_raw[start..i].parse::<usize>().ok()?;
                if key == 'w' {
                    marker.width = Some(num);
                } else {
                    marker.height = Some(num);
                }
            }
            _ => return None,
        }
    }

    Some(marker)
}

fn parse_optional_attrs(input: &str) -> (AttrSpec, &str, bool) {
    let s = input.trim_start();
    let (class_block, after_class, had_class) = if s.starts_with('[') {
        let close_br = match s.find(']') {
            Some(v) => v,
            None => return (AttrSpec::default(), s, false),
        };
        let class_block = &s[1..close_br];
        let after_br = s.get(close_br + 1..).unwrap_or("");
        (Some(class_block), after_br, true)
    } else {
        (None, s, false)
    };

    let after_class = after_class.trim_start();
    if !after_class.starts_with('{') {
        return (AttrSpec::default(), s, false);
    }
    let close_curly = match after_class.find('}') {
        Some(v) => v,
        None => return (AttrSpec::default(), s, false),
    };

    let kv_block = &after_class[1..close_curly];
    let rest = after_class.get(close_curly + 1..).unwrap_or("");

    let mut spec = AttrSpec::default();

    if let Some(class_block) = class_block {
        for token in class_block.split(',').map(|t| t.trim()).filter(|t| !t.is_empty()) {
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
    }

    for pair in split_csv(kv_block) {
        let Some((k, v)) = pair.split_once(':') else {
            continue;
        };
        let key = k.trim();
        let value = unquote(v.trim());
        if key.is_empty() {
            continue;
        }
        if key.starts_with("--") {
            spec.styles.push((key.to_string(), value));
        } else {
            spec.data.push((key.to_string(), value));
        }
    }

    (spec, rest, had_class || !kv_block.trim().is_empty())
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

fn push_common_attrs(out: &mut String, attrs: &AttrSpec) {
    if let Some(id) = attrs.id.as_deref() {
        out.push_str(" id=\"");
        escape_html(id, out);
        out.push_str("\"");
    }

    if !attrs.classes.is_empty() {
        out.push_str(" class=\"");
        escape_html(&attrs.classes.join(" "), out);
        out.push_str("\"");
    }

    for (k, v) in &attrs.data {
        out.push(' ');
        out.push_str("data:");
        escape_html(k, out);
        out.push_str("=\"");
        escape_html(v, out);
        out.push_str("\"");
    }

    if !attrs.styles.is_empty() {
        out.push_str(" style=\"");
        for (k, v) in &attrs.styles {
            escape_html(k, out);
            out.push(':');
            escape_html(v, out);
            out.push(';');
        }
        out.push_str("\"");
    }
}

fn push_image_marker_attrs(out: &mut String, marker: &ImageMarker) {
    if let Some(width) = marker.width {
        out.push_str(" width=\"");
        out.push_str(&width.to_string());
        out.push_str("\"");
    }
    if let Some(height) = marker.height {
        out.push_str(" height=\"");
        out.push_str(&height.to_string());
        out.push_str("\"");
    }
    if marker.async_decoding {
        out.push_str(" decoding=\"async\"");
    }
    if marker.lazy {
        out.push_str(" loading=\"lazy\"");
    }
}

fn render_inline_fragment(input: &str) -> String {
    if input.trim().is_empty() {
        return String::new();
    }
    let parsed = parse(input, &Options::default());
    let rendered = pendon_renderer_html::render_html(&process_markdown(&parsed));
    let trimmed = rendered.trim();
    if let Some(inner) = trimmed.strip_prefix("<p>").and_then(|s| s.strip_suffix("</p>")) {
        return inner.trim().to_string();
    }
    trimmed.to_string()
}

fn collect_text_only(events: &[Event]) -> Option<String> {
    let mut out = String::new();
    for ev in events {
        match ev {
            Event::Text(t) => out.push_str(t),
            _ => return None,
        }
    }
    Some(out)
}

fn find_matching_end(events: &[Event], start_idx: usize, kind: NodeKind) -> Option<usize> {
    let mut depth = 0isize;
    for (idx, ev) in events.iter().enumerate().skip(start_idx) {
        match ev {
            Event::StartNode(k) if *k == kind => depth += 1,
            Event::EndNode(k) if *k == kind => {
                depth -= 1;
                if depth == 0 {
                    return Some(idx);
                }
            }
            _ => {}
        }
    }
    None
}

fn escape_html(s: &str, out: &mut String) {
    for ch in s.chars() {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn paragraph_events(text: &str) -> Vec<Event> {
        vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Paragraph),
            Event::Text(text.to_string()),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ]
    }

    #[test]
    fn renders_figure_with_markdown_caption() {
        let events = paragraph_events(
            "!![Alt](https://x.test/a.webp) Caption **bold** [link](/x)",
        );
        let out = process(&events);
        let html = out
            .iter()
            .find_map(|ev| match ev {
                Event::Text(t) if t.contains("<figure") => Some(t.clone()),
                _ => None,
            })
            .unwrap();

        assert!(html.contains("<figure>"));
        assert!(html.contains("<img alt=\"Alt\" src=\"https://x.test/a.webp\" />"));
        assert!(html.contains("<figcaption>Caption <strong>bold</strong> <a href=\"/x\">link</a></figcaption>"));
    }

    #[test]
    fn renders_decorated_image_attributes() {
        let events = paragraph_events(
            "![Alt](https://x.test/a.webp)[.x,#hero]{foo: \"bar\", --r: \"5deg\"}",
        );
        let out = process(&events);
        let html = out
            .iter()
            .find_map(|ev| match ev {
                Event::Text(t) if t.contains("<img ") => Some(t.clone()),
                _ => None,
            })
            .unwrap();

        assert!(html.contains("id=\"hero\""));
        assert!(html.contains("class=\"x\""));
        assert!(html.contains("data:foo=\"bar\""));
        assert!(html.contains("style=\"--r:5deg;\""));
    }

    #[test]
    fn renders_decorated_image_attributes_without_class_block() {
        let events = paragraph_events("![Alt](https://x.test/a.webp){foo: \"bar\", --r: \"5deg\"}");
        let out = process(&events);
        let html = out
            .iter()
            .find_map(|ev| match ev {
                Event::Text(t) if t.contains("<img ") => Some(t.clone()),
                _ => None,
            })
            .unwrap();

        assert!(html.contains("data:foo=\"bar\""));
        assert!(html.contains("style=\"--r:5deg;\""));
        assert!(!html.contains("figcaption"));
    }

    #[test]
    fn renders_figure_attributes_without_class_block() {
        let events = paragraph_events("!![Alt](https://x.test/a.webp){foo: \"bar\", --r: \"5deg\"}");
        let out = process(&events);
        let html = out
            .iter()
            .find_map(|ev| match ev {
                Event::Text(t) if t.contains("<figure") => Some(t.clone()),
                _ => None,
            })
            .unwrap();

        assert!(html.contains("<figure"));
        assert!(html.contains("data:foo=\"bar\""));
        assert!(html.contains("style=\"--r:5deg;\""));
        assert!(!html.contains("figcaption"));
    }

    #[test]
    fn parses_marker_with_width_height_and_flags() {
        let marker = parse_marker("~?!!w300h800").unwrap();
        assert_eq!(marker.container, Some(ContainerKind::Figure));
        assert!(marker.lazy);
        assert!(marker.async_decoding);
        assert_eq!(marker.width, Some(300));
        assert_eq!(marker.height, Some(800));
    }

    #[test]
    fn parses_marker_with_paragraph_container() {
        let marker = parse_marker("p!w300h800").unwrap();
        assert_eq!(marker.container, Some(ContainerKind::Paragraph));
        assert_eq!(marker.width, Some(300));
        assert_eq!(marker.height, Some(800));
    }

    #[test]
    fn renders_single_image_modifiers_without_attr_block() {
        let events = paragraph_events("!?w320h180[alt](https://x.test/a.webp)");
        let out = process(&events);
        let html = out
            .iter()
            .find_map(|ev| match ev {
                Event::Text(t) if t.contains("<img ") => Some(t.clone()),
                _ => None,
            })
            .unwrap();

        assert!(html.contains("width=\"320\""));
        assert!(html.contains("height=\"180\""));
        assert!(html.contains("loading=\"lazy\""));
        assert!(html.contains("alt=\"alt\""));
    }

    #[test]
    fn renders_figure_with_mixed_modifiers() {
        let events = paragraph_events("~?!![alt](https://x.test/a.webp)");
        let out = process(&events);
        let html = out
            .iter()
            .find_map(|ev| match ev {
                Event::Text(t) if t.contains("<figure") => Some(t.clone()),
                _ => None,
            })
            .unwrap();

        assert!(html.contains("<figure>"));
        assert!(html.contains("decoding=\"async\""));
        assert!(html.contains("loading=\"lazy\""));
    }

    #[test]
    fn renders_paragraph_container_image() {
        let events = paragraph_events("p![foo](https://x.test/a.webp)");
        let out = process(&events);
        let html = out
            .iter()
            .find_map(|ev| match ev {
                Event::Text(t) if t.contains("<p") && t.contains("<img") => Some(t.clone()),
                _ => None,
            })
            .unwrap();

        assert!(html.contains("<p><img alt=\"foo\" src=\"https://x.test/a.webp\" /></p>"));
    }

    #[test]
    fn renders_div_container_with_attrs_and_modifiers() {
        let events = paragraph_events(
            "d~!w300h800[foo](https://x.test/a.webp)[.extra,#hero]{foo: \"bar\", --r: \"5deg\"}",
        );
        let out = process(&events);
        let html = out
            .iter()
            .find_map(|ev| match ev {
                Event::Text(t) if t.contains("<div") && t.contains("<img") => Some(t.clone()),
                _ => None,
            })
            .unwrap();

        assert!(html.contains("<div"));
        assert!(html.contains("id=\"hero\""));
        assert!(html.contains("class=\"extra\""));
        assert!(html.contains("data:foo=\"bar\""));
        assert!(html.contains("style=\"--r:5deg;\""));
        assert!(html.contains("decoding=\"async\""));
        assert!(html.contains("width=\"300\""));
        assert!(html.contains("height=\"800\""));
    }
}
