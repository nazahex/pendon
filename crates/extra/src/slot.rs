//! Slot grammar: the `[…]` / `(…)` heads, read before Markdown reaches them
//! (`docs/spec/SYNTAX.md` §5.2–§5.4, ADR-0012, ADR-0017).
//!
//! A slot holds up to three *contents* in a fixed order — [`SlotKind::Bare`] →
//! [`SlotKind::Quote`] → [`SlotKind::Tick`]. The kind of a content is decided by
//! its **first character** (`"`/`'` quote, `` ` `` tick, anything else bare) and
//! a space is what ends a content; without a space the rest belongs to the
//! content before it:
//!
//! ```text
//! [bare"quote"`tick`]   one bare content: bare"quote"`tick`
//! [bare "quote"`tick`]  bare `bare`, quote `quote"`tick`
//! ```
//!
//! A head is a bracket, an *attached* paren (§4.1) or both, and the bracket may
//! carry a modifier run ([`SlotPair::modifiers`]). A tag sigil — `@`, `!` or `#`
//! before the `[`, `^` right after it — is read here too, but only in **head
//! position**: the caller passes the remaining line, so `foo#[bar]` is not a
//! slot head (§5.3, D-11).
//!
//! This module is pure: it never looks at Markdown, never decides whether a slot
//! is wanted (that is a plugin's [`SlotContract`]), and never falls back to
//! literal text — that is the caller's §4.3 duty, and both
//! [`SlotMatch::Absent`] and [`SlotMatch::Malformed`] mean "emit the text
//! verbatim".

use std::fmt;

/// One of the three contents a slot can carry, in canonical order.
///
/// The order is the grammar: contents must appear bare → quote → tick, once
/// each. The discriminants are declared in that order so `Ord`/`PartialOrd` and
/// a plain `as u8` compare agree with the spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SlotKind {
    /// Unquoted text: `[Foo]`.
    Bare,
    /// `"Foo"` or `'Foo'`: `[Foo "Bar"]`.
    Quote,
    /// `` `Foo` ``: `[Foo "Bar" `baz`]`.
    Tick,
}

impl SlotKind {
    /// The spec's order number: bare = 0, quote = 1, tick = 2.
    pub const fn order(self) -> u8 {
        self as u8
    }

    /// The config key suffix used to assign this content (§6.1, `field.*`).
    pub const fn name(self) -> &'static str {
        match self {
            SlotKind::Bare => "bare",
            SlotKind::Quote => "quote",
            SlotKind::Tick => "tick",
        }
    }

    /// Every kind, in canonical (bare → quote → tick) order.
    pub const ALL: [Self; 3] = [Self::Bare, Self::Quote, Self::Tick];

    /// The kind a config key names (`field.bracket.tick`), or `None`.
    ///
    /// The inverse of [`SlotKind::name`], so a typo can never be read as a
    /// position (it is a load error, D-16).
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.name() == name)
    }
}

/// Which of the two bracket kinds a slot is, in head order.
///
/// A head carries at most one of each, bracket first; the paren must be attached
/// to the bracket (§4.1). The discriminant is the index into
/// [`SlotPair::parts`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Slot {
    /// `[…]` — always the first slot of a head.
    Bracket,
    /// `(…)` — only after a bracket, never alone (§5.2).
    Paren,
}

impl Slot {
    /// How many slot kinds exist; the length of [`SlotPair::parts`].
    pub const COUNT: usize = 2;

    /// Index into [`SlotPair::parts`] and the `<slot>.<field>` config keys.
    pub const fn index(self) -> usize {
        self as usize
    }

    /// The config key prefix — `bracket.bare`, `paren.quote` (§5.2).
    pub const fn name(self) -> &'static str {
        match self {
            Self::Bracket => "bracket",
            Self::Paren => "paren",
        }
    }

    /// Every slot, in head order (bracket first).
    pub const ALL: [Self; Self::COUNT] = [Self::Bracket, Self::Paren];

    /// The slot a config key names (`field.bracket.bare`), or `None`.
    ///
    /// The inverse of [`Slot::name`]: an unknown prefix is a load error, never
    /// an ignored key (D-16).
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|slot| slot.name() == name)
    }
}

