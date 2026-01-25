use pendon_core::Event;
use pendon_renderer_ast::render_ast_to_string;
use serde_json::{self, Map, Value};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
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
    pub node_type: String,
    pub node_name: Option<String>,
    pub template: String,
}

#[derive(Clone, Debug, Default)]
pub struct SolidRenderHints {
    pub imports: Vec<ImportEntry>,
    pub templates: Vec<ComponentTemplate>,
}

// Render events → AST JSON → Solid TSX string, optionally guided by renderer hints
pub fn render_solid(events: &[Event]) -> String {
    render_solid_with_hints(events, None)
}

pub fn render_solid_with_hints(events: &[Event], hints: Option<&SolidRenderHints>) -> String {
    let ast_json = render_ast_to_string(events).expect("AST serialization failed");
    let v: serde_json::Value = serde_json::from_str(&ast_json).expect("Invalid AST JSON");
    let frontmatter = extract_frontmatter(&v);
    let mut body = String::new();
    render_node(&v, &mut body, hints);

    let mut out = String::new();
    let import_lines = normalize_imports(hints);
    for line in import_lines {
        out.push_str(&line);
        out.push('\n');
    }
    if !out.is_empty() {
        out.push('\n');
    }
    if let Some(fm) = frontmatter {
        out.push_str("export const frontmatter = ");
        out.push_str(&fm);
        out.push_str(";\n");
    }
    out.push_str("export default function PendonView() { return (<>");
    out.push('\n');
    out.push_str(&body);
    out.push_str("\n</>); }\n");
    out
}

