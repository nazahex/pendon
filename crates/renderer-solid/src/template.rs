use serde_json::Value;
use std::collections::BTreeSet;

use crate::imports::ComponentTemplate;
use crate::node::{
    is_js_identifier, is_jsx_attr_name, is_jsx_expression_token, js_value, render_dotted_attr,
    to_js_literal,
};
use crate::resolve::Ctx;
use pendon_core::{json_attr_payload, jsx_attr_payload};

/// Renders one component template.
///
/// `attrs` is the node's **projection** (`pendon_extra::project`), not its raw
/// `attrs` map: `{...attrs}` and `{attrs.k}` therefore read public fields only,
/// which is where `__`-prefixed internals already stopped existing
/// (ADR-0014 rule 2).
pub fn render_template(
    template: &ComponentTemplate,
    attrs: &[(String, Value)],
    children: &str,
    text: Option<&str>,
    ctx: &Ctx,
) -> String {
    let mut out = String::new();
    let tpl = template.template.as_str();
    let bytes = tpl.as_bytes();

    // Pre-scan the template to find explicitly used {attrs.X} keys.
    let mut used_attr_keys = BTreeSet::new();
    let mut scan_i = 0;
    while scan_i < bytes.len() {
        if bytes[scan_i] == b'{' && tpl[scan_i..].starts_with("{attrs.") {
            if let Some(end) = tpl[scan_i + 7..].find('}') {
                let key = &tpl[scan_i + 7..scan_i + 7 + end];
                used_attr_keys.insert(key.to_string());
                scan_i += 7 + end + 1;
                continue;
            }
        }
        scan_i += 1;
    }

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
            // Handle the spread operator {...attrs} with smart spacing
            if tpl[i..].starts_with("{...attrs}") {
                let spread_str = if attrs.is_empty() {
                    String::new()
                } else {
                    generate_spread_attrs(attrs, &used_attr_keys, ctx)
                };

                if !spread_str.is_empty() {
                    // Add a leading space only if there isn't one already in the buffer
                    if !out.is_empty() && !out.ends_with(' ') && !out.ends_with('\n') {
                        out.push(' ');
                    }
                    out.push_str(&spread_str);
                } else {
                    // If spread is empty, remove trailing space from output buffer
                    // to prevent `<Epis >` when template is `<Epis {...attrs}>`
                    if out.ends_with(' ') {
                        out.pop();
                    }
                }
                i += "{...attrs}".len();
                continue;
            }
            if tpl[i..].starts_with("{attrs.") {
                if let Some(end) = tpl[i + 7..].find('}') {
                    let key = &tpl[i + 7..i + 7 + end];
                    let val = get_attr_value(attrs, key);
                    let next = i + 7 + end + 1;
                    // §6.3: a custom component must not receive an empty slot.
                    // When `{attrs.X}` resolves to nothing, drop the whole
                    // `X={…}` attribute instead of emitting `X=""` — a template
                    // like `slug={"{attrs.slug}"}` only rides along when the
                    // component config actually set `slug`. `alt` is exempt: an
                    // empty `alt=""` is meaningful (a decorative image).
                    if val.is_empty() && key != "alt" {
                        if let Some((start, end)) = enclosing_attribute(tpl, i, next) {
                            // `out` holds everything rendered for `tpl[..i]`, so a
                            // template index maps back by that delta.
                            out.truncate(out.len() - (i - start));
                            while out.ends_with([' ', '\t', '\n', '\r']) {
                                out.pop();
                            }
                            i = end;
                            continue;
                        }
                    }
                    out.push_str(&val);
                    i = next;
                    continue;
                }
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

/// Locates the `name={…}` attribute in `tpl` that owns the `{attrs.X}` reference
/// sitting at `attr_start`..`attr_end`.
///
/// Returns the template span to drop only when the value is *nothing but* that
/// reference — i.e. the head between `=` and the reference holds only `{`/`"`
/// wrappers, and the tail past it holds only the matching `}`/`"` closers. A
/// value that mixes literals with `{attrs.X}` (or an `{attrs.X}` that is not an
/// attribute value at all) is left untouched.
fn enclosing_attribute(tpl: &str, attr_start: usize, attr_end: usize) -> Option<(usize, usize)> {
    let bytes = tpl.as_bytes();

    // Walk back over the wrappers, the `=` and the attribute name.
    let mut start = attr_start;
    while start > 0 {
        let prev = bytes[start - 1];
        if prev.is_ascii_whitespace() || prev == b'<' || prev == b'>' {
            break;
        }
        start -= 1;
    }
    let head = &tpl[start..attr_start];
    let eq = head.find('=')?;
    if !head[eq + 1..].chars().all(|c| c == '{' || c == '"') {
        return None;
    }

    // Walk forward over the value's closing quotes/braces.
    let mut end = attr_end;
    while end < bytes.len() && (bytes[end] == b'"' || bytes[end] == b'}') {
        end += 1;
    }
    if end == attr_end {
        return None;
    }
    if end < bytes.len() && !bytes[end].is_ascii_whitespace() && bytes[end] != b'>' {
        return None;
    }
    Some((start, end))
}

/// The value a template's `{attrs.k}` reference resolves to: the field's text,
/// or the empty string when the projection has no such field (which is what
/// makes `enclosing_attribute` drop the whole attribute — §6.3).
fn get_attr_value(attrs: &[(String, Value)], key: &str) -> String {
    match attrs.iter().find(|(name, _)| name == key) {
        Some((_, Value::String(text))) => text.clone(),
        Some((_, value)) => value.to_string(),
        None => String::new(),
    }
}

/// Generates JSX attributes using curly brace expressions for all values.
/// Automatically infers type from string values (boolean, number, or string).
///
/// A dotted public field (`locator.label`, `bracket.quote`) is **regrouped into a
/// nested prop** — `locator={{ label: "page", value: "110-112" }}` — because a
/// component takes JS values, not DOM attribute names ([ADR-0019]). Sibling paths
/// share one prop, deeper paths nest recursively, and the prop sits where its
/// first leaf sat in the field map. A prefix that also exists as a scalar field
/// keeps the flat one-key spread ([`render_dotted_attr`]) instead, so no prop is
/// ever written twice.
///
/// [ADR-0019]: ../../../docs/decisions/0019-component-props-nest.md
fn generate_spread_attrs(
    attrs: &[(String, Value)],
    used_keys: &BTreeSet<String>,
    ctx: &Ctx,
) -> String {
    let nested = nested_prefixes(attrs, used_keys);
    let mut emitted: BTreeSet<&str> = BTreeSet::new();
    let mut parts = Vec::new();
    for (key, val) in attrs {
        if !is_spread(key, val, used_keys) {
            continue;
        }
        // §6.5: a public field is a **path** (`specs.tick`, `bracket.bare`), and
        // JSX reads `a.b=` as a member expression rather than an attribute name,
        // so a dotted field cannot be written verbatim — `<X specs.tick="…">` is
        // not JSX at all.
        if !is_jsx_attr_name(key) {
            let prefix = key.split('.').next().unwrap_or(key);
            if !nested.contains(prefix) {
                let mut part = String::new();
                render_dotted_attr(key, val, &mut part, ctx);
                parts.push(part);
                continue;
            }
            // One prop per prefix, written where its first leaf belongs.
            if !emitted.insert(prefix) {
                continue;
            }
            let mut part = String::new();
            part.push_str(prefix);
            part.push_str("={");
            part.push_str(&nested_prop(prefix, attrs, used_keys, ctx));
            part.push('}');
            parts.push(part);
            continue;
        }

        let mut part = String::new();
        part.push_str(key);

        match val {
            Value::String(s) => {
                let trimmed = s.trim();
                // A `JSX_ATTR_PREFIX` payload is a JSX expression reference
                // (e.g. `{valueA}` / `{frontmatter.fm1}`). Emit it unquoted as
                // `name={valueA}` so it is passed as a variable reference,
                // not as a string literal.
                if let Some(payload) = jsx_attr_payload(s) {
                    if is_jsx_expression_token(payload) {
                        part.push_str("={");
                        part.push_str(payload);
                        part.push('}');
                        parts.push(part);
                        continue;
                    }
                    part.push_str("={(");
                    part.push_str(payload);
                    part.push_str(")}");
                    parts.push(part);
                    continue;
                }
                // A `JSON_ATTR_PREFIX` payload carries a structured value
                // (object/array) produced by `plugin-bind`. Re-hydrate it and
                // emit as a JS expression.
                if let Some(payload) = json_attr_payload(s) {
                    if let Ok(parsed) = serde_json::from_str::<Value>(payload) {
                        match parsed {
                            Value::Object(_) | Value::Array(_) => {
                                part.push_str("={");
                                part.push_str(&to_js_literal(&parsed, ctx));
                                part.push('}');
                                parts.push(part);
                                continue;
                            }
                            _ => {}
                        }
                    }
                }
                // A string that looks like structured JSON (starts with `{` or
                // `[`) but was not carried through `JSON_ATTR_PREFIX` — e.g. a
                // nested object/array written directly in the extras head without
                // `plugin-bind`. Emit it as a JS expression.
                if (trimmed.starts_with('{') || trimmed.starts_with('['))
                    && serde_json::from_str::<Value>(trimmed)
                        .map(|p| matches!(p, Value::Object(_) | Value::Array(_)))
                        .unwrap_or(false)
                {
                    if let Ok(parsed) = serde_json::from_str::<Value>(trimmed) {
                        part.push_str("={");
                        part.push_str(&to_js_literal(&parsed, ctx));
                        part.push('}');
                        parts.push(part);
                        continue;
                    }
                }
                // Infer boolean
                if trimmed == "true" {
                    part.push_str("={true}");
                } else if trimmed == "false" {
                    part.push_str("={false}");
                }
                // Infer integer
                else if let Ok(n) = trimmed.parse::<i64>() {
                    part.push_str("={");
                    part.push_str(&n.to_string());
                    part.push('}');
                }
                // Infer float
                else if let Ok(f) = trimmed.parse::<f64>() {
                    part.push_str("={");
                    part.push_str(&f.to_string());
                    part.push('}');
                }
                // Default to a plain quoted string attribute.
                else {
                    part.push_str("=\"");
                    part.push_str(&escape_js_string(s));
                    part.push('"');
                }
            }
            Value::Number(n) => {
                part.push_str("={");
                part.push_str(&n.to_string());
                part.push('}');
            }
            Value::Bool(b) => {
                part.push_str("={");
                part.push_str(if *b { "true" } else { "false" });
                part.push('}');
            }
            Value::Null => {
                // Skip null values to keep JSX clean
                continue;
            }
            _ => {
                // Arrays or Objects: pass as JS literal expression
                part.push_str("={");
                part.push_str(&to_js_literal(val, ctx));
                part.push('}');
            }
        }
        parts.push(part);
    }
    parts.join(" ")
}

/// Whether an `attrs` entry takes part in the `{...attrs}` spread.
///
/// Three exclusions, all §6.3: `name` is the AST node name (internals never reach
/// the projection, so there is no `__` prefix to test here), a key the template
/// already binds explicitly (`{attrs.k}`) is not spread twice, and an empty value
/// is dropped rather than written as `k=""` — `alt` excepted, because an empty
/// `alt=""` marks a decorative image.
fn is_spread(key: &str, val: &Value, used_keys: &BTreeSet<String>) -> bool {
    if key == "name" || used_keys.contains(key) {
        return false;
    }
    key == "alt" || !matches!(val, Value::String(s) if s.is_empty())
}

/// The first segments of the dotted paths the spread writes as a nested prop.
///
/// A prefix only nests when nothing flat claims the same name — nesting over a
/// scalar prop would write `locator` twice. `FieldMap` already refuses a parent
/// and its child in one map, so this is a guard, not a live case.
fn nested_prefixes(attrs: &[(String, Value)], used_keys: &BTreeSet<String>) -> BTreeSet<String> {
    let flat: BTreeSet<&str> = attrs
        .iter()
        .filter(|(key, val)| is_jsx_attr_name(key) && is_spread(key, val, used_keys))
        .map(|(key, _)| key.as_str())
        .collect();
    attrs
        .iter()
        .filter(|(key, val)| !is_jsx_attr_name(key) && is_spread(key, val, used_keys))
        .filter_map(|(key, _)| key.split('.').next())
        .filter(|prefix| is_jsx_attr_name(prefix) && !flat.contains(prefix))
        .map(str::to_string)
        .collect()
}

/// One node of a nested prop: a field value, or the segments below it.
enum PropTree<'a> {
    Leaf(&'a Value),
    Branch(Vec<(String, PropTree<'a>)>),
}

/// Writes every spread-able field under `prefix` as one object literal, in field
/// map order: `{ label: "page", value: "110-112" }`.
fn nested_prop(
    prefix: &str,
    attrs: &[(String, Value)],
    used_keys: &BTreeSet<String>,
    ctx: &Ctx,
) -> String {
    let mut tree: Vec<(String, PropTree<'_>)> = Vec::new();
    for (key, val) in attrs {
        if !is_jsx_attr_name(key) && is_spread(key, val, used_keys) {
            let mut segments: Vec<&str> = key.split('.').collect();
            if segments.first() == Some(&prefix) {
                segments.remove(0);
                nest(&mut tree, &segments, val);
            }
        }
    }
    render_prop_tree(&tree, ctx)
}

/// Inserts `value` at `segments` (`["label"]`, `["detail", "amount"]`) inside
/// `tree`, creating a branch per missing segment. A branch's position is its
/// first appearance, so the object keeps the field map's order.
fn nest<'a>(tree: &mut Vec<(String, PropTree<'a>)>, segments: &[&str], value: &'a Value) {
    let Some((head, rest)) = segments.split_first() else {
        return;
    };
    if rest.is_empty() {
        tree.push(((*head).to_string(), PropTree::Leaf(value)));
        return;
    }
    if let Some((_, PropTree::Branch(children))) = tree
        .iter_mut()
        .find(|(name, node)| name == head && matches!(node, PropTree::Branch(_)))
    {
        nest(children, rest, value);
        return;
    }
    // A leaf already sits where a branch has to go. `FieldMap` forbids that, so
    // the map cannot produce it; keep the deeper field out rather than write a
    // duplicate key into the object.
    if tree.iter().any(|(name, _)| name == head) {
        debug_assert!(false, "`{head}` is both a field and a nested prefix");
        return;
    }
    let mut children = Vec::new();
    nest(&mut children, rest, value);
    tree.push(((*head).to_string(), PropTree::Branch(children)));
}

/// Renders a nested prop's tree as one JS object literal. A segment that is not
/// a bare JS identifier is quoted, so the object is always valid JS.
fn render_prop_tree(tree: &[(String, PropTree<'_>)], ctx: &Ctx) -> String {
    let mut out = String::from("{ ");
    for (index, (name, node)) in tree.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        if is_js_identifier(name) {
            out.push_str(name);
        } else {
            out.push('"');
            for ch in name.chars() {
                match ch {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    c => out.push(c),
                }
            }
            out.push('"');
        }
        out.push_str(": ");
        match node {
            PropTree::Leaf(value) => out.push_str(&js_value(value, ctx)),
            PropTree::Branch(children) => out.push_str(&render_prop_tree(children, ctx)),
        }
    }
    out.push_str(" }");
    out
}

/// Escapes a string for use inside a JavaScript string literal (inside JSX {}).
/// This is different from HTML attribute escaping.
fn escape_js_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
