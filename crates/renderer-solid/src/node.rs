// node.rs
use serde_json::Value;

use crate::{
    imports::ComponentTemplate,
    resolve::{Ctx, LayerKey},
    template,
};

pub fn render_node(v: &Value, out: &mut String, ctx: &Ctx) {
    if let Some(kind) = v.get("type").and_then(|t| t.as_str()) {
        let attrs = v.get("attrs").and_then(Value::as_object);
        if let Some(template) = template_for(kind, attrs, ctx) {
            let text = v.get("text").and_then(|t| t.as_str());
            let mut children = render_children_to_string(v, ctx);
            // A node whose content is pure text is serialized as `text` rather
            // than `children` (see `renderer-ast`), but a template's
            // `{children}` must still see that content.
            if children.is_empty() {
                if let Some(text) = text {
                    escape_jsx(text, &mut children);
                }
            }
            let fields = pendon_extra::project(v, None).attrs();
            out.push_str(&template::render_template(
                template, &fields, &children, text, ctx,
            ));
            return;
        }

        match kind {
            "Document" => {
                if let Some(children) = v.get("children").and_then(|c| c.as_array()) {
                    for ch in children {
                        render_node(ch, out, ctx);
                    }
                }
            }
            "Frontmatter" | "Headings" => {
                // Metadata only; skip emitting markup.
            }
            "Paragraph" => {
                out.push_str("<p");
                render_attrs(v, out, &[], ctx);
                out.push('>');
                render_children(v, out, ctx);
                out.push_str("</p>\n");
            }
            "Blockquote" => {
                out.push_str("<blockquote");
                render_attrs(v, out, &[], ctx);
                out.push_str(">\n");
                render_children(v, out, ctx);
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
                // `raw_title` / `slug` are template-only reads (§11 rule 3): the
                // built-in `<h*>` fallback keeps the authored extras, including
                // the `type` marker, but none of those. `number` is **not** one
                // of them any more: `plugin-number` writes it for every element
                // (ADR-0010), so the fallback prints it like any other attribute
                // (`number={["1","a"]}`).
                render_attrs(v, out, &["level", "raw_title", "slug"], ctx);
                out.push('>');
                render_text_or_children(v, out, ctx);
                out.push_str("</h");
                out.push_str(level);
                out.push_str(">\n");
            }
            "Section" => {
                out.push_str("<section");
                render_attrs(v, out, &[], ctx);
                out.push_str(">\n");
                render_children(v, out, ctx);
                out.push_str("</section>\n");
            }
            "ThematicBreak" => {
                out.push_str("<hr />\n");
            }
            "CodeFence" => {
                out.push_str("<pre");
                render_attrs(v, out, &["raw_html"], ctx);
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
                out.push_str("<ul");
                render_attrs(v, out, &[], ctx);
                out.push_str(">\n");
                render_children(v, out, ctx);
                out.push_str("</ul>\n");
            }
            "OrderedList" => {
                out.push_str("<ol");
                if let Some(attrs) = v.get("attrs") {
                    if let Some(start) = attrs.get("start").and_then(|s| s.as_str()) {
                        out.push_str(" start={");
                        out.push_str(start);
                        out.push('}');
                    }
                }
                render_attrs(v, out, &["start"], ctx);
                out.push_str(">\n");
                render_children(v, out, ctx);
                out.push_str("</ol>\n");
            }
            "ListItem" => {
                out.push_str("<li>");
                if let Some(text) = v.get("text").and_then(|t| t.as_str()) {
                    escape_jsx(text, out);
                }
                render_children(v, out, ctx);
                out.push_str("</li>\n");
            }
            "Table" => {
                out.push_str("<table>\n");
                render_children(v, out, ctx);
                out.push_str("</table>\n");
            }
            "TableHead" => {
                out.push_str("<thead>\n");
                render_children(v, out, ctx);
                out.push_str("</thead>\n");
            }
            "TableBody" => {
                out.push_str("<tbody>\n");
                render_children(v, out, ctx);
                out.push_str("</tbody>\n");
            }
            "TableRow" => {
                out.push_str("<tr>\n");
                render_children(v, out, ctx);
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
                render_children(v, out, ctx);
                out.push_str("</");
                out.push_str(tag);
                out.push_str(">\n");
            }
            "Emphasis" => {
                out.push_str("<em>");
                render_children(v, out, ctx);
                out.push_str("</em>");
            }
            "Strong" => {
                out.push_str("<strong>");
                render_children(v, out, ctx);
                out.push_str("</strong>");
            }
            "Bold" => {
                out.push_str("<b>");
                render_children(v, out, ctx);
                out.push_str("</b>");
            }
            "Italic" => {
                out.push_str("<i>");
                render_children(v, out, ctx);
                out.push_str("</i>");
            }
            "InlineCode" => {
                out.push_str("<code>");
                render_children(v, out, ctx);
                out.push_str("</code>");
            }
            "Link" => {
                out.push_str("<a");
                render_attrs(v, out, &[], ctx);
                out.push('>');
                render_children(v, out, ctx);
                out.push_str("</a>");
            }
            "Image" => {
                out.push_str("<img");
                render_attrs(v, out, &[], ctx);
                out.push_str(" />");
            }
            "Element" => render_element(v, out, ctx),
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
                    render_children(v, out, ctx);
                }
            }
            "HtmlInline" => {
                if let Some(text) = v.get("text").and_then(|t| t.as_str()) {
                    render_raw_html(text, out);
                } else {
                    render_children(v, out, ctx);
                }
            }
            _ => {
                render_children(v, out, ctx);
            }
        }
    }
}

