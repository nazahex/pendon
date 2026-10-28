//! `plugin-image` — the `!` construct (§7.1).
//!
//! `![alt](src)` is **one head**: `!` is the tag that claims it (D-11), the
//! bracket is the preset's `alt`, the paren its `src`, and every other claimed
//! position survives as a bare field (D-7). The extras head decorates the same
//! `<img>` node, because there is no second node any more: the marker run
//! (`?`, `~`, `w<n>`, `h<n>`, `!!`, `p!`, `d!`), the trailing caption and the
//! `<figure>`/`<figcaption>` pair are retired (§14, D-16).

use pendon_core::{element_close, element_open, raw_inline, Event, NodeKind, Severity};
use pendon_extra::{
    issue_event, parse_extras, parse_slots, to_attributes, ElementHead, ExtrasAttr, ExtrasHead,
    ExtrasIssue, ExtrasMatch, FieldConfig, FieldName, Preset, Slot, SlotContract, SlotError,
    SlotKind, SlotMatch, SlotPair, SlotPos, SlotTag, SpecsKey,
};
use pendon_renderer_solid::{layer_templates, ComponentSet, ImportEntry, SolidRenderHints};

// --- Slot head (§7.1, §6.1) ---

/// The tag that claims this element's head (§5.3, D-11).
///
/// `![…](…)` without a bracket is an ordinary CommonMark image: the bracket and
/// paren belong to whoever contracts them, and `plugin-markdown` handles it.
const TAG: char = '!';

/// Every position image claims: `![alt "bracket quote" `bracket tick`](src
/// "paren quote" `paren tick`)`.
///
/// A claimed head is read whole (§6.1): a content the preset does not name is
/// not dropped, it becomes a bare field under its slot-qualified path (D-7).
const SLOTS: &[SlotPos] = &[
    (Slot::Bracket, SlotKind::Bare),
    (Slot::Bracket, SlotKind::Quote),
    (Slot::Bracket, SlotKind::Tick),
    (Slot::Paren, SlotKind::Bare),
    (Slot::Paren, SlotKind::Quote),
    (Slot::Paren, SlotKind::Tick),
];

/// §7.1's preset assignment: `bracket.bare` → `alt`, `paren.bare` → `src`.
///
/// Both are optional (D-2: no element may refuse a head for looking
/// incomplete), so `![alt]` and `![]()` are images too — what they mean is the
/// document's business, not the parser's.
const PRESET: &[Preset] = &[
    Preset::new(Slot::Bracket, SlotKind::Bare, FieldName::new("alt")),
    Preset::new(Slot::Paren, SlotKind::Bare, FieldName::new("src")),
];

/// D-12 rule 5: the two fields `<img>` writes itself, so an extras item that
/// spells one is an `Error` (`field.override.alt` publishes it under another
/// name). Everything else is an ordinary extras key.
const OWNED: &[FieldName] = &[FieldName::new("alt"), FieldName::new("src")];

/// The tag that claims image's head (D-11), the positions it claims, and where
/// each lands when the user configures nothing (§6.1, §6.5, D-7).
pub const HEAD: ElementHead = ElementHead::new(
    SlotContract {
        accepted: SLOTS,
        tag: SlotTag::Image,
    },
    PRESET,
)
.with_owned(OWNED);

/// The extras positions this element reads as a DOM field of its own (§6.3): an
/// extras `"quote"` is the `<img>`'s `title`, while `class` / `id` / `style` are
/// the universal ones. Every other extras path is already a field of its own.
const ELEMENT_READS: &[(SpecsKey, &str)] = &[(SpecsKey::Quote, "title")];

// --- Configuration ---

#[derive(Clone, Debug, Default)]
pub struct ImageOptions {
    /// §11 `img` layer: the `<img>` element itself, and the only layer there is
    /// (the `figure` container is retired, §14).
    pub img: ComponentSet<ImageCustomNode>,
}

#[derive(Clone, Debug, Default)]
pub struct ImageCustomNode {
    pub name: String,
    pub template: String,
    pub imports: Vec<ImportEntry>,
}

// --- §11 layers (ADR-0008 `D-3a`) ---

/// §11 primary layer of `plugin-image`: the `<img>` element itself.
const IMG_LAYER: &str = "img";

/// The layer key `[[task.image.custom]]` addresses (always the primary).
pub fn primary_layer() -> &'static str {
    IMG_LAYER
}

/// Every §11 layer this plugin emits, in the order a config may name them.
pub fn layers() -> [&'static str; 1] {
    [IMG_LAYER]
}

