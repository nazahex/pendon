//! Public field names and the field map — the vocabulary the projection emits
//! (`docs/decisions/0014-field-model.md`, proposal §7).
//!
//! Two things live here, and neither renames an IR attribute:
//!
//! * the **name rules** — [`validate_field_name`] and [`Path`], a validated
//!   dotted name such as `price.details.amount`;
//! * [`FieldMap`] — the ordered `path → AttrValue` map the projection fills and
//!   that renderers and `extract` read instead of recomputing a name
//!   (ADR-0014 rules 1–2), plus [`project`], the identity pass that fills it.
//!
//! [`project`] is the only function that reads the AST, and it only *reads*: the
//! node keeps its own attributes, so a plugin's internals stay untouched.
//!
//! Names are configuration, so every rejection here is `Severity::Error` at
//! config load — never a warning and never a silent drop (D-16). The rules are
//! the proposal's, verbatim: `A-Z a-z 0-9 _ -` only, a segment starts with a
//! letter or `_`, names are case-sensitive, and `.` nests.

use std::collections::BTreeMap;
use std::fmt;

use crate::slot::{Slot, SlotKind};
use crate::value::AttrValue;

/// Why a public field name was rejected.
///
/// `segment` is a **1-based** index into the `.`-separated name, so a message
/// can point at the part the user mistyped (`price.1x` → segment 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldNameError {
    /// The name is empty.
    Empty,
    /// A segment is empty: `.a`, `a..b`, `a.`.
    EmptySegment {
        /// 1-based index of the empty segment.
        segment: usize,
    },
    /// A character outside `A-Z a-z 0-9 _ -`.
    InvalidChar {
        /// 1-based index of the offending segment.
        segment: usize,
        /// The character that is not allowed.
        ch: char,
    },
    /// A segment starts with a digit or `-` (`1x`, `-x`, `a.1b`).
    InvalidStart {
        /// 1-based index of the offending segment.
        segment: usize,
        /// The first character of that segment.
        ch: char,
    },
}

impl fmt::Display for FieldNameError {
    /// The clause **after** the offending name: a config error reads
    /// `field.bracket.bare = "1x" is not a valid field name: <this>`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "a field name cannot be empty"),
            Self::EmptySegment { segment } => write!(
                f,
                "segment {segment} is empty (a `.` needs a name on both sides)"
            ),
            Self::InvalidChar { segment, ch } => write!(
                f,
                "segment {segment} contains `{ch}`; only `A-Z`, `a-z`, `0-9`, `_` and `-` are allowed"
            ),
            Self::InvalidStart { segment, ch } => write!(
                f,
                "segment {segment} starts with `{ch}`; the first character must be a letter or `_`"
            ),
        }
    }
}

/// One segment of a name: a letter or `_`, then `A-Z a-z 0-9 _ -`.
///
/// `index` is 1-based and only used for the error.
fn check_segment(segment: &str, index: usize) -> Result<(), FieldNameError> {
    let mut chars = segment.chars();
    match chars.next() {
        None => return Err(FieldNameError::EmptySegment { segment: index }),
        Some(first) if !(first.is_ascii_alphabetic() || first == '_') => {
            return Err(FieldNameError::InvalidStart {
                segment: index,
                ch: first,
            });
        }
        Some(_) => {}
    }
    match chars.find(|ch| !(ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'))) {
        Some(ch) => Err(FieldNameError::InvalidChar { segment: index, ch }),
        None => Ok(()),
    }
}

/// Validates one public field name — one or more `.`-separated segments.
///
/// Used by every place a user writes a name: `field.<slot>.<k>`,
/// `field.specs.<k>` and `field.override.<path>`. An empty string never reaches
/// here: `""` means *unassign* and is recognised by the config layer (OBJ-10),
/// while an empty **name** is a plain error.
pub fn validate_field_name(name: &str) -> Result<(), FieldNameError> {
    if name.is_empty() {
        return Err(FieldNameError::Empty);
    }
    for (index, segment) in name.split('.').enumerate() {
        check_segment(segment, index + 1)?;
    }
    Ok(())
}

/// A **built-in** field name, validated at compile time: `href`, `inner_text`.
///
/// This is the name half of a [`Preset`](crate::Preset) assignment, so it is
/// spelled in a plugin's own source rather than in a config file. That is why it
/// wraps a `&'static str`: [`FieldName::new`] is a `const fn` that asserts the
/// name rules, so a preset table with a typo (`inner-text`, `1x`, `price.amount`)
/// **fails to compile** instead of failing at load. Limits:
///
/// * one segment only — a preset lands in a top-level field; dotted names are
///   config business ([`Path`]) and nesting is the projection's (ADR-0014);
/// * the rest is exactly [`validate_field_name`]'s rule, so the two cannot drift.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FieldName(&'static str);

impl FieldName {
    /// Declares a built-in field name, panicking at compile time when invalid.
    ///
    /// Usable in a `const`/`static` initializer (every preset table is one), which
    /// is the whole point: the check runs while compiling, not while loading.
    pub const fn new(name: &'static str) -> Self {
        let bytes = name.as_bytes();
        assert!(!bytes.is_empty(), "a field name cannot be empty");
        let mut index = 0;
        while index < bytes.len() {
            let byte = bytes[index];
            assert!(
                byte != b'.',
                "a built-in field name is one segment; dotted names belong in `field.*`"
            );
            assert!(
                byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-',
                "a field name allows only `A-Z`, `a-z`, `0-9`, `_` and `-`, ASCII only"
            );
            if index == 0 {
                assert!(
                    byte.is_ascii_alphabetic() || byte == b'_',
                    "a field name must start with a letter or `_`"
                );
            }
            index += 1;
        }
        Self(name)
    }

    /// The name as written.
    pub const fn as_str(self) -> &'static str {
        self.0
    }

    /// The name as a one-segment [`Path`], which is what the projection emits.
    pub fn path(self) -> Path {
        // `new` already proved the segment obeys the rules, so this cannot fail.
        Path::from_segments(&[self.0]).expect("a FieldName is a valid path segment")
    }
}

