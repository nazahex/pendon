use pendon_core::{Event, NodeKind, Severity};
use serde::ser::{Serialize, SerializeMap, Serializer};
use serde::Serialize as DeriveSerialize;
use std::collections::BTreeMap;

struct AstNode {
    kind: String,
    text: Option<String>,
    // Move children before attrs to enforce field order: type -> text -> children -> attrs
    children: Vec<AstNode>,
    attrs: BTreeMap<String, String>,
}

struct AstDocument {
    kind: String,
    children: Vec<AstNode>,
    diagnostics: Vec<AstDiag>,
}

#[derive(DeriveSerialize)]
struct AstDiag {
    severity: String,
    message: String,
}

impl Serialize for AstNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut len = 1; // type
        if self.text.is_some() {
            len += 1;
        }
        if !self.children.is_empty() {
            len += 1;
        }
        if !self.attrs.is_empty() {
            len += 1;
        }
        let mut map = serializer.serialize_map(Some(len))?;
        map.serialize_entry("type", &self.kind)?;
        if let Some(ref t) = self.text {
            map.serialize_entry("text", t)?;
        }
        if !self.children.is_empty() {
            map.serialize_entry("children", &self.children)?;
        }
        if !self.attrs.is_empty() {
            map.serialize_entry("attrs", &self.attrs)?;
        }
        map.end()
    }
}

impl Serialize for AstDocument {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut len = 2; // type + children
        if !self.diagnostics.is_empty() {
            len += 1;
        }
        let mut map = serializer.serialize_map(Some(len))?;
        map.serialize_entry("type", &self.kind)?;
        map.serialize_entry("children", &self.children)?;
        if !self.diagnostics.is_empty() {
            map.serialize_entry("diagnostics", &self.diagnostics)?;
        }
        map.end()
    }
}

