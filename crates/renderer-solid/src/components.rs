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

use pendon_extra::{FieldConfig, PositionalNames};
use pendon_types::Selector;
use serde::{Deserialize, Serialize};

use crate::imports::{ComponentTemplate, ImportEntry};

/// One entry of a §11 component set: the `type` selectors it answers, the
/// `field.*` config it carries (the one read side of §6.1 positional naming, D-5)
/// and the component it renders with.
///
/// An empty `selectors` list marks the **layer default** (§11 rule 2). A
/// wildcard (`?`, `_`, `*`) is still a *rule*, never the default slot.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypedComponent<C> {
    #[serde(default)]
    pub selectors: Vec<Selector>,
    pub component: C,
    /// §6.1 / D-5: the entry's `field.*` keys. It is the *only* source of
    /// positional naming besides the built-in `slug` / `title`, and an entry that
    /// configures nothing keeps every built-in
    /// ([`PositionalNames::resolve`]).
    ///
    /// Not serialized: the renderer contract is `(selectors, component)`, and
    /// `FieldConfig` is configuration, not part of a rendered template.
    #[serde(skip)]
    pub field: FieldConfig,
}

impl<C> TypedComponent<C> {
    /// A typed entry: it answers exactly the listed `@@type{…}` markers.
    ///
    /// The strings are literal marker names; use
    /// [`typed_selectors`](Self::typed_selectors) for `?` / `_` / `*`.
    pub fn typed<I, S>(types: I, component: C) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::typed_selectors(
            types.into_iter().map(|name| Selector::Exact(name.into())),
            component,
        )
    }

    /// A typed entry from already-parsed selectors (wildcards included).
    pub fn typed_selectors<I>(selectors: I, component: C) -> Self
    where
        I: IntoIterator<Item = Selector>,
    {
        Self {
            selectors: selectors.into_iter().collect(),
            component,
            field: FieldConfig::new(),
        }
    }

    /// The layer default: it answers every instance no typed entry claims.
    pub fn default_component(component: C) -> Self {
        Self {
            selectors: Vec::new(),
            component,
            field: FieldConfig::new(),
        }
    }

    /// Attaches the §6.1 `field.*` config the entry carries (D-5).
    pub fn with_field(mut self, field: FieldConfig) -> Self {
        self.field = field;
        self
    }

    /// `true` for the single default entry of a layer.
    pub fn is_default(&self) -> bool {
        self.selectors.is_empty()
    }

    /// Answers `type_marker` through a **literal** selector (§11 rule 2).
    ///
    /// Wildcards need the document's occurring types, so they are resolved by
    /// the renderer (decision D-3a); this literal check stays for the paths that
    /// only have one node at hand (`names_for`, the CLI's duplicate check).
    pub fn matches_type(&self, type_marker: &str) -> bool {
        self.selectors
            .iter()
            .any(|selector| selector.literal() == Some(type_marker))
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

    /// Appends one entry, optionally typed (literal marker names).
    pub fn push(&mut self, types: Vec<String>, component: C) {
        self.entries.push(TypedComponent::typed(types, component));
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

    /// §6.1 / D-5: the positional names of the entry that answers `type_marker`.
    /// An exact `type` match wins, then the layer default, then the built-in
    /// `slug` / `title`. The construct plugins pass the result to
    /// `pendon_extra::to_attributes` when they map an extras head onto attributes.
    pub fn names_for(&self, type_marker: Option<&str>) -> PositionalNames {
        self.names_for_opt(type_marker).unwrap_or_default()
    }

    /// Like [`names_for`](Self::names_for) but `None` when this set has neither an
    /// exact-`type` entry nor a layer default — so a plugin that owns several
    /// layers (`plugin-list`) can try each in turn before falling back.
    pub fn names_for_opt(&self, type_marker: Option<&str>) -> Option<PositionalNames> {
        if let Some(marker) = type_marker {
            if let Some(entry) = self.entries.iter().find(|entry| entry.matches_type(marker)) {
                return Some(PositionalNames::resolve(Some(&entry.field)));
            }
        }
        self.default_entry()
            .map(|entry| PositionalNames::resolve(Some(&entry.field)))
    }
}

impl<C> FromIterator<TypedComponent<C>> for ComponentSet<C> {
    fn from_iter<I: IntoIterator<Item = TypedComponent<C>>>(iter: I) -> Self {
        Self::from_entries(iter)
    }
}

/// One renderer template per entry of `set`, tagged with the §11 `layer` it
/// belongs to (ADR-0008 `D-3a`).
///
/// A plugin registers **what each entry answers** — the selectors the user
/// declared, wildcards included — and the renderer decides once per document
/// which entry actually renders, so `_` / `*` see the whole document.
/// `payload` extracts the `(template, imports)` pair from an entry's component.
pub fn layer_templates<C, F>(
    layer: &str,
    set: &ComponentSet<C>,
    mut payload: F,
) -> Vec<ComponentTemplate>
where
    F: FnMut(&C) -> (String, Vec<ImportEntry>),
{
    set.entries()
        .iter()
        .map(|entry| {
            let (template, imports) = payload(&entry.component);
            ComponentTemplate::new_layer(layer, entry.selectors.clone(), template, imports)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pendon_extra::{Path, Slot, SlotKind};

    fn set() -> ComponentSet<&'static str> {
        ComponentSet::from_entries([
            TypedComponent::typed(["a", "b"], "AB"),
            TypedComponent::typed(["c"], "C"),
            TypedComponent::default_component("Default"),
        ])
    }

    /// §11 rule 3 / decision D-3a: positional naming is literal-only. A `_` or
    /// `*` entry has no single component to read a name from, so it contributes
    /// nothing — a wildcard never silently overrides `slug`/`title`.
    #[test]
    fn names_for_ignores_wildcard_entries() {
        let mut wildcard: ComponentSet<&str> =
            ComponentSet::from_entries([TypedComponent::typed_selectors(
                [Selector::Underscore],
                "Rest",
            )]);
        let mut field = FieldConfig::new();
        field.assign_slot(
            Slot::Bracket,
            SlotKind::Bare,
            Some(Path::from_dotted("anchor").unwrap()),
        );
        wildcard.entries_mut()[0].field = field;
        assert_eq!(wildcard.names_for_opt(Some("TypeA")), None);
        assert_eq!(
            wildcard.names_for(Some("TypeA")),
            PositionalNames::default()
        );
        assert!(!wildcard.entries()[0].matches_type("TypeA"));

        let star: ComponentSet<&str> =
            ComponentSet::from_entries([TypedComponent::typed_selectors([Selector::Star], "All")]);
        assert_eq!(star.names_for_opt(None), None);
        assert_eq!(star.names_for(Some("TypeA")), PositionalNames::default());
    }

    #[test]
    fn components_keep_declaration_order() {
        let names: Vec<&str> = set().components().copied().collect();
        assert_eq!(names, vec!["AB", "C", "Default"]);
    }
}
