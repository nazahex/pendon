//! The two automatic `export const` registrations (§20, [ADR-0009]).
//!
//! Both are properties of the **document**, not of a plugin, so the renderer
//! computes them from the AST: `frontmatter` is the JSON text `micromatter` put
//! on its node, and `headings` is the outline built from every `Heading` node.
//! The retired `plugin-extract-heading` used to inject that outline as a
//! `Headings` node; building it here removes the node from the IR without moving
//! a byte of the export (T4.5, [ADR-0011]).
//!
//! [ADR-0009]: ../../../docs/decisions/0009-generic-data-exports.md
//! [ADR-0011]: ../../../docs/decisions/0011-plugin-extract.md

use serde_json::{Map, Value};

pub fn extract_frontmatter(v: &Value) -> Option<String> {
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

/// The `headings` export: a nested outline of every heading in the document, or
/// `None` when it has none (§20).
///
/// The shape is the one `plugin-extract-heading` injected, so consumers and
/// goldens do not move: `[{ id, level, text, subheadings? }]`, keys alphabetical
/// and `subheadings` omitted when empty.
pub fn extract_headings(v: &Value) -> Option<Value> {
    if v.get("type")?.as_str()? != "Document" {
        return None;
    }

    let mut roots: Vec<Outline> = Vec::new();
    let mut path: Vec<usize> = Vec::new();
    let mut sections: Vec<Option<String>> = Vec::new();
    walk(v, &mut roots, &mut path, &mut sections);

    if roots.is_empty() {
        return None;
    }
    Some(Value::Array(
        roots.into_iter().map(Outline::into_value).collect(),
    ))
}

/// One heading of the outline, before nesting.
#[derive(Debug, Default, PartialEq, Eq)]
struct Outline {
    id: String,
    text: String,
    level: usize,
    subheadings: Vec<Outline>,
}

impl Outline {
    /// The export's JSON shape: `id`, `level`, `subheadings` (only when it has
    /// any) then `text` — alphabetical, the order `serde_json::Map` sorts them
    /// into anyway, written explicitly so the bytes never depend on the crate's
    /// map implementation.
    fn into_value(self) -> Value {
        let mut map = Map::new();
        map.insert("id".to_string(), Value::String(self.id));
        map.insert("level".to_string(), Value::Number(self.level.into()));
        if !self.subheadings.is_empty() {
            map.insert(
                "subheadings".to_string(),
                Value::Array(self.subheadings.into_iter().map(Self::into_value).collect()),
            );
        }
        map.insert("text".to_string(), Value::String(self.text));
        Value::Object(map)
    }
}

/// Walks one node in document order, so headings keep the order they have in the
/// document — including the ones nested in a section, a blockquote or a list item
/// (the legacy builder walked the event stream the same way).
fn walk(
    node: &Value,
    roots: &mut Vec<Outline>,
    path: &mut Vec<usize>,
    sections: &mut Vec<Option<String>>,
) {
    let kind = node.get("type").and_then(Value::as_str).unwrap_or_default();

    if kind == "Heading" {
        collect(node, roots, path, sections);
    }

    // §9.5: when `plugin-section` owns the outline the heading carries no `id`;
    // its section does. Only the **innermost** section is consulted — a section
    // without an id does not inherit the enclosing one's (the legacy rule, kept
    // so the export does not move).
    let section = (kind == "Section").then(|| {
        node.get("attrs")
            .and_then(|attrs| attrs.get("id"))
            .and_then(Value::as_str)
            .map(str::to_string)
    });
    let opens_section = section.is_some();
    if let Some(id) = section {
        sections.push(id);
    }

    if let Some(children) = node.get("children").and_then(Value::as_array) {
        for child in children {
            walk(child, roots, path, sections);
        }
    }

    if opens_section {
        sections.pop();
    }
}

/// Builds one outline node from a `Heading` node and nests it by level.
fn collect(
    node: &Value,
    roots: &mut Vec<Outline>,
    path: &mut Vec<usize>,
    sections: &[Option<String>],
) {
    let attrs = node.get("attrs");
    let level = attr(attrs, "level")
        .and_then(|level| level.parse::<usize>().ok())
        .unwrap_or(1)
        .max(1);
    let text = heading_text(node);

    // §6.2's chain: the section id beats the heading's own `id`, which beats its
    // auto `slug`, which beats a slug of the text (§7.4 heads are already folded
    // into those attributes by `plugin-heading`).
    let id = sections
        .last()
        .and_then(|id| id.clone())
        .or_else(|| attr(attrs, "id"))
        .or_else(|| attr(attrs, "slug"))
        .unwrap_or_else(|| slugify(&text));

    insert(
        Outline {
            id,
            text,
            level,
            subheadings: Vec::new(),
        },
        roots,
        path,
    );
}

fn attr(attrs: Option<&Value>, name: &str) -> Option<String> {
    attrs?.get(name)?.as_str().map(str::to_string)
}

/// A heading's visible text.
///
/// `plugin-heading` already reduced the line to its title (`text`), so only an
/// authored numbering is left to drop: `## 1.2. Intro` outlines as `Intro`, as
/// the legacy builder did.
fn heading_text(node: &Value) -> String {
    let mut text = String::new();
    plain_text(node, &mut text);
    strip_number_prefix(&text).trim().to_string()
}

/// The concatenated `text` of a node and its descendants: a heading keeps inline
/// markup as children under some lexers.
fn plain_text(node: &Value, out: &mut String) {
    if let Some(text) = node.get("text").and_then(Value::as_str) {
        out.push_str(text);
        return;
    }
    if let Some(children) = node.get("children").and_then(Value::as_array) {
        for child in children {
            plain_text(child, out);
        }
    }
}

/// Drops an authored `1.` / `1.2. ` prefix. A number that is not followed by a
/// full stop and a space (`42 Things`) is part of the title and stays.
fn strip_number_prefix(input: &str) -> &str {
    let bytes = input.as_bytes();
    let mut index = 0;
    let mut saw_digit = false;
    while index < bytes.len() {
        if bytes[index].is_ascii_digit() {
            saw_digit = true;
            index += 1;
        } else if bytes[index] == b'.' && saw_digit {
            saw_digit = false;
            index += 1;
        } else if bytes[index].is_ascii_whitespace() {
            return if index > 0 && !saw_digit {
                input[index..].trim_start()
            } else {
                input
            };
        } else {
            return input;
        }
    }
    input
}

/// A URL-safe slug of a heading's text: the last resort of the id chain.
fn slugify(input: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in input.chars() {
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() {
            out.push(lower);
            last_dash = false;
        } else if matches!(lower, ' ' | '-' | '_' | '.') && !last_dash && !out.is_empty() {
            out.push('-');
            last_dash = true;
        }
    }
    if out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "section".to_string()
    } else {
        out
    }
}

