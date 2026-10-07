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
            ctx.flush_block_decorator();
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
            ctx.flush_block_decorator();
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
        // Structured HTML element containers emitted by the img/table plugins
        // (their children are already rendered events) and `Element` nodes an
        // unclaimed directive/marker falls back to (§10.2/§10.3, D8). Both carry
        // their placement in `__plugin_kind` (§11 rule 3), so an `Element` is
        // re-lexed exactly like a `Custom` node when it declares `block` or
        // `inline`; only the undeclared/`element` case stays verbatim.
        NodeKind::Element(_) | NodeKind::Custom(_) => match placement {
            CustomPlacement::Block | CustomPlacement::ListContainer(_) => {
                ctx.close_blockquotes();
                ctx.close_all_lists();
                ctx.close_table_if_open();
                ctx.emit_start(kind.clone());
                // §9.3/§9.4: a list-container wrapper (the `unordered` and
                // `ordered` layers of `plugin-list`) is the container node of the
                // list below it — the list build adopts it instead of opening a
                // `<ul>`/`<ol>` of its own, so the wrapper's attributes land on
                // the container. Without a list in the body the wrapper stays an
                // ordinary block node.
                if let CustomPlacement::ListContainer(layer) = placement {
                    ctx.arm_list_container(layer, kind.clone());
                    // `plugin-list` consumed the paragraph that held the marker
                    // lines, so without this the first marker line would be
                    // emitted as inline text of the container.
                    ctx.at_line_start = true;
                }
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
