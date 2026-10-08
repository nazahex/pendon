//! §11 component sets: one layer = typed entries + at most one default.
//!
//! A task-level plugin owns one **layer** per element it emits (`table`,
//! `thead`, `caption`, `img`, `figure`, `heading`, `anchor`, `cite`, …). Each
//! layer is a *component set*: entries answering one or more `@@type{…}`
//! markers, plus an optional default that answers every instance the typed
//! entries do not.
//!
//! Selection is §11 rule 3 — exact `type` match → layer default → `None`, where
//! `None` means "use the built-in fallback element, which still carries every
//! extra as an attribute".
//!
//! The type lives next to [`ComponentTemplate`](crate::ComponentTemplate) and
//! [`ImportEntry`](crate::ImportEntry) because it is part of the custom
//! component contract the plugins hand to the renderer.

use pendon_extra::{ExtrasOptions, PositionalKeys};
use serde::{Deserialize, Serialize};

/// One entry of a §11 component set: the `type` markers it answers, the §6.1
/// positional keys it names (its own `backtick_key` / `quote_key`, plus
/// `bracket_key` / `parentheses_key` for a directive head) and the component it
/// renders with.
///
/// An empty `types` list marks the **layer default** (§11 rule 2).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypedComponent<C> {
    #[serde(default)]
    pub types: Vec<String>,
    pub component: C,
    /// §6.1 / §11 rule 5: the positional overrides this entry contributes when
    /// it answers a `@@type{…}` marker. All-`Option`, so leaving every key unset
    /// keeps the built-in `slug` / `title`.
    #[serde(default)]
    pub positional: PositionalKeys,
}

impl<C> TypedComponent<C> {
    /// A typed entry: it answers exactly the listed `@@type{…}` markers.
    pub fn typed<I, S>(types: I, component: C) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            types: types.into_iter().map(Into::into).collect(),
            component,
            positional: PositionalKeys::default(),
        }
    }

    /// The layer default: it answers every instance no typed entry claims.
    pub fn default_component(component: C) -> Self {
        Self {
            types: Vec::new(),
            component,
            positional: PositionalKeys::default(),
        }
    }

    /// Overrides the §6.1 positional keys the entry names (§11 rule 5).
    pub fn with_positional(mut self, positional: PositionalKeys) -> Self {
        self.positional = positional;
        self
    }

    /// `true` for the single default entry of a layer.
    pub fn is_default(&self) -> bool {
        self.types.is_empty()
    }

    /// Answers `type_marker` (§11 rule 2).
    pub fn matches_type(&self, type_marker: &str) -> bool {
        self.types.iter().any(|known| known == type_marker)
    }
}

/// The §11 component set of one layer.
///
/// Entries are kept in declaration order so hints are emitted deterministically;
/// the CLI rejects a second default at config load (§11 rule 2), so `select`
/// may assume at most one.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentSet<C> {
    #[serde(default)]
    entries: Vec<TypedComponent<C>>,
}

