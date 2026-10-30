//! `docs/spec/FIELDS-INVENTORY.md` against the code it describes
//! (S2-7.3, S2-7.4).
//!
//! The inventory claims to be "the contract `plugin-extract` implements", so it is
//! checked the way the goldens check the pipeline: the document is parsed **here**
//! (`include_str!`, so editing it is a rebuild) and every table it prints is
//! compared with the data the crates declare. Nothing is copied into this file —
//! a row, a mark, an order and a slot position all come from the code.
//!
//! What is compared, and with what:
//!
//! * §12's preset table — [`pendon_preset`], which reads `PRESETS` (the table is the
//!   one place the preset is spelled);
//! * each element's `field | … | preset | …` table — the same preset, in both
//!   directions: a row marked `yes` the code does not list fails, and a preset
//!   name that is neither a row nor `*` in §12 fails. §0's rule is only "a row is
//!   a built-in field", so a preset may recommend a name no element produces yet;
//! * the four `**Slot contract**` tables — each plugin's `HEAD`: the tag from the
//!   sentence above the table, the positions from `SlotContract::accepted`, and
//!   the landing field from `ElementHead::preset_for` (else the slot-qualified
//!   fallback, §0);
//! * §0's `specs.*` table — `SPECS_KEYS` and `universal_field`;
//! * §0's `number` table — the three public `number.*` fields the numbering pass
//!   publishes.
//!
//! A rename is therefore a two-file change or a failing test, which is the point
//! (D-16: the document is not a second source of truth, it is the same one in
//! prose). The bytes of an actual extraction stay the goldens' job.

use std::collections::BTreeMap;

use pendon_extra::{
    fallback_path, universal_field, ElementHead, Slot, SlotKind, SPECS_KEYS, SPECS_NAMESPACE,
};
use pendon_extract::{pendon_preset, ELEMENTS};
use pendon_plugin_anchor::HEAD as ANCHOR;
use pendon_plugin_cite::HEAD as CITE;
use pendon_plugin_heading::HEAD as HEADING;
use pendon_plugin_image::HEAD as IMAGE;

/// The inventory, embedded at compile time.
const DOC: &str = include_str!("../../../docs/spec/FIELDS-INVENTORY.md");

/// The four elements that claim a slot head, with the contract each declares
/// (§0's "The slot head", S2-7.3).
const HEADS: [(&str, ElementHead); 4] = [
    ("anchor", ANCHOR),
    ("cite", CITE),
    ("heading", HEADING),
    ("image", IMAGE),
];

/// Names every element can carry (§0), so a preset may list one without a row.
const UNIVERSAL: [&str; 2] = ["inner_text", "depth"];

/// One markdown table of the document.
struct Table {
    /// The heading it sits under, `#` markers stripped (`2. `anchor` — …`).
    section: String,
    /// The last non-blank line before it — for a contract table, the sentence
    /// that names the tag.
    lead: String,
    header: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl Table {
    /// The column whose header is `name`.
    fn column(&self, name: &str) -> Option<usize> {
        self.header.iter().position(|cell| cell == name)
    }

    /// The element a per-element section is about (§0–§1 and §12 have none).
    fn element(&self) -> Option<&str> {
        element_of_section(&self.section)
    }
}

/// The element a section heading is about: the first backticked token of a
/// `2. `anchor` — …` heading, so `None` for §0, §1 and §12.
fn element_of_section(section: &str) -> Option<&str> {
    let rest = section.split_once(". ")?.1;
    let rest = rest.strip_prefix('`')?;
    rest.split_once('`').map(|(name, _)| name)
}

/// One element's section, `##` heading excluded — its prose, tables and all.
fn section_body(doc: &str, element: &str) -> String {
    let mut body = String::new();
    let mut inside = false;
    for line in doc.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            inside = element_of_section(heading) == Some(element);
            continue;
        }
        if inside {
            body.push_str(line);
            body.push('\n');
        }
    }
    body
}

