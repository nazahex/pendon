use pendon_core::{Event, NodeKind, Severity};
use serde::ser::{Serialize, SerializeMap, Serializer};
use serde::Serialize as DeriveSerialize;
use std::collections::BTreeMap;

struct AstNode {
    kind: String,
    text: Option<String>,
    children: Vec<AstNode>,
    attrs: BTreeMap<String, String>,
}

pub(super) struct AstDocument {
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
        let mut len = 1;
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
        let mut len = 2;
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

struct AstBuilder {
    stack: Vec<AstNode>,
    text_bufs: Vec<String>,
    has_inline: Vec<bool>,
    kind_stack: Vec<NodeKind>,
    doc: AstDocument,
}

impl AstBuilder {
    fn new() -> Self {
        Self {
            stack: Vec::new(),
            text_bufs: Vec::new(),
            has_inline: Vec::new(),
            kind_stack: Vec::new(),
            doc: AstDocument {
                kind: "Document".to_string(),
                children: Vec::new(),
                diagnostics: Vec::new(),
            },
        }
    }

    fn consume(&mut self, events: &[Event]) {
        for ev in events {
            match ev {
                Event::StartNode(kind) => self.handle_start(kind),
                Event::EndNode(kind) => self.handle_end(kind),
                Event::Text(text) => self.handle_text(text),
                Event::Attribute { name, value } => self.handle_attribute(name, value),
                Event::Diagnostic {
                    severity, message, ..
                } => self.handle_diagnostic(severity, message),
            }
        }
    }

    fn finalize(mut self) -> AstDocument {
        self.close_remaining();
        self.doc
    }

    fn handle_start(&mut self, kind: &NodeKind) {
        if matches!(kind, NodeKind::Document) {
            return;
        }
        if !self.stack.is_empty() {
            self.flush_text_child();
        }
        match kind {
            NodeKind::Emphasis
            | NodeKind::Strong
            | NodeKind::Bold
            | NodeKind::Italic
            | NodeKind::InlineCode
            | NodeKind::Link => {
                if let Some(flag) = self.has_inline.last_mut() {
                    *flag = true;
                }
            }
            _ => {}
        }
        self.stack.push(AstNode {
            kind: kind.name().into_owned(),
            text: None,
            children: Vec::new(),
            attrs: BTreeMap::new(),
        });
        self.text_bufs.push(String::new());
        self.has_inline.push(false);
        self.kind_stack.push(kind.clone());
    }

    fn handle_end(&mut self, kind: &NodeKind) {
        if let Some(node) = self.finish_node(kind) {
            self.push_to_parent(node);
        }
    }

    fn handle_text(&mut self, text: &str) {
        if let Some(cur) = self.stack.last_mut() {
            let is_paragraph = matches!(self.kind_stack.last(), Some(NodeKind::Paragraph));
            if !is_paragraph {
                if cur.text.is_none() {
                    cur.text = Some(String::new());
                }
                if let Some(t) = cur.text.as_mut() {
                    t.push_str(text);
                }
            }
            if let Some(buf) = self.text_bufs.last_mut() {
                buf.push_str(text);
            }
        } else {
            self.doc.children.push(AstNode {
                kind: "Text".to_string(),
                text: Some(text.to_string()),
                children: Vec::new(),
                attrs: BTreeMap::new(),
            });
        }
    }

    fn handle_attribute(&mut self, name: &str, value: &str) {
        if let Some(cur) = self.stack.last_mut() {
            cur.attrs.insert(name.to_string(), value.to_string());
        }
    }

    fn handle_diagnostic(&mut self, severity: &Severity, message: &str) {
        self.doc.diagnostics.push(AstDiag {
            severity: match severity {
                Severity::Warning => "Warning".to_string(),
                Severity::Error => "Error".to_string(),
            },
            message: message.to_string(),
        });
    }

    fn finish_node(&mut self, kind: &NodeKind) -> Option<AstNode> {
        if matches!(kind, NodeKind::Document) {
            return None;
        }
        if let Some(top) = self.kind_stack.last() {
            if top != kind {
                return None;
            }
        } else {
            return None;
        }
        self.flush_text_child();
        let mut node = self.stack.pop().unwrap();
        let buf = self.text_bufs.pop().unwrap_or_default();
        let inline = self.has_inline.pop().unwrap_or(false);
        let nk = self.kind_stack.pop().unwrap_or(NodeKind::Paragraph);
        Self::apply_text(&mut node, buf, inline, &nk);
        Some(node)
    }

    fn push_to_parent(&mut self, node: AstNode) {
        if let Some(parent) = self.stack.last_mut() {
            parent.children.push(node);
        } else {
            self.doc.children.push(node);
        }
    }

    fn flush_text_child(&mut self) {
        if let (Some(cur), Some(buf)) = (self.stack.last_mut(), self.text_bufs.last()) {
            if !buf.is_empty() {
                cur.children.push(AstNode {
                    kind: "Text".to_string(),
                    text: Some(buf.clone()),
                    children: Vec::new(),
                    attrs: BTreeMap::new(),
                });
                if let Some(buffer) = self.text_bufs.last_mut() {
                    buffer.clear();
                }
            }
        }
    }

    fn close_remaining(&mut self) {
        while let Some(kind) = self.kind_stack.last().cloned() {
            if let Some(node) = self.finish_node(&kind) {
                self.push_to_parent(node);
            }
        }
    }

    fn apply_text(node: &mut AstNode, buf: String, inline: bool, kind: &NodeKind) {
        if Self::children_only(kind, inline) {
            if !buf.is_empty() {
                node.children.push(AstNode {
                    kind: "Text".to_string(),
                    text: Some(buf),
                    children: Vec::new(),
                    attrs: BTreeMap::new(),
                });
            }
            node.text = None;
            return;
        }
        if !inline {
            node.children.retain(|child| child.kind != "Text");
            if matches!(kind, NodeKind::Heading | NodeKind::ListItem)
                && node.text.is_none()
                && !node.children.is_empty()
            {
                let mut acc = String::new();
                Self::collect_texts(&node.children, &mut acc);
                if !acc.is_empty() {
                    node.text = Some(acc);
                    node.children.clear();
                }
            }
        }
    }

    fn children_only(kind: &NodeKind, inline: bool) -> bool {
        match kind {
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
            | NodeKind::Section
            | NodeKind::Frontmatter => true,
            NodeKind::HtmlBlock | NodeKind::HtmlInline => false,
            NodeKind::Emphasis
            | NodeKind::Strong
            | NodeKind::Bold
            | NodeKind::Italic
            | NodeKind::InlineCode
            | NodeKind::Link
            | NodeKind::ThematicBreak
            | NodeKind::Document
            | NodeKind::Custom(_) => true,
        }
    }

    fn collect_texts(nodes: &[AstNode], out: &mut String) {
        for node in nodes {
            if let Some(text) = &node.text {
                out.push_str(text);
            }
            Self::collect_texts(&node.children, out);
        }
    }
}

pub(crate) fn build_ast_document(events: &[Event]) -> AstDocument {
    let mut builder = AstBuilder::new();
    builder.consume(events);
    builder.finalize()
}