/// Whether the `img` layer has a component, i.e. whether the node keeps its
/// fields verbatim (`AdvancedImage`) or is written as a plain `<img>` whose
/// extras props carry the `data-` prefix.
fn img_props(options: &ImageOptions) -> bool {
    !options.img.is_empty()
}

/// Tags a node with the §11 layer that owns it (§11 rule 3, `D-3a`).
fn push_layer(out: &mut Vec<Event>) {
    out.push(Event::Attribute {
        name: "__layer".to_string(),
        value: IMG_LAYER.to_string(),
    });
}

// --- Solid Hints Integration ---

/// Builds SolidRenderHints for the configured image component (the `img` layer,
/// §11). Returns None when the layer is not configured, letting the renderer fall
/// back to its built-in handler.
pub fn solid_hints(options: &ImageOptions) -> Option<SolidRenderHints> {
    if options.img.is_empty() {
        return None;
    }
    // §11 rule 3 / D-3a: one template per entry, tagged with its own layer; the
    // renderer decides per document which entry answers which `type`.
    let payload = |custom: &ImageCustomNode| (custom.template.clone(), custom.imports.clone());
    let mut hints = SolidRenderHints::default();
    hints
        .templates
        .extend(layer_templates(IMG_LAYER, &options.img, payload));
    Some(hints)
}

// --- Main Processor ---

