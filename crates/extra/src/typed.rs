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

use crate::value::{classify_scalar, AttrValue};

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
    /// `` `value` `` — mapped to `backtick_key` (`slug` by default).
    Slug(String),
    /// `"value"` / `'value'` — mapped to `quote_key` (`title` by default).
    Title(String),
    /// `key: value`
    Prop { key: String, value: AttrValue },
    /// `--name: value` — folded into `style` (§6.3).
    CssVar { name: String, value: String },
    /// `isFoo` — rendered verbatim.
    Flag { name: String },
}

/// A parsed `@@type{…}` / `@@{…}` head.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExtrasHead {
    /// `type` of `@@type{…}`; `None` for `@@{…}`.
    pub type_marker: Option<String>,
    pub items: Vec<ExtrasItem>,
}

/// Positional names, per component (`backtick_key` / `quote_key`, §6.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtrasOptions {
    pub backtick_key: String,
    pub quote_key: String,
}

impl Default for ExtrasOptions {
    fn default() -> Self {
        Self {
            backtick_key: "slug".to_string(),
            quote_key: "title".to_string(),
        }
    }
}

impl ExtrasOptions {
    pub fn new(backtick_key: impl Into<String>, quote_key: impl Into<String>) -> Self {
        Self {
            backtick_key: backtick_key.into(),
            quote_key: quote_key.into(),
        }
    }
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
    /// Raw contents of `[…]` (key = `bracket_key`).
    pub bracket: Option<String>,
    /// Raw contents of `(…)` (key = `parentheses_key`), unquoted.
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

/// Non-fatal diagnostics attached to a parsed head (§13).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtrasWarning {
    /// Two `#id` items in one head; the last wins.
    DuplicateId,
    /// A duplicate key was resolved: the competing value was dropped when a
    /// construct head collided with extras (§6.2), or this later value won while
    /// the position of the first occurrence was kept (§6.4).
    Overridden { key: String },
    /// A construct-owned key (`href`, `src`, `start`) was passed as an extra.
    OwnedKeyIgnored { key: String },
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

/// Attributes produced from a head, plus its warnings.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Attrs {
    pub items: Vec<(String, ExtrasAttr)>,
    pub warnings: Vec<ExtrasWarning>,
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
    /// that extras must never override.
    pub fn merge_with(self, extras: Attrs, owned: &[&str]) -> Attrs {
        let mut merged = self;
        merged.warnings.extend(extras.warnings);
        for (key, value) in extras.items {
            if owned.contains(&key.as_str()) {
                merged.warnings.push(ExtrasWarning::OwnedKeyIgnored { key });
            } else if merged.has(&key) {
                merged.warnings.push(ExtrasWarning::Overridden { key });
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

/// Reads the type name and optional `{…}` body of a `@@` head, returning the
/// bytes consumed **after** the `@@`.
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

    if bytes.get(cursor) == Some(&b'{') {
        let (mut head, consumed) = read_body(&after[cursor..])?;
        head.type_marker = type_marker;
        return Ok((head, cursor + consumed));
    }

    // No `{` follows. Only a type can carry the head on its own (§4.1).
    let Some(type_marker) = type_marker else {
        return Err(ExtrasError::InvalidHead);
    };
    // §4.1: `@@type {…}` is not a head — the `{` must touch the type.
    if matches!(bytes.get(cursor), Some(b' ' | b'\t')) && opens_brace_on_line(&after[cursor..]) {
        return Err(ExtrasError::InvalidHead);
    }
    // A type-only head: the type run ends at the first symbol (or the end of
    // input) and only the type is consumed. `@@anchorA.` keeps the `.` as text.
    Ok((
        ExtrasHead {
            type_marker: Some(type_marker),
            items: Vec::new(),
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
    let close = find_closing_unquoted(line, b'}').ok_or(ExtrasError::UnterminatedHead)?;
    let head = parse_extras_body(&line[..close])?;
    Ok((head, 1 + close + 1))
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
        items,
    })
}

/// Splits a body on top-level commas, honouring quotes and `\` escapes.
fn split_items(body: &str) -> Result<Vec<&str>, ExtrasError> {
    let mut parts = Vec::new();
    let mut start = 0usize;
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
            ',' if quote.is_none() => {
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
                    value: parse_scalar(value)?,
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

/// Finds the first unquoted `closing` byte.
fn find_closing_unquoted(text: &str, closing: u8) -> Option<usize> {
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
            _ if quote.is_none() && character as u8 == closing => return Some(index),
            _ => {}
        }
    }
    None
}

// ----------------------------------------------------- extras → attributes

/// Maps a head onto attributes in the deterministic §6.4 order: `class`, `id`,
/// the positional keys (`backtick_key` then `quote_key`), then every remaining
/// prop and flag in source order with `style` where the first `style:` prop or
/// `--var` item appeared.
pub fn to_attributes(head: &ExtrasHead, options: &ExtrasOptions) -> Attrs {
    let mut classes: Vec<String> = Vec::new();
    let mut id: Option<String> = None;
    let mut slug: Option<String> = None;
    let mut title: Option<String> = None;
    let mut rest: Vec<(String, ExtrasAttr)> = Vec::new();
    let mut style: Vec<String> = Vec::new();
    let mut style_pos: Option<usize> = None;
    let mut warnings = Vec::new();

    for item in &head.items {
        match item {
            // `class` accumulates; every other kind is last-wins (§5).
            ExtrasItem::Class(name) => classes.push(name.clone()),
            ExtrasItem::Id(value) => {
                if id.replace(value.clone()).is_some() {
                    warnings.push(ExtrasWarning::DuplicateId);
                }
            }
            ExtrasItem::Slug(value) => slug = Some(value.clone()),
            ExtrasItem::Title(value) => title = Some(value.clone()),
            ExtrasItem::CssVar { name, value } => {
                style_pos.get_or_insert(rest.len());
                style.push(format!("--{name}: {value}"));
            }
            ExtrasItem::Prop { key, value } if key == "class" => {
                // `class` accumulates, whether it arrives as `.x` or `class: "x"`.
                classes.push(value.literal());
            }
            ExtrasItem::Prop { key, value } if key == "style" => {
                style_pos.get_or_insert(rest.len());
                style.push(value.literal());
            }
            ExtrasItem::Prop { key, value } => {
                set(&mut rest, key, ExtrasAttr::Value(value.clone()))
            }
            ExtrasItem::Flag { name } => set(&mut rest, name, ExtrasAttr::Flag),
        }
    }

    let mut items = Vec::new();
    if !classes.is_empty() {
        items.push((
            "class".to_string(),
            ExtrasAttr::Value(AttrValue::Str(classes.join(" "))),
        ));
    }
    if let Some(id) = id {
        items.push(("id".to_string(), ExtrasAttr::Value(AttrValue::Str(id))));
    }
    if let Some(slug) = slug {
        items.push((
            options.backtick_key.clone(),
            ExtrasAttr::Value(AttrValue::Str(slug)),
        ));
    }
    if let Some(title) = title {
        items.push((
            options.quote_key.clone(),
            ExtrasAttr::Value(AttrValue::Str(title)),
        ));
    }

    let style_pos = style_pos.unwrap_or(rest.len()).min(rest.len());
    for index in 0..=rest.len() {
        if index == style_pos && !style.is_empty() {
            items.push((
                "style".to_string(),
                ExtrasAttr::Value(AttrValue::Str(style.join("; "))),
            ));
        }
        if let Some((key, value)) = rest.get(index) {
            items.push((key.clone(), value.clone()));
        }
    }

    // §6.4: duplicates collapse to one attribute — the position of the first
    // occurrence wins, the value of the last occurrence wins.
    let mut collapsed: Vec<(String, ExtrasAttr)> = Vec::new();
    for (key, value) in items {
        match collapsed.iter_mut().find(|(name, _)| *name == key) {
            Some(slot) => {
                warnings.push(ExtrasWarning::Overridden { key: key.clone() });
                slot.1 = value;
            }
            None => collapsed.push((key, value)),
        }
    }

    Attrs {
        items: collapsed,
        warnings,
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

    let mut bracket = None;
    if let Some(remainder) = rest.strip_prefix('[') {
        let Some(close) = remainder.find(']') else {
            return DirectiveMatch::Malformed {
                error: ExtrasError::UnterminatedHead,
                rest: input,
            };
        };
        bracket = Some(remainder[..close].to_string());
        rest = &remainder[close + 1..];
    }

    let mut parentheses = None;
    if let Some(remainder) = rest.strip_prefix('(') {
        let Some(close) = remainder.find(')') else {
            return DirectiveMatch::Malformed {
                error: ExtrasError::UnterminatedHead,
                rest: input,
            };
        };
        parentheses = Some(unquote_wrapping(remainder[..close].trim()));
        rest = &remainder[close + 1..];
    }

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
