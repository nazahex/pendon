// HTML-specific mapping from syntect CSS classes to minimal tags.
// Minimal tags: b (bold), i (italic), u (underline), s (strikethrough), mark (highlight), em (emphasis)

pub fn map_classes_to_tag(classes: &str) -> Option<&'static str> {
    let c = classes;
    // Do not wrap plain text nodes; keep main inline content untagged
    if contains_any(c, &["text"]) {
        return None;
    }
    // Strong emphasis for structural keywords and tag names
    if contains_any(c, &["tag", "doctype", "keyword", "storage", "support"]) {
        return Some("b"); // very frequent → smallest strong tag
    }
    // Names and selectors
    if contains_any(c, &["selector"]) {
        return Some("dfn"); // definition-ish selector tokens
    }
    if contains_any(c, &["class-name", "id"]) {
        return Some("abbr"); // class/id often abbreviated
    }
    if contains_any(c, &["attribute-name", "name"]) {
        return Some("em"); // short and readable for names
    }
    // Values and text
    if contains_any(
        c,
        &[
            "string",
            "quoted",
            "unquoted",
            "attribute-value",
            "property-value",
        ],
    ) {
        return Some("i"); // very frequent → smallest italic tag
    }
    // Numeric constants
    if contains_any(c, &["number", "numeric", "integer", "decimal", "rgb-value"]) {
        return Some("u"); // underline for numbers
    }
    // Variables and parameters
    if contains_any(c, &["variable", "parameter", "parameters"]) {
        return Some("var");
    }
    // Comments (including SGML comment scope)
    if contains_any(c, &["comment", "sgml"]) {
        return Some("s"); // strikethrough
    }
    // Insertions / inline markers
    if contains_any(c, &["inline"]) {
        return Some("ins");
    }
    // Operators, punctuation, separators, terminators and embedded blocks
    if contains_any(
        c,
        &[
            "operator",
            "punctuation",
            "separator",
            "terminator",
            "embedded",
            "script",
            "style",
        ],
    ) {
        return Some("mark");
    }
    // Fallback for miscellaneous tokens
    if contains_any(c, &["other", "meta", "entity"]) {
        return Some("span");
    }
    None
}

fn contains_any(hay: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| hay.contains(n))
}
