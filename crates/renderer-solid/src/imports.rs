use pendon_extra::FieldConfig;
use pendon_types::Selector;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// One import entry for a custom component template.
///
/// Every task-level plugin (cite, img, anchor, heading, table) shares this type
/// and the same TOML syntax: either a raw import line (`"import X from 'y'"`)
/// or a structured `{ module, default, names }` table.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ImportEntry {
    Raw(String),
    Structured {
        module: String,
        default: Option<String>,
        names: Vec<String>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentTemplate {
    /// The §11 layer this entry answers (`heading`, `row`, `figure`, …), as the
    /// element plugin tags its nodes with `__layer`.
    ///
    /// `None` for components that are not §11 layers (`plugin-custom`'s `toml:`
    /// specs, `plugin-vicado`): those match on the node identity below.
    pub layer: Option<String>,
    pub node_type: String,
    pub node_name: Option<String>,
    /// §11 rule 2: the `type` selectors the entry answers. An empty list marks
    /// the **layer default**; `?`, `_` and `*` are rules like any other.
    pub selectors: Vec<Selector>,
    pub template: String,
    /// §11 rule 3: the imports this entry needs. They are emitted only when the
    /// entry wins at least one occurring type, so a losing component's import
    /// line never appears.
    pub imports: Vec<ImportEntry>,
    /// §6.1 / D-4: the `field.*` keys of the entry that owns this template.
    ///
    /// The projection is what spells a node's attributes, so it is also the only
    /// place `field.override.<path>` can rename one — and the entry is the only
    /// source of that config (`S2-4.4`). `None` for a template that is not a
    /// configured entry (identity matches, the CLI's single template), which
    /// projects the node as it stands.
    pub field: Option<FieldConfig>,
}

impl ComponentTemplate {
    /// A §11 layer entry: the layer it answers, the type selectors, its
    /// template and the imports it needs.
    ///
    /// Resolution keys on `layer` (the `__layer` tag the plugin emits); the
    /// node identity fields mirror it and are not consulted.
    pub fn new_layer(
        layer: impl Into<String>,
        selectors: Vec<Selector>,
        template: impl Into<String>,
        imports: Vec<ImportEntry>,
    ) -> Self {
        let layer = layer.into();
        Self {
            node_type: layer.clone(),
            node_name: Some(layer.clone()),
            layer: Some(layer),
            selectors,
            template: template.into(),
            imports,
            field: None,
        }
    }

    /// Attaches the entry's `field.*` (§6.1, D-4), so an override renames the
    /// attribute the projection writes.
    ///
    /// Set by [`layer_templates`](crate::layer_templates), which is where a §11
    /// entry is turned into a template — the element plugins never see this.
    pub fn with_field(mut self, field: FieldConfig) -> Self {
        self.field = if field.is_empty() { None } else { Some(field) };
        self
    }

    /// A component that is **not** a §11 layer: it answers the AST node
    /// identity — the node type plus, when the node carries one, its `name`
    /// attribute.
    ///
    /// `plugin-custom`'s `toml:` specs and `plugin-vicado` are registered this
    /// way, as is the CLI's `[<plugin>.renderer.solid]` template.
    pub fn new_identity(
        node_type: impl Into<String>,
        node_name: Option<String>,
        template: impl Into<String>,
        imports: Vec<ImportEntry>,
    ) -> Self {
        Self {
            layer: None,
            node_type: node_type.into(),
            node_name,
            selectors: Vec::new(),
            template: template.into(),
            imports,
            field: None,
        }
    }
}

/// Where a module-level `export const` came from — **reporting only**
/// (ADR-0009). Emission treats every registration the same.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExportSource {
    /// `renderer-solid` registered it from the AST (`frontmatter`,
    /// `headings`).
    Automatic,
    /// A plugin registered it (`cite`, `bind`, …).
    Plugin(String),
}

