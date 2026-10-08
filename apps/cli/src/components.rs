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

use pendon_renderer_solid::{ComponentTemplate, ImportEntry};

use crate::plugins::parse_import_entries;

/// A fatal configuration problem: §11 rules 2 and 6 are hard errors at load.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError(pub String);

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ConfigError {}

/// Positional naming (§6.1, §11 rule 5): the per-component `backtick_key` /
/// `quote_key`, plus `bracket_key` / `parentheses_key` for directive heads.
///
/// Defined in `pendon-extra` (the all-`Option` config form) so the renderer, the
/// plugins and the CLI share one shape and one `resolve`.
pub use pendon_extra::PositionalKeys;

/// One `custom` entry: the types it answers, its imports and its template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentEntry {
    /// `type` markers. Empty means "this is the layer default" (rule 2).
    pub types: Vec<String>,
    pub name: Option<String>,
    pub imports: Vec<ImportEntry>,
    pub template: Option<String>,
    pub positional: PositionalKeys,
}

impl ComponentEntry {
    /// `true` for the single default entry of a layer.
    pub fn is_default(&self) -> bool {
        self.types.is_empty()
    }

    /// Answers `type_marker`, or `None` when it is not this entry's type.
    pub fn matches_type(&self, type_marker: &str) -> bool {
        self.types.iter().any(|known| known == type_marker)
    }

    /// Renders this entry as a renderer hint for `node_type` (§11 rule 3).
    #[allow(dead_code)] // §11 rule-3 API: the Phase-1 binder builds hints from it.
    pub fn to_template(
        &self,
        node_type: &str,
        node_name: Option<&str>,
    ) -> Option<ComponentTemplate> {
        Some(ComponentTemplate {
            node_type: node_type.to_string(),
            node_name: node_name.map(str::to_string),
            template: self.template.clone()?,
        })
    }
}

/// The component set of one layer (§11 rule 1).
#[derive(Debug, Clone, PartialEq, Eq)]
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

    #[allow(dead_code)] // §11 rule-3 API: the Phase-1 binder routes by `type`.
    pub fn layer(&self) -> &str {
        &self.layer
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

    /// §11 rule 3: exact `type` match → layer default → `None` (built-in
    /// fallback, which must still carry every extra as an attribute).
    #[allow(dead_code)] // §11 rule-3 API: the Phase-1 binder selects per node.
    pub fn select(&self, type_marker: Option<&str>) -> Option<&ComponentEntry> {
        if let Some(marker) = type_marker {
            if let Some(entry) = self.typed.iter().find(|entry| entry.matches_type(marker)) {
                return Some(entry);
            }
        }
        self.default.as_ref()
    }

    /// Convenience for hint construction: a template per entry, tagged with the
    /// node kind the layer emits.
    #[allow(dead_code)] // §11 rule-3 API: kept as the config-side mirror of the plugin sets.
    pub fn templates(&self, node_type: &str, node_name: Option<&str>) -> Vec<ComponentTemplate> {
        self.typed
            .iter()
            .chain(self.default.iter())
            .filter_map(|entry| entry.to_template(node_type, node_name))
            .collect()
    }
}

/// Every layer's component set for one task-level plugin.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
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

    #[allow(dead_code)] // §11 rule-3 API: the Phase-1 binder skips empty sets.
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    /// §11 rule 3, per layer.
    #[allow(dead_code)] // §11 rule-3 API: the Phase-1 binder selects per node.
    pub fn select(&self, layer: &str, type_marker: Option<&str>) -> Option<&ComponentEntry> {
        self.layers
            .get(layer)
            .and_then(|set| set.select(type_marker))
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

/// Result of loading one plugin's component config: the sets plus non-fatal
/// deprecation notices (§11 rule 4).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LoadedComponents {
    pub components: PluginComponents,
    pub warnings: Vec<String>,
}