impl fmt::Display for FieldName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

/// A validated dotted public field path, e.g. `price.details.amount`.
///
/// The segments are the truth and the dotted spelling is only a rendering, so
/// prefix tests stay segment-wise (`priced` is **not** under `price`). A `Path`
/// can only be built by [`Path::from_dotted`] or [`Path::from_segments`], so
/// every instance obeys the name rules.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Path(Vec<String>);

impl Path {
    /// Parses a dotted name, rejecting it when it breaks the name rules.
    pub fn from_dotted(name: &str) -> Result<Self, FieldNameError> {
        validate_field_name(name)?;
        Ok(Self(name.split('.').map(str::to_string).collect()))
    }

    /// Builds a path from already-split segments — a slot position's
    /// `bracket.bare`, an override's target. Every segment is validated exactly
    /// like a dotted name's, so the two constructors cannot drift.
    pub fn from_segments(segments: &[&str]) -> Result<Self, FieldNameError> {
        if segments.is_empty() {
            return Err(FieldNameError::Empty);
        }
        for (index, segment) in segments.iter().enumerate() {
            check_segment(segment, index + 1)?;
        }
        Ok(Self(
            segments
                .iter()
                .map(|segment| (*segment).to_string())
                .collect(),
        ))
    }

    /// The segments, outermost first.
    pub fn segments(&self) -> &[String] {
        &self.0
    }

    /// How many segments the path has; always at least one.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// False by construction — a `Path` always has a segment. Kept so the
    /// `len`/`is_empty` pair reads naturally.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The dotted spelling (`bracket.bare`).
    pub fn dotted(&self) -> String {
        self.0.join(".")
    }

    /// True when `prefix` is this path's first segments.
    ///
    /// Inclusive: a path is a prefix of itself, which is what `only`/`omit`
    /// selection needs (a rule for `specs` also reaches `specs.class`).
    pub fn starts_with(&self, prefix: &Self) -> bool {
        self.0.len() >= prefix.0.len() && self.0[..prefix.0.len()] == prefix.0[..]
    }

    /// True when this path is `path`'s first segments — the mirror of
    /// [`Path::starts_with`], and inclusive in the same way.
    pub fn is_prefix_of(&self, path: &Self) -> bool {
        path.starts_with(self)
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.dotted())
    }
}

/// Why a value could not be stored in a [`FieldMap`].
///
/// Both cases are the same rule seen from two sides: one path cannot be a
/// scalar **and** have children. `parent` is always the shorter path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldMapError {
    /// `parent` holds a scalar, so `child` cannot be nested under it.
    ParentIsNotObject {
        /// The path that already holds a value.
        parent: Path,
        /// The path that needed `parent` to be an object.
        child: Path,
    },
    /// `parent` already has children, so it cannot hold a scalar itself.
    ParentHasChildren {
        /// The path that was being assigned a value.
        parent: Path,
        /// One of its existing children, named to make the clash obvious.
        child: Path,
    },
}

impl fmt::Display for FieldMapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParentIsNotObject { parent, child } => write!(
                f,
                "`{parent}` already holds a value, so `{child}` cannot be nested under it"
            ),
            Self::ParentHasChildren { parent, child } => write!(
                f,
                "`{parent}` already has children (`{child}`), so it cannot hold a value itself"
            ),
        }
    }
}

/// The public field map: ordered `path → value`, built once per element by the
/// projection (ADR-0014).
///
/// Insertion order **is** output order — the projection's order is the only one
/// a user can predict, and a nested [`AttrValue::Object`] keeps it. Storing
/// dotted paths instead of a tree is what makes a prefix test
/// ([`Path::starts_with`]) and an override lookup plain comparisons.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FieldMap {
    entries: Vec<(Path, AttrValue)>,
}

