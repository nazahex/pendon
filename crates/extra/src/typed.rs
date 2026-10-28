//! Parser for the typed extras head (`@@type{…}` / `{…}` / `@@type`) and for
//! directive heads, implementing `docs/spec/SYNTAX.md` §4–§6.
//!
//! Three rules dominate this module:
//!
//! * **Two spellings (§3).** The bare `{…}` head and the `@@`-prefixed head
//!   (`@@type{…}`, `@@{…}`, `@@type`) are equivalent; the prefix only adds the
//!   optional type marker.
//! * **Literal fallback (§4.3):** a head that looks like a head but is malformed
//!   is never dropped and never partially applied. The caller gets
//!   [`ExtrasMatch::Malformed`] and MUST render the original text verbatim.
//! * **Adjacency (§4.1):** nothing is trimmed from the front, and a type is not
//!   separated from its `{` by whitespace. The caller passes the text at the
//!   exact cursor position, so `x@@type{…}` and `@@type {…}` are *not* heads.
//!   A type that ends on a symbol (`@@anchorA.`) is a type-only head and the
//!   symbol stays literal text.

use crate::field::{FieldConfig, Path, SPECS_NAMESPACE};
use crate::preset::fallback_path;
use crate::slot::{Slot, SlotKind};
use crate::value::{classify_scalar, AttrValue, SPREAD_KEY};

/// Why a head was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtrasError {
    /// `@@` is not followed by a type name or `{`.
    InvalidHead,
    /// `{` opened but never closed (or the type name runs to the end).
    UnterminatedHead,
    /// An item is neither a class, an id, a positional value, a prop, a CSS var
    /// nor a well-formed bare flag.
    InvalidItem,
    /// `key:` with nothing after it.
    EmptyValue,
    /// A quote or backtick opened but never closed.
    UnterminatedValue,
    /// Text after the closing quote of a quoted value.
    TrailingCharacters,
    /// An unquoted value contains whitespace (quotes are mandatory, §6.3).
    UnquotedWhitespace,
}

/// One item inside the extras body.
#[derive(Debug, Clone, PartialEq)]
pub enum ExtrasItem {
    /// `.name`
    Class(String),
    /// `#name`
    Id(String),
    /// `` `value` `` — mapped to the `tick` name (`slug` by default).
    Slug(String),
    /// `"value"` / `'value'` — mapped to the `quote` name (`title` by default).
    Title(String),
    /// `key: value`
    Prop { key: String, value: AttrValue },
    /// `--name: value` — folded into `style` (§6.3).
    CssVar { name: String, value: String },
    /// `isFoo` — rendered verbatim.
    Flag { name: String },
    /// `%%numberType` — a numbering marker (ADR-0010).
    Number(NumberMark),
}

/// The `%%numberType` numbering marker (§5, ADR-0010).
///
/// `depth` is **already normalized** — the `%` run length − 1, so `%%` is depth
/// 1 and `%%%%%%%` is depth 6. Normalization happens here, once, because two
/// parsers of `%` would be the layering violation `CONTRIBUTORS.md` forbids;
/// `plugin-number` only reads the depth back out of `__number`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberMark {
    /// Normalized depth, 1…=6 (the `%` count minus one).
    pub depth: u8,
    /// The `numberType` the marker names (`numberB`).
    pub number_type: String,
}

/// Reads a `%%numberType` marker off the front of `input`, returning it and the
/// bytes consumed.
///
/// The shape is `%{2,7}` immediately followed by a name (`[A-Za-z][A-Za-z0-9]*`)
/// and then a non-name character or the end. Anything else is `None`, i.e.
/// literal text: a bare `%`, `%%` with no name, a run of 8+ `%` and a name that
/// runs into punctuation all stay exactly as authored (D-9, no diagnostic).
pub fn read_number_mark(input: &str) -> Option<(NumberMark, usize)> {
    let bytes = input.as_bytes();
    let mut run = 0usize;
    while matches!(bytes.get(run), Some(b'%')) {
        run += 1;
    }
    if !(2..=7).contains(&run) {
        return None;
    }
    if !matches!(bytes.get(run), Some(byte) if byte.is_ascii_alphabetic()) {
        return None;
    }
    let mut cursor = run + 1;
    while matches!(bytes.get(cursor), Some(byte) if byte.is_ascii_alphanumeric()) {
        cursor += 1;
    }
    Some((
        NumberMark {
            depth: (run - 1) as u8,
            number_type: input[run..cursor].to_string(),
        },
        cursor,
    ))
}

/// A parsed `@@type{…}` / `@@{…}` head.
///
/// The positional groups of §6.1 may sit between the type and the body:
/// `@@type[…](…){…}` (and `@@[…](…)` without a type or a body).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExtrasHead {
    /// `type` of `@@type{…}`; `None` for `@@{…}`.
    pub type_marker: Option<String>,
    /// Raw contents of the optional `[…]` group (name = `bracket`).
    pub bracket: Option<String>,
    /// Raw contents of the optional `(\u2026)` group, unquoted
    /// (name = `paren`).
    pub parentheses: Option<String>,
    pub items: Vec<ExtrasItem>,
}

/// The **specs** of an extras head: the public field each of its positional
/// contents lands in (§5, §6.1).
///
/// A head carries at most seven positional contents — the `[...]` / `(...)` **slot
/// groups**, the `` `...` `` / `"..."` **specs items**, and the `.class` / `#id` /
/// `style:` items — and each lands in exactly one field. The vocabularies are
/// named together here because [`to_attributes`] maps a whole head in one pass,
/// and because all of them answer the same question: *where does this content
/// go?* The five `specs.*` rows are the [`SpecsKey`] positions.
///
/// | content        | field (`field.*` key)                | default      | meaning                                            |
/// | -------------- | ------------------------------------ | ------------ | -------------------------------------------------- |
/// | `` `x` `` tick | `field.specs.tick`                   | `specs.tick` | the construct's short identifier (its slug)         |
/// | `"x"` quote    | `field.specs.quote`                  | `specs.quote`| the construct's human-readable title                |
/// | `[x]` bracket  | `field.bracket.bare`                 | `bracket.bare`| the content of the bracket slot                     |
/// | `("x")` paren  | `field.paren.bare`                   | `paren.bare` | the content of the parentheses slot                 |
/// | `.class`       | `field.specs.class`                  | `specs.class`| CSS class list                                      |
/// | `#id`          | `field.specs.id`                     | `specs.id`   | element ID                                          |
/// | `style:`       | `field.specs.style`                  | `specs.style`| inline style declarations                           |
///
/// The `meaning` column is the one piece of this table that exists **only**
/// here: an assigned field carries no semantics of its own, so a rename never
/// changes what the content *is*. The `default` column is
/// [`SpecsKey::default_path`] — the `specs.*` paths a construct keeps when its
/// `custom` entry configures nothing (D-7).
///
/// A **path** is not a **position**: once `field.specs.<k>` moves a position
/// elsewhere, the assigned path is the author's own public field, and the
/// element stops reading the content at all — see [`SpecsKey`] and
/// [`element_path`](Self::element_path).
///
/// An entry's `field.*` keys move those contents (D-5);
/// [`resolve`](Self::resolve) is the only place that reads them for naming.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Specs {
    /// `[...]` — `field.bracket.bare`.
    pub bracket: String,
    /// `(...)` — `field.paren.bare`.
    pub paren: String,
    /// `` `...` `` — `field.specs.tick`.
    pub tick: String,
    /// `"..."` — `field.specs.quote`.
    pub quote: String,
    /// `.class` / `class: "x"` — `field.specs.class`.
    pub class: String,
    /// `#id` — `field.specs.id`.
    pub id: String,
    /// `style:` / `--var` — `field.specs.style`.
    pub style: String,
}

