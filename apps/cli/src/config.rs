use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ConfigTask {
    pub name: Option<String>,
    pub input: String,
    pub output: String,
    pub plugin: Option<String>,
    pub markdown_allow_html: Option<bool>,
    pub markdown_strip_comments: Option<bool>,
    pub wiki_link_prefix: Option<String>,
    pub format: String,
    pub pretty: Option<bool>,
    pub strict: Option<bool>,
    pub max_doc_bytes: Option<usize>,
    pub max_line_len: Option<usize>,
    pub max_blank_run: Option<usize>,
    pub cite: Option<CiteTaskConfig>,
    pub bind: Option<BindTaskConfig>,
    pub anchor: Option<AnchorTaskConfig>,
    pub heading: Option<HeadingTaskConfig>,
    pub image: Option<ImageTaskConfig>,
    pub table: Option<TableTaskConfig>,
    pub marker: Option<MarkerTaskConfig>,
    pub directive: Option<DirectiveTaskConfig>,
    pub list: Option<ListTaskConfig>,
    pub blockquote: Option<BlockquoteTaskConfig>,
    pub section: Option<SectionTaskConfig>,
    /// `[[task.number.custom]]`: the `%%numberType` group (ADR-0010).
    pub number: Option<NumberTaskConfig>,
    /// `true` bypasses the cache for **this** task only: it is never consulted
    /// and nothing is recorded for it, while the other tasks keep theirs.
    /// See `docs/rfc/no-cache-and-post-hook.md`.
    pub no_cache: Option<bool>,
    /// Shell command(s) run after this task rendered a file successfully.
    /// See `docs/rfc/no-cache-and-post-hook.md`.
    pub post_hook: Option<PostHook>,
    /// §13: keys this version does not know. They are a **load error** (not a
    /// warning): a typo must never change the output while the build succeeds.
    /// Collected here so `run_from_config` can report every one of them.
    #[serde(flatten)]
    pub unknown: std::collections::BTreeMap<String, toml::Value>,
}

/// A task's `post_hook`: one command, or an ordered list of commands.
///
/// Deserialized from either TOML shape:
///
/// ```toml
/// post_hook = "prettier --write ./out/[slug].jsx"
/// post_hook = ["prettier --write ./out/[slug].jsx", "eslint ./out"]
/// ```
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum PostHook {
    One(String),
    Many(Vec<String>),
}

impl PostHook {
    /// The commands in declared (execution) order.
    pub fn commands(&self) -> Vec<&str> {
        match self {
            Self::One(command) => vec![command.as_str()],
            Self::Many(commands) => commands.iter().map(String::as_str).collect(),
        }
    }
}

/// One `[[task.<element>.number]]` block (ADR-0010).
///
/// Both keys are optional: no `type` numbers every element of that element, no
/// `style` is `decimal` at every depth.
#[derive(Debug, Deserialize, Default, Clone)]
pub struct NumberRule {
    /// `type = "HeadingA"` / `type = ["HeadingA", "HeadingB"]` / `?` / `_` /
    /// `*` — §11 selectors, validated by `build_number_options`.
    #[serde(rename = "type")]
    pub types: Option<OneOrMany>,
    /// `style = "lower-alpha"` (every depth) or an array (one per depth).
    pub style: Option<OneOrMany>,
}

/// A config value that accepts one string or a list of strings.
#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum OneOrMany {
    One(String),
    Many(Vec<String>),
}

impl OneOrMany {
    /// The value as a list, in authored order.
    pub fn tokens(&self) -> Vec<&str> {
        match self {
            Self::One(token) => vec![token.as_str()],
            Self::Many(tokens) => tokens.iter().map(String::as_str).collect(),
        }
    }
}

/// One `[[task.<element>.extract]]` block (P.1 of
/// `docs/issues/plugin-extract.md`).
///
/// Every value is still raw: `build_extract_options` turns the rule into a
/// validated `ExtractBlock`, so an unknown format or a non-extractable element is
/// reported against the key the user wrote.
#[derive(Debug, Deserialize, Default, Clone)]
pub struct ExtractRule {
    /// `type = "CiteA"` / `type = ["CiteA", "CiteB"]` / `_` / `*` — §11
    /// selectors; absent means `_` (every type the element has).
    #[serde(rename = "type")]
    pub types: Option<OneOrMany>,
    /// One path or an array; `format` must have the same length.
    pub output: Option<OneOrMany>,
    /// `json`, `yaml` or `toml` — one per output.
    pub format: Option<OneOrMany>,
    /// Allowlist: keep only these fields.
    pub only: Option<OneOrMany>,
    /// Denylist: drop these fields. Always wins over `only`/`recommended`.
    pub omit: Option<OneOrMany>,
    /// Narrow the fields to the element's `recommended` preset first.
    #[serde(default)]
    pub recommended: bool,
    /// `export const <name>` (solid only); never an array.
    pub export: Option<String>,
}

