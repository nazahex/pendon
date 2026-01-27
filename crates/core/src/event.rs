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
            NodeKind::Custom(name) => Cow::Owned(name.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
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
