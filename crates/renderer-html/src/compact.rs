use pendon_core::Event;
use serde_json::Value;

use crate::events_to_ast_value;
use crate::utils::{attr_bool, attr_str, children, escape_html};

pub fn render_html(events: &[Event]) -> String {
    let ast = events_to_ast_value(events);
    let mut out = String::new();
    render_node(&ast, &mut out);
    out
}

fn render_node(v: &Value, out: &mut String) {
    if let Some(kind) = v.get("type").and_then(|t| t.as_str()) {
        match kind {
            "Document" => {
                if let Some(children) = children(v) {
                    for child in children {
                        render_node(child, out);
                    }
                }
            }
            "Frontmatter" => {}
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
                let level = attr_str(v, "level").unwrap_or("1");
                out.push('<');
                out.push('h');
                out.push_str(level);
                out.push('>');
                render_node_text_or_children(v, out);
                out.push_str("</h");
                out.push_str(level);
                out.push_str(">\n");
            }
            "Section" => {
                out.push_str("<section");
                if let Some(id) = attr_str(v, "id") {
                    out.push(' ');
                    out.push_str("id=\"");
                    escape_html(id, out);
                    out.push_str("\"");
                }
                out.push_str(">\n");
                render_children(v, out);
                out.push_str("</section>\n");
            }
            "ThematicBreak" => {
                out.push_str("<hr />\n");
            }
            "CodeFence" => {
                out.push_str("<pre");
                let class_attr = attr_str(v, "class");
                if let Some(class) = class_attr {
                    out.push(' ');
                    out.push_str("class=\"");
                    escape_html(class, out);
                    out.push_str("\"");
                }
                out.push_str("><code>");
                let raw = attr_str(v, "raw_html");
                if let Some(text) = v.get("text").and_then(|t| t.as_str()) {
                    if matches!(raw, Some("1")) {
                        let payload = if let Some(cls) = class_attr {
                            strip_pre_classes_from_lines(text, cls)
                        } else {
                            text.to_string()
                        };
                        out.push_str(&payload);
                    } else {
                        escape_html(text, out);
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
                if let Some(start) = attr_str(v, "start") {
                    out.push(' ');
                    out.push_str("start=\"");
                    out.push_str(start);
                    out.push_str("\"");
                }
                out.push_str(">\n");
                render_children(v, out);
                out.push_str("</ol>\n");
            }
            "ListItem" => {
                out.push_str("<li>");
                if let Some(text) = v.get("text").and_then(|t| t.as_str()) {
                    escape_html(text, out);
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
                let tag = if attr_bool(v, "header") { "th" } else { "td" };
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
                if let Some(href) = attr_str(v, "href") {
                    out.push(' ');
                    out.push_str("href=\"");
                    escape_html(href, out);
                    out.push_str("\"");
                }
                out.push('>');
                render_children(v, out);
                out.push_str("</a>");
            }
            "Text" => {
                if let Some(text) = v.get("text").and_then(|t| t.as_str()) {
                    escape_html(text, out);
                }
            }
            "HtmlBlock" => {
                if let Some(text) = v.get("text").and_then(|t| t.as_str()) {
                    out.push_str(text);
                    out.push('\n');
                } else {
                    render_children(v, out);
                }
            }
            "HtmlInline" => {
                if let Some(text) = v.get("text").and_then(|t| t.as_str()) {
                    out.push_str(text);
                } else {
                    render_children(v, out);
                }
            }
            _ => {
                render_children(v, out);
            }
        }
    }
}

fn render_children(v: &Value, out: &mut String) {
    if let Some(children) = children(v) {
        for child in children {
            render_node(child, out);
        }
    }
}

fn render_node_text_or_children(v: &Value, out: &mut String) {
    if let Some(text) = v.get("text").and_then(|t| t.as_str()) {
        escape_html(text, out);
    } else {
        render_children(v, out);
    }
}

fn strip_pre_classes_from_lines(html: &str, pre_classes: &str) -> String {
    let bases: Vec<&str> = pre_classes
        .split_whitespace()
        .filter(|c| !c.is_empty())
        .collect();
    if bases.is_empty() {
        return html.to_string();
    }

    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(idx) = rest.find("<p") {
        out.push_str(&rest[..idx]);
        rest = &rest[idx..];
        if let Some(after) = rest.strip_prefix("<p class=\"") {
            if let Some(end_idx) = after.find('"') {
                let class_str = &after[..end_idx];
                let mut classes: Vec<&str> = class_str.split_whitespace().collect();
                classes.retain(|c| !bases.contains(c));
                out.push_str("<p");
                if !classes.is_empty() {
                    out.push_str(" class=\"");
                    out.push_str(&classes.join(" "));
                    out.push('"');
                }
                rest = &after[end_idx + 1..];
                continue;
            }
        }
        out.push_str("<p");
        rest = &rest[2..];
    }
    out.push_str(rest);
    out
}
