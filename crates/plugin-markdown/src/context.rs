use pendon_core::{Event, NodeKind};
use pendon_extra::{emit_attr_warnings, emit_attrs, Attrs};

use crate::ContainerLayer;
use crate::MarkdownOptions;

use crate::helpers::{adjust_blockquote, close_table, emit_html_event};

#[derive(Clone, Debug)]
pub(crate) struct ListFrame {
    pub kind: NodeKind,
    pub indent: usize,
    pub content_indent: usize,
    pub item_open: bool,
    pub blockquote_depth: usize,
    /// §9.3/§9.4: when the container node of this list was already emitted by a
    /// `plugin-list` wrapper, the wrapper's node kind. Closing the list then
    /// emits no `EndNode` of its own (the wrapper's does) and the frame ends
    /// together with the wrapper.
    pub container: Option<NodeKind>,
}

/// §9.3/§9.4: a `plugin-list` container wrapper that is waiting for the list it
/// decorates.
#[derive(Clone, Debug)]
pub(crate) struct PendingListContainer {
    /// The layer the wrapper declared (`unordered` / `ordered`).
    pub layer: ContainerLayer,
    /// The wrapper's own node, which becomes the container of the list.
    pub node: NodeKind,
}

/// §9.1: a decorator line bound to the block node the Markdown pass is about to
/// emit. Paragraphs, headings, code fences and tables are the blocks
/// `plugin-markdown` owns; the construct plugins (`plugin-list`,
/// `plugin-blockquote`) bind their own decorators before this pass runs.
#[derive(Clone, Debug)]
pub(crate) struct PendingBlockDecorator {
    /// The head's type marker, emitted as the `type` attribute (§11 rule 3).
    pub type_marker: Option<String>,
    /// The decorator's extras, in canonical order (§6.4).
    pub attrs: Attrs,
}

pub struct ParseContext {
    pub(crate) out: Vec<Event>,
    pub(crate) stack: Vec<NodeKind>,
    // Parallel stack to safely track inline custom contexts without desyncing
    pub(crate) stack_is_inline_custom: Vec<bool>,
    // Parallel stack marking structured HTML element subtrees (`figure`,
    // `table`, …) whose children are already fully processed.
    pub(crate) stack_is_element: Vec<bool>,
    pub(crate) in_heading: bool,
    pub(crate) heading_prefix_consumed: bool,
    pub(crate) heading_prefix_from_input: bool,
    pub(crate) in_code_fence: bool,
    /// Leading indentation of the opening fence line. CommonMark removes up to
    /// this many spaces from every content line, so code inside an indented
    /// fence (a list item, for example) keeps its relative layout.
    pub(crate) code_fence_indent: usize,
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
    pub(crate) in_html_comment: bool,
    pub(crate) html_comment_buffer: String,
    /// §9.3/§9.4: a `plugin-list` container wrapper was opened and is waiting
    /// for the list it decorates. Cleared by the first line that is not a list
    /// item, by the list that adopts it, and by the wrapper itself.
    pub(crate) pending_list_container: Option<PendingListContainer>,
    /// §9.1: a decorator line bound to the block node about to be emitted.
    /// Armed when the decorated input node is seen and flushed onto the first
    /// block node the pass opens for it.
    pub(crate) pending_block_decorator: Option<PendingBlockDecorator>,
}

impl ParseContext {
    pub fn new(capacity: usize, options: MarkdownOptions) -> Self {
        Self {
            out: Vec::with_capacity(capacity),
            stack: Vec::new(),
            stack_is_inline_custom: Vec::new(),
            stack_is_element: Vec::new(),
            in_heading: false,
            heading_prefix_consumed: false,
            heading_prefix_from_input: false,
            in_code_fence: false,
            code_fence_indent: 0,
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
            in_html_comment: false,
            html_comment_buffer: String::new(),
            pending_list_container: None,
            pending_block_decorator: None,
        }
    }

    pub fn emit_start(&mut self, kind: NodeKind) {
        self.out.push(Event::StartNode(kind.clone()));
        self.stack.push(kind);
        self.stack_is_inline_custom.push(false);
        self.stack_is_element.push(false);
    }

