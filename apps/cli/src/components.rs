//! §11 component-set config: `[task.<plugin>.custom.<layer>]`.
//!
//! One task-level plugin owns one **layer** per element it emits (`table`,
//! `thead`, `caption`, or `img`/`figure`, `list`/`unordered`/`ordered`, …). Each
//! layer has a component set: typed entries plus at most one default:
//!
//! ```toml
//! [task.table.custom.table]                  # single table = the layer default
//! name = "TableTpl"
//! template = "<TableTpl {...attrs}>{children}</TableTpl>"
//!
//! [[task.table.custom.thead]]                # array of tables = typed + default
//! type = ["theadA", "theadB"]
//! name = "HeadAB"
//!
//! [task.anchor.custom]                       # single-layer plugin: `custom` is
//! name = "AnchorDefault"                     # the primary layer itself
//! ```
//!
//! Both forms are equivalent to "the component set of one layer" (rule 1),
//! `type` accepts a string or an array (rule 2) and at most one default per
//! layer is a hard error.

use std::collections::BTreeMap;
use std::fmt;

use pendon_renderer_solid::ImportEntry;
use pendon_types::Selector;

use pendon_number::{NUMBER_DEPTH_ATTR, NUMBER_RAW_ATTR, NUMBER_TYPE_ATTR};

use pendon_extra::{AttrValue, ModifierConfig, Path, Slot, SlotKind, SPECS_KEYS};

use crate::plugins::parse_import_entries;

/// A fatal configuration problem: §11 rules 2 and 6 and any unknown key are hard
/// errors at load (D-16).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError(pub String);

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ConfigError {}

/// §7 field vocabulary — the `field.*` keys of one entry (D-5, ADR-0014).
///
/// Defined in `pendon-extra` with the name rules and the [`pendon_extra::Path`]
/// type, so this module only maps TOML onto it. Positional naming reads it too
/// ([`pendon_extra::Specs::resolve`]): since §14 there is no separate
/// naming key, so `field.*` is the only way to rename a slot or a specs position.
pub use pendon_extra::FieldConfig;

/// One `custom` entry: the `type` selectors it answers, its imports and its
/// template.
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentEntry {
    /// §11 selectors: literals plus `?` / `_` / `*`. Empty means "this is the
    /// layer default" (rule 2) — an explicit wildcard is a rule, not the
    /// default slot.
    pub selectors: Vec<Selector>,
    pub name: Option<String>,
    pub imports: Vec<ImportEntry>,
    pub template: Option<String>,
    /// §7 `field.*` — the entry's public-field assignment. Empty for an entry
    /// that configures no fields, which is what keeps the element's preset
    /// assignments alive (D-7).
    pub field: FieldConfig,
    /// §5.4 / D-9 `bracket.modifier."<symbol>"` — what a bracket's modifier run
    /// contributes to the element. Empty by default: Pendon ships no modifier.
    pub modifier: ModifierConfig,
}

impl ComponentEntry {
    /// `true` for the single default entry of a layer.
    pub fn is_default(&self) -> bool {
        self.selectors.is_empty()
    }

    /// True when the entry names a component (`name` / `template` / `imports`).
    ///
    /// This is what reaches the renderer. A **fields-only** entry (D-8) is
    /// `false` here even though it is a perfectly good entry: it configures
    /// public field names instead of a component.
    pub fn is_configured(&self) -> bool {
        self.name.is_some() || self.template.is_some() || !self.imports.is_empty()
    }

    /// True when the entry carries configuration that reaches no template:
    /// `field.*` or a `bracket.modifier."…"` table.
    ///
    /// Two configs for one literal would silently drop one, so they clash — the
    /// same rule two components for one literal follow (D-4, D-8).
    pub fn carries_config(&self) -> bool {
        !self.field.is_empty() || !self.modifier.is_empty()
    }

    /// Answers `type_marker` through a **literal** selector, or `None` when it
    /// is not this entry's type.
    ///
    /// Wildcards are resolved per document by the renderer (decision D-3a), so
    /// they never match here — `specs_for` and the duplicate check are
    /// literal-only by design (see `pendon-types`).
    pub fn matches_type(&self, type_marker: &str) -> bool {
        self.selectors
            .iter()
            .any(|selector| selector.literal() == Some(type_marker))
    }

    /// The literal marker names this entry answers (wildcards excluded).
    pub fn literal_types(&self) -> impl Iterator<Item = &str> {
        self.selectors.iter().filter_map(Selector::literal)
    }
}

/// The component set of one layer (§11 rule 1).
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentSet {
    layer: String,
    typed: Vec<ComponentEntry>,
    default: Option<ComponentEntry>,
}

impl ComponentSet {
    pub fn new(layer: impl Into<String>) -> Self {
        Self {
            layer: layer.into(),
            typed: Vec::new(),
            default: None,
        }
    }

    pub fn typed(&self) -> &[ComponentEntry] {
        &self.typed
    }

    pub fn default_entry(&self) -> Option<&ComponentEntry> {
        self.default.as_ref()
    }

    #[allow(dead_code)] // §11 rule-3 API: the Phase-1 binder skips empty sets.
    pub fn is_empty(&self) -> bool {
        self.typed.is_empty() && self.default.is_none()
    }

    /// Adds an entry, rejecting a second default (hard error).
    pub fn insert(&mut self, entry: ComponentEntry) -> Result<(), ConfigError> {
        if entry.is_default() {
            if self.default.is_some() {
                return Err(ConfigError(format!(
                    "layer `{}` declares two default components; at most one is allowed",
                    self.layer
                )));
            }
            self.default = Some(entry);
            return Ok(());
        }
        self.typed.push(entry);
        Ok(())
    }

    /// The field config a node of `type_marker` uses (D-8 merge order): a typed
    /// entry for that marker beats the layer default, and an entry with no
    /// `field.*` keys is transparent — the next level applies.
    ///
    /// `None` means "configured nothing", which keeps the element's preset
    /// assignments alive (D-7).
    #[allow(dead_code)] // D-7 read side: wired per element in `S2-2.6`.
    pub fn field_config(&self, type_marker: Option<&str>) -> Option<&FieldConfig> {
        if let Some(marker) = type_marker {
            if let Some(field) = self
                .typed
                .iter()
                .filter(|entry| entry.matches_type(marker))
                .map(|entry| &entry.field)
                .find(|field| !field.is_empty())
            {
                return Some(field);
            }
        }
        self.default
            .as_ref()
            .map(|entry| &entry.field)
            .filter(|field| !field.is_empty())
    }
}

/// Every layer's component set for one task-level plugin.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PluginComponents {
    layers: BTreeMap<String, ComponentSet>,
}

impl PluginComponents {
    pub fn layers(&self) -> &BTreeMap<String, ComponentSet> {
        &self.layers
    }

    pub fn layer(&self, layer: &str) -> Option<&ComponentSet> {
        self.layers.get(layer)
    }

    /// The field config for one layer and type marker (see
    /// [`ComponentSet::field_config`]); `None` for an unknown layer.
    #[allow(dead_code)] // D-7 read side: wired per element in `S2-2.6`.
    pub fn field_config(&self, layer: &str, type_marker: Option<&str>) -> Option<&FieldConfig> {
        self.layer(layer)
            .and_then(|set| set.field_config(type_marker))
    }

    fn entry(&mut self, layer: &str) -> &mut ComponentSet {
        self.layers
            .entry(layer.to_string())
            .or_insert_with(|| ComponentSet::new(layer))
    }
}