/// Every table of the document, in order.
fn tables(doc: &str) -> Vec<Table> {
    let lines: Vec<&str> = doc.lines().collect();
    let mut tables = Vec::new();
    let mut section = String::new();
    let mut lead = String::new();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index].trim();
        if let Some(heading) = line.strip_prefix('#') {
            section = heading.trim_start_matches('#').trim().to_string();
            lead.clear();
            index += 1;
            continue;
        }
        if !line.starts_with('|') {
            if !line.is_empty() {
                lead = line.to_string();
            }
            index += 1;
            continue;
        }
        let mut block = Vec::new();
        while index < lines.len() && lines[index].trim().starts_with('|') {
            block.push(cells(lines[index].trim()));
            index += 1;
        }
        let mut block = block.into_iter();
        let Some(header) = block.next() else {
            continue;
        };
        tables.push(Table {
            section: section.clone(),
            lead: lead.clone(),
            header,
            rows: block
                .filter(|row| !row.iter().all(|cell| is_rule(cell)))
                .collect(),
        });
    }
    tables
}

/// A table line split into trimmed cells, outer pipes removed.
fn cells(line: &str) -> Vec<String> {
    line.trim_matches('|')
        .split('|')
        .map(|cell| cell.trim().to_string())
        .collect()
}

/// The `| --- |` separator row.
fn is_rule(cell: &str) -> bool {
    !cell.is_empty() && cell.chars().all(|c| c == '-' || c == ':')
}

/// A cell's text with its inline-code ticks removed; prose (`—`) stays as written.
fn plain(cell: &str) -> String {
    let cell = cell.trim();
    if cell.len() >= 2 && cell.starts_with('`') && cell.ends_with('`') {
        cell[1..cell.len() - 1].to_string()
    } else {
        cell.to_string()
    }
}

/// The names of a §12 preset cell, each with its `*` flag: `*` marks a name that
/// is not a built-in field of that element (§12).
fn preset_names(cell: &str) -> Vec<(String, bool)> {
    cell.split(',')
        .map(|name| {
            let name = name.trim();
            match name.strip_suffix("\\*").or_else(|| name.strip_suffix('*')) {
                Some(name) => (plain(name), true),
                None => (plain(name), false),
            }
        })
        .filter(|(name, _)| !name.is_empty())
        .collect()
}

/// Each element's `preset = yes` rows, keyed by element, in table order.
fn documented_rows(doc: &str) -> BTreeMap<String, Vec<String>> {
    let mut map: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for table in tables(doc) {
        let (Some(element), Some(preset)) = (table.element(), table.column("preset")) else {
            continue;
        };
        let field = table
            .column("field")
            .expect("a table with a `preset` column has a `field` column");
        let rows = map.entry(element.to_string()).or_default();
        rows.extend(
            table
                .rows
                .iter()
                .filter(|row| row[preset] == "yes")
                .map(|row| plain(&row[field])),
        );
    }
    map
}

/// §12 is the code's preset table: the same elements, the same order, the same
/// names.
#[test]
fn the_preset_table_is_the_code_preset() {
    let table = tables(DOC)
        .into_iter()
        .find(|table| table.section.starts_with("12.") && table.column("preset").is_some())
        .expect("§12 prints the preset table");
    let documented: Vec<(String, Vec<String>)> = table
        .rows
        .iter()
        .map(|row| {
            (
                plain(&row[0]),
                preset_names(&row[1])
                    .into_iter()
                    .map(|(name, _)| name)
                    .collect(),
            )
        })
        .collect();
    let code: Vec<(String, Vec<String>)> = ELEMENTS
        .iter()
        .map(|element| {
            (
                element.to_string(),
                pendon_preset(element)
                    .iter()
                    .map(|name| name.to_string())
                    .collect(),
            )
        })
        .collect();
    assert_eq!(
        documented, code,
        "§12 and `PRESETS` name the same fields, in the same order"
    );
}

