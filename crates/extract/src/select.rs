//! Field selection: which leaves of an occurrence survive (§21 rule 4, D-6).
//!
//! A block selects with four keys, each a list of **dotted paths**
//! ([`pendon_extra::Path`], so `price.details.amount` is three segments and
//! `priced` is not under `price`):
//!
//! | key       | meaning                                                             |
//! | --------- | ------------------------------------------------------------------- |
//! | `preset`  | the starting set: `all` = every field, `pendon` = the element's list |
//! | `include` | **adds** paths to the starting set                                  |
//! | `only`    | keeps its own paths and nothing else                                |
//! | `omit`    | removes paths; **always wins**, over every other key                |
//!
//! Every key names a path whose **prefix selects the whole subtree** — so
//! `preset = "pendon"` + `include = ["price.details"]` +
//! `omit = ["price.details.tax"]` ships `price.details` minus `tax`, while a
//! leaf no key reaches never ships.
//!
//! ## The algebra
//!
//! For one **leaf** path `L` (a scalar, an array, or a whole object kept as a
//! value), in order:
//!
//! 1. `omit` — dropped when some omit path covers `L` (is `L` or an ancestor).
//!    Nothing after this can bring it back.
//! 2. `preset` + `include` — kept when the preset is `all`, or when a `pendon`
//!    preset name covers `L`, or when an `include` path covers `L`. `include`
//!    is additive: it never narrows what the preset already gave.
//! 3. `only`, when present — kept only when some `only` path covers `L`. `only`
//!    is a **filter, not an addition** (PE.4): it can only keep fields the
//!    preset or `include` already reached, so `only` naming a path outside the
//!    `pendon` preset empties the row.
//!
//! ## Objects
//!
//! Structure comes from wherever the field map put it: a dotted attribute name
//! (`soo.baz`), or a structured value that is an object (`price` → `details` →
//! `amount`). Both are walked the same way — an object is a container,
//! everything else is a leaf. A container survives **only when a child
//! survived**, so omitting every child never leaves an empty `{}` husk.
//!
//! ## Empty rows
//!
//! A selection that keeps nothing is still a row: `{}` (PE.4), because the
//! occurrence *does* carry fields and the user's filter matched none. Only an
//! occurrence with no fields at all emits no row (OBJ-7) — that decision belongs
//! to the caller, [`crate::run`], which owns the "one row per occurrence" loop.
//!
//! ## Path algebra
//!
//! | fields           | preset   | include         | only                   | omit                   | result                      |
//! | ---------------- | -------- | --------------- | ---------------------- | ---------------------- | --------------------------- |
//! | `href`, `price`  | `all`    | —               | —                      | —                      | both                        |
//! | `href`, `price`  | `pendon` | —               | —                      | —                      | `href`                      |
//! | `href`, `price`  | `pendon` | `price.details` | —                      | —                      | `href`, `price.details`     |
//! | `href`, `price`  | `all`    | —               | `price.details.amount` | —                      | `price.details.amount`      |
//! | `href`, `price`  | `pendon` | `price`         | —                      | `price.details.tax`    | `href`, `price` minus `tax` |
//! | `href`, `price`  | `pendon` | `href`          | —                      | `href`                 | `{}` — omit wins            |
//! | `href`, `price`  | `pendon` | —               | `price.details.amount` | `price.details.amount` | `{}` — PE.4                 |
//! | `href`, `priced` | `all`    | —               | `price`                | —                      | `{}` — `priced` ≠ `price.*` |

use pendon_extra::Path;
use serde_json::{Map, Value};

use crate::block::ExtractBlock;

/// The four selection keys of one block, resolved into one predicate over paths.
///
/// Built once per block (the lists are tiny) and applied to every occurrence of
/// that block's element. [`Selection::apply`] is the only writer of a row body:
/// it clones the surviving leaves and drops the rest, so nothing here can add a
/// field the occurrence does not carry.
#[derive(Clone, Debug, Default)]
pub struct Selection {
    /// The starting set, already parsed: `None` is `preset = "all"` — "no
    /// restriction", not "no fields".
    preset: Option<Vec<Path>>,
    include: Vec<Path>,
    only: Vec<Path>,
    omit: Vec<Path>,
}