/// Rejects `custom` layers the plugin cannot carry yet.
///
/// Layer names are the elements a plugin emits (§11 rule 1), so anything
/// outside `wired` is either a typo or a layer whose cutover (§14) has not
/// landed. Both are hard errors: an unread layer would silently do nothing.
pub fn reject_unwired_layers(
    plugin: &str,
    components: &PluginComponents,
    wired: &[&str],
) -> Result<(), ConfigError> {
    for layer in components.layers().keys() {
        if wired.contains(&layer.as_str()) {
            continue;
        }
        return Err(ConfigError(format!(
            "task.{plugin}.custom.{layer}: unsupported layer for `{plugin}`; supported \
             layers are {}",
            wired.join(", ")
        )));
    }
    Ok(())
}

/// Result of loading one plugin's component config: the sets each layer got.
///
/// There is no warnings list: a config key this version does not know is a load
/// error, never a notice (D-16, ADR-0018).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LoadedComponents {
    pub components: PluginComponents,
}

/// Keys that mark a table as a component entry rather than a map of layers.
///
/// `field` belongs here: `[task.anchor.custom]` with only `field.*` keys is a
/// (field-only) **entry**, never a layer named `field` (D-8). `bracket` is here
/// for the same reason: it carries the §5.4 modifier tables and is not a layer.
///
/// Every other key is a load error (D-16): an entry is a closed vocabulary, so a
/// typo — or a removed key (§14) — can never be silently ignored.
const ENTRY_KEYS: [&str; 6] = ["type", "name", "imports", "template", "field", "bracket"];

/// Loads the `custom` key of one task-level plugin.
///
/// * `plugin` — the task key, used in error messages only.
/// * `primary_layer` — the layer a bare `custom = [...]` array or a
///   `custom = { … }` entry table belongs to (the plugin's main element:
///   `anchor`, `img`, `table`, `list`, …).
/// * `custom` — the raw `task.<plugin>.custom` value, if present.
pub fn load(
    plugin: &str,
    primary_layer: &str,
    custom: Option<&toml::Value>,
) -> Result<LoadedComponents, ConfigError> {
    let mut out = LoadedComponents::default();
    let Some(custom) = custom else {
        return Ok(out);
    };

    match custom {
        toml::Value::Array(_) => {
            let set = parse_set(plugin, primary_layer, custom)?;
            merge_set(&mut out.components, set)?;
        }
        toml::Value::Table(table) if is_layer_map(table) => {
            for (layer, value) in table {
                let set = parse_set(plugin, layer, value)?;
                merge_set(&mut out.components, set)?;
            }
        }
        // D-8: `custom` is **always** an array of entries. A single table is
        // ambiguous — it reads as one entry *and* as a map of layers — so it is
        // a load error, never a guess (D-16).
        toml::Value::Table(_) => {
            return Err(ConfigError(format!(
                "task.{plugin}.custom must be an array of tables \
                 (`[[task.{plugin}.custom]]`); a single `[task.{plugin}.custom]` table is \
                 ambiguous. Use `[[task.{plugin}.custom]]` for an entry, or \
                 `[task.{plugin}.custom.<layer>]` for a layer"
            )))
        }
        other => {
            return Err(ConfigError(format!(
                "task.{plugin}.custom must be a table or an array of tables, found {}",
                other.type_str()
            )))
        }
    }

    Ok(out)
}

/// A table is a *layer map* when none of its keys is a component-entry key
/// (`[task.table.custom.thead]`), and a *single entry* otherwise
/// (`[task.anchor.custom]` with `name`/`template`).
fn is_layer_map(table: &toml::Table) -> bool {
    !table.is_empty() && !table.keys().any(|key| ENTRY_KEYS.contains(&key.as_str()))
}

fn merge_set(components: &mut PluginComponents, set: ComponentSet) -> Result<(), ConfigError> {
    let layer = set.layer.clone();
    for entry in set.typed {
        components.entry(&layer).insert(entry)?;
    }
    if let Some(default) = set.default {
        components.entry(&layer).insert(default)?;
    }
    Ok(())
}

fn parse_set(plugin: &str, layer: &str, value: &toml::Value) -> Result<ComponentSet, ConfigError> {
    let mut set = ComponentSet::new(layer);
    let label = format!("task.{plugin}.custom.{layer}");

    match value {
        toml::Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                let table = item.as_table().ok_or_else(|| {
                    ConfigError(format!("{label}: entry #{} must be a table", index + 1))
                })?;
                let entry = parse_entry(&format!("{label}[{}]", index + 1), table)?;
                check_duplicate_type(&set, &entry, &label)?;
                set.insert(entry)?;
            }
        }
        toml::Value::Table(table) => {
            let entry = parse_entry(&label, table)?;
            set.insert(entry)?;
        }
        other => {
            return Err(ConfigError(format!(
                "{label} must be a table or an array of tables, found {}",
                other.type_str()
            )))
        }
    }

    Ok(set)
}

/// Decision D-4: an exact literal may appear once per layer, wildcards may
/// repeat and are ordered (later wins). Two entries with no `type` at all are
/// still the "two defaults" hard error, handled by [`ComponentSet::insert`].
///
/// D-8 refines "once": a **fields-only** entry is not a component, so it may
/// share a literal with the component entry of the same type — that is how
/// `[[custom]] type = "A"` + `field.*` merges with `[[custom]] type = "A"` +
/// `name`. Two components (or two field configs) for one literal still clash.
fn check_duplicate_type(
    set: &ComponentSet,
    entry: &ComponentEntry,
    label: &str,
) -> Result<(), ConfigError> {
    for known in entry.literal_types() {
        for other in set.typed().iter().filter(|other| other.matches_type(known)) {
            let both_components = entry.is_configured() && other.is_configured();
            let both_configs = entry.carries_config() && other.carries_config();
            if both_components || both_configs {
                return Err(ConfigError(format!(
                    "{label}: duplicate `type` marker `{known}`"
                )));
            }
        }
    }
    Ok(())
}

fn parse_entry(label: &str, table: &toml::Table) -> Result<ComponentEntry, ConfigError> {
    // D-16: the entry vocabulary is closed. A key this version does not know is a
    // load error, not a warning — a typo or a removed key (§14) must never be
    // dropped silently and leave the output looking configured.
    for key in table.keys() {
        if ENTRY_KEYS.contains(&key.as_str()) {
            continue;
        }
        // D-16: a removed key is an error that names its replacement, never an
        // alias (ADR-0018).
        if key == "import" {
            return Err(ConfigError(format!(
                "{label}: `import` was removed; use `imports = [\"…\"]` (§11 rule 4, §14)"
            )));
        }
        // §5.4 / D-9 rule 5: a modifier run exists in a bracket only, so a
        // `paren.modifier` table is not syntax (and not a typo either).
        if Slot::from_name(key).is_some() {
            return Err(ConfigError(format!(
                "{label}: `{key}.modifier` is not syntax; modifiers are bracket-only (§5.4, D-9)"
            )));
        }
        return Err(ConfigError(format!(
            "{label}: unknown key `{key}`; a custom entry takes `type`, `name`, \
             `imports`, `template`, `field.*` and `bracket.modifier.\"<symbol>\"` keys (§11, D-16)"
        )));
    }

    let tokens: Vec<String> = match table.get("type") {
        None => Vec::new(),
        Some(toml::Value::String(marker)) => vec![marker.clone()],
        Some(toml::Value::Array(items)) => items
            .iter()
            .map(|item| {
                item.as_str().map(str::to_string).ok_or_else(|| {
                    ConfigError(format!("{label}: every `type` entry must be a string"))
                })
            })
            .collect::<Result<Vec<String>, ConfigError>>()?,
        Some(other) => {
            return Err(ConfigError(format!(
                "{label}: `type` must be a string or an array of strings, found {}",
                other.type_str()
            )))
        }
    };
    // `[]` is equivalent to an absent `type` (§11 rule 2); every other entry
    // must be a selector, so a typo is a load-time error, never a wildcard.
    let selectors = Selector::parse_all(&tokens)
        .map_err(|error| ConfigError(format!("{label}: `type` entry {error}")))?;

    let name = optional_string(table, "name", label)?;
    let template = optional_string(table, "template", label)?;
    if let Some(template) = &template {
        validate_template(label, template)?;
    }

    let imports = match table.get("imports") {
        Some(value) => parse_imports(label, value)?,
        None => Vec::new(),
    };

    let field = match table.get("field") {
        Some(value) => parse_field_config(label, value)?,
        None => FieldConfig::new(),
    };
    let modifier = match table.get("bracket") {
        Some(value) => parse_modifier_config(label, value)?,
        None => ModifierConfig::new(),
    };
    let entry = ComponentEntry {
        selectors,
        name,
        imports,
        template,
        field,
        modifier,
    };
    // D-8: an entry that carries neither a component nor a `field.*` / modifier
    // config says nothing, so it is a load error rather than a silent no-op.
    if !entry.is_configured() && entry.field.is_empty() && entry.modifier.is_empty() {
        return Err(ConfigError(format!(
            "{label}: empty custom entry; give it a component (`name`, `template` or \
             `imports`), a `field.*` key or a `bracket.modifier.\"<symbol>\"` table"
        )));
    }
    Ok(entry)
}

