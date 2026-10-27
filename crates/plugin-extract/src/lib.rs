//! `plugin-extract`: per-element data extraction.
//!
//! A task block (`[[task.<element>.extract]]`, P.1 of
//! [`docs/issues/plugin-extract.md`](../../../docs/issues/plugin-extract.md))
//! names an element, an optional `type` scope, one or more destinations
//! (`output` + `format`, `export`, or both) and a field selection
//! (`recommended`, `only`, `omit`). This crate turns those blocks into a flat
//! array of objects per destination — one row per element occurrence.
//!
//! Three invariants shape the implementation:
//!
//! - **It only reads.** The event stream is not rewritten (P.2), so adding an
//!   extract block never changes a rendered byte.
//! - **The fields are frozen.** [`fields::ELEMENTS`] and [`fields::recommended`]
//!   mirror [`docs/spec/EXTRAS-INVENTORY.md`](../../../docs/spec/EXTRAS-INVENTORY.md);
//!   the extractor knows exactly what a node can carry and nothing else.
//! - **Disjointness is `pendon-types`.** A block's `type` scope is a
//!   [`pendon_types::Selector`] list resolved per element, so two blocks that name
//!   different types cannot both claim a node (P.4).

mod block;
mod fields;
mod format;
mod scan;

pub use block::{BlockSpec, ExtractBlock, ExtractError};
pub use fields::{select_fields, ELEMENTS};
pub use format::Format;
pub use scan::{ElementNode, DEPTH, INNER_TEXT};

use std::collections::BTreeMap;

use pendon_core::Event;
use pendon_types::{Group, Rule, Selector, TypeId, TypeSet};
use serde_json::{Map, Value};

/// The validated blocks of one task, grouped the way the config wrote them.
#[derive(Clone, Debug, Default)]
pub struct ExtractOptions {
    pub blocks: Vec<ExtractBlock>,
}

impl ExtractOptions {
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }
}

/// One destination path, with the rows written to it.
#[derive(Clone, Debug, PartialEq)]
pub struct SideFile {
    /// The config's path, still a template: `[slug]` is substituted per page by
    /// whoever writes the file.
    pub path: String,
    pub format: Format,
    /// The element the rows belong to (TOML's array-of-tables key).
    pub element: String,
    pub rows: Vec<Value>,
}

impl SideFile {
    pub fn render(&self) -> Result<String, String> {
        self.format.render(&self.element, &self.rows)
    }
}

/// Everything one document extracted.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ExtractOutput {
    pub files: Vec<SideFile>,
    /// `export const` entries for the solid renderer (ADR-0009).
    pub exports: Vec<(String, Value)>,
}

impl ExtractOutput {
    pub fn is_empty(&self) -> bool {
        self.files.is_empty() && self.exports.is_empty()
    }

    /// P.1.c: a later block writing the same path replaces the earlier value;
    /// the position stays where the first block put it.
    fn push_file(&mut self, file: SideFile) {
        match self
            .files
            .iter_mut()
            .find(|existing| existing.path == file.path)
        {
            Some(existing) => *existing = file,
            None => self.files.push(file),
        }
    }

    /// D-6: last registration wins entirely — value *and* position.
    fn push_export(&mut self, name: String, value: Value) {
        self.exports.retain(|(existing, _)| *existing != name);
        self.exports.push((name, value));
    }
}

/// Where a rule writes. Equality is the *destination*, not the block: two blocks
/// aiming at one path share a target, which is what makes the later one win.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Destination {
    File(String),
    /// One `export const` binding. The *name* is part of the target: two blocks
    /// exporting different variables are different destinations (D-6's "last
    /// wins" only collapses two blocks that write the same name).
    Export(String),
}