impl FieldMap {
    /// An empty map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Stores `value` at `path`, nesting as needed (`price.details.amount`
    /// becomes `price` → `details` → `amount` in [`FieldMap::to_value`]).
    ///
    /// Re-inserting an existing path replaces the value **in place**, so the
    /// order survives; which duplicate wins is the caller's rule (D-12 for
    /// `specs.*`), never the map's. A structural clash is an error and leaves
    /// the map untouched.
    pub fn insert(&mut self, path: Path, value: AttrValue) -> Result<(), FieldMapError> {
        if let Some(index) = self
            .entries
            .iter()
            .position(|(existing, _)| existing == &path)
        {
            self.entries[index].1 = value;
            return Ok(());
        }
        for (existing, _) in &self.entries {
            if existing.is_prefix_of(&path) {
                return Err(FieldMapError::ParentIsNotObject {
                    parent: existing.clone(),
                    child: path,
                });
            }
            if path.is_prefix_of(existing) {
                return Err(FieldMapError::ParentHasChildren {
                    parent: path,
                    child: existing.clone(),
                });
            }
        }
        self.entries.push((path, value));
        Ok(())
    }

    /// The value at exactly `path`, if any.
    pub fn get(&self, path: &Path) -> Option<&AttrValue> {
        self.entries
            .iter()
            .find(|(existing, _)| existing == path)
            .map(|(_, value)| value)
    }

    /// True when `path` holds a value.
    pub fn contains(&self, path: &Path) -> bool {
        self.get(path).is_some()
    }

    /// Every entry, in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = (&Path, &AttrValue)> {
        self.entries.iter().map(|(path, value)| (path, value))
    }

    /// How many paths are stored (leaves only — a parent is implicit).
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True when nothing is stored.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The whole map as one nested object — the shape the JSON and AST
    /// renderers emit, and what a user reading the proposal's examples sees.
    pub fn to_value(&self) -> AttrValue {
        let flattened: Vec<(&Path, &AttrValue)> = self.iter().collect();
        AttrValue::Object(nested(&flattened, 0))
    }

    /// `(name, value)` pairs in insertion order, values in AST (JSON) form —
    /// what a renderer emits, and the inverse of [`project`].
    ///
    /// Flat: a multi-segment path keeps its dotted spelling ([`Path::dotted`]),
    /// because that is the key a renderer writes; [`FieldMap::to_value`] nests
    /// the same entries instead.
    pub fn attrs(&self) -> Vec<(String, serde_json::Value)> {
        self.entries
            .iter()
            .map(|(path, value)| (path.dotted(), value.to_json()))
            .collect()
    }
}

/// The public field map of one AST node — the projection, and the **only** place
/// overrides are read (ADR-0014 rules 1–4).
///
/// Phase 2 is the identity pass: a node's attributes *are* its public fields, so
/// the map mirrors them key for key, in node order, with the same values — which
/// is why this step changes no golden byte. Phases 3–4 stop copying the node and
/// build the map from `specs.*` and each element's preset assignment instead;
/// the renderers already read whatever the map holds.
///
/// What is public:
///
/// 1. a `__`-prefixed key is an **internal** marker (`__layer`,
///    `__plugin_kind`, §11 rule 3) and never reaches the map — which is why the
///    renderers no longer test the prefix themselves;
/// 2. every remaining key is a [`Path`]. A key that breaks the name rules cannot
///    be a public field and is dropped, with a `debug_assert!` so a plugin
///    emitting one fails the suite instead of leaking it (every key the pipeline
///    produces today — authored `k: v`, `type`, `name`, `level`, `src`, … —
///    obeys the rules);
/// 3. `config`'s `field.override.<path>` renames an entry **as it is written**
///    (D-4). No override exists until Phase 4, so `None` is what every caller
///    passes today;
/// 4. the entries are ordered by their **public** path, so the §6.3 rename
///    `specs.class` → `class` writes the same attribute in the same place.
pub fn project(node: &serde_json::Value, config: Option<&FieldConfig>) -> FieldMap {
    let mut map = FieldMap::new();
    let Some(attrs) = node.get("attrs").and_then(serde_json::Value::as_object) else {
        return map;
    };
    let overrides = config.map(FieldConfig::overrides);
    let mut entries: Vec<(Path, &serde_json::Value)> = Vec::new();
    for (key, value) in attrs {
        if key.starts_with("__") {
            continue;
        }
        let Ok(path) = Path::from_dotted(key) else {
            debug_assert!(false, "`{key}` is not a public field name");
            continue;
        };
        let target = overrides
            .and_then(|table| table.get(&path))
            .cloned()
            .unwrap_or(path);
        // §6.3/S2-3.4: a **universal** specs path is written as the DOM field it
        // names (`specs.class` → `class`). The element-specific positions
        // (`specs.tick`, `specs.quote`, the slot groups) keep their dotted path:
        // what a slug or a title means is the element's business (§7), and each
        // plugin projects those before the node is built (`Specs::dom_fields`).
        let target = match crate::typed::universal_field(&target.dotted()) {
            Some(field) => {
                Path::from_dotted(field).expect("a universal specs field name is a valid path")
            }
            None => target,
        };
        entries.push((target, value));
    }
    // The map is ordered by the **public** path, not by the node's spelling: a
    // node arrives name-ordered (`attrs` is a sorted map), so renaming
    // `specs.class` → `class` must not move the attribute it writes.
    entries.sort_by_cached_key(|(path, _)| path.dotted());
    for (path, value) in entries {
        // One attribute is one entry, so the only structural clash left is an
        // override renaming onto a path the node already carries. The load-time
        // checks (`FieldConfig::add_override`) own the user-facing error; a clash
        // leaves the map untouched instead of panicking a renderer.
        if let Err(error) = map.insert(path, AttrValue::from_json(value)) {
            debug_assert!(false, "projecting an attribute failed: {error}");
        }
    }
    map
}