/// Parses one entry's `field.*` keys (D-5, ADR-0014).
///
/// TOML already turns a dotted key into nested tables, so the three families
/// arrive as `field.<slot>.<kind>`, `field.specs.<k>` and
/// `field.override.<path…>`. Every other shape is a load error naming the three
/// families — no key is ignored (D-16), and every name goes through
/// `pendon_extra`'s name rules. A value of `""` unassigns the position
/// (OBJ-10).
fn parse_field_config(label: &str, value: &toml::Value) -> Result<FieldConfig, ConfigError> {
    let table = value.as_table().ok_or_else(|| {
        ConfigError(format!(
            "{label}: `field` must be a table of dotted keys, found {}",
            value.type_str()
        ))
    })?;

    let mut config = FieldConfig::new();
    for (key, value) in table {
        if key == "specs" {
            for (spec, target) in field_sub_table(label, "specs", value)? {
                let target = field_name(label, &format!("field.specs.{spec}"), target)?;
                config.assign_specs(spec, target).map_err(|error| {
                    ConfigError(format!("{label}: field.specs.{spec}: {error}"))
                })?;
            }
            continue;
        }
        if key == "override" {
            if value.as_table().is_none() {
                return Err(ConfigError(format!(
                    "{label}: `field.override` must be a table of dotted paths, found {}",
                    value.type_str()
                )));
            }
            let mut leaves = Vec::new();
            flatten_override_paths(Vec::new(), value, &mut leaves);
            for (segments, target) in leaves {
                let names: Vec<&str> = segments.iter().map(String::as_str).collect();
                let source = Path::from_segments(&names).map_err(|error| {
                    ConfigError(format!(
                        "{label}: field.override.{}: {error}",
                        segments.join(".")
                    ))
                })?;
                let target = field_name(label, &format!("field.override.{source}"), target)?
                    .ok_or_else(|| {
                        ConfigError(format!(
                            "{label}: field.override.{source} needs a name; `\"\"` only unassigns \
                             a slot or specs position"
                        ))
                    })?;
                config
                    .add_override(source.clone(), target)
                    .map_err(|error| {
                        ConfigError(format!("{label}: field.override.{source}: {error}"))
                    })?;
            }
            continue;
        }
        let Some(slot) = Slot::from_name(key) else {
            return Err(ConfigError(format!(
                "{label}: `{key}` is not a field config key; `field` takes \
                 `field.bracket.<bare|quote|tick>`, `field.paren.<bare|quote|tick>`, \
                 `field.specs.<{}>` or `field.override.<path>`",
                SPECS_KEYS.join("|")
            )));
        };
        for (kind_name, target) in field_sub_table(label, key, value)? {
            let Some(kind) = SlotKind::from_name(kind_name) else {
                return Err(ConfigError(format!(
                    "{label}: `field.{key}.{kind_name}` is not a slot content; use \
                     bare, quote or tick"
                )));
            };
            let target = field_name(label, &format!("field.{key}.{kind_name}"), target)?;
            config.assign_slot(slot, kind, target);
        }
    }
    Ok(config)
}

/// Parses one numbering block's `field.*` keys — **renames only** (S2-9.3).
///
/// A `[[task.<element>.number]]` block and a `[[task.number.custom]]` block own a
/// counter, and a counter is the one thing that knows which elements it numbers:
/// a `%%` group is declared once for the whole task, so making every numbered
/// element's `custom` entry rename `number.raw` would mean writing the same
/// table on every element the group might reach. So the block declares the
/// public names instead, and the pass applies them as it writes (ADR-0016
/// rule 2).
///
/// Only `field.override.<number.type|raw|depth>` is legal — a numbering block
/// publishes those three fields and nothing else, so a `field.specs`, a
/// `field.bracket` or a path outside the namespace has no meaning here and is a
/// load error rather than an ignored key (D-16). Two renames onto one name, and
/// a rename onto a name the same block still publishes, are errors too: both
/// would leave the document depending on attribute order.
pub(crate) fn number_field_overrides(
    label: &str,
    value: &toml::Value,
) -> Result<BTreeMap<String, String>, ConfigError> {
    let table = value.as_table().ok_or_else(|| {
        ConfigError(format!(
            "{label}: `field` must be a table of dotted keys, found {}",
            value.type_str()
        ))
    })?;

    let mut overrides: BTreeMap<String, String> = BTreeMap::new();
    for (key, value) in table {
        if key != "override" {
            return Err(ConfigError(format!(
                "{label}: `field.{key}` is not a numbering field config key; a numbering \
                 block publishes three fields, so it takes only {}",
                number_override_keys()
            )));
        }
        if value.as_table().is_none() {
            return Err(ConfigError(format!(
                "{label}: `field.override` must be a table of dotted paths, found {}",
                value.type_str()
            )));
        }
        let mut leaves = Vec::new();
        flatten_override_paths(Vec::new(), value, &mut leaves);
        for (segments, target) in leaves {
            let names: Vec<&str> = segments.iter().map(String::as_str).collect();
            let source = Path::from_segments(&names).map_err(|error| {
                ConfigError(format!(
                    "{label}: field.override.{}: {error}",
                    segments.join(".")
                ))
            })?;
            let source = source.to_string();
            if !NUMBER_FIELDS.contains(&source.as_str()) {
                return Err(ConfigError(format!(
                    "{label}: field.override.{source} is not a numbering field; {}",
                    number_override_keys()
                )));
            }
            let target = field_name(label, &format!("field.override.{source}"), target)?
                .ok_or_else(|| {
                    ConfigError(format!(
                        "{label}: field.override.{source} needs a name; `\"\"` only unassigns a \
                         slot or specs position"
                    ))
                })?;
            let target = target.to_string();
            if NUMBER_FIELDS.contains(&target.as_str()) {
                return Err(ConfigError(format!(
                    "{label}: field.override.{source} = \"{target}\" renames onto a field this \
                     block publishes; pick a name outside {}",
                    NUMBER_FIELDS.join(" / ")
                )));
            }
            if let Some(other) = overrides
                .iter()
                .find(|(other, existing)| existing.as_str() == target && other.as_str() != source)
                .map(|(other, _)| other.clone())
            {
                return Err(ConfigError(format!(
                    "{label}: field.override.{other} and field.override.{source} both write \
                     `{target}`; a rename target is unique"
                )));
            }
            overrides.insert(source, target);
        }
    }
    Ok(overrides)
}

/// The three fields a numbering block publishes (§0 of `FIELDS-INVENTORY.md`).
const NUMBER_FIELDS: [&str; 3] = [NUMBER_TYPE_ATTR, NUMBER_RAW_ATTR, NUMBER_DEPTH_ATTR];

