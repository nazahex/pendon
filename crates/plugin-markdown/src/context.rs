use pendon_core::{Event, NodeKind};

use crate::MarkdownOptions;

use crate::helpers::{adjust_blockquote, close_table, emit_html_event};

#[derive(Clone, Debug)]
pub(crate) struct ListFrame {
    pub kind: NodeKind,
    pub indent: usize,
    pub content_indent: usize,
    pub item_open: bool,
    pub blockquote_depth: usize,
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
    pub(crate) last_line_text: Option<String>,
    pub(crate) previous_line_blank: bool,
    pub(crate) display_math_open: bool,
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
            last_line_text: None,
            previous_line_blank: false,
            display_math_open: false,
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

    pub fn pop_list(&mut self) {
        if let Some(frame) = self.list_frames.pop() {
            if frame.item_open {
                self.emit_end(NodeKind::ListItem);
            }
            self.emit_end(frame.kind);
        }
    }

    pub fn close_all_lists(&mut self) {
        while !self.list_frames.is_empty() {
            self.pop_list();
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

    pub fn capture_line_text(&mut self, text: &str) {
        self.last_line_text = Some(text.to_string());
    }

    pub fn use_line_break(&mut self) -> bool {
        if self.in_code_fence || self.in_heading {
            self.last_line_text = None;
            return false;
        }
        if let Some(line) = self.last_line_text.take() {
            if line.ends_with("  ") {
                self.remove_trailing_chars(' ', 2);
                emit_html_event(&mut self.out, "<br />", NodeKind::HtmlInline);
                self.previous_line_blank = false;
                return true;
            }
            if line.ends_with("\\\\") {
                self.remove_trailing_chars('\\', 2);
                emit_html_event(&mut self.out, "<br />", NodeKind::HtmlInline);
                self.previous_line_blank = false;
                return true;
            }
        }
        false
    }

    fn remove_trailing_chars(&mut self, ch: char, mut count: usize) {
        let mut idx = self.out.len();
        while count > 0 && idx > 0 {
            idx -= 1;
            if let Event::Text(text) = &mut self.out[idx] {
                let removed = trim_line_end(text, ch, count);
                if removed > 0 {
                    count -= removed;
                    if text.is_empty() {
                        self.out.remove(idx);
                        idx = self.out.len();
                    }
                }
            }
        }
    }

    pub fn in_list_item(&self) -> bool {
        self.list_frames
            .last()
            .map(|f| f.item_open)
            .unwrap_or(false)
    }
}

fn trim_line_end(text: &mut String, ch: char, max: usize) -> usize {
    let mut removed = 0;
    while removed < max {
        if text.ends_with(ch) {
            text.pop();
            removed += 1;
        } else {
            break;
        }
    }
    removed
}
