//! §5.4 slot modifiers — the configuration half (D-9, ADR-0017).
//!
//! A bracket's `|…` run is **pure configuration**: every symbol of the run names
//! a set of public fields in the entry's config
//! (`bracket.modifier."?" = { rel = "nofollow" }`), and the run contributes
//! those fields to the element. Pendon ships no modifier of its own, and a
//! symbol with no config is silently ignored — the run is legal, it just says
//! nothing.
//!
//! Two rules live here and nowhere else:
//!
//! 1. **Canonical order** is first appearance in the bracket, left to right,
//!    after deduplication ([`SlotPair::modifiers`] already collapsed it);
//! 2. **Merge** is by target path: two contributions to one path become an
//!    array in symbol order, an array beats a scalar, two arrays nest, two
//!    objects deep merge, and a path that cannot hold both values is an error
//!    — the element then drops the whole run ([`merge`]).
//!
//! Contributions are **specs-level fields**: they are ordinary public field
//! paths (`class`, `rel`, `soo.baz`), mapped through
//! [`universal_field`](crate::typed::universal_field) exactly like a specs
//! position, and never renamed from `field.*` config (D-5: a modifier is not a
//! field config key).

use std::collections::BTreeMap;
use std::fmt;

use crate::field::{FieldMap, FieldMapError, Path};
use crate::slot::SlotPair;
use crate::typed::{universal_field, ExtrasAttr};
use crate::value::AttrValue;

/// One entry's `bracket.modifier."<symbol>"` tables: the fields each symbol of
/// a run contributes.
///
/// The map is keyed by symbol because the *run* is what the document writes;
/// [`ModifierConfig::define`] fills it at config load, so an empty config (the
/// common case — Pendon ships no modifier) costs nothing at render time.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ModifierConfig {
    contributions: BTreeMap<char, Vec<(Path, AttrValue)>>,
}

impl ModifierConfig {
    /// A config that defines no symbol.
    pub fn new() -> Self {
        Self::default()
    }

    /// True when no `bracket.modifier."…"` key was configured.
    pub fn is_empty(&self) -> bool {
        self.contributions.is_empty()
    }

    /// Defines (or replaces) one symbol's contribution.
    ///
    /// A symbol is **one** `char` — TOML already forbids the same key twice, so
    /// a replacement can only come from a re-parsed entry.
    pub fn define(&mut self, symbol: char, fields: Vec<(Path, AttrValue)>) {
        self.contributions.insert(symbol, fields);
    }

    /// What `symbol` contributes; empty for a symbol the entry does not define.
    pub fn for_symbol(&self, symbol: char) -> &[(Path, AttrValue)] {
        self.contributions
            .get(&symbol)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Every configured symbol, in `char` order (config order is irrelevant:
    /// the document decides the run's order).
    pub fn symbols(&self) -> impl Iterator<Item = char> + '_ {
        self.contributions.keys().copied()
    }
}

/// One field a modifier run contributes to the element (§5.4, D-9).
#[derive(Debug, Clone, PartialEq)]
pub struct Contribution {
    /// The public field name — already mapped through the universal DOM fields,
    /// so `specs.class` arrives as `class`.
    pub name: String,
    /// The symbol that first contributed this path; named by the diagnostic a
    /// field the element already carries raises.
    pub symbol: char,
    /// The value, or [`ExtrasAttr::Flag`] for a bare `true`.
    pub attr: ExtrasAttr,
}

impl Contribution {
    /// The `Event::Attribute` value for this field (§6.3 transport).
    ///
    /// A structured value (object / array) travels as `JSON_ATTR_PREFIX` +
    /// compact JSON, so the AST — and therefore `plugin-extract` — keeps an
    /// array or an object instead of a string that merely looks like one. A
    /// scalar keeps its literal text, exactly like a specs item.
    pub fn text(&self) -> String {
        match &self.attr {
            ExtrasAttr::Flag => self.name.clone(),
            ExtrasAttr::Value(value @ (AttrValue::Object(_) | AttrValue::Array(_))) => format!(
                "{}{}",
                pendon_core::JSON_ATTR_PREFIX,
                serde_json::to_string(&value.to_json()).unwrap_or_else(|_| "null".to_string())
            ),
            ExtrasAttr::Value(value) => value.literal(),
        }
    }