fn normalize_imports(hints: Option<&SolidRenderHints>) -> Vec<String> {
    let mut raw: BTreeSet<String> = BTreeSet::new();
    let mut structured: BTreeMap<String, (Option<String>, BTreeSet<String>)> = BTreeMap::new();

    if let Some(h) = hints {
        for imp in &h.imports {
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

fn extract_frontmatter(v: &serde_json::Value) -> Option<String> {
    if v.get("type")?.as_str()? != "Document" {
        return None;
    }
    let children = v.get("children")?.as_array()?;
    for ch in children {
        if ch.get("type").and_then(|t: &serde_json::Value| t.as_str()) == Some("Frontmatter") {
            if let Some(attrs) = ch
                .get("attrs")
                .and_then(|a: &serde_json::Value| a.as_object())
            {
                if let Some(data) = attrs
                    .get("data")
                    .and_then(|d: &serde_json::Value| d.as_str())
                {
                    return Some(data.to_string());
                }
            }
        }
    }
    None
}

fn render_node(v: &serde_json::Value, out: &mut String, hints: Option<&SolidRenderHints>) {
    if let Some(kind) = v.get("type").and_then(|t: &serde_json::Value| t.as_str()) {
        if let Some(template) = select_template(hints, kind, v) {
            let children = render_children_to_string(v, hints);
            let attrs = v.get("attrs").and_then(|a| a.as_object());
            let text = v.get("text").and_then(|t| t.as_str());
            out.push_str(&render_template(template, attrs, &children, text));
            return;
        }

        match kind {
            "Document" => {
                if let Some(children) = v
                    .get("children")
                    .and_then(|c: &serde_json::Value| c.as_array())
                {
                    for ch in children {
                        render_node(ch, out, hints);
                    }
                }
            }
            "Frontmatter" => {
                // Metadata only; skip emitting markup.
            }
            "Paragraph" => {
                out.push_str("<p>");
                render_children(v, out, hints);
                out.push_str("</p>\n");
            }
            "Blockquote" => {
                out.push_str("<blockquote>\n");
                render_children(v, out, hints);
                out.push_str("</blockquote>\n");
            }
            "Heading" => {
                let level = v
                    .get("attrs")
                    .and_then(|a: &serde_json::Value| a.get("level"))
                    .and_then(|l: &serde_json::Value| l.as_str())
                    .unwrap_or("1");
                out.push('<');
                out.push('h');
                out.push_str(level);
                out.push('>');
                render_text_or_children(v, out, hints);
                out.push_str("</h");
                out.push_str(level);
                out.push_str(">\n");
            }
            "Section" => {
                out.push_str("<section");
                if let Some(id) = v
                    .get("attrs")
                    .and_then(|a: &serde_json::Value| a.get("id"))
                    .and_then(|l: &serde_json::Value| l.as_str())
                {
                    out.push_str(" id=\"");
                    escape_jsx(id, out);
                    out.push_str("\"");
                }
                out.push_str(">\n");
                render_children(v, out, hints);
                out.push_str("</section>\n");
            }
            "ThematicBreak" => {
                out.push_str("<hr />\n");
            }
            "CodeFence" => {
                out.push_str("<pre><code>");
                let raw = v
                    .get("attrs")
                    .and_then(|a: &serde_json::Value| a.get("raw_html"))
                    .and_then(|x: &serde_json::Value| x.as_str());
                if let Some(text) = v.get("text").and_then(|t: &serde_json::Value| t.as_str()) {
                    if matches!(raw, Some("1")) {
                        out.push_str(text);
                    } else {
                        escape_jsx(text, out);
                    }
                }
                out.push_str("</code></pre>\n");
            }
            "BulletList" => {
                out.push_str("<ul>\n");
                render_children(v, out, hints);
                out.push_str("</ul>\n");
            }
            "OrderedList" => {
                out.push_str("<ol");
                if let Some(attrs) = v.get("attrs") {
                    if let Some(start) = attrs
                        .get("start")
                        .and_then(|s: &serde_json::Value| s.as_str())
                    {
                        out.push_str(" start={");
                        out.push_str(start);
                        out.push_str("}");
                    }
                }
                out.push_str(">\n");
                render_children(v, out, hints);
                out.push_str("</ol>\n");
            }
            "ListItem" => {
                out.push_str("<li>");
                if let Some(text) = v.get("text").and_then(|t: &serde_json::Value| t.as_str()) {
                    escape_jsx(text, out);
                }
                render_children(v, out, hints);
                out.push_str("</li>\n");
            }
            "Table" => {
                out.push_str("<table>\n");
                render_children(v, out, hints);
                out.push_str("</table>\n");
            }
            "TableHead" => {
                out.push_str("<thead>\n");
                render_children(v, out, hints);
                out.push_str("</thead>\n");
            }
            "TableBody" => {
                out.push_str("<tbody>\n");
                render_children(v, out, hints);
                out.push_str("</tbody>\n");
            }
            "TableRow" => {
                out.push_str("<tr>\n");
                render_children(v, out, hints);
                out.push_str("</tr>\n");
            }
            "TableCell" => {
                let is_header = v
                    .get("attrs")
                    .and_then(|a: &serde_json::Value| a.get("header"))
                    .and_then(|h| h.as_str())
                    .map(|h| h == "1")
                    .unwrap_or(false);
                let tag = if is_header { "th" } else { "td" };
                out.push('<');
                out.push_str(tag);
                out.push('>');
                render_children(v, out, hints);
                out.push_str("</");
                out.push_str(tag);
                out.push_str(">\n");
            }
            "Emphasis" => {
                out.push_str("<em>");
                render_children(v, out, hints);
                out.push_str("</em>");
            }
            "Strong" => {
                out.push_str("<strong>");
                render_children(v, out, hints);
                out.push_str("</strong>");
            }
            "Bold" => {
                out.push_str("<b>");
                render_children(v, out, hints);
                out.push_str("</b>");
            }
            "Italic" => {
                out.push_str("<i>");
                render_children(v, out, hints);
                out.push_str("</i>");
            }
            "InlineCode" => {
                out.push_str("<code>");
                render_children(v, out, hints);
                out.push_str("</code>");
            }
            "Link" => {
                out.push_str("<a");
                if let Some(attrs) = v.get("attrs") {
                    if let Some(href) = attrs
                        .get("href")
                        .and_then(|h: &serde_json::Value| h.as_str())
                    {
                        out.push_str(" href=\"");
                        escape_jsx(href, out);
                        out.push_str("\"");
                    }
                }
                out.push('>');
                render_children(v, out, hints);
                out.push_str("</a>");
            }
            "Text" => {
                if let Some(text) = v.get("text").and_then(|t: &serde_json::Value| t.as_str()) {
                    escape_jsx(text, out);
                }
            }
            _ => {
                render_children(v, out, hints);
            }
        }
    }
}

fn render_children(v: &serde_json::Value, out: &mut String, hints: Option<&SolidRenderHints>) {
    if let Some(children) = v
        .get("children")
        .and_then(|c: &serde_json::Value| c.as_array())
    {
        for ch in children {
            render_node(ch, out, hints);
        }
    }
}

fn render_children_to_string(v: &serde_json::Value, hints: Option<&SolidRenderHints>) -> String {
    let mut buf = String::new();
    if let Some(children) = v
        .get("children")
        .and_then(|c: &serde_json::Value| c.as_array())
    {
        for ch in children {
            render_node(ch, &mut buf, hints);
        }
    }
    buf
}

fn render_text_or_children(
    v: &serde_json::Value,
    out: &mut String,
    hints: Option<&SolidRenderHints>,
) {
    if let Some(text) = v.get("text").and_then(|t: &serde_json::Value| t.as_str()) {
        escape_jsx(text, out);
    } else {
        render_children(v, out, hints);
    }
}

fn escape_jsx(s: &str, out: &mut String) {
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
}

fn select_template<'a>(
    hints: Option<&'a SolidRenderHints>,
    node_type: &str,
    v: &serde_json::Value,
) -> Option<&'a ComponentTemplate> {
    let Some(h) = hints else {
        return None;
    };
    let node_name = v
        .get("attrs")
        .and_then(|a: &serde_json::Value| a.get("name"))
        .and_then(|n| n.as_str());

    let mut fallback: Option<&ComponentTemplate> = None;
    for tpl in &h.templates {
        if tpl.node_type != node_type {
            continue;
        }
        if let Some(expected) = tpl.node_name.as_deref() {
            if Some(expected) == node_name {
                return Some(tpl);
            }
        } else if fallback.is_none() {
            fallback = Some(tpl);
        }
    }
    fallback
}

fn render_template(
    template: &ComponentTemplate,
    attrs: Option<&Map<String, Value>>,
    children: &str,
    text: Option<&str>,
) -> String {
    let mut out = String::new();
    let tpl = template.template.as_str();
    let bytes = tpl.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            if tpl[i..].starts_with("{children}") {
                out.push_str(children);
                i += "{children}".len();
                continue;
            }
            if tpl[i..].starts_with("{text}") {
                out.push_str(text.unwrap_or(""));
                i += "{text}".len();
                continue;
            }
            if tpl[i..].starts_with("{attrs.") {
                if let Some(end) = tpl[i + 7..].find('}') {
                    let key = &tpl[i + 7..i + 7 + end];
                    let val = get_attr_value(attrs, key);
                    out.push_str(&val);
                    i = i + 7 + end + 1;
                    continue;
                }
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

fn get_attr_value(attrs: Option<&Map<String, Value>>, key: &str) -> String {
    let Some(map) = attrs else {
        return String::new();
    };
    match map.get(key) {
        Some(Value::String(s)) => s.clone(),
        Some(v) => v.to_string(),
        None => String::new(),
    }
}