/// The sigil that may claim a head's slots (§5.3, D-11).
///
/// Tags are hard-coded here and matched **only in head position**. Whether the
/// owner element is enabled is the caller's business: a disabled owner makes the
/// head literal text with no diagnostic (the user did not opt into the syntax).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SlotTag {
    /// No sigil: `[x](y)`. Standing alone it is literal text; an element's
    /// [`SlotContract`] decides who claims it (D-11).
    #[default]
    None,
    /// `@[x](y)` — `plugin-anchor` (§7.2).
    Anchor,
    /// `[^ref]` — `plugin-cite` (§7.3). One caret: `crates/core` has no footnote
    /// extension, so `[^ref]` is never claimed before us (OBJ-1).
    Cite,
    /// `![alt](src)` — `plugin-image` (§7.1). The `!!` figure marker is that
    /// element's own concern, not a second tag.
    Image,
    /// `#[slug](title)` — `plugin-heading` (§7.4), and only when `[` follows at
    /// once, so `# Word` stays an ATX heading.
    Heading,
}

impl SlotTag {
    /// The sigil character, or `None` for an untagged head.
    pub const fn sigil(self) -> Option<char> {
        match self {
            Self::None => None,
            Self::Anchor => Some('@'),
            Self::Cite => Some('^'),
            Self::Image => Some('!'),
            Self::Heading => Some('#'),
        }
    }
}

/// The three contents of one slot (`[…]` or `(…)`), each optional.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SlotParts {
    /// `[Foo]`
    pub bare: Option<String>,
    /// `["Foo"]` / `['Foo']`
    pub quote: Option<String>,
    /// `` [`Foo`] ``
    pub tick: Option<String>,
}

impl SlotParts {
    /// True when every content is absent — `[]` and `()`.
    pub fn is_empty(&self) -> bool {
        self.bare.is_none() && self.quote.is_none() && self.tick.is_none()
    }

    /// The value of `kind`, if present.
    pub fn get(&self, kind: SlotKind) -> Option<&str> {
        match kind {
            SlotKind::Bare => self.bare.as_deref(),
            SlotKind::Quote => self.quote.as_deref(),
            SlotKind::Tick => self.tick.as_deref(),
        }
    }

    fn set(&mut self, kind: SlotKind, value: String) {
        match kind {
            SlotKind::Bare => self.bare = Some(value),
            SlotKind::Quote => self.quote = Some(value),
            SlotKind::Tick => self.tick = Some(value),
        }
    }
}

/// One head's slots: the tag that claimed it and one [`SlotParts`] per [`Slot`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SlotPair {
    /// The sigil that claimed the head ([`SlotTag::None`] when untagged).
    pub tag: SlotTag,
    /// The parsed contents, indexed by [`Slot::index`].
    pub parts: [SlotParts; Slot::COUNT],
    /// The bracket's modifier symbols (§5.4, ADR-0017): first-appearance order,
    /// duplicates already collapsed. A paren has no modifiers — `|…` inside `(…)`
    /// is ordinary content.
    pub modifiers: Vec<char>,
}

impl SlotPair {
    /// The contents of one slot.
    pub fn slot(&self, slot: Slot) -> &SlotParts {
        &self.parts[slot.index()]
    }

    /// The value of one position, if the head carried it.
    pub fn get(&self, slot: Slot, kind: SlotKind) -> Option<&str> {
        self.slot(slot).get(kind)
    }

    /// True when no content is present anywhere (`[]()`, `[|?x]`).
    pub fn is_empty(&self) -> bool {
        self.parts.iter().all(SlotParts::is_empty)
    }

    /// True when the bracket's modifier run contains `symbol`.
    pub fn has_modifier(&self, symbol: char) -> bool {
        self.modifiers.contains(&symbol)
    }
}

/// Why slot content was rejected (D-2 tier 2: an error, and the caller falls
/// back to literal text).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotError {
    /// A quote or backtick opened and never closed (`["abc]`).
    Unterminated(SlotKind),
    /// The same content appears twice (`[a b]`, `["x" "y"]`).
    Duplicate(SlotKind),
    /// Contents are not in bare → quote → tick order (`["x" bare]`).
    OutOfOrder {
        /// The kind that came first.
        before: SlotKind,
        /// The kind that came second, out of order.
        after: SlotKind,
    },
}

impl fmt::Display for SlotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unterminated(kind) => write!(
                f,
                "the {} content of a slot is opened and never closed",
                kind.name()
            ),
            Self::Duplicate(kind) => write!(
                f,
                "the {} content appears twice in one slot; each kind may appear once",
                kind.name()
            ),
            Self::OutOfOrder { before, after } => write!(
                f,
                "slot content must be bare, then quote, then tick; `{}` came after `{}`",
                after.name(),
                before.name()
            ),
        }
    }
}