/// The one legal key path of a numbering block, spelled out for errors.
fn number_override_keys() -> String {
    format!(
        "`field.override.{NUMBER_TYPE_ATTR}`, `field.override.{NUMBER_RAW_ATTR}` or \
         `field.override.{NUMBER_DEPTH_ATTR}`"
    )
}
///
/// A symbol is **exactly one character**, quoted because a symbol is
/// punctuation: `"?"`, `"~"`, `"|"`, `"Œ"`, `"\\"`, `"ئ"` are all valid and all
/// mean only what this entry's config says. The symbol is never interpreted — a
/// run of symbols with no config is legal and contributes nothing.
///
/// `bracket` is the only slot key: a modifier run lives in a bracket, so
/// `paren.modifier` is rejected at the entry level ([`parse_entry`]).
fn parse_modifier_config(label: &str, value: &toml::Value) -> Result<ModifierConfig, ConfigError> {
    let table = value.as_table().ok_or_else(|| {
        ConfigError(format!(
            "{label}: `bracket` must be a table with a `modifier` key, found {}",
            value.type_str()
        ))
    })?;
    let mut config = ModifierConfig::new();
    for (key, value) in table {
        if key != "modifier" {
            return Err(ConfigError(format!(
                "{label}: `bracket.{key}` is not a modifier config key; use \
                 `bracket.modifier.\"<symbol>\"` (D-5)"
            )));
        }
        let symbols = value.as_table().ok_or_else(|| {
            ConfigError(format!(
                "{label}: `bracket.modifier` must be a table of quoted symbols, found {}",
                value.type_str()
            ))
        })?;
        for (symbol, contribution) in symbols {
            let mut chars = symbol.chars();
            let symbol = match (chars.next(), chars.next()) {
                (Some(symbol), None) => symbol,
                _ => {
                    return Err(ConfigError(format!(
                        "{label}: `bracket.modifier.\"{symbol}\"` is not a symbol; a modifier \
                         symbol is exactly one character"
                    )))
                }
            };
            config.define(symbol, modifier_fields(label, symbol, contribution)?);
        }
    }
    Ok(config)
}

/// One symbol's contribution: public field paths to values.
///
/// A dotted key (`"soo.baz" = "wax"`) and the nested spelling (`[bracket.modifier."?".soo]
/// baz = "wax"`) are the same path — a table is a **path segment**, never an
/// object value, because the field model is flat (ADR-0014). The same
/// flattening `field.override` uses.
fn modifier_fields(
    label: &str,
    symbol: char,
    contribution: &toml::Value,
) -> Result<Vec<(Path, AttrValue)>, ConfigError> {
    if contribution.as_table().is_none() {
        return Err(ConfigError(format!(
            "{label}: `bracket.modifier.\"{symbol}\"` must be a table of fields, found {}",
            contribution.type_str()
        )));
    }
    let mut leaves = Vec::new();
    flatten_override_paths(Vec::new(), contribution, &mut leaves);
    let mut fields = Vec::with_capacity(leaves.len());
    for (segments, value) in leaves {
        let names: Vec<&str> = segments.iter().map(String::as_str).collect();
        let path = Path::from_segments(&names).map_err(|error| {
            ConfigError(format!(
                "{label}: bracket.modifier.\"{symbol}\".{}: {error}",
                segments.join(".")
            ))
        })?;
        fields.push((path, modifier_value(label, symbol, &segments, value)?));
    }
    Ok(fields)
}

/// A contribution's value as the typed value a head would produce (§6.3).
///
/// A table is a path segment ([`modifier_fields`]), so only a scalar or an
/// array of scalars reaches here; anything else — a date, or a table nested in
/// an array — is a load error rather than a silently dropped field.
fn modifier_value(
    label: &str,
    symbol: char,
    segments: &[String],
    value: &toml::Value,
) -> Result<AttrValue, ConfigError> {
    Ok(match value {
        toml::Value::String(text) => AttrValue::Str(text.clone()),
        toml::Value::Integer(number) => AttrValue::Int(*number),
        toml::Value::Float(number) => AttrValue::Float(*number),
        toml::Value::Boolean(flag) => AttrValue::Bool(*flag),
        toml::Value::Array(items) => AttrValue::Array(
            items
                .iter()
                .map(|item| modifier_value(label, symbol, segments, item))
                .collect::<Result<Vec<_>, _>>()?,
        ),
        other => {
            return Err(ConfigError(format!(
                "{label}: bracket.modifier.\"{symbol}\".{} must be a string, a number, a boolean \
                 or an array, found {}",
                segments.join("."),
                other.type_str()
            )))
        }
    })
}

/// One `field` family's sub-table (`field.specs`, `field.bracket`, …).
fn field_sub_table<'a>(
    label: &str,
    family: &str,
    value: &'a toml::Value,
) -> Result<&'a toml::Table, ConfigError> {
    value.as_table().ok_or_else(|| {
        ConfigError(format!(
            "{label}: `field.{family}` must be a table of dotted keys, found {}",
            value.type_str()
        ))
    })
}

/// A field name (`field.bracket.bare = "anchor_id"`), validated by the §7 rules.
///
/// `Ok(None)` is the OBJ-10 unassign spelling: `""`.
fn field_name(label: &str, key: &str, value: &toml::Value) -> Result<Option<Path>, ConfigError> {
    let name = value.as_str().ok_or_else(|| {
        ConfigError(format!(
            "{label}: `{key}` must be a string, found {}",
            value.type_str()
        ))
    })?;
    if name.is_empty() {
        return Ok(None);
    }
    Path::from_dotted(name).map(Some).map_err(|error| {
        ConfigError(format!(
            "{label}: `{key}` = \"{name}\" is not a valid field name: {error}"
        ))
    })
}

/// Flattens `field.override.<path> = "<name>"` into dotted sources.
///
/// A quoted key (`field.override."number.raw"`) and the nested spelling
/// (`field.override.number.raw`) are the same path, so both are split on `.`;
/// a name can never contain a dot, so nothing legitimate is lost.
fn flatten_override_paths<'a>(
    prefix: Vec<String>,
    value: &'a toml::Value,
    out: &mut Vec<(Vec<String>, &'a toml::Value)>,
) {
    let Some(table) = value.as_table() else {
        out.push((prefix, value));
        return;
    };
    for (key, value) in table {
        let mut path = prefix.clone();
        path.extend(key.split('.').map(str::to_string));
        flatten_override_paths(path, value, out);
    }
}

fn optional_string(
    table: &toml::Table,
    key: &str,
    label: &str,
) -> Result<Option<String>, ConfigError> {
    match table.get(key) {
        None => Ok(None),
        Some(toml::Value::String(value)) => Ok(Some(value.clone())),
        Some(other) => Err(ConfigError(format!(
            "{label}: `{key}` must be a string, found {}",
            other.type_str()
        ))),
    }
}

/// `imports` accepts a string, a structured `{ module, … }` table, or an array
/// of either (§11 rule 4).
fn parse_imports(label: &str, value: &toml::Value) -> Result<Vec<ImportEntry>, ConfigError> {
    let values: Vec<toml::Value> = match value {
        toml::Value::Array(items) => items.clone(),
        toml::Value::String(_) | toml::Value::Table(_) => vec![value.clone()],
        other => {
            return Err(ConfigError(format!(
                "{label}: `imports` must be a string or an array, found {}",
                other.type_str()
            )))
        }
    };

    let entries = parse_import_entries(&values);
    if entries.is_empty() && !values.is_empty() {
        return Err(ConfigError(format!(
            "{label}: `imports` entries must be strings or `{{ module = … }}` tables"
        )));
    }
    Ok(entries)
}

