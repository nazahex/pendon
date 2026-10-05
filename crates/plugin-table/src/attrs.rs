// src/attrs.rs
use pendon_core::Event;
use pendon_extra::{
    parse_attrs, to_attributes, ExtraAttrs as AttrSpec, ExtrasAttr, ExtrasHead, ExtrasOptions,
};

/// The attributes of one table layer (table/caption/thead/tbody/tfoot/row/cell).
///
/// §8 sources them from the `@@type{…}` heads (§11) and, for compatibility, the
/// pre-§11 `[.class,#id]{key: value}` blocks. Bare flags (§6.3) and the routing
/// marker (§11 rule 3) have no place in [`AttrSpec`], so they travel alongside.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LayerAttrs {
    pub attrs: AttrSpec,
    /// Bare flags of the extras head, emitted as bare attributes (§6.3).
    pub flags: Vec<String>,
    /// The `@@type{…}` marker: the §11 routing key of this layer instance.
    pub type_marker: Option<String>,
}

impl LayerAttrs {
    pub fn is_empty(&self) -> bool {
        self.attrs.id.is_none()
            && self.attrs.classes.is_empty()
            && self.attrs.properties.is_empty()
            && self.flags.is_empty()
    }
}

/// Parses the pre-§11 `[.class,#id]{key: value}` suffix (§14) and returns the
/// attributes plus the unconsumed text.
pub fn parse_attr_block(input: &str) -> (AttrSpec, &str) {
    let parsed = parse_attrs(input);
    (parsed.attrs, parsed.rest)
}

pub fn attr_block(input: &str) -> LayerAttrs {
    let (attrs, _) = parse_attr_block(input);
    LayerAttrs {
        attrs,
        flags: Vec::new(),
        type_marker: None,
    }
}

/// Maps a §11 extras head onto the layer's attributes (§6).
///
/// `class` accumulates head-first (§6.4), `#id`/`slug` fill the element id
/// (`#id` > slug, §6.2), `--var` items merge into `style` (§6.3) and bare flags
/// stay bare attributes. The marker is carried as a `type` property (first) so
/// both renderers see it; an explicit `type:` prop keeps its own value.
pub fn extras_to_layer(head: &ExtrasHead) -> LayerAttrs {
    let parsed = to_attributes(head, &ExtrasOptions::default());
    let mut layer = LayerAttrs {
        type_marker: head.type_marker.clone(),
        ..LayerAttrs::default()
    };

    let mut styles: Vec<String> = Vec::new();
    let mut props: Vec<(String, String)> = Vec::new();
    for (key, value) in &parsed.items {
        match (key.as_str(), value) {
            ("class", ExtrasAttr::Value(value)) => layer
                .attrs
                .classes
                .extend(value.literal().split_whitespace().map(str::to_string)),
            ("id", ExtrasAttr::Value(value)) => layer.attrs.id = Some(value.literal()),
            // §6.2: `slug` fills the id when nothing else did.
            ("slug", ExtrasAttr::Value(value)) => {
                if layer.attrs.id.is_none() {
                    layer.attrs.id = Some(value.literal());
                }
            }
            ("style", ExtrasAttr::Value(value)) => styles.push(value.literal()),
            (name, ExtrasAttr::Value(value)) if name.starts_with("--") => {
                styles.push(format!("{}:{}", name, value.literal()))
            }
            (name, ExtrasAttr::Value(value)) => props.push((name.to_string(), value.literal())),
            (name, ExtrasAttr::Flag) => layer.flags.push(name.to_string()),
        }
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