/// What [`parse_slots`] found at the head of a line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlotMatch<'a> {
    /// Nothing slot-shaped: the text is literal, with no diagnostic (§4.3).
    /// This covers `(x)` alone, `[Foo` with no `]`, `[[wiki]]`, and a sigil
    /// without its bracket (`# Word`).
    Absent {
        /// The whole input, unconsumed.
        rest: &'a str,
    },
    /// A head was parsed; `rest` is what follows it.
    Slots {
        /// The tag and both slots.
        pair: SlotPair,
        /// The remaining line.
        rest: &'a str,
    },
    /// Slot-shaped but illegal: the caller emits `Severity::Error` and the text
    /// stays literal (D-2, tier 2). Nothing is consumed.
    Malformed {
        /// The grammar error.
        error: SlotError,
        /// The whole input, unconsumed.
        rest: &'a str,
    },
}

/// One position of a slot contract: which bracket, which content.
pub type SlotPos = (Slot, SlotKind);

/// Which slots an element accepts, in head order, and the tag that claims an
/// untagged standalone slot (§6.1, ADR-0012).
///
/// A plugin supplies only this — it never splits a bracket itself. The `accepted`
/// order is the head's written order, because the same `[x](y)` text is offered
/// to every enabled element and the first layer that accepts it wins (§6.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlotContract {
    /// The positions this element accepts, in head order.
    pub accepted: &'static [SlotPos],
    /// The tag that claims an untagged standalone slot, if any.
    pub tag: SlotTag,
}

impl SlotContract {
    /// True when this element accepts `slot`.`kind`.
    pub fn accepts(&self, slot: Slot, kind: SlotKind) -> bool {
        self.accepted.contains(&(slot, kind))
    }
}

/// Characters a backslash may escape inside slot content (§4.2). A backslash
/// before anything else stays literal.
const ESCAPABLE: &[char] = &[
    '"', '\'', '`', '[', ']', '(', ')', '{', '}', '#', '.', '|', '~', '\\', '@', '%',
];

/// Reads a slot head at the **start** of `rest` (§5.2, §5.3).
///
/// `rest` must already be the remaining line with its indentation consumed: a tag
/// sigil counts only in head position, so `foo#[bar]` never gets here. Only the
/// first two characters are inspected, and nothing is consumed on
/// [`SlotMatch::Absent`] or [`SlotMatch::Malformed`].
pub fn parse_slots(rest: &str) -> SlotMatch<'_> {
    let Some(head) = read_head(rest) else {
        return SlotMatch::Absent { rest };
    };

    // `[[…]]` is `plugin-wiki`'s opener (§7.5), not a slot.
    if rest[head.open..].starts_with("[[") {
        return SlotMatch::Absent { rest };
    }

    // Cite's tag sits *inside* the bracket, so it is not part of the content.
    let body = head.open + 1 + usize::from(head.tag == SlotTag::Cite);
    let Some(close) = find_close(rest, body, '[', ']') else {
        return SlotMatch::Absent { rest };
    };

    let mut parts: [SlotParts; Slot::COUNT] = Default::default();
    let scan = match scan_content(&rest[body..close], true) {
        Ok(scan) => scan,
        Err(error) => return SlotMatch::Malformed { error, rest },
    };
    parts[Slot::Bracket.index()] = scan.parts;

    let mut consumed = close + 1;
    if rest[consumed..].starts_with('(') {
        match read_paren(rest, consumed) {
            Ok(Some((paren, paren_end))) => {
                parts[Slot::Paren.index()] = paren;
                consumed = paren_end;
            }
            // An unterminated paren makes the whole head incomplete (§4.3).
            Ok(None) => return SlotMatch::Absent { rest },
            Err(error) => return SlotMatch::Malformed { error, rest },
        }
    }

    SlotMatch::Slots {
        pair: SlotPair {
            tag: head.tag,
            parts,
            modifiers: collect_modifiers(scan.run),
        },
        rest: &rest[consumed..],
    }
}

/// Parses the content of one slot — the text between `[` and `]` (or `(` and
/// `)`), with no modifier run.
///
/// This is the paren rule (§5.2): `|…` is ordinary content there. For a bracket,
/// call [`parse_slots`], which splits the run off first. An empty or
/// whitespace-only content (`[]`, `( )`) yields [`SlotParts::default`].
pub fn parse_slot_parts(content: &str) -> Result<SlotParts, SlotError> {
    scan_content(content, false).map(|scan| scan.parts)
}

