use pendon_core::{Event, NodeKind};

use crate::context::ParseContext;

pub fn handle(ctx: &mut ParseContext, kind: &NodeKind) {
    match kind {
        NodeKind::Heading => {
            ctx.emit_end(NodeKind::Heading);
            ctx.in_heading = false;
        }
        NodeKind::CodeFence => {
            ctx.emit_end(NodeKind::CodeFence);
            ctx.in_code_fence = false;
            ctx.skip_initial_code_newline = false;
            ctx.skip_backticks_once = true;
        }
        NodeKind::Document => {
            ctx.close_all_lists();
            ctx.close_table_if_open();
            if ctx.blockquote_depth > 0 {
                for _ in 0..ctx.blockquote_depth {
                    ctx.out.push(Event::EndNode(NodeKind::Blockquote));
                }
                ctx.blockquote_depth = 0;
            }
            ctx.emit_end(NodeKind::Document);
        }
        NodeKind::Paragraph => {
            if ctx.pending_para_start {
                ctx.pending_para_start = false;
            } else if ctx.skip_para_close > 0 {
                ctx.skip_para_close = ctx.skip_para_close.saturating_sub(1);
            } else {
                ctx.emit_end(NodeKind::Paragraph);
            }
            ctx.at_line_start = false;
        }
        _ => ctx.emit_end(kind.clone()),
    }
}
