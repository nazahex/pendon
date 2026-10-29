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

use pendon_extra::{FieldConfig, ModifierConfig, Specs};
use pendon_types::Selector;
use serde::{Deserialize, Serialize};

use crate::imports::{ComponentTemplate, ImportEntry};

/// One entry of a §11 component set: the `type` selectors it answers, the
/// `field.*` config it carries (the one read side of §6.1 specs naming, D-5)
/// and the component it renders with.
///
/// An empty `selectors` list marks the **layer default** (§11 rule 2). A
/// wildcard (`?`, `_`, `*`) is still a *rule*, never the default slot.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TypedComponent<C> {
    #[serde(default)]
    pub selectors: Vec<Selector>,
    pub component: C,
    /// §6.1 / D-5: the entry's `field.*` keys. It is the *only* source of
    /// specs naming besides the built-in `slug` / `title`, and an entry that
    /// configures nothing keeps every built-in
    /// ([`Specs::resolve`]).
    ///
    /// Not serialized: the renderer contract is `(selectors, component)`, and
    /// `FieldConfig` is configuration, not part of a rendered template.
    #[serde(skip)]
    pub field: FieldConfig,
    /// §5.4 / D-9: the entry's `bracket.modifier."<symbol>"` tables — what a
    /// bracket's modifier run contributes to the element (ADR-0017).
    ///
    /// Not serialized for the same reason as `field`: it configures the node's
    /// fields, not its template.
    #[serde(skip)]
    pub modifier: ModifierConfig,
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
            modifier: ModifierConfig::new(),
        }
    }

    /// The layer default: it answers every instance no typed entry claims.
    pub fn default_component(component: C) -> Self {
        Self {
            selectors: Vec::new(),
            component,
            field: FieldConfig::new(),
            modifier: ModifierConfig::new(),
        }
    }

    /// Attaches the §6.1 `field.*` config the entry carries (D-5).
    pub fn with_field(mut self, field: FieldConfig) -> Self {
        self.field = field;
        self
    }

    /// Attaches the §5.4 `bracket.modifier."…"` config the entry carries (D-9).
    pub fn with_modifier(mut self, modifier: ModifierConfig) -> Self {
        self.modifier = modifier;
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
    /// only have one node at hand (`specs_for`, the CLI's duplicate check).
    pub fn matches_type(&self, type_marker: &str) -> bool {
        self.selectors
            .iter()
            .any(|selector| selector.literal() == Some(type_marker))
    }
}

/// §11 rule 2 / D-8: an entry that **configures** an element without naming a
/// component — `field.*` keys and/or `bracket.modifier."…"` tables only.
///
/// A modifier is a definition on its own (D-8), so an element does not need a
/// custom component to accept one: a config-only entry contributes to whatever
/// the layer already renders with, and to the built-in fallback when the layer
/// has no component at all. It never becomes a template, so
/// [`layer_templates`](crate::layer_templates) never sees it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConfigEntry {
    /// The `@@type{…}` markers this entry answers; empty marks the **layer
    /// default** (§11 rule 2), exactly like [`TypedComponent::selectors`].
    pub selectors: Vec<Selector>,
    /// §6.1 / D-5: the entry's `field.*` keys.
    pub field: FieldConfig,
    /// §5.4 / D-9: the entry's `bracket.modifier."<symbol>"` tables.
    pub modifier: ModifierConfig,
}

impl ConfigEntry {
    /// A config answering exactly the listed `@@type{…}` markers.
    pub fn typed<I>(selectors: I) -> Self
    where
        I: IntoIterator<Item = Selector>,
    {
        Self {
            selectors: selectors.into_iter().collect(),
            ..Self::default()
        }
    }

    /// A config answering every instance no typed entry claims.
    pub fn default_config() -> Self {
        Self::default()
    }

    /// Attaches the §6.1 `field.*` config (D-5).
    pub fn with_field(mut self, field: FieldConfig) -> Self {
        self.field = field;
        self
    }

