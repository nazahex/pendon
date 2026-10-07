use pendon_core::{
    element_close, element_open, parse, raw_inline, Event, InlinePipeline, NodeKind, Options,
    Pipeline, Severity,
};
use pendon_extra::{scan_extras_chars, to_attributes, ExtrasAttr, ExtrasHead, ExtrasOptions};
use pendon_plugin_markdown::process as process_markdown;
use pendon_renderer_solid::{ComponentSet, ComponentTemplate, ImportEntry, SolidRenderHints};
use serde::{Deserialize, Serialize};

// --- Configuration & Types ---

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ImgOptions {
    /// §11 `img` layer: the `<img>` element itself.
    pub img: ComponentSet<ImgCustomNode>,
    /// §11 `figure` layer: the `<figure>` container of `~?!!`.
    pub figure: ComponentSet<ImgCustomNode>,
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
    attrs: ImageAttrs,
    marker: ImageMarker,
    /// Bare flags of the §7.1 extras head (§6.3), emitted on the outermost node.
    flags: Vec<String>,
    /// The `@@type{…}` marker, when present: the §11 routing key (rule 3).
    type_marker: Option<String>,
    /// §13 warnings raised while resolving the head and extras.
    warnings: Vec<String>,
}

/// The attributes of one image, assembled from its §7.1 extras head (§6).
///
/// The pre-§11 `[.c,#id]{k:v}` block is retired (§14), so extras are the only
/// source and the construct's own `src`/`alt` are emitted separately.
#[derive(Debug, Clone, Default)]
struct ImageAttrs {
    id: Option<String>,
    classes: Vec<String>,
    properties: Vec<(String, String)>,
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
                    match route_image(&parsed, options) {
                        Some(route) => {
                            emit_custom_image(&parsed, &route, inline_pipeline, context, &mut out);
                        }
                        None => {
                            emit_element_image(&parsed, inline_pipeline, context, &mut out);
                            // Block level images own their line (the old HtmlBlock
                            // rendering appended a newline after the markup).
                            out.extend(raw_inline("\n"));
                        }
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

// --- §11 routing -----------------------------------------------------------

/// §11 rule 3 routing of one image: which component renders the outermost node,
/// and whether the `<img>` element nests inside it.
struct CustomRoute<'a> {
    outer: &'a ImgCustomNode,
    /// `Some` when the `img` layer renders the element nested inside the outer
    /// component (a figure whose `figure` *and* `img` layers are configured).
    inner: Option<&'a ImgCustomNode>,
    /// `true` when the outer node is the `<figure>` container of `~?!!`.
    outer_is_figure: bool,
}

/// Selects the component of the outermost node (§11 rule 3).
///
/// - `~?!!` (a figure): the `figure` layer is the container, the `img` layer the
///   element inside it; either one alone replaces the whole image.
/// - any other image: the `img` layer is the node; the `figure` layer is used
///   when no `img` entry answers the marker (pre-cutover configs keep working).
fn route_image<'a>(parsed: &ParsedImage, options: &'a ImgOptions) -> Option<CustomRoute<'a>> {
    let marker = parsed.type_marker.as_deref();
    let img = options.img.select(marker);
    let figure = options.figure.select(marker);
    let is_figure = matches!(
        parsed.container.or(parsed.marker.container),
        Some(ContainerKind::Figure)
    );

    match (is_figure, figure, img) {
        // Both layers on a figure: the container wraps the element.
        (true, Some(figure), Some(img)) => Some(CustomRoute {
            outer: figure,
            inner: Some(img),
            outer_is_figure: true,
        }),
        (true, Some(figure), None) => Some(CustomRoute {
            outer: figure,
            inner: None,
            outer_is_figure: true,
        }),
        // A single component replaces the whole image (pre-cutover behaviour).
        (_, None, Some(img)) => Some(CustomRoute {
            outer: img,
            inner: None,
            outer_is_figure: false,
        }),
        // A non-figure image with both layers: the `img` layer is the node — the
        // `figure` layer only has meaning for a figure.
        (false, Some(_), Some(img)) => Some(CustomRoute {
            outer: img,
            inner: None,
            outer_is_figure: false,
        }),
        (false, Some(figure), None) => Some(CustomRoute {
            outer: figure,
            inner: None,
            outer_is_figure: false,
        }),
        (true, None, None) | (false, None, None) => None,
    }
}