impl Selection {
    /// The selection a validated block writes.
    ///
    /// The preset names come from [`PRESETS`](crate::fields::PRESETS), whose
    /// entries are compile-time field names, so parsing them cannot fail.
    pub fn new(block: &ExtractBlock) -> Self {
        Self {
            preset: block.preset.fields(block.element).map(|names| {
                names
                    .iter()
                    .map(|name| {
                        Path::from_dotted(name).expect("a preset field name is a valid path")
                    })
                    .collect()
            }),
            include: block.include.clone(),
            only: block.only.clone(),
            omit: block.omit.clone(),
        }
    }

    /// True when the leaf at `path` survives — the algebra, for one field.
    ///
    /// A *container* is never asked: it survives by having a surviving child
    /// ([`Selection::apply`]), which is what keeps `{}` husks out of the output.
    pub fn keeps(&self, path: &Path) -> bool {
        self.keeps_segments(path.segments())
    }

    /// The same test for a path already split into segments, which is how the
    /// walk carries it (one reused buffer, no per-node [`Path`]).
    fn keeps_segments(&self, path: &[String]) -> bool {
        if self.omit.iter().any(|omit| covers(omit, path)) {
            return false;
        }
        let started = match &self.preset {
            None => true,
            Some(names) => names.iter().any(|name| covers(name, path)),
        } || self.include.iter().any(|include| covers(include, path));
        started && (self.only.is_empty() || self.only.iter().any(|only| covers(only, path)))
    }

    /// The occurrence's fields, filtered: one object, the row body.
    ///
    /// An empty selection is `{}`, never an error and never the unfiltered set
    /// (PE.4). Order is the field map's own, minus what was dropped.
    pub fn apply(&self, fields: &Value) -> Value {
        self.filter(fields, &mut Vec::new())
            .unwrap_or_else(|| Value::Object(Map::new()))
    }

    /// Keeps the leaves `keeps` accepts and drops the containers that lose every
    /// child. `None` means "nothing under this node survived".
    fn filter(&self, value: &Value, path: &mut Vec<String>) -> Option<Value> {
        match value {
            Value::Object(entries) => {
                let mut kept = Map::new();
                for (name, child) in entries {
                    path.push(name.clone());
                    let surviving = self.filter(child, path);
                    path.pop();
                    if let Some(child) = surviving {
                        kept.insert(name.clone(), child);
                    }
                }
                (!kept.is_empty()).then_some(Value::Object(kept))
            }
            _ => self.keeps_segments(path).then(|| value.clone()),
        }
    }
}

