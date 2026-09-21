// text.rs
use pendon_core::{Event, NodeKind};

use crate::context::ParseContext;
use crate::helpers::{
    adjust_blockquote, capture_html_block, close_table, emit_html_event, emit_inline,
    emit_table_row, is_table_row, is_table_separator, parse_blockquote_prefix, split_table_cells,
    start_table,
};
use crate::math::toggle_display_math_on_line;

fn emit_line_content(ctx: &mut ParseContext, line: &str) {
    if ctx.display_math_open {
        ctx.out.push(Event::Text(line.to_string()));
    } else {
        emit_inline(line, &mut ctx.out, ctx.options);
    }
    toggle_display_math_on_line(line, &mut ctx.display_math_open);
}

pub fn handle(ctx: &mut ParseContext, s: &str) {
    if matches!(
        ctx.stack.last(),
        Some(NodeKind::HtmlBlock | NodeKind::HtmlInline)
    ) {
        ctx.out.push(Event::Text(s.to_string()));
        ctx.at_line_start = s == "\n";
        return;
    }

    if matches!(ctx.stack.last(), Some(NodeKind::ThematicBreak)) {
        return;
    }

    if s == "\n" {
        if ctx.display_math_open {
            ctx.out.push(Event::Text("\n".to_string()));
            ctx.at_line_start = true;
            return;
        }
        let blank_line = ctx.at_line_start;

        if ctx.use_line_break() {
            ctx.at_line_start = true;
            return;
        }

        // CommonMark Soft Break
        if !blank_line && matches!(ctx.stack.last(), Some(NodeKind::Paragraph)) {
            ctx.out.push(Event::Text("\n".to_string()));
        }

        if ctx.in_code_fence {
            if ctx.skip_initial_code_newline {
                ctx.skip_initial_code_newline = false;
            } else {
                ctx.out.push(Event::Text("\n".to_string()));
            }
            ctx.previous_line_blank = false;
            ctx.at_line_start = true;
            return;
        }
        if ctx.in_heading {
            ctx.emit_end(NodeKind::Heading);
            ctx.in_heading = false;
            ctx.previous_line_blank = false;
            ctx.at_line_start = true;
            return;
        }
        if blank_line {
            ctx.close_all_lists();
            if matches!(ctx.stack.last(), Some(NodeKind::Paragraph)) {
                ctx.emit_end(NodeKind::Paragraph);
            }
            // close blockquote if the line is completely empty and we are at the start of a new line
            if ctx.blockquote_depth > 0 {
                adjust_blockquote(&mut ctx.out, &mut ctx.blockquote_depth, 0);
            }
        }
        ctx.previous_line_blank = true;
        ctx.at_line_start = true;
        return;
    }

    ctx.capture_line_text(s);
    let mut line = s.to_string();
    let original_line = line.clone();

    if ctx.at_line_start && !ctx.in_heading && !ctx.in_code_fence {
        let previous_line_blank = ctx.previous_line_blank;
        ctx.previous_line_blank = false;
        let (depth, tail) = parse_blockquote_prefix(&line);

        if depth > 0 && previous_line_blank && !ctx.list_frames.is_empty() {
            ctx.close_all_lists();
        }
        if depth != ctx.blockquote_depth && ctx.in_table {
            ctx.close_table_if_open();
        }

        // sctrict blockquote: only adjust blockquote depth if the detected depth is different from the current depth
        adjust_blockquote(&mut ctx.out, &mut ctx.blockquote_depth, depth);

        if depth > 0 {
            line = tail.to_string();
        } else {
            line = original_line;
        }

        // Empty line inside blockquote should close paragraph and lists, but not the blockquote itself
        if depth > 0 && line.trim().is_empty() {
            if matches!(ctx.stack.last(), Some(NodeKind::Paragraph)) {
                ctx.emit_end(NodeKind::Paragraph);
            }
            ctx.close_all_lists();
            if ctx.in_table {
                close_table(&mut ctx.out, &mut ctx.in_table);
            }
            ctx.pending_para_start = true;
            ctx.previous_line_blank = true;
            ctx.at_line_start = false;
            return;
        }

        if ctx.options.allow_html {
            if let Some(html_line) = capture_html_block(&line) {
                if matches!(ctx.stack.last(), Some(NodeKind::Paragraph)) {
                    ctx.emit_end(NodeKind::Paragraph);
                }
                emit_html_event(&mut ctx.out, &html_line, NodeKind::HtmlBlock);
                ctx.pending_para_start = false;
                ctx.at_line_start = false;
                return;
            }
        }
        let trimmed_for_table = line.trim_start();

        if ctx.in_table {
            if is_table_row(trimmed_for_table) {
                let cells = split_table_cells(trimmed_for_table);
                if !ctx.first_table_row && is_table_separator(&cells) {
                    ctx.at_line_start = false;
                    return;
                }
                emit_table_row(
                    trimmed_for_table,
                    ctx.first_table_row,
                    &mut ctx.out,
                    ctx.options,
                );
                ctx.first_table_row = false;
                ctx.at_line_start = false;
                return;
            } else {
                close_table(&mut ctx.out, &mut ctx.in_table);
                ctx.first_table_row = true;
            }
        }

        if is_table_row(trimmed_for_table) {
            let cells = split_table_cells(trimmed_for_table);
            if cells.is_empty() || !is_table_separator(&cells) {
                if ctx.pending_para_start {
                    ctx.pending_para_start = false;
                }
                if matches!(ctx.stack.last(), Some(NodeKind::Paragraph)) {
                    ctx.emit_end(NodeKind::Paragraph);
                }
                start_table(&mut ctx.out);
                emit_table_row(trimmed_for_table, true, &mut ctx.out, ctx.options);
                ctx.out.push(Event::EndNode(NodeKind::TableHead));
                ctx.out.push(Event::StartNode(NodeKind::TableBody));
                ctx.in_table = true;
                ctx.first_table_row = false;
                ctx.at_line_start = false;
                return;
            }
        }
    } else if ctx.in_table {
        close_table(&mut ctx.out, &mut ctx.in_table);
        ctx.first_table_row = true;
    }

    if ctx.at_line_start && !ctx.in_heading && !ctx.in_code_fence {
        let indent = line.chars().take_while(|c| *c == ' ').count();
        let line = &line[indent..];
        let trimmed_for_block = line.trim_start();

        // Detect heading atx-style (e.g., # Heading 1)
        if line.starts_with('#') {
            let hashes = line.chars().take_while(|&c| c == '#').count();
            if hashes >= 1 && hashes <= 6 {
                let after_hashes = line.chars().nth(hashes);
                if after_hashes == Some(' ') || after_hashes.is_none() {
                    if matches!(ctx.stack.last(), Some(NodeKind::Paragraph)) {
                        ctx.emit_end(NodeKind::Paragraph);
                    }
                    ctx.emit_start(NodeKind::Heading);
                    ctx.in_heading = true;
                    ctx.heading_prefix_consumed = true;
                    ctx.skip_para_open = ctx.skip_para_open.saturating_add(1);
                    ctx.skip_para_close = ctx.skip_para_close.saturating_add(1);

                    ctx.out.push(Event::Attribute {
                        name: "level".to_string(),
                        value: hashes.to_string(),
                    });

                    let rest = if after_hashes == Some(' ') {
                        &line[hashes + 1..]
                    } else {
                        ""
                    };
                    if !rest.is_empty() {
                        emit_line_content(ctx, rest);
                    }
                    ctx.at_line_start = false;
                    return;
                }
            }
        }

        // Detect code fence in blockquote
        if line.starts_with("```") {
            let fence_count = line.chars().take_while(|&c| c == '`').count();
            if fence_count >= 3 {
                if matches!(ctx.stack.last(), Some(NodeKind::Paragraph)) {
                    ctx.emit_end(NodeKind::Paragraph);
                }
                ctx.emit_start(NodeKind::CodeFence);
                ctx.in_code_fence = true;
                ctx.skip_initial_code_newline = true;
                ctx.skip_para_open = ctx.skip_para_open.saturating_add(1);
                ctx.skip_para_close = ctx.skip_para_close.saturating_add(1);

                let info = line[fence_count..].trim();
                if !info.is_empty() {
                    ctx.out.push(Event::Attribute {
                        name: "lang".to_string(),
                        value: info.to_string(),
                    });
                }
                ctx.at_line_start = false;
                return;
            }
        }

        // Detect ThematicBreak in blockquote
        if trimmed_for_block.len() >= 3 && trimmed_for_block.chars().all(|c| c == '-') {
            if matches!(ctx.stack.last(), Some(NodeKind::Paragraph)) {
                ctx.emit_end(NodeKind::Paragraph);
            }
            ctx.emit_start(NodeKind::ThematicBreak);
            ctx.out.push(Event::Text(line.to_string()));
            ctx.emit_end(NodeKind::ThematicBreak);
            ctx.at_line_start = false;
            return;
        }

        // Ordered list detection
        let mut chars = line.chars();
        let mut num_str = String::new();
        while let Some(c) = chars.next() {
            if c.is_ascii_digit() {
                num_str.push(c);
            } else {
                break;
            }
        }
        if !num_str.is_empty() {
            let consumed = num_str.len();
            if let Some(delim) = line.chars().nth(consumed) {
                if (delim == '.' || delim == ')') && line.chars().nth(consumed + 1) == Some(' ') {
                    ctx.close_lists_above(indent);
                    if matches!(ctx.stack.last(), Some(NodeKind::Paragraph)) {
                        ctx.emit_end(NodeKind::Paragraph);
                    }
                    let start_num = num_str.parse::<usize>().ok();
                    let start_attr = if let Some(n) = start_num {
                        if !ctx.current_list_start_emitted() {
                            Some(n)
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    ctx.ensure_list(NodeKind::OrderedList, indent, start_attr);
                    if start_attr.is_some() {
                        ctx.mark_current_list_start_emitted();
                    }
                    ctx.start_list_item();
                    ctx.pending_para_start = false;
                    let tail = &line[(consumed + 2)..];
                    if !tail.is_empty() {
                        emit_line_content(ctx, tail);
                    }
                    ctx.at_line_start = false;
                    return;
                }
            }
        }

        // Bullet list detection
        if line.starts_with("- ") || line.starts_with("* ") || line.starts_with("+ ") {
            ctx.close_lists_above(indent);
            if matches!(ctx.stack.last(), Some(NodeKind::Paragraph)) {
                ctx.emit_end(NodeKind::Paragraph);
            }
            ctx.ensure_list(NodeKind::BulletList, indent, None);
            ctx.start_list_item();
            ctx.pending_para_start = false;
            let tail = &line[2..];
            if !tail.is_empty() {
                emit_line_content(ctx, tail);
            }
            ctx.at_line_start = false;
            return;
        }

        // Continuation line inside current list
        if ctx.list_frames.last().is_some() {
            ctx.close_lists_above(indent);
            if ctx.list_frames.last().is_some() {
                ctx.start_list_item();
                if matches!(ctx.stack.last(), Some(NodeKind::Paragraph)) {
                    ctx.emit_end(NodeKind::Paragraph);
                }
                let tail = line;
                if !tail.is_empty() {
                    emit_line_content(ctx, tail);
                }
                ctx.at_line_start = false;
                return;
            }
        }

        if ctx.pending_para_start {
            ctx.emit_start(NodeKind::Paragraph);
            ctx.pending_para_start = false;
        }
    }

    if ctx.in_heading {
        if !ctx.heading_prefix_consumed {
            if line.chars().all(|c| c == '#') {
                let level = line.chars().count();
                ctx.out.push(Event::Attribute {
                    name: "level".to_string(),
                    value: level.to_string(),
                });
                return;
            }
            if line == " " {
                return;
            }
            ctx.heading_prefix_consumed = true;
        }
        emit_line_content(ctx, &line);
    } else if ctx.in_code_fence {
        let trimmed = line.trim();
        if !trimmed.is_empty() && trimmed.chars().all(|c| c == '`') && trimmed.len() >= 3 {
            ctx.emit_end(NodeKind::CodeFence);
            ctx.in_code_fence = false;
            ctx.skip_initial_code_newline = false;
            ctx.skip_backticks_once = true;
            ctx.at_line_start = false;
            return;
        }
        ctx.out.push(Event::Text(line.clone()));
    } else if ctx.in_list_item() {
        emit_line_content(ctx, &line);
    } else {
        if ctx.skip_backticks_once {
            let trimmed = line.trim();
            if !trimmed.is_empty() && trimmed.chars().all(|c| c == '`') {
                ctx.skip_backticks_once = false;
                return;
            }
            ctx.skip_backticks_once = false;
        }
        let trimmed = line.trim();
        if trimmed.len() >= 3 && trimmed.chars().all(|c| c == '`') {
            return;
        }
        if ctx.pending_para_start {
            ctx.emit_start(NodeKind::Paragraph);
            ctx.pending_para_start = false;
        }
        emit_line_content(ctx, &line);
    }
    ctx.at_line_start = false;
}