/// §11 rule 6: a template that opens a non-void element must be able to receive
/// children, and its tags must balance. Self-closing templates are exempt.
fn validate_template(label: &str, template: &str) -> Result<(), ConfigError> {
    let trimmed = template.trim();
    if !trimmed.starts_with('<') {
        return Err(ConfigError(format!(
            "{label}: `template` must open with an element, found `{trimmed}`"
        )));
    }

    let name = tag_name(&trimmed[1..]);
    let open_end = match find_tag_end(trimmed) {
        Some(index) => index,
        None => {
            return Err(ConfigError(format!(
                "{label}: `template` never closes `<{name}`"
            )))
        }
    };
    let opening = &trimmed[..=open_end];
    let self_closing = opening.trim_end().ends_with("/>");
    if !self_closing && !trimmed.contains("{children}") && !trimmed.contains("{text}") {
        return Err(ConfigError(format!(
            "{label}: `template` opens `<{name}>` but has no {{children}} token"
        )));
    }

    balance_tags(label, trimmed)
}

/// The tag name of `rest`, where `rest` starts just after the opening `<`.
fn tag_name(rest: &str) -> String {
    rest.split(|ch: char| ch.is_whitespace() || ch == '>' || ch == '/')
        .next()
        .unwrap_or("")
        .to_string()
}

/// Index of the `>` that closes the tag starting at the beginning of `text`,
/// skipping `{ … }` interpolation spans.
fn find_tag_end(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'{' => index += skip_interpolation(&text[index..]),
            b'>' => return Some(index),
            _ => index += 1,
        }
    }
    None
}

/// Length of the `{ … }` span at the start of `text` (at least 1).
fn skip_interpolation(text: &str) -> usize {
    match text.find('}') {
        Some(end) => end + 1,
        None => 1,
    }
}