impl Default for Specs {
    fn default() -> Self {
        Self {
            bracket: "bracket.bare".to_string(),
            paren: "paren.bare".to_string(),
            tick: specs_path(SpecsKey::Tick).dotted(),
            quote: specs_path(SpecsKey::Quote).dotted(),
            class: specs_path(SpecsKey::Class).dotted(),
            id: specs_path(SpecsKey::Id).dotted(),
            style: specs_path(SpecsKey::Style).dotted(),
        }
    }
}

impl Specs {
    /// The two **specs item** names, leaving the slot groups at their built-ins (§6.1).
    pub fn new(tick: impl Into<String>, quote: impl Into<String>) -> Self {
        Self {
            tick: tick.into(),
            quote: quote.into(),
            ..Self::default()
        }
    }

    /// The specs the matched `custom` entry configures (D-5, D-7).
    ///
    /// `None` — no entry, or an entry that configured no `field.*` key — keeps
    /// every built-in name, which is the D-7 "preset is alive" case. Otherwise
    /// each name is the entry's assignment, and a position it left **unassigned**
    /// falls back to its own qualified spelling: `bracket.bare` / `paren.bare` for
    /// the slot groups (D-7) and `specs.tick` / `specs.quote` for the specs items
    /// (§6.3, S2-3.2). A name the user chose is therefore never silently reused
    /// for an unnamed position, and an unnamed value is never lost.
    pub fn resolve(config: Option<&FieldConfig>) -> Self {
        let Some(config) = config.filter(|config| !config.is_empty()) else {
            return Self::default();
        };
        Self {
            bracket: slot_name(config, Slot::Bracket, SlotKind::Bare),
            paren: slot_name(config, Slot::Paren, SlotKind::Bare),
            tick: specs_name(config, SpecsKey::Tick),
            quote: specs_name(config, SpecsKey::Quote),
            class: specs_name(config, SpecsKey::Class),
            id: specs_name(config, SpecsKey::Id),
            style: specs_name(config, SpecsKey::Style),
        }
    }
}

/// One of the five **specs positions** (§6.1, [`crate::field::SPECS_KEYS`]).
///
/// A position is what an element may attach meaning to — a construct's slug, its
/// title, an element id, its class list, its inline style. A *path* is where that
/// content is published, and `field.specs.<k>` moves it (§6.1, D-5).
///
/// Only the **default** path carries meaning: once an entry assigns a position
/// elsewhere (`field.specs.quote = "_extra-note"`), the content is the author's
/// own public field, and the element neither reads it (§6.2) nor renames it
/// (§6.3) — see [`Specs::element_path`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecsKey {
    /// `` `x` `` — `field.specs.tick`, default path `specs.tick`.
    Tick,
    /// `"x"` — `field.specs.quote`, default path `specs.quote`.
    Quote,
    /// `.x` / `class: "x"` — `field.specs.class`, default path `specs.class`.
    Class,
    /// `#x` — `field.specs.id`, default path `specs.id`.
    Id,
    /// `style:` / `--var` — `field.specs.style`, default path `specs.style`.
    Style,
}

impl SpecsKey {
    /// The `<k>` in the `field.specs.<k>` config key (§7, D-5).
    pub fn key(self) -> &'static str {
        match self {
            Self::Tick => "tick",
            Self::Quote => "quote",
            Self::Class => "class",
            Self::Id => "id",
            Self::Style => "style",
        }
    }

    /// The path a position keeps when no `field.specs.<k>` assigns it: the one
    /// spelling that carries element meaning (§6.1, §6.3).
    pub fn default_path(self) -> &'static str {
        match self {
            Self::Tick => "specs.tick",
            Self::Quote => "specs.quote",
            Self::Class => "specs.class",
            Self::Id => "specs.id",
            Self::Style => "specs.style",
        }
    }
}

impl Specs {
    /// The resolved path of `position` — the entry's assignment or, when it left
    /// the position alone, its [`default_path`](SpecsKey::default_path).
    pub fn path(&self, position: SpecsKey) -> &str {
        match position {
            SpecsKey::Tick => &self.tick,
            SpecsKey::Quote => &self.quote,
            SpecsKey::Class => &self.class,
            SpecsKey::Id => &self.id,
            SpecsKey::Style => &self.style,
        }
    }

    /// The path an **element** may read `position` from: the
    /// [`default`](SpecsKey::default_path) spelling, or `None` when the entry
    /// moved the position with `field.specs.<k>` (D-5).
    ///
    /// The assignment *is* the rename (§6.1), so a moved position is published
    /// under the name the author chose and nothing else: with
    /// `field.specs.quote = "_extra-note"` an anchor's `"…"` item is the
    /// `_extra-note` field — not its `title` attribute, and not a second copy
    /// under both names.
    pub fn element_path(&self, position: SpecsKey) -> Option<&'static str> {
        let default = position.default_path();
        (self.path(position) == default).then_some(default)
    }

    /// Projects a head's attribute paths onto the field names its element and the
    /// DOM use (§6.3).
    ///
    /// Three steps, in order:
    ///
    /// 1. `element` names the positions this element interprets as `(position,
    ///    the field it feeds)` pairs — a citation reads its [`Id`](SpecsKey::Id)
    ///    position into `cite-id`, a heading its [`Quote`](SpecsKey::Quote) into
    ///    `title`. Only a position that still sits on its
    ///    [`default path`](Self::element_path) matches, so a `field.specs.<k>`
    ///    assignment is never captured by the element.
    /// 2. The **universal** positions written as their DOM field:
    ///    `specs.class` / `specs.id` / `specs.style` → `class` / `id` / `style`
    ///    ([`universal_field`]). A moved path is not one of them, so `foo.class`
    ///    stays `foo.class`.
    /// 3. Every other key is a public field of its own (`k: v` stays `k`,
    ///    D-12), returned unchanged — `specs.tick`, `bracket.bare`, an assigned
    ///    path.
    ///
    /// A position `element` does not name keeps its dotted path, which is what
    /// the JSON/AST renderers carry and what a phase-4 preset fills in.
    pub fn dom_fields<'a>(
        &self,
        items: &'a [(String, ExtrasAttr)],
        element: &[(SpecsKey, &'a str)],
    ) -> Vec<(&'a str, &'a ExtrasAttr)> {
        items
            .iter()
            .map(|(key, value)| {
                // The element's own reading wins at the default path; the
                // universal DOM field is the next fallback, and a key that is
                // neither is already a field of its own.
                let field = element
                    .iter()
                    .find(|(position, _)| self.element_path(*position) == Some(key.as_str()))
                    .map(|(_, field)| *field)
                    .or_else(|| universal_field(key))
                    .unwrap_or(key.as_str());
                (field, value)
            })
            .collect()
    }
}

