use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AttrType {
    String,
    Int,
    Bool,
    ListString,
}

impl AttrType {
    pub(crate) fn parse(&self, raw: &str) -> Option<String> {
        let trimmed = raw.trim();
        match self {
            AttrType::String => Some(strip_quotes(trimmed)),
            AttrType::Int => trimmed.parse::<i64>().ok().map(|v| v.to_string()),
            AttrType::Bool => match trimmed {
                "true" => Some("true".to_string()),
                "false" => Some("false".to_string()),
                _ => None,
            },
            AttrType::ListString => Some(parse_list_string(trimmed)),
        }
    }
}

impl From<&str> for AttrType {
    fn from(s: &str) -> Self {
        match s {
            "int" => AttrType::Int,
            "bool" => AttrType::Bool,
            "list<string>" => AttrType::ListString,
            _ => AttrType::String,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AttrSpec {
    pub name: String,
    #[serde(default = "default_attr_type")]
    pub r#type: String,
    #[serde(default)]
    pub required: bool,
    pub default: Option<String>,
}

fn default_attr_type() -> String {
    "string".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct MatcherSpec {
    pub start: Option<String>,
    pub start_regex: Option<String>,
    pub end: Option<String>,
    pub inline_marker: Option<String>,
    pub capture: Option<String>,
    pub parse_hint: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AstSpec {
    pub node: Option<String>,
    pub node_name: Option<String>,
    pub attrs_map: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SolidImportEntry {
    pub module: Option<String>,
    pub default: Option<String>,
    pub names: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SolidRendererSpec {
    #[serde(default)]
    pub imports: Vec<toml::Value>,
    pub component_template: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RendererSpec {
    pub solid: Option<SolidRendererSpec>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PluginSpec {
    pub name: String,
    pub kind: String,
    pub matcher: MatcherSpec,
    #[serde(default)]
    pub attrs: Vec<AttrSpec>,
    pub ast: Option<AstSpec>,
    pub renderer: Option<RendererSpec>,
    pub meta: Option<BTreeMap<String, toml::Value>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PluginIndexEntry {
    pub id: String,
    pub path: Option<String>,
    pub inline: Option<PluginSpec>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(rename = "override")]
    pub override_: Option<bool>,
    pub props: Option<BTreeMap<String, toml::Value>>,
}

fn default_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
pub struct PluginIndexFile {
    #[serde(rename = "plugin")]
    pub plugins: Vec<PluginIndexEntry>,
}

#[derive(Debug, Clone)]
pub struct IndexedPlugin {
    pub id: String,
    pub spec: PluginSpec,
}

pub(crate) fn strip_quotes(raw: &str) -> String {
    let trimmed = raw.trim();
    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        trimmed[1..trimmed.len().saturating_sub(1)].to_string()
    } else {
        trimmed.to_string()
    }
}

pub(crate) fn parse_list_string(raw: &str) -> String {
    let inner = raw
        .trim()
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .unwrap_or(raw)
        .trim();
    if inner.is_empty() {
        return "[]".to_string();
    }
    let mut items: Vec<String> = Vec::new();
    let mut buf = String::new();
    let mut in_quote = false;
    let mut escape = false;
    for ch in inner.chars() {
        if escape {
            buf.push(ch);
            escape = false;
            continue;
        }
        if ch == '\\' {
            escape = true;
            continue;
        }
        if ch == '\'' || ch == '"' {
            in_quote = !in_quote;
            continue;
        }
        if ch == ',' && !in_quote {
            if !buf.trim().is_empty() {
                items.push(strip_quotes(buf.trim()));
            }
            buf.clear();
            continue;
        }
        buf.push(ch);
    }
    if !buf.trim().is_empty() {
        items.push(strip_quotes(buf.trim()));
    }
    let rendered: Vec<String> = items.into_iter().map(|v| format!("\"{}\"", v)).collect();
    format!("[{}]", rendered.join(", "))
}