/// Emits `id`/`class`/props/`style`/flags of the outermost node. Custom
/// components receive the keys verbatim (`type` included), unlike the HTML
/// fallback which prefixes extra props with `data-`.
fn push_custom_extra_attrs(parsed: &ParsedImage, out: &mut Vec<Event>) {
    if let Some(id) = &parsed.attrs.id {
        attribute(out, "id", id);
    }
    if !parsed.attrs.classes.is_empty() {
        attribute(out, "class", &parsed.attrs.classes.join(" "));
    }
    for (k, v) in &parsed.attrs.properties {
        if !k.starts_with("--") && k != "style" {
            attribute(out, k, v);
        }
    }
    let style_str = image_style(&parsed.attrs);
    if !style_str.is_empty() {
        attribute(out, "style", &style_str);
    }
    // §6.3: a bare flag stays a bare attribute, also on a custom component.
    for name in &parsed.flags {
        out.push(Event::AttributeFlag { name: name.clone() });
    }
}

/// The `container` attribute, when the marker asked for one.
fn push_container_attribute(parsed: &ParsedImage, out: &mut Vec<Event>) {
    if let Some(container) = parsed.container.or(parsed.marker.container) {
        let value = match container {
            ContainerKind::Figure => "figure",
            ContainerKind::Paragraph => "p",
            ContainerKind::Division => "div",
        };
        attribute(out, "container", value);
    }
}

/// `alt`/`src` plus the `w`/`h` and lazy/async marker attributes: the payload of
/// the `<img>` element itself, which stays on the inner node (§7.1).
fn push_image_element_attrs(parsed: &ParsedImage, out: &mut Vec<Event>) {
    attribute(out, "alt", &parsed.alt);
    attribute(out, "src", &parsed.src);
    if parsed.marker.lazy {
        attribute(out, "lazy", "1");
    }
    if parsed.marker.async_decoding {
        attribute(out, "async_decoding", "1");
    }
    if let Some(width) = parsed.marker.width {
        attribute(out, "width", &width.to_string());
    }
    if let Some(height) = parsed.marker.height {
        attribute(out, "height", &height.to_string());
    }
}

fn emit_caption_children<C>(
    parsed: &ParsedImage,
    inline_pipeline: &impl InlinePipeline<C>,
    context: &mut C,
    out: &mut Vec<Event>,
) {
    let Some(caption) = &parsed.caption else {
        return;
    };
    out.extend(render_caption_events(caption, inline_pipeline, context));
}

// --- Custom Node Emission ---

fn emit_custom_image<C>(
    parsed: &ParsedImage,
    route: &CustomRoute<'_>,
    inline_pipeline: &impl InlinePipeline<C>,
    context: &mut C,
    out: &mut Vec<Event>,
) {
    push_image_warnings(parsed, out);
    let node_kind = NodeKind::Custom(route.outer.name.clone());
    out.push(Event::StartNode(node_kind.clone()));

    // Emit component name for renderer template matching
    out.push(Event::Attribute {
        name: "name".to_string(),
        value: route.outer.name.clone(),
    });

    if route.outer_is_figure {
        // §7.1: the extras belong to the outermost node (here the `<figure>`),
        // its caption stays a child, and the `<img>` element nests inside (§11).
        push_container_attribute(parsed, out);
        push_custom_extra_attrs(parsed, out);
        match route.inner {
            Some(inner) => {
                let inner_kind = NodeKind::Custom(inner.name.clone());
                out.push(Event::StartNode(inner_kind.clone()));
                out.push(Event::Attribute {
                    name: "name".to_string(),
                    value: inner.name.clone(),
                });
                push_image_element_attrs(parsed, out);
                out.push(Event::EndNode(inner_kind));
            }
            None => push_img_element(parsed, out),
        }
        emit_caption_children(parsed, inline_pipeline, context, out);
        out.push(Event::EndNode(node_kind));
        return;
    }

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

    // Extra attributes (id, class, props, style) and the bare flags (§6.3):
    // custom Solid components receive the keys verbatim (unlike the HTML path).
    push_custom_extra_attrs(parsed, out);

    // Caption as rendered inline children with full inline pipeline support
    emit_caption_children(parsed, inline_pipeline, context, out);

    out.push(Event::EndNode(node_kind));
}

