use pendon_core::{
    element_close, element_open, parse, raw_inline, Event, InlinePipeline, NodeKind, Options,
    Pipeline, Severity,
};
use pendon_extra::{
    legacy_extras_warning, parse_attrs, scan_extras_chars, to_attributes, ExtraAttrs, ExtrasAttr,
    ExtrasHead, ExtrasOptions,
};
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
    pub imports: Vec<ImportEntry>,
}

// --- Parsed Image Data (shared between HTML and Custom rendering) ---

#[derive(Debug, Clone)]
struct ParsedImage {
    alt: String,
    src: String,
    caption: Option<String>,
    container: Option<ContainerKind>,
    attrs: ExtraAttrs,
    marker: ImageMarker,
    /// Bare flags of the §7.1 extras head (§6.3), emitted on the outermost node.
    flags: Vec<String>,
    /// §13 warnings raised while resolving the head and extras.
    warnings: Vec<String>,
    /// Whether the deprecated `[.c,#id]{k:v}` block was read (§14).
    legacy: bool,
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
    process_inner(events, options, inline_pipeline, &mut ())
}

pub fn process_with_context<C, P>(
    events: &[Event],
    options: &ImgOptions,
    inline_pipeline: &P,
    context: &mut C,
) -> Vec<Event>
where
    P: InlinePipeline<C>,
{
    process_inner(events, options, inline_pipeline, context)
}