/// Every `preset` mark of a per-element table agrees with the code — in both
/// directions, so neither a row nor a preset entry can be edited alone.
#[test]
fn every_row_is_a_preset_member_exactly_when_it_says_so() {
    let mut audited: Vec<String> = Vec::new();
    for table in tables(DOC) {
        let (Some(element), Some(preset_column)) = (table.element(), table.column("preset")) else {
            continue;
        };
        let field_column = table
            .column("field")
            .expect("a table with a `preset` column has a `field` column");
        assert!(
            ELEMENTS.contains(&element),
            "§{} documents `{element}`, which is not an extractable element",
            table.section
        );
        let preset = pendon_preset(element);
        for row in &table.rows {
            let field = plain(&row[field_column]);
            let listed = preset.contains(&field.as_str());
            match row[preset_column].as_str() {
                "yes" => assert!(
                    listed,
                    "{element}: `{field}` is marked `preset = yes`, but the code preset does not list it"
                ),
                "no" => assert!(
                    !listed,
                    "{element}: `{field}` is marked `preset = no`, but the code preset lists it"
                ),
                other => panic!(
                    "{element}: `{field}` has the preset mark `{other}`; §0 allows only `yes` and `no`"
                ),
            }
        }
        audited.push(element.to_string());
    }
    // §3 (`blockquote`) and §11 (`table`) document **no** built-in field: the one
    // element with nothing it writes itself prints `**None.**` and no table. Every
    // other element must have one, and the order must still be §12's.
    let without: Vec<&str> = ELEMENTS
        .iter()
        .copied()
        .filter(|element| !audited.iter().any(|seen| seen == element))
        .collect();
    assert_eq!(
        without,
        ["blockquote", "table"],
        "an element with no built-in field is the exception, not a silent omission"
    );
    let expected: Vec<&str> = ELEMENTS
        .iter()
        .copied()
        .filter(|element| !without.contains(element))
        .collect();
    assert_eq!(
        audited, expected,
        "the per-element tables follow §12's element order"
    );
    for element in &without {
        assert!(
            section_body(DOC, element).contains("**None.**"),
            "§{element} has no field table, so its prose must say **None.**"
        );
    }
}

/// Every name the code preset recommends is accounted for: a `yes` row of the
/// element's table, a `*` in §12, or one of §0's universal names.
#[test]
fn every_preset_name_is_a_row_or_starred_in_section_twelve() {
    let rows = documented_rows(DOC);
    let mut starred: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for table in tables(DOC) {
        if !table.section.starts_with("12.") || table.column("preset").is_none() {
            continue;
        }
        for row in &table.rows {
            let names: Vec<String> = preset_names(&row[1])
                .into_iter()
                .filter(|(_, starred)| *starred)
                .map(|(name, _)| name)
                .collect();
            starred.insert(plain(&row[0]), names);
        }
    }

    for element in ELEMENTS {
        let documented = rows.get(element).cloned().unwrap_or_default();
        let marked = starred.get(element).cloned().unwrap_or_default();
        for name in pendon_preset(element) {
            let row = documented.iter().any(|field| field == name);
            let star = marked.iter().any(|field| field == name);
            if UNIVERSAL.contains(name) && !row {
                continue;
            }
            assert!(
                row || star,
                "{element}: the preset recommends `{name}`, but no row of its table produces it and §12 does not star it"
            );
            assert!(
                !(row && star),
                "{element}: `{name}` is a `yes` row of its table and starred in §12 — it cannot be both"
            );
        }
    }
}

/// Every `SlotContract` in the code has a table, and every table is a contract:
/// the tag, the accepted positions (in head order) and each landing field.
#[test]
fn every_slot_contract_in_code_has_a_table_and_none_is_orphaned() {
    let mut documented: Vec<String> = Vec::new();
    for table in tables(DOC) {
        if table.header != ["position", "lands in"] {
            continue;
        }
        let element = table.element().unwrap_or_else(|| {
            panic!(
                "§{} prints a slot contract outside an element section",
                table.section
            )
        });
        let (_, head) = HEADS
            .iter()
            .find(|(name, _)| *name == element)
            .unwrap_or_else(|| {
                panic!("§{element} prints a slot contract, but `{element}` declares no `HEAD`")
            });
        documented.push(element.to_string());

        let tag = table
            .lead
            .strip_prefix("**Slot contract** — tag `")
            .and_then(|rest| rest.chars().next())
            .unwrap_or_else(|| {
                panic!(
                    "§{element}: the line above the contract table must read `**Slot contract** — tag `<sigil>`, …`"
                )
            });
        assert_eq!(
            Some(tag),
            head.contract.tag.sigil(),
            "§{element}: the tag the contract claims an untagged head with"
        );

        let expected: Vec<(String, String)> = head
            .contract
            .accepted
            .iter()
            .map(|(slot, kind)| {
                let fallback = fallback_path(*slot, *kind).dotted();
                let landing = head
                    .preset_for(*slot, *kind)
                    .map(|name| name.as_str().to_string())
                    .unwrap_or_else(|| fallback.clone());
                (fallback, landing)
            })
            .collect();
        let got: Vec<(String, String)> = table
            .rows
            .iter()
            .map(|row| (plain(&row[0]), plain(&row[1])))
            .collect();
        assert_eq!(
            got, expected,
            "§{element}: `SlotContract::accepted` in head order, each position landing in its preset name or its bare fallback"
        );
    }

    for (element, _) in HEADS {
        assert!(
            documented.iter().any(|name| name == element),
            "`{element}` declares a `SlotContract`, so the inventory must print it"
        );
    }
}

