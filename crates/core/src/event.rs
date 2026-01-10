#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    Document,
    Paragraph,
    // Skeleton variants for future parsing; currently unused by renderer
    CodeFence,
    Heading,
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
    // Non-fatal diagnostic event; does not affect renderer concatenation
    Diagnostic {
        severity: Severity,
        message: String,
        span: Option<Span>,
    },
}