/// The result of scanning one slot's content: the contents plus, for a bracket,
/// the modifier run that ended it (`""` when there is none).
struct Scan<'a> {
    parts: SlotParts,
    run: &'a str,
}

/// Scans one slot's content (§5.2).
///
/// With `run_allowed` (a bracket) an unescaped `|` that starts a word opens the
/// modifier run and ends the content (§5.4); without it (a paren) `|` is just
/// another character.
fn scan_content(content: &str, run_allowed: bool) -> Result<Scan<'_>, SlotError> {
    let mut parts = SlotParts::default();
    let mut previous: Option<SlotKind> = None;
    let mut run = "";
    let mut cursor = 0usize;

    while let Some(ch) = content[cursor..].chars().next() {
        if ch.is_whitespace() {
            cursor += ch.len_utf8();
            continue;
        }

        if run_allowed && ch == '|' {
            run = &content[cursor + 1..];
            break;
        }

        let kind = match ch {
            '"' | '\'' => SlotKind::Quote,
            '`' => SlotKind::Tick,
            _ => SlotKind::Bare,
        };

        let (value, next) = match kind {
            SlotKind::Bare => read_bare(content, cursor, run_allowed),
            SlotKind::Quote | SlotKind::Tick => read_delimited(content, cursor, ch, kind)?,
        };

        if let Some(before) = previous {
            if before == kind {
                return Err(SlotError::Duplicate(kind));
            }
            if before.order() > kind.order() {
                return Err(SlotError::OutOfOrder {
                    before,
                    after: kind,
                });
            }
        }

        parts.set(kind, value);
        previous = Some(kind);
        cursor = next;
    }

    Ok(Scan { parts, run })
}

/// A head that starts with a tag sigil and a bracket.
struct Head {
    tag: SlotTag,
    /// Byte index of the `[`.
    open: usize,
}

/// Matches a tag sigil in head position (§5.3).
///
/// `@`, `!` and `#` sit **before** the bracket; cite's `^` sits right after it,
/// because the bracket is what makes it a slot rather than footnote text.
fn read_head(rest: &str) -> Option<Head> {
    let mut chars = rest.char_indices();
    let (_, first) = chars.next()?;
    let (second_at, second) = chars.next()?;

    let (tag, open) = match (first, second) {
        ('[', '^') => (SlotTag::Cite, 0),
        ('[', _) => (SlotTag::None, 0),
        ('@', '[') => (SlotTag::Anchor, second_at),
        ('!', '[') => (SlotTag::Image, second_at),
        ('#', '[') => (SlotTag::Heading, second_at),
        _ => return None,
    };

    Some(Head { tag, open })
}

/// Reads the paren that must be attached to the bracket (§4.1).
///
/// `Ok(None)` means the paren never closes, which makes the head incomplete: the
/// caller must not consume the bracket either (§4.3).
fn read_paren(rest: &str, open: usize) -> Result<Option<(SlotParts, usize)>, SlotError> {
    let body = open + 1;
    let Some(close) = find_close(rest, body, '(', ')') else {
        return Ok(None);
    };
    let scan = scan_content(&rest[body..close], false)?;
    Ok(Some((scan.parts, close + 1)))
}

