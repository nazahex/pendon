//! Renderable-field spans: `;;…;;` (inline) and `;;;…;;;` (block).
//!
//! `docs/spec/SYNTAX.md` §6.7, decision `ADR-0021`. A renderable is a **whole
//! value** authored where a field value goes — a slot content or a spec value —
//! whose text is an `mdp` payload rather than a string:
//!
//! ```text
//! {{markerX}}[;;lorem *ipsum*;;]
//! @@type{note: ;;## A heading;;}
//! ```
//!
//! [`span`] is the one scanner every head lexer shares ([`crate::slot`],
//! [`crate::typed`]). It is the only reason "rendered as full mdp" holds: the
//! interior of a span is opaque, so `,` `"` `` ` `` `[` `]` `{` `}` `(` `)` `:`
//! and whitespace inside it cannot end a slot content, split a spec item or
//! close a group.
//!
//! The scanner is deliberately separate from the payload: it never parses
//! Markdown and never looks at a plugin. It answers two questions only — "does a
//! span start here, and where does it end?" ([`span`]) and "is this whole value
//! exactly one span?" ([`whole`]).

use std::ops::Range;

use pendon_core::MdpMode;

/// An opened span, located in the text it was found in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    /// `;;` → [`MdpMode::Inline`], `;;;` → [`MdpMode::Block`].
    pub mode: MdpMode,
    /// Byte range of the payload between the opener and the closer.
    pub body: Range<usize>,
    /// Byte index just past the closer — the closer is consumed **whole**, so
    /// no stray `;` of a longer run is left behind.
    pub end: usize,
}

impl Span {
    /// The authored payload, [`materialize`]d.
    pub fn source(&self, text: &str) -> String {
        materialize(&text[self.body.clone()])
    }
}

/// A whole value that is one span: the payload source and how to parse it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Renderable {
    /// `;;` → inline, `;;;` → block.
    pub mode: MdpMode,
    /// The payload, [`materialize`]d (`\n` is already a newline).
    pub source: String,
}

/// Reads a span starting at byte `start`, or `None` when no opener is there.
///
/// The opener is chosen by the **longest** match (`;;;` is block, otherwise `;;`
/// is inline), and a run of `;` at least as long as the opener closes the span
/// (D-RF-3): a `;;;` run inside a `;;` span cannot leave a stray `;` behind.
/// `\` escapes the next character, so `\;` is payload text and cannot close.
/// An unclosed span is `None`, never an error — the caller keeps the text
/// literal (§4.3).
pub fn span(text: &str, start: usize) -> Option<Span> {
    let rest = text.get(start..)?;
    let (opener, mode) = if rest.starts_with(";;;") {
        (3, MdpMode::Block)
    } else if rest.starts_with(";;") {
        (2, MdpMode::Inline)
    } else {
        return None;
    };

    let bytes = text.as_bytes();
    let body_start = start + opener;
    let mut cursor = body_start;

    while cursor < bytes.len() {
        match bytes[cursor] {
            // An escape covers the next *character*, so a multi-byte one keeps
            // the cursor on a char boundary (the body is sliced by range).
            b'\\' => {
                cursor += 1;
                if let Some(next) = text.get(cursor..).and_then(|rest| rest.chars().next()) {
                    cursor += next.len_utf8();
                }
            }
            b';' => {
                let mut end = cursor;
                while bytes.get(end) == Some(&b';') {
                    end += 1;
                }
                if end - cursor >= opener {
                    return Some(Span {
                        mode,
                        body: body_start..cursor,
                        end,
                    });
                }
                cursor = end;
            }
            _ => cursor += 1,
        }
    }

    None
}

/// The end of a span that starts at `start`, if any — the shape every lexer
/// loop wants.
pub fn end_after(text: &str, start: usize) -> Option<usize> {
    span(text, start).map(|span| span.end)
}