    /// Attaches the §5.4 `bracket.modifier."…"` config (D-9).
    pub fn with_modifier(mut self, modifier: ModifierConfig) -> Self {
        self.modifier = modifier;
        self
    }

    /// `true` for the single default config of a layer.
    pub fn is_default(&self) -> bool {
        self.selectors.is_empty()
    }

    /// Answers `type_marker` through a **literal** selector (§11 rule 2).
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
///
/// The set carries two kinds of entry: components ([`TypedComponent`]) and
/// **config-only** entries ([`ConfigEntry`], D-8) that configure fields without
/// naming a component. Both answer the same §11 rule 3 lookups, so an element
/// accepts a modifier whether or not it renders a custom component.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ComponentSet<C> {
    #[serde(default)]
    entries: Vec<TypedComponent<C>>,
    /// Config-only entries (D-8). Not serialized for the same reason
    /// [`TypedComponent::field`] is: config is not part of a rendered template.
    #[serde(skip)]
    configs: Vec<ConfigEntry>,
}

/// What [`ComponentSet::field_for`] / [`ComponentSet::modifier_for`] resolved for
/// one `@@type{…}` marker: the entry the element renders with, plus the first
/// level that actually configures a field / a modifier (D-8 transparency).
struct ConfigMatch<'a> {
    /// The matched level's pair — the same entry for both configs, so an element
    /// never mixes a `field.*` from one entry with a modifier of another.
    matched: (&'a FieldConfig, &'a ModifierConfig),
    /// The first level whose `field.*` assigns something.
    field: Option<&'a FieldConfig>,
    /// The first level whose `bracket.modifier."…"` defines a symbol.
    modifier: Option<&'a ModifierConfig>,
}