/// Extracts every block of `options` from `events`.
///
/// Elements are independent (D-2): each one gets its own [`TypeSet`], built from
/// the types that element actually occurs with, so `_` means "every type this
/// element has" and a type only headings carry can never affect a table.
pub fn run(events: &[Event], options: &ExtractOptions) -> ExtractOutput {
    let nodes = scan::scan(events);
    let mut output = ExtractOutput::default();
    if options.is_empty() {
        return output;
    }
    for element in ELEMENTS {
        let occurrences: Vec<&ElementNode> = nodes
            .iter()
            .filter(|node| node.element == element)
            .collect();
        let blocks: Vec<usize> = options
            .blocks
            .iter()
            .enumerate()
            .filter(|(_, block)| block.element == element)
            .map(|(index, _)| index)
            .collect();
        if occurrences.is_empty() || blocks.is_empty() {
            continue;
        }

        let mut types = TypeSet::new();
        for node in &occurrences {
            match node.get("type") {
                Some(name) => types.insert(name),
                None => types.insert_untyped(),
            }
        }

        // One rule per destination: an empty `type` list is `_` (P.1.b's
        // "safety net": every type of this element that no other block names),
        // resolved by `pendon-types` for the whole element, so registration
        // order cannot change what `_` means (ADR-0008 §2).
        let mut rules: Vec<Rule<usize, Destination>> = Vec::new();
        for index in &blocks {
            let block = &options.blocks[*index];
            let selectors = if block.selectors.is_empty() {
                vec![Selector::Underscore]
            } else {
                block.selectors.clone()
            };
            for (path, _) in &block.outputs {
                rules.push(Rule::new(
                    selectors.clone(),
                    Destination::File(path.clone()),
                    *index,
                ));
            }
            if let Some(name) = &block.export {
                rules.push(Rule::new(
                    selectors.clone(),
                    Destination::Export(name.clone()),
                    *index,
                ));
            }
        }
        let group = Group::new(rules);

        // A node contributes at most one row per block, however many
        // destinations that block has.
        let mut rows: BTreeMap<usize, Vec<Value>> = BTreeMap::new();
        for node in &occurrences {
            let id = match node.get("type") {
                Some(name) => TypeId::Literal(name),
                None => TypeId::Untyped,
            };
            let mut claimed: Vec<usize> = Vec::new();
            for rule in group.matching(&types, id) {
                if !claimed.contains(&rule.payload) {
                    claimed.push(rule.payload);
                }
            }
            for index in claimed {
                if let Some(row) = row_of(&options.blocks[index], node) {
                    rows.entry(index).or_default().push(row);
                }
            }
        }

        // A block that matched nothing writes nothing (T4.2): no empty file.
        for (index, element_rows) in rows {
            if element_rows.is_empty() {
                continue;
            }
            let block = &options.blocks[index];
            for (path, format) in &block.outputs {
                output.push_file(SideFile {
                    path: path.clone(),
                    format: *format,
                    element: block.element.to_string(),
                    rows: element_rows.clone(),
                });
            }
            if let Some(name) = &block.export {
                output.push_export(name.clone(), Value::Array(element_rows));
            }
        }
    }
    output
}
/// One occurrence as a JSON object of the selected fields, or `None` when the
/// selection leaves nothing to write.
fn row_of(block: &ExtractBlock, node: &ElementNode) -> Option<Value> {
    let fields = fields::select_fields(block, &node.available());
    let mut row = Map::new();
    for name in fields {
        if let Some(value) = node.value(&name) {
            row.insert(name, value);
        }
    }
    (!row.is_empty()).then_some(Value::Object(row))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pendon_core::NodeKind;

    fn attribute(name: &str, value: &str) -> Event {
        Event::Attribute {
            name: name.to_string(),
            value: value.to_string(),
        }
    }

    fn text(chunk: &str) -> Event {
        Event::Text(chunk.to_string())
    }

    /// A heading node: `__layer`, `level`, `id`, a `__number` and its text.
    fn heading(level: &str, id: &str, ty: Option<&str>) -> Vec<Event> {
        let mut events = vec![
            Event::StartNode(NodeKind::Heading),
            attribute("__layer", "heading"),
            attribute("level", level),
            attribute("id", id),
            attribute("__number", "[\"1\"]"),
        ];
        if let Some(name) = ty {
            events.push(attribute("type", name));
        }
        events.push(text("Intro"));
        events.push(Event::EndNode(NodeKind::Heading));
        events
    }

    fn block(
        element: &str,
        types: &[&str],
        output: &[&str],
        format: &[&str],
        export: Option<&str>,
    ) -> ExtractBlock {
        ExtractBlock::build(
            BlockSpec {
                element: element.to_string(),
                types: types.iter().map(|name| name.to_string()).collect(),
                output: output.iter().map(|path| path.to_string()).collect(),
                format: format.iter().map(|name| name.to_string()).collect(),
                export: export.map(str::to_string),
                ..BlockSpec::default()
            },
            0,
        )
        .expect("a valid block")
    }

    fn options(blocks: Vec<ExtractBlock>) -> ExtractOptions {
        ExtractOptions { blocks }
    }

    #[test]
    fn scan_reads_the_layer_and_hides_internal_keys() {
        let nodes = scan::scan(&heading("2", "intro", Some("HeadingA")));
        assert_eq!(nodes.len(), 1);
        let node = &nodes[0];
        assert_eq!(node.element, "heading");
        assert_eq!(node.depth, 2);
        assert_eq!(node.inner_text, "Intro");
        assert_eq!(node.get("id"), Some("intro"));
        assert_eq!(node.get("__layer"), None);
        assert_eq!(node.get("__number"), None);
        assert!(!node.available().iter().any(|name| name.starts_with("__")));
    }

    #[test]
    fn scan_flattens_nested_inline_text() {
        let events = vec![
            Event::StartNode(NodeKind::Heading),
            attribute("__layer", "heading"),
            attribute("level", "1"),
            text("A "),
            Event::StartNode(NodeKind::Strong),
            text("bold"),
            Event::EndNode(NodeKind::Strong),
            text(" heading"),
            Event::EndNode(NodeKind::Heading),
        ];
        let nodes = scan::scan(&events);
        assert_eq!(nodes[0].inner_text, "A bold heading");
    }

    #[test]
    fn scan_counts_section_depth() {
        let events = vec![
            Event::StartNode(NodeKind::Section),
            attribute("__layer", "section"),
            attribute("id", "outer"),
            Event::StartNode(NodeKind::Section),
            attribute("__layer", "section"),
            attribute("id", "inner"),
            text("body"),
            Event::EndNode(NodeKind::Section),
            Event::EndNode(NodeKind::Section),
        ];
        let nodes = scan::scan(&events);
        let depths: Vec<(&str, u8)> = nodes
            .iter()
            .map(|node| (node.get("id").unwrap_or(""), node.depth))
            .collect();
        assert_eq!(depths, [("outer", 1), ("inner", 2)]);
    }

    /// The default block (`recommended = false`, no `type`) writes every field the
    /// element carries, internal keys excluded.
    #[test]
    fn run_writes_every_carried_field_by_default() {
        let events = heading("2", "intro", Some("HeadingA"));
        let output = run(
            &events,
            &options(vec![block(
                "heading",
                &[],
                &["out/h.json"],
                &["json"],
                None,
            )]),
        );
        assert_eq!(output.files.len(), 1);
        let file = &output.files[0];
        assert_eq!(file.path, "out/h.json");
        assert_eq!(file.element, "heading");
        assert_eq!(
            file.rows[0],
            serde_json::json!({
                "level": "2",
                "id": "intro",
                "type": "HeadingA",
                "inner_text": "Intro",
                "depth": 2,
            })
        );
    }

    #[test]
    fn run_scopes_by_type() {
        let mut events = heading("2", "a", Some("HeadingA"));
        events.extend(heading("3", "b", Some("HeadingB")));
        let output = run(
            &events,
            &options(vec![block(
                "heading",
                &["HeadingA"],
                &["out/a.json"],
                &["json"],
                None,
            )]),
        );
        assert_eq!(output.files.len(), 1);
        assert_eq!(output.files[0].rows.len(), 1);
        assert_eq!(output.files[0].rows[0]["id"], "a");
    }

    /// P.1.b: a block with no `type` is `_` — a safety net for the types no
    /// other block of that element names — not `*`.
    #[test]
    fn an_untyped_block_is_the_remainder() {
        let mut events = heading("2", "a", Some("HeadingA"));
        events.extend(heading("2", "b", Some("HeadingB")));
        events.extend(heading("2", "c", Some("HeadingC")));
        events.extend(heading("2", "u", None));
        let output = run(
            &events,
            &options(vec![
                block(
                    "heading",
                    &["HeadingA", "HeadingB"],
                    &["out/ab.json"],
                    &["json"],
                    None,
                ),
                block("heading", &[], &["out/rest.json"], &["json"], Some("rest")),
            ]),
        );
        let ids = |path: &str| -> Vec<String> {
            output
                .files
                .iter()
                .find(|file| file.path == path)
                .unwrap_or_else(|| panic!("no {path} in {output:?}"))
                .rows
                .iter()
                .map(|row| row["id"].as_str().unwrap_or("").to_string())
                .collect()
        };
        assert_eq!(ids("out/ab.json"), ["a", "b"]);
        // `HeadingC` is named by no block, and the untyped heading has no `?`
        // entry either: both are the remainder.
        assert_eq!(ids("out/rest.json"), ["c", "u"]);
        assert_eq!(output.exports[0].1, serde_json::json!(output.files[1].rows));
    }

    /// `*` is the escape hatch that keeps the named types too.
    #[test]
    fn a_star_block_sees_every_type() {
        let events = heading("2", "a", Some("HeadingA"));
        let output = run(
            &events,
            &options(vec![
                block("heading", &["HeadingA"], &["out/a.json"], &["json"], None),
                block("heading", &["*"], &["out/all.json"], &["json"], None),
            ]),
        );
        assert_eq!(output.files.len(), 2);
        assert_eq!(output.files[1].rows.len(), 1);
    }

    #[test]
    fn two_blocks_on_one_path_last_wins() {
        let events = heading("2", "intro", None);
        let mut first = block("heading", &[], &["out/x.json"], &["json"], None);
        first.only = vec!["id".to_string()];
        let mut second = block("heading", &[], &["out/x.json"], &["json"], None);
        second.only = vec!["level".to_string()];
        let output = run(&events, &options(vec![first, second]));
        assert_eq!(output.files.len(), 1);
        assert_eq!(output.files[0].rows[0], serde_json::json!({"level": "2"}));
    }

    #[test]
    fn two_blocks_on_two_paths_both_run() {
        let events = heading("2", "intro", None);
        let output = run(
            &events,
            &options(vec![
                block("heading", &[], &["out/a.json"], &["json"], None),
                block("heading", &[], &["out/b.json"], &["json"], None),
            ]),
        );
        assert_eq!(output.files.len(), 2);
    }

    #[test]
    fn export_registration_is_last_wins_and_never_a_file() {
        let events = heading("2", "intro", None);
        let output = run(
            &events,
            &options(vec![block("heading", &[], &[], &[], Some("headings"))]),
        );
        assert!(output.files.is_empty());
        assert_eq!(output.exports.len(), 1);
        assert_eq!(output.exports[0].0, "headings");
        assert!(output.exports[0].1.is_array());
    }

    /// Two blocks under *different* export names are two bindings, not one: the
    /// target of a rule is the binding, never "any export" (only a repeated name
    /// collapses).
    #[test]
    fn two_export_names_both_register() {
        let events = heading("2", "intro", None);
        let output = run(
            &events,
            &options(vec![
                block("heading", &[], &[], &[], Some("headingsA")),
                block("heading", &[], &[], &[], Some("headingsB")),
            ]),
        );
        let names: Vec<&str> = output
            .exports
            .iter()
            .map(|(name, _)| name.as_str())
            .collect();
        assert_eq!(names, ["headingsA", "headingsB"]);
    }

    /// …and writing one name twice keeps the later value (P.1.c).
    #[test]
    fn a_repeated_export_name_keeps_the_later_value() {
        let events = heading("2", "intro", None);
        let mut first = block("heading", &[], &[], &[], Some("headings"));
        first.only = vec!["id".to_string()];
        let mut second = block("heading", &[], &[], &[], Some("headings"));
        second.only = vec!["level".to_string()];
        let output = run(&events, &options(vec![first, second]));
        assert_eq!(output.exports.len(), 1);
        assert_eq!(
            output.exports[0].1,
            serde_json::json!([{ "level": "2" }]),
            "the later block's selection wins"
        );
    }

    #[test]
    fn a_block_that_matches_nothing_writes_no_file() {
        let events = heading("2", "intro", Some("HeadingA"));
        let output = run(
            &events,
            &options(vec![block("cite", &[], &["out/c.json"], &["json"], None)]),
        );
        assert!(output.is_empty(), "{output:?}");
    }

    #[test]
    fn a_row_is_rendered_by_its_format() {
        let events = heading("2", "intro", None);
        let output = run(
            &events,
            &options(vec![block(
                "heading",
                &[],
                &["out/h.toml"],
                &["toml"],
                None,
            )]),
        );
        let rendered = output.files[0].render().expect("toml renders");
        assert!(rendered.starts_with("[[heading]]\n"), "{rendered}");
    }
}
