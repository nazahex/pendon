//! The frozen field vocabulary.
//!
//! The single source of truth is
//! [`docs/spec/FIELDS-INVENTORY.md`](../../../docs/spec/FIELDS-INVENTORY.md): a
//! name here that is not a row there (or a `preset` mark that disagrees with
//! §12) is a bug, and this module's tests are written against the document.
//!
//! [`PRESETS`] is that document's §12 in code — one table, so a preset cannot be
//! spelled two ways. `apps/cli/tests/fields_inventory_spec.rs` parses §12 and the
//! per-element tables **out of the document** and compares them with it, so a
//! rename on either side is a failing test rather than a stale doc (S2-7.4).

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
        "img" => "image",
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

/// The ten elements' preset field lists, in the order the inventory documents
/// them — §12 of
/// [`docs/spec/FIELDS-INVENTORY.md`](../../../docs/spec/FIELDS-INVENTORY.md).
///
/// One table, because two readers share it: [`pendon_preset`] at run time and
/// `apps/cli/tests/fields_inventory_spec.rs`, which parses §12 out of the
/// document and fails when a name here and a name there disagree — in either
/// direction, order included. Renaming a preset field is therefore a two-file
/// change or a test failure (S2-7.4).
///
/// A name no element produces today is still legal here (the document marks it
/// `*`): a preset *recommends* names, and a field an occurrence does not carry is
/// dropped from that occurrence ([`select_fields`]), so an inert name costs
/// nothing.
pub const PRESETS: [(&str, &[&str]); 10] = [
    ("anchor", &["href", "id", "title", "type", "inner_text"]),
    ("blockquote", &["type", "id", "class", "inner_text"]),
    (
        "cite",
        &[
            "reference_id",
            "locator",
            "citation_number",
            "cite-id",
            "type",
        ],
    ),
    (
        "directive",
        &["type", "id", "level", "summary", "inner_text"],
    ),
    (
        "heading",
        &[
            "depth",
            "id",
            "slug",
            "title",
            "number",
            "type",
            "inner_text",
        ],
    ),
    ("image", &["src", "alt", "width", "height", "type", "id"]),
    ("list", &["type", "id", "start", "inner_text"]),
    ("marker", &["type", "slug", "id", "title", "inner_text"]),
    ("section", &["id", "type", "class", "depth"]),
    ("table", &["type", "id", "caption"]),
];

/// The `preset = "pendon"` field list of `element` — §12 of the inventory, read
/// from [`PRESETS`].
///
/// An unknown element has no preset; [`element`] rejects it at config time, so an
/// empty slice here is unreachable from a validated block.
pub fn pendon_preset(element: &str) -> &'static [&'static str] {
    PRESETS
        .iter()
        .find(|(candidate, _)| *candidate == element)
        .map(|(_, fields)| *fields)
        .unwrap_or(&[])
}