impl<C> ComponentSet<C> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            configs: Vec::new(),
        }
    }

    pub fn from_entries(entries: impl IntoIterator<Item = TypedComponent<C>>) -> Self {
        Self {
            entries: entries.into_iter().collect(),
            configs: Vec::new(),
        }
    }

    /// Appends one entry, optionally typed (literal marker names).
    pub fn push(&mut self, types: Vec<String>, component: C) {
        self.entries.push(TypedComponent::typed(types, component));
    }

    /// Appends a config-only entry (D-8): the fields and modifier
    /// contributions an element accepts without naming a component.
    pub fn push_config(&mut self, config: ConfigEntry) {
        self.configs.push(config);
    }

    /// Attaches config-only entries (D-8), in declaration order.
    pub fn with_configs(mut self, configs: impl IntoIterator<Item = ConfigEntry>) -> Self {
        self.configs = configs.into_iter().collect();
        self
    }

    /// The config-only entries of the layer, in declaration order (D-8).
    pub fn configs(&self) -> &[ConfigEntry] {
        &self.configs
    }

    /// `true` when no entry names a component.
    ///
    /// A config-only entry (D-8) does not make this `false`: it renders nothing,
    /// so the hint path stays off while its fields still resolve.
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

    /// The layer-default config, when declared (D-8).
    pub fn default_config(&self) -> Option<&ConfigEntry> {
        self.configs.iter().find(|config| config.is_default())
    }

    /// The layer default, when declared.
    pub fn default_component(&self) -> Option<&C> {
        self.default_entry().map(|entry| &entry.component)
    }

    /// §6.1 / D-5: the specs of the entry that answers `type_marker`.
    /// An exact `type` match wins, then the layer default, then the built-in
    /// `slug` / `title`. The construct plugins pass the result to
    /// `pendon_extra::to_attributes` when they map an extras head onto attributes.
    pub fn specs_for(&self, type_marker: Option<&str>) -> Specs {
        self.specs_for_opt(type_marker).unwrap_or_default()
    }

    /// §6.1 / D-5: the `field.*` keys of the entry that answers `type_marker` —
    /// the **matched** entry, which is the one D-7's preset voiding reads.
    ///
    /// Resolution is §11 rule 3 over both kinds of entry: an exact `type`
    /// component, then an exact-`type` config-only entry (D-8), then the layer
    /// default (component before config). An entry that assigns no field is
    /// transparent — the next level applies — and the matched entry is still
    /// reported when every level is empty, so a caller can tell "no entry"
    /// (`None`: `ElementHead::assign` keeps every preset row alive) from "an
    /// entry that assigns nothing".
    pub fn field_for(&self, type_marker: Option<&str>) -> Option<&FieldConfig> {
        let matched = self.config_match(type_marker)?;
        Some(matched.field.unwrap_or(matched.matched.0))
    }

    /// §5.4 / D-9: the `bracket.modifier."<symbol>"` tables of the entry that
    /// answers `type_marker`.
    ///
    /// Same resolution and transparency as [`field_for`](Self::field_for), so a
    /// run contributes through the entry the element actually renders with — and
    /// an entry that defines no symbol leaves the layer default's symbols alive.
    pub fn modifier_for(&self, type_marker: Option<&str>) -> Option<&ModifierConfig> {
        let matched = self.config_match(type_marker)?;
        Some(matched.modifier.unwrap_or(matched.matched.1))
    }

    /// Like [`specs_for`](Self::specs_for) but `None` when this set has neither an
    /// exact-`type` entry nor a layer default — so a plugin that owns several
    /// layers (`plugin-list`) can try each in turn before falling back.
    pub fn specs_for_opt(&self, type_marker: Option<&str>) -> Option<Specs> {
        self.field_for(type_marker)
            .map(|field| Specs::resolve(Some(field)))
    }

    /// The `field.*` / `bracket.modifier."…"` pair that answers `type_marker`,
    /// resolved as §11 rule 3 over both kinds of entry: exact-`type` components,
    /// then exact-`type` config-only entries (D-8), then the layer default
    /// (component before config).
    ///
    /// `matched` is the entry the element renders with (the one D-7's preset
    /// voiding and §13's diagnostics read); `field` / `modifier` are the first
    /// levels that actually configure something, so an entry that configures
    /// nothing is transparent and the next level applies.
    fn config_match(&self, type_marker: Option<&str>) -> Option<ConfigMatch<'_>> {
        let answers = move |selectors: &[Selector]| {
            type_marker.is_some_and(|marker| {
                selectors
                    .iter()
                    .any(|selector| selector.literal() == Some(marker))
            })
        };
        let mut matched = None;
        let mut field = None;
        let mut modifier = None;
        for (candidate_field, candidate_modifier) in self
            .entries
            .iter()
            .filter(|entry| answers(&entry.selectors))
            .map(|entry| (&entry.field, &entry.modifier))
            .chain(
                self.configs
                    .iter()
                    .filter(|config| answers(&config.selectors))
                    .map(|config| (&config.field, &config.modifier)),
            )
            .chain(
                self.default_entry()
                    .into_iter()
                    .map(|entry| (&entry.field, &entry.modifier)),
            )
            .chain(
                self.default_config()
                    .into_iter()
                    .map(|config| (&config.field, &config.modifier)),
            )
        {
            matched.get_or_insert((candidate_field, candidate_modifier));
            if field.is_none() && !candidate_field.is_empty() {
                field = Some(candidate_field);
            }
            if modifier.is_none() && !candidate_modifier.is_empty() {
                modifier = Some(candidate_modifier);
            }
        }
        Some(ConfigMatch {
            matched: matched?,
            field,
            modifier,
        })
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
                .with_field(entry.field.clone())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pendon_extra::{AttrValue, Path, Slot, SlotKind};

    fn set() -> ComponentSet<&'static str> {
        ComponentSet::from_entries([
            TypedComponent::typed(["a", "b"], "AB"),
            TypedComponent::typed(["c"], "C"),
            TypedComponent::default_component("Default"),
        ])
    }

    /// §11 rule 3 / decision D-3a: specs naming is literal-only. A `_` or
    /// `*` entry has no single component to read a name from, so it contributes
    /// nothing — a wildcard never silently overrides `slug`/`title`.
    #[test]
    fn specs_for_ignores_wildcard_entries() {
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
        assert_eq!(wildcard.specs_for_opt(Some("TypeA")), None);
        assert_eq!(wildcard.specs_for(Some("TypeA")), Specs::default());
        assert!(!wildcard.entries()[0].matches_type("TypeA"));

        let star: ComponentSet<&str> =
            ComponentSet::from_entries([TypedComponent::typed_selectors([Selector::Star], "All")]);
        assert_eq!(star.specs_for_opt(None), None);
        assert_eq!(star.specs_for(Some("TypeA")), Specs::default());
    }

    #[test]
    fn components_keep_declaration_order() {
        let names: Vec<&str> = set().components().copied().collect();
        assert_eq!(names, vec!["AB", "C", "Default"]);
    }

    fn path(dotted: &str) -> Path {
        Path::from_dotted(dotted).expect("valid path")
    }

    fn text(value: &str) -> AttrValue {
        AttrValue::Str(value.to_string())
    }

    fn modifier_config(symbol: char, dotted: &str, value: &str) -> ModifierConfig {
        let mut config = ModifierConfig::new();
        config.define(symbol, vec![(path(dotted), text(value))]);
        config
    }

    /// §5.4 / D-9: a modifier is a definition on its own (D-8), so an entry that
    /// names no component still configures the element — the layer renders the
    /// built-in fallback and the run reaches it.
    #[test]
    fn a_config_only_entry_configures_the_element_it_renders() {
        let set: ComponentSet<&str> = ComponentSet::new()
            .with_configs([ConfigEntry::default_config()
                .with_modifier(modifier_config('?', "rel", "nofollow"))]);
        assert!(set.is_empty(), "a config-only entry emits no template");
        for marker in [None, Some("TypeA")] {
            let modifier = set.modifier_for(marker).expect("the default config");
            assert_eq!(modifier.for_symbol('?').len(), 1);
            assert!(modifier.for_symbol('!').is_empty());
        }
    }

    /// §11 rule 3: an exact `type` config-only entry wins over the layer default,
    /// and the two configs of an element always come from the same level.
    #[test]
    fn an_exact_type_config_only_entry_beats_the_layer_default() {
        let set: ComponentSet<&str> = ComponentSet::new().with_configs([
            ConfigEntry::typed([Selector::Exact("A".to_string())])
                .with_modifier(modifier_config('?', "rel", "nofollow")),
            ConfigEntry::default_config().with_modifier(modifier_config('?', "rel", "noopener")),
        ]);
        let typed = set.modifier_for(Some("A")).expect("the typed config");
        let default = set.modifier_for(Some("B")).expect("the default config");
        assert_eq!(typed.for_symbol('?')[0].1, text("nofollow"));
        assert_eq!(default.for_symbol('?')[0].1, text("noopener"));
    }

    /// D-8: an entry that configures nothing is transparent, so the run still
    /// reaches the level that does — a typed component never silences the layer
    /// default's symbols.
    #[test]
    fn an_empty_config_is_transparent() {
        let set: ComponentSet<&str> =
            ComponentSet::from_entries([TypedComponent::typed(["A"], "A")])
                .with_configs([ConfigEntry::default_config()
                    .with_modifier(modifier_config('?', "rel", "nofollow"))]);
        let modifier = set.modifier_for(Some("A")).expect("a matched entry");
        assert_eq!(modifier.for_symbol('?').len(), 1);
        assert_eq!(
            set.modifier_for(None).map(|m| m.for_symbol('?').len()),
            Some(1),
            "a marker-less head is answered by the layer default"
        );
    }

    /// A config-only entry is configuration, never a component: it adds no
    /// template, so the renderer keeps falling back to the built-in element.
    #[test]
    fn a_config_only_entry_emits_no_template() {
        let set: ComponentSet<&str> =
            ComponentSet::from_entries([TypedComponent::default_component("Default")])
                .with_configs([ConfigEntry::default_config()
                    .with_modifier(modifier_config('?', "rel", "nofollow"))]);
        let templates = layer_templates("anchor", &set, |component| {
            (format!("<{component}/>"), Vec::new())
        });
        assert_eq!(templates.len(), 1);
        assert_eq!(set.configs().len(), 1);
        assert!(set.default_config().is_some());
    }
}
