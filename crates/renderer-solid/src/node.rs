// node.rs
use serde_json::Value;

use crate::{imports::SolidRenderHints, template};

pub fn render_node(v: &Value, out: &mut String, hints: Option<&SolidRenderHints>) {
    if let Some(kind) = v.get("type").and_then(|t| t.as_str()) {
        if let Some(template) = template::select_template(hints, kind, v) {
            let children = render_children_to_string(v, hints);
            let attrs = v.get("attrs").and_then(|a| a.as_object());
            let text = v.get("text").and_then(|t| t.as_str());
            out.push_str(&template::render_template(template, attrs, &children, text));
            return;
        }

        match kind {
            "Document" => {
                if let Some(children) = v.get("children").and_then(|c| c.as_array()) {
                    for ch in children {
                        render_node(ch, out, hints);
                    }
                }
            }
            "Frontmatter" | "Headings" => {
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
                    .and_then(|a| a.get("level"))
                    .and_then(|l| l.as_str())
                    .unwrap_or("1");
                out.push('<');
                out.push('h');
                out.push_str(level);
                render_attrs(v, out, &["level"]);
                out.push('>');
                render_text_or_children(v, out, hints);
                out.push_str("</h");
                out.push_str(level);
                out.push_str(">\n");
            }
            "Section" => {
                out.push_str("<section");
                render_attrs(v, out, &[]);
                out.push_str(">\n");
                render_children(v, out, hints);
                out.push_str("</section>\n");
            }
            "ThematicBreak" => {
                out.push_str("<hr />\n");
            }
            "CodeFence" => {
                out.push_str("<pre");
                render_attrs(v, out, &["raw_html"]);
                let raw = v
                    .get("attrs")
                    .and_then(|a| a.get("raw_html"))
                    .and_then(|x| x.as_str());
                if matches!(raw, Some("1")) {
                    let text = v.get("text").and_then(|t| t.as_str()).unwrap_or("");
                    out.push_str("><code innerHTML={");
                    out.push_str(&json_string_literal(text));
                    out.push_str("} /></pre>\n");
                } else {
                    out.push_str("><code>");
                    if let Some(text) = v.get("text").and_then(|t| t.as_str()) {
                        escape_jsx(text, out);
                    }
                    out.push_str("</code></pre>\n");
                }
            }
            "BulletList" => {
                out.push_str("<ul>\n");
                render_children(v, out, hints);
                out.push_str("</ul>\n");
            }
            "OrderedList" => {
                out.push_str("<ol");
                if let Some(attrs) = v.get("attrs") {
                    if let Some(start) = attrs.get("start").and_then(|s| s.as_str()) {
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
                if let Some(text) = v.get("text").and_then(|t| t.as_str()) {
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
                    .and_then(|a| a.get("header"))
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
                render_attrs(v, out, &[]);
                out.push('>');
                render_children(v, out, hints);
                out.push_str("</a>");
            }
            "Image" => {
                out.push_str("<img");
                render_attrs(v, out, &[]);
                out.push_str(" />");
            }
            "Element" => render_element(v, out, hints),
            "Text" => {
                if let Some(text) = v.get("text").and_then(|t| t.as_str()) {
                    escape_jsx(text, out);
                }
            }
            "HtmlBlock" => {
                if let Some(text) = v.get("text").and_then(|t| t.as_str()) {
                    render_raw_html(text, out);
                    out.push('\n');
                } else {
                    render_children(v, out, hints);
                }
            }
            "HtmlInline" => {
                if let Some(text) = v.get("text").and_then(|t| t.as_str()) {
                    render_raw_html(text, out);
                } else {
                    render_children(v, out, hints);
                }
            }
            _ => {
                render_children(v, out, hints);
            }
        }
    }
}

/// Renders a structured `Element` node as a real JSX element, so custom
/// components nested in its children (e.g. `<Cite/>` inside a `<figcaption>`)
/// keep working. The tag name is carried in the `name` attribute.
fn render_element(v: &Value, out: &mut String, hints: Option<&SolidRenderHints>) {
    let tag = v
        .get("attrs")
        .and_then(|a| a.get("name"))
        .and_then(|n| n.as_str());

    let Some(tag) = tag else {
        render_children(v, out, hints);
        return;
    };

    out.push('<');
    out.push_str(tag);
    render_attrs(v, out, &[]);

    if pendon_core::is_void_element(tag) {
        out.push_str(" />");
        return;
    }

    out.push('>');
    render_children(v, out, hints);
    out.push_str("</");
    out.push_str(tag);
    out.push('>');
}

/// Renders raw HTML content. Converts HTML comments to JSX comment syntax
/// while passing through other HTML tags unchanged.
fn render_raw_html(text: &str, out: &mut String) {
    let trimmed = text.trim();
    if trimmed.starts_with("<!--") && trimmed.ends_with("-->") {
        let content = &trimmed[4..trimmed.len().saturating_sub(3)];
        out.push_str("{/*");
        out.push_str(content);
        out.push_str("*/}");
    } else {
        out.push_str(text);
    }
}

fn render_children(v: &Value, out: &mut String, hints: Option<&SolidRenderHints>) {
    if let Some(children) = v.get("children").and_then(|c| c.as_array()) {
        for ch in children {
            render_node(ch, out, hints);
        }
    }
}

fn render_children_to_string(v: &Value, hints: Option<&SolidRenderHints>) -> String {
    let mut buf = String::new();
    if let Some(children) = v.get("children").and_then(|c| c.as_array()) {
        for ch in children {
            render_node(ch, &mut buf, hints);
        }
    }
    buf
}

fn render_text_or_children(v: &Value, out: &mut String, hints: Option<&SolidRenderHints>) {
    if let Some(text) = v.get("text").and_then(|t| t.as_str()) {
        escape_jsx(text, out);
    } else {
        render_children(v, out, hints);
    }
}

fn render_attrs(v: &Value, out: &mut String, skipped: &[&str]) {
    let Some(attrs) = v.get("attrs").and_then(Value::as_object) else {
        return;
    };

    for (name, value) in attrs {
        if name == "name" || skipped.contains(&name.as_str()) {
            continue;
        }
        out.push(' ');
        out.push_str(name);
        out.push_str("=\"");
        match value {
            Value::String(value) => escape_jsx(value, out),
            value => escape_jsx(&value.to_string(), out),
        }
        out.push('"');
    }
}

/// Escapes special characters for safe rendering in JSX text nodes and attributes.
/// Curly braces are escaped to HTML entities to prevent SolidJS from treating
/// them as JSX expression containers (e.g., `{p}` would cause a ReferenceError).
fn escape_jsx(s: &str, out: &mut String) {
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            '{' => out.push_str("&#123;"),
            '}' => out.push_str("&#125;"),
            _ => out.push(ch),
        }
    }
}

/// Wraps a string in JSON quotes with proper escaping for use inside
/// JSX curly brace expressions (e.g., innerHTML bindings).
fn json_string_literal(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_string())
}