/// `[task.number]`: the task-level numbering group.
#[derive(Debug, Deserialize, Default)]
pub struct NumberTaskConfig {
    #[serde(default)]
    pub custom: Vec<NumberRule>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ImageTaskConfig {
    /// `[[task.image.number]]` — numbering blocks (ADR-0010).
    #[serde(default)]
    pub number: Vec<NumberRule>,
    /// §11 layer component sets: `[task.img.custom.figure]`.
    pub custom: Option<toml::Value>,
    /// Removed pre-§11 key (`[task.img.custom_node]`, §14). The table is still
    /// parsed so a stale config fails with a migration message instead of
    /// silently dropping the custom component.
    pub custom_node: Option<toml::Value>,
    /// `[[task.image.extract]]` — data extraction blocks (plugin-extract).
    #[serde(default)]
    pub extract: Vec<ExtractRule>,
}

#[derive(Debug, Deserialize, Default)]
pub struct HeadingTaskConfig {
    /// Removed in favour of `[[task.heading.number]]` (ADR-0010, §14). Parsed as
    /// an opaque value so a stale config fails with a migration message instead
    /// of silently rendering an unnumbered document.
    pub auto_number: Option<toml::Value>,
    /// Removed with `auto_number` (§14).
    pub number_style: Option<toml::Value>,
    /// `[[task.heading.number]]` — numbering blocks (ADR-0010).
    #[serde(default)]
    pub number: Vec<NumberRule>,
    /// §11 layer component sets: `[task.heading.custom.heading]`.
    pub custom: Option<toml::Value>,
    /// Removed pre-§11 key (`[task.heading.custom_node]`, §14).
    pub custom_node: Option<toml::Value>,
    /// `[[task.heading.extract]]` — data extraction blocks (plugin-extract).
    #[serde(default)]
    pub extract: Vec<ExtractRule>,
}

#[derive(Debug, Deserialize, Default)]
pub struct AnchorTaskConfig {
    /// `[[task.anchor.number]]` — numbering blocks (ADR-0010).
    #[serde(default)]
    pub number: Vec<NumberRule>,
    /// §11 layer component sets: `[[task.anchor.custom]]`.
    pub custom: Option<toml::Value>,
    /// Removed pre-§11 key (`[task.anchor.custom_node]`, §14).
    pub custom_node: Option<toml::Value>,
    /// `[[task.anchor.extract]]` — data extraction blocks (plugin-extract).
    #[serde(default)]
    pub extract: Vec<ExtractRule>,
}

#[derive(Debug, Deserialize, Default)]
pub struct CiteTaskConfig {
    /// `[[task.cite.number]]` — numbering blocks (ADR-0010).
    #[serde(default)]
    pub number: Vec<NumberRule>,
    pub reference_source: Option<String>,
    pub reference_file: Option<String>,
    /// Prefix for the bibliography anchor `href`s (`ref_id_prefix`).
    pub ref_id_prefix: Option<String>,
    /// CSS class of the inline citation element (`citation_class`).
    pub citation_class: Option<String>,
    /// Prefix for the inline anchor ids (`citation_id_prefix`).
    pub citation_id_prefix: Option<String>,
    /// Name of the Solid/JS export holding the reference map.
    pub references_export: Option<String>,
    /// Name of the Solid/JS export holding the citation list.
    pub citations_export: Option<String>,
    /// §2.6 of RFC-0002: which references the export carries — `cited` or
    /// `all`. Unset keeps the source-dependent default.
    pub references_scope: Option<String>,
    /// Removed pre-RFC-0002 name (`prefix` → `ref_id_prefix`). Parsed so a
    /// stale config fails with a migration message instead of silently
    /// rendering with defaults.
    pub prefix: Option<toml::Value>,
    /// Removed pre-RFC-0002 name (`class` → `citation_class`).
    pub class: Option<toml::Value>,
    /// Removed pre-RFC-0002 name (`id_prefix` → `citation_id_prefix`).
    pub id_prefix: Option<toml::Value>,
    /// §11 layer component sets: `[[task.cite.custom]]` (primary layer `cite`).
    pub custom: Option<toml::Value>,
    /// Removed pre-§11 key (`[task.cite.custom_node]`, §14).
    pub custom_node: Option<toml::Value>,
    /// `[[task.cite.extract]]` — data extraction blocks (plugin-extract).
    #[serde(default)]
    pub extract: Vec<ExtractRule>,
}

#[derive(Debug, Deserialize, Default)]
pub struct TableTaskConfig {
    /// `[[task.table.number]]` — numbering blocks (ADR-0010).
    #[serde(default)]
    pub number: Vec<NumberRule>,
    /// §11 layer component sets: `[task.table.custom.table]`, `…custom.thead`, …
    pub custom: Option<toml::Value>,
    /// Removed pre-§11 key (`[task.table.custom_node]`, §14).
    pub custom_node: Option<toml::Value>,
    /// `[[task.table.extract]]` — data extraction blocks (plugin-extract).
    #[serde(default)]
    pub extract: Vec<ExtractRule>,
}

#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
pub struct PluginCustomSection {
    pub source: Option<Vec<String>>,
    pub order: Option<Vec<String>>,
    pub enable_unsafe_hooks: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
pub struct PluginVicadoSolidSection {
    pub imports: Option<Vec<toml::Value>>,
}

#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
pub struct PluginVicadoRendererSection {
    pub solid: Option<PluginVicadoSolidSection>,
}

#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
pub struct PluginVicadoSection {
    pub renderer: Option<PluginVicadoRendererSection>,
}

#[derive(Debug, Deserialize)]
pub struct PendonConfig {
    #[serde(rename = "task")]
    pub tasks: Vec<ConfigTask>,
    #[serde(rename = "plugin-custom")]
    pub plugin_custom: Option<PluginCustomSection>,
    #[serde(rename = "plugin-vicado")]
    pub plugin_vicado: Option<PluginVicadoSection>,
}

#[derive(Debug, Deserialize, Default)]
pub struct MarkerTaskConfig {
    /// `[[task.marker.number]]` — numbering blocks (ADR-0010).
    #[serde(default)]
    pub number: Vec<NumberRule>,
    /// §11 layer component sets: `[task.marker.custom.marker]` (the primary
    /// layer, so `[[task.marker.custom]]` addresses it too).
    pub custom: Option<toml::Value>,
    /// Removed pre-§11 key (`[task.marker.custom_node]`, §14).
    pub custom_node: Option<toml::Value>,
    /// `[[task.marker.extract]]` — data extraction blocks (plugin-extract).
    #[serde(default)]
    pub extract: Vec<ExtractRule>,
}

#[derive(Debug, Deserialize, Default)]
pub struct DirectiveTaskConfig {
    /// `[[task.directive.number]]` — numbering blocks (ADR-0010).
    #[serde(default)]
    pub number: Vec<NumberRule>,
    /// §11 layer component sets: `[task.directive.custom.directive]` (the primary
    /// layer, so `[[task.directive.custom]]` addresses it too).
    pub custom: Option<toml::Value>,
    /// Removed pre-§11 key (`[task.directive.custom_node]`, §14).
    pub custom_node: Option<toml::Value>,
    /// `[[task.directive.extract]]` — data extraction blocks (plugin-extract).
    #[serde(default)]
    pub extract: Vec<ExtractRule>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ListTaskConfig {
    /// `[[task.list.number]]` — numbering blocks (ADR-0010).
    #[serde(default)]
    pub number: Vec<NumberRule>,
    /// §11/§9.4 layer component sets: `[task.list.custom.list]` (the primary
    /// layer, the `<li>` item) plus `…custom.unordered` / `…custom.ordered`
    /// (the `<ul>` / `<ol>` containers). A bare `[[task.list.custom]]` addresses
    /// the primary layer.
    pub custom: Option<toml::Value>,
    /// Removed pre-§11 key (`[task.list.custom_node]`, §14).
    pub custom_node: Option<toml::Value>,
    /// `[[task.list.extract]]` — data extraction blocks (plugin-extract).
    #[serde(default)]
    pub extract: Vec<ExtractRule>,
}

#[derive(Debug, Deserialize, Default)]
pub struct BlockquoteTaskConfig {
    /// `[[task.blockquote.number]]` — numbering blocks (ADR-0010).
    #[serde(default)]
    pub number: Vec<NumberRule>,
    /// §11/§9.2 layer component set: `[task.blockquote.custom.blockquote]`
    /// (the primary layer, so `[[task.blockquote.custom]]` addresses it too).
    pub custom: Option<toml::Value>,
    /// Removed pre-§11 key (`[task.blockquote.custom_node]`, §14).
    pub custom_node: Option<toml::Value>,
    /// `[[task.blockquote.extract]]` — data extraction blocks (plugin-extract).
    #[serde(default)]
    pub extract: Vec<ExtractRule>,
}

#[derive(Debug, Deserialize, Default)]
pub struct SectionTaskConfig {
    /// `[[task.section.number]]` — numbering blocks (ADR-0010).
    #[serde(default)]
    pub number: Vec<NumberRule>,
    /// §11/§9.5 layer component set: `[task.section.custom.section]`
    /// (the primary layer, so `[[task.section.custom]]` addresses it too).
    pub custom: Option<toml::Value>,
    /// Removed pre-§11 key (`[task.section.custom_node]`, §14).
    pub custom_node: Option<toml::Value>,
    /// `[[task.section.extract]]` — data extraction blocks (plugin-extract).
    #[serde(default)]
    pub extract: Vec<ExtractRule>,
}

/// `[task.bind]`: the export side of `plugin-bind` (§6.1 of
/// `workflows/numbering-extract-type-export.md`).
///
/// The `{{{lang[var] … }}}` blocks carry their export names in the head; an
/// inject entry names a **file** instead, so a page can publish a payload it
/// never mentions in its Markdown.
#[derive(Debug, Deserialize, Default)]
pub struct BindTaskConfig {
    #[serde(default)]
    pub inject: Vec<BindInject>,
}

/// `[[task.bind.inject]]`: one data file published as `export const <name>`.
#[derive(Debug, Deserialize, Default)]
pub struct BindInject {
    /// The file, resolved like `input`/`output`: relative to `pendon.toml` and
    /// with the task's dynamic `[foo]`/`[bar]` segments substituted per page.
    pub source: String,
    /// The binding name; when absent the file's stem is used, so
    /// `./src/[foo]/rows.yaml` exports `rows` and `./data/settings.toml` exports
    /// `settings`.
    pub export: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_task(extra: &str) -> ConfigTask {
        toml::from_str(&format!(
            "input = \"./src/[slug].mdp\"\noutput = \"./out/[slug].jsx\"\nformat = \"solid\"\n{extra}"
        ))
        .expect("valid task config")
    }