/// Byte index of the closer that matches the opener just before `body`,
/// honouring §4.2 escapes and counting nested openers (so a paren inside a URL
/// such as `(a(b).png)` does not close the slot early).
fn find_close(text: &str, body: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 1usize;
    let mut escaped = false;

    for (offset, ch) in text[body..].char_indices() {
        if escaped {
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == open {
            depth += 1;
        } else if ch == close {
            depth -= 1;
            if depth == 0 {
                return Some(body + offset);
            }
        }
    }

    None
}

/// The symbols of a bracket's modifier run, in first-appearance order and
/// deduplicated (ADR-0017 rules 1–2). Whitespace separates symbols and is
/// dropped, so `|?xx?|ee?` and `|?x|e` are the same run.
fn collect_modifiers(run: &str) -> Vec<char> {
    let mut symbols: Vec<char> = Vec::new();
    for ch in run.chars() {
        if ch.is_whitespace() || symbols.contains(&ch) {
            continue;
        }
        symbols.push(ch);
    }
    symbols
}

/// Reads a bare content: text up to the point where a space is followed by a
/// character that opens a new content — or the end of the slot.
///
/// Bare text may contain spaces (`[Foo Bar]` is one bare content, §5.2 "no
/// prohibition on spaces in bare"), and `bare\"quoted\"` stays bare because the
/// delimiter is escaped. Only an unescaped `"`, `'` or `` ` `` — or a `|` that
/// starts the modifier run (`run_allowed`) — ends it.
fn read_bare(content: &str, start: usize, run_allowed: bool) -> (String, usize) {
    let mut end = content.len();
    let mut cursor = start;

    while let Some(ch) = content[cursor..].chars().next() {
        if ch.is_whitespace() {
            let after = cursor + ch.len_utf8();
            match content[after..]
                .find(|c: char| !c.is_whitespace())
                .map(|offset| after + offset)
            {
                Some(next) => match content[next..].chars().next() {
                    Some(opens) if opens_content(opens, run_allowed) => {
                        end = cursor;
                        break;
                    }
                    _ => {
                        cursor = next;
                        continue;
                    }
                },
                None => {
                    end = cursor;
                    break;
                }
            }
        }
        cursor += ch.len_utf8();
    }

    (unescape(&content[start..end]), end)
}

/// The three characters that start a quote or tick content.
fn is_delimiter(ch: char) -> bool {
    matches!(ch, '"' | '\'' | '`')
}

/// Characters that begin a new content, so that a space before them ends the
/// bare content. `|` does it only where a modifier run is possible (a bracket).
fn opens_content(ch: char, run_allowed: bool) -> bool {
    is_delimiter(ch) || (run_allowed && ch == '|')
}

/// Reads a quote or tick content starting at `start`.
///
/// A closing delimiter terminates the content only when the next character is a
/// separator or the end of the content. Otherwise the content runs to the next
/// separator, keeping the closing delimiter — `"quote"`tick`` is one *quote*
/// content, not a quote plus a tick (§5.2).
fn read_delimited(
    content: &str,
    start: usize,
    delimiter: char,
    kind: SlotKind,
) -> Result<(String, usize), SlotError> {
    let body_start = start + delimiter.len_utf8();
    let mut cursor = body_start;
    let mut escaped = false;

    while let Some(ch) = content[cursor..].chars().next() {
        if escaped {
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == delimiter {
            let close_end = cursor + delimiter.len_utf8();
            let at_boundary = content[close_end..]
                .chars()
                .next()
                .is_none_or(char::is_whitespace);
            if at_boundary {
                return Ok((unescape(&content[body_start..cursor]), close_end));
            }
            let end = content[close_end..]
                .find(char::is_whitespace)
                .map_or(content.len(), |offset| close_end + offset);
            return Ok((unescape(&content[body_start..end]), end));
        }
        cursor += ch.len_utf8();
    }

    Err(SlotError::Unterminated(kind))
}

/// Resolves §4.2 escapes inside slot content: `\"` → `"`, `` \` `` → `` ` ``,
/// `\|` → `|`, `\\` → `\`. A backslash before any other character is literal and
/// preserved.
fn unescape(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars();

    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        match chars.next() {
            Some(next) if ESCAPABLE.contains(&next) => out.push(next),
            Some(next) => {
                out.push('\\');
                out.push(next);
            }
            None => out.push('\\'),
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parts(content: &str) -> SlotParts {
        parse_slot_parts(content).expect("expected valid slot content")
    }

    fn slots(source: &str) -> SlotPair {
        match parse_slots(source) {
            SlotMatch::Slots { pair, .. } => pair,
            other => panic!("expected a slot head for {source:?}, got {other:?}"),
        }
    }

    fn trailing(source: &str) -> &str {
        match parse_slots(source) {
            SlotMatch::Slots { rest, .. } => rest,
            other => panic!("expected a slot head for {source:?}, got {other:?}"),
        }
    }

    fn absent(source: &str) {
        match parse_slots(source) {
            SlotMatch::Absent { rest } => assert_eq!(rest, source),
            other => panic!("expected literal text for {source:?}, got {other:?}"),
        }
    }

    #[test]
    fn reads_all_three_contents() {
        let got = parts(r#"bare "quote" `tick`"#);
        assert_eq!(got.bare.as_deref(), Some("bare"));
        assert_eq!(got.quote.as_deref(), Some("quote"));
        assert_eq!(got.tick.as_deref(), Some("tick"));
    }

    #[test]
    fn without_a_space_the_rest_is_one_bare_content() {
        // §5.2's own example: no separator, so the quotes and the tick stay bare.
        let got = parts(r#"bare"quote"`tick`"#);
        assert_eq!(got.bare.as_deref(), Some(r#"bare"quote"`tick`"#));
        assert_eq!(got.quote, None);
        assert_eq!(got.tick, None);
    }

    #[test]
    fn an_unseparated_closer_extends_the_content() {
        // §5.2's second example: `"quote"`tick`` is one quote content.
        let got = parts(r#"bare "quote"`tick`"#);
        assert_eq!(got.bare.as_deref(), Some("bare"));
        assert_eq!(got.quote.as_deref(), Some(r#"quote"`tick`"#));
        assert_eq!(got.tick, None);
    }

    #[test]
    fn bare_content_may_contain_spaces() {
        let got = parts("Foo Bar");
        assert_eq!(got.bare.as_deref(), Some("Foo Bar"));
    }

    #[test]
    fn escapes_keep_delimiters_inside_bare() {
        // §5.2: `[bare \"bare\" bare "quote"]`.
        let got = parts(r#"bare \"bare\" bare "quote""#);
        assert_eq!(got.bare.as_deref(), Some(r#"bare "bare" bare"#));
        assert_eq!(got.quote.as_deref(), Some("quote"));
    }

    #[test]
    fn escapes_inside_a_tick() {
        let got = parts(r"`a\`b`");
        assert_eq!(got.tick.as_deref(), Some("a`b"));
    }

    #[test]
    fn single_quotes_are_quote_content() {
        let got = parts("'q'");
        assert_eq!(got.quote.as_deref(), Some("q"));
    }

    #[test]
    fn empty_contents() {
        for content in ["", "   "] {
            assert!(parts(content).is_empty(), "content {content:?}");
        }
        let got = parts("\"\" `tick`");
        assert_eq!(got.quote.as_deref(), Some(""));
        assert_eq!(got.tick.as_deref(), Some("tick"));
    }

    #[test]
    fn duplicate_quote_is_an_error() {
        assert_eq!(
            parse_slot_parts(r#""a" "b""#),
            Err(SlotError::Duplicate(SlotKind::Quote))
        );
        assert_eq!(
            parse_slot_parts(r"`a` `b`"),
            Err(SlotError::Duplicate(SlotKind::Tick))
        );
    }

    #[test]
    fn order_is_bare_quote_tick() {
        assert_eq!(
            parse_slot_parts(r#""q" bare"#),
            Err(SlotError::OutOfOrder {
                before: SlotKind::Quote,
                after: SlotKind::Bare,
            })
        );
        assert_eq!(
            parse_slot_parts(r"`t` bare"),
            Err(SlotError::OutOfOrder {
                before: SlotKind::Tick,
                after: SlotKind::Bare,
            })
        );
    }

    #[test]
    fn unterminated_delimiters_are_errors() {
        assert_eq!(
            parse_slot_parts(r#""abc"#),
            Err(SlotError::Unterminated(SlotKind::Quote))
        );
        assert_eq!(
            parse_slot_parts("`abc"),
            Err(SlotError::Unterminated(SlotKind::Tick))
        );
    }

    #[test]
    fn bare_keeps_an_unescaped_backtick() {
        let got = parts("a`b");
        assert_eq!(got.bare.as_deref(), Some("a`b"));
    }

    #[test]
    fn kind_order_and_names_match_the_spec() {
        assert_eq!(SlotKind::Bare.order(), 0);
        assert_eq!(SlotKind::Quote.order(), 1);
        assert_eq!(SlotKind::Tick.order(), 2);
        assert_eq!(SlotKind::Bare.name(), "bare");
        assert_eq!(SlotKind::Quote.name(), "quote");
        assert_eq!(SlotKind::Tick.name(), "tick");
        assert_eq!(Slot::Bracket.index(), 0);
        assert_eq!(Slot::Paren.index(), 1);
        assert_eq!(Slot::Bracket.name(), "bracket");
        assert_eq!(Slot::Paren.name(), "paren");
    }

    /// A config key's names round-trip, and a typo is `None` — never a silent
    /// default (`field.bracket.tikc` is a load error).
    #[test]
    fn names_are_the_inverse_of_the_config_keys() {
        for kind in SlotKind::ALL {
            assert_eq!(SlotKind::from_name(kind.name()), Some(kind));
        }
        for slot in Slot::ALL {
            assert_eq!(Slot::from_name(slot.name()), Some(slot));
        }
        assert_eq!(SlotKind::from_name("bare "), None);
        assert_eq!(SlotKind::from_name("Bare"), None);
        assert_eq!(Slot::from_name("brackets"), None);
        assert_eq!(Slot::from_name("specs"), None);
        assert_eq!(Slot::from_name(""), None);
    }

    // ---- S2-1.2: both slots and adjacency (§5.2, §4.1) ----

    #[test]
    fn empty_slots() {
        let got = slots("[]()");
        assert!(got.is_empty());
        assert_eq!(got.tag, SlotTag::None);
        assert_eq!(trailing("[]()"), "");
    }

    #[test]
    fn both_slots_are_read() {
        let got = slots(r#"[Foo Bar](url "Title")"#);
        assert_eq!(got.get(Slot::Bracket, SlotKind::Bare), Some("Foo Bar"));
        assert_eq!(got.get(Slot::Paren, SlotKind::Bare), Some("url"));
        assert_eq!(got.get(Slot::Paren, SlotKind::Quote), Some("Title"));
        assert_eq!(got.get(Slot::Paren, SlotKind::Tick), None);
    }

    #[test]
    fn the_paren_must_be_attached() {
        // §4.1: a space ends adjacency, so `(y)` is somebody else's text.
        let got = slots("[x] (y)");
        assert_eq!(got.get(Slot::Bracket, SlotKind::Bare), Some("x"));
        assert_eq!(got.get(Slot::Paren, SlotKind::Bare), None);
        assert_eq!(trailing("[x] (y)"), " (y)");
    }

    #[test]
    fn a_paren_without_a_bracket_is_not_a_slot() {
        absent("(x)");
        absent("(x)(y)");
    }

    #[test]
    fn trailing_text_is_returned() {
        assert_eq!(trailing("[x](y) tail"), " tail");
        assert_eq!(trailing("[x]"), "");
    }

    #[test]
    fn incomplete_heads_are_literal() {
        // D-2 tier 1: incomplete grammar, no diagnostic.
        absent("[Foo");
        absent("[Foo](bar");
    }

    #[test]
    fn wikilinks_are_left_to_their_own_element() {
        absent("[[Page]]");
        absent("[[Page|Label]]");
    }

    // ---- S2-1.3: tags in head position only (§5.3, D-11) ----

    #[test]
    fn each_tag_is_read_in_head_position() {
        assert_eq!(slots("@[x](y)").tag, SlotTag::Anchor);
        assert_eq!(slots("![alt](src)").tag, SlotTag::Image);
        assert_eq!(slots("#[slug](title)").tag, SlotTag::Heading);
        assert_eq!(slots("[^ref]").tag, SlotTag::Cite);
        assert_eq!(slots("[x](y)").tag, SlotTag::None);
    }

    #[test]
    fn a_tag_is_not_part_of_the_content() {
        let got = slots("#[slug](title)");
        assert_eq!(got.get(Slot::Bracket, SlotKind::Bare), Some("slug"));
        assert_eq!(got.get(Slot::Paren, SlotKind::Bare), Some("title"));
        // Cite's caret sits inside the bracket, so it belongs to the tag.
        assert_eq!(
            slots("[^ref]").get(Slot::Bracket, SlotKind::Bare),
            Some("ref")
        );
        assert_eq!(
            slots("![alt](src)").get(Slot::Bracket, SlotKind::Bare),
            Some("alt")
        );
    }

    #[test]
    fn a_sigil_without_its_bracket_is_literal() {
        absent("# Word"); // an ATX heading, not a slot (§7.4)
        absent("@(x)");
        absent("#");
        absent("!![a](b)"); // `!!` is the figure marker, not two tags
    }

    #[test]
    fn tags_are_not_read_mid_text() {
        absent("foo#[bar]");
        absent("x[^1]");
        absent("a![b](c)");
    }

    #[test]
    fn tag_sigils_match_the_spec() {
        assert_eq!(SlotTag::Anchor.sigil(), Some('@'));
        assert_eq!(SlotTag::Cite.sigil(), Some('^'));
        assert_eq!(SlotTag::Image.sigil(), Some('!'));
        assert_eq!(SlotTag::Heading.sigil(), Some('#'));
        assert_eq!(SlotTag::None.sigil(), None);
    }

    // ---- S2-1.5: the bracket modifier run (§5.4, ADR-0017) ----

    #[test]
    fn modifier_run_keeps_written_order() {
        let got = slots("[Foo |?~uv$%]");
        assert_eq!(got.get(Slot::Bracket, SlotKind::Bare), Some("Foo"));
        assert_eq!(got.modifiers, vec!['?', '~', 'u', 'v', '$', '%']);
    }

    #[test]
    fn modifier_repetition_collapses() {
        // ADR-0017 rule 2: `|?xx?|ee?` ≡ `|?x|e`, and `||||` ≡ `||`.
        assert_eq!(slots("[Foo |?xx?|ee?]").modifiers, vec!['?', 'x', '|', 'e']);
        assert_eq!(slots("[Foo |?x|e]").modifiers, vec!['?', 'x', '|', 'e']);
        assert_eq!(slots("[||||]").modifiers, vec!['|']);
        assert_eq!(slots("[||]").modifiers, vec!['|']);
    }

    #[test]
    fn modifiers_need_no_content() {
        let got = slots("[|?xu*](foo.png)");
        assert_eq!(got.get(Slot::Bracket, SlotKind::Bare), None);
        assert_eq!(got.modifiers, vec!['?', 'x', 'u', '*']);
        assert_eq!(got.get(Slot::Paren, SlotKind::Bare), Some("foo.png"));
        assert!(got.has_modifier('*'));
        assert!(!got.has_modifier('~'));
    }

    #[test]
    fn an_escaped_pipe_stays_content() {
        let got = slots(r"[Foo \|?x]");
        assert_eq!(got.get(Slot::Bracket, SlotKind::Bare), Some("Foo |?x"));
        assert!(got.modifiers.is_empty());
    }

    #[test]
    fn an_escaped_pipe_after_a_quote_is_an_error() {
        // ADR-0017 rule 6: it would put bare after quote (slot order, §5.2).
        assert_eq!(
            parse_slots(r#"[Foo "Bar" \|?x]"#),
            SlotMatch::Malformed {
                error: SlotError::OutOfOrder {
                    before: SlotKind::Quote,
                    after: SlotKind::Bare,
                },
                rest: r#"[Foo "Bar" \|?x]"#,
            }
        );
    }

    #[test]
    fn modifiers_are_bracket_only() {
        // ADR-0017 rule 5: in `(…)` a `|…` is literal content, not an error.
        let got = slots("[Foo](bar |?x)");
        assert_eq!(got.get(Slot::Paren, SlotKind::Bare), Some("bar |?x"));
        assert!(got.modifiers.is_empty());
    }

    // ---- S2-1.6: slot contracts (§6.1) ----

    /// A grammar fixture, **not** an element's contract: the real ones are
    /// declared next to their element (`plugin_anchor::HEAD`, …) and that is the
    /// only copy (§6.1). This one just has a shape to test `accepts` with.
    const FIXTURE: SlotContract = SlotContract {
        accepted: &[
            (Slot::Bracket, SlotKind::Bare),
            (Slot::Paren, SlotKind::Bare),
            (Slot::Paren, SlotKind::Quote),
        ],
        tag: SlotTag::Anchor,
    };

    #[test]
    fn a_contract_accepts_the_positions_it_lists_and_no_others() {
        assert!(FIXTURE.accepts(Slot::Bracket, SlotKind::Bare));
        assert!(FIXTURE.accepts(Slot::Paren, SlotKind::Bare));
        assert!(FIXTURE.accepts(Slot::Paren, SlotKind::Quote));
        assert!(!FIXTURE.accepts(Slot::Bracket, SlotKind::Quote));
        assert!(!FIXTURE.accepts(Slot::Bracket, SlotKind::Tick));
        assert!(!FIXTURE.accepts(Slot::Paren, SlotKind::Tick));
        // Listing a position and then not accepting it is impossible.
        for &(slot, kind) in FIXTURE.accepted {
            assert!(FIXTURE.accepts(slot, kind));
        }
    }

    #[test]
    fn an_untagged_head_is_claimed_through_a_contract() {
        // D-11: an untagged `[x](y)` parses as `SlotTag::None`; whether an
        // element may claim it is that element's `SlotContract::tag`.
        assert_eq!(slots("[x](y)").tag, SlotTag::None);
        assert_eq!(FIXTURE.tag, SlotTag::Anchor);
    }
}
