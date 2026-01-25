use pendon_core::{Event, NodeKind, Severity};
use regex::Regex;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AttrType {
    String,
    Int,
    Bool,
    ListString,
}

impl AttrType {
    fn parse(&self, raw: &str) -> Option<String> {
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

pub fn load_spec_from_path<P: AsRef<Path>>(path: P) -> Result<PluginSpec, String> {
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("cannot read plugin spec {}: {}", path.as_ref().display(), e))?;
    toml::from_str(&text)
        .map_err(|e| format!("invalid plugin spec {}: {}", path.as_ref().display(), e))
}

pub fn load_index_from_path<P: AsRef<Path>>(path: P) -> Result<Vec<IndexedPlugin>, String> {
    let text = std::fs::read_to_string(&path).map_err(|e| {
        format!(
            "cannot read plugin index {}: {}",
            path.as_ref().display(),
            e
        )
    })?;
    let idx: PluginIndexFile = toml::from_str(&text)
        .map_err(|e| format!("invalid plugin index {}: {}", path.as_ref().display(), e))?;
    let base = path
        .as_ref()
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(PathBuf::new);
    let mut out: Vec<IndexedPlugin> = Vec::new();
    for entry in idx.plugins.into_iter().filter(|p| p.enabled) {
        if let Some(spec) = entry.inline {
            out.push(IndexedPlugin { id: entry.id, spec });
            continue;
        }
        if let Some(rel) = entry.path {
            let resolved = base.join(rel);
            let spec = load_spec_from_path(&resolved)?;
            out.push(IndexedPlugin { id: entry.id, spec });
        }
    }
    Ok(out)
}

pub fn process(events: &[Event], spec: &PluginSpec) -> Vec<Event> {
    let Some(detector) = build_start_detector(spec) else {
        return events.to_vec();
    };
    let end_marker = spec
        .matcher
        .end
        .as_deref()
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| ":::".to_string());

    let mut out: Vec<Event> = Vec::with_capacity(events.len() + 4);
    let mut active: Option<ActiveBlock> = None;

    for ev in events.iter() {
        if active.is_none() {
            if let Event::Text(line) = ev {
                if detector.is_match(line.trim()) {
                    let captures = detector.captures(line.trim());
                    let (attrs, diags) = collect_attrs(spec, captures.as_ref());
                    for d in diags {
                        out.push(d);
                    }
                    let mut block = ActiveBlock::new(spec.clone(), attrs);
                    if matches!(out.last(), Some(Event::StartNode(NodeKind::Paragraph))) {
                        out.pop();
                        block.skip_para_close = true;
                    }
                    active = Some(block);
                    continue;
                }
            }
            out.push(ev.clone());
            continue;
        }

        if let Some(block) = active.as_mut() {
            if let Event::Text(line) = ev {
                if line.trim() == end_marker {
                    if let Some(idx) = block.last_para_start.take() {
                        block.inner.truncate(idx);
                        block.skip_para_close = true;
                    }
                    let mut flushed = active.take().unwrap().finish();
                    out.append(&mut flushed);
                    continue;
                }
            }

            match ev {
                Event::Text(line) => {
                    block.inner.push(Event::Text(line.clone()));
                }
                Event::StartNode(NodeKind::Paragraph) => {
                    block.last_para_start = Some(block.inner.len());
                    block.inner.push(ev.clone());
                }
                Event::EndNode(NodeKind::Paragraph) => {
                    if block.skip_para_close {
                        block.skip_para_close = false;
                    } else {
                        block.inner.push(ev.clone());
                    }
                }
                _ => block.inner.push(ev.clone()),
            }
        }
    }

    if let Some(block) = active {
        let mut flushed = block.finish();
        out.append(&mut flushed);
    }

    out
}

#[derive(Debug, Clone)]
struct ActiveBlock {
    spec: PluginSpec,
    attrs: BTreeMap<String, String>,
    inner: Vec<Event>,
    skip_para_close: bool,
    last_para_start: Option<usize>,
}

impl ActiveBlock {
    fn new(spec: PluginSpec, attrs: BTreeMap<String, String>) -> Self {
        ActiveBlock {
            spec,
            attrs,
            inner: Vec::new(),
            skip_para_close: false,
            last_para_start: None,
        }
    }

    fn finish(self) -> Vec<Event> {
        let mut out = Vec::with_capacity(self.inner.len() + 4);
        let nk = resolve_node_kind(&self.spec);
        out.push(Event::StartNode(nk.clone()));
        if let Some(ast) = &self.spec.ast {
            if let Some(name) = &ast.node_name {
                out.push(Event::Attribute {
                    name: "name".to_string(),
                    value: name.clone(),
                });
            }
            if let Some(map) = &ast.attrs_map {
                for (from, to) in map.iter() {
                    if let Some(val) = self.attrs.get(from) {
                        out.push(Event::Attribute {
                            name: to.clone(),
                            value: val.clone(),
                        });
                    }
                }
            }
        }
        out.extend(self.inner.into_iter());
        out.push(Event::EndNode(nk));
        out
    }
}