/// Replaces every claimed image head (§7.1) with its `<img>` element.
///
/// A paragraph whose whole content is one head becomes a **block** image: the
/// `<img>` replaces the paragraph (the newline is the block separator the old
/// `HtmlBlock` rendering carried). A head that shares its line with text stays
/// **inline**, inside the paragraph, and the text around it is untouched — there
/// is no caption any more, so `![alt](src) Caption` is an image followed by an
/// ordinary word.
pub fn process(events: &[Event], options: &ImageOptions) -> Vec<Event> {
    let mut out: Vec<Event> = Vec::with_capacity(events.len());
    let mut i = 0usize;

    while i < events.len() {
        if matches!(events.get(i), Some(Event::StartNode(NodeKind::Paragraph))) {
            if let Some(end) = find_matching_end(events, i, NodeKind::Paragraph) {
                let block = &events[i + 1..end];
                if let Some(parts) = block_image(block, options) {
                    emit_image(&parts, options, &mut out);
                    out.extend(raw_inline("\n"));
                    i = end + 1;
                    continue;
                }
                // A paragraph that is pure text is scanned for inline heads; one
                // that already carries nodes is left to whoever produced them.
                if !block.is_empty() && block.iter().all(|ev| matches!(ev, Event::Text(_))) {
                    out.push(events[i].clone());
                    for ev in block {
                        match ev {
                            Event::Text(text) => emit_text_with_images(text, options, &mut out),
                            other => out.push(other.clone()),
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

/// The head of a paragraph that carries nothing else, i.e. a **block** image.
///
/// `None` when the block is not one line, when the line does not start with the
/// tag, or when text follows the head and its extras — then the line is running
/// text and the image is inline.
fn block_image(block: &[Event], options: &ImageOptions) -> Option<ImageParts> {
    let mut line = String::new();
    for event in block {
        match event {
            Event::Text(text) => line.push_str(text),
            _ => return None,
        }
    }
    let line = line.trim();
    if line.is_empty() || line.contains('\n') || !line.starts_with(TAG) {
        return None;
    }
    match read_head(line, options) {
        Read::Image { parts, consumed } if line[consumed..].trim().is_empty() => Some(parts),
        _ => None,
    }
}

/// What one `!` in the text turned out to be (§7.1).
enum Read {
    /// Not an image head — a lone `!`, or a bracket another tag claims.
    Literal,
    /// Slot-shaped but illegal (§4.3, D-2): `Error` plus literal text.
    Malformed(SlotError),
    /// A complete head: the fields the `<img>` carries and the bytes it used.
    Image { parts: ImageParts, consumed: usize },
}

/// Reads one image head at the cursor: the tagged slots plus the extras head
/// that must touch the closing `)` (§4.1).
fn read_head(rest: &str, options: &ImageOptions) -> Read {
    debug_assert!(rest.starts_with(TAG), "the cursor sits on the tag");

    let (pair, mut consumed) = match parse_slots(rest) {
        SlotMatch::Slots { pair, rest: after } if pair.tag == SlotTag::Image => {
            (pair, rest.len() - after.len())
        }
        // Only a **tagged** head is ours to complain about: an untagged malformed
        // bracket belongs to whoever else wants it (§4.3).
        SlotMatch::Malformed { error, .. } if rest.starts_with(TAG) => {
            return Read::Malformed(error)
        }
        _ => return Read::Literal,
    };

    // §4.1: the extras head must touch the closing `)`.
    let extras = match parse_extras(&rest[consumed..]) {
        ExtrasMatch::Head { head, rest: after } => {
            consumed = rest.len() - after.len();
            Some(head)
        }
        ExtrasMatch::Absent { .. } | ExtrasMatch::Malformed { .. } => None,
    };

    Read::Image {
        parts: build_parts(&pair, extras.as_ref(), options, consumed),
        consumed,
    }
}

// --- Fields (§6.2, §6.3) ---

/// What one `<img>` carries: its fields in head-then-extras order (§6.2/§6.4),
/// the names the construct head produced, and the bare flags of the extras head.
///
/// The construct writes no other field: `alt` and `src` come from the preset, and
/// every other position keeps its slot-qualified path, so a claimed head never
/// drops a content (D-7).
#[derive(Debug, Default)]
struct ImageParts {
    /// `alt`, `src`, the bare slot fields and every extras key, in order.
    values: Vec<(String, String)>,
    /// The names the construct head produced: they win over the extras (§6.2).
    head_keys: Vec<String>,
    /// §6.3: a bare flag stays a bare attribute (`<img isLazy>`).
    flags: Vec<String>,
    /// §13 diagnostics raised while resolving the head and the extras.
    issues: Vec<ExtrasIssue>,
    /// The bytes the whole head used, extras included.
    consumed: usize,
}

impl ImageParts {
    fn set(&mut self, name: &str, value: String) {
        match self.values.iter_mut().find(|(key, _)| key == name) {
            Some((_, slot)) => *slot = value,
            None => self.values.push((name.to_string(), value)),
        }
    }

    fn get(&self, name: &str) -> Option<&str> {
        self.values
            .iter()
            .find_map(|(key, value)| (key == name).then_some(value.as_str()))
    }

    fn has(&self, name: &str) -> bool {
        self.get(name).is_some()
    }
}

/// Turns the two slots and the extras head into the fields the `<img>` exposes
/// (§6.5 preset, §6.2 priority, §6.3 projection).
///
/// `field` is the **matched** entry's `field.*` (D-7): when it configures one,
/// every preset row is void for this element and each position keeps its
/// slot-qualified name, which the image writes as an attribute rather than
/// dropping.
fn build_parts(
    pair: &SlotPair,
    extras: Option<&ExtrasHead>,
    options: &ImageOptions,
    consumed: usize,
) -> ImageParts {
    let mut parts = ImageParts::default();

    // §6.5 / D-7: the preset is voided by the **matched** entry, never by another
    // entry's `field.*`. No extras head means no `type` marker, which is exactly
    // the entry the renderer picks (the layer default) — so a layer default's
    // `field.*` voids the preset for every untagged image on the page.
    let marker = extras.and_then(|head| head.type_marker.as_deref());
    let matched = options.img.field_for(marker);
    let fields = HEAD.assign(pair, matched).unwrap_or_else(|error| {
        // The entry's assignments contradict each other (one path is a prefix of
        // another), so they cannot all be published. A config clash is the
        // author's bug: say so and keep the preset.
        parts.issues.push(ExtrasIssue::error(format!(
            "the matched entry's `field.*` assignments clash ({error}); the preset is kept"
        )));
        HEAD.assign(pair, None)
            .expect("an untouched preset cannot clash")
    });
    // §7.1's preset names, and only those: a moved or renamed position
    // (`field.bracket.bare = "label"`) publishes a field of the author's own,
    // which the image neither renames nor reads (D-4, §6.1).
    for (path, value) in fields.iter() {
        let name = path.dotted();
        parts.head_keys.push(name.clone());
        parts.set(&name, value.literal());
    }

    if let Some(extras) = extras {
        merge_extras(&mut parts, extras, options);
    }

    // §6.3: an empty `alt=""` is meaningful — it marks a decorative image — so the
    // preset position renders even when the bracket is empty. A preset the entry
    // voided writes no `alt` at all: the author moved the position.
    if matched.is_none_or(FieldConfig::is_empty_slots) && !parts.has("alt") {
        parts.set("alt", String::new());
    }
    parts.consumed = consumed;
    parts
}

/// Merges a §7.1 extras head into the image's fields.
///
/// The construct side wins (§6.2): an item that spells a name the head produced
/// is dropped with a warning, `alt`/`src` are owned so an item spelling one is an
/// `Error` (D-12 rule 5), `class` accumulates (§6.4) and a bare flag stays a bare
/// attribute (§6.3).
fn merge_extras(parts: &mut ImageParts, extras: &ExtrasHead, options: &ImageOptions) {
    // §6.1/§11 rule 5: the entry answering this marker names the extras specs.
    let specs = options.img.specs_for(extras.type_marker.as_deref());
    let parsed = to_attributes(extras, &specs);
    parts.issues.extend(parsed.issues.iter().cloned());

    // §6.2/§6.3: the tick position feeds the id fallback **while it keeps its
    // default path**; `field.specs.tick = "x"` moves it to a field of the
    // author's own, which the image then does not read.
    let tick = specs.element_path(SpecsKey::Tick);
    let specs_id = tick
        .and_then(|path| parsed.value(path))
        .map(|value| value.literal());

    for (key, value) in specs.dom_fields(&parsed.items, ELEMENT_READS) {
        if tick == Some(key) {
            continue;
        }
        if HEAD.owns(key) && parts.has(key) {
            // D-12's built-in collision: `<img>` writes `alt`/`src` itself, so an
            // extras item that spells one is an `Error`, and `field.override.alt`
            // is the way to publish it under another name. A preset the entry
            // voided writes neither, so the key is an ordinary prop again.
            parts.issues.push(ExtrasIssue::owned_key(key));
            continue;
        }
        match value {
            ExtrasAttr::Flag => {
                if parts.has(key) {
                    parts.issues.push(ExtrasIssue::overridden(key));
                } else {
                    parts.flags.push(key.to_string());
                }
            }
            ExtrasAttr::Value(value) => {
                let text = value.literal();
                // §6.4: `class` accumulates into the one `class` attribute.
                if key == "class" {
                    let merged = match parts.get("class") {
                        Some(existing) => format!("{existing} {text}"),
                        None => text,
                    };
                    parts.set("class", merged);
                    continue;
                }
                if parts.has(key) {
                    parts.issues.push(ExtrasIssue::overridden(key));
                    continue;
                }
                parts.set(key, text);
            }
        }
    }

    // §6.2: `#id` beats the extras `slug` (the tick position, at its default path).
    if let Some(id) = specs_id {
        if parts.has("id") {
            parts.issues.push(ExtrasIssue::warning(
                "extras `slug` was dropped because the element already has an id",
            ));
        } else {
            parts.set("id", id);
        }
    }

    // §11 rule 3: the `@@type{…}` marker is the routing key of the instance, so a
    // `{attrs.type}` template can read it back; an explicit `type:` prop keeps
    // its own value.
    if let Some(marker) = extras.type_marker.as_deref() {
        if !parts.has("type") {
            parts.set("type", marker.to_string());
        }
    }
}

// --- Emission (§11 rule 3 / `D-3a`) ---

/// Emits the image as one `<img>` element, tagged with the `img` layer.
///
/// A component receives every field verbatim; the built-in `<img>` keeps the
/// `data-` prefix for an extras prop it has no attribute for (§6.3), so an
/// unknown key never lands on the element as a fake attribute.
fn emit_image(parts: &ImageParts, options: &ImageOptions, out: &mut Vec<Event>) {
    for issue in &parts.issues {
        out.push(issue_event("img", issue));
    }

    out.extend(element_open("img"));
    push_layer(out);
    let props = img_props(options);
    for (name, value) in &parts.values {
        // §6.3: `style` is a declaration list, so a value that came from `--x:` /
        // `style:` ends with the terminator the attribute spelling expects.
        let mut value = value.clone();
        if name == "style" && !value.ends_with(';') {
            value.push(';');
        }
        if props || is_verbatim(name, &parts.head_keys) {
            attribute(out, name, &value);
        } else {
            attribute(out, &format!("data-{name}"), &value);
        }
    }
    // §6.3: a bare flag stays a bare attribute (`<img isLazy>`), it never becomes
    // `isLazy="isLazy"`.
    for name in &parts.flags {
        out.push(Event::AttributeFlag { name: name.clone() });
    }
    out.push(element_close("img"));
}

/// The names the `<img>` writes as they are, whatever the layer: the construct's
/// own fields (§7.1's preset and the bare slot fields), the universal DOM fields
/// and the §11/§6.3 routing keys. Everything else is an extras prop, which the
/// built-in `<img>` carries under the `data-` prefix (the fallback's own
/// convention, unchanged by the rework).
fn is_verbatim(name: &str, head_keys: &[String]) -> bool {
    head_keys.iter().any(|key| key == name)
        // §7.1's preset names are the element's own fields even when the head left
        // the position empty: `alt=""` is a decorative image, never `data-alt`.
        || matches!(name, "alt" | "src" | "id" | "class" | "style" | "type")
        || name.starts_with("__")
}

/// Emits `text` with every claimed image head (§7.1) replaced by its `<img>`.
///
/// A `!` that is not a head — the second `!` of the retired `!!`, the `!` of a
/// cite's `[^…]`, a lone `!` — stays exactly as written.
fn emit_text_with_images(text: &str, options: &ImageOptions, out: &mut Vec<Event>) {
    let mut normal = String::new();
    let mut cursor = 0usize;

    while cursor < text.len() {
        let rest = &text[cursor..];
        // A code span is never an image: markdown renders its content verbatim, so
        // a head written inside backticks has to survive (`a `![a](b)` b`). A run
        // of n backticks closes at the next run of exactly n (CommonMark's rule).
        if rest.starts_with('`') {
            let run = rest.chars().take_while(|ch| *ch == '`').count();
            if let Some(close) = find_code_span_end(&rest[run..], run) {
                let end = run + close;
                normal.push_str(&rest[..end]);
                cursor += end;
                continue;
            }
        }
        if rest.starts_with(TAG) {
            match read_head(rest, options) {
                Read::Image { parts, consumed } => {
                    flush_text(&mut normal, out);
                    emit_image(&parts, options, out);
                    cursor += consumed;
                    continue;
                }
                // §4.3 / D-2: slot-shaped but illegal — an `Error`, and the text
                // stays exactly as it was written.
                Read::Malformed(error) => {
                    flush_text(&mut normal, out);
                    out.push(Event::Diagnostic {
                        severity: Severity::Error,
                        message: format!("[img] {error}"),
                        span: None,
                    });
                    normal.push(TAG);
                    cursor += TAG.len_utf8();
                    continue;
                }
                Read::Literal => {}
            }
        }
        let ch = rest.chars().next().expect("cursor is in bounds");
        normal.push(ch);
        cursor += ch.len_utf8();
    }

    flush_text(&mut normal, out);
}

fn flush_text(normal: &mut String, out: &mut Vec<Event>) {
    if !normal.is_empty() {
        out.push(Event::Text(std::mem::take(normal)));
    }
}

/// The byte offset just past the closing run of exactly `run` backticks, relative
/// to `text` (which starts immediately after the opening run).
///
/// `None` when the span never closes: an unclosed run is literal text, exactly as
/// markdown reads it.
fn find_code_span_end(text: &str, run: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        if bytes[cursor] == b'`' {
            let start = cursor;
            while cursor < bytes.len() && bytes[cursor] == b'`' {
                cursor += 1;
            }
            if cursor - start == run {
                return Some(cursor);
            }
        } else {
            cursor += 1;
        }
    }
    None
}

fn attribute(out: &mut Vec<Event>, name: &str, value: &str) {
    out.push(Event::Attribute {
        name: name.to_string(),
        value: value.to_string(),
    });
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
    use pendon_extra::{FieldConfig, Path};
    use pendon_renderer_solid::TypedComponent;

    /// §7.1's preset, position by position. This is the element's contract as
    /// data: a change here changes the syntax, not the plumbing.
    #[test]
    fn the_head_declares_the_spec_preset() {
        assert_eq!(HEAD.contract.tag, SlotTag::Image);
        assert!(
            HEAD.is_consistent(),
            "a preset must sit on a claimed position"
        );
        // Every position of the head is claimed; the preset names two of them.
        for slot in Slot::ALL {
            for kind in SlotKind::ALL {
                assert!(
                    HEAD.contract.accepts(slot, kind),
                    "image drops {}.{}",
                    slot.name(),
                    kind.name()
                );
            }
        }
        let got: Vec<String> = HEAD
            .presets()
            .iter()
            .map(|preset| {
                format!(
                    "{}.{} -> {}",
                    preset.slot.name(),
                    preset.kind.name(),
                    preset.name
                )
            })
            .collect();
        assert_eq!(got, ["bracket.bare -> alt", "paren.bare -> src"]);
        // D-12 rule 5: `<img>` writes these itself, so extras may not spell them.
        assert!(HEAD.owns("alt"));
        assert!(HEAD.owns("src"));
        assert!(!HEAD.owns("title"));
    }

    fn paragraph_events(text: &str) -> Vec<Event> {
        vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Paragraph),
            Event::Text(text.to_string()),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ]
    }

    /// The attributes of the first `<img>` element, in emission order.
    fn img_attrs(events: &[Event]) -> Vec<(String, String)> {
        let mut attrs = Vec::new();
        let mut inside = false;
        for event in events {
            match event {
                Event::StartNode(NodeKind::Element(name)) if name == "img" && !inside => {
                    inside = true;
                }
                Event::EndNode(NodeKind::Element(name)) if name == "img" && inside => return attrs,
                Event::Attribute { name, value } if inside => {
                    attrs.push((name.clone(), value.clone()));
                }
                _ => {}
            }
        }
        attrs
    }

    fn attr_value(events: &[Event], name: &str) -> Option<String> {
        img_attrs(events)
            .into_iter()
            .find_map(|(key, value)| (key == name).then_some(value))
    }

    fn has_attribute(events: &[Event], name: &str, value: &str) -> bool {
        attr_value(events, name).as_deref() == Some(value)
    }

    /// Text is emitted as many small chunks (sometimes one per character), so
    /// concatenate before searching.
    fn text_of(events: &[Event]) -> String {
        let mut text = String::new();
        for event in events {
            if let Event::Text(chunk) = event {
                text.push_str(chunk);
            }
        }
        text
    }

    fn has_text(events: &[Event], needle: &str) -> bool {
        text_of(events).contains(needle)
    }

    fn has_element(events: &[Event], tag: &str) -> bool {
        events
            .iter()
            .any(|event| matches!(event, Event::StartNode(NodeKind::Element(name)) if name == tag))
    }

    #[test]
    fn a_bare_image_is_an_image() {
        // §7.1's new minimum: `![alt](src)` needs nothing else. It used to need a
        // marker modifier, so this line reached `plugin-markdown` untouched.
        let events = paragraph_events("![Alt](https://x.test/a.webp)");
        let out = process(&events, &ImageOptions::default());

        assert!(has_attribute(&out, "alt", "Alt"));
        assert!(has_attribute(&out, "src", "https://x.test/a.webp"));
        assert!(has_attribute(&out, "__layer", "img"));
        // A block image: the paragraph is gone, and a newline separates the block.
        assert!(!out
            .iter()
            .any(|event| matches!(event, Event::StartNode(NodeKind::Paragraph))));
        assert!(has_text(&out, "\n"));
    }

    #[test]
    fn a_shared_line_stays_inside_its_paragraph() {
        let out = process(
            &paragraph_events("Ad ex tempor ![Alt](https://x.test/a.webp) consectetur."),
            &ImageOptions::default(),
        );

        assert!(out
            .iter()
            .any(|event| matches!(event, Event::StartNode(NodeKind::Paragraph))));
        assert!(has_attribute(&out, "alt", "Alt"));
        assert!(has_text(&out, "Ad ex tempor "));
        assert!(has_text(&out, " consectetur."));
    }

    #[test]
    fn trailing_text_is_not_a_caption() {
        // The caption belonged to the retired figure syntax (§14): text after the
        // head is ordinary paragraph content, and the image stays inline.
        let out = process(
            &paragraph_events("![Alt](https://x.test/a.webp) A caption with **markup**."),
            &ImageOptions::default(),
        );

        assert!(has_attribute(&out, "alt", "Alt"));
        assert!(has_text(&out, " A caption with **markup**."));
        assert!(!has_element(&out, "figure"));
        assert!(!has_element(&out, "figcaption"));
    }

    #[test]
    fn the_retired_marker_run_is_literal_text() {
        // `~?!![](src)` used to be a lazy, async figure with explicit dimensions.
        // The run is gone (D-16), so only the `![…](…)` part is a head and the
        // symbols stay exactly as written.
        let out = process(
            &paragraph_events("~?!![](https://x.test/a.webp)"),
            &ImageOptions::default(),
        );

        assert!(has_text(&out, "~?!"), "{}", text_of(&out));
        assert!(has_attribute(&out, "src", "https://x.test/a.webp"));
        assert!(attr_value(&out, "loading").is_none());
        assert!(attr_value(&out, "width").is_none());
        assert!(!has_element(&out, "figure"));
    }

    #[test]
    fn the_retired_container_prefixes_are_literal_text() {
        // `p!` / `d!` wrapped the image in `<p>` / `<div>`; a component renders its
        // own wrapper now, so the letter is ordinary text.
        let out = process(
            &paragraph_events("p![Alt](https://x.test/a.webp)"),
            &ImageOptions::default(),
        );

        assert!(has_text(&out, "p"), "{}", text_of(&out));
        assert!(has_attribute(&out, "alt", "Alt"));
        assert!(attr_value(&out, "container").is_none());
        assert!(!has_element(&out, "div"));
    }

    #[test]
    fn bare_slot_fields_keep_their_paths() {
        // D-7: a claimed head is read whole, so a content the preset does not name
        // is a field of its own rather than dropped.
        let out = process(
            &paragraph_events(
                "![Alt \"bracket quote\" `bracket tick`](src \"paren quote\" `paren tick`)",
            ),
            &ImageOptions::default(),
        );

        assert!(has_attribute(&out, "alt", "Alt"));
        assert!(has_attribute(&out, "src", "src"));
        assert!(has_attribute(&out, "bracket.quote", "bracket quote"));
        assert!(has_attribute(&out, "bracket.tick", "bracket tick"));
        assert!(has_attribute(&out, "paren.quote", "paren quote"));
        assert!(has_attribute(&out, "paren.tick", "paren tick"));
    }

    #[test]
    fn an_extras_alt_is_an_error() {
        // D-12 rule 5: `<img>` writes `alt` and `src` itself.
        let out = process(
            &paragraph_events("![Alt](https://x.test/a.webp){alt: \"other\", .x}"),
            &ImageOptions::default(),
        );

        assert!(out.iter().any(|event| matches!(
            event,
            Event::Diagnostic { severity: Severity::Error, message, .. }
                if message.contains("`alt` is owned by the construct")
        )));
        assert!(has_attribute(&out, "alt", "Alt"));
        assert_eq!(
            img_attrs(&out)
                .iter()
                .filter(|(name, _)| name == "alt")
                .count(),
            1
        );
        // The rest of the extras head still decorates the element.
        assert!(has_attribute(&out, "class", "x"));
    }

    #[test]
    fn extras_decorate_the_same_node() {
        let out = process(
            &paragraph_events(
                "![Alt](https://x.test/a.webp){.x, #hero, foo: \"bar\", --r: \"5deg\"}",
            ),
            &ImageOptions::default(),
        );

        assert!(has_attribute(&out, "id", "hero"));
        assert!(has_attribute(&out, "class", "x"));
        // §6.3: an unknown key never lands on `<img>` as a fake attribute.
        assert!(has_attribute(&out, "data-foo", "bar"));
        assert!(attr_value(&out, "foo").is_none());
        // §6.3: `--r: …` folds into `style`.
        assert!(has_attribute(&out, "style", "--r: 5deg;"));
    }

    #[test]
    fn extras_class_accumulates() {
        let Read::Image { parts, .. } = read_head(
            "![Alt](https://x.test/a.webp){.head, .extra, class: \"tail\"}",
            &ImageOptions::default(),
        ) else {
            panic!("a head");
        };
        // §6.4: `class` accumulates.
        assert_eq!(parts.get("class"), Some("head extra tail"));
    }

    #[test]
    fn extras_slug_fills_the_id_when_no_id_is_present() {
        let Read::Image { parts, .. } = read_head(
            "![Alt](https://x.test/a.webp){`the-image`}",
            &ImageOptions::default(),
        ) else {
            panic!("a head");
        };
        // §6.2: `#id` > extras slug.
        assert_eq!(parts.get("id"), Some("the-image"));
    }

    #[test]
    fn bare_flags_stay_bare_attributes() {
        let out = process(
            &paragraph_events("![Alt](https://x.test/a.webp){isFoo}"),
            &ImageOptions::default(),
        );

        assert!(out
            .iter()
            .any(|event| matches!(event, Event::AttributeFlag { name } if name == "isFoo")));
    }

    /// ADR-0010 `D-9`: `__number` is internal transport, not an extra prop, so the
    /// HTML fallback passes it verbatim instead of `data-`-prefixing it (the
    /// renderers strip it; `plugin-number` reads it).
    #[test]
    fn internal_number_attribute_is_not_prefixed() {
        let out = process(
            &paragraph_events("![Alt](https://x.test/a.webp)%%numberA"),
            &ImageOptions::default(),
        );

        assert!(has_attribute(&out, "__layer", "img"));
        assert!(has_attribute(
            &out,
            "__number",
            r#"{"depth":1,"type":"numberA"}"#
        ));
        assert!(attr_value(&out, "data-__number").is_none());
        // The numbering marker is part of the head, so the line is still a block.
        assert!(!out
            .iter()
            .any(|event| matches!(event, Event::StartNode(NodeKind::Paragraph))));
    }

    #[test]
    fn a_slot_quote_is_a_bare_field() {
        // D-7: only the preset's two rows are `alt`/`src`; a quote in the bracket
        // is `bracket.quote` and a quote in the paren is `paren.quote`.
        let out = process(
            &paragraph_events("![\"the quote\"](https://x.test/a.webp \"paren quote\")"),
            &ImageOptions::default(),
        );

        assert!(has_attribute(&out, "src", "https://x.test/a.webp"));
        assert!(has_attribute(&out, "bracket.quote", "the quote"));
        assert!(has_attribute(&out, "paren.quote", "paren quote"));
        // §6.3: an empty preset position still renders, so this is a decorative
        // image rather than one with a missing `alt`.
        assert!(has_attribute(&out, "alt", ""));
    }

    #[test]
    fn an_empty_head_is_still_an_image() {
        // D-2: no element refuses a head for looking incomplete — the document
        // decides what a sourceless image means.
        let out = process(&paragraph_events("![]()"), &ImageOptions::default());

        assert!(has_element(&out, "img"));
        // §6.3: the decorative image keeps its empty `alt`.
        assert!(has_attribute(&out, "alt", ""));
        assert!(attr_value(&out, "src").is_none());
        assert!(!out
            .iter()
            .any(|event| matches!(event, Event::Diagnostic { .. })));
    }

    #[test]
    fn a_component_receives_every_field_verbatim() {
        let options = ImageOptions {
            img: ComponentSet::from_entries([TypedComponent::default_component(ImageCustomNode {
                name: "AdvancedImage".into(),
                template: "<AdvancedImage {...attrs} />".into(),
                imports: Vec::new(),
            })]),
        };
        let events = paragraph_events("![Alt](https://x.test/a.webp)@@imageX{foo: \"bar\", .hero}");

        // The plain document (no entry) keeps the `data-` prefix…
        let out = process(&events, &ImageOptions::default());
        assert!(has_attribute(&out, "data-foo", "bar"));

        // …a configured layer passes every key through, `type` included.
        let out = process(&events, &options);
        assert!(has_attribute(&out, "foo", "bar"));
        assert!(has_attribute(&out, "class", "hero"));
        assert!(has_attribute(&out, "type", "imageX"));
        assert!(has_attribute(&out, "__layer", "img"));
    }

    #[test]
    fn a_moved_slot_is_a_field_of_the_author() {
        // D-7/D-4: one `field.<slot>.<kind>` assignment voids the whole preset for
        // this entry, so every position keeps its slot-qualified name.
        let mut field = FieldConfig::new();
        field.assign_slot(
            Slot::Bracket,
            SlotKind::Bare,
            Some(Path::from_dotted("label").expect("a path")),
        );
        let options = ImageOptions {
            img: ComponentSet::from_entries([TypedComponent::default_component(ImageCustomNode {
                name: "AdvancedImage".into(),
                template: "<AdvancedImage {...attrs} />".into(),
                imports: Vec::new(),
            })
            .with_field(field)]),
        };
        let out = process(&paragraph_events("![Alt](https://x.test/a.webp)"), &options);

        assert!(has_attribute(&out, "label", "Alt"));
        // The paren keeps its slot-qualified name: the preset is void as a whole
        // (§6.1), so nothing is called `src` any more.
        assert!(has_attribute(&out, "paren.bare", "https://x.test/a.webp"));
        assert!(attr_value(&out, "alt").is_none());
        assert!(attr_value(&out, "src").is_none());
        assert!(!out
            .iter()
            .any(|event| matches!(event, Event::Diagnostic { .. })));
    }

    #[test]
    fn malformed_extras_stay_literal_text() {
        let out = process(
            &paragraph_events("Ad ex tempor ![Alt](https://x.test/a.webp)@@figure{.a ex."),
            &ImageOptions::default(),
        );

        // The head itself is complete, so the image is emitted…
        assert!(has_attribute(&out, "alt", "Alt"));
        // …and the unterminated extras stay what they were.
        assert!(has_text(&out, "@@figure{.a ex."));
    }

    #[test]
    fn a_head_inside_a_code_span_stays_text() {
        // A code span is verbatim by definition, and the plugin runs before
        // markdown: writing the syntax down must not produce an image.
        let out = process(
            &paragraph_events("Write `![a](b)` to embed, then ![c](d) really."),
            &ImageOptions::default(),
        );

        assert!(has_text(&out, "`![a](b)`"), "{}", text_of(&out));
        assert_eq!(
            img_attrs(&out)
                .iter()
                .filter(|(name, _)| name == "src")
                .count(),
            1
        );
        assert!(has_attribute(&out, "src", "d"));
    }

    #[test]
    fn a_malformed_head_is_an_error_and_stays_literal() {
        let out = process(
            &paragraph_events("Ad ex tempor ![Alt](/a \"b.webp) consectetur."),
            &ImageOptions::default(),
        );

        assert!(out.iter().any(|event| matches!(
            event,
            Event::Diagnostic { severity: Severity::Error, message, .. } if message.starts_with("[img]")
        )));
        assert!(!has_element(&out, "img"));
        assert!(has_text(&out, "![Alt](/a \"b.webp)"));
    }
}
