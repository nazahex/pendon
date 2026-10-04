// src/attrs.rs
use pendon_extra::parse_attrs;
pub use pendon_extra::ExtraAttrs as AttrSpec;

pub fn parse_attr_block(input: &str) -> (AttrSpec, &str) {
    let parsed = parse_attrs(input);
    (parsed.attrs, parsed.rest)
}

pub fn merge_attrs(base: &mut AttrSpec, overlay: &AttrSpec) {
    if let Some(id) = &overlay.id {
        base.id = Some(id.clone());
    }
    base.classes.extend(overlay.classes.clone());
    // Properties must be carried over too, otherwise a cell level
    // `{ key: value }` block is silently dropped when the grid is built.
    for (key, value) in &overlay.properties {
        base.properties.retain(|(existing, _)| existing != key);
        base.properties.push((key.clone(), value.clone()));
    }
}
