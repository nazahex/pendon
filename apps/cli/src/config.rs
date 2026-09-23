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
    pub heading: Option<pendon_plugin_heading::HeadingOptions>,
    pub img: Option<pendon_plugin_img::ImgOptions>,
    pub table: Option<TableTaskConfig>,
}

#[derive(Debug, Deserialize, Default)]
pub struct AnchorTaskConfig {
    pub custom_node: Option<AnchorCustomNodeConfig>,
}

#[derive(Debug, Deserialize, Default)]
pub struct AnchorCustomNodeConfig {
    pub name: Option<String>,
    pub template: Option<String>,
    pub imports: Option<Vec<toml::Value>>,
}

#[derive(Debug, Deserialize, Default)]
pub struct CiteTaskConfig {
    pub reference_source: Option<String>,
    pub reference_file: Option<String>,
    pub prefix: Option<String>,
    pub class: Option<String>,
    pub id_prefix: Option<String>,
    pub custom_node: Option<CiteCustomNodeConfig>,
    pub section: Option<CiteSectionConfig>,
}

#[derive(Debug, Deserialize, Default)]
pub struct CiteCustomNodeConfig {
    pub name: Option<String>,
    pub template: Option<String>,
    pub imports: Option<Vec<toml::Value>>,
}

#[derive(Debug, Deserialize, Default)]
pub struct CiteSectionConfig {
    pub marker: Option<String>,
    pub node: Option<String>,
    pub template: Option<String>,
    pub imports: Option<Vec<toml::Value>>,
}

#[derive(Debug, Deserialize, Default)]
pub struct TableTaskConfig {
    pub custom_node: Option<TableCustomNodeConfig>,
}

#[derive(Debug, Deserialize, Default)]
pub struct TableCustomNodeConfig {
    pub table: Option<TableComponentConfig>,
    pub caption: Option<TableComponentConfig>,
    pub row: Option<TableComponentConfig>,
    pub cell: Option<TableComponentConfig>,
}

#[derive(Debug, Deserialize, Default)]
pub struct TableComponentConfig {
    pub name: Option<String>,
    pub template: Option<String>,
    pub imports: Option<Vec<toml::Value>>,
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