pub fn render_ast_to_string(events: &[Event]) -> Result<String, serde_json::Error> {
    let mut stack: Vec<AstNode> = Vec::new();
    let mut text_bufs: Vec<String> = Vec::new();
    let mut has_inline: Vec<bool> = Vec::new();
    let mut kind_stack: Vec<NodeKind> = Vec::new();
    let mut doc = AstDocument {
        kind: "Document".to_string(),
        children: Vec::new(),
        diagnostics: Vec::new(),
    };

    let flush_text_child = |stack: &mut Vec<AstNode>, text_bufs: &mut Vec<String>| {
        if let (Some(cur), Some(buf)) = (stack.last_mut(), text_bufs.last_mut()) {
            if !buf.is_empty() {
                cur.children.push(AstNode {
                    kind: "Text".to_string(),
                    text: Some(buf.clone()),
                    children: Vec::new(),
                    attrs: BTreeMap::new(),
                });
                buf.clear();
            }
        }
    };

    for ev in events {
        match ev {
            Event::StartNode(kind) => {
                if matches!(kind, NodeKind::Document) {
                    continue;
                }
                if !stack.is_empty() {
                    flush_text_child(&mut stack, &mut text_bufs);
                }
                match kind {
                    NodeKind::Emphasis
                    | NodeKind::Strong
                    | NodeKind::Bold
                    | NodeKind::Italic
                    | NodeKind::InlineCode
                    | NodeKind::Link => {
                        if let Some(flag) = has_inline.last_mut() {
                            *flag = true;
                        }
                    }
                    _ => {}
                }
                stack.push(AstNode {
                    kind: node_str(kind).to_string(),
                    text: None,
                    children: Vec::new(),
                    attrs: BTreeMap::new(),
                });
                text_bufs.push(String::new());
                has_inline.push(false);
                kind_stack.push(kind.clone());
            }
            Event::EndNode(kind) => {
                if matches!(kind, NodeKind::Document) {
                    continue;
                }
                if let Some(top) = kind_stack.last() {
                    if top != kind {
                        continue;
                    }
                } else {
                    continue;
                }

                let mut node = stack.pop().unwrap();
                let buf = text_bufs.pop().unwrap_or_default();
                let inline = has_inline.pop().unwrap_or(false);
                let nk = kind_stack.pop().unwrap_or(NodeKind::Paragraph);
                let children_only = match nk {
                    NodeKind::Paragraph => true,
                    NodeKind::CodeFence => false,
                    NodeKind::Heading | NodeKind::ListItem => inline,
                    NodeKind::BulletList
                    | NodeKind::OrderedList
                    | NodeKind::Blockquote
                    | NodeKind::Table
                    | NodeKind::TableHead
                    | NodeKind::TableBody
                    | NodeKind::TableRow
                    | NodeKind::TableCell
                    | NodeKind::Frontmatter => true,
                    NodeKind::Emphasis
                    | NodeKind::Strong
                    | NodeKind::Bold
                    | NodeKind::Italic
                    | NodeKind::InlineCode
                    | NodeKind::Link => true,
                    NodeKind::ThematicBreak => true,
                    NodeKind::Document => true,
                };

                if children_only {
                    if !buf.is_empty() {
                        node.children.push(AstNode {
                            kind: "Text".to_string(),
                            text: Some(buf),
                            children: Vec::new(),
                            attrs: BTreeMap::new(),
                        });
                    }
                    node.text = None;
                } else {
                    if !inline {
                        node.children.retain(|child| child.kind != "Text");
                        if matches!(nk, NodeKind::Heading | NodeKind::ListItem)
                            && node.text.is_none()
                            && !node.children.is_empty()
                        {
                            fn collect_texts(n: &AstNode, out: &mut String) {
                                if let Some(t) = &n.text {
                                    out.push_str(t);
                                }
                                for ch in &n.children {
                                    collect_texts(ch, out);
                                }
                            }
                            let mut acc = String::new();
                            for ch in &node.children {
                                collect_texts(ch, &mut acc);
                            }
                            if !acc.is_empty() {
                                node.text = Some(acc);
                                node.children.clear();
                            }
                        }
                    }
                }

                if let Some(parent) = stack.last_mut() {
                    parent.children.push(node);
                } else {
                    doc.children.push(node);
                }
            }
            Event::Text(s) => {
                if let Some(cur) = stack.last_mut() {
                    let is_paragraph = matches!(kind_stack.last(), Some(NodeKind::Paragraph));
                    if !is_paragraph {
                        if cur.text.is_none() {
                            cur.text = Some(String::new());
                        }
                        if let Some(t) = &mut cur.text {
                            t.push_str(s);
                        }
                    }
                    if let Some(buf) = text_bufs.last_mut() {
                        buf.push_str(s);
                    }
                } else {
                    doc.children.push(AstNode {
                        kind: "Text".to_string(),
                        text: Some(s.clone()),
                        children: Vec::new(),
                        attrs: BTreeMap::new(),
                    });
                }
            }
            Event::Attribute { name, value } => {
                if let Some(cur) = stack.last_mut() {
                    cur.attrs.insert(name.clone(), value.clone());
                }
            }
            Event::Diagnostic {
                severity, message, ..
            } => {
                doc.diagnostics.push(AstDiag {
                    severity: match severity {
                        Severity::Warning => "Warning".to_string(),
                        Severity::Error => "Error".to_string(),
                    },
                    message: message.clone(),
                });
            }
        }
    }

    while let Some(mut node) = stack.pop() {
        let buf = text_bufs.pop().unwrap_or_default();
        let inline = has_inline.pop().unwrap_or(false);
        let nk = kind_stack.pop().unwrap_or(NodeKind::Paragraph);
        let children_only = match nk {
            NodeKind::Paragraph => true,
            NodeKind::CodeFence => false,
            NodeKind::Heading | NodeKind::ListItem => inline,
            NodeKind::BulletList
            | NodeKind::OrderedList
            | NodeKind::Blockquote
            | NodeKind::Table
            | NodeKind::TableHead
            | NodeKind::TableBody
            | NodeKind::TableRow
            | NodeKind::TableCell
            | NodeKind::Frontmatter => true,
            NodeKind::Emphasis
            | NodeKind::Strong
            | NodeKind::Bold
            | NodeKind::Italic
            | NodeKind::InlineCode
            | NodeKind::Link => true,
            NodeKind::ThematicBreak => true,
            NodeKind::Document => true,
        };
        if children_only {
            if !buf.is_empty() {
                node.children.push(AstNode {
                    kind: "Text".to_string(),
                    text: Some(buf),
                    children: Vec::new(),
                    attrs: BTreeMap::new(),
                });
            }
            node.text = None;
        } else if !inline {
            node.children.retain(|child| child.kind != "Text");
            if matches!(nk, NodeKind::Heading | NodeKind::ListItem)
                && node.text.is_none()
                && !node.children.is_empty()
            {
                fn collect_texts(n: &AstNode, out: &mut String) {
                    if let Some(t) = &n.text {
                        out.push_str(t);
                    }
                    for ch in &n.children {
                        collect_texts(ch, out);
                    }
                }
                let mut acc = String::new();
                for ch in &node.children {
                    collect_texts(ch, &mut acc);
                }
                if !acc.is_empty() {
                    node.text = Some(acc);
                    node.children.clear();
                }
            }
        }

        if let Some(parent) = stack.last_mut() {
            parent.children.push(node);
        } else {
            doc.children.push(node);
        }
    }

    serde_json::to_string(&doc)
}