/// The DOM field a **universal** specs path is written as (§6.3): `specs.class`
/// → `class`, `specs.id` → `id`, `specs.style` → `style`.
///
/// `tick` / `quote` and the two slot groups are not universal — what a slug or a
/// title means is the element's business (§7) — so an element names them through
/// [`Specs::dom_fields`]. This is the default-path-only spelling the projection
/// ([`crate::field::project`]) can apply without a `FieldConfig`. A path
/// `field.specs.*` assigned is **not** universal: `foo.class` stays `foo.class`
/// (§6.3 rule 2, [`Specs::element_path`]).
pub fn universal_field(key: &str) -> Option<&'static str> {
    let key = key.strip_prefix(SPECS_NAMESPACE)?.strip_prefix('.')?;
    match key {
        "class" => Some("class"),
        "id" => Some("id"),
        "style" => Some("style"),
        _ => None,
    }
}

/// One slot position's assigned name, or the qualified bare name it falls back
/// to (D-7).
fn slot_name(config: &FieldConfig, slot: Slot, kind: SlotKind) -> String {
    match config.assignment(slot, kind) {
        Some(Some(path)) => path.dotted(),
        _ => fallback_path(slot, kind).dotted(),
    }
}

/// One specs position's assigned name, or its `specs.<k>` spelling (§6.3).
fn specs_name(config: &FieldConfig, key: SpecsKey) -> String {
    match config.specs().get(key.key()) {
        Some(Some(path)) => path.dotted(),
        _ => specs_path(key).dotted(),
    }
}

/// `<specs>.<k>` as a [`Path`]; the one place the namespace literal is built.
fn specs_path(key: SpecsKey) -> Path {
    Path::from_segments(&[SPECS_NAMESPACE, key.key()])
        .expect("`specs` and its keys are valid path segments")
}

/// Outcome of scanning extras off the front of some text.
#[derive(Debug, Clone, PartialEq)]
pub enum ExtrasMatch<'a> {
    /// Not a head at all: the text is literal (nothing consumed).
    Absent { rest: &'a str },
    /// A head was parsed; `rest` is what follows it.
    Head { head: ExtrasHead, rest: &'a str },
    /// Looks like a head but is malformed (nothing consumed, §4.3).
    Malformed { error: ExtrasError, rest: &'a str },
}

/// Which sigil opened a directive (`::` inline, `==` block).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectiveSigil {
    Colon,
    Equal,
}

impl DirectiveSigil {
    /// The character the sigil is made of.
    pub fn character(self) -> char {
        match self {
            Self::Colon => ':',
            Self::Equal => '=',
        }
    }
}

/// The head of a directive: `::type[slug]("title")` / `==type[slug]("title")`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectiveHead {
    pub sigil: DirectiveSigil,
    /// Sigil run length, 2..=7.
    pub count: usize,
    /// `None` for a bare fence, which only closes an open directive (§10.3).
    pub type_marker: Option<String>,
    /// Raw contents of `[…]` (name = `bracket`).
    pub bracket: Option<String>,
    /// Raw contents of `(…)` (name = `paren`), unquoted.
    pub parentheses: Option<String>,
}

/// Outcome of scanning a directive head off the front of some text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectiveMatch<'a> {
    /// Not a directive head (nothing consumed).
    Absent {
        rest: &'a str,
    },
    Head {
        head: DirectiveHead,
        rest: &'a str,
    },
    /// A sigil run of 2..=7 that cannot be read as a head (nothing consumed).
    Malformed {
        error: ExtrasError,
        rest: &'a str,
    },
}

/// How loud a §13 diagnostic about an extras head is.
///
/// The grammar layer stays free of `pendon_core`: severity is the IR's
/// vocabulary, and [`crate::emit`] is the one adapter that maps this onto
/// [`pendon_core::Severity`]. A head resolves either way, so no issue is ever
/// fatal — D-12's `Error`s keep the first value and stay usable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueSeverity {
    /// Something was dropped in favour of the source that wins (§6.2).
    Warning,
    /// Two sources claimed one public path — a duplicate inside one head, or a
    /// `k: v` naming a built-in field — which is a bug and not a merge (D-12).
    Error,
}

/// One §13 diagnostic a head resolved: its severity and its message.
///
/// A plugin that emits a head's diagnostics later (marker, cite, directive,
/// anchor, image, heading) carries these instead of bare strings, because D-12
/// makes three of the rules `Error` and a message alone cannot say how loud it
/// is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtrasIssue {
    pub severity: IssueSeverity,
    pub message: String,
}

impl ExtrasIssue {
    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            severity: IssueSeverity::Warning,
            message: message.into(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            severity: IssueSeverity::Error,
            message: message.into(),
        }
    }

    /// D-12: one public path, two sources inside one head — the first wins.
    pub fn duplicate(field: &str) -> Self {
        Self::error(format!(
            "`{field}` is set twice in one extras head; the first one wins"
        ))
    }

    /// §6.2: the construct head already set `field`, so the extras value lost.
    pub fn overridden(field: &str) -> Self {
        Self::warning(format!(
            "`{field}` was dropped because the construct head already sets it"
        ))
    }

    /// D-12's last rule: a `k: v` named like one of the element's **built-in**
    /// fields (`href` on an anchor, `src` on an image).
    ///
    /// The construct keeps its own value — a built-in field is written by the
    /// element, not by the head — and the way to give that value another public
    /// name is `field.override.<field>` (D-4), never a second source in the head.
    pub fn owned_key(field: &str) -> Self {
        Self::error(format!(
            "`{field}` is owned by the construct; rename it with `field.override.{field}` instead"
        ))
    }
}

/// A resolved attribute: either a typed value or a bare flag (§6.3).
///
/// A flag is deliberately *not* an [`AttrValue`]: it maps to
/// `Event::AttributeFlag { name }` and renders as a bare name, never as
/// `name="…"`.
#[derive(Debug, Clone, PartialEq)]
pub enum ExtrasAttr {
    Value(AttrValue),
    Flag,
}

/// Attributes produced from a head, plus the §13 diagnostics it resolved.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Attrs {
    pub items: Vec<(String, ExtrasAttr)>,
    pub issues: Vec<ExtrasIssue>,
}

impl Attrs {
    pub fn get(&self, key: &str) -> Option<&ExtrasAttr> {
        self.items
            .iter()
            .find_map(|(name, value)| (name == key).then_some(value))
    }

    /// The typed value of `key`, when it is a value and not a flag.
    pub fn value(&self, key: &str) -> Option<&AttrValue> {
        match self.get(key) {
            Some(ExtrasAttr::Value(value)) => Some(value),
            _ => None,
        }
    }

