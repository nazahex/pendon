// text.rs
use pendon_core::{Event, NodeKind};

use crate::context::{ListFrame, ParseContext};
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

    // Baris yang hanya berisi spasi dianggap sebagai baris kosong (blank line).
    if s != "\n" && s.trim().is_empty() && !ctx.in_code_fence && !ctx.display_math_open {
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
            if matches!(ctx.stack.last(), Some(NodeKind::Paragraph)) {
                ctx.emit_end(NodeKind::Paragraph);
            }
            if ctx.blockquote_depth > 0 {
                // PENTING: Baris kosong di luar blockquote akan mengakhiri blockquote.
                // Kita HARUS menutup semua list yang ada di dalam blockquote tersebut terlebih dahulu.
                while let Some(frame) = ctx.list_frames.last() {
                    if frame.blockquote_depth > 0 {
                        let popped = ctx.list_frames.pop().unwrap();
                        if popped.item_open {
                            ctx.out.push(Event::EndNode(NodeKind::ListItem));
                        }
                        ctx.out.push(Event::EndNode(popped.kind));
                    } else {
                        break;
                    }
                }
                adjust_blockquote(&mut ctx.out, &mut ctx.blockquote_depth, 0);
            }
            ctx.previous_line_blank = true;
        } else {
            ctx.previous_line_blank = false;
        }
        ctx.at_line_start = true;
        return;
    }

    ctx.capture_line_text(s);
    let line = s.to_string();
    let original_line = line.clone();

    if ctx.at_line_start && !ctx.in_heading && !ctx.in_code_fence {
        ctx.previous_line_blank = false;

        // PENTING: Hitung spasi SEBELUM blockquote prefix untuk indentasi list yang akurat
        let spaces_before_quote = line.chars().take_while(|c| *c == ' ').count();
        let (depth, tail) = parse_blockquote_prefix(&line);
        let current_line = if depth > 0 {
            tail.to_string()
        } else {
            original_line
        };

        let spaces_after_quote = current_line.chars().take_while(|c| *c == ' ').count();
        let stripped_for_marker = &current_line[spaces_after_quote..];

        // Total indentasi = spasi sebelum quote + spasi setelah quote
        let leading_spaces = spaces_before_quote + spaces_after_quote;

        let mut is_list_marker = false;
        let mut marker_width = 0;
        let mut list_kind = NodeKind::Paragraph;
        let mut start_attr = None;

        let mut chars = stripped_for_marker.chars();
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
            if let Some(delim) = stripped_for_marker.chars().nth(consumed) {
                if (delim == '.' || delim == ')')
                    && stripped_for_marker.chars().nth(consumed + 1) == Some(' ')
                {
                    is_list_marker = true;
                    marker_width = consumed + 2;
                    list_kind = NodeKind::OrderedList;
                    start_attr = num_str.parse::<usize>().ok();
                }
            }
        }

        if !is_list_marker {
            if stripped_for_marker.starts_with("- ")
                || stripped_for_marker.starts_with("* ")
                || stripped_for_marker.starts_with("+ ")
            {
                is_list_marker = true;
                marker_width = 2;
                list_kind = NodeKind::BulletList;
            }
        }

        let content_indent = leading_spaces + marker_width;

        // ATURAN EMAS: Tutup list yang tidak bisa menampung baris ini SECARA EKSPLISIT.
        // Ini HARUS dilakukan sebelum kita membuka elemen blok lain seperti blockquote.
        while let Some(frame) = ctx.list_frames.last() {
            let can_contain = if is_list_marker {
                leading_spaces == frame.indent || leading_spaces >= frame.content_indent
            } else {
                leading_spaces >= frame.content_indent
            };

            if !can_contain {
                let popped = ctx.list_frames.pop().unwrap();
                if popped.item_open {
                    ctx.out.push(Event::EndNode(NodeKind::ListItem));
                }
                ctx.out.push(Event::EndNode(popped.kind));
            } else {
                break;
            }
        }

        // SEKARANG baru kita sesuaikan blockquote depth
        if depth != ctx.blockquote_depth && ctx.in_table {
            ctx.close_table_if_open();
        }
        while let Some(frame) = ctx.list_frames.last() {
            if frame.blockquote_depth > depth {
                let popped = ctx.list_frames.pop().unwrap();
                if popped.item_open {
                    ctx.out.push(Event::EndNode(NodeKind::ListItem));
                }
                ctx.out.push(Event::EndNode(popped.kind));
            } else {
                break;
            }
        }

        adjust_blockquote(&mut ctx.out, &mut ctx.blockquote_depth, depth);

        if depth > 0 && current_line.trim().is_empty() {
            if matches!(ctx.stack.last(), Some(NodeKind::Paragraph)) {
                ctx.emit_end(NodeKind::Paragraph);
            }
            if ctx.in_table {
                close_table(&mut ctx.out, &mut ctx.in_table);
            }
            ctx.pending_para_start = true;
            ctx.previous_line_blank = true;
            ctx.at_line_start = false;
            return;
        }

        if is_list_marker {
            if ctx.in_table {
                close_table(&mut ctx.out, &mut ctx.in_table);
            }

            if matches!(ctx.stack.last(), Some(NodeKind::Paragraph)) {
                ctx.emit_end(NodeKind::Paragraph);
            }

            let mut need_new_list = true;
            if let Some(frame) = ctx.list_frames.last() {
                if leading_spaces == frame.indent {
                    if frame.kind == list_kind {
                        if frame.item_open {
                            ctx.out.push(Event::EndNode(NodeKind::ListItem));
                            if let Some(f) = ctx.list_frames.last_mut() {
                                f.item_open = false;
                            }
                        }
                        need_new_list = false;
                    } else {
                        let popped = ctx.list_frames.pop().unwrap();
                        if popped.item_open {
                            ctx.out.push(Event::EndNode(NodeKind::ListItem));
                        }
                        ctx.out.push(Event::EndNode(popped.kind));
                    }
                }
            }

            if need_new_list {
                ctx.out.push(Event::StartNode(list_kind.clone()));
                ctx.stack.push(list_kind.clone());
                if let (NodeKind::OrderedList, Some(n)) = (&list_kind, start_attr) {
                    ctx.out.push(Event::Attribute {
                        name: "start".to_string(),
                        value: n.to_string(),
                    });
                }
                ctx.list_frames.push(ListFrame {
                    kind: list_kind.clone(),
                    indent: leading_spaces,
                    content_indent,
                    item_open: false,
                    blockquote_depth: ctx.blockquote_depth,
                });
            }

            ctx.out.push(Event::StartNode(NodeKind::ListItem));
            ctx.stack.push(NodeKind::ListItem);
            if let Some(f) = ctx.list_frames.last_mut() {
                f.item_open = true;
            }

            ctx.pending_para_start = false;
            let tail_marker = &stripped_for_marker[marker_width..];
            if !tail_marker.is_empty() {
                emit_line_content(ctx, tail_marker);
            }
            ctx.at_line_start = false;
            return;
        }

        if ctx.options.allow_html {
            if let Some(html_line) = capture_html_block(&current_line) {
                if matches!(ctx.stack.last(), Some(NodeKind::Paragraph)) {
                    ctx.emit_end(NodeKind::Paragraph);
                }
                emit_html_event(&mut ctx.out, &html_line, NodeKind::HtmlBlock);
                ctx.pending_para_start = false;
                ctx.at_line_start = false;
                return;
            }
        }

        let trimmed_for_table = current_line.trim_start();

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

        let trimmed_for_block = stripped_for_marker;

        if trimmed_for_block.starts_with('#') {
            let hashes = trimmed_for_block.chars().take_while(|&c| c == '#').count();
            if hashes >= 1 && hashes <= 6 {
                let after_hashes = trimmed_for_block.chars().nth(hashes);
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
                        &trimmed_for_block[hashes + 1..]
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

        if trimmed_for_block.starts_with("```") {
            let fence_count = trimmed_for_block.chars().take_while(|&c| c == '`').count();
            if fence_count >= 3 {
                if matches!(ctx.stack.last(), Some(NodeKind::Paragraph)) {
                    ctx.emit_end(NodeKind::Paragraph);
                }
                ctx.emit_start(NodeKind::CodeFence);
                ctx.in_code_fence = true;
                ctx.skip_initial_code_newline = true;
                ctx.skip_para_open = ctx.skip_para_open.saturating_add(1);
                ctx.skip_para_close = ctx.skip_para_close.saturating_add(1);

                let info = trimmed_for_block[fence_count..].trim();
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

        if trimmed_for_block.len() >= 3 && trimmed_for_block.chars().all(|c| c == '-') {
            if matches!(ctx.stack.last(), Some(NodeKind::Paragraph)) {
                ctx.emit_end(NodeKind::Paragraph);
            }
            ctx.emit_start(NodeKind::ThematicBreak);
            ctx.out.push(Event::Text(trimmed_for_block.to_string()));
            ctx.emit_end(NodeKind::ThematicBreak);
            ctx.at_line_start = false;
            return;
        }

        if ctx.pending_para_start {
            if !matches!(ctx.stack.last(), Some(NodeKind::Paragraph)) {
                ctx.emit_start(NodeKind::Paragraph);
            }
            ctx.pending_para_start = false;
        } else if !matches!(ctx.stack.last(), Some(NodeKind::Paragraph)) {
            ctx.emit_start(NodeKind::Paragraph);
        }
        emit_line_content(ctx, trimmed_for_block);

        ctx.at_line_start = false;
        return;
    } else if ctx.in_table {
        close_table(&mut ctx.out, &mut ctx.in_table);
        ctx.first_table_row = true;
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
