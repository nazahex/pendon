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

/// The ten layer-tagged elements `plugin-extract` understands, in the order the
/// inventory documents them (§2-§11).
///
/// It is the same set `number` numbers: an element is extractable exactly
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

/// The one element that is **not** preset-driven: the document's own frontmatter
/// (D-14).
///
/// It is not in [`ELEMENTS`] on purpose — it has no element fields and therefore
/// no [`PRESETS`] row: its fields are whatever the author wrote, its paths are
/// the nested keys, and `only` / `omit` are the whole selection.
pub const FRONTMATTER: &str = "frontmatter";

/// The element a config names, frontmatter included.
pub fn any_element(name: &str) -> Option<&'static str> {
    if name == FRONTMATTER {
        Some(FRONTMATTER)
    } else {
        element(name)
    }
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
/// dropped from that occurrence ([`crate::select`]), so an inert name costs
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

#[cfg(test)]
mod tests {
    use super::*;

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