    /// `true` when `key` is present, whatever its kind.
    pub fn has(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    pub fn push(&mut self, key: impl Into<String>, value: ExtrasAttr) {
        self.items.push((key.into(), value));
    }

    /// Merges this **head** (which wins, §6.2) with the extras of the same
    /// construct. `owned` lists construct-owned keys (`href`, `src`, `start`)
    /// that extras must never override: an attempt is D-12's built-in-field
    /// collision, so it is an `Error` ([`ExtrasIssue::owned_key`]) and not the
    /// `Warning` a plain §6.2 override raises.
    pub fn merge_with(self, extras: Attrs, owned: &[&str]) -> Attrs {
        let mut merged = self;
        merged.issues.extend(extras.issues);
        for (key, value) in extras.items {
            if owned.contains(&key.as_str()) {
                merged.issues.push(ExtrasIssue::owned_key(&key));
            } else if merged.has(&key) {
                merged.issues.push(ExtrasIssue::overridden(&key));
            } else {
                merged.items.push((key, value));
            }
        }
        merged
    }

    /// Stringly-typed view for renderers without attribute-value support: a flag
    /// becomes its own name (§6.3).
    pub fn to_dom(&self) -> Vec<(String, String)> {
        self.items
            .iter()
            .map(|(key, value)| {
                let text = match value {
                    ExtrasAttr::Flag => key.clone(),
                    ExtrasAttr::Value(value) => value.literal(),
                };
                (key.clone(), text)
            })
            .collect()
    }
}

// ---------------------------------------------------------------- extras head

/// Scans an extras head off the front of `input` (spec §4–§5).
///
/// Both spellings are heads, and both are canonical (§3):
///
/// * `{…}` / `{}` — a bare extras head;
/// * `@@type{…}` / `@@{…}` / `@@type` — the `@@`-prefixed form, where the type
///   may stand alone as a **type-only head** (§4.1).
///
/// Text that is not a head — including a type that is separated from its `{`
/// by whitespace — is [`ExtrasMatch::Absent`] or [`ExtrasMatch::Malformed`], and
/// is rendered verbatim (§4.3).
pub fn parse_extras(input: &str) -> ExtrasMatch<'_> {
    // A bare `%%numberType` is a head of its own: it is the marker's canonical
    // stand-alone spelling, attached directly to a construct (`[link](/foo)%%a`).
    // A bare `%`, `%%` with no name and a run of 8+ `%` are literal text (D-9).
    if input.starts_with('%') {
        return match read_number_mark(input) {
            Some((mark, consumed)) => ExtrasMatch::Head {
                head: ExtrasHead {
                    items: vec![ExtrasItem::Number(mark)],
                    ..ExtrasHead::default()
                },
                rest: &input[consumed..],
            },
            None => ExtrasMatch::Absent { rest: input },
        };
    }
    if let Some(after) = input.strip_prefix("@@") {
        return match read_head(after) {
            Ok((head, consumed)) => ExtrasMatch::Head {
                head,
                rest: &input[2 + consumed..],
            },
            Err(error) => ExtrasMatch::Malformed { error, rest: input },
        };
    }
    if input.starts_with('{') {
        return match read_body(input) {
            Ok((head, consumed)) => ExtrasMatch::Head {
                head,
                rest: &input[consumed..],
            },
            Err(error) => ExtrasMatch::Malformed { error, rest: input },
        };
    }
    ExtrasMatch::Absent { rest: input }
}

/// Reads the optional positional groups of a head: an immediate `[…]` followed
/// by an immediate `(\u2026)` (§6.1), returning them and the bytes consumed.
///
/// Adjacency is strict — a space before `[` means no group — and a group that
/// opens without its closer is [`ExtrasError::UnterminatedHead`] (§4.3: the
/// caller keeps the whole head literal). The `[…]` contents stay raw, the
/// `(\u2026)` contents are trimmed and unquoted, exactly like a directive head.
pub fn read_positional_groups(
    after: &str,
) -> Result<(Option<String>, Option<String>, usize), ExtrasError> {
    let mut cursor = 0usize;
    let mut bracket = None;
    if let Some(remainder) = after[cursor..].strip_prefix('[') {
        let Some(close) = remainder.find(']') else {
            return Err(ExtrasError::UnterminatedHead);
        };
        bracket = Some(remainder[..close].to_string());
        cursor = after.len() - remainder[close + 1..].len();
    }
    let mut parentheses = None;
    if let Some(remainder) = after[cursor..].strip_prefix('(') {
        let Some(close) = remainder.find(')') else {
            return Err(ExtrasError::UnterminatedHead);
        };
        parentheses = Some(unquote_wrapping(remainder[..close].trim()));
        cursor = after.len() - remainder[close + 1..].len();
    }
    Ok((bracket, parentheses, cursor))
}

/// Reads the type name and optional positional groups / `{…}` body of a `@@`
/// head, returning the bytes consumed **after** the `@@`.
///
/// The grammar is `@@` type? `[…]`? `(\u2026)`? `{…}?` — each part must touch
/// the previous one (§4.1). A head without a `{` stands alone when it carries a
/// type or at least one positional group; a bare `@@` is still not a head.
fn read_head(after: &str) -> Result<(ExtrasHead, usize), ExtrasError> {
    let bytes = after.as_bytes();
    let mut cursor = 0;
    if matches!(bytes.first(), Some(byte) if byte.is_ascii_alphabetic()) {
        cursor = 1;
        while matches!(bytes.get(cursor), Some(byte) if byte.is_ascii_alphanumeric()) {
            cursor += 1;
        }
    }
    let type_marker = (cursor > 0).then(|| after[..cursor].to_string());

    let (bracket, parentheses, groups) = read_positional_groups(&after[cursor..])?;
    cursor += groups;

    if bytes.get(cursor) == Some(&b'{') {
        let (mut head, consumed) = read_body(&after[cursor..])?;
        let mut total = cursor + consumed;
        // §5: `%%numberType` sits at the very end of the extras and attaches to
        // the body it follows (`@@headingA{…}%%%numberB`), with no separator.
        if let Some((mark, used)) = read_number_mark(&after[total..]) {
            head.items.push(ExtrasItem::Number(mark));
            total += used;
        }
        head.type_marker = type_marker;
        head.bracket = bracket;
        head.parentheses = parentheses;
        return Ok((head, total));
    }

    // No `{` follows. A trailing `%%numberType` still attaches to the type-only
    // head (`@@AnchorA%%numberB`, and the bare `@@%%%numberB` form).
    let mut number = None;
    if let Some((mark, used)) = read_number_mark(&after[cursor..]) {
        number = Some(mark);
        cursor += used;
    }

    // A type, a positional group or a number marker can carry the head on its
    // own (§4.1/§6.1); a bare `@@` cannot.
    if type_marker.is_none() && bracket.is_none() && parentheses.is_none() && number.is_none() {
        return Err(ExtrasError::InvalidHead);
    }
    // §4.1: `@@type {…}` is not a head — the `{` must touch the type.
    if number.is_none()
        && matches!(bytes.get(cursor), Some(b' ' | b'\t'))
        && opens_brace_on_line(&after[cursor..])
    {
        return Err(ExtrasError::InvalidHead);
    }
    // A type-only (or groups-only) head: the run ends at the first symbol (or
    // the end of input) and only the type / groups are consumed.
    // `@@anchorA.` keeps the `.` as text.
    Ok((
        ExtrasHead {
            type_marker,
            bracket,
            parentheses,
            items: number.into_iter().map(ExtrasItem::Number).collect(),
        },
        cursor,
    ))
}

/// Reads a bare `{…}` head, returning the head and the bytes it consumed
/// (including both braces).
fn read_body(input: &str) -> Result<(ExtrasHead, usize), ExtrasError> {
    // A head may not span lines (§4.3): only the current line is searched.
    let body = &input[1..];
    let line = match body.split_once('\n') {
        Some((line, _)) => line,
        None => body,
    };
    // Depth-aware so a nested value (`{ k: { a: 1 } }`) does not close the head
    // at its inner `}` (`docs/rfc/plugin-bind.md`).
    let close = find_group_close(line, '}').ok_or(ExtrasError::UnterminatedHead)?;
    let head = parse_extras_body(&line[..close])?;
    Ok((head, 1 + close + 1))
}