    /// True for a bare flag (`isFoo = true`), which renders as a bare attribute
    /// (`<a isFoo>`) and never as `isFoo="isFoo"` (§6.3).
    pub fn is_flag(&self) -> bool {
        matches!(self.attr, ExtrasAttr::Flag)
    }
}

/// Why a run contributes nothing (D-9): the merged fields are unusable, so the
/// element falls back to **no** modifier contribution at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModifierError(String);

impl ModifierError {
    /// Two symbols wrote one path with values that cannot live together.
    fn irreconcilable(path: &Path, symbol: char, other: char) -> Self {
        Self(format!(
            "modifiers `{other}` and `{symbol}` both write `{path}` with values that cannot be \
             merged; the modifier run is ignored"
        ))
    }

    /// One contributed path cannot be nested beside another (`soo` vs `soo.baz`).
    fn clash(path: &Path, symbol: char, error: FieldMapError) -> Self {
        Self(format!(
            "modifier `{symbol}` cannot contribute `{path}` ({error}); the modifier run is ignored"
        ))
    }

    /// The message a plugin reports as a `Severity::Error` (§13).
    pub fn message(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ModifierError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ModifierError {}

/// The fields `pair`'s modifier run contributes, merged per D-9.
///
/// `None` / an empty run / a symbol the config does not define all answer an
/// empty list with no diagnostic: a modifier without config is legal.
///
/// The output order is the **first appearance** of each path across the run
/// (D-9 rule 1), so a fixture can pin it.
pub fn merge(
    config: Option<&ModifierConfig>,
    modifiers: &[char],
) -> Result<Vec<Contribution>, ModifierError> {
    let Some(config) = config else {
        return Ok(Vec::new());
    };
    if config.is_empty() || modifiers.is_empty() {
        return Ok(Vec::new());
    }

    let mut fields = FieldMap::new();
    let mut origin: BTreeMap<String, char> = BTreeMap::new();
    for &symbol in modifiers {
        for (path, value) in config.for_symbol(symbol) {
            let target = public_path(path);
            let merged = match fields.get(&target) {
                None => value.clone(),
                Some(existing) => merge_value(existing, value.clone()).ok_or_else(|| {
                    let other = origin.get(&target.dotted()).copied().unwrap_or(symbol);
                    ModifierError::irreconcilable(&target, symbol, other)
                })?,
            };
            fields
                .insert(target.clone(), merged)
                .map_err(|error| ModifierError::clash(&target, symbol, error))?;
            origin.entry(target.dotted()).or_insert(symbol);
        }
    }

    Ok(fields
        .iter()
        .map(|(path, value)| {
            let name = path.dotted();
            Contribution {
                symbol: origin.get(&name).copied().unwrap_or('?'),
                attr: match value {
                    // A `true` contribution is a bare flag, exactly like the
                    // extras `isFoo` item it mirrors (§6.3).
                    AttrValue::Bool(true) => ExtrasAttr::Flag,
                    other => ExtrasAttr::Value(other.clone()),
                },
                name,
            }
        })
        .collect())
}

/// [`merge`] for the run of a parsed slot pair.
pub fn contributions(
    config: Option<&ModifierConfig>,
    pair: &SlotPair,
) -> Result<Vec<Contribution>, ModifierError> {
    merge(config, &pair.modifiers)
}

/// §6.3: a universal specs path is written as the DOM field it names
/// (`specs.class` → `class`) — the same rename [`project`](crate::project) does
/// for a node attribute, applied here because a contribution never passes
/// through `to_attributes`.
fn public_path(path: &Path) -> Path {
    match universal_field(&path.dotted()) {
        Some(field) => Path::from_dotted(field).expect("a universal field name is a valid path"),
        None => path.clone(),
    }
}

/// `true` / a bare flag aside, two contributions to one path merge per D-9 — but
/// `class` is the one field that **accumulates** (§6.4, D-12) instead of
/// clashing, so a modifier can add tokens to the classes the head already set.
///
/// `existing` is the element's current `class` value; the result keeps its order
/// and never repeats a token it already carries.
pub fn append_class(existing: Option<&str>, text: &str) -> String {
    let mut tokens: Vec<&str> = existing.unwrap_or_default().split_whitespace().collect();
    for token in text.split_whitespace() {
        if !tokens.contains(&token) {
            tokens.push(token);
        }
    }
    tokens.join(" ")
}

/// D-9's merge table for one target path; `None` when the two values cannot
/// live together.
fn merge_value(existing: &AttrValue, incoming: AttrValue) -> Option<AttrValue> {
    use AttrValue::{Array, Object};
    match (existing, incoming) {
        // Two lists stay two lists: nesting keeps both, in symbol order.
        (Array(left), Array(right)) => Some(Array(vec![Array(left.clone()), Array(right)])),
        // A list is never flattened back into text.
        (Array(_), _) => Some(existing.clone()),
        (_, right @ Array(_)) => Some(right),
        // Two objects merge key by key; a key only one of them has survives.
        (Object(left), Object(right)) => deep_merge(left.clone(), right).map(Object),
        // A scalar and an object have no common shape.
        (Object(_), _) | (_, Object(_)) => None,
        // Two scalars become an array in symbol order (`rel` from `?` and `!`).
        (_, incoming) => Some(Array(vec![existing.clone(), incoming])),
    }
}

/// D-9's object + object rule: the same path merge, one level down.
fn deep_merge(
    mut left: Vec<(String, AttrValue)>,
    right: Vec<(String, AttrValue)>,
) -> Option<Vec<(String, AttrValue)>> {
    for (key, value) in right {
        match left.iter_mut().find(|(known, _)| *known == key) {
            Some((_, existing)) => *existing = merge_value(existing, value)?,
            None => left.push((key, value)),
        }
    }
    Some(left)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::slot::{parse_slots, SlotMatch};

    fn path(name: &str) -> Path {
        Path::from_dotted(name).expect("a valid field name")
    }

    /// The proposal's own pair of symbols (D-9): `?` and `!` both write `rel`,
    /// and both write `soo.baz`.
    fn proposal() -> ModifierConfig {
        let mut config = ModifierConfig::new();
        config.define(
            '?',
            vec![
                (path("rel"), AttrValue::Str("nofollow".to_string())),
                (path("soo.baz"), AttrValue::Str("wax".to_string())),
            ],
        );
        config.define(
            '!',
            vec![
                (path("rel"), AttrValue::Str("noopener".to_string())),
                (path("soo.baz"), AttrValue::Str("wax".to_string())),
            ],
        );
        config
    }

    fn run(source: &str, config: &ModifierConfig) -> Vec<Contribution> {
        let pair = match parse_slots(source) {
            SlotMatch::Slots { pair, .. } => pair,
            other => panic!("{source}: {other:?}"),
        };
        contributions(Some(config), &pair).expect("mergeable")
    }

    fn names(fields: &[Contribution]) -> Vec<String> {
        fields.iter().map(|field| field.name.clone()).collect()
    }

    fn text(value: &str) -> AttrValue {
        AttrValue::Str(value.to_string())
    }

    /// D-9's worked example, field for field: two symbols, two paths, two
    /// arrays in written order.
    #[test]
    fn two_symbols_merge_into_arrays_in_written_order() {
        let fields = run("[Foo |?!]", &proposal());
        assert_eq!(names(&fields), ["rel", "soo.baz"]);
        assert_eq!(
            fields[0].attr,
            ExtrasAttr::Value(AttrValue::Array(vec![text("nofollow"), text("noopener")]))
        );
        assert_eq!(
            fields[1].attr,
            ExtrasAttr::Value(AttrValue::Array(vec![text("wax"), text("wax")]))
        );
        assert_eq!(fields[0].symbol, '?');
    }

    /// Canonical order is the document's, never the config's or the code
    /// point's: `|!?` still contributes `!` first.
    #[test]
    fn the_run_order_is_the_documents() {
        let fields = run("[Foo |!?]", &proposal());
        assert_eq!(fields[0].symbol, '!');
        assert_eq!(
            fields[0].attr,
            ExtrasAttr::Value(AttrValue::Array(vec![text("noopener"), text("nofollow")]))
        );
    }

    /// Repetition collapsed already (the slot parser's job), so a symbol can
    /// never contribute twice from one bracket.
    #[test]
    fn repetition_collapses_before_merging() {
        let fields = run("[Foo |??!]", &proposal());
        assert_eq!(
            fields[0].attr,
            ExtrasAttr::Value(AttrValue::Array(vec![text("nofollow"), text("noopener")]))
        );
    }

    /// A symbol with no config is ignored — a modifier without config is legal,
    /// and says nothing about the element.
    #[test]
    fn an_undefined_symbol_contributes_nothing() {
        assert_eq!(run("[Foo |xyz]", &proposal()), Vec::new());
    }

    /// No run at all, and no config at all: the common case costs nothing.
    #[test]
    fn an_empty_run_or_config_is_silent() {
        assert!(run("[Foo]", &proposal()).is_empty());
        assert!(merge(None, &['?']).expect("no config").is_empty());
        assert!(merge(Some(&ModifierConfig::new()), &['?'])
            .expect("empty config")
            .is_empty());
    }

    /// D-9's merge table, case by case.
    #[test]
    fn merge_follows_the_table() {
        use AttrValue::{Array, Object};

        // array + scalar → the array survives untouched.
        assert_eq!(
            merge_value(&Array(vec![text("a")]), text("b")),
            Some(Array(vec![text("a")]))
        );
        // scalar + array → the array.
        assert_eq!(
            merge_value(&text("b"), Array(vec![text("a")])),
            Some(Array(vec![text("a")]))
        );
        // array + array → nested, so both lists stay lists.
        assert_eq!(
            merge_value(&Array(vec![text("a")]), Array(vec![text("b")])),
            Some(Array(vec![Array(vec![text("a")]), Array(vec![text("b")]),]))
        );
        // object + object → deep merge; a scalar written twice becomes an array
        // one level down, and a key only one object has survives.
        assert_eq!(
            merge_value(
                &Object(vec![("a".to_string(), text("x"))]),
                Object(vec![
                    ("a".to_string(), text("y")),
                    ("b".to_string(), text("z")),
                ])
            ),
            Some(Object(vec![
                ("a".to_string(), Array(vec![text("x"), text("y")])),
                ("b".to_string(), text("z")),
            ]))
        );
        // object + scalar → irreconcilable, both ways round.
        assert_eq!(
            merge_value(&Object(vec![("a".to_string(), text("x"))]), text("y")),
            None
        );
        assert_eq!(
            merge_value(&text("y"), Object(vec![("a".to_string(), text("x"))])),
            None
        );
    }

    /// An irreconcilable pair drops the whole run (D-9), with a message naming
    /// both symbols and the path.
    #[test]
    fn an_irreconcilable_merge_drops_the_run() {
        let mut config = ModifierConfig::new();
        config.define('?', vec![(path("soo"), AttrValue::Object(Vec::new()))]);
        config.define('!', vec![(path("soo"), text("x"))]);
        let error = merge(Some(&config), &['?', '!']).expect_err("object vs scalar");
        assert!(
            error.message().contains("`?`") && error.message().contains("`!`"),
            "{}",
            error.message()
        );
        assert!(error.message().contains("`soo`"), "{}", error.message());
    }

    /// A path that cannot be nested beside another (`soo` and `soo.baz`) is the
    /// other failure D-9 keeps in the `Severity::Error` tier.
    #[test]
    fn a_nesting_clash_drops_the_run() {
        let mut config = ModifierConfig::new();
        config.define('?', vec![(path("soo"), text("x"))]);
        config.define('!', vec![(path("soo.baz"), text("y"))]);
        let error = merge(Some(&config), &['?', '!']).expect_err("prefix clash");
        assert!(error.message().contains("soo.baz"), "{}", error.message());
    }

    /// §6.3: a contribution is a public field, so `specs.class` is `class` —
    /// and two symbols writing the two spellings merge into one path.
    #[test]
    fn universal_specs_paths_are_renamed() {
        let mut config = ModifierConfig::new();
        config.define('?', vec![(path("specs.class"), text("a"))]);
        config.define('!', vec![(path("class"), text("b"))]);
        let fields = merge(Some(&config), &['?', '!']).expect("mergeable");
        assert_eq!(names(&fields), ["class"]);
        assert_eq!(
            fields[0].attr,
            ExtrasAttr::Value(AttrValue::Array(vec![text("a"), text("b")]))
        );
    }

    /// `true` is a bare flag (§6.3); every other value keeps its literal text,
    /// and a structured one is carried as JSON so extraction sees the array.
    #[test]
    fn values_are_carried_the_way_a_specs_item_is() {
        let mut config = ModifierConfig::new();
        config.define(
            '?',
            vec![
                (path("isLazy"), AttrValue::Bool(true)),
                (path("rel"), text("nofollow")),
                (path("parts"), AttrValue::Array(vec![text("a")])),
            ],
        );
        let fields = merge(Some(&config), &['?']).expect("mergeable");
        assert!(fields[0].is_flag());
        assert_eq!(fields[0].attr, ExtrasAttr::Flag);
        assert_eq!(fields[1].text(), "nofollow");
        assert_eq!(
            fields[2].text(),
            format!("{}[\"a\"]", pendon_core::JSON_ATTR_PREFIX)
        );
    }
}
