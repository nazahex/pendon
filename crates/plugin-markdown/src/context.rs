use pendon_core::{Event, NodeKind};

use crate::helpers::{adjust_blockquote, close_table};

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
    pub(crate) list_stack: Vec<(NodeKind, usize)>,
    pub(crate) in_list_item: bool,
    pub(crate) at_line_start: bool,
    pub(crate) ordered_start: Option<usize>,
    pub(crate) pending_para_start: bool,
    pub(crate) blockquote_depth: usize,
    pub(crate) in_table: bool,
    pub(crate) first_table_row: bool,
}

impl ParseContext {
    pub fn new(capacity: usize) -> Self {
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
            list_stack: Vec::new(),
            in_list_item: false,
            at_line_start: false,
            ordered_start: None,
            pending_para_start: false,
            blockquote_depth: 0,
            in_table: false,
            first_table_row: true,
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
        if self.in_list_item {
            self.emit_end(NodeKind::ListItem);
            self.in_list_item = false;
        }
        while let Some((kind, _)) = self.list_stack.pop() {
            self.emit_end(kind);
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
}
