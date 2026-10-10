use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    Document,
    Frontmatter,
    Paragraph,
    Blockquote,
    CodeFence,
    Heading,
    ThematicBreak,
    BulletList,
    OrderedList,
    ListItem,
    Table,
    TableHead,
    TableBody,
    TableRow,
    TableCell,
    Section,
    HtmlBlock,
    // Inline nodes
    Emphasis,
    Strong,
    InlineCode,
    Link,
    Bold,
    Italic,
    HtmlInline,
    Image,
    /// A structured HTML element (`figure`, `figcaption`, `table`, `td`, …)
    /// emitted as real events instead of a raw HTML string. The tag name is
    /// carried on the node (as the `name` attribute) so renderers can emit
    /// `<tag …>children</tag>` and custom components can nest inside it.
    Element(String),
    // Custom node kinds (e.g., Component, user-defined)
    Custom(String),
}

impl NodeKind {
    pub fn name(&self) -> Cow<'_, str> {
        match self {
            NodeKind::Document => Cow::Borrowed("Document"),
            NodeKind::Frontmatter => Cow::Borrowed("Frontmatter"),
            NodeKind::Paragraph => Cow::Borrowed("Paragraph"),
            NodeKind::Blockquote => Cow::Borrowed("Blockquote"),
            NodeKind::CodeFence => Cow::Borrowed("CodeFence"),
            NodeKind::Heading => Cow::Borrowed("Heading"),
            NodeKind::ThematicBreak => Cow::Borrowed("ThematicBreak"),
            NodeKind::BulletList => Cow::Borrowed("BulletList"),
            NodeKind::OrderedList => Cow::Borrowed("OrderedList"),
            NodeKind::ListItem => Cow::Borrowed("ListItem"),
            NodeKind::Table => Cow::Borrowed("Table"),
            NodeKind::TableHead => Cow::Borrowed("TableHead"),
            NodeKind::TableBody => Cow::Borrowed("TableBody"),
            NodeKind::TableRow => Cow::Borrowed("TableRow"),
            NodeKind::TableCell => Cow::Borrowed("TableCell"),
            NodeKind::Section => Cow::Borrowed("Section"),
            NodeKind::HtmlBlock => Cow::Borrowed("HtmlBlock"),
            NodeKind::Emphasis => Cow::Borrowed("Emphasis"),
            NodeKind::Strong => Cow::Borrowed("Strong"),
            NodeKind::InlineCode => Cow::Borrowed("InlineCode"),
            NodeKind::Link => Cow::Borrowed("Link"),
            NodeKind::Bold => Cow::Borrowed("Bold"),
            NodeKind::Italic => Cow::Borrowed("Italic"),
            NodeKind::HtmlInline => Cow::Borrowed("HtmlInline"),
            NodeKind::Image => Cow::Borrowed("Image"),
            NodeKind::Element(_) => Cow::Borrowed("Element"),
            NodeKind::Custom(name) => Cow::Owned(name.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

/// Helper for emitting structured HTML elements as events.
pub fn element_open(tag: &str) -> [Event; 2] {
    [
        Event::StartNode(NodeKind::Element(tag.to_string())),
        Event::Attribute {
            name: "name".to_string(),
            value: tag.to_string(),
        },
    ]
}

/// Closing event for a structured HTML element.
pub fn element_close(tag: &str) -> Event {
    Event::EndNode(NodeKind::Element(tag.to_string()))
}

/// A raw markup fragment (a newline, `<br />`, …) that renderers emit verbatim.
///
/// Used for the decorative whitespace that separates block level elements: as a
/// plain `Event::Text` the markdown plugin would swallow it outside of a
/// paragraph, while `HtmlInline` content is always passed through untouched.
pub fn raw_inline(text: &str) -> [Event; 3] {
    [
        Event::StartNode(NodeKind::HtmlInline),
        Event::Text(text.to_string()),
        Event::EndNode(NodeKind::HtmlInline),
    ]
}

/// Returns `true` for HTML void elements: tags that never have children and are
/// rendered self-closing (`<img … />`).
pub fn is_void_element(tag: &str) -> bool {
    matches!(
        tag,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Warning,
    Error,
}

/// Private-Use-Area sentinel that prefixes an [`Event::Attribute`] value when it
/// carries an **encoded JSON literal** — a bound object, array, or non-string
/// scalar produced by a data-binding transform (e.g. `plugin-bind`,
/// `docs/rfc/plugin-bind.md`).
///
/// `Event::Attribute.value` is a `String`, so a structured value is transported
/// as `JSON_ATTR_PREFIX` + compact JSON. The AST builder
/// (`renderer-ast`) strips the prefix and re-hydrates the payload into a real
/// JSON value, which the Solid spread path then emits as `={…}`.
/// Distinct from the `U+E000` pre-markdown protect sentinel (§12.1); no other
/// stage may emit `U+E001`.
pub const JSON_ATTR_PREFIX: &str = "\u{E001}";

/// Private-Use-Area sentinel that prefixes an [`Event::Attribute`] value when it
/// carries a **raw JSX expression** — an already-rendered fragment (e.g. Pendon
/// Markdown rendered to JSX by `plugin-bind`) that must reach Solid wrapped in
/// parentheses: `name={( … )}`. Other renderers strip the prefix and treat the
/// payload as text (the data-binding surface is Solid-first).
pub const JSX_ATTR_PREFIX: &str = "\u{E002}";

/// `true` when `value` carries an encoded JSON literal ([`JSON_ATTR_PREFIX`]).
pub fn is_json_attr(value: &str) -> bool {
    value.starts_with(JSON_ATTR_PREFIX)
}

/// The compact-JSON payload of an encoded JSON literal, or `None` when `value`
/// is not one.
pub fn json_attr_payload(value: &str) -> Option<&str> {
    value.strip_prefix(JSON_ATTR_PREFIX)
}

/// `true` when `value` carries a raw JSX expression ([`JSX_ATTR_PREFIX`]).
pub fn is_jsx_attr(value: &str) -> bool {
    value.starts_with(JSX_ATTR_PREFIX)
}

/// The JSX fragment of a raw-JSX-expression attribute, or `None` when `value`
/// is not one.
pub fn jsx_attr_payload(value: &str) -> Option<&str> {
    value.strip_prefix(JSX_ATTR_PREFIX)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    StartNode(NodeKind),
    EndNode(NodeKind),
    Text(String),
    // Node attribute attached to the nearest open node
    Attribute {
        name: String,
        value: String,
    },
    /// A bare (value-less) attribute on the nearest open node, e.g. `disabled`
    /// or an extras flag (`@@type{isFoo}`). HTML emits `<tag isFoo>` and JSX
    /// emits `<Tag isFoo />`, so no renderer has to invent a placeholder value.
    AttributeFlag {
        name: String,
    },
    // Non-fatal diagnostic event; does not affect renderer concatenation
    Diagnostic {
        severity: Severity,
        message: String,
        span: Option<Span>,
    },
}