/// Groups `entries` (whose paths share their first `depth` segments) one level
/// deeper. A group's position is its **first** appearance, so the output order
/// is the insertion order of the map.
fn nested(entries: &[(&Path, &AttrValue)], depth: usize) -> Vec<(String, AttrValue)> {
    let mut object: Vec<(String, AttrValue)> = Vec::new();
    for &(path, value) in entries {
        let segments = path.segments();
        let name = segments[depth].clone();
        if segments.len() == depth + 1 {
            object.push((name, value.clone()));
            continue;
        }
        if object.iter().any(|(existing, _)| *existing == name) {
            continue;
        }
        let group: Vec<(&Path, &AttrValue)> = entries
            .iter()
            .copied()
            .filter(|(other, _)| other.segments()[depth] == name)
            .collect();
        object.push((name, AttrValue::Object(nested(&group, depth + 1))));
    }
    object
}

/// The dotted namespace of the five specs positions (§6.3, D-5).
///
/// Written once because two things build a `specs.*` path: the config parser
/// (`field.specs.tick`) and the naming fallback an unassigned specs position
/// keeps ([`crate::Specs`]).
pub const SPECS_NAMESPACE: &str = "specs";

/// The five specs positions a field can be assigned to (§6.3, D-5).
///
/// `specs` is a **field namespace**, not a slot: a modifier never lives inside
/// `field.*`, and `specs.tick` is a different path from `bracket.tick`.
pub const SPECS_KEYS: [&str; 5] = ["tick", "quote", "class", "id", "style"];

/// Why a `field.*` key was rejected at load (`Severity::Error`, D-16).
///
/// The caller prefixes the offending key, e.g.
/// `task.anchor.custom[1]: field.specs.tikc: unknown specs position …`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldConfigError {
    /// `field.specs.<k>` with a `k` that is not a specs position.
    UnknownSpecsKey {
        /// The offending sub-key.
        key: String,
    },
    /// Two spellings name the same override source (`a.b` and `"a.b"`).
    DuplicateOverride {
        /// The source path that was named twice.
        source: Path,
    },
    /// Two overrides rename to one name (ADR-0014 rule 3).
    DuplicateOverrideTarget {
        /// The target that was produced twice.
        target: Path,
    },
}

impl fmt::Display for FieldConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownSpecsKey { key } => write!(
                f,
                "unknown specs position `{key}`; expected one of {}",
                SPECS_KEYS.join(", ")
            ),
            Self::DuplicateOverride { source } => {
                write!(f, "`{source}` is overridden twice")
            }
            Self::DuplicateOverrideTarget { target } => write!(
                f,
                "two overrides rename to `{target}`; a name can have only one source"
            ),
        }
    }
}

/// The `field.*` keys of one `custom` entry (D-5, ADR-0014).
///
/// Exactly three of the proposal's four prefix families — the fourth,
/// `<slot>.modifier."<c>"`, configures a modifier and is not a field:
///
/// | key                     | stores                                              |
/// | ----------------------- | --------------------------------------------------- |
/// | `field.<slot>.<kind>`   | [`FieldConfig::slots`] — `bracket.bare` → a name     |
/// | `field.specs.<k>`       | [`FieldConfig::specs`] — `tick` → a name            |
/// | `field.override.<path>` | [`FieldConfig::overrides`] — a path → a new name    |
///
/// A value of `""` means **unassign** (OBJ-10), stored as `None`: the position
/// falls back to its bare name. Names are [`Path`]s, so an invalid one never
/// reaches this type. Nothing here is applied yet — Phase 3 wires `specs.*`,
/// Phase 4 each element's preset, and Phase 2's preset *voiding* only asks
/// whether the entry configured anything at all (D-7).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FieldConfig {
    slot: BTreeMap<Slot, BTreeMap<SlotKind, Option<Path>>>,
    specs: BTreeMap<String, Option<Path>>,
    overrides: BTreeMap<Path, Path>,
}

impl FieldConfig {
    /// An entry that configured no `field.*` key at all.
    pub fn new() -> Self {
        Self::default()
    }

    /// True when no `field.*` key was configured at all.
    ///
    /// Not the D-7 trigger — that is [`is_empty_slots`](Self::is_empty_slots),
    /// because the preset assigns slot positions. This predicate answers the
    /// coarser question "was this entry configured at all", which is what the
    /// loader's duplicate-config check wants.
    pub fn is_empty(&self) -> bool {
        self.slot.is_empty() && self.specs.is_empty() && self.overrides.is_empty()
    }

