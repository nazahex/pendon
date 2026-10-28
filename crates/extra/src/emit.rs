//! Glue between a parsed extras head (`docs/spec/SYNTAX.md` §5) and the event IR
//! (§6).
//!
//! Construct plugins (`plugin-anchor`, `-cite`, `-heading`, `-img`, `-wiki`)
//! own *where* an extras head attaches and *which* keys are construct-owned;
//! this module owns the boilerplate every one of them repeats: scanning
//! `@@type{…}` at a character cursor, turning an [`Attrs`] set into
//! `Event::Attribute` / `Event::AttributeFlag` events, and reporting the §13
//! warnings a head resolved.
//!
//! It is the only place in `crates/extra` that knows about `pendon-core`.

use pendon_core::{Event, Severity};

use crate::typed::{
    parse_extras, Attrs, ExtrasAttr, ExtrasHead, ExtrasIssue, ExtrasMatch, IssueSeverity,
};

/// Scans `@@type{…}` / `@@{…}` at a **character** cursor.
///
/// Returns the parsed head and the cursor just past it, or `None` when the text
/// at the cursor is not a well-formed head. Malformed heads stay literal text
/// (§4.3), so the caller must keep the original characters.
pub fn scan_extras_chars(chars: &[char], cursor: usize) -> Option<(ExtrasHead, usize)> {
    if cursor >= chars.len() {
        return None;
    }
    let text: String = chars[cursor..].iter().collect();
    match parse_extras(&text) {
        ExtrasMatch::Head { head, rest } => {
            let consumed = text.len() - rest.len();
            Some((head, cursor + text[..consumed].chars().count()))
        }
        ExtrasMatch::Absent { .. } | ExtrasMatch::Malformed { .. } => None,
    }
}

/// Pushes the attribute events of `attrs` onto `out` in canonical order (§6.4):
/// `Event::Attribute` for values (stringly typed through `AttrValue::literal`,
/// §6.3) and `Event::AttributeFlag` for bare flags.
pub fn emit_attrs(attrs: &Attrs, out: &mut Vec<Event>) {
    for (name, value) in &attrs.items {
        match value {
            ExtrasAttr::Flag => out.push(Event::AttributeFlag { name: name.clone() }),
            ExtrasAttr::Value(value) => out.push(Event::Attribute {
                name: name.clone(),
                value: value.literal(),
            }),
        }
    }
}

/// Pushes a §13 diagnostic for every issue a head resolved.
pub fn emit_attr_issues(context: &str, attrs: &Attrs, out: &mut Vec<Event>) {
    for issue in &attrs.issues {
        out.push(issue_event(context, issue));
    }
}

/// An `Event::Diagnostic` carrying `issue`'s own §13 severity (D-12 makes a
/// duplicate an `Error`), prefixed with the plugin name.
///
/// This is the one adapter between the grammar layer's [`IssueSeverity`] and the
/// IR's [`Severity`].
pub fn issue_event(context: &str, issue: &ExtrasIssue) -> Event {
    let severity = match issue.severity {
        IssueSeverity::Warning => Severity::Warning,
        IssueSeverity::Error => Severity::Error,
    };
    Event::Diagnostic {
        severity,
        message: format!("[{context}] {}", issue.message),
        span: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typed::{to_attributes, Specs};

    fn chars(text: &str) -> Vec<char> {
        text.chars().collect()
    }

    #[test]
    fn scans_a_head_at_a_character_cursor() {
        let text = chars("tail@@note{.a,`slug-a`} rest");
        let (head, cursor) = scan_extras_chars(&text, 4).expect("head");
        assert_eq!(head.type_marker.as_deref(), Some("note"));
        assert_eq!(cursor, 4 + "@@note{.a,`slug-a`}".chars().count());
    }

    #[test]
    fn scans_a_bare_and_a_type_only_head() {
        // §3: the `@@` prefix is optional.
        let text = chars("x{.a} rest");
        let (head, cursor) = scan_extras_chars(&text, 1).expect("head");
        assert_eq!(head.type_marker, None);
        assert_eq!(cursor, 1 + "{.a}".chars().count());

        // §4.1: a type may stand alone; the symbol after it stays literal.
        let text = chars("x@@anchorA. rest");
        let (head, cursor) = scan_extras_chars(&text, 1).expect("head");
        assert_eq!(head.type_marker.as_deref(), Some("anchorA"));
        assert!(head.items.is_empty());
        assert_eq!(cursor, 1 + "@@anchorA".chars().count());
        assert_eq!(text[cursor], '.');

        // §4.1: `@@type {…}` is not a head.
        let text = chars("x@@anchorA {.a}");
        assert!(scan_extras_chars(&text, 1).is_none());
    }

    #[test]
    fn malformed_or_absent_heads_consume_nothing() {
        let text = chars("@@note{.a body");
        assert!(scan_extras_chars(&text, 0).is_none());
        let text = chars("plain text");
        assert!(scan_extras_chars(&text, 0).is_none());
        let text = chars("@@note{.a}");
        assert!(scan_extras_chars(&text, text.len()).is_none());
    }

    #[test]
    fn emits_values_and_flags_in_order() {
        let head = match parse_extras("@@note{`a`, \"T\", .c, #i, foo: 12, isFoo}") {
            ExtrasMatch::Head { head, .. } => head,
            other => panic!("expected head, got {other:?}"),
        };
        let attrs = to_attributes(&head, &Specs::default());
        let mut out = Vec::new();
        emit_attrs(&attrs, &mut out);
        let names: Vec<String> = out
            .iter()
            .map(|event| match event {
                Event::Attribute { name, .. } => name.clone(),
                Event::AttributeFlag { name } => name.clone(),
                other => panic!("unexpected event {other:?}"),
            })
            .collect();
        assert_eq!(
            names,
            vec![
                "specs.class",
                "specs.id",
                "specs.tick",
                "specs.quote",
                "foo",
                "isFoo"
            ]
        );
        assert!(matches!(out[5], Event::AttributeFlag { .. }));
    }

    #[test]
    fn describes_and_emits_issues() {
        let attrs = Attrs {
            items: Vec::new(),
            issues: vec![
                ExtrasIssue::duplicate("specs.id"),
                ExtrasIssue::overridden("title"),
            ],
        };
        let mut out = Vec::new();
        emit_attr_issues("anchor", &attrs, &mut out);
        assert_eq!(out.len(), 2);
        match &out[0] {
            Event::Diagnostic {
                severity,
                message,
                span,
            } => {
                assert_eq!(*severity, Severity::Error);
                assert!(message.starts_with("[anchor] `specs.id`"), "{message}");
                assert!(span.is_none());
            }
            other => panic!("unexpected event {other:?}"),
        }
        match &out[1] {
            Event::Diagnostic {
                severity,
                message,
                span,
            } => {
                assert_eq!(*severity, Severity::Warning);
                assert!(message.starts_with("[anchor] `title`"), "{message}");
                assert!(span.is_none());
            }
            other => panic!("unexpected event {other:?}"),
        }
    }
}