/// §0's fallback table is the grammar: every position, named as itself (D-7).
#[test]
fn the_fallback_table_is_every_position() {
    let table = tables(DOC)
        .into_iter()
        .find(|table| table.column("fallback name").is_some())
        .expect("§0 prints the slot-qualified fallbacks");
    let position = table.column("position").expect("a `position` column");
    let name = table
        .column("fallback name")
        .expect("a `fallback name` column");
    let got: Vec<(String, String)> = table
        .rows
        .iter()
        .map(|row| (plain(&row[position]), plain(&row[name])))
        .collect();
    let expected: Vec<(String, String)> = Slot::ALL
        .iter()
        .flat_map(|slot| SlotKind::ALL.iter().map(move |kind| (*slot, *kind)))
        .map(|(slot, kind)| {
            let path = fallback_path(slot, kind).dotted();
            (path.clone(), path)
        })
        .collect();
    assert_eq!(
        got, expected,
        "§0 lists every `slot.kind` position, and each falls back to its own spelling"
    );
}

/// §0's `specs.*` table is the code's vocabulary: the five keys, their default
/// paths, and which of them are universal DOM fields.
#[test]
fn the_specs_table_is_the_code_vocabulary() {
    let table = tables(DOC)
        .into_iter()
        .find(|table| table.column("default path").is_some())
        .expect("§0 prints the five `specs.*` positions");
    let path_column = table
        .column("default path")
        .expect("a `default path` column");
    let dom_column = table.column("DOM field").expect("a `DOM field` column");

    let mut keys: Vec<String> = Vec::new();
    for row in &table.rows {
        let path = plain(&row[path_column]);
        let key = path
            .strip_prefix(SPECS_NAMESPACE)
            .and_then(|rest| rest.strip_prefix('.'))
            .unwrap_or_else(|| panic!("`{path}` is not a `{SPECS_NAMESPACE}.*` path"));
        match universal_field(&path) {
            Some(field) => assert_eq!(
                row[dom_column],
                format!("`{field}`"),
                "`{path}` is projected onto `{field}` (§6.3)"
            ),
            None => assert_eq!(
                row[dom_column], "—",
                "`{path}` is not a DOM field, so its cell is `—`"
            ),
        }
        keys.push(key.to_string());
    }

    let expected: Vec<String> = SPECS_KEYS.iter().map(|key| key.to_string()).collect();
    assert_eq!(keys, expected, "§0 lists exactly the five positions");
}

/// §0's `number` table is the three public fields the numbering pass publishes
/// (ADR-0016, S2-9.2) — one computation, so the table and the crate cannot drift.
#[test]
fn the_number_table_is_the_numbering_passes_public_fields() {
    let table = tables(DOC)
        .into_iter()
        .find(|table| table.header == ["name", "type", "meaning"])
        .expect("§0 prints the `number` namespace");
    let rows: Vec<String> = table.rows.iter().map(|row| plain(&row[0])).collect();
    assert_eq!(
        rows,
        vec![
            pendon_number::NUMBER_TYPE_ATTR.to_string(),
            pendon_number::NUMBER_RAW_ATTR.to_string(),
            pendon_number::NUMBER_DEPTH_ATTR.to_string(),
        ],
        "the namespace is `number.type` / `number.raw` / `number.depth`, in that order"
    );
}