    /// `field.<slot>.<kind>` assignments, by slot then kind.
    ///
    /// `None` for a kind means the position was explicitly unassigned.
    pub fn slots(&self) -> &BTreeMap<Slot, BTreeMap<SlotKind, Option<Path>>> {
        &self.slot
    }

    /// True when no `field.<slot>.<kind>` key was configured.
    ///
    /// This is the D-7 trigger: the preset assigns **slot positions**, so a slot
    /// assignment is what voids it — for this entry only, and as a whole. A
    /// `field.specs.*` or `field.override.*` key defines a field for an extras
    /// item, not a new home for a slot, so it leaves the preset alone
    /// (`S2-4.2`; fixture `34-specs-assignment` is the evidence).
    pub fn is_empty_slots(&self) -> bool {
        self.slot.is_empty()
    }

    /// One position's assignment: `None` = not configured, `Some(None)` =
    /// unassigned (`= ""`), `Some(Some(path))` = assigned.
    pub fn assignment(&self, slot: Slot, kind: SlotKind) -> Option<&Option<Path>> {
        self.slot.get(&slot).and_then(|kinds| kinds.get(&kind))
    }

    /// `field.specs.<k>` assignments (see [`SPECS_KEYS`]).
    pub fn specs(&self) -> &BTreeMap<String, Option<Path>> {
        &self.specs
    }

    /// `field.override.<path>` — the rename table, read only by
    /// `project(node)` (ADR-0014 rules 3–4).
    pub fn overrides(&self) -> &BTreeMap<Path, Path> {
        &self.overrides
    }

    /// Assigns one slot position to `target`, or unassigns it with `None`.
    pub fn assign_slot(&mut self, slot: Slot, kind: SlotKind, target: Option<Path>) {
        self.slot.entry(slot).or_default().insert(kind, target);
    }

    /// Assigns one specs position; the key must be one of [`SPECS_KEYS`].
    pub fn assign_specs(
        &mut self,
        key: &str,
        target: Option<Path>,
    ) -> Result<(), FieldConfigError> {
        if !SPECS_KEYS.contains(&key) {
            return Err(FieldConfigError::UnknownSpecsKey {
                key: key.to_string(),
            });
        }
        self.specs.insert(key.to_string(), target);
        Ok(())
    }

