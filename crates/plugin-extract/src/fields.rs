//! The frozen field vocabulary.
//!
//! The single source of truth is
//! [`docs/spec/EXTRAS-INVENTORY.md`](../../../docs/spec/EXTRAS-INVENTORY.md): a
//! name here that is not a row there (or a `recommended` mark that disagrees with
//! §12) is a bug, and this module's tests are written against the document.

use crate::block::ExtractBlock;

/// The ten layer-tagged elements `plugin-extract` understands, in the order the
/// inventory documents them (§2-§11).
///
/// It is the same set `plugin-number` numbers: an element is extractable exactly
/// when its plugin tags the node with `__layer`. The five other in-scope elements
/// of §8.6 are explained in §1 of the inventory.
pub const ELEMENTS: [&str; 10] = [
    "anchor",
    "blockquote",
    "cite",
    "directive",
    "heading",
    "image",
    "list",
    "marker",
    "section",
    "table",
];

/// The element a `__layer` value belongs to, or `None` when the layer is outside
/// the extract surface (`thead`/`tbody`/`tfoot`, a custom plugin's layer, ...).
pub fn element_of_layer(layer: &str) -> Option<&'static str> {
    Some(match layer {
        "heading" => "heading",
        "anchor" => "anchor",
        "img" | "figure" => "image",
        "table" => "table",
        "marker" => "marker",
        "blockquote" => "blockquote",
        "directive" => "directive",
        "list" | "unordered" | "ordered" => "list",
        "cite" => "cite",
        "section" => "section",
        _ => return None,
    })
}

/// The element a config names, or `None` when it is not extractable.
pub fn element(name: &str) -> Option<&'static str> {
    ELEMENTS
        .iter()
        .copied()
        .find(|candidate| *candidate == name)
}

/// The `recommended = true` field list of `element` — §12 of the inventory.
///
/// An unknown element has no preset; [`element`] rejects it at config time, so an
/// empty slice here is unreachable from a validated block.
pub fn recommended(element: &str) -> &'static [&'static str] {
    match element {
        "anchor" => &["href", "id", "title", "type", "inner_text"],
        "blockquote" => &["type", "id", "class", "inner_text"],
        "cite" => &[
            "citation_number",
            "instance_index",
            "cite-id",
            "loc",
            "type",
        ],
        "directive" => &["type", "id", "level", "summary", "inner_text"],
        "heading" => &[
            "level",
            "id",
            "slug",
            "raw_title",
            "number",
            "type",
            "inner_text",
        ],
        "image" => &["src", "alt", "width", "height", "type", "id"],
        "list" => &["type", "id", "start", "inner_text"],
        "marker" => &["type", "slug", "id", "title", "inner_text"],
        "section" => &["id", "type", "class", "depth"],
        "table" => &["type", "id", "caption"],
        _ => &[],
    }
}

/// The fields of one occurrence: `recommended` → `only` → `omit`, in that order,
/// with `omit` always winning (P.3).
///
/// `available` is the occurrence's own field list in attribute order, so the
/// result keeps emission order rather than the order of the `only` array, and a
/// field the node does not carry is simply absent (never emitted empty).
pub fn select_fields(block: &ExtractBlock, available: &[String]) -> Vec<String> {
    let preset = recommended(block.element);
    available
        .iter()
        .filter(|name| !block.omit.iter().any(|omit| omit == *name))
        .filter(|name| !block.recommended || preset.contains(&name.as_str()))
        .filter(|name| block.only.is_empty() || block.only.iter().any(|only| only == *name))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::BlockSpec;

    fn block(element: &str, recommended: bool, only: &[&str], omit: &[&str]) -> ExtractBlock {
        ExtractBlock::build(
            BlockSpec {
                element: element.to_string(),
                output: vec!["out/x.json".to_string()],
                format: vec!["json".to_string()],
                only: only.iter().map(|name| name.to_string()).collect(),
                omit: omit.iter().map(|name| name.to_string()).collect(),
                recommended,
                ..BlockSpec::default()
            },
            0,
        )
        .expect("a valid spec")
    }

    fn available(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| name.to_string()).collect()
    }

    /// The P.3 replay: preset + `only` + `omit` intersect in that order, so a
    /// preset field that `only` excludes is gone and `omit` wins over both.
    #[test]
    fn presets_narrow_through_only_and_omit() {
        let occurrence = available(&[
            "citation_number",
            "instance_index",
            "cite-id",
            "loc",
            "class",
        ]);
        assert_eq!(
            select_fields(&block("cite", true, &[], &[]), &occurrence),
            ["citation_number", "instance_index", "cite-id", "loc"]
        );
        assert_eq!(
            select_fields(&block("cite", true, &["citation_number"], &[]), &occurrence),
            ["citation_number"]
        );
        assert_eq!(
            select_fields(&block("cite", true, &["loc"], &["loc"]), &occurrence),
            Vec::<String>::new()
        );
        assert_eq!(
            select_fields(&block("cite", false, &[], &["loc"]), &occurrence),
            ["citation_number", "instance_index", "cite-id", "class"]
        );
    }

    /// A field the node does not carry is dropped, even when `only` names it.
    #[test]
    fn an_absent_field_is_dropped_not_emptied() {
        let occurrence = available(&["href", "id"]);
        assert_eq!(
            select_fields(&block("anchor", true, &["href", "title"], &[]), &occurrence),
            ["href"]
        );
    }

    /// Attribute order wins over the order of the `only` array.
    #[test]
    fn selection_keeps_emission_order() {
        let occurrence = available(&["href", "id", "title"]);
        assert_eq!(
            select_fields(
                &block("anchor", false, &["title", "href"], &[]),
                &occurrence
            ),
            ["href", "title"]
        );
    }

    #[test]
    fn every_preset_is_non_empty_and_leaks_no_internal_key() {
        for element in ELEMENTS {
            let preset = recommended(element);
            assert!(!preset.is_empty(), "{element} has a preset");
            for field in preset {
                assert!(
                    !field.starts_with("__"),
                    "{element}'s preset must not leak an internal key ({field})"
                );
            }
        }
    }

    #[test]
    fn every_layer_maps_to_an_element() {
        for layer in [
            "heading",
            "anchor",
            "img",
            "figure",
            "table",
            "marker",
            "blockquote",
            "directive",
            "list",
            "unordered",
            "ordered",
            "cite",
            "section",
        ] {
            let mapped = element_of_layer(layer).expect("a known layer");
            assert!(element(mapped).is_some(), "{mapped} is listed");
        }
        assert_eq!(element_of_layer("thead"), None);
    }
}
