// plugin-markdown/end.rs
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
            } else if matches!(ctx.stack.last(), Some(NodeKind::Paragraph)) {
                ctx.emit_end(NodeKind::Paragraph);
            }
            ctx.at_line_start = false;
        }
        _ => {
            // Auto-close node apapun yang masih menggantung di dalam stack
            while let Some(top) = ctx.stack.last() {
                if top == kind {
                    break;
                }
                let top_clone = top.clone();
                match top_clone {
                    NodeKind::Paragraph => {
                        ctx.emit_end(NodeKind::Paragraph);
                    }
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
                    NodeKind::BulletList | NodeKind::OrderedList => {
                        if let Some(frame) = ctx.list_frames.last() {
                            if frame.item_open {
                                ctx.emit_end(NodeKind::ListItem);
                            }
                        }
                        ctx.emit_end(top_clone);
                        ctx.list_frames.pop();
                    }
                    NodeKind::ListItem => {
                        ctx.emit_end(NodeKind::ListItem);
                        if let Some(f) = ctx.list_frames.last_mut() {
                            f.item_open = false;
                        }
                    }
                    NodeKind::Blockquote => {
                        ctx.emit_end(NodeKind::Blockquote);
                        if ctx.blockquote_depth > 0 {
                            ctx.blockquote_depth -= 1;
                        }
                    }
                    _ => {
                        ctx.emit_end(top_clone);
                    }
                }
            }
            ctx.emit_end(kind.clone());
        }
    }
}
