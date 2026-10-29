// TypeScript-specific mapping from syntect CSS classes to minimal tags.
// Minimal tags: b, em, i, u, s, mark, var, span

pub fn map_classes_to_tag(classes: &str) -> Option<&'static str> {
    let c = classes;
    // Specific: declaration identifiers for type/interface/namespace → definition tag
    if contains_any(
        c,
        &[
            "entity name type",
            "entity name interface",
            "entity name namespace",
        ],
    ) {
        return Some("dfn");
    }
    // Specific: declaration keywords for type/interface/namespace
    if contains_any(
        c,
        &[
            "keyword declaration type",
            "keyword declaration interface",
            "keyword declaration namespace",
        ],
    ) {
        return Some("b");
    }
    // Strong/structural: keywords, storage, support, builtin, import/export, declaration, modifier
    if contains_any(
        c,
        &[
            "keyword",
            "storage",
            "support",
            "builtin",
            "import",
            "export",
            "declaration",
            "modifier",
        ],
    ) {
        return Some("b");
    }

    // Names: entities and generic names (function/class/type identifiers)
    if contains_any(c, &["entity", "name"]) {
        return Some("em");
    }

    // Variables and parameters
    if contains_any(c, &["variable", "parameter", "readwrite"]) {
        return Some("var");
    }

    // Strings (quoted, template)
    if contains_any(c, &["string", "quoted", "template"]) {
        return Some("i");
    }

    // Numbers
    if contains_any(c, &["numeric", "constant numeric"]) {
        return Some("u");
    }

    // Comments
    if contains_any(c, &["comment", "double-slash"]) {
        return Some("s");
    }

    // Operators & punctuation (arithmetic, assignment, bitwise, comparison, logical, relational,
    // as well as separators, terminators, commas, grouping)
    if contains_any(
        c,
        &[
            "operator",
            "arithmetic",
            "assignment",
            "bitwise",
            "comparison",
            "logical",
            "relational",
            "punctuation",
            "separator",
            "terminator",
            "comma",
            "group",
            "section",
        ],
    ) {
        return Some("mark");
    }

    // Types: map to names
    if contains_any(c, &["type"]) {
        return Some("em");
    }

    // Template expression wrappers: treat as punctuation/marker
    if contains_any(c, &["template-expression"]) {
        return Some("mark");
    }

    // Fallback
    if contains_any(
        c,
        &[
            "meta",
            "other",
            "object-literal",
            "embedded",
            "js",
            "ts",
            "source",
        ],
    ) {
        return Some("span");
    }

    None
}

fn contains_any(hay: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| hay.contains(n))
}
