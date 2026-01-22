use pendon_core::Event;
use pendon_renderer_ast::render_ast_to_string;
use serde_json;

// Render events → AST JSON → Solid TSX string
// Pure function component, static render, no props used.
pub fn render_solid(events: &[Event]) -> String {
    let ast_json = render_ast_to_string(events).expect("AST serialization failed");
    let v: serde_json::Value = serde_json::from_str(&ast_json).expect("Invalid AST JSON");
    let frontmatter = extract_frontmatter(&v);
    let mut body = String::new();
    render_node(&v, &mut body);
    let mut out = String::new();
    if let Some(fm) = frontmatter {
        out.push_str("export const frontmatter = ");
        out.push_str(&fm);
        out.push_str(";\n");
    }
    out.push_str("export default function PendonView() { return (<>\n");
    out.push_str(&body);
    out.push_str("\n</>); }\n");
    out
}

fn extract_frontmatter(v: &serde_json::Value) -> Option<String> {
    if v.get("type")?.as_str()? != "Document" {
        return None;
    }
    let children = v.get("children")?.as_array()?;
    for ch in children {
        if ch.get("type").and_then(|t| t.as_str()) == Some("Frontmatter") {
            if let Some(attrs) = ch.get("attrs").and_then(|a| a.as_object()) {
                if let Some(data) = attrs.get("data").and_then(|d| d.as_str()) {
                    return Some(data.to_string());
                }
            }
        }
    }
    None
}

fn render_node(v: &serde_json::Value, out: &mut String) {
    if let Some(kind) = v.get("type").and_then(|t: &serde_json::Value| t.as_str()) {
        match kind {
            "Document" => {
                if let Some(children) = v
                    .get("children")
                    .and_then(|c: &serde_json::Value| c.as_array())
                {
                    for ch in children {
                        render_node(ch, out);
                    }
                }
            }
            "Frontmatter" => {
                // Metadata only; skip emitting markup.
            }
            "Paragraph" => {
                out.push_str("<p>");
                render_children(v, out);
                out.push_str("</p>\n");
            }
            "Blockquote" => {
                out.push_str("<blockquote>\n");
                render_children(v, out);
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
                render_text_or_children(v, out);
                out.push_str("</h");
                out.push_str(level);
                out.push_str(">\n");
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
                        // insert as raw HTML fragment inside code
                        out.push_str(text);
                    } else {
                        escape_jsx(text, out);
                    }
                }
                out.push_str("</code></pre>\n");
            }
            "BulletList" => {
                out.push_str("<ul>\n");
                render_children(v, out);
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
                render_children(v, out);
                out.push_str("</ol>\n");
            }
            "ListItem" => {
                out.push_str("<li>");
                if let Some(text) = v.get("text").and_then(|t: &serde_json::Value| t.as_str()) {
                    escape_jsx(text, out);
                }
                render_children(v, out);
                out.push_str("</li>\n");
            }
            "Table" => {
                out.push_str("<table>\n");
                render_children(v, out);
                out.push_str("</table>\n");
            }
            "TableHead" => {
                out.push_str("<thead>\n");
                render_children(v, out);
                out.push_str("</thead>\n");
            }
            "TableBody" => {
                out.push_str("<tbody>\n");
                render_children(v, out);
                out.push_str("</tbody>\n");
            }
            "TableRow" => {
                out.push_str("<tr>\n");
                render_children(v, out);
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
                render_children(v, out);
                out.push_str("</");
                out.push_str(tag);
                out.push_str(">\n");
            }
            "Emphasis" => {
                out.push_str("<em>");
                render_children(v, out);
                out.push_str("</em>");
            }
            "Strong" => {
                out.push_str("<strong>");
                render_children(v, out);
                out.push_str("</strong>");
            }
            "Bold" => {
                out.push_str("<b>");
                render_children(v, out);
                out.push_str("</b>");
            }
            "Italic" => {
                out.push_str("<i>");
                render_children(v, out);
                out.push_str("</i>");
            }
            "InlineCode" => {
                out.push_str("<code>");
                render_children(v, out);
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
                render_children(v, out);
                out.push_str("</a>");
            }
            "Text" => {
                if let Some(text) = v.get("text").and_then(|t: &serde_json::Value| t.as_str()) {
                    escape_jsx(text, out);
                }
            }
            _ => {}
        }
    }
}

fn render_children(v: &serde_json::Value, out: &mut String) {
    if let Some(children) = v
        .get("children")
        .and_then(|c: &serde_json::Value| c.as_array())
    {
        for ch in children {
            render_node(ch, out);
        }
    }
}

fn render_text_or_children(v: &serde_json::Value, out: &mut String) {
    if let Some(text) = v.get("text").and_then(|t: &serde_json::Value| t.as_str()) {
        escape_jsx(text, out);
    } else {
        render_children(v, out);
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
