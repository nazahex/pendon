// plugin-markdown/start.rs
use pendon_core::NodeKind;

use crate::context::ParseContext;
use crate::CustomPlacement;

fn is_inline_node(kind: &NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::Emphasis
            | NodeKind::Strong
            | NodeKind::InlineCode
            | NodeKind::Link
            | NodeKind::Bold
            | NodeKind::Italic
            | NodeKind::HtmlInline
            | NodeKind::Image
    )
}

pub fn handle(ctx: &mut ParseContext, kind: &NodeKind, placement: CustomPlacement) {
    match kind {
        NodeKind::Heading => {
            ctx.close_all_lists();
            ctx.close_table_if_open();
            ctx.emit_start(NodeKind::Heading);
            ctx.in_heading = true;
            ctx.heading_prefix_consumed = true;
            ctx.heading_prefix_from_input = true;
            ctx.skip_para_open = ctx.skip_para_open.saturating_add(1);
            ctx.skip_para_close = ctx.skip_para_close.saturating_add(1);
        }
        NodeKind::CodeFence => {
            ctx.close_all_lists();
            ctx.close_table_if_open();
            ctx.emit_start(NodeKind::CodeFence);
            ctx.in_code_fence = true;
            // A fence that reaches `plugin-markdown` as a `CodeFence` node starts
            // at column 0; indented fences are detected on the raw line instead.
            ctx.code_fence_indent = 0;
            ctx.skip_initial_code_newline = true;
            ctx.skip_para_open = ctx.skip_para_open.saturating_add(1);
            ctx.skip_para_close = ctx.skip_para_close.saturating_add(1);
        }
        NodeKind::ThematicBreak => {
            ctx.close_all_lists();
            ctx.close_table_if_open();
            ctx.emit_start(NodeKind::ThematicBreak);
        }
        NodeKind::Paragraph => {
            if ctx.skip_para_open > 0 {
                ctx.skip_para_open = ctx.skip_para_open.saturating_sub(1);
            } else {
                ctx.pending_para_start = true;
            }
            ctx.at_line_start = true;
        }
        // Structured HTML element containers emitted by the img/table plugins.
        // Their children are already rendered events (including custom
        // components), so the whole subtree is passed through verbatim.
        NodeKind::Element(_) => {
            if ctx.pending_para_start {
                ctx.emit_start(NodeKind::Paragraph);
                ctx.pending_para_start = false;
            }
            ctx.emit_start_element(kind.clone());
        }
        // Handle Custom nodes explicitly based on __plugin_kind metadata
        NodeKind::Custom(_) => match placement {
            CustomPlacement::Block => {
                ctx.close_blockquotes();
                ctx.close_all_lists();
                ctx.close_table_if_open();
                ctx.emit_start(kind.clone());
            }
            CustomPlacement::Element => {
                // Children were already rendered by the emitting plugin (table
                // cells, captions, …): mark the subtree so its text is passed
                // through verbatim instead of being re-lexed.
                if ctx.pending_para_start {
                    ctx.emit_start(NodeKind::Paragraph);
                    ctx.pending_para_start = false;
                }
                ctx.emit_start_element(kind.clone());
            }
            CustomPlacement::Inline => {
                if ctx.pending_para_start {
                    ctx.emit_start(NodeKind::Paragraph);
                    ctx.pending_para_start = false;
                }
                // Mark as inline-only context to prevent block-level parsing inside it
                ctx.emit_start_inline_custom(kind.clone());
                // An inline node continues the current line; without this the
                // node's own text would be mistaken for the start of a new line.
                ctx.at_line_start = false;
            }
        },
        _ => {
            if !is_inline_node(kind) {
                ctx.close_blockquotes();
                ctx.close_all_lists();
                ctx.close_table_if_open();
            } else {
                if ctx.pending_para_start {
                    ctx.emit_start(NodeKind::Paragraph);
                    ctx.pending_para_start = false;
                }
                // Inline nodes never begin a source line. Leaving the flag set
                // made the deferred line-begin handling wrap e.g. a link label
                // in a spurious `<p>` inside the `<a>`.
                ctx.at_line_start = false;
            }
            ctx.emit_start(kind.clone());
        }
    }
}