    /// Adds an override from `source` to `target`.
    ///
    /// Both ends are checked here so the failure names the config key the user
    /// wrote, not the map that noticed: a source may be renamed once, and no
    /// two sources may collapse onto one name.
    pub fn add_override(&mut self, source: Path, target: Path) -> Result<(), FieldConfigError> {
        if self.overrides.contains_key(&source) {
            return Err(FieldConfigError::DuplicateOverride { source });
        }
        if self.overrides.values().any(|existing| existing == &target) {
            return Err(FieldConfigError::DuplicateOverrideTarget { target });
        }
        self.overrides.insert(source, target);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(name: &str) -> Path {
        Path::from_dotted(name).expect("valid path")
    }

    fn segments(path: &Path) -> Vec<&str> {
        path.segments().iter().map(String::as_str).collect()
    }

    /// The name rules, accepted side — including every name the proposal's own
    /// assignment block uses (`S2-2.1` exit).
    #[test]
    fn valid_names_are_accepted() {
        for name in [
            "id-123",
            "_extra-note",
            "price.details.amount",
            "class",
            "styles",
            "popupID",
            "ref-id",
            "inner_text",
            "a",
            "A",
            "_",
            "_a",
            "x9",
            "a-b.c_d",
        ] {
            assert_eq!(validate_field_name(name), Ok(()), "{name}");
        }
    }

    /// The rejected side: a leading digit or `-`, an empty segment (leading,
    /// doubled or trailing dot) and every character outside the four classes.
    #[test]
    fn invalid_names_are_rejected() {
        assert_eq!(validate_field_name(""), Err(FieldNameError::Empty));
        for (name, error) in [
            (
                "1x",
                FieldNameError::InvalidStart {
                    segment: 1,
                    ch: '1',
                },
            ),
            (
                "-x",
                FieldNameError::InvalidStart {
                    segment: 1,
                    ch: '-',
                },
            ),
            ("a..b", FieldNameError::EmptySegment { segment: 2 }),
            ("a.", FieldNameError::EmptySegment { segment: 2 }),
            (".a", FieldNameError::EmptySegment { segment: 1 }),
            (
                "a.1b",
                FieldNameError::InvalidStart {
                    segment: 2,
                    ch: '1',
                },
            ),
            (
                "a b",
                FieldNameError::InvalidChar {
                    segment: 1,
                    ch: ' ',
                },
            ),
            (
                "a/b",
                FieldNameError::InvalidChar {
                    segment: 1,
                    ch: '/',
                },
            ),
            // Names are ASCII: an accented letter is not `a-z`.
            (
                "ünicode",
                FieldNameError::InvalidStart {
                    segment: 1,
                    ch: 'ü',
                },
            ),
            (
                "a.$b",
                FieldNameError::InvalidStart {
                    segment: 2,
                    ch: '$',
                },
            ),
        ] {
            assert_eq!(validate_field_name(name), Err(error), "{name}");
        }
    }

    /// Case sensitivity is not a rule to implement — it is a rule to *not*
    /// break: two names that differ only in case are two paths.
    #[test]
    fn names_are_case_sensitive() {
        assert_eq!(validate_field_name("Class"), Ok(()));
        assert_ne!(path("class"), path("Class"));
    }

    /// A config error must say *which* segment is wrong, not just that
    /// something is (D-16: the load aborts and names the offender).
    #[test]
    fn errors_name_the_offending_segment() {
        let message = validate_field_name("price.1x").unwrap_err().to_string();
        assert!(message.contains("segment 2"), "{message}");
        assert!(message.contains('1'), "{message}");
        assert_eq!(
            FieldNameError::Empty.to_string(),
            "a field name cannot be empty"
        );
    }

    /// `from_dotted` is the inverse of the dotted spelling, and `from_segments`
    /// enforces the same rules (one validator, two doors).
    #[test]
    fn paths_split_and_round_trip() {
        let named = path("price.details.amount");
        assert_eq!(segments(&named), ["price", "details", "amount"]);
        assert_eq!(named.len(), 3);
        assert!(!named.is_empty());
        assert_eq!(named.dotted(), "price.details.amount");
        assert_eq!(named.to_string(), "price.details.amount");

        assert_eq!(
            Path::from_dotted("a..b"),
            Err(FieldNameError::EmptySegment { segment: 2 })
        );
        assert_eq!(Path::from_segments(&[]), Err(FieldNameError::Empty));
        assert_eq!(
            Path::from_segments(&["bracket", "bare"]).unwrap().dotted(),
            "bracket.bare"
        );
        assert_eq!(
            Path::from_segments(&["bracket", "1"]),
            Err(FieldNameError::InvalidStart {
                segment: 2,
                ch: '1',
            })
        );
    }

    /// The path-matching table: prefix means *segments*, is inclusive, and
    /// reads in both directions (`S2-2.2` exit).
    #[test]
    fn prefixes_are_segment_wise_and_inclusive() {
        assert!(path("price.details").starts_with(&path("price")));
        assert!(path("price").is_prefix_of(&path("price.details.amount")));
        assert!(path("price").is_prefix_of(&path("price")));
        assert!(path("price").starts_with(&path("price")));
        // A string prefix is not a path prefix.
        assert!(!path("priced").starts_with(&path("price")));
        assert!(!path("price").starts_with(&path("price.details")));
        assert!(!path("price.details").is_prefix_of(&path("price")));
    }

    /// A dotted path nests into real objects, in first-appearance order.
    #[test]
    fn dotted_paths_nest_into_objects() {
        let mut map = FieldMap::new();
        map.insert(path("price.details.amount"), AttrValue::Int(12))
            .unwrap();
        map.insert(path("price.currency"), AttrValue::Str("EUR".to_string()))
            .unwrap();
        assert_eq!(
            map.to_value(),
            AttrValue::Object(vec![(
                "price".to_string(),
                AttrValue::Object(vec![
                    (
                        "details".to_string(),
                        AttrValue::Object(vec![("amount".to_string(), AttrValue::Int(12))]),
                    ),
                    ("currency".to_string(), AttrValue::Str("EUR".to_string())),
                ]),
            )])
        );
    }

    /// Insertion order is output order, and a group keeps the position of its
    /// **first** member even when later siblings arrive.
    #[test]
    fn insertion_order_is_output_order() {
        let mut map = FieldMap::new();
        map.insert(path("b"), AttrValue::Int(1)).unwrap();
        map.insert(path("a.y"), AttrValue::Int(2)).unwrap();
        map.insert(path("a.x"), AttrValue::Int(3)).unwrap();
        map.insert(path("c"), AttrValue::Int(4)).unwrap();
        assert_eq!(
            map.to_value(),
            AttrValue::Object(vec![
                ("b".to_string(), AttrValue::Int(1)),
                (
                    "a".to_string(),
                    AttrValue::Object(vec![
                        ("y".to_string(), AttrValue::Int(2)),
                        ("x".to_string(), AttrValue::Int(3)),
                    ]),
                ),
                ("c".to_string(), AttrValue::Int(4)),
            ])
        );
    }

    /// `price` is a value, so `price.details` has nowhere to nest: the error
    /// names both paths and the map is left as it was.
    #[test]
    fn a_scalar_cannot_become_an_object() {
        let mut map = FieldMap::new();
        map.insert(path("price"), AttrValue::Int(12)).unwrap();
        let error = map
            .insert(path("price.details"), AttrValue::Int(1))
            .expect_err("structure clash");
        assert_eq!(
            error,
            FieldMapError::ParentIsNotObject {
                parent: path("price"),
                child: path("price.details"),
            }
        );
        assert!(error.to_string().contains("`price`"), "{error}");
        assert_eq!(map.len(), 1);
        assert_eq!(map.get(&path("price")), Some(&AttrValue::Int(12)));
    }

    /// The other direction: `price.details` exists, so `price` cannot collapse
    /// into a scalar.
    #[test]
    fn an_object_cannot_collapse_into_a_scalar() {
        let mut map = FieldMap::new();
        map.insert(path("price.details"), AttrValue::Int(1))
            .unwrap();
        assert_eq!(
            map.insert(path("price"), AttrValue::Int(2)),
            Err(FieldMapError::ParentHasChildren {
                parent: path("price"),
                child: path("price.details"),
            })
        );
        assert_eq!(map.len(), 1);
    }

    /// Re-inserting a path replaces the value where it stands, so the order
    /// does not move (the duplicate *policy* is D-12's job, not the map's).
    #[test]
    fn reinserting_a_path_replaces_it_in_place() {
        let mut map = FieldMap::new();
        map.insert(path("x"), AttrValue::Int(1)).unwrap();
        map.insert(path("y"), AttrValue::Int(2)).unwrap();
        map.insert(path("x"), AttrValue::Int(3)).unwrap();
        assert_eq!(map.len(), 2);
        assert_eq!(map.get(&path("x")), Some(&AttrValue::Int(3)));
        assert_eq!(
            map.iter()
                .map(|(path, _)| path.dotted())
                .collect::<Vec<_>>(),
            ["x", "y"]
        );
    }

    /// A sibling of a group is a leaf of that group: `a.b` and `a.c.d` coexist
    /// because neither path is a prefix of the other.
    #[test]
    fn leaves_and_groups_can_share_a_parent() {
        let mut map = FieldMap::new();
        map.insert(path("a.b"), AttrValue::Int(1)).unwrap();
        map.insert(path("a.c.d"), AttrValue::Int(2)).unwrap();
        assert_eq!(
            map.to_value(),
            AttrValue::Object(vec![(
                "a".to_string(),
                AttrValue::Object(vec![
                    ("b".to_string(), AttrValue::Int(1)),
                    (
                        "c".to_string(),
                        AttrValue::Object(vec![("d".to_string(), AttrValue::Int(2))]),
                    ),
                ]),
            )])
        );
    }

    /// An empty map is an empty object, not an absent value.
    #[test]
    fn an_empty_map_is_an_empty_object() {
        let map = FieldMap::new();
        assert!(map.is_empty());
        assert!(!map.contains(&path("a")));
        assert_eq!(map.get(&path("a")), None);
        assert_eq!(map.to_value(), AttrValue::Object(Vec::new()));
    }

    // ---- S2-2.3: the `field.*` config shape ----

    /// A fresh config configures nothing, which is what keeps D-7's preset
    /// alive; any assignment flips it.
    #[test]
    fn a_field_config_reports_whether_it_configured_anything() {
        let mut config = FieldConfig::new();
        assert!(config.is_empty());
        config.assign_slot(Slot::Bracket, SlotKind::Bare, Some(path("anchor_id")));
        assert!(!config.is_empty());
        assert_eq!(
            config.assignment(Slot::Bracket, SlotKind::Bare),
            Some(&Some(path("anchor_id")))
        );
        // An untouched position is absent, not unassigned.
        assert_eq!(config.assignment(Slot::Bracket, SlotKind::Tick), None);
        assert_eq!(config.assignment(Slot::Paren, SlotKind::Bare), None);
    }

    /// `""` (OBJ-10) is stored as `None`: configured, but with no name.
    #[test]
    fn an_empty_name_unassigns_the_position() {
        let mut config = FieldConfig::new();
        config.assign_slot(Slot::Paren, SlotKind::Quote, None);
        assert!(!config.is_empty());
        assert_eq!(config.assignment(Slot::Paren, SlotKind::Quote), Some(&None));
    }

    /// The specs namespace is a fixed list of five, so a typo is a load error
    /// naming the accepted set (D-16, no silent drop).
    #[test]
    fn specs_keys_are_exactly_the_five_positions() {
        let mut config = FieldConfig::new();
        for key in SPECS_KEYS {
            config.assign_specs(key, Some(path("ref-id"))).unwrap();
        }
        assert_eq!(config.specs().len(), 5);
        assert_eq!(SPECS_KEYS, ["tick", "quote", "class", "id", "style"]);

        let error = config
            .assign_specs("tikc", Some(path("ref-id")))
            .expect_err("unknown specs position");
        assert_eq!(
            error,
            FieldConfigError::UnknownSpecsKey {
                key: "tikc".to_string()
            }
        );
        let message = error.to_string();
        assert!(message.contains("tikc"), "{message}");
        assert!(
            message.contains("tick, quote, class, id, style"),
            "{message}"
        );
    }

    /// ADR-0014 rule 3: one source, one target — the two collision directions
    /// are caught where the key is read.
    #[test]
    fn overrides_may_not_collide() {
        let mut config = FieldConfig::new();
        config
            .add_override(path("number.raw"), path("bar"))
            .unwrap();
        assert_eq!(config.overrides().len(), 1);

        assert_eq!(
            config.add_override(path("number.raw"), path("baz")),
            Err(FieldConfigError::DuplicateOverride {
                source: path("number.raw")
            })
        );
        assert_eq!(
            config.add_override(path("specs.tick"), path("bar")),
            Err(FieldConfigError::DuplicateOverrideTarget {
                target: path("bar")
            })
        );
        assert_eq!(config.overrides().len(), 1);
    }

    /// `Path` is `Ord` so a config can key a `BTreeMap` by it. The order is
    /// segment-wise, which is what makes a dotted map readable in a diff.
    #[test]
    fn paths_are_ordered_by_segment() {
        assert!(path("a.b") < path("a.c"));
        assert!(path("a") < path("a.b"));
        assert!(path("a.b.c") < path("a.c"));
    }

    /// A `FieldConfig` is the entry's; the same path in two different configs
    /// is not a collision.
    #[test]
    fn separate_configs_do_not_share_overrides() {
        let mut first = FieldConfig::new();
        let mut second = FieldConfig::new();
        first.add_override(path("number.raw"), path("bar")).unwrap();
        second
            .add_override(path("number.raw"), path("bar"))
            .unwrap();
        assert_eq!(first, second);
    }

    /// The built-in names a preset table writes. Each is one segment, so
    /// `path()` is total and the projection never sees a dotted preset name.
    #[test]
    fn a_built_in_name_is_validated_at_compile_time() {
        const NAMES: [FieldName; 6] = [
            FieldName::new("inner_text"),
            FieldName::new("href"),
            FieldName::new("title"),
            FieldName::new("reference_id"),
            FieldName::new("_private"),
            FieldName::new("alt"),
        ];
        for name in NAMES {
            assert_eq!(name.path().segments(), [name.as_str()]);
            assert!(validate_field_name(name.as_str()).is_ok());
        }
        assert_eq!(FieldName::new("href").to_string(), "href");
    }

    /// The identity law (`S2-2.7` exit): the public fields of a node are exactly
    /// its attributes, in the AST's own order, with their own values — that is
    /// what lets both renderers read the map instead of the node without moving
    /// a byte. Internals (`__`-prefixed, §11 rule 3) are the one exception.
    #[test]
    fn the_projection_is_an_identity_pass_over_public_fields() {
        let node = serde_json::json!({
            "type": "Heading",
            "attrs": {
                "__layer": "heading",
                "class": "hero",
                "id": "h01",
                "level": "2",
                "number": ["1", "a"],
                "score": 12,
                "draft": true,
            }
        });
        let projected = project(&node, None);

        let ast: Vec<(String, serde_json::Value)> = node["attrs"]
            .as_object()
            .unwrap()
            .iter()
            .filter(|(key, _)| !key.starts_with("__"))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        assert_eq!(projected.attrs(), ast);

        // The renderer-facing view is also the JSON/AST renderers' view.
        assert_eq!(
            projected.to_value().to_json(),
            serde_json::json!({
                "class": "hero",
                "id": "h01",
                "level": "2",
                "number": ["1", "a"],
                "score": 12,
                "draft": true,
            })
        );
        assert!(!projected.contains(&path("__layer")));
    }

    /// A node without attributes projects to an empty map — renderers then emit
    /// no attributes at all, exactly as before.
    #[test]
    fn a_node_without_attrs_projects_to_nothing() {
        assert!(project(&serde_json::json!({ "type": "ThematicBreak" }), None).is_empty());
        assert!(project(&serde_json::json!({ "type": "Text", "text": "hi" }), None).is_empty());
    }

    /// D-4: the override table renames a field *at the projection boundary*, and
    /// the renamed field keeps the position of the field it replaced — so an
    /// override cannot reorder the output.
    #[test]
    fn an_override_renames_a_field_in_place() {
        let node = serde_json::json!({
            "attrs": { "class": "hero", "slug": "s1", "title": "T" }
        });
        let mut config = FieldConfig::new();
        config
            .add_override(path("slug"), path("heading.anchor"))
            .unwrap();

        let projected = project(&node, Some(&config));
        let names: Vec<String> = projected.iter().map(|(path, _)| path.dotted()).collect();
        assert_eq!(names, ["class", "heading.anchor", "title"]);
        assert_eq!(
            projected.get(&path("heading.anchor")),
            Some(&AttrValue::Str("s1".to_string()))
        );
        // The source name is gone: the field was renamed, not duplicated.
        assert!(!projected.contains(&path("slug")));
        // Without the config the same node keeps its own name.
        assert!(project(&node, None).contains(&path("slug")));
    }

    /// An override naming a field the node does not carry is simply not applied
    /// (a preset may have left the slot empty — that is not an error).
    #[test]
    fn an_override_for_a_missing_field_is_ignored() {
        let node = serde_json::json!({ "attrs": { "class": "hero" } });
        let mut config = FieldConfig::new();
        config.add_override(path("slug"), path("id")).unwrap();
        assert_eq!(
            project(&node, Some(&config)).attrs(),
            [("class".to_string(), serde_json::json!("hero"))]
        );
    }
}