fn resolve_node_kind(spec: &PluginSpec) -> NodeKind {
    if let Some(ast) = &spec.ast {
        if let Some(node) = ast.node.as_deref() {
            return match node {
                "Document" => NodeKind::Document,
                "Frontmatter" => NodeKind::Frontmatter,
                "Paragraph" => NodeKind::Paragraph,
                "Blockquote" => NodeKind::Blockquote,
                "CodeFence" => NodeKind::CodeFence,
                "Heading" => NodeKind::Heading,
                "ThematicBreak" => NodeKind::ThematicBreak,
                "BulletList" => NodeKind::BulletList,
                "OrderedList" => NodeKind::OrderedList,
                "ListItem" => NodeKind::ListItem,
                "Table" => NodeKind::Table,
                "TableHead" => NodeKind::TableHead,
                "TableBody" => NodeKind::TableBody,
                "TableRow" => NodeKind::TableRow,
                "TableCell" => NodeKind::TableCell,
                "Section" => NodeKind::Section,
                "Emphasis" => NodeKind::Emphasis,
                "Strong" => NodeKind::Strong,
                "InlineCode" => NodeKind::InlineCode,
                "Link" => NodeKind::Link,
                "Bold" => NodeKind::Bold,
                "Italic" => NodeKind::Italic,
                other => NodeKind::Custom(other.to_string()),
            };
        }
    }
    NodeKind::Custom(spec.name.clone())
}

fn build_start_detector(spec: &PluginSpec) -> Option<Regex> {
    if let Some(re) = &spec.matcher.start_regex {
        Regex::new(re).ok()
    } else if let Some(lit) = &spec.matcher.start {
        let escaped = regex::escape(lit.trim());
        Regex::new(&format!("^{}$", escaped)).ok()
    } else {
        None
    }
}

fn collect_attrs(
    spec: &PluginSpec,
    caps: Option<&regex::Captures>,
) -> (BTreeMap<String, String>, Vec<Event>) {
    let mut out: BTreeMap<String, String> = BTreeMap::new();
    let mut diags: Vec<Event> = Vec::new();

    let kv_blob = caps
        .and_then(|c| c.name("kv"))
        .map(|m| m.as_str().to_string())
        .unwrap_or_default();
    let kv_map = if kv_blob.is_empty() {
        BTreeMap::new()
    } else {
        parse_keyvals(&kv_blob)
    };

    for attr in &spec.attrs {
        let attr_type: AttrType = attr.r#type.as_str().into();
        let mut value: Option<String> = None;
        if let Some(caps) = caps {
            if let Some(m) = caps.name(&attr.name) {
                value = Some(m.as_str().to_string());
            }
        }
        if value.is_none() {
            if let Some(val) = kv_map.get(&attr.name) {
                value = Some(val.clone());
            }
        }
        if value.is_none() {
            if let Some(def) = &attr.default {
                value = Some(def.clone());
            }
        }
        match value {
            Some(v) => match attr_type.parse(&v) {
                Some(parsed) => {
                    out.insert(attr.name.clone(), parsed);
                }
                None => {
                    diags.push(Event::Diagnostic {
                        severity: Severity::Error,
                        message: format!(
                            "[plugin-custom:{}] attribute '{}' failed to parse as {}",
                            spec.name, attr.name, attr.r#type
                        ),
                        span: None,
                    });
                }
            },
            None => {
                if attr.required {
                    diags.push(Event::Diagnostic {
                        severity: Severity::Error,
                        message: format!(
                            "[plugin-custom:{}] missing required attribute '{}'",
                            spec.name, attr.name
                        ),
                        span: None,
                    });
                }
            }
        }
    }

    (out, diags)
}

fn strip_quotes(raw: &str) -> String {
    let trimmed = raw.trim();
    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        trimmed[1..trimmed.len().saturating_sub(1)].to_string()
    } else {
        trimmed.to_string()
    }
}

fn parse_list_string(raw: &str) -> String {
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
        if ch == '"' {
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

fn parse_keyvals(raw: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    let trimmed = raw
        .trim()
        .trim_start_matches('{')
        .trim_end_matches('}')
        .trim();
    if trimmed.is_empty() {
        return map;
    }
    let mut buf = String::new();
    let mut in_quote = false;
    let mut escape = false;
    for ch in trimmed.chars() {
        if escape {
            buf.push(ch);
            escape = false;
            continue;
        }
        if ch == '\\' {
            escape = true;
            continue;
        }
        if ch == '"' {
            in_quote = !in_quote;
            buf.push(ch);
            continue;
        }
        if ch == ',' && !in_quote {
            ingest_kv(&mut map, &buf);
            buf.clear();
            continue;
        }
        buf.push(ch);
    }
    if !buf.is_empty() {
        ingest_kv(&mut map, &buf);
    }
    map
}

fn ingest_kv(map: &mut BTreeMap<String, String>, raw: &str) {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return;
    }
    let (key, val) = match trimmed.split_once(':') {
        Some(pair) => pair,
        None => return,
    };
    map.insert(key.trim().to_string(), val.trim().to_string());
}