/// True when `selector` names `path` or one of its ancestors.
///
/// Segment-wise by construction: `price` covers `price.details.amount` but never
/// `priced`, and a path covers itself, which is what makes a key for a leaf
/// also work for the subtree below it.
fn covers(selector: &Path, path: &[String]) -> bool {
    let segments = selector.segments();
    path.len() >= segments.len() && path[..segments.len()] == segments[..]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::BlockSpec;
    use serde_json::json;

    fn block(preset: Option<&str>, include: &[&str], only: &[&str], omit: &[&str]) -> ExtractBlock {
        ExtractBlock::build(
            BlockSpec {
                element: "anchor".to_string(),
                output: vec!["out/anchor.json".to_string()],
                format: vec!["json".to_string()],
                preset: preset.map(str::to_string),
                include: include.iter().map(|name| name.to_string()).collect(),
                only: only.iter().map(|name| name.to_string()).collect(),
                omit: omit.iter().map(|name| name.to_string()).collect(),
                ..BlockSpec::default()
            },
            0,
        )
        .expect("a valid block")
    }

    /// The PE.1 shape: a scalar, an array and two nested objects.
    fn occurrence() -> Value {
        json!({
            "href": "/buah/jambu_biji",
            "inner_text": "Jambu biji",
            "depth": 1,
            "number": { "type": "nomorBuah", "raw": ["A", "1"], "depth": 2 },
            "price": {
                "details": {
                    "currency": "IDR",
                    "amount": { "total": 12, "unit": "thousand" }
                }
            },
            "tags": ["a", "b"]
        })
    }

    /// D-6: `all` is "no restriction" — every field the occurrence carries.
    #[test]
    fn the_all_preset_keeps_every_field() {
        let selection = Selection::new(&block(Some("all"), &[], &[], &[]));
        assert_eq!(selection.apply(&occurrence()), occurrence());
    }

    /// The default preset is `pendon` (S2-8.6), so an absent `preset` and
    /// `preset = "pendon"` are the same selection.
    #[test]
    fn an_absent_preset_is_the_pendon_preset() {
        assert_eq!(
            Selection::new(&block(None, &[], &[], &[])).apply(&occurrence()),
            Selection::new(&block(Some("pendon"), &[], &[], &[])).apply(&occurrence())
        );
    }

    /// D-6: `pendon` is the element's own list, so what the author added
    /// (`price`, `tags`) and what only a preset assignment would publish
    /// (`number`, `depth` on an anchor) stay out.
    #[test]
    fn the_pendon_preset_keeps_the_elements_own_fields() {
        let selection = Selection::new(&block(Some("pendon"), &[], &[], &[]));
        assert_eq!(
            selection.apply(&occurrence()),
            json!({ "href": "/buah/jambu_biji", "inner_text": "Jambu biji" })
        );
    }

    /// PE.1: `include` adds a nested path, and only that path — the sibling
    /// `currency` the author also wrote is not part of it, while everything
    /// below `amount` is (a prefix selects the whole subtree).
    #[test]
    fn include_adds_a_nested_path_not_its_siblings() {
        let selection = Selection::new(&block(
            Some("pendon"),
            &["number.raw", "price.details.amount"],
            &[],
            &[],
        ));
        assert_eq!(
            selection.apply(&occurrence()),
            json!({
                "href": "/buah/jambu_biji",
                "inner_text": "Jambu biji",
                "number": { "raw": ["A", "1"] },
                "price": { "details": { "amount": { "total": 12, "unit": "thousand" } } }
            })
        );
    }

    /// A key naming a container keeps the whole subtree; `omit` still cuts
    /// inside it, because `omit` is applied per leaf and wins.
    #[test]
    fn omit_wins_inside_an_included_subtree() {
        let selection = Selection::new(&block(
            Some("pendon"),
            &["price"],
            &[],
            &["price.details.amount.unit"],
        ));
        assert_eq!(
            selection.apply(&occurrence()),
            json!({
                "href": "/buah/jambu_biji",
                "inner_text": "Jambu biji",
                "price": {
                    "details": {
                        "currency": "IDR",
                        "amount": { "total": 12 }
                    }
                }
            })
        );
    }

    /// PE.3: `only` keeps its own subtrees — and nothing else, however much the
    /// `all` preset offers.
    #[test]
    fn only_keeps_its_subtrees_and_nothing_else() {
        let selection = Selection::new(&block(
            Some("all"),
            &[],
            &["number.raw", "price.details.amount"],
            &[],
        ));
        assert_eq!(
            selection.apply(&occurrence()),
            json!({
                "number": { "raw": ["A", "1"] },
                "price": { "details": { "amount": { "total": 12, "unit": "thousand" } } }
            })
        );
    }

    /// PE.4: `only` cannot add. Both names are outside the anchor preset, so the
    /// selection is empty — `{}`, not an error and not a quiet fall back.
    #[test]
    fn an_only_outside_the_preset_is_an_empty_row() {
        let selection = Selection::new(&block(
            Some("pendon"),
            &[],
            &["number.raw", "price.details.amount"],
            &[],
        ));
        assert_eq!(selection.apply(&occurrence()), json!({}));
    }

    /// `omit` naming everything under a container removes the container itself:
    /// no `{}` husk in the middle of a row.
    #[test]
    fn a_container_that_lost_every_child_is_dropped() {
        let selection = Selection::new(&block(
            Some("pendon"),
            &["price.details.currency", "price.details.amount"],
            &[],
            &["price.details.currency", "price.details.amount"],
        ));
        assert_eq!(
            selection.apply(&occurrence()),
            json!({ "href": "/buah/jambu_biji", "inner_text": "Jambu biji" })
        );
    }

    /// Paths are segment-wise: `priced` is not `price` and `pri` is not a prefix
    /// of anything here.
    #[test]
    fn a_path_is_matched_by_segments_never_by_text() {
        let selection = Selection::new(&block(Some("all"), &[], &["priced", "pri"], &[]));
        assert_eq!(selection.apply(&occurrence()), json!({}));
    }

    /// [`Selection::keeps`] is the same algebra without a tree, so a caller can
    /// ask about one field name directly.
    #[test]
    fn keeps_answers_for_a_single_path() {
        let selection = Selection::new(&block(Some("pendon"), &[], &[], &["inner_text"]));
        assert!(selection.keeps(&Path::from_dotted("href").unwrap()));
        assert!(!selection.keeps(&Path::from_dotted("inner_text").unwrap()));
        assert!(!selection.keeps(&Path::from_dotted("price.details").unwrap()));
    }
}
