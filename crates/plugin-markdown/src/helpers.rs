use pendon_core::{Event, NodeKind};

pub fn parse_blockquote_prefix(line: &str) -> (usize, &str) {
    let bytes = line.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() && bytes[i] == b' ' {
        i += 1;
    }
    let mut depth = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'>' {
            depth += 1;
            i += 1;
            if i < bytes.len() && bytes[i] == b' ' {
                i += 1;
            }
        } else if bytes[i] == b' ' {
            i += 1;
        } else {
            break;
        }
    }
    (depth, line.get(i..).unwrap_or(""))
}

pub fn adjust_blockquote(out: &mut Vec<Event>, current: &mut usize, target: usize) {
    while *current > target {
        out.push(Event::EndNode(NodeKind::Blockquote));
        *current -= 1;
    }
    while *current < target {
        out.push(Event::StartNode(NodeKind::Blockquote));
        *current += 1;
    }
}

pub fn is_table_row(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return false;
    }
    if !(trimmed.starts_with('|') || trimmed.contains(" | ")) {
        return false;
    }
    let cells = split_table_cells(trimmed);
    cells.len() >= 2
}

pub fn start_table(out: &mut Vec<Event>) {
    out.push(Event::StartNode(NodeKind::Table));
    out.push(Event::StartNode(NodeKind::TableHead));
}

pub fn close_table(out: &mut Vec<Event>, in_table: &mut bool) {
    if *in_table {
        out.push(Event::EndNode(NodeKind::TableBody));
        out.push(Event::EndNode(NodeKind::Table));
        *in_table = false;
    }
}

pub fn emit_table_row(line: &str, is_header: bool, out: &mut Vec<Event>) {
    out.push(Event::StartNode(NodeKind::TableRow));
    for cell in split_table_cells(line) {
        out.push(Event::StartNode(NodeKind::TableCell));
        if is_header {
            out.push(Event::Attribute {
                name: "header".to_string(),
                value: "1".to_string(),
            });
        }
        if !cell.is_empty() {
            emit_inline(&cell, out);
        }
        out.push(Event::EndNode(NodeKind::TableCell));
    }
    out.push(Event::EndNode(NodeKind::TableRow));
}

pub fn is_table_separator(cells: &[String]) -> bool {
    cells.iter().all(|c| {
        let trimmed = c.trim_matches(|ch: char| ch == ':' || ch == '-');
        trimmed.is_empty() && !c.is_empty()
    })
}

pub fn emit_inline(s: &str, out: &mut Vec<Event>) {
    let bytes: Vec<char> = s.chars().collect();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == '`' {
            if let Some(end) = find_next(&bytes, i + 1, '`') {
                out.push(Event::StartNode(NodeKind::InlineCode));
                if end > i + 1 {
                    let content: String = bytes[i + 1..end].iter().collect();
                    out.push(Event::Text(content));
                }
                out.push(Event::EndNode(NodeKind::InlineCode));
                i = end + 1;
                continue;
            }
        }
        if bytes[i] == '[' {
            if let Some(close_br) = find_next(&bytes, i + 1, ']') {
                if close_br + 1 < bytes.len() && bytes[close_br + 1] == '(' {
                    if let Some(close_par) = find_next(&bytes, close_br + 2, ')') {
                        let text: String = bytes[i + 1..close_br].iter().collect();
                        let url: String = bytes[close_br + 2..close_par].iter().collect();
                        out.push(Event::StartNode(NodeKind::Link));
                        out.push(Event::Attribute {
                            name: "href".to_string(),
                            value: url,
                        });
                        emit_inline(&text, out);
                        out.push(Event::EndNode(NodeKind::Link));
                        i = close_par + 1;
                        continue;
                    }
                }
            }
        }
        if i + 1 < bytes.len() && bytes[i] == '*' && bytes[i + 1] == '*' {
            if let Some(end) = find_delim_pair(&bytes, i + 2, b'*', true) {
                out.push(Event::StartNode(NodeKind::Strong));
                let content: String = bytes[i + 2..end].iter().collect();
                emit_inline(&content, out);
                out.push(Event::EndNode(NodeKind::Strong));
                i = end + 2;
                continue;
            }
        }
        if i + 1 < bytes.len() && bytes[i] == '_' && bytes[i + 1] == '_' {
            if let Some(end) = find_delim_pair(&bytes, i + 2, b'_', true) {
                out.push(Event::StartNode(NodeKind::Bold));
                let content: String = bytes[i + 2..end].iter().collect();
                emit_inline(&content, out);
                out.push(Event::EndNode(NodeKind::Bold));
                i = end + 2;
                continue;
            }
        }
        if bytes[i] == '*' {
            if let Some(end) = find_next(&bytes, i + 1, '*') {
                out.push(Event::StartNode(NodeKind::Emphasis));
                let content: String = bytes[i + 1..end].iter().collect();
                emit_inline(&content, out);
                out.push(Event::EndNode(NodeKind::Emphasis));
                i = end + 1;
                continue;
            }
        }
        if bytes[i] == '_' {
            if let Some(end) = find_next(&bytes, i + 1, '_') {
                out.push(Event::StartNode(NodeKind::Italic));
                let content: String = bytes[i + 1..end].iter().collect();
                emit_inline(&content, out);
                out.push(Event::EndNode(NodeKind::Italic));
                i = end + 1;
                continue;
            }
        }
        out.push(Event::Text(bytes[i].to_string()));
        i += 1;
    }
}

fn find_next(hay: &[char], mut i: usize, ch: char) -> Option<usize> {
    while i < hay.len() {
        if hay[i] == ch {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn find_delim_pair(hay: &[char], start: usize, delim: u8, double: bool) -> Option<usize> {
    let d = delim as char;
    let mut i = start;
    while i + if double { 1 } else { 0 } < hay.len() {
        if hay[i] == d {
            if double {
                if i + 1 < hay.len() && hay[i + 1] == d {
                    return Some(i);
                }
            } else {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

pub fn split_table_cells(line: &str) -> Vec<String> {
    let mut s = line.trim();
    if s.starts_with('|') {
        s = s.trim_start_matches('|');
    }
    if s.ends_with('|') {
        s = s.trim_end_matches('|');
    }
    s.split('|').map(|c| c.trim().to_string()).collect()
}
