// plugin-markdown/start.rs
use pendon_core::NodeKind;

use crate::context::ParseContext;

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

pub fn handle(ctx: &mut ParseContext, kind: &NodeKind, is_block_custom: bool) {
    match kind {
        NodeKind::Heading => {
            ctx.close_all_lists();
            ctx.close_table_if_open();
            ctx.emit_start(NodeKind::Heading);
            ctx.in_heading = true;
            ctx.heading_prefix_consumed = false;
            ctx.skip_para_open = ctx.skip_para_open.saturating_add(1);
            ctx.skip_para_close = ctx.skip_para_close.saturating_add(1);
        }
        NodeKind::CodeFence => {
            ctx.close_all_lists();
            ctx.close_table_if_open();
            ctx.emit_start(NodeKind::CodeFence);
            ctx.in_code_fence = true;
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
        // Tangani Custom secara eksplisit berdasarkan metadata __plugin_kind
        NodeKind::Custom(_) => {
            if is_block_custom {
                ctx.close_blockquotes();
                ctx.close_all_lists();
                ctx.close_table_if_open();
            } else if ctx.pending_para_start {
                ctx.emit_start(NodeKind::Paragraph);
                ctx.pending_para_start = false;
            }
            ctx.emit_start(kind.clone());
        }
        _ => {
            if !is_inline_node(kind) {
                ctx.close_blockquotes();
                ctx.close_all_lists();
                ctx.close_table_if_open();
            } else if ctx.pending_para_start {
                ctx.emit_start(NodeKind::Paragraph);
                ctx.pending_para_start = false;
            }
            ctx.emit_start(kind.clone());
        }
    }
}