/// The §11 template `kind`/`attrs` resolves to, if any (ADR-0008, `D-3a`).
///
/// The decision is document-wide: the plugin only tagged the node with the layer
/// it belongs to (`__layer`) and kept its `type` marker; [`Ctx::resolution`] has
/// already worked out which entry of that layer answers each occurring type, so
/// `?` / `_` / `*` see the whole document rather than this one element.
fn template_for<'a>(
    kind: &str,
    attrs: Option<&serde_json::Map<String, Value>>,
    ctx: &'a Ctx,
) -> Option<&'a ComponentTemplate> {
    let hints = ctx.hints?;
    let key = ctx.resolution.key_for(kind, attrs)?;
    // A §11 layer keys on the `type` marker; a node identity (`plugin-custom`,
    // `plugin-vicado`) has no marker to answer and keeps matching as before.
    let marker = match key {
        LayerKey::Layer(_) => attrs
            .and_then(|attrs| attrs.get("type"))
            .and_then(Value::as_str),
        LayerKey::Identity(..) => None,
    };
    let index = ctx.resolution.select(key, marker)?;
    hints.templates.get(index)
}

/// Renders a structured `Element` node as a real JSX element, so custom
/// components nested in its children (e.g. `<Cite/>` inside a `<figcaption>`)
/// keep working. The tag name is carried in the `name` attribute.
fn render_element(v: &Value, out: &mut String, ctx: &Ctx) {
    let tag = v
        .get("attrs")
        .and_then(|a| a.get("name"))
        .and_then(|n| n.as_str());

    let Some(tag) = tag else {
        render_children(v, out, ctx);
        return;
    };

    out.push('<');
    out.push_str(tag);
    render_attrs(v, out, &[], ctx);

    if pendon_core::is_void_element(tag) {
        out.push_str(" />");
        return;
    }

    out.push('>');
    render_children(v, out, ctx);
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

fn render_children(v: &Value, out: &mut String, ctx: &Ctx) {
    if let Some(children) = v.get("children").and_then(|c| c.as_array()) {
        for ch in children {
            render_node(ch, out, ctx);
        }
    }
}

fn render_children_to_string(v: &Value, ctx: &Ctx) -> String {
    let mut buf = String::new();
    if let Some(children) = v.get("children").and_then(|c| c.as_array()) {
        for ch in children {
            render_node(ch, &mut buf, ctx);
        }
    }
    buf
}

/// Renders a fragment's node list (a nested `mdp` payload, see
/// `pendon_renderer_ast::FRAGMENT_KEY`). Every node goes through the same
/// `render_node` the document body uses, so templates, custom components and
/// block plugins behave identically inside a bound payload.
pub(crate) fn render_fragment(nodes: &[Value], ctx: &Ctx) -> String {
    let mut buf = String::new();
    for node in nodes {
        render_node(node, &mut buf, ctx);
    }
    buf
}

fn render_text_or_children(v: &Value, out: &mut String, ctx: &Ctx) {
    if let Some(text) = v.get("text").and_then(|t| t.as_str()) {
        escape_jsx(text, out);
    } else {
        render_children(v, out, ctx);
    }
}

/// Emits the attributes of `v` as JSX attributes. A JSON `true` value is a bare
/// attribute (§6.3 flag) and is emitted in JSX boolean shorthand: `<Tag isFoo />`.
///
/// A structured value (object/array, re-hydrated from a data-binding transform,
/// `docs/rfc/plugin-bind.md`) is emitted as a JSX expression `name={…}`; a raw
/// JSX expression (`JSX_ATTR_PREFIX`, the extras `( … )` literal) is wrapped in
/// parentheses `name={( … )}`. A rendered fragment (`$fragment`, a nested
/// `mdp` payload) reaches this branch too, so it emits `name={(<>…</>)}`.
fn render_attrs(v: &Value, out: &mut String, skipped: &[&str], ctx: &Ctx) {
    // The fields come from the projection, not from the node's own `attrs` map
    // (ADR-0014 rule 2): internals (`__`-prefixed, §11 rule 3) are gone, `specs.*`
    // is in, and `project` has already renamed the universal positions to
    // `class`/`id`/`style` (§6.3). Phase 4 adds each element's preset assignment
    // on top of the same map.
    for (name, value) in pendon_extra::project(v, None).attrs() {
        // `name` is the AST node name, not an attribute — the template path
        // skips it in `generate_spread_attrs` for the same reason.
        if name == "name" || skipped.contains(&name.as_str()) {
            continue;
        }
        // A **public** field path may nest (`specs.tick`, `bracket.bare`, §6.3),
        // and JSX reads `a.b=` as a member expression rather than an attribute
        // name, so such a field is emitted as a one-key spread
        // ([`render_dotted_attr`]) instead of `name=…`.
        if !is_jsx_attr_name(&name) {
            render_dotted_attr(&name, &value, out, ctx);
            continue;
        }
        out.push(' ');
        out.push_str(&name);
        if matches!(&value, Value::Bool(true)) {
            continue;
        }
        // A raw JSX fragment (rendered MDP) reaches Solid as a parenthesised
        // expression so the markup is parsed, not stringified.
        if let Value::String(text) = &value {
            if let Some(fragment) = pendon_core::jsx_attr_payload(text) {
                if is_jsx_expression_token(fragment) {
                    // A bare identifier / dotted path is emitted unquoted so it
                    // is passed as a variable reference: `name={valueA}`.
                    out.push_str("={");
                    out.push_str(fragment);
                    out.push('}');
                    continue;
                }
                out.push_str("={(");
                out.push_str(fragment);
                out.push_str(")}");
                continue;
            }
        }
        match &value {
            Value::String(text) => {
                out.push_str("=\"");
                escape_jsx(text, out);
                out.push('"');
            }
            // Objects and arrays are JS expressions, not strings. Emit clean JS
            // literal syntax (`{ key: value }`, `[ item, ... ]`) rather than
            // compact JSON with escaped quotes. A fragment value renders its
            // node list as JSX.
            Value::Object(_) | Value::Array(_) => {
                out.push_str("={");
                out.push_str(&to_js_literal(&value, ctx));
                out.push('}');
            }
            Value::Number(_) => {
                out.push_str("={");
                out.push_str(&value.to_string());
                out.push('}');
            }
            Value::Bool(false) => {
                out.push_str("={false}");
            }
            Value::Null => {
                out.push_str("={null}");
            }
            // `Bool(true)` was already emitted as a bare flag above.
            Value::Bool(true) => {}
        }
    }
}

/// Whether `name` can be written as a JSX attribute name.
///
/// The public field map is dotted (`specs.tick`, `bracket.bare`, §6.3/S2-3.4), so
/// a field name is not automatically an attribute name. Hyphens and a namespace
/// colon are legal in JSX attribute names (`cite-id`, `xlink:href`), which is why
/// only the rest is a failure.
fn is_jsx_attr_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | ':'))
}

