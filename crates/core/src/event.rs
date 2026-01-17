#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    Document,
    Paragraph,
    CodeFence,
    Heading,
    ThematicBreak,
    BulletList,
    OrderedList,
    ListItem,
    // Inline nodes
    Emphasis,
    Strong,
    InlineCode,
    Link,
    Bold,
    Italic,
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