/// The fields of one occurrence: the preset, then `include`, `only` and `omit`,
/// with `omit` always winning (D-6, P.3).
///
/// `available` is the occurrence's own field list in attribute order, so the
/// result keeps emission order rather than the order of the `include` / `only`
/// arrays, and a field the node does not carry is simply absent (never emitted
/// empty). `include` and `only` never meet — [`ExtractBlock::build`] rejects the
/// pair — so the two filters cannot disagree about a name.
pub fn select_fields(block: &ExtractBlock, available: &[String]) -> Vec<String> {
    let preset = block.preset.fields(block.element);
    available
        .iter()
        // `omit` is last in the prose and first in the code: whatever the other
        // two keys said, an omitted name never ships (P.3).
        .filter(|name| !block.omit.iter().any(|omit| omit == *name))
        // The preset: `None` is `all` (no restriction), `Some(list)` keeps the
        // element's own names plus whatever `include` adds.
        .filter(|name| match preset {
            Some(fields) => {
                fields.contains(&name.as_str()) || block.include.iter().any(|add| add == *name)
            }
            None => true,
        })
        .filter(|name| block.only.is_empty() || block.only.iter().any(|only| only == *name))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::BlockSpec;

    fn block(element: &str, preset: Option<&str>, only: &[&str], omit: &[&str]) -> ExtractBlock {
        ExtractBlock::build(
            BlockSpec {
                element: element.to_string(),
                output: vec!["out/x.json".to_string()],
                format: vec!["json".to_string()],
                preset: preset.map(str::to_string),
                only: only.iter().map(|name| name.to_string()).collect(),
                omit: omit.iter().map(|name| name.to_string()).collect(),
                ..BlockSpec::default()
            },
            0,
        )
        .expect("a valid spec")
    }

    fn available(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| name.to_string()).collect()
    }

    /// The D-6 replay: the preset is the starting set, `include` adds to it, and
    /// `omit` wins over both. `only` narrows in the same test, after the preset.
    #[test]
    fn presets_narrow_through_only_and_omit() {
        let occurrence = available(&[
            "reference_id",
            "locator",
            "citation_number",
            "cite-id",
            "class",
        ]);
        assert_eq!(
            select_fields(&block("cite", Some("pendon"), &[], &[]), &occurrence),
            ["reference_id", "locator", "citation_number", "cite-id"]
        );
        assert_eq!(
            select_fields(
                &block("cite", Some("pendon"), &["citation_number"], &[]),
                &occurrence
            ),
            ["citation_number"]
        );
        assert_eq!(
            select_fields(
                &block("cite", Some("pendon"), &["locator"], &["locator"]),
                &occurrence
            ),
            Vec::<String>::new()
        );
        assert_eq!(
            select_fields(&block("cite", None, &[], &["locator"]), &occurrence),
            ["reference_id", "citation_number", "cite-id", "class"]
        );
    }

    /// `preset = "all"` is the whole occurrence, internals aside — the pre-D-6
    /// default, now spelled out instead of implied.
    #[test]
    fn the_all_preset_keeps_every_field() {
        let occurrence = available(&["href", "title", "rel", "soo.baz"]);
        assert_eq!(
            select_fields(&block("anchor", Some("all"), &[], &[]), &occurrence),
            ["href", "title", "rel", "soo.baz"]
        );
    }

    /// A block with `include` (and never `only` — D-6 rejects that pair).
    fn block_with_include(
        element: &str,
        preset: &str,
        include: &[&str],
        omit: &[&str],
    ) -> ExtractBlock {
        ExtractBlock::build(
            BlockSpec {
                element: element.to_string(),
                output: vec!["out/x.json".to_string()],
                format: vec!["json".to_string()],
                preset: Some(preset.to_string()),
                include: include.iter().map(|name| name.to_string()).collect(),
                omit: omit.iter().map(|name| name.to_string()).collect(),
                ..BlockSpec::default()
            },
            0,
        )
        .expect("a valid spec")
    }

    /// D-6: `include` adds to the preset instead of replacing it. With `all`
    /// there is nothing to add, and D-6 calls that redundant but valid.
    #[test]
    fn include_adds_to_the_preset() {
        let occurrence = available(&["reference_id", "citation_number", "class", "rel"]);
        assert_eq!(
            select_fields(
                &block_with_include("cite", "pendon", &["class"], &[]),
                &occurrence
            ),
            ["reference_id", "citation_number", "class"]
        );
        // Redundant with `all`, not an error (D-6).
        assert_eq!(
            select_fields(
                &block_with_include("cite", "all", &["class"], &[]),
                &occurrence
            ),
            ["reference_id", "citation_number", "class", "rel"]
        );
    }

    /// `omit` is the last word, `include` or not (P.3).
    #[test]
    fn omit_wins_over_include() {
        let occurrence = available(&["href", "class"]);
        assert_eq!(
            select_fields(
                &block_with_include("anchor", "pendon", &["class"], &["class"]),
                &occurrence
            ),
            ["href"]
        );
    }

    /// PE.4 / D-6: a preset narrowed by a name nothing carries is **empty
    /// output**, not an error and not a quiet fall back to the whole preset.
    #[test]
    fn a_preset_with_an_unmatched_only_is_empty() {
        let occurrence = available(&["reference_id", "locator"]);
        assert_eq!(
            select_fields(
                &block("cite", Some("pendon"), &["no_such_field"], &[]),
                &occurrence
            ),
            Vec::<String>::new()
        );
    }

    /// A field the node does not carry is dropped, even when `only` names it.
    #[test]
    fn an_absent_field_is_dropped_not_emptied() {
        let occurrence = available(&["href", "id"]);
        assert_eq!(
            select_fields(
                &block("anchor", Some("pendon"), &["href", "title"], &[]),
                &occurrence
            ),
            ["href"]
        );
    }

    /// Attribute order wins over the order of the `only` array.
    #[test]
    fn selection_keeps_emission_order() {
        let occurrence = available(&["href", "id", "title"]);
        assert_eq!(
            select_fields(&block("anchor", None, &["title", "href"], &[]), &occurrence),
            ["href", "title"]
        );
    }

    #[test]
    fn every_preset_is_non_empty_and_leaks_no_internal_key() {
        for element in ELEMENTS {
            let preset = pendon_preset(element);
            assert!(!preset.is_empty(), "{element} has a preset");
            for field in preset {
                assert!(
                    !field.starts_with("__"),
                    "{element}'s preset must not leak an internal key ({field})"
                );
            }
        }
    }

    /// [`ELEMENTS`] and [`PRESETS`] are two spellings of one list: every
    /// extractable element has a preset row, in the same order, and the table
    /// names nothing [`element`] would reject.
    #[test]
    fn the_preset_table_and_the_element_list_agree() {
        let named: Vec<&str> = PRESETS.iter().map(|(name, _)| *name).collect();
        assert_eq!(named, ELEMENTS);
        for (name, _) in PRESETS {
            assert_eq!(element(name), Some(name));
        }
    }

    /// The lookup is the table: an unknown element gets an empty slice, never a
    /// neighbour's fields.
    #[test]
    fn an_unknown_element_has_no_preset() {
        assert!(pendon_preset("figure").is_empty());
        assert_eq!(pendon_preset("cite").len(), 5);
    }

    #[test]
    fn every_layer_maps_to_an_element() {
        for layer in [
            "heading",
            "anchor",
            "img",
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
