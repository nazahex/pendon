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
    pub anchor: Option<AnchorTaskConfig>,
    pub heading: Option<HeadingTaskConfig>,
    pub img: Option<ImgTaskConfig>,
    pub table: Option<TableTaskConfig>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ImgTaskConfig {
    /// §11 layer component sets: `[task.img.custom.figure]`.
    pub custom: Option<toml::Value>,
    /// Removed pre-§11 key (`[task.img.custom_node]`, §14). The table is still
    /// parsed so a stale config fails with a migration message instead of
    /// silently dropping the custom component.
    pub custom_node: Option<toml::Value>,
}

#[derive(Debug, Deserialize, Default)]
pub struct HeadingTaskConfig {
    pub auto_number: Option<bool>,
    pub number_style: Option<pendon_plugin_heading::NumberStyle>,
    /// §11 layer component sets: `[task.heading.custom.heading]`.
    pub custom: Option<toml::Value>,
    /// Removed pre-§11 key (`[task.heading.custom_node]`, §14).
    pub custom_node: Option<toml::Value>,
}

#[derive(Debug, Deserialize, Default)]
pub struct AnchorTaskConfig {
    /// §11 layer component sets: `[task.anchor.custom]`.
    pub custom: Option<toml::Value>,
    /// Removed pre-§11 key (`[task.anchor.custom_node]`, §14).
    pub custom_node: Option<toml::Value>,
}

#[derive(Debug, Deserialize, Default)]
pub struct CiteTaskConfig {
    pub reference_source: Option<String>,
    pub reference_file: Option<String>,
    pub prefix: Option<String>,
    pub class: Option<String>,
    pub id_prefix: Option<String>,
    /// §11 layer component sets: `[task.cite.custom]` (primary layer `cite`).
    pub custom: Option<toml::Value>,
    /// Removed pre-§11 key (`[task.cite.custom_node]`, §14).
    pub custom_node: Option<toml::Value>,
}

#[derive(Debug, Deserialize, Default)]
pub struct TableTaskConfig {
    /// §11 layer component sets: `[task.table.custom.table]`, `…custom.thead`, …
    pub custom: Option<toml::Value>,
    /// Removed pre-§11 key (`[task.table.custom_node]`, §14).
    pub custom_node: Option<toml::Value>,
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
