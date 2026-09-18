use pendon_core::{parse, Event, NodeKind, Options, Pipeline};
use pendon_plugin_markdown::process as process_markdown;
use pendon_renderer_solid::{ComponentTemplate, ImportEntry, SolidRenderHints};
use serde::{Deserialize, Serialize};

// --- Configuration & Types ---

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ImgOptions {
    pub custom_node: Option<ImgCustomNode>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ImgCustomNode {
    pub name: String,
    pub template: String,
    #[serde(default)]
    pub imports: Vec<ImgImport>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImgImport {
    pub module: String,
    pub default: Option<String>,
    #[serde(default)]
    pub names: Vec<String>,
}

// --- Parsed Image Data (shared between HTML and Custom rendering) ---

#[derive(Debug, Clone)]
struct ParsedImage {
    alt: String,
    src: String,
    caption: Option<String>,
    container: Option<ContainerKind>,
    attrs: AttrSpec,
    marker: ImageMarker,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct AttrSpec {
    id: Option<String>,
    classes: Vec<String>,
    data: Vec<(String, String)>,
    styles: Vec<(String, String)>,
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

// --- Main Processor ---

pub fn process(events: &[Event], options: &ImgOptions, inline_pipeline: &Pipeline) -> Vec<Event> {
    let mut out: Vec<Event> = Vec::with_capacity(events.len());
    let mut i = 0usize;

    while i < events.len() {
        if matches!(events.get(i), Some(Event::StartNode(NodeKind::Paragraph))) {
            if let Some(end) = find_matching_end(events, i, NodeKind::Paragraph) {
                let block = &events[i + 1..end];
                if let Some(parsed) = maybe_parse_advanced_image(block) {
                    if options.custom_node.is_some() {
                        emit_custom_image(&parsed, options, inline_pipeline, &mut out);
                    } else {
                        let html = render_html_from_parsed(&parsed, inline_pipeline);
                        out.push(Event::StartNode(NodeKind::HtmlBlock));
                        out.push(Event::Text(html));
                        out.push(Event::EndNode(NodeKind::HtmlBlock));
                    }
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

// --- Custom Node Emission ---

fn emit_custom_image(
    parsed: &ParsedImage,
    options: &ImgOptions,
    inline_pipeline: &Pipeline,
    out: &mut Vec<Event>,
) {
    let custom = match options.custom_node.as_ref() {
        Some(c) => c,
        None => return,
    };

    let node_kind = NodeKind::Custom(custom.name.clone());
    out.push(Event::StartNode(node_kind.clone()));

    // Emit component name for renderer template matching
    out.push(Event::Attribute {
        name: "name".to_string(),
        value: custom.name.clone(),
    });

    // Core image attributes
    out.push(Event::Attribute {
        name: "alt".to_string(),
        value: parsed.alt.clone(),
    });
    out.push(Event::Attribute {
        name: "src".to_string(),
        value: parsed.src.clone(),
    });

    // Container type
    if let Some(container) = parsed.container.or(parsed.marker.container) {
        let value = match container {
            ContainerKind::Figure => "figure",
            ContainerKind::Paragraph => "p",
            ContainerKind::Division => "div",
        };
        out.push(Event::Attribute {
            name: "container".to_string(),
            value: value.to_string(),
        });
    }

    // Marker modifiers
    if parsed.marker.lazy {
        out.push(Event::Attribute {
            name: "lazy".to_string(),
            value: "1".to_string(),
        });
    }
    if parsed.marker.async_decoding {
        out.push(Event::Attribute {
            name: "async_decoding".to_string(),
            value: "1".to_string(),
        });
    }
    if let Some(w) = parsed.marker.width {
        out.push(Event::Attribute {
            name: "width".to_string(),
            value: w.to_string(),
        });
    }
    if let Some(h) = parsed.marker.height {
        out.push(Event::Attribute {
            name: "height".to_string(),
            value: h.to_string(),
        });
    }

    // Extra attributes (id, class, data-*, style)
    if let Some(id) = &parsed.attrs.id {
        out.push(Event::Attribute {
            name: "id".to_string(),
            value: id.clone(),
        });
    }
    if !parsed.attrs.classes.is_empty() {
        out.push(Event::Attribute {
            name: "class".to_string(),
            value: parsed.attrs.classes.join(" "),
        });
    }
    for (k, v) in &parsed.attrs.data {
        out.push(Event::Attribute {
            name: format!("data-{}", k),
            value: v.clone(),
        });
    }
    if !parsed.attrs.styles.is_empty() {
        let style_str: String = parsed
            .attrs
            .styles
            .iter()
            .map(|(k, v)| format!("{}:{};", k, v))
            .collect();
        out.push(Event::Attribute {
            name: "style".to_string(),
            value: style_str,
        });
    }

    // Caption as rendered inline children with full inline pipeline support
    if let Some(caption) = &parsed.caption {
        let caption_events = render_caption_events(caption, inline_pipeline);
        for ev in caption_events {
            out.push(ev);
        }
    }

    out.push(Event::EndNode(node_kind));
}

// --- Solid Hints Integration ---

/// Builds SolidRenderHints for the custom image component when configured.
/// Returns None if no custom_node is defined, letting the renderer fall back
/// to its built-in HtmlBlock handler.
pub fn solid_hints(options: &ImgOptions) -> Option<SolidRenderHints> {
    let custom = options.custom_node.as_ref()?;
    let key = (custom.name.clone(), Some(custom.name.clone()));

    let mut hints = SolidRenderHints::default();
    hints.templates.push(ComponentTemplate {
        node_type: custom.name.clone(),
        node_name: Some(custom.name.clone()),
        template: custom.template.clone(),
    });

    let imports: Vec<ImportEntry> = custom
        .imports
        .iter()
        .map(|imp| ImportEntry::Structured {
            module: imp.module.clone(),
            default: imp.default.clone(),
            names: imp.names.clone(),
        })
        .collect();

    if !imports.is_empty() {
        hints.template_imports.insert(key, imports);
    }

    Some(hints)
}

// --- Unified Parsing (extracts structured data from paragraph text) ---

fn maybe_parse_advanced_image(block_events: &[Event]) -> Option<ParsedImage> {
    let raw = collect_text_only(block_events)?;
    let line = raw.trim();
    if line.is_empty() || line.contains('\n') {
        return None;
    }

    parse_figure_syntax(line).or_else(|| parse_decorated_image_syntax(line))
}

fn parse_figure_syntax(line: &str) -> Option<ParsedImage> {
    let core = parse_image_core(line)?;
    if core.marker.container != Some(ContainerKind::Figure) {
        return None;
    }
    let (attrs, rest, _had_attrs) = parse_optional_attrs(core.rest);
    let caption = rest.trim();

    Some(ParsedImage {
        alt: core.alt,
        src: core.src,
        caption: if caption.is_empty() {
            None
        } else {
            Some(caption.to_string())
        },
        container: Some(ContainerKind::Figure),
        attrs,
        marker: core.marker,
    })
}

fn parse_decorated_image_syntax(line: &str) -> Option<ParsedImage> {
    let core = parse_image_core(line)?;
    if core.marker.container == Some(ContainerKind::Figure) {
        return None;
    }

    if let Some(container) = core.marker.container {
        let (attrs, rest, _had_attrs) = parse_optional_attrs(core.rest);
        if !rest.trim().is_empty() {
            return None;
        }
        return Some(ParsedImage {
            alt: core.alt,
            src: core.src,
            caption: None,
            container: Some(container),
            attrs,
            marker: core.marker,
        });
    }

    let (attrs, rest, had_attrs) = parse_optional_attrs(core.rest);
    let has_marker_mod = core.marker.has_modifiers();
    if !has_marker_mod && (!had_attrs || !rest.trim().is_empty()) {
        return None;
    }
    if has_marker_mod && !rest.trim().is_empty() {
        return None;
    }

    Some(ParsedImage {
        alt: core.alt,
        src: core.src,
        caption: None,
        container: None,
        attrs,
        marker: core.marker,
    })
}

// --- HTML Rendering (from parsed data, used when no custom node) ---

fn render_html_from_parsed(parsed: &ParsedImage, inline_pipeline: &Pipeline) -> String {
    let container = parsed.container.or(parsed.marker.container);
    let mut out = String::new();

    match container {
        Some(ContainerKind::Figure) => {
            out.push_str("<figure");
            push_common_attrs(&mut out, &parsed.attrs);
            out.push('>');
            out.push_str("<img");
            push_image_marker_attrs(&mut out, &parsed.marker);
            out.push_str(" alt=\"");
            escape_html(&parsed.alt, &mut out);
            out.push_str("\" src=\"");
            escape_html(&parsed.src, &mut out);
            out.push_str("\" />");
            if let Some(caption) = &parsed.caption {
                let caption_html = render_caption_html(caption, inline_pipeline);
                if !caption_html.is_empty() {
                    out.push_str("<figcaption>");
                    out.push_str(&caption_html);
                    out.push_str("</figcaption>");
                }
            }
            out.push_str("</figure>");
        }
        Some(ContainerKind::Paragraph) | Some(ContainerKind::Division) => {
            let tag = match container.unwrap() {
                ContainerKind::Paragraph => "p",
                ContainerKind::Division => "div",
                _ => unreachable!(),
            };
            out.push('<');
            out.push_str(tag);
            push_common_attrs(&mut out, &parsed.attrs);
            out.push('>');
            out.push_str("<img");
            push_image_marker_attrs(&mut out, &parsed.marker);
            out.push_str(" alt=\"");
            escape_html(&parsed.alt, &mut out);
            out.push_str("\" src=\"");
            escape_html(&parsed.src, &mut out);
            out.push_str("\" />");
            out.push_str("</");
            out.push_str(tag);
            out.push('>');
        }
        None => {
            out.push_str("<img");
            push_image_marker_attrs(&mut out, &parsed.marker);
            out.push_str(" alt=\"");
            escape_html(&parsed.alt, &mut out);
            out.push_str("\"");
            push_common_attrs(&mut out, &parsed.attrs);
            out.push_str(" src=\"");
            escape_html(&parsed.src, &mut out);
            out.push_str("\" />");
        }
    }

    out
}

// --- Low-Level Parsing Helpers ---

#[derive(Debug, Clone)]
struct ImageCore<'a> {
    alt: String,
    src: String,
    rest: &'a str,
    marker: ImageMarker,
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
    let mut spec = AttrSpec::default();
    let mut cursor = 0;
    let mut had_any = false;
    let bytes = s.as_bytes();

    // 1. Parse optional class/id block: [.class,#id]
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
            had_any = true;
        }
    }

    // Skip whitespace between blocks
    while cursor < bytes.len() && bytes[cursor] == b' ' {
        cursor += 1;
    }

    // 2. Parse optional kv block: {key: "val", ...}
    if cursor < bytes.len() && bytes[cursor] == b'{' {
        if let Some(close_curly) = s[cursor + 1..].find('}') {
            let kv_block = &s[cursor + 1..cursor + 1 + close_curly];
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
            cursor += 1 + close_curly + 1;
            had_any = true;
        }
    }

    let rest = s.get(cursor..).unwrap_or("");
    (spec, rest, had_any)
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
        out.push_str("data-");
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

/// Renders caption markdown into inline AST events for custom node children.
/// Inline plugins (anchor, cite, wiki) run BEFORE markdown so they can process
/// raw text patterns like [text](url) and [[wiki links]].
fn render_caption_events(input: &str, inline_pipeline: &Pipeline) -> Vec<Event> {
    if input.trim().is_empty() {
        return Vec::new();
    }
    let parsed = parse(input, &Options::default());

    // Run inline plugins FIRST on raw text events
    let processed = inline_pipeline.run(parsed);

    // THEN run markdown to convert remaining patterns to AST nodes
    let processed = process_markdown(&processed);

    // Strip Document and Paragraph wrappers so only inline nodes remain
    let mut out = Vec::new();
    for ev in processed {
        match &ev {
            Event::StartNode(NodeKind::Document) | Event::EndNode(NodeKind::Document) => {}
            Event::StartNode(NodeKind::Paragraph) | Event::EndNode(NodeKind::Paragraph) => {}
            _ => out.push(ev),
        }
    }
    out
}

/// Renders caption markdown into an HTML string for figcaption in HTML mode.
/// Inline plugins run BEFORE markdown for the same reason as render_caption_events.
fn render_caption_html(input: &str, inline_pipeline: &Pipeline) -> String {
    if input.trim().is_empty() {
        return String::new();
    }
    let parsed = parse(input, &Options::default());

    // Run inline plugins FIRST on raw text events
    let processed = inline_pipeline.run(parsed);

    // THEN run markdown
    let processed = process_markdown(&processed);

    let rendered = pendon_renderer_html::render_html(&processed);
    let trimmed = rendered.trim();
    if let Some(inner) = trimmed
        .strip_prefix("<p>")
        .and_then(|s| s.strip_suffix("</p>"))
    {
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
        let pipeline = Pipeline::default();
        let events = paragraph_events("!![Alt](https://x.test/a.webp) Caption **bold** [link](/x)");
        let out = process(&events, &ImgOptions::default(), &pipeline);
        let html = out
            .iter()
            .find_map(|ev| match ev {
                Event::Text(t) if t.contains("<figure") => Some(t.clone()),
                _ => None,
            })
            .unwrap();

        assert!(html.contains("<figure>"));
        assert!(html.contains("<img alt=\"Alt\" src=\"https://x.test/a.webp\" />"));
        assert!(html.contains(
            "<figcaption>Caption <strong>bold</strong> <a href=\"/x\">link</a></figcaption>"
        ));
    }

    #[test]
    fn renders_decorated_image_attributes() {
        let pipeline = Pipeline::default();
        let events = paragraph_events(
            "![Alt](https://x.test/a.webp)[.x,#hero]{foo: \"bar\", --r: \"5deg\"}",
        );
        let out = process(&events, &ImgOptions::default(), &pipeline);
        let html = out
            .iter()
            .find_map(|ev| match ev {
                Event::Text(t) if t.contains("<img ") => Some(t.clone()),
                _ => None,
            })
            .unwrap();

        assert!(html.contains("id=\"hero\""));
        assert!(html.contains("class=\"x\""));
        assert!(html.contains("data-foo=\"bar\""));
        assert!(html.contains("style=\"--r:5deg;\""));
    }

    #[test]
    fn emits_custom_node_with_all_attributes() {
        let pipeline = Pipeline::default();
        let options = ImgOptions {
            custom_node: Some(ImgCustomNode {
                name: "AdvancedImage".into(),
                template: "<AdvancedImage />".into(),
                imports: Vec::new(),
            }),
        };
        let events =
            paragraph_events("!![Alt](https://x.test/a.webp)[.hero]{foo: \"bar\"} A caption");
        let out = process(&events, &options, &pipeline);

        assert!(out.iter().any(|e| matches!(
            e,
            Event::StartNode(NodeKind::Custom(name)) if name == "AdvancedImage"
        )));
        assert!(out.iter().any(|e| matches!(
            e,
            Event::Attribute { name, value } if name == "src" && value == "https://x.test/a.webp"
        )));
        assert!(out.iter().any(|e| matches!(
            e,
            Event::Attribute { name, value } if name == "class" && value == "hero"
        )));
        assert!(out.iter().any(|e| matches!(
            e,
            Event::Attribute { name, value } if name == "data-foo" && value == "bar"
        )));
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
}
