// src/attrs.rs
use crate::parser::Align;
use pendon_core::Event;
use pendon_extra::{to_attributes, ExtrasAttr, ExtrasHead, Specs};

/// The attributes of one table layer (table/caption/thead/tbody/tfoot/row/cell).
///
/// §8 sources them from the `{…}` / `@@type{…}` extras heads (§11); the pre-§11
/// `[.class,#id]{key: value}` blocks are retired and stay literal text (§14).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LayerSpec {
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub properties: Vec<(String, String)>,
}

/// One table layer's extras: the values above, plus the §6.3 bare flags and the
/// §11 routing marker.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LayerAttrs {
    pub attrs: LayerSpec,
    /// Bare flags of the extras head, emitted as bare attributes (§6.3).
    pub flags: Vec<String>,
    /// The `type` marker: the §11 routing key of this layer instance.
    pub type_marker: Option<String>,
}

/// Maps a §11 extras head onto the layer's attributes (§6).
///
/// `class` accumulates (§6.4), `#id`/`slug` fill the element id
/// (`#id` > slug, §6.2), `--var` items merge into `style` (§6.3) and bare flags
/// stay bare attributes. The marker is carried as a `type` property (first) so
/// both renderers see it; an explicit `type:` prop keeps its own value.
pub fn extras_to_layer(head: &ExtrasHead) -> LayerAttrs {
    let specs = Specs::default();
    let parsed = to_attributes(head, &specs);
    let mut layer = LayerAttrs {
        type_marker: head.type_marker.clone(),
        ..LayerAttrs::default()
    };

    let mut styles: Vec<String> = Vec::new();
    let mut props: Vec<(String, String)> = Vec::new();
    // §6.2: the extras slug (the resolved `tick` path) is the id fallback.
    let extras_slug = parsed.value(&specs.tick).map(|value| value.literal());
    // §6.3: a layer's own positions — `class` / `id` / `style` are the universal
    // DOM fields and an extras `"title"` is the `title`.
    for (key, value) in specs.dom_fields(&parsed.items, &[(&specs.quote, "title")]) {
        if key == specs.tick {
            continue;
        }
        match (key, value) {
            ("class", ExtrasAttr::Value(value)) => layer
                .attrs
                .classes
                .extend(value.literal().split_whitespace().map(str::to_string)),
            ("id", ExtrasAttr::Value(value)) => layer.attrs.id = Some(value.literal()),
            ("style", ExtrasAttr::Value(value)) => styles.push(value.literal()),
            (name, ExtrasAttr::Value(value)) if name.starts_with("--") => {
                styles.push(format!("{}:{}", name, value.literal()))
            }
            (name, ExtrasAttr::Value(value)) => props.push((name.to_string(), value.literal())),
            (name, ExtrasAttr::Flag) => layer.flags.push(name.to_string()),
        }
    }
    if layer.attrs.id.is_none() {
        layer.attrs.id = extras_slug;
    }

    if let Some(marker) = &layer.type_marker {
        if !props.iter().any(|(name, _)| name == "type") {
            props.insert(0, ("type".to_string(), marker.clone()));
        }
    }
    layer.attrs.properties = props;

    if !styles.is_empty() {
        let style = styles.join("; ");
        match layer
            .attrs
            .properties
            .iter_mut()
            .find(|(name, _)| name == "style")
        {
            Some(slot) => slot.1 = style,
            None => layer.attrs.properties.push(("style".to_string(), style)),
        }
    }

    layer
}

/// Merges `overlay` into `base`: the id of the overlay wins, classes accumulate
/// and properties are last-wins. Used to fold column attributes into a cell and
/// the head (construct) side over the extras.
pub fn merge_layer(base: &mut LayerAttrs, overlay: &LayerAttrs) {
    if let Some(id) = &overlay.attrs.id {
        base.attrs.id = Some(id.clone());
    }
    base.attrs.classes.extend(overlay.attrs.classes.clone());
    for (key, value) in &overlay.attrs.properties {
        base.attrs
            .properties
            .retain(|(existing, _)| existing != key);
        base.attrs.properties.push((key.clone(), value.clone()));
    }
    for flag in &overlay.flags {
        if !base.flags.contains(flag) {
            base.flags.push(flag.clone());
        }
    }
    if overlay.type_marker.is_some() {
        base.type_marker = overlay.type_marker.clone();
    }
}

/// Pushes §6.3 bare flags as bare attributes.
pub fn push_flags(out: &mut Vec<Event>, flags: &[String]) {
    for name in flags {
        out.push(Event::AttributeFlag { name: name.clone() });
    }
}

// --- emission (§6): the two shapes a layer's attributes can take -------------

/// How a layer's attributes reach the output, decided **once per document and
/// per layer** (ADR-0008 `D-3a`).
///
/// A layer that has a component set is emitted in [`Shape::Props`]: the values
/// stay typed (`align="left"`, `data-*`, `--var` merged into `style`) because a
/// template reads them through `{attrs.*}` / `{...attrs}`. A layer with no
/// component is emitted in [`Shape::Html`]: the browser needs the fallback
/// spelling (`style="text-align: left;"`). The choice is made per layer and
/// never per element, so a `_` (remainder) or `*` (everything) selector can
/// never see two shapes of one layer in the same document.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// No component: DOM-shaped values (`style`, no column typing).
    Html,
    /// A component answers this layer: typed props, no `style` rewrite.
    Props,
}

impl Shape {
    /// The shape of a layer that does (`true`) or does not (`false`) have a
    /// component set.
    pub fn of(configured: bool) -> Self {
        if configured {
            Shape::Props
        } else {
            Shape::Html
        }
    }
}

