use pendon_core::{Event, NodeKind};

mod helpers;

use helpers::{
    adjust_blockquote, close_table, emit_inline, emit_table_row, is_table_row, parse_blockquote_prefix,
    is_table_separator, start_table,
};

// Markdown plugin: normalize heading/code fence/thematic break blocks.
// - Heading: removes leading '#' run and optional single space; drops newline within heading;
//   skips paragraph wrapper around heading line.
// - CodeFence: removes opening/closing backtick lines; preserves inner code content;
//   skips paragraph wrapper around fence lines.
// - ThematicBreak: keeps nodes, drops hyphen text line.
pub fn process(events: &[Event]) -> Vec<Event> {
    let mut out: Vec<Event> = Vec::with_capacity(events.len());
    let mut stack: Vec<NodeKind> = Vec::new();
    let mut in_heading = false;
    let mut heading_prefix_consumed = false;
    let mut in_code_fence = false;
    let mut skip_initial_code_newline = false;
    let mut skip_backticks_once = false;
    let mut skip_para_open: usize = 0;
    let mut skip_para_close: usize = 0;
    let mut list_stack: Vec<(NodeKind, usize)> = Vec::new(); // (kind, indent)
    let mut in_list_item = false;
    let mut at_line_start = false;
    let mut ordered_start: Option<usize> = None;
    let mut pending_para_start = false;
    let mut blockquote_depth: usize = 0;
    let mut in_table = false;
    let mut first_table_row = true;

    for ev in events {
        match ev {
            Event::StartNode(kind) => match kind {
                NodeKind::Heading => {
                    // Close any open list before starting a heading
                    if in_list_item {
                        out.push(Event::EndNode(NodeKind::ListItem));
                        in_list_item = false;
                    }
                    while let Some((k, _)) = list_stack.pop() {
                        out.push(Event::EndNode(k));
                    }
                    out.push(Event::StartNode(NodeKind::Heading));
                    stack.push(NodeKind::Heading);
                    in_heading = true;
                    heading_prefix_consumed = false;
                    skip_para_open = skip_para_open.saturating_add(1);
                    skip_para_close = skip_para_close.saturating_add(1);
                }
                NodeKind::CodeFence => {
                    // Close any open list before starting a code fence
                    if in_list_item {
                        out.push(Event::EndNode(NodeKind::ListItem));
                        in_list_item = false;
                    }
                    while let Some((k, _)) = list_stack.pop() {
                        out.push(Event::EndNode(k));
                    }
                    out.push(Event::StartNode(NodeKind::CodeFence));
                    stack.push(NodeKind::CodeFence);
                    in_code_fence = true;
                    skip_initial_code_newline = true;
                    skip_para_open = skip_para_open.saturating_add(1);
                    skip_para_close = skip_para_close.saturating_add(1);
                }
                NodeKind::Paragraph => {
                    if skip_para_open > 0 {
                        skip_para_open = skip_para_open.saturating_sub(1);
                        // skip paragraph wrapper
                    } else {
                        // defer emitting paragraph start until we know if it's a list line
                        pending_para_start = true;
                        at_line_start = true;
                    }
                }
                _ => {
                    // For other block-level nodes (e.g., ThematicBreak), close any open list first
                    if in_list_item {
                        out.push(Event::EndNode(NodeKind::ListItem));
                        in_list_item = false;
                    }
                    while let Some((k, _)) = list_stack.pop() {
                        out.push(Event::EndNode(k));
                    }
                    if in_table {
                        close_table(&mut out, &mut in_table);
                        first_table_row = true;
                    }
                    out.push(Event::StartNode(kind.clone()));
                    stack.push(kind.clone());
                }
            },
            Event::EndNode(kind) => match kind {
                NodeKind::Heading => {
                    out.push(Event::EndNode(NodeKind::Heading));
                    in_heading = false;
                    let _ = stack.pop();
                }
                NodeKind::CodeFence => {
                    out.push(Event::EndNode(NodeKind::CodeFence));
                    in_code_fence = false;
                    skip_initial_code_newline = false;
                    skip_backticks_once = true;
                    let _ = stack.pop();
                }
                NodeKind::Document => {
                    // Ensure lists are closed before the document ends
                    if in_list_item {
                        out.push(Event::EndNode(NodeKind::ListItem));
                        in_list_item = false;
                    }
                    while let Some((k, _)) = list_stack.pop() {
                        out.push(Event::EndNode(k));
                    }
                    if in_table {
                        out.push(Event::EndNode(NodeKind::TableBody));
                        out.push(Event::EndNode(NodeKind::Table));
                        in_table = false;
                        first_table_row = true;
                    }
                    if blockquote_depth > 0 {
                        for _ in 0..blockquote_depth {
                            out.push(Event::EndNode(NodeKind::Blockquote));
                        }
                        blockquote_depth = 0;
                    }
                    out.push(Event::EndNode(NodeKind::Document));
                    let _ = stack.pop();
                }
                NodeKind::Paragraph => {
                    if pending_para_start {
                        // matching deferred start; skip entirely
                        pending_para_start = false;
                    } else if skip_para_close > 0 {
                        skip_para_close = skip_para_close.saturating_sub(1);
                    } else {
                        out.push(Event::EndNode(NodeKind::Paragraph));
                        let _ = stack.pop();
                    }
                    at_line_start = false;
                }
                _ => {
                    out.push(Event::EndNode(kind.clone()));
                    let _ = stack.pop();
                }
            },
            Event::Text(s) => {
                if matches!(stack.last(), Some(NodeKind::ThematicBreak)) {
                    continue;
                }
                // newline handling
                if s == "\n" {
                    // Preserve newline within code fences; drop elsewhere
                    if in_code_fence {
                        if skip_initial_code_newline {
                            // skip the newline immediately after opening fence
                            skip_initial_code_newline = false;
                        } else {
                            out.push(Event::Text("\n".to_string()));
                        }
                        // stay at line start for subsequent processing
                        at_line_start = true;
                        continue;
                    }
                    // drop newline within heading and lists; treat as soft wrap
                    if in_heading {
                        at_line_start = true;
                        continue;
                    }
                    at_line_start = true;
                    continue;
                }

                let mut line = s.clone();

                if at_line_start && !in_heading && !in_code_fence {
                    let (depth, tail) = parse_blockquote_prefix(&line);
                    adjust_blockquote(&mut out, &mut blockquote_depth, depth);
                    line = tail.to_string();
                    let trimmed_for_table = line.trim_start();

                    if in_table {
                        if is_table_row(trimmed_for_table) {
                            let cells = helpers::split_table_cells(trimmed_for_table);
                            if !first_table_row && is_table_separator(&cells) {
                                at_line_start = false;
                                continue;
                            }
                            emit_table_row(trimmed_for_table, first_table_row, &mut out);
                            first_table_row = false;
                            at_line_start = false;
                            continue;
                        } else {
                            close_table(&mut out, &mut in_table);
                            first_table_row = true;
                        }
                    }

                    if is_table_row(trimmed_for_table) {
                        let cells = helpers::split_table_cells(trimmed_for_table);
                        if !cells.is_empty() && is_table_separator(&cells) {
                            // do not start a table on a separator row alone
                        } else {
                            if pending_para_start {
                                pending_para_start = false;
                            }
                            if matches!(stack.last(), Some(NodeKind::Paragraph)) {
                                out.push(Event::EndNode(NodeKind::Paragraph));
                                let _ = stack.pop();
                            }
                            start_table(&mut out);
                            emit_table_row(trimmed_for_table, true, &mut out);
                            out.push(Event::EndNode(NodeKind::TableHead));
                            out.push(Event::StartNode(NodeKind::TableBody));
                            in_table = true;
                            first_table_row = false;
                            at_line_start = false;
                            continue;
                        }
                    }
                } else if in_table {
                    close_table(&mut out, &mut in_table);
                    first_table_row = true;
                }

                // list detection at line start
                if at_line_start && !in_heading && !in_code_fence {
                    // indentation count (spaces)
                    let indent = line.chars().take_while(|c| *c == ' ').count();
                    let line = &line[indent..];
                    // ordered: digits + ('.' or ')') + space
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
                            if (delim == '.' || delim == ')')
                                && line.chars().nth(consumed + 1) == Some(' ')
                            {
                                // close deeper lists if indent decreased
                                while let Some((_, d)) = list_stack.last() {
                                    if *d > indent {
                                        let (k, _) = list_stack.pop().unwrap();
                                        out.push(Event::EndNode(k));
                                    } else {
                                        break;
                                    }
                                }
                                // open nested if indent increased or type changed
                                if !matches!(list_stack.last(), Some((NodeKind::OrderedList, d)) if *d == indent)
                                {
                                    out.push(Event::StartNode(NodeKind::OrderedList));
                                    list_stack.push((NodeKind::OrderedList, indent));
                                    if let Ok(n) = num_str.parse::<usize>() {
                                        if ordered_start.is_none() {
                                            ordered_start = Some(n);
                                            out.push(Event::Attribute {
                                                name: "start".to_string(),
                                                value: n.to_string(),
                                            });
                                        }
                                    }
                                }
                                // if an item is currently open, close it before starting new
                                if in_list_item {
                                    out.push(Event::EndNode(NodeKind::ListItem));
                                }
                                // this is a list line; do not emit pending paragraph start
                                pending_para_start = false;
                                out.push(Event::StartNode(NodeKind::ListItem));
                                in_list_item = true;
                                let tail = &line[(consumed + 2)..];
                                if !tail.is_empty() {
                                    emit_inline(tail, &mut out);
                                }
                                at_line_start = false;
                                continue;
                            }
                        }
                    }
                    // bullet: '-', '*', '+' + space
                    if line.starts_with("- ") || line.starts_with("* ") || line.starts_with("+ ") {
                        // close deeper lists if indent decreased
                        while let Some((_, d)) = list_stack.last() {
                            if *d > indent {
                                let (k, _) = list_stack.pop().unwrap();
                                out.push(Event::EndNode(k));
                            } else {
                                break;
                            }
                        }
                        // open nested if indent increased or type changed
                        if !matches!(list_stack.last(), Some((NodeKind::BulletList, d)) if *d == indent)
                        {
                            out.push(Event::StartNode(NodeKind::BulletList));
                            list_stack.push((NodeKind::BulletList, indent));
                        }
                        // if an item is currently open, close it before starting new
                        if in_list_item {
                            out.push(Event::EndNode(NodeKind::ListItem));
                        }
                        // this is a list line; do not emit pending paragraph start
                        pending_para_start = false;
                        out.push(Event::StartNode(NodeKind::ListItem));
                        in_list_item = true;
                        let tail = &line[2..];
                        if !tail.is_empty() {
                            emit_inline(tail, &mut out);
                        }
                        at_line_start = false;
                        continue;
                    }
                    // non-list line at start: treat as continuation within current list
                    if let Some((_k, _d)) = list_stack.last() {
                        if !in_list_item {
                            out.push(Event::StartNode(NodeKind::ListItem));
                            in_list_item = true;
                        }
                        let tail = line;
                        if !tail.is_empty() {
                            emit_inline(tail, &mut out);
                        }
                        at_line_start = false;
                        continue;
                    }
                    // if we were deferring a paragraph start and it's not a list, emit it now
                    if pending_para_start {
                        out.push(Event::StartNode(NodeKind::Paragraph));
                        stack.push(NodeKind::Paragraph);
                        pending_para_start = false;
                    }
                }

                if in_heading {
                    if !heading_prefix_consumed {
                        if line.chars().all(|c| c == '#') {
                            // Capture heading level from leading '#' run
                            let level = line.chars().count();
                            out.push(Event::Attribute {
                                name: "level".to_string(),
                                value: level.to_string(),
                            });
                            continue;
                        }
                        if line == " " {
                            continue;
                        }
                        heading_prefix_consumed = true;
                    }
                    emit_inline(&line, &mut out);
                } else if in_code_fence {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() && trimmed.chars().all(|c| c == '`') {
                        continue;
                    }
                    out.push(Event::Text(line.clone()));
                } else if in_list_item {
                    emit_inline(&line, &mut out);
                } else {
                    if skip_backticks_once {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() && trimmed.chars().all(|c| c == '`') {
                            skip_backticks_once = false;
                            continue;
                        }
                        skip_backticks_once = false;
                    }
                    let trimmed = line.trim();
                    if trimmed.len() >= 3 && trimmed.chars().all(|c| c == '`') {
                        continue;
                    }
                    // if we were deferring a paragraph start, emit it before text
                    if pending_para_start {
                        out.push(Event::StartNode(NodeKind::Paragraph));
                        stack.push(NodeKind::Paragraph);
                        pending_para_start = false;
                    }
                    emit_inline(&line, &mut out);
                }
                at_line_start = false;
            }
            Event::Diagnostic { .. } => out.push(ev.clone()),
            Event::Attribute { .. } => out.push(ev.clone()),
        }
    }

    if in_list_item {
        out.push(Event::EndNode(NodeKind::ListItem));
    }
    while let Some((k, _)) = list_stack.pop() {
        out.push(Event::EndNode(k));
    }
    if in_table {
        out.push(Event::EndNode(NodeKind::TableBody));
        out.push(Event::EndNode(NodeKind::Table));
    }
    if blockquote_depth > 0 {
        for _ in 0..blockquote_depth {
            out.push(Event::EndNode(NodeKind::Blockquote));
        }
    }

    out
}