fn process_inner<C, P>(
    events: &[Event],
    options: &ImgOptions,
    inline_pipeline: &P,
    context: &mut C,
) -> Vec<Event>
where
    P: InlinePipeline<C>,
{
    let mut out: Vec<Event> = Vec::with_capacity(events.len());
    let mut i = 0usize;

    while i < events.len() {
        if matches!(events.get(i), Some(Event::StartNode(NodeKind::Paragraph))) {
            if let Some(end) = find_matching_end(events, i, NodeKind::Paragraph) {
                let block = &events[i + 1..end];
                if let Some(parsed) = maybe_parse_advanced_image(block) {
                    if options.custom_node.is_some() {
                        emit_custom_image(&parsed, options, inline_pipeline, context, &mut out);
                    } else {
                        emit_element_image(&parsed, inline_pipeline, context, &mut out);
                        // Block level images own their line (the old HtmlBlock
                        // rendering appended a newline after the markup).
                        out.extend(raw_inline("\n"));
                    }
                    i = end + 1;
                    continue;
                }
                // Advanced images may also be embedded inline inside a paragraph
                // alongside other text (e.g. "foo !?~[alt](src)[.c] bar").
                if !block.is_empty() && block.iter().all(|ev| matches!(ev, Event::Text(_))) {
                    out.push(events[i].clone());
                    for ev in block {
                        if let Event::Text(text) = ev {
                            emit_text_with_inline_images(
                                text,
                                options,
                                inline_pipeline,
                                context,
                                &mut out,
                            );
                        } else {
                            out.push(ev.clone());
                        }
                    }
                    out.push(events[end].clone());
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

fn emit_custom_image<C>(
    parsed: &ParsedImage,
    options: &ImgOptions,
    inline_pipeline: &impl InlinePipeline<C>,
    context: &mut C,
    out: &mut Vec<Event>,
) {
    let custom = match options.custom_node.as_ref() {
        Some(c) => c,
        None => return,
    };

    push_image_warnings(parsed, out);
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

    // Extra attributes (id, class, props, style).
    // Custom Solid components receive plain props, so keys are passed through
    // verbatim instead of being prefixed with `data-` (unlike the HTML path).
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
    for (k, v) in &parsed.attrs.properties {
        if !k.starts_with("--") && k != "style" {
            out.push(Event::Attribute {
                name: k.clone(),
                value: v.clone(),
            });
        }
    }
    let style_str = image_style(&parsed.attrs);
    if !style_str.is_empty() {
        out.push(Event::Attribute {
            name: "style".to_string(),
            value: style_str,
        });
    }
    // §6.3: a bare flag stays a bare attribute, also on a custom component.
    for name in &parsed.flags {
        out.push(Event::AttributeFlag { name: name.clone() });
    }

    // Caption as rendered inline children with full inline pipeline support
    if let Some(caption) = &parsed.caption {
        let caption_events = render_caption_events(caption, inline_pipeline, context);
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

    if !custom.imports.is_empty() {
        hints.template_imports.insert(key, custom.imports.clone());
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
    let blocks = parse_attached_blocks(core.rest);
    let caption = blocks.rest.trim();
    let caption = if caption.is_empty() {
        None
    } else {
        Some(caption.to_string())
    };

    Some(build_parsed_image(
        core,
        Some(ContainerKind::Figure),
        caption,
        blocks,
    ))
}

fn parse_decorated_image_syntax(line: &str) -> Option<ParsedImage> {
    let core = parse_image_core(line)?;
    if core.marker.container == Some(ContainerKind::Figure) {
        return None;
    }

    if let Some(container) = core.marker.container {
        let blocks = parse_attached_blocks(core.rest);
        if !blocks.rest.trim().is_empty() {
            return None;
        }
        return Some(build_parsed_image(core, Some(container), None, blocks));
    }

    let blocks = parse_attached_blocks(core.rest);
    // §7.1: an extras head alone makes the image advanced, exactly like the
    // `[.class,#id]{k:v}` block does.
    let has_attrs = blocks.legacy || blocks.extras.is_some();
    let has_marker_mod = core.marker.has_modifiers();
    if !has_marker_mod && (!has_attrs || !blocks.rest.trim().is_empty()) {
        return None;
    }
    if has_marker_mod && !blocks.rest.trim().is_empty() {
        return None;
    }

    Some(build_parsed_image(core, None, None, blocks))
}

// --- Structured Element Emission (used when no custom component is configured) ---

/// Emits the image as structured [`NodeKind::Element`] events.
///
/// Emitting real events (instead of one raw HTML string) keeps custom
/// components alive inside `<figcaption>`: citations, custom inline plugins and
/// custom anchors all nest correctly in every renderer.
fn emit_element_image<C, P>(
    parsed: &ParsedImage,
    inline_pipeline: &P,
    context: &mut C,
    out: &mut Vec<Event>,
) where
    P: InlinePipeline<C>,
{
    push_image_warnings(parsed, out);
    let container = parsed.container.or(parsed.marker.container);

    match container {
        Some(ContainerKind::Figure) => {
            out.extend(element_open("figure"));
            push_common_attrs(out, &parsed.attrs, &parsed.flags);
            push_img_element(parsed, out);

            if let Some(caption) = &parsed.caption {
                let caption_events = render_caption_events(caption, inline_pipeline, context);
                if !caption_events.is_empty() {
                    out.extend(element_open("figcaption"));
                    out.extend(caption_events);
                    out.push(element_close("figcaption"));
                }
            }

            out.push(element_close("figure"));
        }
        Some(ContainerKind::Paragraph) | Some(ContainerKind::Division) => {
            let tag = match container {
                Some(ContainerKind::Paragraph) => "p",
                Some(ContainerKind::Division) => "div",
                _ => unreachable!(),
            };
            out.extend(element_open(tag));
            push_common_attrs(out, &parsed.attrs, &parsed.flags);
            push_img_element(parsed, out);
            out.push(element_close(tag));
        }
        None => {
            out.extend(element_open("img"));
            push_image_marker_attrs(out, &parsed.marker);
            attribute(out, "alt", &parsed.alt);
            push_common_attrs(out, &parsed.attrs, &parsed.flags);
            attribute(out, "src", &parsed.src);
            out.push(element_close("img"));
        }
    }
}

/// Emits the `<img … />` element itself.
fn push_img_element(parsed: &ParsedImage, out: &mut Vec<Event>) {
    out.extend(element_open("img"));
    push_image_marker_attrs(out, &parsed.marker);
    attribute(out, "alt", &parsed.alt);
    attribute(out, "src", &parsed.src);
    out.push(element_close("img"));
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

/// The attribute blocks attached after an image's `(url)` head: the deprecated
/// `[.class,#id]{key: value}` form (§14) and the §11 `@@type{…}` extras head
/// (§7.1). Both attach to the outermost node of the construct.
struct AttachedBlocks<'a> {
    attrs: ExtraAttrs,
    extras: Option<ExtrasHead>,
    /// Whether the deprecated block was present and is therefore reported (§14).
    legacy: bool,
    /// Text left after the blocks: the figure caption or trailing text.
    rest: &'a str,
}

impl<'a> AttachedBlocks<'a> {
    fn empty(rest: &'a str) -> Self {
        Self {
            attrs: ExtraAttrs::default(),
            extras: None,
            legacy: false,
            rest,
        }
    }
}

/// Parses the attribute blocks attached to an image. The extras head must be
/// adjacent to the legacy block (§4.1), so it is scanned on the untrimmed rest.
fn parse_attached_blocks(input: &str) -> AttachedBlocks<'_> {
    let parsed = parse_attrs(input);
    let consumed = input.len().saturating_sub(parsed.rest.len());
    let rest = &input[consumed..];
    let chars: Vec<char> = rest.chars().collect();

    match scan_extras_chars(&chars, 0) {
        Some((head, next)) => {
            let bytes: usize = chars[..next].iter().map(|ch| ch.len_utf8()).sum();
            AttachedBlocks {
                attrs: parsed.attrs,
                extras: Some(head),
                legacy: parsed.had_attrs,
                rest: &rest[bytes..],
            }
        }
        None => AttachedBlocks {
            attrs: parsed.attrs,
            extras: None,
            legacy: parsed.had_attrs,
            rest,
        },
    }
}

/// Assembles a parsed image: merges the attached blocks into the attributes and
/// keeps the flags and warnings the merge resolved.
fn build_parsed_image(
    core: ImageCore<'_>,
    container: Option<ContainerKind>,
    caption: Option<String>,
    blocks: AttachedBlocks<'_>,
) -> ParsedImage {
    let AttachedBlocks {
        mut attrs,
        extras,
        legacy,
        ..
    } = blocks;
    let mut flags = Vec::new();
    let mut warnings = Vec::new();
    merge_image_extras(&mut attrs, extras.as_ref(), &mut flags, &mut warnings);

    ParsedImage {
        alt: core.alt,
        src: core.src,
        caption,
        container,
        attrs,
        marker: core.marker,
        flags,
        warnings,
        legacy,
    }
}

/// Whether the construct side (head or deprecated block) already sets `key`.
fn has_attribute(attrs: &ExtraAttrs, key: &str) -> bool {
    match key {
        "id" => attrs.id.is_some(),
        "class" => !attrs.classes.is_empty(),
        _ => attrs.properties.iter().any(|(name, _)| name == key),
    }
}

/// Merges a §7.1 extras head into the image attributes.
///
/// The construct side wins (§6.2): an `id`/`class`/prop already set by the head
/// or the deprecated block is kept and the extras value is reported as dropped.
/// `class` accumulates (§6.4), `slug` fills `id` when no `id` is present, and
/// bare flags stay bare attributes (§6.3).
fn merge_image_extras(
    attrs: &mut ExtraAttrs,
    extras: Option<&ExtrasHead>,
    flags: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    let Some(head) = extras else {
        return;
    };

    let parsed = to_attributes(head, &ExtrasOptions::default());
    for warning in &parsed.warnings {
        warnings.push(pendon_extra::warning_message(warning));
    }

    let extras_id = parsed.value("id").map(|value| value.literal());
    let extras_slug = parsed.value("slug").map(|value| value.literal());

    for (key, value) in &parsed.items {
        if key == "id" || key == "slug" {
            continue;
        }
        match value {
            ExtrasAttr::Flag => {
                if has_attribute(attrs, key) {
                    warnings.push(format!(
                        "`{key}` was dropped because the construct head already sets it"
                    ));
                } else {
                    flags.push(key.clone());
                }
            }
            ExtrasAttr::Value(value) => {
                let text = value.literal();
                // §6.4: `class` accumulates, head classes first.
                if key == "class" {
                    for token in text.split_whitespace() {
                        attrs.classes.push(token.to_string());
                    }
                    continue;
                }
                if has_attribute(attrs, key) {
                    warnings.push(format!(
                        "`{key}` was dropped because the construct head already sets it"
                    ));
                    continue;
                }
                attrs.properties.push((key.clone(), text));
            }
        }
    }

    // §6.2: `#id` > extras `slug`; an `id` already on the image wins.
    match extras_id.or(extras_slug) {
        Some(id) if attrs.id.is_none() => attrs.id = Some(id),
        Some(_) => warnings
            .push("extras `id` was dropped because the image already has an id (§6.2)".to_string()),
        None => {}
    }
}

fn is_inline_marker_char(ch: char) -> bool {
    matches!(ch, '!' | '?' | '~' | 'w' | 'h') || ch.is_ascii_digit()
}

fn find_char_index(chars: &[char], mut index: usize, wanted: char) -> Option<usize> {
    while index < chars.len() {
        if chars[index] == wanted {
            return Some(index);
        }
        index += 1;
    }
    None
}

/// Parses the attribute blocks attached to an inline image, returning them and
/// the number of consumed characters (0 when nothing valid was attached).
fn parse_inline_blocks(input: &str) -> (AttachedBlocks<'_>, usize) {
    let starts_with_block = input.starts_with('{')
        || input.starts_with("@@")
        || (input.starts_with('[')
            && input
                .chars()
                .nth(1)
                .is_some_and(|ch| ch == '.' || ch == '#'));
    if !starts_with_block {
        return (AttachedBlocks::empty(input), 0);
    }

    let blocks = parse_attached_blocks(input);
    let has_content = blocks.extras.is_some()
        || blocks.attrs.id.is_some()
        || !blocks.attrs.classes.is_empty()
        || !blocks.attrs.properties.is_empty();
    if !has_content {
        return (AttachedBlocks::empty(input), 0);
    }

    let consumed_bytes = input.len().saturating_sub(blocks.rest.len());
    (blocks, input[..consumed_bytes].chars().count())
}

/// Attempts to parse an advanced image starting at `start` (the first marker
/// character). Returns the exclusive end index and the parsed image when the
/// marker describes an inline (non-figure, non-container) image, i.e. a bare
/// `<img>` that is only considered "advanced" when it carries marker modifiers
/// or an attached attribute block.
fn parse_inline_image_at(chars: &[char], start: usize) -> Option<(usize, ParsedImage)> {
    if !is_inline_marker_char(*chars.get(start)?) {
        return None;
    }

    let mut marker_end = start;
    while marker_end < chars.len() && is_inline_marker_char(chars[marker_end]) {
        marker_end += 1;
    }
    let marker_raw: String = chars[start..marker_end].iter().collect();
    let marker = parse_marker(&marker_raw)?;
    if marker.container.is_some() {
        return None;
    }

    if chars.get(marker_end) != Some(&'[') {
        return None;
    }
    let close_br = find_char_index(chars, marker_end + 1, ']')?;
    if chars.get(close_br + 1) != Some(&'(') {
        return None;
    }
    let close_par = find_char_index(chars, close_br + 2, ')')?;

    let alt: String = chars[marker_end + 1..close_br].iter().collect();
    let src: String = chars[close_br + 2..close_par].iter().collect();

    let rest_source: String = chars[close_par + 1..].iter().collect();
    let (blocks, consumed) = parse_inline_blocks(&rest_source);

    if !marker.has_modifiers() && consumed == 0 {
        return None;
    }

    let end = close_par + 1 + consumed;
    let core = ImageCore {
        alt,
        src,
        rest: "",
        marker,
    };
    Some((end, build_parsed_image(core, None, None, blocks)))
}

/// Emits `text` while replacing any inline advanced image patterns with their
/// rendered representation (raw HTML or a custom component node).
fn emit_text_with_inline_images<C, P>(
    text: &str,
    options: &ImgOptions,
    inline_pipeline: &P,
    context: &mut C,
    out: &mut Vec<Event>,
) where
    P: InlinePipeline<C>,
{
    let chars: Vec<char> = text.chars().collect();
    let mut cursor = 0usize;
    let mut normal = String::new();

    while cursor < chars.len() {
        if is_inline_marker_char(chars[cursor]) {
            if let Some((end, parsed)) = parse_inline_image_at(&chars, cursor) {
                if !normal.is_empty() {
                    out.push(Event::Text(std::mem::take(&mut normal)));
                }
                emit_inline_image(&parsed, options, inline_pipeline, context, out);
                cursor = end;
                continue;
            }
        }
        normal.push(chars[cursor]);
        cursor += 1;
    }

    if !normal.is_empty() {
        out.push(Event::Text(normal));
    }
}

fn emit_inline_image<C, P>(
    parsed: &ParsedImage,
    options: &ImgOptions,
    inline_pipeline: &P,
    context: &mut C,
    out: &mut Vec<Event>,
) where
    P: InlinePipeline<C>,
{
    if options.custom_node.is_some() {
        emit_custom_image(parsed, options, inline_pipeline, context, out);
    } else {
        emit_element_image(parsed, inline_pipeline, context, out);
    }
}

fn push_common_attrs(out: &mut Vec<Event>, attrs: &ExtraAttrs, flags: &[String]) {
    if let Some(id) = attrs.id.as_deref() {
        attribute(out, "id", id);
    }

    if !attrs.classes.is_empty() {
        attribute(out, "class", &attrs.classes.join(" "));
    }

    // HTML elements keep the `data-` prefix (custom components receive the
    // keys verbatim, see `emit_custom_image`). The `style` value is emitted as
    // the `style` attribute itself (§6.3).
    for (k, v) in &attrs.properties {
        if k.starts_with("--") || k == "style" {
            continue;
        }
        attribute(out, &format!("data-{}", k), v);
    }

    let styles = image_style(attrs);
    if !styles.is_empty() {
        attribute(out, "style", &styles);
    }

    // §6.3: a bare flag stays a bare attribute (`<figure isFoo>`).
    for name in flags {
        out.push(Event::AttributeFlag { name: name.clone() });
    }
}

/// The `style` value of an image: the `--var` properties of the deprecated
/// block, followed by the `style` value the extras head merged (§6.3).
fn image_style(attrs: &ExtraAttrs) -> String {
    let mut styles: String = attrs
        .properties
        .iter()
        .filter(|(k, _)| k.starts_with("--"))
        .map(|(k, v)| format!("{}:{};", k, v))
        .collect();
    for (key, value) in &attrs.properties {
        if key != "style" {
            continue;
        }
        if !styles.is_empty() {
            styles.push(' ');
        }
        styles.push_str(value);
        if !styles.ends_with(';') {
            styles.push(';');
        }
    }
    styles
}

/// §13/§14 diagnostics raised while parsing and merging an image head.
fn push_image_warnings(parsed: &ParsedImage, out: &mut Vec<Event>) {
    if parsed.legacy {
        out.push(legacy_extras_warning("img"));
    }
    for message in &parsed.warnings {
        out.push(Event::Diagnostic {
            severity: Severity::Warning,
            message: format!("[img] {message}"),
            span: None,
        });
    }
}

fn push_image_marker_attrs(out: &mut Vec<Event>, marker: &ImageMarker) {
    if let Some(width) = marker.width {
        attribute(out, "width", &width.to_string());
    }
    if let Some(height) = marker.height {
        attribute(out, "height", &height.to_string());
    }
    if marker.async_decoding {
        attribute(out, "decoding", "async");
    }
    if marker.lazy {
        attribute(out, "loading", "lazy");
    }
}

fn attribute(out: &mut Vec<Event>, name: &str, value: &str) {
    out.push(Event::Attribute {
        name: name.to_string(),
        value: value.to_string(),
    });
}

/// Renders caption markdown into inline AST events for custom node children.
/// Inline plugins (anchor, cite, wiki) run BEFORE markdown so they can process
/// raw text patterns like [text](url) and [[wiki links]].
fn render_caption_events<C, P>(input: &str, inline_pipeline: &P, context: &mut C) -> Vec<Event>
where
    P: InlinePipeline<C>,
{
    if input.trim().is_empty() {
        return Vec::new();
    }
    // Whitespace around a caption/figcaption content is insignificant.
    let parsed = parse(input.trim(), &Options::default());

    // Run inline plugins FIRST on raw text events
    let processed = inline_pipeline.run_with(context, parsed);

    // THEN run markdown to convert remaining patterns to AST nodes
    let processed = process_markdown(&processed);

    // FINALLY run processors that must see markdown-expanded nodes (math),
    // so a `$` inside a link destination is not mistaken for inline math.
    let processed = inline_pipeline.run_after_markdown(context, processed);

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

#[cfg(test)]
mod tests {
    use super::*;
    use pendon_core::ContextPipeline;

    fn paragraph_events(text: &str) -> Vec<Event> {
        vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Paragraph),
            Event::Text(text.to_string()),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ]
    }

    /// Collects the events nested directly inside the first `tag` element.
    fn element_children(events: &[Event], tag: &str) -> Vec<Event> {
        let mut children = Vec::new();
        let mut depth = 0usize;
        let mut inside = false;

        for event in events {
            match event {
                Event::StartNode(NodeKind::Element(name)) => {
                    if inside {
                        depth += 1;
                    } else if name == tag {
                        inside = true;
                        continue;
                    }
                }
                Event::EndNode(NodeKind::Element(_)) if inside => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                }
                _ => {}
            }

            if inside {
                children.push(event.clone());
            }
        }

        children
    }

    fn has_attribute(events: &[Event], name: &str, value: &str) -> bool {
        events.iter().any(|event| {
            matches!(event, Event::Attribute { name: n, value: v } if n == name && v == value)
        })
    }

    /// Text is emitted as many small chunks (sometimes one per character), so
    /// concatenate before searching.
    fn has_text(events: &[Event], needle: &str) -> bool {
        let mut text = String::new();
        for event in events {
            if let Event::Text(chunk) = event {
                text.push_str(chunk);
            }
        }
        text.contains(needle)
    }

    /// Stand-in for an inline custom plugin (like `cite`): rewrites `{{cite}}`
    /// text into a custom component node.
    fn custom_component_pipeline() -> ContextPipeline<()> {
        let mut pipeline = ContextPipeline::new();
        pipeline.add(|_: &mut (), events: Vec<Event>| {
            let mut out = Vec::new();
            for event in events {
                match event {
                    Event::Text(text) => {
                        for (index, part) in text.split("{{cite}}").enumerate() {
                            if index > 0 {
                                out.push(Event::StartNode(NodeKind::Custom("Cite".into())));
                                out.push(Event::Attribute {
                                    name: "id".into(),
                                    value: "book".into(),
                                });
                                out.push(Event::EndNode(NodeKind::Custom("Cite".into())));
                            }
                            if !part.is_empty() {
                                out.push(Event::Text(part.to_string()));
                            }
                        }
                    }
                    other => out.push(other),
                }
            }
            out
        });
        pipeline
    }

    #[test]
    fn renders_figure_with_markdown_caption() {
        let pipeline = Pipeline::default();
        let events = paragraph_events("!![Alt](https://x.test/a.webp) Caption **bold** [link](/x)");
        let out = process(&events, &ImgOptions::default(), &pipeline);

        assert!(out.iter().any(|event| matches!(
            event,
            Event::StartNode(NodeKind::Element(name)) if name == "figure"
        )));
        assert!(has_attribute(&out, "alt", "Alt"));
        assert!(has_attribute(&out, "src", "https://x.test/a.webp"));

        // The caption is a real element whose children are inline AST nodes
        // (not one escaped HTML string), so custom components survive inside it.
        let caption = element_children(&out, "figcaption");
        assert!(caption
            .iter()
            .any(|event| matches!(event, Event::StartNode(NodeKind::Strong))));
        assert!(caption
            .iter()
            .any(|event| matches!(event, Event::StartNode(NodeKind::Link))));
        assert!(has_attribute(&caption, "href", "/x"));
        assert!(has_text(&caption, "Caption "));
        assert!(has_text(&caption, "bold"));
    }

    #[test]
    fn nests_custom_components_inside_figcaption() {
        let pipeline = custom_component_pipeline();
        let events = paragraph_events("!![Alt](https://x.test/a.webp) See {{cite}} for details.");
        let out = process_with_context(&events, &ImgOptions::default(), &pipeline, &mut ());

        let caption = element_children(&out, "figcaption");
        assert!(caption.iter().any(|event| matches!(
            event,
            Event::StartNode(NodeKind::Custom(name)) if name == "Cite"
        )));
        assert!(has_attribute(&caption, "id", "book"));
        assert!(has_text(&caption, "See "));
        assert!(has_text(&caption, " for details."));

        // <img> and <figcaption> are both children of the <figure> element.
        let figure = element_children(&out, "figure");
        assert!(figure.iter().any(|event| matches!(
            event,
            Event::StartNode(NodeKind::Element(name)) if name == "img"
        )));
        assert!(figure.iter().any(|event| matches!(
            event,
            Event::StartNode(NodeKind::Element(name)) if name == "figcaption"
        )));
    }

    #[test]
    fn renders_decorated_image_attributes() {
        let pipeline = Pipeline::default();
        let events = paragraph_events(
            "![Alt](https://x.test/a.webp)[.x,#hero]{foo: \"bar\", --r: \"5deg\"}",
        );
        let out = process(&events, &ImgOptions::default(), &pipeline);

        assert!(out.iter().any(|event| matches!(
            event,
            Event::StartNode(NodeKind::Element(name)) if name == "img"
        )));
        assert!(has_attribute(&out, "id", "hero"));
        assert!(has_attribute(&out, "class", "x"));
        assert!(has_attribute(&out, "data-foo", "bar"));
        assert!(has_attribute(&out, "style", "--r:5deg;"));
    }

    #[test]
    fn renders_inline_advanced_image_inside_paragraph() {
        let pipeline = Pipeline::default();
        let events = paragraph_events(
            "Ad ex tempor !?~[Alt](https://x.test/a.webp)[.foo]{con: \"jux\"} consectetur.",
        );
        let out = process(&events, &ImgOptions::default(), &pipeline);

        // The paragraph stays open: the image is an inline element inside it.
        assert!(out
            .iter()
            .any(|event| matches!(event, Event::StartNode(NodeKind::Paragraph))));
        assert!(out.iter().any(|event| matches!(
            event,
            Event::StartNode(NodeKind::Element(name)) if name == "img"
        )));
        assert!(has_attribute(&out, "decoding", "async"));
        assert!(has_attribute(&out, "loading", "lazy"));
        assert!(has_attribute(&out, "alt", "Alt"));
        assert!(has_attribute(&out, "class", "foo"));
        assert!(has_attribute(&out, "data-con", "jux"));
        assert!(has_text(&out, "consectetur."));
    }

    #[test]
    fn leaves_vanilla_inline_image_for_markdown() {
        let pipeline = Pipeline::default();
        let events = paragraph_events("Ad ex tempor ![Alt](https://x.test/a.webp) consectetur.");
        let out = process(&events, &ImgOptions::default(), &pipeline);

        assert!(!out
            .iter()
            .any(|e| matches!(e, Event::StartNode(NodeKind::HtmlInline))));
        assert!(out
            .iter()
            .any(|e| matches!(e, Event::Text(t) if t.contains("![Alt](https://x.test/a.webp)"))));
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
            Event::Attribute { name, value } if name == "foo" && value == "bar"
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

    #[test]
    fn extras_attach_to_the_outermost_figure() {
        let pipeline = Pipeline::default();
        let events = paragraph_events(
            "~?!!h300w800[Alt](https://x.test/a.webp)@@figure{.wide, #fig, foo: \"bar\"} Caption",
        );
        let out = process(&events, &ImgOptions::default(), &pipeline);

        // §7.1: extras go to the outer `<figure>`, the `w`/`h` marker stays on
        // the inner `<img>`.
        let figure = element_children(&out, "figure");
        assert!(has_attribute(&figure, "id", "fig"));
        assert!(has_attribute(&figure, "class", "wide"));
        assert!(has_attribute(&figure, "data-foo", "bar"));
        let img = element_children(&figure, "img");
        assert!(has_attribute(&img, "width", "800"));
        assert!(has_attribute(&img, "height", "300"));
        assert!(!has_attribute(&img, "class", "wide"));

        // The whitespace after the head does not leak into the caption.
        let caption = element_children(&out, "figcaption");
        assert!(has_text(&caption, "Caption"));
        assert!(!has_text(&caption, "@@figure"));
    }

    #[test]
    fn extras_attach_to_an_inline_image() {
        let pipeline = Pipeline::default();
        let events = paragraph_events(
            "Ad ex tempor !?~[Alt](https://x.test/a.webp)@@img{con: \"jux\", .foo} consectetur.",
        );
        let out = process(&events, &ImgOptions::default(), &pipeline);

        assert!(out
            .iter()
            .any(|event| matches!(event, Event::StartNode(NodeKind::Paragraph))));
        assert!(has_attribute(&out, "class", "foo"));
        assert!(has_attribute(&out, "data-con", "jux"));
        assert!(has_attribute(&out, "loading", "lazy"));
        assert!(has_text(&out, "consectetur."));
        assert!(!has_text(&out, "@@img"));
    }

    #[test]
    fn extras_class_accumulates_after_the_legacy_classes() {
        let parsed = parse_figure_syntax("!![Alt](https://x.test/a.webp)[.head]@@figure{.extra}")
            .expect("figure");
        // §6.4: class accumulates, head classes first.
        assert_eq!(parsed.attrs.classes, vec!["head", "extra"]);
    }

    #[test]
    fn the_legacy_block_wins_over_extras_and_reports_it() {
        let parsed = parse_figure_syntax("!![Alt](https://x.test/a.webp)[#hero]@@figure{#other}")
            .expect("figure");
        assert_eq!(parsed.attrs.id.as_deref(), Some("hero"));
        assert!(parsed.legacy);
        assert!(parsed
            .warnings
            .iter()
            .any(|warning| warning.contains("already has an id")));

        let pipeline = Pipeline::default();
        let events = paragraph_events("!![Alt](https://x.test/a.webp)[#hero]@@figure{#other} Cap");
        let out = process(&events, &ImgOptions::default(), &pipeline);
        // §14: the deprecated block is reported, not applied silently.
        assert!(out.iter().any(|event| matches!(
            event,
            Event::Diagnostic { message, .. } if message.contains("[img]") && message.contains("deprecated")
        )));
    }

    #[test]
    fn extras_slug_fills_the_id_when_no_id_is_present() {
        let parsed = parse_figure_syntax("!![Alt](https://x.test/a.webp)@@figure{`the-figure`}")
            .expect("figure");
        // §6.2: `#id` > head slug > extras slug.
        assert_eq!(parsed.attrs.id.as_deref(), Some("the-figure"));
    }

    #[test]
    fn bare_flags_stay_bare_attributes() {
        let pipeline = Pipeline::default();
        let events = paragraph_events("!![Alt](https://x.test/a.webp)@@figure{isFoo} Cap");
        let out = process(&events, &ImgOptions::default(), &pipeline);

        let figure = element_children(&out, "figure");
        assert!(figure
            .iter()
            .any(|event| matches!(event, Event::AttributeFlag { name } if name == "isFoo")));
    }

    #[test]
    fn extras_style_merges_into_the_style_attribute() {
        let pipeline = Pipeline::default();
        let events = paragraph_events("![Alt](https://x.test/a.webp)@@img{--r: \"5deg\"}");
        let out = process(&events, &ImgOptions::default(), &pipeline);

        assert!(has_attribute(&out, "style", "--r: 5deg;"));
        assert!(!out.iter().any(|event| matches!(
            event,
            Event::Attribute { name, .. } if name == "data-style"
        )));
    }

    #[test]
    fn malformed_extras_stay_literal_text() {
        let pipeline = Pipeline::default();
        let events = paragraph_events("Ad ex tempor ![Alt](https://x.test/a.webp)@@figure{.a ex.");
        let out = process(&events, &ImgOptions::default(), &pipeline);

        assert!(!out.iter().any(
            |event| matches!(event, Event::StartNode(NodeKind::Element(name)) if name == "img")
        ));
        assert!(has_text(&out, "@@figure{.a"));
    }

    #[test]
    fn custom_component_receives_extras_props() {
        let pipeline = Pipeline::default();
        let options = ImgOptions {
            custom_node: Some(ImgCustomNode {
                name: "AdvancedImage".into(),
                template: "<AdvancedImage />".into(),
                imports: Vec::new(),
            }),
        };
        let events = paragraph_events(
            "!![Alt](https://x.test/a.webp)@@figure{.hero, foo: \"bar\"} A caption",
        );
        let out = process(&events, &options, &pipeline);

        assert!(out.iter().any(|e| matches!(
            e,
            Event::StartNode(NodeKind::Custom(name)) if name == "AdvancedImage"
        )));
        assert!(out.iter().any(|e| matches!(
            e,
            Event::Attribute { name, value } if name == "class" && value == "hero"
        )));
        assert!(out.iter().any(|e| matches!(
            e,
            Event::Attribute { name, value } if name == "foo" && value == "bar"
        )));
        assert!(has_text(&out, "A caption"));
    }

    #[test]
    fn supports_borrowed_context_for_caption_pipeline() {
        let mut pipeline = ContextPipeline::new();
        pipeline.add(|count: &mut usize, events| {
            *count += 1;
            events
        });

        let mut count = 0;
        let events = paragraph_events("!![Alt](https://x.test/a.webp) Caption");
        let out = process_with_context(&events, &ImgOptions::default(), &pipeline, &mut count);

        // A borrowed (&mut) context works with the caption inline pipeline.
        assert!(count > 0, "caption pipeline did not run");
        let caption = element_children(&out, "figcaption");
        assert!(has_text(&caption, "Caption"));
    }
}