fn node_str(k: &NodeKind) -> &str {
    match k {
        NodeKind::Document => "Document",
        NodeKind::Frontmatter => "Frontmatter",
        NodeKind::Paragraph => "Paragraph",
        NodeKind::Blockquote => "Blockquote",
        NodeKind::CodeFence => "CodeFence",
        NodeKind::Heading => "Heading",
        NodeKind::ThematicBreak => "ThematicBreak",
        NodeKind::BulletList => "BulletList",
        NodeKind::OrderedList => "OrderedList",
        NodeKind::ListItem => "ListItem",
        NodeKind::Table => "Table",
        NodeKind::TableHead => "TableHead",
        NodeKind::TableBody => "TableBody",
        NodeKind::TableRow => "TableRow",
        NodeKind::TableCell => "TableCell",
        NodeKind::Emphasis => "Emphasis",
        NodeKind::Strong => "Strong",
        NodeKind::InlineCode => "InlineCode",
        NodeKind::Link => "Link",
        NodeKind::Bold => "Bold",
        NodeKind::Italic => "Italic",
    }
}

// Provide a pretty variant that preserves ordering via custom Serialize
pub fn render_ast_to_string_pretty(events: &[Event]) -> Result<String, serde_json::Error> {
    // Build doc using same logic by reusing render_ast_to_string construction
    // Simpler approach: duplicate builder here for clarity
    let mut stack: Vec<AstNode> = Vec::new();
    let mut text_bufs: Vec<String> = Vec::new();
    let mut has_inline: Vec<bool> = Vec::new();
    let mut kind_stack: Vec<NodeKind> = Vec::new();
    let mut doc = AstDocument {
        kind: "Document".to_string(),
        children: Vec::new(),
        diagnostics: Vec::new(),
    };

    let flush_text_child = |stack: &mut Vec<AstNode>, text_bufs: &mut Vec<String>| {
        if let (Some(cur), Some(buf)) = (stack.last_mut(), text_bufs.last_mut()) {
            if !buf.is_empty() {
                cur.children.push(AstNode {
                    kind: "Text".to_string(),
                    text: Some(buf.clone()),
                    children: Vec::new(),
                    attrs: BTreeMap::new(),
                });
                buf.clear();
            }
        }
    };

    for ev in events {
        match ev {
            Event::StartNode(kind) => {
                if !matches!(kind, NodeKind::Document) {
                    if !stack.is_empty() {
                        flush_text_child(&mut stack, &mut text_bufs);
                    }
                    match kind {
                        NodeKind::Emphasis
                        | NodeKind::Strong
                        | NodeKind::Bold
                        | NodeKind::Italic
                        | NodeKind::InlineCode
                        | NodeKind::Link => {
                            if let Some(flag) = has_inline.last_mut() {
                                *flag = true;
                            }
                        }
                        _ => {}
                    }
                    stack.push(AstNode {
                        kind: node_str(kind).to_string(),
                        text: None,
                        children: Vec::new(),
                        attrs: BTreeMap::new(),
                    });
                    text_bufs.push(String::new());
                    has_inline.push(false);
                    kind_stack.push(kind.clone());
                }
            }
            Event::EndNode(kind) => {
                if !matches!(kind, NodeKind::Document) {
                    if let Some(top) = kind_stack.last() {
                        if top != kind {
                            continue;
                        }
                    } else {
                        continue;
                    }
                    let mut node = stack.pop().unwrap();
                    let buf = text_bufs.pop().unwrap_or_default();
                    let inline = has_inline.pop().unwrap_or(false);
                    let nk = kind_stack.pop().unwrap_or(NodeKind::Paragraph);
                    let children_only = match nk {
                        NodeKind::Paragraph => true,
                        NodeKind::CodeFence => false,
                        NodeKind::Heading | NodeKind::ListItem => inline,
                        NodeKind::BulletList
                        | NodeKind::OrderedList
                        | NodeKind::Blockquote
                        | NodeKind::Table
                        | NodeKind::TableHead
                        | NodeKind::TableBody
                        | NodeKind::TableRow
                        | NodeKind::TableCell
                        | NodeKind::Frontmatter => true,
                        NodeKind::Emphasis
                        | NodeKind::Strong
                        | NodeKind::Bold
                        | NodeKind::Italic
                        | NodeKind::InlineCode
                        | NodeKind::Link => true,
                        NodeKind::ThematicBreak => true,
                        NodeKind::Document => true,
                    };
                    if children_only {
                        if !buf.is_empty() {
                            node.children.push(AstNode {
                                kind: "Text".to_string(),
                                text: Some(buf),
                                children: Vec::new(),
                                attrs: BTreeMap::new(),
                            });
                        }
                        node.text = None;
                    } else {
                        if !inline {
                            node.children.retain(|child| child.kind != "Text");
                            if matches!(nk, NodeKind::Heading | NodeKind::ListItem)
                                && node.text.is_none()
                                && !node.children.is_empty()
                            {
                                fn collect_texts(n: &AstNode, out: &mut String) {
                                    if let Some(t) = &n.text {
                                        out.push_str(t);
                                    }
                                    for ch in &n.children {
                                        collect_texts(ch, out);
                                    }
                                }
                                let mut acc = String::new();
                                for ch in &node.children {
                                    collect_texts(ch, &mut acc);
                                }
                                if !acc.is_empty() {
                                    node.text = Some(acc);
                                    node.children.clear();
                                }
                            }
                        }
                    }
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(node);
                    } else {
                        doc.children.push(node);
                    }
                }
            }
            Event::Text(s) => {
                if let Some(cur) = stack.last_mut() {
                    let is_paragraph = matches!(kind_stack.last(), Some(NodeKind::Paragraph));
                    if !is_paragraph {
                        if cur.text.is_none() {
                            cur.text = Some(String::new());
                        }
                        if let Some(t) = &mut cur.text {
                            t.push_str(s);
                        }
                    }
                    if let Some(buf) = text_bufs.last_mut() {
                        buf.push_str(s);
                    }
                } else {
                    doc.children.push(AstNode {
                        kind: "Text".to_string(),
                        text: Some(s.clone()),
                        children: Vec::new(),
                        attrs: BTreeMap::new(),
                    });
                }
            }
            Event::Attribute { name, value } => {
                if let Some(cur) = stack.last_mut() {
                    cur.attrs.insert(name.clone(), value.clone());
                }
            }
            Event::Diagnostic {
                severity, message, ..
            } => {
                doc.diagnostics.push(AstDiag {
                    severity: match severity {
                        Severity::Warning => "Warning".to_string(),
                        Severity::Error => "Error".to_string(),
                    },
                    message: message.clone(),
                });
            }
        }
    }
    serde_json::to_string_pretty(&doc)
}