/// Keys that mark a table as a component entry rather than a map of layers.
const ENTRY_KEYS: [&str; 9] = [
    "type",
    "name",
    "imports",
    "import",
    "template",
    "backtick_key",
    "quote_key",
    "bracket_key",
    "parentheses_key",
];

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
            let set = parse_set(plugin, primary_layer, custom, &mut out.warnings)?;
            merge_set(&mut out.components, set)?;
        }
        toml::Value::Table(table) if is_layer_map(table) => {
            for (layer, value) in table {
                let set = parse_set(plugin, layer, value, &mut out.warnings)?;
                merge_set(&mut out.components, set)?;
            }
        }
        toml::Value::Table(_) => {
            let set = parse_set(plugin, primary_layer, custom, &mut out.warnings)?;
            merge_set(&mut out.components, set)?;
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

fn parse_set(
    plugin: &str,
    layer: &str,
    value: &toml::Value,
    warnings: &mut Vec<String>,
) -> Result<ComponentSet, ConfigError> {
    let mut set = ComponentSet::new(layer);
    let label = format!("task.{plugin}.custom.{layer}");

    match value {
        toml::Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                let table = item.as_table().ok_or_else(|| {
                    ConfigError(format!("{label}: entry #{} must be a table", index + 1))
                })?;
                let entry = parse_entry(&format!("{label}[{}]", index + 1), table, warnings)?;
                check_duplicate_type(&set, &entry, &label)?;
                set.insert(entry)?;
            }
        }
        toml::Value::Table(table) => {
            let entry = parse_entry(&label, table, warnings)?;
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

fn check_duplicate_type(
    set: &ComponentSet,
    entry: &ComponentEntry,
    label: &str,
) -> Result<(), ConfigError> {
    for known in &entry.types {
        if set.typed().iter().any(|other| other.matches_type(known)) {
            return Err(ConfigError(format!(
                "{label}: duplicate `type` marker `{known}`"
            )));
        }
    }
    Ok(())
}

fn parse_entry(
    label: &str,
    table: &toml::Table,
    warnings: &mut Vec<String>,
) -> Result<ComponentEntry, ConfigError> {
    let types = match table.get("type") {
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
    // `[]` is equivalent to an absent `type` (§11 rule 2).
    let types: Vec<String> = types
        .into_iter()
        .map(|marker| marker.trim().to_string())
        .filter(|marker| !marker.is_empty())
        .collect();

    let name = optional_string(table, "name", label)?;
    let template = optional_string(table, "template", label)?;
    if let Some(template) = &template {
        validate_template(label, template)?;
    }

    // §11 rule 4: `imports` is canonical, the singular `import` is deprecated.
    let mut imports = Vec::new();
    if let Some(value) = table.get("import") {
        warnings.push(format!("{label}: `import` is deprecated, use `imports`"));
        imports = parse_imports(label, value)?;
    }
    if let Some(value) = table.get("imports") {
        imports = parse_imports(label, value)?;
    }

    Ok(ComponentEntry {
        types,
        name,
        imports,
        template,
        positional: PositionalKeys {
            backtick_key: optional_string(table, "backtick_key", label)?,
            quote_key: optional_string(table, "quote_key", label)?,
            bracket_key: optional_string(table, "bracket_key", label)?,
            parentheses_key: optional_string(table, "parentheses_key", label)?,
        },
    })
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
        assert!(loaded.warnings.is_empty());

        let thead = loaded.components.layer("thead").unwrap();
        assert_eq!(thead.typed().len(), 1);
        assert!(thead.default_entry().is_none());
        assert_eq!(
            thead.select(Some("theadB")).unwrap().name.as_deref(),
            Some("HeadAB")
        );
        assert_eq!(thead.select(Some("theadA")).unwrap().imports.len(), 1);
        // rule 3: no match and no default -> built-in fallback node.
        assert!(thead.select(Some("nope")).is_none());

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
        assert_eq!(
            loaded
                .components
                .select("anchor", Some("anchorA"))
                .unwrap()
                .name
                .as_deref(),
            Some("AnchorAB")
        );
        // Unmatched type falls back to the layer default.
        assert_eq!(
            loaded
                .components
                .select("anchor", Some("other"))
                .unwrap()
                .name
                .as_deref(),
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
    fn unbalanced_template_and_deprecated_import_are_reported() {
        let custom =
            custom_of(r#"custom = { name = "X", template = "<X {...attrs}>{children}<span>" }"#);
        let error = load("anchor", "anchor", Some(&custom)).unwrap_err();
        assert!(error.to_string().contains("unclosed"), "{error}");

        // `type` accepts a single string, and `import` only warns.
        let single = custom_of(
            r#"custom = { type = "anchorA", name = "A", import = "import A from 'x'", template = "<A {...attrs}>{children}</A>" }"#,
        );
        let loaded = load("anchor", "anchor", Some(&single)).unwrap();
        assert_eq!(loaded.warnings.len(), 1);
        let entry = loaded.components.select("anchor", Some("anchorA")).unwrap();
        assert_eq!(entry.types, vec!["anchorA".to_string()]);
        assert_eq!(entry.imports.len(), 1);
    }

    /// §11 rule 3, at the config level: exact `type` match → layer default →
    /// `None` (the plugin then falls back to its built-in element), and every
    /// entry of a layer gets a template for the renderer hints.
    #[test]
    fn select_routes_type_then_default_then_fallback() {
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

        let entry = loaded.components.select("thead", Some("theadA"));
        assert_eq!(
            entry.and_then(|entry| entry.name.as_deref()),
            Some("HeadAB")
        );

        let entry = loaded.components.select("thead", Some("theadZ"));
        assert_eq!(
            entry.and_then(|entry| entry.name.as_deref()),
            Some("HeadDefault")
        );

        // A layer without a default falls back to the built-in element.
        let typed_only = custom_of(
            r#"
            [[custom.thead]]
            type = "theadA"
            name = "HeadA"
            template = "<HeadA>{children}</HeadA>"
            "#,
        );
        let loaded = load("table", "table", Some(&typed_only)).unwrap();
        assert!(loaded.components.select("thead", Some("theadZ")).is_none());

        // §11 rule 3 hints: one template per entry, both entries covered.
        let set = loaded.components.layer("thead").expect("thead layer");
        assert_eq!(set.templates("thead", None).len(), 1);
        assert!(loaded.components.layer("tfoot").is_none());
    }

    /// §11 rule 1: layer names are the elements a plugin emits, so an unknown
    /// layer is a typo or an unwired cutover, never a silent no-op.
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
}
