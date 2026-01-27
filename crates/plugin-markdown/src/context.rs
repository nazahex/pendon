use pendon_core::{Event, NodeKind};

use crate::MarkdownOptions;

use crate::helpers::{adjust_blockquote, close_table};

#[derive(Clone, Debug)]
pub(crate) struct ListFrame {
    pub kind: NodeKind,
    pub indent: usize,
    pub start_emitted: bool,
    pub item_open: bool,
}

pub struct ParseContext {
    pub(crate) out: Vec<Event>,
    pub(crate) stack: Vec<NodeKind>,
    pub(crate) in_heading: bool,
    pub(crate) heading_prefix_consumed: bool,
    pub(crate) in_code_fence: bool,
    pub(crate) skip_initial_code_newline: bool,
    pub(crate) skip_backticks_once: bool,
    pub(crate) skip_para_open: usize,
    pub(crate) skip_para_close: usize,
    pub(crate) list_frames: Vec<ListFrame>,
    pub(crate) at_line_start: bool,
    pub(crate) pending_para_start: bool,
    pub(crate) blockquote_depth: usize,
    pub(crate) in_table: bool,
    pub(crate) first_table_row: bool,
    pub(crate) options: MarkdownOptions,
}

impl ParseContext {
    pub fn new(capacity: usize, options: MarkdownOptions) -> Self {
        Self {
            out: Vec::with_capacity(capacity),
            stack: Vec::new(),
            in_heading: false,
            heading_prefix_consumed: false,
            in_code_fence: false,
            skip_initial_code_newline: false,
            skip_backticks_once: false,
            skip_para_open: 0,
            skip_para_close: 0,
            list_frames: Vec::new(),
            at_line_start: false,
            pending_para_start: false,
            blockquote_depth: 0,
            in_table: false,
            first_table_row: true,
            options,
        }
    }

    pub fn emit_start(&mut self, kind: NodeKind) {
        self.out.push(Event::StartNode(kind.clone()));
        self.stack.push(kind);
    }

    pub fn emit_end(&mut self, kind: NodeKind) {
        self.out.push(Event::EndNode(kind.clone()));
        let _ = self.stack.pop();
    }

    pub fn push_event(&mut self, event: &Event) {
        self.out.push(event.clone());
    }

    pub fn close_all_lists(&mut self) {
        while let Some(frame) = self.list_frames.pop() {
            if frame.item_open {
                self.emit_end(NodeKind::ListItem);
            }
            self.emit_end(frame.kind);
        }
    }

    pub fn close_table_if_open(&mut self) {
        if self.in_table {
            close_table(&mut self.out, &mut self.in_table);
            self.first_table_row = true;
        }
    }

    pub fn close_blockquotes(&mut self) {
        if self.blockquote_depth > 0 {
            adjust_blockquote(&mut self.out, &mut self.blockquote_depth, 0);
        }
    }

    pub fn finalize(mut self) -> Vec<Event> {
        self.close_all_lists();
        if self.in_table {
            close_table(&mut self.out, &mut self.in_table);
            self.first_table_row = true;
        }
        if self.blockquote_depth > 0 {
            for _ in 0..self.blockquote_depth {
                self.out.push(Event::EndNode(NodeKind::Blockquote));
            }
            self.blockquote_depth = 0;
        }
        self.out
    }

    pub fn close_lists_above(&mut self, indent: usize) {
        while self
            .list_frames
            .last()
            .map(|f| f.indent > indent)
            .unwrap_or(false)
        {
            let frame = self.list_frames.pop().unwrap();
            if frame.item_open {
                self.emit_end(NodeKind::ListItem);
            }
            self.emit_end(frame.kind);
        }
    }

    pub fn ensure_list(&mut self, kind: NodeKind, indent: usize, start: Option<usize>) {
        if let Some(frame) = self.list_frames.last_mut() {
            if frame.indent == indent && frame.kind == kind {
                if let (NodeKind::OrderedList, Some(n)) = (&kind, start) {
                    if !frame.start_emitted {
                        self.out.push(Event::Attribute {
                            name: "start".to_string(),
                            value: n.to_string(),
                        });
                        frame.start_emitted = true;
                    }
                }
                return;
            }
            if frame.indent == indent && frame.kind != kind {
                let popped = self.list_frames.pop().unwrap();
                if popped.item_open {
                    self.emit_end(NodeKind::ListItem);
                }
                self.emit_end(popped.kind);
            }
        }

        if let Some(frame) = self.list_frames.last() {
            if frame.indent < indent {
                self.open_list(kind, indent, start);
                return;
            }
        }

        if self
            .list_frames
            .last()
            .map(|f| f.indent > indent)
            .unwrap_or(false)
        {
            self.close_lists_above(indent);
        }
        self.open_list(kind, indent, start);
    }

    fn open_list(&mut self, kind: NodeKind, indent: usize, start: Option<usize>) {
        self.emit_start(kind.clone());
        if let (NodeKind::OrderedList, Some(n)) = (kind.clone(), start) {
            self.out.push(Event::Attribute {
                name: "start".to_string(),
                value: n.to_string(),
            });
        }
        self.list_frames.push(ListFrame {
            kind,
            indent,
            start_emitted: start.is_some(),
            item_open: false,
        });
    }

    pub fn start_list_item(&mut self) {
        let item_already_open = self.in_list_item();
        if item_already_open {
            self.emit_end(NodeKind::ListItem);
            if let Some(frame) = self.list_frames.last_mut() {
                frame.item_open = false;
            }
        }
        self.emit_start(NodeKind::ListItem);
        if let Some(frame) = self.list_frames.last_mut() {
            frame.item_open = true;
        }
    }

    pub fn current_list_start_emitted(&self) -> bool {
        self.list_frames
            .last()
            .map(|f| f.start_emitted)
            .unwrap_or(false)
    }

    pub fn mark_current_list_start_emitted(&mut self) {
        if let Some(frame) = self.list_frames.last_mut() {
            frame.start_emitted = true;
        }
    }

    pub fn in_list_item(&self) -> bool {
        self.list_frames
            .last()
            .map(|f| f.item_open)
            .unwrap_or(false)
    }
}
