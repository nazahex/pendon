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
    // Non-fatal diagnostic event; does not affect renderer concatenation
    Diagnostic {
        severity: Severity,
        message: String,
        span: Option<Span>,
    },
}
