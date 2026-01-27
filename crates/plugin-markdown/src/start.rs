use pendon_core::NodeKind;

use crate::context::ParseContext;

pub fn handle(ctx: &mut ParseContext, kind: &NodeKind) {
    match kind {
        NodeKind::Heading => {
            ctx.close_blockquotes();
            ctx.close_all_lists();
            ctx.emit_start(NodeKind::Heading);
            ctx.in_heading = true;
            ctx.heading_prefix_consumed = false;
            ctx.skip_para_open = ctx.skip_para_open.saturating_add(1);
            ctx.skip_para_close = ctx.skip_para_close.saturating_add(1);
        }
        NodeKind::CodeFence => {
            ctx.close_all_lists();
            ctx.emit_start(NodeKind::CodeFence);
            ctx.in_code_fence = true;
            ctx.skip_initial_code_newline = true;
            ctx.skip_para_open = ctx.skip_para_open.saturating_add(1);
            ctx.skip_para_close = ctx.skip_para_close.saturating_add(1);
        }
        NodeKind::Paragraph => {
            if ctx.skip_para_open > 0 {
                ctx.skip_para_open = ctx.skip_para_open.saturating_sub(1);
            } else {
                ctx.pending_para_start = true;
                ctx.at_line_start = true;
            }
        }
        _ => {
            ctx.close_blockquotes();
            ctx.close_all_lists();
            ctx.close_table_if_open();
            ctx.emit_start(kind.clone());
        }
    }
}