impl<C> ComponentSet<C> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn from_entries(entries: impl IntoIterator<Item = TypedComponent<C>>) -> Self {
        Self {
            entries: entries.into_iter().collect(),
        }
    }

    /// Appends one entry, optionally typed.
    pub fn push(&mut self, types: Vec<String>, component: C) {
        self.entries.push(TypedComponent {
            types,
            component,
            positional: PositionalKeys::default(),
        });
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn entries(&self) -> &[TypedComponent<C>] {
        &self.entries
    }

    /// Mutable access to the entries, for configuration-time edits.
    pub fn entries_mut(&mut self) -> &mut [TypedComponent<C>] {
        &mut self.entries
    }

    /// Every component of the set (typed entries first, then the default) — the
    /// order the renderer hints are built in.
    pub fn components(&self) -> impl Iterator<Item = &C> {
        self.entries.iter().map(|entry| &entry.component)
    }

    /// The layer-default entry, when declared.
    pub fn default_entry(&self) -> Option<&TypedComponent<C>> {
        self.entries.iter().find(|entry| entry.is_default())
    }

    /// The layer default, when declared.
    pub fn default_component(&self) -> Option<&C> {
        self.default_entry().map(|entry| &entry.component)
    }

    /// §11 rule 3: exact `type` match → layer default → `None` (built-in
    /// fallback element).
    pub fn select(&self, type_marker: Option<&str>) -> Option<&C> {
        if let Some(marker) = type_marker {
            if let Some(entry) = self.entries.iter().find(|entry| entry.matches_type(marker)) {
                return Some(&entry.component);
            }
        }
        self.default_component()
    }

    /// §6.1 / §11 rules 3 and 5: the positional keys named by the entry that
    /// answers `type_marker`. An exact `type` match wins, then the layer default,
    /// then the built-in `slug` / `title`. The construct plugins pass the result
    /// to `pendon_extra::to_attributes` when they map an extras head onto
    /// attributes.
    pub fn keys_for(&self, type_marker: Option<&str>) -> ExtrasOptions {
        self.keys_for_opt(type_marker).unwrap_or_default()
    }

    /// Like [`keys_for`](Self::keys_for) but `None` when this set has neither an
    /// exact-`type` entry nor a layer default — so a plugin that owns several
    /// layers (`plugin-list`) can try each in turn before falling back.
    pub fn keys_for_opt(&self, type_marker: Option<&str>) -> Option<ExtrasOptions> {
        if let Some(marker) = type_marker {
            if let Some(entry) = self.entries.iter().find(|entry| entry.matches_type(marker)) {
                return Some(entry.positional.resolve());
            }
        }
        self.default_entry().map(|entry| entry.positional.resolve())
    }

    /// The one component that applies to every instance: the layer default, or
    /// the layer's only entry.
    pub fn single(&self) -> Option<&C> {
        if let Some(default) = self.default_component() {
            return Some(default);
        }
        match self.entries.as_slice() {
            [only] => Some(&only.component),
            _ => None,
        }
    }
}

impl<C> FromIterator<TypedComponent<C>> for ComponentSet<C> {
    fn from_iter<I: IntoIterator<Item = TypedComponent<C>>>(iter: I) -> Self {
        Self::from_entries(iter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set() -> ComponentSet<&'static str> {
        ComponentSet::from_entries([
            TypedComponent::typed(["a", "b"], "AB"),
            TypedComponent::typed(["c"], "C"),
            TypedComponent::default_component("Default"),
        ])
    }

    /// §11 rule 3: exact match first, then the default, then the fallback.
    #[test]
    fn selects_type_then_default_then_fallback() {
        let set = set();
        assert_eq!(set.select(Some("a")), Some(&"AB"));
        assert_eq!(set.select(Some("c")), Some(&"C"));
        assert_eq!(set.select(Some("unknown")), Some(&"Default"));
        assert_eq!(set.select(None), Some(&"Default"));
    }

    /// A set without a default falls back to the built-in element for every
    /// unclaimed marker (§11 rule 3) — it never guesses an entry.
    #[test]
    fn without_a_default_unclaimed_markers_fall_back() {
        let set = ComponentSet::from_entries([TypedComponent::typed(["a"], "A")]);
        assert_eq!(set.select(Some("a")), Some(&"A"));
        assert_eq!(set.select(Some("b")), None);
        assert_eq!(set.select(None), None);
    }

    /// The CLI's routable-layer check maps onto `single`.
    #[test]
    fn single_reports_the_unambiguous_component() {
        let only = ComponentSet::from_entries([TypedComponent::typed(["a"], "A")]);
        assert_eq!(only.single(), Some(&"A"));

        let with_default = set();
        assert_eq!(with_default.single(), Some(&"Default"));

        let ambiguous = ComponentSet::from_entries([
            TypedComponent::typed(["a"], "A"),
            TypedComponent::typed(["b"], "B"),
        ]);
        assert_eq!(ambiguous.single(), None);
        assert!(ComponentSet::<&str>::new().is_empty());
    }

    #[test]
    fn components_keep_declaration_order() {
        let names: Vec<&str> = set().components().copied().collect();
        assert_eq!(names, vec!["AB", "C", "Default"]);
    }
}
