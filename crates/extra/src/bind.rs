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

use crate::typed::{parse_extras, Attrs, ExtrasAttr, ExtrasHead, ExtrasMatch, ExtrasWarning};

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

/// Pushes a §13 `Warning` for every warning a head resolved.
pub fn emit_attr_warnings(context: &str, attrs: &Attrs, out: &mut Vec<Event>) {
    for warning in &attrs.warnings {
        out.push(warning_event(context, warning));
    }
}

/// A `Severity::Warning` diagnostic carrying the §13 message for `warning`,
/// prefixed with the plugin name.
pub fn warning_event(context: &str, warning: &ExtrasWarning) -> Event {
    Event::Diagnostic {
        severity: Severity::Warning,
        message: format!("[{context}] {}", warning_message(warning)),
        span: None,
    }
}

/// Human-readable §13 message for a resolved warning.
pub fn warning_message(warning: &ExtrasWarning) -> String {
    match warning {
        ExtrasWarning::DuplicateId => {
            "duplicate `#id` in one extras head; the last one wins".to_string()
        }
        ExtrasWarning::Overridden { key } => {
            format!("`{key}` was dropped because the construct head already sets it")
        }
        ExtrasWarning::OwnedKeyIgnored { key } => {
            format!("`{key}` is owned by the construct and cannot be overridden by extras")
        }
    }
}

/// The pending extras syntax replaced by `@@type{…}` (§14). Emitted by every
/// construct plugin that still reads the legacy form.
pub fn legacy_extras_warning(context: &str) -> Event {
    Event::Diagnostic {
        severity: Severity::Warning,
        message: format!(
            "[{context}] the `[.class,#id]{{key: value}}` extras form is deprecated; \
             use `@@type{{…}}` (§14)"
        ),
        span: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typed::{to_attributes, ExtrasOptions};

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
        let attrs = to_attributes(&head, &ExtrasOptions::default());
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
        assert_eq!(names, vec!["class", "id", "slug", "title", "foo", "isFoo"]);
        assert!(matches!(out[5], Event::AttributeFlag { .. }));
    }

    #[test]
    fn describes_and_emits_warnings() {
        let attrs = Attrs {
            items: Vec::new(),
            warnings: vec![
                ExtrasWarning::DuplicateId,
                ExtrasWarning::Overridden {
                    key: "title".to_string(),
                },
            ],
        };
        let mut out = Vec::new();
        emit_attr_warnings("anchor", &attrs, &mut out);
        assert_eq!(out.len(), 2);
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

    #[test]
    fn legacy_warning_names_the_replacement() {
        match legacy_extras_warning("cite") {
            Event::Diagnostic { message, .. } => {
                assert!(message.contains("[cite]"), "{message}");
                assert!(message.contains("@@type"), "{message}");
            }
            other => panic!("unexpected event {other:?}"),
        }
    }
}