    // Specifically marks the node as an inline-only container (e.g., Figcaption)
    pub fn emit_start_inline_custom(&mut self, kind: NodeKind) {
        self.out.push(Event::StartNode(kind.clone()));
        self.stack.push(kind);
        self.stack_is_inline_custom.push(true);
        self.stack_is_element.push(false);
    }

    // Marks a structured HTML element container whose children are already
    // rendered events and must be passed through verbatim.
    pub fn emit_start_element(&mut self, kind: NodeKind) {
        self.out.push(Event::StartNode(kind.clone()));
        self.stack.push(kind);
        self.stack_is_inline_custom.push(false);
        self.stack_is_element.push(true);
    }

    pub fn emit_end(&mut self, kind: NodeKind) {
        self.out.push(Event::EndNode(kind.clone()));
        let _ = self.stack.pop();
        let _ = self.stack_is_inline_custom.pop();
        let _ = self.stack_is_element.pop();
    }

    // Checks if we are currently inside any inline custom container
    pub fn is_in_inline_context(&self) -> bool {
        self.stack_is_inline_custom.iter().any(|&b| b)
    }

    // Checks if we are currently inside a structured HTML element subtree.
    pub fn is_in_element_context(&self) -> bool {
        self.stack_is_element.iter().any(|&b| b)
    }

    pub fn push_event(&mut self, event: &Event) {
        self.out.push(event.clone());
    }

    pub fn pop_list(&mut self) {
        if let Some(frame) = self.list_frames.pop() {
            if frame.item_open {
                self.emit_end(NodeKind::ListItem);
            }
            // §9.3/§9.4: a merged container was emitted by the wrapper.
            if frame.container.is_none() {
                self.emit_end(frame.kind);
            }
        }
    }

    /// Closes a list frame the way the line handlers do — a raw `out.push` with
    /// no `ctx.stack` bookkeeping: its open item, and unless the container node
    /// was already emitted by a §9.3/§9.4 wrapper, the container itself.
    pub fn close_list_frame(&mut self, frame: ListFrame) {
        if frame.item_open {
            self.out.push(Event::EndNode(NodeKind::ListItem));
        }
        if frame.container.is_none() {
            self.out.push(Event::EndNode(frame.kind));
        }
    }

    /// §9.3/§9.4: the `plugin-list` wrapper node just opened — the list it
    /// decorates **is** that node, so the list build adopts it as its container.
    pub fn arm_list_container(&mut self, layer: ContainerLayer, node: NodeKind) {
        self.pending_list_container = Some(PendingListContainer { layer, node });
    }

    /// Consumes the armed wrapper when it stands for the list `kind` about to be
    /// built, returning the wrapper node that now acts as the list container. A
    /// list that adopts it must not open a `<ul>`/`<ol>` of its own.
    pub fn take_list_container(&mut self, kind: &NodeKind) -> Option<NodeKind> {
        let pending = self.pending_list_container.take()?;
        (pending.layer == layer_of_list(kind)).then_some(pending.node)
    }

    /// §9.1: arms the decorator that decorates the block node about to be emitted.
    /// The next block node `plugin-markdown` opens flushes it (see
    /// [`Self::flush_block_decorator`]).
    pub fn arm_block_decorator(&mut self, type_marker: Option<String>, attrs: Attrs) {
        self.pending_block_decorator = Some(PendingBlockDecorator { type_marker, attrs });
    }

    /// §9.1: flushes the armed decorator onto the block node just opened. The
    /// type marker becomes the `type` attribute (the §11 rule 3 fallback keeps
    /// every extra) and the extras follow in canonical order (§6.4). A no-op when
    /// nothing is armed.
    pub fn flush_block_decorator(&mut self) {
        let Some(decorator) = self.pending_block_decorator.take() else {
            return;
        };
        if let Some(marker) = decorator.type_marker {
            self.out.push(Event::Attribute {
                name: "type".to_string(),
                value: marker,
            });
        }
        emit_attr_warnings("markdown", &decorator.attrs, &mut self.out);
        emit_attrs(&decorator.attrs, &mut self.out);
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

/// §9.4: the container layer a list of `kind` belongs to.
pub(crate) fn layer_of_list(kind: &NodeKind) -> ContainerLayer {
    match kind {
        NodeKind::OrderedList => ContainerLayer::Ordered,
        _ => ContainerLayer::Unordered,
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
