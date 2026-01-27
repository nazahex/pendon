use pendon_core::{Event, NodeKind};

use crate::context::ParseContext;
use crate::helpers::{
    adjust_blockquote, capture_html_block, close_table, emit_html_event, emit_inline,
    emit_table_row, is_table_row, is_table_separator, parse_blockquote_prefix, split_table_cells,
    start_table,
};

pub fn handle(ctx: &mut ParseContext, s: &str) {
    if matches!(ctx.stack.last(), Some(NodeKind::ThematicBreak)) {
        return;
    }

    if s == "\n" {
        if ctx.in_code_fence {
            if ctx.skip_initial_code_newline {
                ctx.skip_initial_code_newline = false;
            } else {
                ctx.out.push(Event::Text("\n".to_string()));
            }
            ctx.at_line_start = true;
            return;
        }
        if ctx.in_heading {
            ctx.at_line_start = true;
            return;
        }
        ctx.at_line_start = true;
        return;
    }

    let mut line = s.to_string();
    let original_line = line.clone();

    if ctx.at_line_start && !ctx.in_heading && !ctx.in_code_fence {
        let (depth, tail) = parse_blockquote_prefix(&line);
        adjust_blockquote(&mut ctx.out, &mut ctx.blockquote_depth, depth);
        if depth > 0 {
            line = tail.to_string();
        } else {
            line = original_line;
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
                        emit_inline(tail, &mut ctx.out, ctx.options);
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
                emit_inline(tail, &mut ctx.out, ctx.options);
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
                    emit_inline(tail, &mut ctx.out, ctx.options);
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
        emit_inline(&line, &mut ctx.out, ctx.options);
    } else if ctx.in_code_fence {
        let trimmed = line.trim();
        if !trimmed.is_empty() && trimmed.chars().all(|c| c == '`') {
            return;
        }
        ctx.out.push(Event::Text(line.clone()));
    } else if ctx.in_list_item() {
        emit_inline(&line, &mut ctx.out, ctx.options);
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
        emit_inline(&line, &mut ctx.out, ctx.options);
    }
    ctx.at_line_start = false;
}