/// Emits the attributes of a container layer (`table`, `caption`, `thead`,
/// `tbody`, `tfoot`, `tr`) in the shape the layer was given.
pub fn push_layer_attrs(out: &mut Vec<Event>, layer: &LayerAttrs, shape: Shape) {
    match shape {
        Shape::Props => push_props_attrs(out, layer),
        Shape::Html => push_html_attrs(out, layer),
    }
}

/// Component shape: keys verbatim (`data-` prefixes are an HTML-fallback
/// concern) plus the §6.3 bare flags.
fn push_props_attrs(out: &mut Vec<Event>, layer: &LayerAttrs) {
    if let Some(id) = &layer.attrs.id {
        attribute(out, "id", id);
    }
    if !layer.attrs.classes.is_empty() {
        attribute(out, "class", &layer.attrs.classes.join(" "));
    }
    for (key, value) in &layer.attrs.properties {
        attribute(out, key, value);
    }
    push_flags(out, &layer.flags);
}

/// Fallback shape: `--var` properties fold into one `style` attribute (§6.3),
/// empty values are dropped, bare flags stay bare.
fn push_html_attrs(out: &mut Vec<Event>, layer: &LayerAttrs) {
    if let Some(id) = layer.attrs.id.as_deref() {
        attribute(out, "id", id);
    }
    if !layer.attrs.classes.is_empty() {
        attribute(out, "class", &layer.attrs.classes.join(" "));
    }
    for (key, value) in &layer.attrs.properties {
        if key.starts_with("--") {
            continue;
        }
        attribute(out, key, value);
    }
    let styles: Vec<String> = layer
        .attrs
        .properties
        .iter()
        .filter(|(key, _)| key.starts_with("--"))
        .map(|(key, value)| format!("{}:{};", key, value))
        .collect();
    if !styles.is_empty() {
        attribute(out, "style", &styles.concat());
    }
    push_flags(out, &layer.flags);
}

/// Emits the attributes of a `th` / `td` cell in the shape the `cell` layer was
/// given. `colspan` / `rowspan` are emitted only when a span is real, so a
/// template can still read them as an empty string for single cells.
pub fn push_cell_attrs(
    out: &mut Vec<Event>,
    layer: &LayerAttrs,
    align: Align,
    width: Option<&str>,
    colspan: usize,
    rowspan: usize,
    shape: Shape,
) {
    match shape {
        Shape::Props => push_cell_props(out, layer, align, width, colspan, rowspan),
        Shape::Html => push_cell_html(out, layer, align, width, colspan, rowspan),
    }
}

/// Component shape: alignment and width stay typed props, `--var` entries keep
/// their `style` fallback after the spans.
fn push_cell_props(
    out: &mut Vec<Event>,
    layer: &LayerAttrs,
    align: Align,
    width: Option<&str>,
    colspan: usize,
    rowspan: usize,
) {
    if let Some(id) = &layer.attrs.id {
        attribute(out, "id", id);
    }
    if !layer.attrs.classes.is_empty() {
        attribute(out, "class", &layer.attrs.classes.join(" "));
    }
    for (key, value) in &layer.attrs.properties {
        if key.starts_with("--") || value.is_empty() {
            continue;
        }
        attribute(out, key, value);
    }
    if align != Align::None {
        attribute(out, "align", &align_to_string(align));
    }
    if let Some(width) = width.filter(|width| !width.is_empty()) {
        attribute(out, "width", width);
    }
    push_spans(out, colspan, rowspan);
    let styles: Vec<String> = layer
        .attrs
        .properties
        .iter()
        .filter(|(key, _)| key.starts_with("--"))
        .map(|(key, value)| format!("{}:{}", key, value))
        .collect();
    if !styles.is_empty() {
        attribute(out, "style", &styles.join("; "));
    }
    push_flags(out, &layer.flags);
}

/// Fallback shape: alignment and width become CSS, `--var` entries join them in
/// the same `style` attribute, spans come last.
fn push_cell_html(
    out: &mut Vec<Event>,
    layer: &LayerAttrs,
    align: Align,
    width: Option<&str>,
    colspan: usize,
    rowspan: usize,
) {
    if let Some(id) = layer.attrs.id.as_deref() {
        attribute(out, "id", id);
    }
    if !layer.attrs.classes.is_empty() {
        attribute(out, "class", &layer.attrs.classes.join(" "));
    }
    for (key, value) in &layer.attrs.properties {
        if key.starts_with("--") {
            continue;
        }
        attribute(out, key, value);
    }

    let mut styles = Vec::new();
    match align {
        Align::Left => styles.push("text-align: left".to_string()),
        Align::Center => styles.push("text-align: center".to_string()),
        Align::Right => styles.push("text-align: right".to_string()),
        Align::None => {}
    }
    if let Some(width) = width {
        styles.push(format!("width: {}", width));
    }
    for (key, value) in &layer.attrs.properties {
        if key.starts_with("--") {
            styles.push(format!("{}:{}", key, value));
        }
    }
    if !styles.is_empty() {
        attribute(out, "style", &format!("{};", styles.join("; ")));
    }

    push_spans(out, colspan, rowspan);
    push_flags(out, &layer.flags);
}

fn push_spans(out: &mut Vec<Event>, colspan: usize, rowspan: usize) {
    if colspan > 1 {
        attribute(out, "colspan", &colspan.to_string());
    }
    if rowspan > 1 {
        attribute(out, "rowspan", &rowspan.to_string());
    }
}

fn align_to_string(align: Align) -> String {
    match align {
        Align::Left => "left".to_string(),
        Align::Center => "center".to_string(),
        Align::Right => "right".to_string(),
        Align::None => String::new(),
    }
}

fn attribute(out: &mut Vec<Event>, name: &str, value: &str) {
    out.push(Event::Attribute {
        name: name.to_string(),
        value: value.to_string(),
    });
}
