use pendon_core::Event;
use serde_json::Value;

use crate::events_to_ast_value;
use crate::utils::{attr_bool, attr_str, children, escape_html};

pub fn render_html_pretty(events: &[Event]) -> String {
    let ast = events_to_ast_value(events);
    let mut out = String::new();
    let mut indent = 0usize;
    render_node(&ast, &mut out, &mut indent);
    out
}

fn render_node(v: &Value, out: &mut String, indent: &mut usize) {
    let pad = |out: &mut String, n: usize| {
        for _ in 0..n {
            out.push_str("  ");
        }
    };

    if let Some(kind) = v.get("type").and_then(|t| t.as_str()) {
        match kind {
            "Document" => {
                if let Some(children) = children(v) {
                    for child in children {
                        render_node(child, out, indent);
                    }
                }
            }
            "Frontmatter" => {}
            "Paragraph" => {
                pad(out, *indent);
                out.push_str("<p>\n");
                *indent += 1;
                render_children(v, out, indent);
                *indent -= 1;
                pad(out, *indent);
                out.push_str("</p>\n");
            }
            "Blockquote" => {
                pad(out, *indent);
                out.push_str("<blockquote>\n");
                *indent += 1;
                render_children(v, out, indent);
                *indent -= 1;
                pad(out, *indent);
                out.push_str("</blockquote>\n");
            }
            "Heading" => {
                let level = attr_str(v, "level").unwrap_or("1");
                pad(out, *indent);
                out.push('<');
                out.push('h');
                out.push_str(level);
                out.push('>');
                out.push('\n');
                *indent += 1;
                render_node_text_or_children(v, out, indent, pad);
                *indent -= 1;
                pad(out, *indent);
                out.push_str("</h");
                out.push_str(level);
                out.push_str(">\n");
            }
            "Section" => {
                pad(out, *indent);
                out.push_str("<section");
                if let Some(id) = attr_str(v, "id") {
                    out.push(' ');
                    out.push_str("id=\"");
                    escape_html(id, out);
                    out.push_str("\"");
                }
                out.push_str(">\n");
                *indent += 1;
                render_children(v, out, indent);
                *indent -= 1;
                pad(out, *indent);
                out.push_str("</section>\n");
            }
            "ThematicBreak" => {
                pad(out, *indent);
                out.push_str("<hr />\n");
            }
            "CodeFence" => {
                pad(out, *indent);
                out.push_str("<pre><code>");
                let raw = attr_str(v, "raw_html");
                if let Some(text) = v.get("text").and_then(|t| t.as_str()) {
                    if matches!(raw, Some("1")) {
                        out.push_str(text);
                    } else {
                        escape_html(text, out);
                    }
                }
                out.push_str("</code></pre>\n");
            }
            "BulletList" => {
                pad(out, *indent);
                out.push_str("<ul>\n");
                *indent += 1;
                render_children(v, out, indent);
                *indent -= 1;
                pad(out, *indent);
                out.push_str("</ul>\n");
            }
            "OrderedList" => {
                pad(out, *indent);
                out.push_str("<ol");
                if let Some(start) = attr_str(v, "start") {
                    out.push(' ');
                    out.push_str("start=\"");
                    out.push_str(start);
                    out.push_str("\"");
                }
                out.push_str(">\n");
                *indent += 1;
                render_children(v, out, indent);
                *indent -= 1;
                pad(out, *indent);
                out.push_str("</ol>\n");
            }
            "ListItem" => {
                pad(out, *indent);
                out.push_str("<li>\n");
                *indent += 1;
                if let Some(text) = v.get("text").and_then(|t| t.as_str()) {
                    pad(out, *indent);
                    escape_html(text, out);
                    out.push('\n');
                }
                render_children(v, out, indent);
                *indent -= 1;
                pad(out, *indent);
                out.push_str("</li>\n");
            }
            "Table" => {
                pad(out, *indent);
                out.push_str("<table>\n");
                *indent += 1;
                render_children(v, out, indent);
                *indent -= 1;
                pad(out, *indent);
                out.push_str("</table>\n");
            }
            "TableHead" => {
                pad(out, *indent);
                out.push_str("<thead>\n");
                *indent += 1;
                render_children(v, out, indent);
                *indent -= 1;
                pad(out, *indent);
                out.push_str("</thead>\n");
            }
            "TableBody" => {
                pad(out, *indent);
                out.push_str("<tbody>\n");
                *indent += 1;
                render_children(v, out, indent);
                *indent -= 1;
                pad(out, *indent);
                out.push_str("</tbody>\n");
            }
            "TableRow" => {
                pad(out, *indent);
                out.push_str("<tr>\n");
                *indent += 1;
                render_children(v, out, indent);
                *indent -= 1;
                pad(out, *indent);
                out.push_str("</tr>\n");
            }
            "TableCell" => {
                let tag = if attr_bool(v, "header") { "th" } else { "td" };
                pad(out, *indent);
                out.push('<');
                out.push_str(tag);
                out.push('>');
                render_children_inline(v, out);
                out.push_str("</");
                out.push_str(tag);
                out.push_str(">\n");
            }
            "Emphasis" => {
                pad(out, *indent);
                out.push_str("<em>");
                render_children(v, out, indent);
                out.push_str("</em>\n");
            }
            "Strong" => {
                pad(out, *indent);
                out.push_str("<strong>");
                render_children(v, out, indent);
                out.push_str("</strong>\n");
            }
            "Bold" => {
                pad(out, *indent);
                out.push_str("<b>");
                render_children(v, out, indent);
                out.push_str("</b>\n");
            }
            "Italic" => {
                pad(out, *indent);
                out.push_str("<i>");
                render_children(v, out, indent);
                out.push_str("</i>\n");
            }
            "InlineCode" => {
                pad(out, *indent);
                out.push_str("<code>");
                render_children(v, out, indent);
                out.push_str("</code>\n");
            }
            "Link" => {
                pad(out, *indent);
                out.push_str("<a");
                if let Some(href) = attr_str(v, "href") {
                    out.push(' ');
                    out.push_str("href=\"");
                    escape_html(href, out);
                    out.push_str("\"");
                }
                out.push('>');
                out.push('\n');
                *indent += 1;
                render_children(v, out, indent);
                *indent -= 1;
                pad(out, *indent);
                out.push_str("</a>\n");
            }
            "Text" => {
                pad(out, *indent);
                if let Some(text) = v.get("text").and_then(|t| t.as_str()) {
                    escape_html(text, out);
                }
                out.push('\n');
            }
            "HtmlBlock" => {
                pad(out, *indent);
                if let Some(text) = v.get("text").and_then(|t| t.as_str()) {
                    out.push_str(text);
                    out.push('\n');
                } else {
                    render_children(v, out, indent);
                }
            }
            "HtmlInline" => {
                pad(out, *indent);
                if let Some(text) = v.get("text").and_then(|t| t.as_str()) {
                    out.push_str(text);
                } else {
                    render_children(v, out, indent);
                }
                out.push('\n');
            }
            _ => {
                render_children(v, out, indent);
            }
        }
    }
}

fn render_children(v: &Value, out: &mut String, indent: &mut usize) {
    if let Some(children) = children(v) {
        for child in children {
            render_node(child, out, indent);
        }
    }
}

fn render_children_inline(v: &Value, out: &mut String) {
    if let Some(children) = children(v) {
        for child in children {
            if let Some(kind) = child.get("type").and_then(|t| t.as_str()) {
                match kind {
                    "Text" => {
                        if let Some(text) = child.get("text").and_then(|t| t.as_str()) {
                            escape_html(text, out);
                        }
                    }
                    _ => {
                        let mut inline_indent = 0usize;
                        render_node(child, out, &mut inline_indent);
                    }
                }
            }
        }
    }
}

fn render_node_text_or_children(
    v: &Value,
    out: &mut String,
    indent: &mut usize,
    pad: impl Fn(&mut String, usize),
) {
    if let Some(text) = v.get("text").and_then(|t| t.as_str()) {
        pad(out, *indent);
        escape_html(text, out);
        out.push('\n');
    } else {
        render_children(v, out, indent);
    }
}