/// Nests `node` under the deepest open heading of a lower level: a lower or equal
/// level closes every sibling at its level (`h2` after `h3` closes the `h3`).
fn insert(node: Outline, roots: &mut Vec<Outline>, path: &mut Vec<usize>) {
    while let Some(level) = open_level(roots, path) {
        if level >= node.level {
            path.pop();
        } else {
            break;
        }
    }

    let target = open_children_mut(roots, path);
    target.push(node);
    let new_index = target.len().saturating_sub(1);
    path.push(new_index);
}

fn open_level(roots: &[Outline], path: &[usize]) -> Option<usize> {
    let mut cursor = roots;
    let mut node: Option<&Outline> = None;
    for &index in path {
        node = cursor.get(index);
        cursor = &node?.subheadings;
    }
    node.map(|node| node.level)
}

fn open_children_mut<'a>(roots: &'a mut Vec<Outline>, path: &[usize]) -> &'a mut Vec<Outline> {
    let mut cursor = roots;
    for &index in path {
        cursor = &mut cursor[index].subheadings;
    }
    cursor
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn document(children: Value) -> Value {
        json!({ "type": "Document", "children": children })
    }

    /// The outline nests by level and takes its ids from the enclosing section
    /// (§9.5) — the shape `plugin-extract-heading` injected, now built here.
    #[test]
    fn an_outline_nests_by_level_and_uses_the_section_id() {
        let v = document(json!([{
            "type": "Section",
            "attrs": { "id": "lorem" },
            "children": [
                { "type": "Heading", "text": "Lorem", "attrs": { "level": "1" } },
                {
                    "type": "Section",
                    "attrs": { "id": "foo" },
                    "children": [
                        { "type": "Heading", "text": "Foo", "attrs": { "level": "2" } },
                        { "type": "Heading", "text": "Deep", "attrs": { "level": "3" } }
                    ]
                },
                { "type": "Heading", "text": "Back", "attrs": { "level": "2" } }
            ]
        }]));

        let headings = extract_headings(&v).expect("an outline");
        assert_eq!(
            serde_json::to_string(&headings).expect("json"),
            concat!(
                r#"[{"id":"lorem","level":1,"subheadings":["#,
                r#"{"id":"foo","level":2,"subheadings":[{"id":"foo","level":3,"text":"Deep"}],"#,
                r#""text":"Foo"},{"id":"lorem","level":2,"text":"Back"}],"text":"Lorem"}]"#
            )
        );
    }

    /// Without a section the heading's own `id` wins, then its auto `slug`, then
    /// a slug of the text (§6.2).
    #[test]
    fn the_id_chain_falls_back_to_the_heading_itself() {
        let v = document(json!([
            { "type": "Heading", "text": "One", "attrs": { "level": "2", "id": "explicit" } },
            { "type": "Heading", "text": "Two", "attrs": { "level": "2", "slug": "auto-slug" } },
            { "type": "Heading", "text": "Three Words!", "attrs": { "level": "2" } }
        ]));

        let headings = extract_headings(&v).expect("an outline");
        assert_eq!(
            serde_json::to_string(&headings).expect("json"),
            concat!(
                r#"[{"id":"explicit","level":2,"text":"One"},"#,
                r#"{"id":"auto-slug","level":2,"text":"Two"},"#,
                r#"{"id":"three-words","level":2,"text":"Three Words!"}]"#
            )
        );
    }

    /// An authored `1.2. ` prefix is numbering, not title (the legacy builder
    /// stripped it, so the export does too); a number without the full stop is
    /// part of the title.
    #[test]
    fn an_authored_number_prefix_is_not_part_of_the_text() {
        let v = document(json!([
            { "type": "Heading", "text": "1.2. Intro", "attrs": { "level": "2" } },
            { "type": "Heading", "text": "42 Things", "attrs": { "level": "2" } }
        ]));

        let headings = extract_headings(&v).expect("an outline");
        assert_eq!(
            serde_json::to_string(&headings).expect("json"),
            r#"[{"id":"intro","level":2,"text":"Intro"},{"id":"42-things","level":2,"text":"42 Things"}]"#
        );
    }

    /// Inline markup is hoisted into children by some lexers; the title is their
    /// concatenated text.
    #[test]
    fn inline_children_contribute_their_text() {
        let v = document(json!([{
            "type": "Heading",
            "attrs": { "level": "2" },
            "children": [
                { "type": "Text", "text": "Hello " },
                { "type": "Strong", "children": [{ "type": "Text", "text": "World" }] }
            ]
        }]));

        let headings = extract_headings(&v).expect("an outline");
        assert_eq!(
            serde_json::to_string(&headings).expect("json"),
            r#"[{"id":"hello-world","level":2,"text":"Hello World"}]"#
        );
    }

    #[test]
    fn a_document_without_headings_has_no_export() {
        let v = document(json!([{ "type": "Paragraph", "text": "Body." }]));
        assert_eq!(extract_headings(&v), None);
    }

    /// A fragment (`mdp` payload) is not a document: it registers nothing (§19).
    #[test]
    fn a_non_document_has_no_heading_export() {
        let v = json!({ "type": "Paragraph", "text": "Body." });
        assert_eq!(extract_headings(&v), None);
    }
}