/// One module-level data binding: `export const <name> = <value>;`.
///
/// Producers push these instead of filling per-payload hint fields
/// (ADR-0009); `renderer-solid` owns the emission, the order and the
/// "same name → last wins" rule.
#[derive(Clone, Debug, PartialEq)]
pub struct DataExport {
    pub name: String,
    pub value: Value,
    pub source: ExportSource,
}

impl DataExport {
    /// An export `renderer-solid` derives from the AST itself.
    pub fn automatic(name: impl Into<String>, value: Value) -> Self {
        Self {
            name: name.into(),
            value,
            source: ExportSource::Automatic,
        }
    }

    /// An export a plugin registered.
    pub fn plugin(plugin: impl Into<String>, name: impl Into<String>, value: Value) -> Self {
        Self {
            name: name.into(),
            value,
            source: ExportSource::Plugin(plugin.into()),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SolidRenderHints {
    pub global_imports: Vec<ImportEntry>,
    pub text_imports: Vec<(String, Vec<ImportEntry>)>,
    pub templates: Vec<ComponentTemplate>,
    /// Module-level `export const` registrations, in insertion order (§20 of
    /// SYNTAX, ADR-0009). `frontmatter`/`headings` are not listed here: the
    /// renderer registers them itself, before these.
    pub exports: Vec<DataExport>,
}

/// The import lines a render needs (§11 rule 3).
///
/// `used_templates` holds the winning entries (indexes into
/// [`SolidRenderHints::templates`]); `used_markers` the text markers a rendered
/// text node contained. Everything else is gated out: an entry that lost — or
/// whose layer never occurs — contributes nothing.
pub fn normalize_imports(
    hints: Option<&SolidRenderHints>,
    used_templates: &BTreeSet<usize>,
    used_markers: &BTreeSet<String>,
) -> Vec<String> {
    let mut raw: BTreeSet<String> = BTreeSet::new();
    let mut structured: BTreeMap<String, (Option<String>, BTreeSet<String>)> = BTreeMap::new();

    if let Some(h) = hints {
        ingest_imports(&mut raw, &mut structured, &h.global_imports);

        for index in used_templates {
            if let Some(template) = h.templates.get(*index) {
                ingest_imports(&mut raw, &mut structured, &template.imports);
            }
        }

        for (marker, imports) in &h.text_imports {
            if !used_markers.contains(marker) {
                continue;
            }
            ingest_imports(&mut raw, &mut structured, imports);
        }
    }

    let mut lines: Vec<String> = Vec::new();
    for (module, (default, names)) in structured {
        if default.is_none() && names.is_empty() {
            continue;
        }
        let mut line = String::from("import ");
        let mut placed = false;
        if let Some(def) = default {
            line.push_str(&def);
            placed = true;
        }
        if !names.is_empty() {
            if placed {
                line.push_str(", ");
            }
            line.push('{');
            let mut first = true;
            for name in names {
                if !first {
                    line.push_str(", ");
                }
                line.push_str(&name);
                first = false;
            }
            line.push('}');
            placed = true;
        }
        if placed {
            line.push_str(" from \"");
            line.push_str(&module);
            line.push_str("\";");
            lines.push(line);
        }
    }

    for line in raw {
        lines.push(line);
    }

    lines
}

fn ingest_imports(
    raw: &mut BTreeSet<String>,
    structured: &mut BTreeMap<String, (Option<String>, BTreeSet<String>)>,
    imports: &[ImportEntry],
) {
    for imp in imports {
        match imp {
            ImportEntry::Raw(line) => {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    raw.insert(trimmed.to_string());
                }
            }
            ImportEntry::Structured {
                module,
                default,
                names,
            } => {
                let entry = structured
                    .entry(module.to_string())
                    .or_insert((None, BTreeSet::new()));
                if entry.0.is_none() {
                    entry.0 = default.clone();
                }
                for n in names {
                    if !n.trim().is_empty() {
                        entry.1.insert(n.to_string());
                    }
                }
            }
        }
    }
}
