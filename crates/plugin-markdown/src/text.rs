use pendon_core::{Event, NodeKind};

use crate::context::ParseContext;
use crate::helpers::{
    adjust_blockquote, close_table, emit_inline, emit_table_row, is_table_row, is_table_separator,
    parse_blockquote_prefix, split_table_cells, start_table,
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

    if ctx.at_line_start && !ctx.in_heading && !ctx.in_code_fence {
        let (depth, tail) = parse_blockquote_prefix(&line);
        adjust_blockquote(&mut ctx.out, &mut ctx.blockquote_depth, depth);
        line = tail.to_string();
        let trimmed_for_table = line.trim_start();

        if ctx.in_table {
            if is_table_row(trimmed_for_table) {
                let cells = split_table_cells(trimmed_for_table);
                if !ctx.first_table_row && is_table_separator(&cells) {
                    ctx.at_line_start = false;
                    return;
                }
                emit_table_row(trimmed_for_table, ctx.first_table_row, &mut ctx.out);
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
                emit_table_row(trimmed_for_table, true, &mut ctx.out);
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
        let mut chars = line.chars();
        let mut num_str = String::new();
        while let Some(c) = chars.next() {
            if c.is_ascii_digit() {
                num_str.push(c);
            } else {
                break;
            }
        }
        let consumed;
        if !num_str.is_empty() {
            consumed = num_str.len();
            if let Some(delim) = line.chars().nth(consumed) {
                if (delim == '.' || delim == ')') && line.chars().nth(consumed + 1) == Some(' ') {
                    while let Some((_, d)) = ctx.list_stack.last() {
                        if *d > indent {
                            let (k, _) = ctx.list_stack.pop().unwrap();
                            ctx.emit_end(k);
                        } else {
                            break;
                        }
                    }
                    if matches!(ctx.stack.last(), Some(NodeKind::Paragraph)) {
                        ctx.emit_end(NodeKind::Paragraph);
                    }
                    if !matches!(ctx.list_stack.last(), Some((NodeKind::OrderedList, d)) if *d == indent)
                    {
                        ctx.emit_start(NodeKind::OrderedList);
                        ctx.list_stack.push((NodeKind::OrderedList, indent));
                        if let Ok(n) = num_str.parse::<usize>() {
                            if ctx.ordered_start.is_none() {
                                ctx.ordered_start = Some(n);
                                ctx.out.push(Event::Attribute {
                                    name: "start".to_string(),
                                    value: n.to_string(),
                                });
                            }
                        }
                    }
                    if ctx.in_list_item {
                        ctx.emit_end(NodeKind::ListItem);
                    }
                    ctx.pending_para_start = false;
                    ctx.emit_start(NodeKind::ListItem);
                    ctx.in_list_item = true;
                    let tail = &line[(consumed + 2)..];
                    if !tail.is_empty() {
                        emit_inline(tail, &mut ctx.out);
                    }
                    ctx.at_line_start = false;
                    return;
                }
            }
        }
        if line.starts_with("- ") || line.starts_with("* ") || line.starts_with("+ ") {
            while let Some((_, d)) = ctx.list_stack.last() {
                if *d > indent {
                    let (k, _) = ctx.list_stack.pop().unwrap();
                    ctx.emit_end(k);
                } else {
                    break;
                }
            }
            if matches!(ctx.stack.last(), Some(NodeKind::Paragraph)) {
                ctx.emit_end(NodeKind::Paragraph);
            }
            if !matches!(ctx.list_stack.last(), Some((NodeKind::BulletList, d)) if *d == indent) {
                ctx.emit_start(NodeKind::BulletList);
                ctx.list_stack.push((NodeKind::BulletList, indent));
            }
            if ctx.in_list_item {
                ctx.emit_end(NodeKind::ListItem);
            }
            ctx.pending_para_start = false;
            ctx.emit_start(NodeKind::ListItem);
            ctx.in_list_item = true;
            let tail = &line[2..];
            if !tail.is_empty() {
                emit_inline(tail, &mut ctx.out);
            }
            ctx.at_line_start = false;
            return;
        }
        if let Some((_kind, _d)) = ctx.list_stack.last() {
            if !ctx.in_list_item {
                ctx.emit_start(NodeKind::ListItem);
                ctx.in_list_item = true;
            }
            if matches!(ctx.stack.last(), Some(NodeKind::Paragraph)) {
                ctx.emit_end(NodeKind::Paragraph);
            }
            let tail = line;
            if !tail.is_empty() {
                emit_inline(tail, &mut ctx.out);
            }
            ctx.at_line_start = false;
            return;
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
        emit_inline(&line, &mut ctx.out);
    } else if ctx.in_code_fence {
        let trimmed = line.trim();
        if !trimmed.is_empty() && trimmed.chars().all(|c| c == '`') {
            return;
        }
        ctx.out.push(Event::Text(line.clone()));
    } else if ctx.in_list_item {
        emit_inline(&line, &mut ctx.out);
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
        emit_inline(&line, &mut ctx.out);
    }
    ctx.at_line_start = false;
}