// --- Solid Hints Integration ---

/// Builds SolidRenderHints for the configured image components (the `img` and
/// `figure` layers, §11). Returns None when neither layer is configured, letting
/// the renderer fall back to its built-in handler.
pub fn solid_hints(options: &ImgOptions) -> Option<SolidRenderHints> {
    let mut hints = SolidRenderHints::default();
    // §11 rule 3: every entry of every layer answers its own instances.
    for custom in options.img.components().chain(options.figure.components()) {
        let key = (custom.name.clone(), Some(custom.name.clone()));
        hints.templates.push(ComponentTemplate {
            node_type: custom.name.clone(),
            node_name: Some(custom.name.clone()),
            template: custom.template.clone(),
        });

        if !custom.imports.is_empty() {
            hints.template_imports.insert(key, custom.imports.clone());
        }
    }

    if hints.templates.is_empty() {
        return None;
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
    // §7.1: an extras head alone makes the image advanced.
    let has_attrs = blocks.extras.is_some();
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

/// The §11 extras head attached after an image's `(url)` head (§7.1). It
/// attaches to the outermost node of the construct; the retired
/// `[.class,#id]{key: value}` block is literal text (§14).
struct AttachedBlocks<'a> {
    extras: Option<ExtrasHead>,
    /// Text left after the head: the figure caption or trailing text.
    rest: &'a str,
}

impl<'a> AttachedBlocks<'a> {
    fn empty(rest: &'a str) -> Self {
        Self { extras: None, rest }
    }
}

/// Parses the extras head attached to an image.
///
/// §4.1: the head must touch the closing `)` of the `(url)` block, so it is
/// scanned at the exact cursor position.
fn parse_attached_blocks(input: &str) -> AttachedBlocks<'_> {
    let chars: Vec<char> = input.chars().collect();
    match scan_extras_chars(&chars, 0) {
        Some((head, next)) => {
            let bytes: usize = chars[..next].iter().map(|ch| ch.len_utf8()).sum();
            AttachedBlocks {
                extras: Some(head),
                rest: &input[bytes..],
            }
        }
        None => AttachedBlocks::empty(input),
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
    let AttachedBlocks { extras, .. } = blocks;
    let mut flags = Vec::new();
    let mut warnings = Vec::new();
    let type_marker = extras.as_ref().and_then(|head| head.type_marker.clone());
    let mut attrs = match extras.as_ref() {
        Some(head) => image_attrs_from_head(head, &mut flags, &mut warnings),
        None => ImageAttrs::default(),
    };
    // §11 rule 3: the marker is the routing key of the instance; it is carried
    // as a `type` attribute so a `{attrs.type}` template can read it back. An
    // explicit `type:` prop keeps its own value.
    if let Some(marker) = &type_marker {
        if !has_attribute(&attrs, "type") {
            attrs
                .properties
                .insert(0, ("type".to_string(), marker.clone()));
        }
    }

    ParsedImage {
        alt: core.alt,
        src: core.src,
        caption,
        container,
        attrs,
        marker: core.marker,
        flags,
        type_marker,
        warnings,
    }
}

/// Whether the image head already sets `key`.
fn has_attribute(attrs: &ImageAttrs, key: &str) -> bool {
    match key {
        "id" => attrs.id.is_some(),
        "class" => !attrs.classes.is_empty(),
        _ => attrs.properties.iter().any(|(name, _)| name == key),
    }
}

/// Assembles the image attributes from a §7.1 extras head (§6).
///
/// `class` accumulates (§6.4), `slug` fills `id` when no `#id` is present
/// (§6.2) and bare flags stay bare attributes (§6.3).
fn image_attrs_from_head(
    head: &ExtrasHead,
    flags: &mut Vec<String>,
    warnings: &mut Vec<String>,
) -> ImageAttrs {
    let parsed = to_attributes(head, &ExtrasOptions::default());
    for warning in &parsed.warnings {
        warnings.push(pendon_extra::warning_message(warning));
    }

    let mut attrs = ImageAttrs::default();
    for (key, value) in &parsed.items {
        match (key.as_str(), value) {
            ("id", _) | ("slug", _) => {}
            ("class", ExtrasAttr::Value(value)) => attrs
                .classes
                .extend(value.literal().split_whitespace().map(str::to_string)),
            (name, ExtrasAttr::Flag) => flags.push(name.to_string()),
            (name, ExtrasAttr::Value(value)) => {
                attrs.properties.push((name.to_string(), value.literal()))
            }
        }
    }

    // §6.2: `#id` beats the extras `slug`.
    attrs.id = parsed
        .value("id")
        .or_else(|| parsed.value("slug"))
        .map(|value| value.literal());
    attrs
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
    if !input.starts_with('{') && !input.starts_with("@@") {
        return (AttachedBlocks::empty(input), 0);
    }

    let blocks = parse_attached_blocks(input);
    if blocks.extras.is_none() {
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
    match route_image(parsed, options) {
        Some(route) => emit_custom_image(parsed, &route, inline_pipeline, context, out),
        None => emit_element_image(parsed, inline_pipeline, context, out),
    }
}

fn push_common_attrs(out: &mut Vec<Event>, attrs: &ImageAttrs, flags: &[String]) {
    if let Some(id) = attrs.id.as_deref() {
        attribute(out, "id", id);
    }

    if !attrs.classes.is_empty() {
        attribute(out, "class", &attrs.classes.join(" "));
    }

    // HTML elements keep the `data-` prefix (custom components receive the
    // keys verbatim, see `push_custom_extra_attrs`), except the §11 routing key
    // `type`, which stays a plain attribute (§11 rule 3). The `style` value is
    // emitted as the `style` attribute itself (§6.3).
    for (k, v) in &attrs.properties {
        if k.starts_with("--") || k == "style" {
            continue;
        }
        if k == "type" {
            attribute(out, "type", v);
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

/// The `style` value of an image (§6.3).
fn image_style(attrs: &ImageAttrs) -> String {
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

/// §13 diagnostics raised while parsing and merging an image head.
fn push_image_warnings(parsed: &ParsedImage, out: &mut Vec<Event>) {
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
    use pendon_renderer_solid::TypedComponent;

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
            "![Alt](https://x.test/a.webp){.x, #hero, foo: \"bar\", --r: \"5deg\"}",
        );
        let out = process(&events, &ImgOptions::default(), &pipeline);

        assert!(out.iter().any(|event| matches!(
            event,
            Event::StartNode(NodeKind::Element(name)) if name == "img"
        )));
        assert!(has_attribute(&out, "id", "hero"));
        assert!(has_attribute(&out, "class", "x"));
        assert!(has_attribute(&out, "data-foo", "bar"));
        assert!(has_attribute(&out, "style", "--r: 5deg;"));
    }

    #[test]
    fn renders_inline_advanced_image_inside_paragraph() {
        let pipeline = Pipeline::default();
        let events = paragraph_events(
            "Ad ex tempor !?~[Alt](https://x.test/a.webp){.foo, con: \"jux\"} consectetur.",
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
            img: ComponentSet::new(),
            figure: ComponentSet::from_entries([TypedComponent::default_component(
                ImgCustomNode {
                    name: "AdvancedImage".into(),
                    template: "<AdvancedImage />".into(),
                    imports: Vec::new(),
                },
            )]),
        };
        let events =
            paragraph_events("!![Alt](https://x.test/a.webp){.hero, foo: \"bar\"} A caption");
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
    fn extras_class_accumulates() {
        let parsed = parse_figure_syntax("!![Alt](https://x.test/a.webp)@@figure{.head, .extra}")
            .expect("figure");
        // §6.4: class accumulates.
        assert_eq!(parsed.attrs.classes, vec!["head", "extra"]);
    }

    /// §14/D3: the retired `[.class,#id]{k:v}` block is literal text.
    #[test]
    fn the_retired_legacy_block_is_literal_text() {
        let pipeline = Pipeline::default();
        let events = paragraph_events("!![Alt](https://x.test/a.webp)[#hero]{k: \"v\"} Cap");
        let out = process(&events, &ImgOptions::default(), &pipeline);

        assert!(!has_attribute(&out, "id", "hero"));
        assert!(!out.iter().any(|event| matches!(
            event,
            Event::Diagnostic { message, .. } if message.contains("deprecated")
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
            img: ComponentSet::new(),
            figure: ComponentSet::from_entries([TypedComponent::default_component(
                ImgCustomNode {
                    name: "AdvancedImage".into(),
                    template: "<AdvancedImage />".into(),
                    imports: Vec::new(),
                },
            )]),
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