fn balance_tags(label: &str, template: &str) -> Result<(), ConfigError> {
    let bytes = template.as_bytes();
    let mut stack: Vec<String> = Vec::new();
    let mut index = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'{' => index += skip_interpolation(&template[index..]),
            b'<' => {
                let Some(end) = template[index..].find('>') else {
                    return Err(ConfigError(format!(
                        "{label}: `template` has an unterminated tag"
                    )));
                };
                let inner = template[index + 1..index + end].trim();
                index += end + 1;

                if inner.is_empty() || inner.starts_with('!') || inner.ends_with('/') {
                    continue;
                }
                if let Some(closing) = inner.strip_prefix('/') {
                    let closing = tag_name(closing);
                    match stack.pop() {
                        Some(open) if open == closing => {}
                        Some(open) => {
                            return Err(ConfigError(format!(
                                "{label}: `template` closes `</{closing}>` but `<{open}>` is open"
                            )))
                        }
                        None => {
                            return Err(ConfigError(format!(
                                "{label}: `template` closes `</{closing}>` which was never opened"
                            )))
                        }
                    }
                    continue;
                }

                let name = tag_name(inner);
                if !pendon_core::is_void_element(&name) {
                    stack.push(name);
                }
            }
            _ => index += 1,
        }
    }

    match stack.pop() {
        None => Ok(()),
        Some(open) => Err(ConfigError(format!(
            "{label}: `template` leaves `<{open}>` unclosed"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parses a fragment that defines the `custom` key and returns its value.
    fn custom_of(src: &str) -> toml::Value {
        let table: toml::Table = toml::from_str(src).expect("valid toml");
        table.get("custom").cloned().expect("custom key")
    }

    #[test]
    fn layered_form_loads_typed_and_default_entries() {
        let custom = custom_of(
            r#"
            [custom.thead]
            type = ["theadA", "theadB"]
            name = "HeadAB"
            imports = "import { HeadAB } from '@comp/Head'"
            template = "<HeadAB {...attrs}>{children}</HeadAB>"

            [custom.caption]
            name = "CaptionDefault"
            template = "<CaptionTpl {...attrs}>{children}</CaptionTpl>"
            "#,
        );
        let loaded = load("table", "table", Some(&custom)).unwrap();

        let thead = loaded.components.layer("thead").unwrap();
        assert_eq!(thead.typed().len(), 1);
        assert!(thead.default_entry().is_none());
        let entry = &thead.typed()[0];
        assert!(entry.matches_type("theadA"));
        assert!(entry.matches_type("theadB"));
        assert_eq!(entry.name.as_deref(), Some("HeadAB"));
        assert_eq!(entry.imports.len(), 1);
        // rule 3: no literal match and no default -> built-in fallback node.
        assert!(!entry.matches_type("nope"));

        let caption = loaded.components.layer("caption").unwrap();
        assert_eq!(
            caption.default_entry().unwrap().name.as_deref(),
            Some("CaptionDefault")
        );
        assert!(loaded.components.layer("table").is_none());
    }

    #[test]
    fn direct_form_uses_the_primary_layer() {
        let custom = custom_of(
            r#"
            custom = [
              { type = "anchorA", name = "AnchorAB", template = "<AnchorAB {...attrs}>{children}</AnchorAB>" },
              { name = "AnchorDefault", template = "<AnchorDefault {...attrs}>{children}</AnchorDefault>" },
            ]
            "#,
        );
        let loaded = load("anchor", "anchor", Some(&custom)).unwrap();
        let anchor = loaded.components.layer("anchor").unwrap();
        assert_eq!(anchor.typed().len(), 1);
        assert!(anchor.typed()[0].matches_type("anchorA"));
        assert_eq!(anchor.typed()[0].name.as_deref(), Some("AnchorAB"));
        // An unmatched type is answered by the layer default.
        assert_eq!(
            anchor.default_entry().unwrap().name.as_deref(),
            Some("AnchorDefault")
        );
    }

    #[test]
    fn two_defaults_in_one_layer_is_a_hard_error() {
        let custom = custom_of(
            r#"
            custom = [
              { name = "A", template = "<A {...attrs}>{children}</A>" },
              { name = "B", template = "<B {...attrs}>{children}</B>" },
            ]
            "#,
        );
        let error = load("anchor", "anchor", Some(&custom)).unwrap_err();
        assert!(
            error.to_string().contains("two default components"),
            "{error}"
        );
    }

    #[test]
    fn template_without_children_token_is_a_hard_error() {
        let custom = custom_of(
            r#"
            [custom.caption]
            template = "<CaptionTpl {...attrs}></CaptionTpl>"
            "#,
        );
        let error = load("table", "table", Some(&custom)).unwrap_err();
        assert!(error.to_string().contains("children"), "{error}");

        // Self-closing leaves are exempt, and `{text}` also receives content.
        let leaf = custom_of(r#"custom = [{ template = "<Marker {...attrs}/>" }]"#);
        assert!(load("marker", "marker", Some(&leaf)).is_ok());
        let text_tpl = custom_of(r#"custom = [{ template = "<Cite>{text}</Cite>" }]"#);
        assert!(load("cite", "cite", Some(&text_tpl)).is_ok());
    }

    #[test]
    fn unbalanced_template_and_removed_import_are_rejected() {
        let custom =
            custom_of(r#"custom = [{ name = "X", template = "<X {...attrs}>{children}<span>" }]"#);
        let error = load("anchor", "anchor", Some(&custom)).unwrap_err();
        assert!(error.to_string().contains("unclosed"), "{error}");

        // D-16: the singular `import` is a load error that names `imports`.
        let single = custom_of(
            r#"custom = [{ type = "anchorA", name = "A", import = "import A from 'x'", template = "<A {...attrs}>{children}</A>" }]"#,
        );
        let error = load("anchor", "anchor", Some(&single)).unwrap_err();
        assert!(
            error.to_string().contains("`import` was removed"),
            "{error}"
        );

        // `type` still accepts a single string, and a single import entry is one.
        let loaded = load(
            "anchor",
            "anchor",
            Some(&custom_of(
                r#"custom = [{ type = "anchorA", imports = ["import A from 'x'"], template = "<A {...attrs}>{children}</A>" }]"#,
            )),
        )
        .unwrap();
        let set = loaded.components.layer("anchor").expect("anchor layer");
        let entry = &set.typed()[0];
        assert_eq!(entry.literal_types().collect::<Vec<_>>(), vec!["anchorA"]);
        assert_eq!(entry.imports.len(), 1);
    }

    /// §11 rule 3: a literal entry and the layer default are two distinct
    /// entries of one layer. Which one answers a node is decided once per
    /// document by the renderer (ADR-0008 `D-3a`), so the loader only records
    /// them — with the selectors the user wrote and the default kept apart.
    #[test]
    fn literal_entries_and_the_layer_default_load_side_by_side() {
        let custom = custom_of(
            r#"
            [[custom.thead]]
            type = ["theadA", "theadB"]
            name = "HeadAB"
            template = "<HeadAB>{children}</HeadAB>"

            [[custom.thead]]
            name = "HeadDefault"
            template = "<HeadDefault>{children}</HeadDefault>"
            "#,
        );
        let loaded = load("table", "table", Some(&custom)).unwrap();

        let set = loaded.components.layer("thead").expect("thead layer");
        assert_eq!(set.typed().len(), 1);
        assert!(set.typed()[0].matches_type("theadA"));
        assert!(set.typed()[0].matches_type("theadB"));
        assert_eq!(set.typed()[0].name.as_deref(), Some("HeadAB"));
        assert_eq!(
            set.default_entry().and_then(|entry| entry.name.as_deref()),
            Some("HeadDefault")
        );

        // A layer without a default leaves an unclaimed type to the built-in
        // element (§11 rule 3).
        let typed_only = custom_of(
            r#"
            [[custom.thead]]
            type = "theadA"
            name = "HeadA"
            template = "<HeadA>{children}</HeadA>"
            "#,
        );
        let loaded = load("table", "table", Some(&typed_only)).unwrap();
        let set = loaded.components.layer("thead").expect("thead layer");
        assert!(set.default_entry().is_none());
        assert!(loaded.components.layer("tfoot").is_none());
    }

    /// §11 rule 1: layer names are the elements a plugin emits, so an unknown
    /// layer is a typo or an unwired cutover, never a silent no-op.
    /// §11 + D-4: `type` entries are selectors, not marker strings. A typo is a
    /// load-time error naming the key and the layer, never a wildcard.
    #[test]
    fn type_entries_are_parsed_as_selectors() {
        let custom = custom_of(
            r#"
            [[custom.thead]]
            type = ["theadA", "_"]
            name = "HeadRest"
            template = "<HeadRest {...attrs}>{children}</HeadRest>"
            "#,
        );
        let loaded = load("table", "table", Some(&custom)).unwrap();
        let entry = &loaded.components.layer("thead").unwrap().typed()[0];
        assert_eq!(entry.selectors.len(), 2);
        assert_eq!(entry.literal_types().collect::<Vec<_>>(), vec!["theadA"]);

        for (source, expected) in [
            (r#"type = ["??"]"#, "`??` is not a selector"),
            (r#"type = [""]"#, "is not a selector"),
        ] {
            let broken = custom_of(&format!(
                r#"[[custom.thead]]
{source}
name = "H"
template = "<H>{{children}}</H>"
"#
            ));
            let error = load("table", "table", Some(&broken)).unwrap_err();
            assert!(
                error.to_string().contains("task.table.custom.thead")
                    && error.to_string().contains(expected),
                "{error}"
            );
        }
    }

    /// D-4: a literal `type` may appear once per layer; wildcards may repeat.
    #[test]
    fn duplicate_literals_stay_an_error_but_wildcards_may_repeat() {
        let duplicated = custom_of(
            r#"
            [[custom.thead]]
            type = "theadA"
            name = "A"
            template = "<A>{children}</A>"

            [[custom.thead]]
            type = "theadA"
            name = "B"
            template = "<B>{children}</B>"
            "#,
        );
        let error = load("table", "table", Some(&duplicated)).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("duplicate `type` marker `theadA`"),
            "{error}"
        );

        let wildcards = custom_of(
            r#"
            [[custom.thead]]
            type = "_"
            name = "First"
            template = "<First>{children}</First>"

            [[custom.thead]]
            type = "*"
            name = "Second"
            template = "<Second>{children}</Second>"
            "#,
        );
        let loaded = load("table", "table", Some(&wildcards)).unwrap();
        let thead = loaded.components.layer("thead").unwrap();
        // A wildcard entry is a *rule*, never the default slot.
        assert!(thead.default_entry().is_none());
        assert_eq!(thead.typed().len(), 2);
    }

    #[test]
    fn unwired_layers_are_rejected() {
        let custom = custom_of(
            r#"
            [custom.thead]
            name = "Head"
            template = "<Head>{children}</Head>"

            [custom.legend]
            name = "Legend"
            template = "<Legend>{children}</Legend>"
            "#,
        );
        let loaded = load("table", "table", Some(&custom)).unwrap();
        let error = reject_unwired_layers("table", &loaded.components, &["thead"]).unwrap_err();
        assert!(error.to_string().contains("unsupported layer"), "{error}");

        assert!(reject_unwired_layers("table", &loaded.components, &["thead", "legend"]).is_ok());
    }

    // ---- S2-2.3: `field.*` keys (D-5, ADR-0014, §7 name rules) ----

    /// The proposal's own assignment block, verbatim (§7 "Assign fields").
    #[test]
    fn the_proposal_assignment_block_parses_verbatim() {
        let custom = custom_of(
            r#"
            [[custom]]
            type = "typeA"
            field.bracket.bare = "anchor_id"
            field.bracket.quote = "inner_text"
            field.bracket.tick = "extra-class"
            field.paren.bare = "href"
            field.paren.quote = "title"
            field.paren.tick = "popupID"
            field.specs.tick = "ref-id"
            field.specs.quote = "_extra-note"
            field.specs.id = "id-123"
            field.specs.class = "class"
            field.specs.style = "styles"
            "#,
        );
        let loaded = load("anchor", "anchor", Some(&custom)).unwrap();
        let entry = &loaded.components.layer("anchor").unwrap().typed()[0];
        assert!(entry.matches_type("typeA"));

        let field = &entry.field;
        assert!(!field.is_empty());
        let assigned = |slot, kind| {
            field
                .assignment(slot, kind)
                .and_then(Option::as_ref)
                .map(Path::dotted)
        };
        assert_eq!(
            assigned(Slot::Bracket, SlotKind::Bare).as_deref(),
            Some("anchor_id")
        );
        assert_eq!(
            assigned(Slot::Bracket, SlotKind::Quote).as_deref(),
            Some("inner_text")
        );
        assert_eq!(
            assigned(Slot::Bracket, SlotKind::Tick).as_deref(),
            Some("extra-class")
        );
        assert_eq!(
            assigned(Slot::Paren, SlotKind::Bare).as_deref(),
            Some("href")
        );
        assert_eq!(
            assigned(Slot::Paren, SlotKind::Quote).as_deref(),
            Some("title")
        );
        assert_eq!(
            assigned(Slot::Paren, SlotKind::Tick).as_deref(),
            Some("popupID")
        );

        let specs: Vec<(&str, Option<String>)> = field
            .specs()
            .iter()
            .map(|(key, target)| (key.as_str(), target.as_ref().map(Path::dotted)))
            .collect();
        assert_eq!(
            specs,
            [
                ("class", Some("class".to_string())),
                ("id", Some("id-123".to_string())),
                ("quote", Some("_extra-note".to_string())),
                ("style", Some("styles".to_string())),
                ("tick", Some("ref-id".to_string())),
            ]
        );
    }

    /// D-8: `custom` is always an array, so a single table is a load error —
    /// and the message points at the array form rather than reading `field` as
    /// a layer name.
    #[test]
    fn a_single_custom_table_points_at_the_array_form() {
        let custom = custom_of(
            r#"
            [custom]
            field.paren.bare = "something"
            "#,
        );
        let error = load("anchor", "anchor", Some(&custom)).unwrap_err();
        let message = error.to_string();
        assert!(message.contains("must be an array of tables"), "{message}");
        assert!(message.contains("[[task.anchor.custom]]"), "{message}");

        // The array form of the same entry is a fields-only entry, and it is
        // the layer default because it carries no `type`.
        let array = custom_of(
            r#"
            [[custom]]
            field.paren.bare = "something"
            "#,
        );
        let loaded = load("anchor", "anchor", Some(&array)).unwrap();
        let entry = loaded
            .components
            .layer("anchor")
            .unwrap()
            .default_entry()
            .expect("field-only entry is the layer default");
        assert_eq!(
            entry
                .field
                .assignment(Slot::Paren, SlotKind::Bare)
                .and_then(Option::as_ref)
                .map(Path::dotted)
                .as_deref(),
            Some("something")
        );
    }

    /// `""` unassigns a position (OBJ-10) and still counts as a field
    /// definition, so it voids the preset (D-7).
    #[test]
    fn an_empty_name_unassigns_the_position() {
        let custom = custom_of(
            r#"
            [[custom]]
            field.paren.quote = ""
            "#,
        );
        let loaded = load("anchor", "anchor", Some(&custom)).unwrap();
        let entry = loaded
            .components
            .layer("anchor")
            .unwrap()
            .default_entry()
            .expect("field-only entry is the layer default");
        assert!(!entry.field.is_empty());
        assert_eq!(
            entry.field.assignment(Slot::Paren, SlotKind::Quote),
            Some(&None)
        );
    }

    /// An `override` source may be written nested or quoted; both mean one path,
    /// and naming it twice is a load error.
    #[test]
    fn overrides_accept_both_spellings_and_reject_a_duplicate() {
        let custom = custom_of(
            r#"
            [[custom]]
            field.override.number.raw = "bar"
            field.override.specs.tick = "ref"
            "#,
        );
        let loaded = load("anchor", "anchor", Some(&custom)).unwrap();
        let entry = loaded
            .components
            .layer("anchor")
            .unwrap()
            .default_entry()
            .expect("field-only entry is the layer default");
        assert_eq!(entry.field.overrides().len(), 2);

        let duplicate = custom_of(
            r#"
            [[custom]]
            field.override.number.raw = "bar"
            field.override."number.raw" = "baz"
            "#,
        );
        let error = load("anchor", "anchor", Some(&duplicate)).unwrap_err();
        let message = error.to_string();
        assert!(message.contains("overridden twice"), "{message}");
        assert!(message.contains("number.raw"), "{message}");
    }

    /// Every malformed `field.*` shape is a load error naming the key — none is
    /// ignored (D-16).
    #[test]
    fn malformed_field_keys_are_load_errors() {
        let cases = [
            ("field.brackett.bare = \"x\"", "is not a field config key"),
            ("field.bracket.tikc = \"x\"", "is not a slot content"),
            ("field.bracket = \"x\"", "must be a table"),
            ("field.bracket.bare = 12", "must be a string"),
            ("field.specs.tikc = \"x\"", "unknown specs position"),
            ("field.specs = \"x\"", "must be a table"),
            ("field.bracket.bare = \"1x\"", "is not a valid field name"),
            ("field.override = \"x\"", "`field.override` must be a table"),
            ("field.override.number.raw = \"\"", "needs a name"),
            ("field = 3", "`field` must be a table"),
        ];
        for (source, expected) in cases {
            let custom = custom_of(&format!("[[custom]]\n{source}\n"));
            let error = load("anchor", "anchor", Some(&custom))
                .err()
                .unwrap_or_else(|| panic!("{source} should not load"));
            let message = error.to_string();
            assert!(message.contains(expected), "{source}: {message}");
            assert!(message.contains("anchor"), "{source}: {message}");
        }
    }

    /// S2-9.3: a `number` block may rename the three fields it publishes and
    /// nothing else — each malformed shape is a load error naming the key, not an
    /// ignored table (D-16).
    #[test]
    fn numbering_blocks_take_only_renames_of_their_three_fields() {
        let field = |source: &str| -> toml::Value {
            let value: toml::Value = toml::from_str(source).expect("valid TOML");
            value.get("field").expect("a `field` key").clone()
        };
        let label = "task.heading.number[0]";

        let overrides = number_field_overrides(
            label,
            &field(
                r#"
                [field.override]
                "number.raw" = "counter"
                number.depth = "level"
                number.type = "counter_type"
                "#,
            ),
        )
        .expect("three renames, quoted and nested spellings alike");
        assert_eq!(
            overrides.get("number.raw").map(String::as_str),
            Some("counter")
        );
        assert_eq!(
            overrides.get("number.depth").map(String::as_str),
            Some("level")
        );
        assert_eq!(
            overrides.get("number.type").map(String::as_str),
            Some("counter_type")
        );

        let cases = [
            (
                "[field]\nspecs.tick = \"x\"\n",
                "is not a numbering field config key",
            ),
            (
                "[field.override]\nnumber.bogus = \"x\"\n",
                "is not a numbering field",
            ),
            (
                "[field.override]\nnumber.raw = \"number.depth\"\n",
                "renames onto a field this block publishes",
            ),
            (
                "[field.override]\nnumber.raw = \"x\"\nnumber.depth = \"x\"\n",
                "a rename target is unique",
            ),
            ("[field.override]\nnumber.raw = \"\"\n", "needs a name"),
            ("field = 3\n", "`field` must be a table"),
            (
                "[field]\noverride = 3\n",
                "`field.override` must be a table",
            ),
        ];
        for (source, expected) in cases {
            let error = number_field_overrides(label, &field(source))
                .err()
                .unwrap_or_else(|| panic!("{source} should not load"));
            let message = error.to_string();
            assert!(message.contains(expected), "{source}: {message}");
            assert!(message.contains(label), "{source}: {message}");
        }
    }

    /// One entry may carry a component **and** a field config (D-8); the
    /// fields reach the layer through the same lookup as a fields-only entry.
    #[test]
    fn one_entry_can_carry_a_component_and_fields() {
        let custom = custom_of(
            r#"
            [[custom]]
            type = "typeA"
            name = "AnchorA"
            template = "<AnchorA>{children}</AnchorA>"
            field.bracket.bare = "anchor_id"
            "#,
        );
        let loaded = load("anchor", "anchor", Some(&custom)).unwrap();
        let layer = loaded.components.layer("anchor").unwrap();
        let entry = &layer.typed()[0];
        assert!(entry.is_configured());
        assert_eq!(
            entry
                .field
                .assignment(Slot::Bracket, SlotKind::Bare)
                .and_then(Option::as_ref)
                .map(Path::dotted)
                .as_deref(),
            Some("anchor_id")
        );
        assert!(layer.field_config(Some("typeA")).is_some());
        // Nothing configures the default slot, so the element's preset stays.
        assert!(layer.field_config(Some("typeZ")).is_none());
        assert!(layer.field_config(None).is_none());
    }

    /// D-8 merge order: a typed entry's fields beat the element-wide default's,
    /// and an entry without `field.*` is transparent in both directions.
    #[test]
    fn a_typed_field_entry_beats_the_element_wide_default() {
        let custom = custom_of(
            r#"
            [[custom]]
            field.bracket.bare = "wide_default"

            [[custom]]
            type = "typeA"
            name = "AnchorA"
            template = "<AnchorA>{children}</AnchorA>"

            [[custom]]
            type = "typeA"
            field.bracket.bare = "typed_name"
            "#,
        );
        let loaded = load("anchor", "anchor", Some(&custom)).unwrap();
        let layer = loaded.components.layer("anchor").unwrap();
        let bare_of = |marker| {
            layer
                .field_config(marker)
                .and_then(|field| field.assignment(Slot::Bracket, SlotKind::Bare))
                .and_then(Option::as_ref)
                .map(Path::dotted)
        };
        assert_eq!(bare_of(Some("typeA")).as_deref(), Some("typed_name"));
        assert_eq!(bare_of(Some("typeZ")).as_deref(), Some("wide_default"));
        assert_eq!(bare_of(None).as_deref(), Some("wide_default"));
    }

    /// D-8: an entry that says nothing at all is a load error, not a silent
    /// no-op — and the array form is the only way to write an entry.
    #[test]
    fn an_empty_entry_is_a_load_error() {
        for source in ["[[custom]]\n", "custom = [{}]"] {
            let custom = custom_of(source);
            let error = load("anchor", "anchor", Some(&custom)).unwrap_err();
            assert!(
                error.to_string().contains("empty custom entry"),
                "{source}: {error}"
            );
        }
    }

    /// §14/D-16: the four legacy naming keys are **removed**, not aliased.
    /// Each one is an unknown key now, so a stale config fails the load instead
    /// of silently keeping `slug` / `title` (the tested `field.*` spelling is
    /// `field.bracket.bare` / `field.paren.bare` / `field.specs.tick|quote`).
    #[test]
    fn each_removed_naming_key_is_a_load_error() {
        for key in [
            "backtick_key",
            "quote_key",
            "bracket_key",
            "parentheses_key",
        ] {
            let source = format!("[[custom]]\nname = \"Anchor\"\n{key} = \"slug\"\n");
            let custom = custom_of(&source);
            let error = load("anchor", "anchor", Some(&custom)).unwrap_err();
            let message = error.to_string();
            assert!(message.contains("unknown key"), "{key}: {message}");
            assert!(message.contains(key), "{key}: {message}");
        }
    }

    /// …and the entry that spends its vocabulary on `field.*` still loads: the
    /// removal above must not turn a valid config into an error.
    #[test]
    fn an_entry_with_field_keys_still_loads() {
        let custom = custom_of(
            r#"
            [[custom]]
            name = "Anchor"
            field.bracket.bare = "anchor"
            field.paren.bare = "level"
            field.specs.tick = "ref"
            field.specs.quote = "blurb"
            "#,
        );
        let loaded = load("anchor", "anchor", Some(&custom)).unwrap();
        let layer = loaded.components.layer("anchor").unwrap();
        let field = layer.field_config(Some("x")).expect("the default entry");
        assert_eq!(
            field.assignment(Slot::Bracket, SlotKind::Bare),
            Some(&Some(Path::from_dotted("anchor").unwrap()))
        );
        assert_eq!(
            field.specs().get("tick"),
            Some(&Some(Path::from_dotted("ref").unwrap()))
        );
    }

    /// D-16: a typo inside an entry is a load error too — the entry vocabulary is
    /// closed, so nothing is ever dropped silently.
    #[test]
    fn an_unknown_entry_key_is_a_load_error() {
        let custom = custom_of("[[custom]]\nname = \"Anchor\"\ncolour = \"red\"\n");
        let error = load("anchor", "anchor", Some(&custom)).unwrap_err();
        assert!(
            error.to_string().contains("unknown key `colour`"),
            "{error}"
        );
    }

    // --- §5.4 slot modifiers (S2-6.1, D-9, ADR-0017) ----------------------

    /// S2-6.1: a symbol is **one UTF-8 character**, never one byte. `?`, `|`,
    /// `Œ`, `\` and `ئ` are all valid and all mean only what the entry says —
    /// Pendon ships no modifier of its own, so nothing is interpreted here.
    #[test]
    fn a_modifier_symbol_is_one_utf8_char() {
        let custom = custom_of(
            r#"
            [[custom]]
            name = "Anchor"
            bracket.modifier."?" = { rel = "nofollow" }
            bracket.modifier."|" = { rel = "nofollow" }
            bracket.modifier."Œ" = { rel = "nofollow" }
            bracket.modifier."\\" = { rel = "nofollow" }
            bracket.modifier."ئ" = { rel = "nofollow" }
            "#,
        );
        let loaded = load("anchor", "anchor", Some(&custom)).expect("loads");
        let entry = loaded
            .components
            .layer("anchor")
            .unwrap()
            .default_entry()
            .expect("the default entry");
        let config = &entry.modifier;
        assert_eq!(config.symbols().count(), 5);
        for symbol in ['?', '|', 'Œ', '\\', 'ئ'] {
            let fields = config.for_symbol(symbol);
            assert_eq!(fields.len(), 1, "{symbol}");
            assert_eq!(fields[0].0, Path::from_dotted("rel").unwrap(), "{symbol}");
            assert_eq!(fields[0].1, AttrValue::Str("nofollow".into()), "{symbol}");
        }
        // A symbol no entry defines contributes nothing; it is not a diagnostic.
        assert!(config.for_symbol('~').is_empty());
    }

    /// A symbol is one character, so a run of two is a typo — and the message
    /// says which rule it broke.
    #[test]
    fn a_multi_char_symbol_is_a_load_error() {
        let custom =
            custom_of("[[custom]]\nname = \"Anchor\"\nbracket.modifier.\"??\" = { rel = \"x\" }\n");
        let error = load("anchor", "anchor", Some(&custom)).unwrap_err();
        assert!(
            error.to_string().contains("exactly one character"),
            "{error}"
        );
    }

    /// D-9 rule 5: a modifier run exists in a bracket, so `paren.modifier` is not
    /// syntax — and it is not reported as a typo either.
    #[test]
    fn a_modifier_outside_the_bracket_is_a_load_error() {
        let custom =
            custom_of("[[custom]]\nname = \"Anchor\"\nparen.modifier.\"?\" = { rel = \"x\" }\n");
        let error = load("anchor", "anchor", Some(&custom)).unwrap_err();
        assert!(error.to_string().contains("bracket-only"), "{error}");
    }

    /// The entry vocabulary under `bracket` is closed (D-5): the only key is
    /// `modifier`, and a `field.*` name is not reachable from here.
    #[test]
    fn an_unknown_bracket_key_is_a_load_error() {
        let custom = custom_of("[[custom]]\nname = \"Anchor\"\nbracket.other = \"x\"\n");
        let error = load("anchor", "anchor", Some(&custom)).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("`bracket.other` is not a modifier config key"),
            "{error}"
        );
    }

    /// A modifier config is a definition on its own (D-8): the entry needs no
    /// component, and a dotted key and the nested spelling are one path.
    #[test]
    fn a_modifier_entry_without_a_component_loads() {
        let custom = custom_of(
            r#"
            [[custom]]
            type = "Check"
            bracket.modifier."?" = { rel = "nofollow", "soo.baz" = "wax" }
            "#,
        );
        let loaded = load("anchor", "anchor", Some(&custom)).expect("a modifier-only entry");
        let entry = &loaded.components.layer("anchor").unwrap().typed()[0];
        assert!(entry.carries_config());
        assert!(!entry.is_configured());
        let fields = entry.modifier.for_symbol('?');
        let names: Vec<String> = fields.iter().map(|(path, _)| path.dotted()).collect();
        assert_eq!(names, ["rel", "soo.baz"]);
    }

    /// D-8/D-4: two configs for one literal would silently drop one, so the load
    /// fails the same way two components for one literal do.
    #[test]
    fn two_modifier_configs_for_one_literal_clash() {
        let custom = custom_of(
            r#"
            [[custom]]
            type = "Check"
            bracket.modifier."?" = { rel = "nofollow" }

            [[custom]]
            type = "Check"
            bracket.modifier."!" = { rel = "noopener" }
            "#,
        );
        let error = load("anchor", "anchor", Some(&custom)).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("duplicate `type` marker `Check`"),
            "{error}"
        );
    }
}