    /// `no_cache` is optional and defaults to "cache enabled".
    #[test]
    fn no_cache_defaults_to_none() {
        assert!(parse_task("").no_cache.is_none());
        assert_eq!(parse_task("no_cache = true\n").no_cache, Some(true));
        assert_eq!(parse_task("no_cache = false\n").no_cache, Some(false));
    }

    /// `post_hook` accepts a single command string.
    #[test]
    fn post_hook_accepts_one_command() {
        let task = parse_task("post_hook = \"prettier --write ./out/[slug].jsx\"\n");
        let hooks = task.post_hook.expect("post_hook parsed");
        assert_eq!(hooks.commands(), vec!["prettier --write ./out/[slug].jsx"]);
    }

    /// `post_hook` accepts an array, and its order is preserved.
    #[test]
    fn post_hook_accepts_an_ordered_array() {
        let task = parse_task("post_hook = [\"one\", \"two\", \"three\"]\n");
        let hooks = task.post_hook.expect("post_hook parsed");
        assert_eq!(hooks.commands(), vec!["one", "two", "three"]);
    }

    /// Both new keys are known to the schema, so §13's unknown-key warning
    /// must stay silent for them.
    #[test]
    fn new_keys_are_not_reported_as_unknown() {
        let task = parse_task("no_cache = true\npost_hook = \"echo hi\"\n");
        assert!(
            task.unknown.is_empty(),
            "unexpected unknown keys: {:?}",
            task.unknown.keys().collect::<Vec<_>>()
        );
    }

    /// §6.1: `[[task.bind.inject]]` is a known section, so a typo inside it is
    /// reported as an unknown key while the entries themselves parse.
    #[test]
    fn bind_inject_entries_are_known_keys() {
        let task = parse_task(
            "[[bind.inject]]\nsource = \"./src/[foo]/[bar].yaml\"\nexport = \"fooData\"\n\n\
             [[bind.inject]]\nsource = \"../../dataX.toml\"\n",
        );
        assert!(
            task.unknown.is_empty(),
            "unexpected unknown keys: {:?}",
            task.unknown.keys().collect::<Vec<_>>()
        );
        let bind = task.bind.expect("bind section parsed");
        assert_eq!(bind.inject.len(), 2);
        assert_eq!(bind.inject[0].source, "./src/[foo]/[bar].yaml");
        assert_eq!(bind.inject[0].export.as_deref(), Some("fooData"));
        assert_eq!(bind.inject[1].source, "../../dataX.toml");
        assert!(bind.inject[1].export.is_none());
    }
}
