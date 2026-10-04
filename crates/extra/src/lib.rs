//! Shared parsing primitives for plugin attribute syntax.
//!
//! Plugins remain responsible for interpreting parsed properties. This crate
//! only defines the common syntax and preserves the unprefixed property key.

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExtraAttrs {
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub properties: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedAttrs<'a> {
    pub attrs: ExtraAttrs,
    pub rest: &'a str,
    pub had_attrs: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolidImportSpec {
    pub module: String,
    pub default: Option<String>,
    pub names: Vec<String>,
}

/// Parses an optional `[.class,#id]{key: value}` suffix.
///
/// The class block and property block must be adjacent apart from whitespace.
/// Values may be quoted with single or double quotes and commas inside quoted
/// values are preserved.
pub fn parse_attrs(input: &str) -> ParsedAttrs<'_> {
    let source = input.trim_start();
    let mut attrs = ExtraAttrs::default();
    let mut cursor = 0;
    let mut had_attrs = false;

    if source.as_bytes().get(cursor) == Some(&b'[') {
        if let Some(close) = find_closing(source, cursor + 1, ']') {
            parse_class_block(&source[cursor + 1..close], &mut attrs);
            cursor = close + 1;
            had_attrs = true;
        }
    }

    // Spaces are allowed between the `[...]` block and a following `{...}`
    // block. When no property block follows, the whitespace belongs to the
    // caller's remaining text, so the cursor is restored (otherwise consumers
    // that measure consumption by `rest.len()` silently delete it).
    let after_bracket_block = cursor;
    while source.as_bytes().get(cursor) == Some(&b' ') {
        cursor += 1;
    }

    if source.as_bytes().get(cursor) == Some(&b'{') {
        if let Some(close) = find_closing(source, cursor + 1, '}') {
            parse_properties(&source[cursor + 1..close], &mut attrs);
            cursor = close + 1;
            had_attrs = true;
        }
    } else {
        cursor = after_bracket_block;
    }

    ParsedAttrs {
        attrs,
        rest: source.get(cursor..).unwrap_or_default(),
        had_attrs,
    }
}

/// Parses a standalone `{key: value}` block and returns the unconsumed suffix.
pub fn parse_property_block(input: &str) -> Option<(ExtraAttrs, &str)> {
    let source = input.trim_start();
    if source.as_bytes().first() != Some(&b'{') {
        return None;
    }
    let close = find_closing(source, 1, '}')?;
    let mut attrs = ExtraAttrs::default();
    parse_properties(&source[1..close], &mut attrs);
    Some((attrs, source.get(close + 1..).unwrap_or_default()))
}

fn parse_class_block(input: &str, attrs: &mut ExtraAttrs) {
    for token in split_csv(input) {
        let token = token.trim();
        if let Some(class) = token.strip_prefix('.') {
            if !class.is_empty() {
                attrs.classes.push(class.to_string());
            }
        } else if let Some(id) = token.strip_prefix('#') {
            if !id.is_empty() {
                attrs.id = Some(id.to_string());
            }
        }
    }
}

fn parse_properties(input: &str, attrs: &mut ExtraAttrs) {
    for pair in split_csv(input) {
        let Some((key, value)) = pair.split_once(':') else {
            continue;
        };
        let key = key.trim();
        if !key.is_empty() {
            attrs
                .properties
                .push((key.to_string(), unquote(value.trim())));
        }
    }
}

/// Splits comma-separated text while preserving commas inside quoted values.
pub fn split_csv(input: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut quote = None;

    for (index, character) in input.char_indices() {
        match character {
            '\'' | '"' if quote == Some(character) => quote = None,
            '\'' | '"' if quote.is_none() => quote = Some(character),
            ',' if quote.is_none() => {
                parts.push(input[start..index].trim());
                start = index + character.len_utf8();
            }
            _ => {}
        }
    }

    parts.push(input[start..].trim());
    parts
}

fn unquote(value: &str) -> String {
    let bytes = value.as_bytes();
    if bytes.len() >= 2
        && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\''))
    {
        return value[1..value.len() - 1].to_string();
    }
    value.to_string()
}

fn find_closing(input: &str, start: usize, closing: char) -> Option<usize> {
    input[start..]
        .char_indices()
        .find_map(|(offset, character)| (character == closing).then_some(start + offset))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_classes_id_and_quoted_commas() {
        let parsed = parse_attrs("[.hero,#main]{title: \"A, B\", --tone: red} trailing");
        assert_eq!(parsed.attrs.classes, vec!["hero"]);
        assert_eq!(parsed.attrs.id.as_deref(), Some("main"));
        assert_eq!(
            parsed.attrs.properties,
            vec![
                ("title".to_string(), "A, B".to_string()),
                ("--tone".to_string(), "red".to_string()),
            ]
        );
        assert_eq!(parsed.rest, " trailing");
        assert!(parsed.had_attrs);
    }

    #[test]
    fn leaves_plain_text_untouched() {
        let parsed = parse_attrs("caption");
        assert_eq!(parsed.rest, "caption");
        assert!(!parsed.had_attrs);
        assert_eq!(parsed.attrs, ExtraAttrs::default());
    }

    #[test]
    fn keeps_whitespace_after_a_bracket_only_block() {
        let parsed = parse_attrs("[.hero] trailing");
        assert_eq!(parsed.attrs.classes, vec!["hero"]);
        assert_eq!(parsed.rest, " trailing");
        assert!(parsed.had_attrs);
    }

    #[test]
    fn still_allows_spaces_before_a_property_block() {
        let parsed = parse_attrs("[.hero]  {title: \"A\"} trailing");
        assert_eq!(parsed.attrs.classes, vec!["hero"]);
        assert_eq!(
            parsed.attrs.properties,
            vec![("title".to_string(), "A".to_string())]
        );
        assert_eq!(parsed.rest, " trailing");
    }
}
