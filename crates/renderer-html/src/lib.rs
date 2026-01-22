use pendon_core::Event;
use pendon_renderer_ast::render_ast_to_string;
use serde_json;

// We'll rely on AST JSON field order already enforced; parse-free rendering via events → AST → HTML.
// For performance, a future optimization can render directly from events.

pub fn render_html(events: &[Event]) -> String {
    // Build AST via renderer-ast, then render to HTML
    let ast_json = render_ast_to_string(events).expect("AST serialization failed");
    let v: serde_json::Value = serde_json::from_str(&ast_json).expect("Invalid AST JSON");
    let mut out = String::new();
    render_html_value(&v, &mut out);
    out
}

pub fn render_html_pretty(events: &[Event]) -> String {
    let ast_json = render_ast_to_string(events).expect("AST serialization failed");
    let v: serde_json::Value = serde_json::from_str(&ast_json).expect("Invalid AST JSON");
    let mut out = String::new();
    let mut indent = 0usize;
    render_html_value_pretty(&v, &mut out, &mut indent);
    out
}

fn render_html_value_pretty(v: &serde_json::Value, out: &mut String, indent: &mut usize) {
    let pad = |out: &mut String, n: usize| {
        for _ in 0..n {
            out.push_str("  ");
        }
    };
    if let Some(kind) = v.get("type").and_then(|t: &serde_json::Value| t.as_str()) {
        match kind {
            "Document" => {
                if let Some(children) = v
                    .get("children")
                    .and_then(|c: &serde_json::Value| c.as_array())
                {
                    for ch in children {
                        render_html_value_pretty(ch, out, indent);
                    }
                }
            }
            "Frontmatter" => {
                // Frontmatter is metadata-only for HTML output.
            }
            "Paragraph" => {
                pad(out, *indent);
                out.push_str("<p>\n");
                *indent += 1;
                render_children_pretty(v, out, indent);
                *indent -= 1;
                pad(out, *indent);
                out.push_str("</p>\n");
            }
            "Blockquote" => {
                pad(out, *indent);
                out.push_str("<blockquote>\n");
                *indent += 1;
                render_children_pretty(v, out, indent);
                *indent -= 1;
                pad(out, *indent);
                out.push_str("</blockquote>\n");
            }
            "Heading" => {
                let level = v
                    .get("attrs")
                    .and_then(|a: &serde_json::Value| a.get("level"))
                    .and_then(|l: &serde_json::Value| l.as_str())
                    .unwrap_or("1");
                pad(out, *indent);
                out.push('<');
                out.push('h');
                out.push_str(level);
                out.push('>');
                out.push('\n');
                *indent += 1;
                render_node_text_or_children_pretty(v, out, indent);
                *indent -= 1;
                pad(out, *indent);
                out.push_str("</h");
                out.push_str(level);
                out.push_str(">\n");
            }
            "ThematicBreak" => {
                pad(out, *indent);
                out.push_str("<hr />\n");
            }
            "CodeFence" => {
                pad(out, *indent);
                out.push_str("<pre><code>");
                let raw = v
                    .get("attrs")
                    .and_then(|a: &serde_json::Value| a.get("raw_html"))
                    .and_then(|x| x.as_str());
                if let Some(text) = v.get("text").and_then(|t: &serde_json::Value| t.as_str()) {
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
                render_children_pretty(v, out, indent);
                *indent -= 1;
                pad(out, *indent);
                out.push_str("</ul>\n");
            }
            "OrderedList" => {
                pad(out, *indent);
                out.push_str("<ol");
                if let Some(attrs) = v.get("attrs") {
                    if let Some(start) = attrs
                        .get("start")
                        .and_then(|s: &serde_json::Value| s.as_str())
                    {
                        out.push(' ');
                        out.push_str("start=\"");
                        out.push_str(start);
                        out.push_str("\"");
                    }
                }
                out.push_str(">\n");
                *indent += 1;
                render_children_pretty(v, out, indent);
                *indent -= 1;
                pad(out, *indent);
                out.push_str("</ol>\n");
            }
            "ListItem" => {
                pad(out, *indent);
                out.push_str("<li>\n");
                *indent += 1;
                if let Some(text) = v.get("text").and_then(|t: &serde_json::Value| t.as_str()) {
                    pad(out, *indent);
                    escape_html(text, out);
                    out.push('\n');
                }
                render_children_pretty(v, out, indent);
                *indent -= 1;
                pad(out, *indent);
                out.push_str("</li>\n");
            }
            "Table" => {
                pad(out, *indent);
                out.push_str("<table>\n");
                *indent += 1;
                render_children_pretty(v, out, indent);
                *indent -= 1;
                pad(out, *indent);
                out.push_str("</table>\n");
            }
            "TableHead" => {
                pad(out, *indent);
                out.push_str("<thead>\n");
                *indent += 1;
                render_children_pretty(v, out, indent);
                *indent -= 1;
                pad(out, *indent);
                out.push_str("</thead>\n");
            }
            "TableBody" => {
                pad(out, *indent);
                out.push_str("<tbody>\n");
                *indent += 1;
                render_children_pretty(v, out, indent);
                *indent -= 1;
                pad(out, *indent);
                out.push_str("</tbody>\n");
            }
            "TableRow" => {
                pad(out, *indent);
                out.push_str("<tr>\n");
                *indent += 1;
                render_children_pretty(v, out, indent);
                *indent -= 1;
                pad(out, *indent);
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
                render_children_pretty(v, out, indent);
                out.push_str("</em>\n");
            }
            "Strong" => {
                pad(out, *indent);
                out.push_str("<strong>");
                render_children_pretty(v, out, indent);
                out.push_str("</strong>\n");
            }
            "Bold" => {
                pad(out, *indent);
                out.push_str("<b>");
                render_children_pretty(v, out, indent);
                out.push_str("</b>\n");
            }
            "Italic" => {
                pad(out, *indent);
                out.push_str("<i>");
                render_children_pretty(v, out, indent);
                out.push_str("</i>\n");
            }
            "InlineCode" => {
                pad(out, *indent);
                out.push_str("<code>");
                render_children_pretty(v, out, indent);
                out.push_str("</code>\n");
            }
            "Link" => {
                pad(out, *indent);
                out.push_str("<a");
                if let Some(attrs) = v.get("attrs") {
                    if let Some(href) = attrs
                        .get("href")
                        .and_then(|h: &serde_json::Value| h.as_str())
                    {
                        out.push(' ');
                        out.push_str("href=\"");
                        escape_html(href, out);
                        out.push_str("\"");
                    }
                }
                out.push('>');
                out.push('\n');
                *indent += 1;
                render_children_pretty(v, out, indent);
                *indent -= 1;
                pad(out, *indent);
                out.push_str("</a>\n");
            }
            "Text" => {
                pad(out, *indent);
                if let Some(text) = v.get("text").and_then(|t: &serde_json::Value| t.as_str()) {
                    escape_html(text, out);
                }
                out.push('\n');
            }
            _ => {}
        }
    }
}

fn render_children_pretty(v: &serde_json::Value, out: &mut String, indent: &mut usize) {
    if let Some(children) = v
        .get("children")
        .and_then(|c: &serde_json::Value| c.as_array())
    {
        for ch in children {
            render_html_value_pretty(ch, out, indent);
        }
    }
}

fn render_children_inline(v: &serde_json::Value, out: &mut String) {
    if let Some(children) = v
        .get("children")
        .and_then(|c: &serde_json::Value| c.as_array())
    {
        for ch in children {
            if let Some(kind) = ch.get("type").and_then(|t| t.as_str()) {
                match kind {
                    "Text" => {
                        if let Some(text) = ch.get("text").and_then(|t| t.as_str()) {
                            escape_html(text, out);
                        }
                    }
                    _ => render_html_value(ch, out),
                }
            }
        }
    }
}

fn render_node_text_or_children_pretty(
    v: &serde_json::Value,
    out: &mut String,
    indent: &mut usize,
) {
    if let Some(text) = v.get("text").and_then(|t: &serde_json::Value| t.as_str()) {
        let pad = |out: &mut String, n: usize| {
            for _ in 0..n {
                out.push_str("  ");
            }
        };
        pad(out, *indent);
        escape_html(text, out);
        out.push('\n');
    } else {
        render_children_pretty(v, out, indent);
    }
}

fn render_html_value(v: &serde_json::Value, out: &mut String) {
    if let Some(kind) = v.get("type").and_then(|t: &serde_json::Value| t.as_str()) {
        match kind {
            "Document" => {
                if let Some(children) = v
                    .get("children")
                    .and_then(|c: &serde_json::Value| c.as_array())
                {
                    for ch in children {
                        render_html_value(ch, out);
                    }
                }
            }
            "Frontmatter" => {
                // Skip metadata in HTML output.
            }
            "Paragraph" => {
                out.push_str("<p>");
                render_children(v, out);
                out.push_str("</p>");
            }
            "Blockquote" => {
                out.push_str("<blockquote>");
                render_children(v, out);
                out.push_str("</blockquote>");
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
                render_node_text_or_children(v, out);
                out.push_str("</h");
                out.push_str(level);
                out.push('>');
            }
            "ThematicBreak" => {
                out.push_str("<hr />");
            }
            "CodeFence" => {
                out.push_str("<pre><code>");
                let raw = v
                    .get("attrs")
                    .and_then(|a: &serde_json::Value| a.get("raw_html"))
                    .and_then(|x| x.as_str());
                if let Some(text) = v.get("text").and_then(|t: &serde_json::Value| t.as_str()) {
                    if matches!(raw, Some("1")) {
                        out.push_str(text);
                    } else {
                        escape_html(text, out);
                    }
                }
                out.push_str("</code></pre>");
            }
            "BulletList" => {
                out.push_str("<ul>");
                render_children(v, out);
                out.push_str("</ul>");
            }
            "OrderedList" => {
                out.push_str("<ol");
                if let Some(attrs) = v.get("attrs") {
                    if let Some(start) = attrs
                        .get("start")
                        .and_then(|s: &serde_json::Value| s.as_str())
                    {
                        out.push(' ');
                        out.push_str("start=\"");
                        out.push_str(start);
                        out.push_str("\"");
                    }
                }
                out.push('>');
                render_children(v, out);
                out.push_str("</ol>");
            }
            "ListItem" => {
                out.push_str("<li>");
                if let Some(text) = v.get("text").and_then(|t: &serde_json::Value| t.as_str()) {
                    escape_html(text, out);
                }
                render_children(v, out);
                out.push_str("</li>");
            }
            "Table" => {
                out.push_str("<table>");
                render_children(v, out);
                out.push_str("</table>");
            }
            "TableHead" => {
                out.push_str("<thead>");
                render_children(v, out);
                out.push_str("</thead>");
            }
            "TableBody" => {
                out.push_str("<tbody>");
                render_children(v, out);
                out.push_str("</tbody>");
            }
            "TableRow" => {
                out.push_str("<tr>");
                render_children(v, out);
                out.push_str("</tr>");
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
                render_children_inline(v, out);
                out.push_str("</");
                out.push_str(tag);
                out.push('>');
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
                        out.push(' ');
                        out.push_str("href=\"");
                        escape_html(href, out);
                        out.push_str("\"");
                    }
                }
                out.push('>');
                render_children(v, out);
                out.push_str("</a>");
            }
            "Text" => {
                if let Some(text) = v.get("text").and_then(|t: &serde_json::Value| t.as_str()) {
                    escape_html(text, out);
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
            render_html_value(ch, out);
        }
    }
}

fn render_node_text_or_children(v: &serde_json::Value, out: &mut String) {
    if let Some(text) = v.get("text").and_then(|t: &serde_json::Value| t.as_str()) {
        escape_html(text, out);
    } else {
        render_children(v, out);
    }
}

fn escape_html(s: &str, out: &mut String) {
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