/// Index within `body` (the text *after* an opening bracket) of the `close`
/// bracket that matches it, honouring nested `{ … }` / `[ … ]`, quotes and `\`
/// escapes. Returns `None` when it is never closed (or a mismatched closer
/// appears) on this line, so the caller treats the value as malformed (§4.3).
fn find_group_close(body: &str, close: char) -> Option<usize> {
    let mut depth = 0i32;
    let mut quote: Option<char> = None;
    let mut escaped = false;

    for (index, character) in body.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match character {
            '\\' => escaped = true,
            '\'' | '"' | '`' => match quote {
                Some(open) if open == character => quote = None,
                None => quote = Some(character),
                _ => {}
            },
            '{' | '[' if quote.is_none() => depth += 1,
            '}' | ']' if quote.is_none() => {
                if depth == 0 {
                    // A mismatched closer at depth 0 is not this group's end.
                    return (character == close).then_some(index);
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    None
}

/// `true` when only spaces or tabs separate the cursor from a `{` on the same
/// line, i.e. the `{` is *not* adjacent to the type (§4.1).
fn opens_brace_on_line(rest: &str) -> bool {
    let line = match rest.split_once('\n') {
        Some((line, _)) => line,
        None => rest,
    };
    line.trim_start_matches([' ', '\t']).starts_with('{')
}

/// Reads only the type marker of a `@@` head (`@@type`), without parsing the
/// body. The returned slice starts just past the type run: it is empty for a
/// type-only head and starts with `{` for a typed head.
pub fn parse_type_marker(input: &str) -> Option<(String, &str)> {
    let after = input.strip_prefix("@@")?;
    let bytes = after.as_bytes();
    if !matches!(bytes.first(), Some(byte) if byte.is_ascii_alphabetic()) {
        return None;
    }
    let mut cursor = 1;
    while matches!(bytes.get(cursor), Some(byte) if byte.is_ascii_alphanumeric()) {
        cursor += 1;
    }
    Some((after[..cursor].to_string(), &after[cursor..]))
}

/// Parses the body of a head, i.e. everything between `{` and `}`.
pub fn parse_extras_body(body: &str) -> Result<ExtrasHead, ExtrasError> {
    let mut items = Vec::new();
    for part in split_items(body)? {
        if part.trim().is_empty() {
            continue;
        }
        items.push(parse_item(part)?);
    }
    Ok(ExtrasHead {
        type_marker: None,
        bracket: None,
        parentheses: None,
        items,
    })
}

/// Splits a body on top-level commas, honouring quotes, `\` escapes, and
/// nested `{ … }` / `[ … ]` (so a structured value's inner commas do not split
/// the item — `docs/rfc/plugin-bind.md`).
fn split_items(body: &str) -> Result<Vec<&str>, ExtrasError> {
    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut quote: Option<char> = None;
    let mut escaped = false;
    // Nesting depth of `{` / `[` opened inside a value; a comma only splits an
    // item when it sits at depth 0.
    let mut depth = 0i32;

    for (index, character) in body.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match character {
            '\\' => escaped = true,
            '\'' | '"' | '`' => match quote {
                Some(open) if open == character => quote = None,
                None => quote = Some(character),
                _ => {}
            },
            '{' | '[' if quote.is_none() => depth += 1,
            '}' | ']' if quote.is_none() => depth = depth.saturating_sub(1),
            ',' if quote.is_none() && depth == 0 => {
                parts.push(&body[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }

    if quote.is_some() {
        return Err(ExtrasError::UnterminatedValue);
    }
    parts.push(&body[start..]);
    Ok(parts)
}

/// Parses a single item of a head body (§5).
fn parse_item(part: &str) -> Result<ExtrasItem, ExtrasError> {
    let part = part.trim();
    match part.as_bytes().first() {
        None => Err(ExtrasError::InvalidItem),
        Some(b'.') => Ok(ExtrasItem::Class(read_name(&part[1..])?)),
        Some(b'#') => Ok(ExtrasItem::Id(read_name(&part[1..])?)),
        Some(b'`') => Ok(ExtrasItem::Slug(read_scalar(part)?)),
        Some(b'"') | Some(b'\'') => Ok(ExtrasItem::Title(read_scalar(part)?)),
        Some(b'%') => {
            // §5: a number marker is one whole item — nothing may trail it.
            let (mark, consumed) = read_number_mark(part).ok_or(ExtrasError::InvalidItem)?;
            if consumed != part.len() {
                return Err(ExtrasError::InvalidItem);
            }
            Ok(ExtrasItem::Number(mark))
        }
        Some(_) => {
            if let Some((key, value)) = split_key_value(part) {
                let key = key.trim();
                if key.is_empty() {
                    return Err(ExtrasError::InvalidItem);
                }
                if let Some(name) = key.strip_prefix("--") {
                    let raw = value.trim();
                    if raw.is_empty() {
                        return Err(ExtrasError::EmptyValue);
                    }
                    return Ok(ExtrasItem::CssVar {
                        name: read_key(name)?,
                        // Unquoted like any other value, so the declaration that
                        // lands in `style` is valid CSS (§6.3).
                        value: parse_scalar(raw)?.literal(),
                    });
                }
                return Ok(ExtrasItem::Prop {
                    key: read_key(key)?,
                    value: parse_value(value)?,
                });
            }
            Ok(ExtrasItem::Flag {
                name: read_key(part)?,
            })
        }
    }
}

/// Splits `key: value` on the first `:` outside quotes.
fn split_key_value(text: &str) -> Option<(&str, &str)> {
    let mut quote: Option<char> = None;
    let mut escaped = false;

    for (index, character) in text.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match character {
            '\\' => escaped = true,
            '\'' | '"' | '`' => match quote {
                Some(open) if open == character => quote = None,
                None => quote = Some(character),
                _ => {}
            },
            ':' if quote.is_none() => return Some((&text[..index], &text[index + 1..])),
            _ => {}
        }
    }
    None
}

/// Parses a scalar: quoted text, a backticked literal, or an unquoted literal.
fn parse_scalar(text: &str) -> Result<AttrValue, ExtrasError> {
    let text = text.trim();
    match text.as_bytes().first() {
        None => Err(ExtrasError::EmptyValue),
        Some(&quote @ (b'"' | b'\'' | b'`')) => Ok(AttrValue::Str(read_quoted(text, quote)?)),
        Some(_) => {
            if text.chars().any(char::is_whitespace) {
                return Err(ExtrasError::UnquotedWhitespace);
            }
            Ok(classify_scalar(text))
        }
    }
}

/// Parses a prop value: a nested `{ … }` object, a `[ … ]` array, or a scalar
/// (`docs/rfc/plugin-bind.md`). Dispatch is on the first non-space byte.
///
/// A `{ … }` group whose balanced body is a single JSX expression token — a
/// dot-separated identifier path such as `valueA` or `frontmatter.fm1` — is
/// parsed as [`AttrValue::JsxExpr`] so it is emitted unquoted as
/// `name={expr}` instead of being rejected as a malformed object. Any other
/// `{ … }` falls through to [`parse_object`], which preserves all existing
/// error behaviour (unbalanced groups, trailing characters, …).
fn parse_value(text: &str) -> Result<AttrValue, ExtrasError> {
    let text = text.trim();
    match text.as_bytes().first() {
        Some(b'{') => {
            if let Some(expr) = read_jsx_expression(text) {
                return Ok(AttrValue::JsxExpr(expr));
            }
            parse_object(text)
        }
        Some(b'[') => parse_array(text),
        _ => parse_scalar(text),
    }
}

/// If `text` is a `{ … }` group whose balanced body is a single JSX expression
/// token — a dot-separated identifier path such as `valueA` or
/// `frontmatter.fm1` — return the expression text (braces stripped).
///
/// Returns `None` for anything else: an actual object (`{ key: value }`),
/// an empty group (`{}`), a group with trailing characters, or any group whose
/// body is not a pure identifier path. The caller then falls through to
/// [`parse_object`], which preserves all existing error behaviour.
fn read_jsx_expression(text: &str) -> Option<String> {
    let inner = text.trim_start().strip_prefix('{')?;
    let close = find_group_close(inner, '}')?;
    let body = &inner[..close];
    if !inner[close + 1..].trim().is_empty() {
        return None;
    }
    let body = body.trim();
    if body.is_empty() {
        return None;
    }
    let mut ok = true;
    for segment in body.split('.') {
        let segment = segment.trim();
        if segment.is_empty() {
            ok = false;
            break;
        }
        let mut chars = segment.chars();
        match chars.next() {
            Some(c) if c.is_ascii_alphabetic() || c == '_' => {
                // first character is valid
            }
            _ => {
                ok = false;
                break;
            }
        }
        if !chars.all(|c| c.is_ascii_alphanumeric() || c == '_') {
            ok = false;
            break;
        }
    }
    ok.then_some(body.to_string())
}

/// Parses a nested `{ key: value, … }` object value. Keys reuse the prop-key
/// rules; values recurse through [`parse_value`], so objects nest arbitrarily.
/// Trailing characters after the closing `}` are rejected (§4.3 fallback).
///
/// A `...$var` **spread** is legal here and only here (§2.6): it is collected
/// into the [`SPREAD_KEY`] entry as the reference strings, in source order.
fn parse_object(text: &str) -> Result<AttrValue, ExtrasError> {
    let inner = &text[1..];
    let close = find_group_close(inner, '}').ok_or(ExtrasError::UnterminatedHead)?;
    let body = &inner[..close];
    if !inner[close + 1..].trim().is_empty() {
        return Err(ExtrasError::TrailingCharacters);
    }
    let mut entries: Vec<(String, AttrValue)> = Vec::new();
    let mut spreads: Vec<AttrValue> = Vec::new();
    for part in split_items(body)? {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        // A spread is recognised only as a whole item, so a value that happens
        // to start with `.` (`a: ...$x`) is still parsed as a value.
        if let Some(token) = part.strip_prefix("...") {
            let reference = token.trim();
            if !reference.starts_with('$')
                || reference.len() == 1
                || reference.chars().any(char::is_whitespace)
            {
                return Err(ExtrasError::InvalidItem);
            }
            // The reference's own grammar belongs to `plugin-bind`; this layer
            // only transports the text.
            spreads.push(AttrValue::Str(reference.to_string()));
            continue;
        }
        let (key, value) = split_key_value(part).ok_or(ExtrasError::InvalidItem)?;
        let key = key.trim();
        if key.is_empty() {
            return Err(ExtrasError::InvalidItem);
        }
        // Last-wins on a duplicate key, matching `to_attributes` (§6.4).
        let value = parse_value(value)?;
        match entries.iter_mut().find(|(existing, _)| existing == key) {
            Some(slot) => slot.1 = value,
            None => entries.push((read_key(key)?.to_string(), value)),
        }
    }
    if !spreads.is_empty() {
        entries.push((SPREAD_KEY.to_string(), AttrValue::Array(spreads)));
    }
    Ok(AttrValue::Object(entries))
}

/// Parses a nested `[ value, … ]` array value. Elements recurse through
/// [`parse_value`]. Trailing characters after the closing `]` are rejected.
fn parse_array(text: &str) -> Result<AttrValue, ExtrasError> {
    let inner = &text[1..];
    let close = find_group_close(inner, ']').ok_or(ExtrasError::UnterminatedHead)?;
    let body = &inner[..close];
    if !inner[close + 1..].trim().is_empty() {
        return Err(ExtrasError::TrailingCharacters);
    }
    let mut items = Vec::new();
    for part in split_items(body)? {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        items.push(parse_value(part)?);
    }
    Ok(AttrValue::Array(items))
}

/// Parses a scalar that must start with a quote or backtick.
fn read_scalar(text: &str) -> Result<String, ExtrasError> {
    match parse_scalar(text)? {
        AttrValue::Str(value) => Ok(value),
        other => Ok(other.literal()),
    }
}

/// Reads quoted text, honouring `\` escapes, and rejects trailing characters.
fn read_quoted(text: &str, quote: u8) -> Result<String, ExtrasError> {
    let mut out = String::new();
    let mut consumed = 1;
    let mut escaped = false;
    let mut closed = false;

    for character in text[1..].chars() {
        consumed += character.len_utf8();
        if escaped {
            out.push(character);
            escaped = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
            continue;
        }
        if character as u8 == quote {
            closed = true;
            break;
        }
        out.push(character);
    }

    if !closed {
        return Err(ExtrasError::UnterminatedValue);
    }
    if !text[consumed..].trim().is_empty() {
        return Err(ExtrasError::TrailingCharacters);
    }
    Ok(out)
}

/// Validates and normalises a class/id name: `1*( ALPHA | DIGIT | "_" | "-" | ":" )`.
fn read_name(text: &str) -> Result<String, ExtrasError> {
    let name = text.trim();
    if is_name(name) {
        Ok(name.to_string())
    } else {
        Err(ExtrasError::InvalidItem)
    }
}

/// Validates and normalises a key (prop, CSS var, bare flag):
/// `( ALPHA | "_" ) { ALPHA | DIGIT | "_" | "-" }`.
fn read_key(text: &str) -> Result<String, ExtrasError> {
    let key = text.trim();
    if is_key(key) {
        Ok(key.to_string())
    } else {
        Err(ExtrasError::InvalidItem)
    }
}

/// ASCII only, so no lookalike character can be mistaken for syntax (§4.4).
fn is_name(text: &str) -> bool {
    !text.is_empty()
        && text.chars().all(|character| {
            character.is_ascii_alphanumeric()
                || character == '_'
                || character == '-'
                || character == ':'
        })
}

/// A key cannot start with a digit or `-`; that would collide with number
/// values and with `--` CSS vars.
fn is_key(text: &str) -> bool {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => {}
        _ => return false,
    }
    chars.all(|character| character.is_ascii_alphanumeric() || character == '_' || character == '-')
}

// ----------------------------------------------------- extras → attributes

/// Maps a head onto attributes in the deterministic §6.4 order: `class`, `id`,
/// the positional specs (`bracket`, `paren`, `tick`, then `quote`), then every
/// remaining prop and flag in source order with `specs.style` where the first
/// `style:` prop or `--var` item appeared.
///
/// A positional **group** (`[...]` / `(...)` of §6.1) wins over a same-key item of
/// the `{...}` body: the item is dropped with an `Overridden` warning (§6.2
/// head-wins style). Two sources for one public path *inside* one head are a bug
/// and not a merge, so D-12 makes them an `Error` and keeps the **first** value:
/// a repeated `specs.tick` / `specs.quote` / `specs.id` and a repeated prop or
/// flag are `Error`s, `specs.class` accumulates and dedupes keeping the first
/// position, and `specs.style` is one declaration per property, last value wins.
///
/// D-12's *last* rule — a `k: v` named like one of the element's built-in
/// fields — is not decidable here, because a head does not know its element.
/// [`Attrs::merge_with`] answers it for the keys a construct passes as `owned`,
/// and an element that owns a key outright reports
/// [`ExtrasIssue::owned_key`] itself; both are `Error`s.
pub fn to_attributes(head: &ExtrasHead, specs: &Specs) -> Attrs {
    let mut classes: Vec<String> = Vec::new();
    let mut id: Option<String> = None;
    let mut slug: Option<String> = None;
    let mut title: Option<String> = None;
    let mut rest: Vec<(String, ExtrasAttr)> = Vec::new();
    let mut style: Vec<(String, String)> = Vec::new();
    let mut style_pos: Option<usize> = None;
    let mut issues = Vec::new();

    // §6.1: the positional groups of the head itself, mapped through their
    // specs. They occupy the §6.4 slot before the body's positional items and
    // win every key collision with them.
    let mut group_keys: Vec<&str> = Vec::new();
    let mut groups: Vec<(String, ExtrasAttr)> = Vec::new();
    if let Some(bracket) = &head.bracket {
        groups.push((
            specs.bracket.clone(),
            ExtrasAttr::Value(AttrValue::Str(bracket.clone())),
        ));
        group_keys.push(&specs.bracket);
    }
    if let Some(parentheses) = &head.parentheses {
        groups.push((
            specs.paren.clone(),
            ExtrasAttr::Value(AttrValue::Str(parentheses.clone())),
        ));
        group_keys.push(&specs.paren);
    }

    for item in &head.items {
        match item {
            // D-12: `class` accumulates and dedupes, keeping the first position.
            ExtrasItem::Class(name) => push_class(&mut classes, name),
            ExtrasItem::Id(value) => match id {
                Some(_) => issues.push(ExtrasIssue::duplicate(&specs.id)),
                None => id = Some(value.clone()),
            },
            ExtrasItem::Slug(value) => {
                if group_keys.contains(&specs.tick.as_str()) {
                    issues.push(ExtrasIssue::overridden(&specs.tick));
                } else if slug.is_some() {
                    issues.push(ExtrasIssue::duplicate(&specs.tick));
                } else {
                    slug = Some(value.clone());
                }
            }
            ExtrasItem::Title(value) => {
                if group_keys.contains(&specs.quote.as_str()) {
                    issues.push(ExtrasIssue::overridden(&specs.quote));
                } else if title.is_some() {
                    issues.push(ExtrasIssue::duplicate(&specs.quote));
                } else {
                    title = Some(value.clone());
                }
            }
            ExtrasItem::CssVar { name, value } => {
                style_pos.get_or_insert(rest.len());
                set_declaration(
                    &mut style,
                    &format!("--{name}"),
                    format!("--{name}: {value}"),
                );
            }
            ExtrasItem::Prop { key, value } if key == "class" => {
                // `class` accumulates, whether it arrives as `.x` or `class: "x"`.
                push_class(&mut classes, &value.literal());
            }
            ExtrasItem::Prop { key, value } if key == "style" => {
                style_pos.get_or_insert(rest.len());
                push_style(&mut style, value.literal());
            }
            ExtrasItem::Prop { key, value: _ } if group_keys.contains(&key.as_str()) => {
                issues.push(ExtrasIssue::overridden(key));
            }
            ExtrasItem::Prop { key, value } => push_unique(
                &mut rest,
                key,
                ExtrasAttr::Value(value.clone()),
                &mut issues,
            ),
            // §5/D-9: the marker is re-emitted as the internal `__number`
            // attribute (ADR-0001's transport carries the object intact), so
            // every construct plugin forwards it without knowing about it.
            ExtrasItem::Number(mark) => set(
                &mut rest,
                "__number",
                ExtrasAttr::Value(AttrValue::Object(vec![
                    ("depth".to_string(), AttrValue::Int(i64::from(mark.depth))),
                    ("type".to_string(), AttrValue::Str(mark.number_type.clone())),
                ])),
            ),
            ExtrasItem::Flag { name } if group_keys.contains(&name.as_str()) => {
                issues.push(ExtrasIssue::overridden(name));
            }
            ExtrasItem::Flag { name } => {
                push_unique(&mut rest, name, ExtrasAttr::Flag, &mut issues)
            }
        }
    }

    let mut items = Vec::new();
    if !classes.is_empty() {
        items.push((
            specs.class.clone(),
            ExtrasAttr::Value(AttrValue::Str(classes.join(" "))),
        ));
    }
    if let Some(id) = id {
        items.push((specs.id.clone(), ExtrasAttr::Value(AttrValue::Str(id))));
    }
    items.extend(groups);
    if let Some(slug) = slug {
        items.push((specs.tick.clone(), ExtrasAttr::Value(AttrValue::Str(slug))));
    }
    if let Some(title) = title {
        items.push((
            specs.quote.clone(),
            ExtrasAttr::Value(AttrValue::Str(title)),
        ));
    }

    let style_pos = style_pos.unwrap_or(rest.len()).min(rest.len());
    for index in 0..=rest.len() {
        if index == style_pos && !style.is_empty() {
            let declarations: Vec<&str> = style
                .iter()
                .map(|(_, declaration)| declaration.as_str())
                .collect();
            items.push((
                specs.style.clone(),
                ExtrasAttr::Value(AttrValue::Str(declarations.join("; "))),
            ));
        }
        if let Some((key, value)) = rest.get(index) {
            items.push((key.clone(), value.clone()));
        }
    }

    // §6.4/D-12: two sources for one public path is a bug, not a merge, so the
    // first occurrence keeps both its place and its value and the later one is
    // reported. Inside one head this only fires when assigned names (`field.*`)
    // collide — every other duplicate was caught above.
    let mut collapsed: Vec<(String, ExtrasAttr)> = Vec::new();
    for (key, value) in items {
        if collapsed.iter().any(|(name, _)| *name == key) {
            issues.push(ExtrasIssue::duplicate(&key));
        } else {
            collapsed.push((key, value));
        }
    }

    Attrs {
        items: collapsed,
        issues,
    }
}

/// Appends a class name, keeping the first position and dropping repeats (D-12).
///
/// A `class:` prop may carry several names (`class: "a b"`), so each is handled
/// like a `.x` item.
fn push_class(classes: &mut Vec<String>, names: &str) {
    for name in names.split_whitespace() {
        if !classes.iter().any(|existing| existing == name) {
            classes.push(name.to_string());
        }
    }
}

/// Inserts `key`, or reports it when an earlier item already claimed it: D-12
/// keeps the **first** value of a repeated prop or flag.
fn push_unique(
    items: &mut Vec<(String, ExtrasAttr)>,
    key: &str,
    value: ExtrasAttr,
    issues: &mut Vec<ExtrasIssue>,
) {
    if items.iter().any(|(name, _)| name == key) {
        issues.push(ExtrasIssue::duplicate(key));
    } else {
        items.push((key.to_string(), value));
    }
}

/// Folds a `style:` value into the declaration list, one declaration per
/// property: a repeated property keeps the first position and the last value
/// (CSS semantics, D-12).
fn push_style(style: &mut Vec<(String, String)>, declaration: String) {
    let property = declaration.split_once(':').map_or_else(
        || declaration.trim().to_string(),
        |(name, _)| name.trim().to_string(),
    );
    set_declaration(style, &property, declaration);
}

/// Replaces the declaration of `property` in place, or appends a new one.
fn set_declaration(style: &mut Vec<(String, String)>, property: &str, declaration: String) {
    match style.iter_mut().find(|(name, _)| name == property) {
        Some(slot) => slot.1 = declaration,
        None => style.push((property.to_string(), declaration)),
    }
}

/// Inserts `key`, or replaces the value of an existing occurrence in place so
/// that the position of the first occurrence is kept.
fn set(items: &mut Vec<(String, ExtrasAttr)>, key: &str, value: ExtrasAttr) {
    if let Some(slot) = items.iter_mut().find(|(name, _)| name == key) {
        slot.1 = value;
    } else {
        items.push((key.to_string(), value));
    }
}

// ------------------------------------------------------------ directive head

/// Scans a directive head (`::type[slug]("title")`, §10) off the front of text.
///
/// A single sigil is not a head, a run longer than seven is literal text, and a
/// bare `::` / `==` head is the close fence (§10.3).
pub fn parse_directive_head(input: &str) -> DirectiveMatch<'_> {
    let sigil = match input.as_bytes().first() {
        Some(b':') => DirectiveSigil::Colon,
        Some(b'=') => DirectiveSigil::Equal,
        _ => return DirectiveMatch::Absent { rest: input },
    };
    let symbol = sigil.character() as u8;
    let mut count = 0;
    while input.as_bytes().get(count) == Some(&symbol) {
        count += 1;
    }
    if !(2..=7).contains(&count) {
        return DirectiveMatch::Absent { rest: input };
    }

    let mut rest = &input[count..];
    let bytes = rest.as_bytes();
    let mut cursor = 0;
    if matches!(bytes.first(), Some(byte) if byte.is_ascii_alphabetic()) {
        cursor = 1;
        while matches!(bytes.get(cursor), Some(byte) if byte.is_ascii_alphanumeric()) {
            cursor += 1;
        }
    }
    let type_marker = (cursor > 0).then(|| rest[..cursor].to_string());
    rest = &rest[cursor..];

    let (bracket, parentheses, groups) = match read_positional_groups(rest) {
        Ok(groups) => groups,
        Err(error) => return DirectiveMatch::Malformed { error, rest: input },
    };
    rest = &rest[groups..];

    if type_marker.is_none() && bracket.is_none() && parentheses.is_none() && !rest.is_empty() {
        // `==5` / `::= x`: an opening fence needs a type name (§10.1).
        return DirectiveMatch::Malformed {
            error: ExtrasError::InvalidHead,
            rest: input,
        };
    }

    DirectiveMatch::Head {
        head: DirectiveHead {
            sigil,
            count,
            type_marker,
            bracket,
            parentheses,
        },
        rest,
    }
}

/// Removes one wrapping pair of quotes or backticks, if present.
fn unquote_wrapping(text: &str) -> String {
    let bytes = text.as_bytes();
    if bytes.len() >= 2 {
        let first = bytes[0];
        if (first == b'"' || first == b'\'' || first == b'`') && bytes[bytes.len() - 1] == first {
            return text[1..text.len() - 1].to_string();
        }
    }
    text.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn head(source: &str) -> (ExtrasHead, &str) {
        match parse_extras(source) {
            ExtrasMatch::Head { head, rest } => (head, rest),
            other => panic!("expected a head for {source:?}, got {other:?}"),
        }
    }

    fn mark(head: &ExtrasHead) -> &NumberMark {
        head.items
            .iter()
            .find_map(|item| match item {
                ExtrasItem::Number(mark) => Some(mark),
                _ => None,
            })
            .expect("a number mark")
    }

    /// §5: the marker is the whole item, and its depth is stored already
    /// normalized (`%` count − 1, so `%%` is 1 and `%%%%%%%` is 6).
    #[test]
    fn a_bare_mark_is_a_head_with_a_normalized_depth() {
        let (parsed, rest) = head("%%numberB tail");
        assert_eq!(rest, " tail");
        assert_eq!(
            mark(&parsed),
            &NumberMark {
                depth: 1,
                number_type: "numberB".to_string()
            }
        );

        let (parsed, rest) = head("%%%%%%%numberB");
        assert_eq!(rest, "");
        assert_eq!(mark(&parsed).depth, 6);
    }

    /// The two legal positions: attached to a body and to a type-only head.
    #[test]
    fn a_mark_attaches_to_a_body_and_to_a_type_only_head() {
        let (parsed, rest) = head("@@headingA{foo: \"bar\"}%%%numberB Body");
        assert_eq!(rest, " Body");
        assert_eq!(parsed.type_marker.as_deref(), Some("headingA"));
        assert_eq!(mark(&parsed).depth, 2);
        assert!(parsed
            .items
            .iter()
            .any(|item| matches!(item, ExtrasItem::Prop { key, .. } if key == "foo")));

        let (parsed, rest) = head("@@AnchorA%%numberB rest");
        assert_eq!(rest, " rest");
        assert_eq!(parsed.type_marker.as_deref(), Some("AnchorA"));
        assert_eq!(mark(&parsed).number_type, "numberB");

        // A bare `@@` before the marker is valid and only contributes the mark.
        let (parsed, rest) = head("@@%%%numberB rest");
        assert_eq!(rest, " rest");
        assert_eq!(parsed.type_marker, None);
        assert_eq!(mark(&parsed).depth, 2);

        // Inside a body, as the canonical last item.
        let (parsed, rest) = head("{.a, %%numberB} tail");
        assert_eq!(rest, " tail");
        assert_eq!(mark(&parsed).number_type, "numberB");
    }

    /// D-9: a bare `%`, `%%` with no name and a run of 8+ `%` are literal text,
    /// with no diagnostic.
    #[test]
    fn malformed_marks_stay_literal() {
        for source in ["%numberB", "%% no", "%% ", "%%%%%%%%numberA"] {
            assert!(
                matches!(parse_extras(source), ExtrasMatch::Absent { .. }),
                "{source:?} was not literal"
            );
        }
        assert!(read_number_mark("%%").is_none());
        assert!(read_number_mark("%").is_none());
        assert!(read_number_mark("%%%%%%%%numberA").is_none());
        // A marker that runs into a symbol ends there, exactly like a type-only
        // head (`@@anchorA.`): the symbol stays text.
        let (parsed, rest) = head("%%numberB- tail");
        assert_eq!(rest, "- tail");
        assert_eq!(mark(&parsed).number_type, "numberB");
        // …and a body keeps the same policy: the whole head stays literal.
        for source in ["{%%numberB extra}", "{%}", "{%%numberB-}"] {
            assert!(
                matches!(parse_extras(source), ExtrasMatch::Malformed { .. }),
                "{source:?} was not malformed"
            );
        }
    }

    /// The marker is re-emitted as the internal `__number` object attribute.
    #[test]
    fn the_marker_becomes_the_internal_number_attribute() {
        let (parsed, _) = head("@@headingA{foo: \"bar\"}%%%numberB");
        let attrs = to_attributes(&parsed, &Specs::default());
        let value = attrs.value("__number").expect("a __number value");
        assert_eq!(
            value.literal(),
            "{\"depth\":2,\"type\":\"numberB\"}".to_string()
        );
        // Its canonical place is last, after the body's own items.
        assert_eq!(
            attrs.items.last().map(|(key, _)| key.as_str()),
            Some("__number")
        );
    }
}