/// Reads `text` as a **whole** renderable value: after trimming, everything
/// from the first byte to the last is one span (D-RF-4).
///
/// `None` when the text is not a span, when something trails the closer
/// (`;;a;; b`), or when the payload is empty or whitespace only (`;;;;`,
/// `;; ;;`), which stays literal — an empty renderable would silently delete the
/// value it was written for.
pub fn whole(text: &str) -> Option<Renderable> {
    let trimmed = text.trim();
    let span = span(trimmed, 0)?;
    if span.end != trimmed.len() {
        return None;
    }
    let source = span.source(trimmed);
    if source.trim().is_empty() {
        return None;
    }
    Some(Renderable {
        mode: span.mode,
        source,
    })
}

/// Resolves a payload's escapes: `\n` is a newline and `\x` is `x`.
///
/// The same rule as §4.2's slot escapes, applied to the payload text instead of
/// the slot text: a `\` before anything else is kept, so a document that never
/// wrote one is unaffected.
pub fn materialize(body: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let mut chars = body.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some(escaped) => out.push(escaped),
            None => out.push('\\'),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(text: &str) -> Option<(MdpMode, String, usize)> {
        span(text, 0).map(|span| (span.mode, span.source(text), span.end))
    }

    #[test]
    fn reads_an_inline_span() {
        assert_eq!(
            found(";;*lorem*;;"),
            Some((MdpMode::Inline, "*lorem*".to_string(), 11))
        );
    }

    #[test]
    fn reads_a_block_span() {
        assert_eq!(
            found(";;;## A heading;;;"),
            Some((MdpMode::Block, "## A heading".to_string(), 18))
        );
    }

    #[test]
    fn the_longest_opener_wins() {
        assert_eq!(
            found(";;;x;;;").map(|(mode, ..)| mode),
            Some(MdpMode::Block)
        );
        assert_eq!(found(";;x;;").map(|(mode, ..)| mode), Some(MdpMode::Inline));
    }

    #[test]
    fn a_closing_run_is_consumed_whole() {
        // A `;;;` run closes a `;;` span and leaves no stray `;`.
        let span = span(";;a;;;", 0).expect("span");
        assert_eq!(span.end, 6);
        assert_eq!(span.source(";;a;;;"), "a");
    }

    #[test]
    fn the_interior_is_opaque() {
        let text = ";;a, b \"c\" [d] {e} (f): g;;";
        let span = span(text, 0).expect("span");
        assert_eq!(span.end, text.len());
    }

    #[test]
    fn a_short_run_does_not_close() {
        assert_eq!(
            found(";;a;b;;"),
            Some((MdpMode::Inline, "a;b".to_string(), 7))
        );
    }

    #[test]
    fn escapes_protect_a_closer_and_decode() {
        assert_eq!(
            found(r";;a\;b;;"),
            Some((MdpMode::Inline, "a;b".to_string(), 8))
        );
        assert_eq!(
            found(r";;a\nb;;"),
            Some((MdpMode::Inline, "a\nb".to_string(), 8))
        );
        assert_eq!(materialize(r"a\"), r"a\");
    }

    #[test]
    fn unclosed_spans_are_not_spans() {
        assert_eq!(found(";;a"), None);
        assert_eq!(found(";;;a;;"), None);
        assert_eq!(found("plain"), None);
        assert_eq!(found(");;"), None);
    }

    #[test]
    fn a_whole_value_is_exactly_one_span() {
        assert_eq!(
            whole(";;a;;"),
            Some(Renderable {
                mode: MdpMode::Inline,
                source: "a".to_string()
            })
        );
        assert_eq!(whole("  ;;a;;  ").map(|r| r.source), Some("a".to_string()));
        assert_eq!(whole(";;a;; b"), None);
        assert_eq!(whole("pre ;;a;;"), None);
        assert_eq!(whole(";;;;"), None);
        assert_eq!(whole(";; ;;"), None);
    }

    #[test]
    fn a_span_is_only_found_where_it_starts() {
        assert!(span("a;;b;;", 0).is_none());
        assert!(span("a;;b;;", 1).is_some());
    }
}