/// Emits one dotted public field as a one-key spread: `{...{ "specs.tick": "…" }}`.
///
/// `specs.tick="x"` is not valid JSX — `a.b=` parses as a member expression — but
/// the field map's spelling is the contract (ADR-0014 rule 1), so the name is
/// kept verbatim as a computed key instead of being rewritten. Solid's spread
/// sets it as an attribute, so the compiled output and `renderer-html`'s
/// `specs.tick="x"` agree, and JSON/AST keep the path untouched.
fn render_dotted_attr(name: &str, value: &Value, out: &mut String, ctx: &Ctx) {
    out.push_str(" {...{ ");
    out.push_str(&json_string_literal(name));
    out.push_str(": ");
    out.push_str(&js_value(value, ctx));
    out.push_str(" }}");
}

/// One field value as a JS expression — the form a spread key needs, where no
/// JSX attribute shorthand exists (no bare flag, no quoted text).
fn js_value(value: &Value, ctx: &Ctx) -> String {
    if let Value::String(text) = value {
        if let Some(payload) = pendon_core::jsx_attr_payload(text) {
            return if is_jsx_expression_token(payload) {
                payload.to_string()
            } else {
                format!("({payload})")
            };
        }
    }
    match value {
        Value::String(text) => json_string_literal(text),
        Value::Bool(true) => "true".to_string(),
        Value::Bool(false) => "false".to_string(),
        Value::Null => "null".to_string(),
        Value::Number(_) => value.to_string(),
        Value::Object(_) | Value::Array(_) => to_js_literal(value, ctx),
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

/// `true` when `text` is a single JSX expression token — a dot-separated
/// identifier path such as `valueA` or `frontmatter.fm1` — which is emitted
/// unquoted as `name={valueA}` rather than wrapped in parentheses.
pub(crate) fn is_jsx_expression_token(text: &str) -> bool {
    let mut segments = text.split('.');
    let first = segments.next();
    if first.is_none() {
        return false;
    }
    for segment in std::iter::once(first.unwrap_or("")).chain(segments) {
        let segment = segment.trim();
        if segment.is_empty() {
            return false;
        }
        let mut chars = segment.chars();
        match chars.next() {
            Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
            _ => return false,
        }
        if !chars.all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return false;
        }
    }
    true
}

/// Converts a `serde_json::Value` to a clean JavaScript literal expression
/// string — object literal syntax (`{ key: value }`) rather than compact JSON
/// with quoted keys and escaped quotes. Used so structured extras values render
/// as `name={ key: "value" }` instead of `name={"{\"key\":\"value\"}"}`.
///
/// A fragment value (a nested `mdp` payload, `renderer_ast::FRAGMENT_KEY`)
/// renders as a JSX fragment expression `(<>…</>)`, so markup can sit anywhere a
/// structured value can — including inside an object (`{body: $mdp}`).
pub(crate) fn to_js_literal(value: &Value, ctx: &Ctx) -> String {
    if let Some(nodes) = pendon_renderer_ast::fragment_nodes(value) {
        return format!("(<>{}</>)", render_fragment(nodes, ctx));
    }
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => {
            let mut out = String::from("\"");
            for ch in s.chars() {
                match ch {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    c => out.push(c),
                }
            }
            out.push('"');
            out
        }
        Value::Array(items) => {
            let mut out = String::from("[");
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                out.push_str(&to_js_literal(item, ctx));
            }
            out.push(']');
            out
        }
        Value::Object(map) => {
            let mut out = String::from("{");
            let mut first = true;
            for (key, val) in map {
                if !first {
                    out.push_str(", ");
                }
                first = false;
                if is_js_identifier(key) {
                    out.push_str(key);
                } else {
                    out.push('"');
                    for ch in key.chars() {
                        match ch {
                            '"' => out.push_str("\\\""),
                            '\\' => out.push_str("\\\\"),
                            c => out.push(c),
                        }
                    }
                    out.push('"');
                }
                out.push_str(": ");
                out.push_str(&to_js_literal(val, ctx));
            }
            out.push('}');
            out
        }
    }
}

/// `true` when `s` is a valid JavaScript identifier that can be used as an
/// unquoted object key.
fn is_js_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}
