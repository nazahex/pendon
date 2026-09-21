use crate::attrs::{parse_attr_block, AttrSpec};
use pendon_core::{Event, NodeKind};

#[derive(Debug, Clone)]
pub struct TableBlock {
    pub start_index: usize,
    pub end_index: usize,
    pub caption: Option<CaptionSpec>,
    pub columns: Vec<ColumnSpec>,
    pub body_rows: Vec<RowSpec>,
    pub footer_rows: Vec<RowSpec>,
}

#[derive(Debug, Clone)]
pub struct CaptionSpec {
    pub text: String,
    pub attrs: AttrSpec,
}

#[derive(Debug, Clone)]
pub struct ColumnSpec {
    pub header_text: String,
    pub align: Align,
    pub width: Option<String>,
    pub attrs: AttrSpec,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Left,
    Center,
    Right,
    None,
}

#[derive(Debug, Clone)]
pub struct RowSpec {
    pub cells: Vec<CellSpec>,
    pub attrs: AttrSpec,
}

#[derive(Debug, Clone)]
pub struct CellSpec {
    pub text: String,
    pub attrs: AttrSpec,
    pub is_colspan_marker: bool,
    pub is_rowspan_marker: bool,
}

pub fn try_parse_table_block(events: &[Event], start_idx: usize) -> Option<TableBlock> {
    if !matches!(
        events.get(start_idx),
        Some(Event::StartNode(NodeKind::Paragraph))
    ) {
        return None;
    }
    let para_end = find_matching_end(events, start_idx, NodeKind::Paragraph)?;
    let raw_text = collect_text_only(&events[start_idx + 1..para_end])?;

    let lines: Vec<&str> = raw_text.lines().collect();
    if lines.is_empty() {
        return None;
    }

    let mut block = parse_table_from_lines(&lines)?;
    block.start_index = start_idx;
    block.end_index = para_end;
    Some(block)
}

fn parse_table_from_lines(lines: &[&str]) -> Option<TableBlock> {
    let mut caption = None;
    let mut start_idx = 0;

    if let Some(cap) = parse_caption_line(lines[0]) {
        caption = Some(cap);
        start_idx = 1;
    }

    if start_idx >= lines.len() {
        return None;
    }

    let header_line = lines.get(start_idx)?.trim();
    if !is_table_row(header_line) {
        return None;
    }
    let header_cells = split_table_cells(header_line);

    let delim_idx = start_idx + 1;
    let delim_line = lines.get(delim_idx)?.trim();
    if !is_table_row(delim_line) {
        return None;
    }
    let delim_cells = split_table_cells(delim_line);

    if header_cells.len() != delim_cells.len() {
        return None;
    }

    let columns = parse_delimiter_cells(&delim_cells, &header_cells)?;

    let mut body_rows = Vec::new();
    let mut footer_rows = Vec::new();
    let mut in_footer = false;

    for i in (delim_idx + 1)..lines.len() {
        let line = lines[i].trim();
        if line.is_empty() {
            break;
        }

        if line == "|===|" || line == "===" {
            in_footer = true;
            continue;
        }

        if !is_table_row(line) {
            return None;
        }

        let cells = split_table_cells(line);
        let (row_attrs, parsed_cells) = parse_body_cells(&cells, columns.len())?;

        let row = RowSpec {
            cells: parsed_cells,
            attrs: row_attrs,
        };

        if in_footer {
            footer_rows.push(row);
        } else {
            body_rows.push(row);
        }
    }

    Some(TableBlock {
        start_index: 0,
        end_index: 0,
        caption,
        columns,
        body_rows,
        footer_rows,
    })
}

fn parse_caption_line(line: &str) -> Option<CaptionSpec> {
    let line = line.trim();
    if !line.starts_with('[') {
        return None;
    }

    // Search for the matching closing bracket, taking into account nested brackets
    let mut depth = 0;
    let mut close_br = None;
    let chars: Vec<char> = line.chars().collect();

    for (i, &ch) in chars.iter().enumerate() {
        match ch {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    close_br = Some(i);
                    break;
                }
            }
            _ => {}
        }
    }

    let close_br = close_br?;
    let caption_text = &line[1..close_br];
    let rest = &line[close_br + 1..];

    let (attrs, remaining) = parse_attr_block(rest);
    if !remaining.trim().is_empty() {
        return None;
    }

    Some(CaptionSpec {
        text: caption_text.to_string(),
        attrs,
    })
}

fn parse_delimiter_cells(
    delim_cells: &[String],
    header_cells: &[String],
) -> Option<Vec<ColumnSpec>> {
    let mut columns = Vec::new();

    for (i, delim) in delim_cells.iter().enumerate() {
        let text = delim.trim();
        let header_text = header_cells.get(i).cloned().unwrap_or_default();

        let mut core = text;

        // 1. Check left alignment (prefix ':')
        let align_left = core.starts_with(':');
        if align_left {
            core = &core[1..];
        }

        // 2. Calculate the number of leading dashes
        let mut dash_end = 0;
        for (idx, ch) in core.char_indices() {
            if ch != '-' {
                break;
            }
            dash_end = idx + ch.len_utf8();
        }

        // 3. Sparate the remaining string after dash
        let mut rest = &core[dash_end..];

        // 4. Check right alignment (suffix ':')
        let align_right = rest.starts_with(':');
        if align_right {
            rest = &rest[1..];
        }

        let align = if align_left && align_right {
            Align::Center
        } else if align_right {
            Align::Right
        } else if align_left {
            Align::Left
        } else {
            Align::None
        };

        // 5. Parse width specifier '(' ... ')'
        let mut width = None;
        if rest.starts_with('(') {
            if let Some(close) = rest.find(')') {
                width = Some(rest[1..close].to_string());
                rest = &rest[close + 1..];
            }
        }

        // 6. Parse attribute block
        let (attrs, remaining) = parse_attr_block(rest);
        if !remaining.trim().is_empty() {
            return None;
        }

        columns.push(ColumnSpec {
            header_text,
            align,
            width,
            attrs,
        });
    }

    Some(columns)
}

fn parse_body_cells(cells: &[String], _expected_cols: usize) -> Option<(AttrSpec, Vec<CellSpec>)> {
    let mut parsed_cells = Vec::new();
    let mut row_attrs = AttrSpec::default();

    for (i, cell) in cells.iter().enumerate() {
        let mut text = cell.trim();

        if i == cells.len() - 1 {
            if text.starts_with('-') {
                text = text[1..].trim();
            }
            let (attrs, content) = parse_attr_block(text);
            if content.trim().is_empty()
                && (!attrs.classes.is_empty() || !attrs.extra.is_empty() || attrs.id.is_some())
            {
                row_attrs = attrs;
                continue;
            }
        }

        let parsed_cell = parse_cell_content(cell.trim())?;
        parsed_cells.push(parsed_cell);
    }

    Some((row_attrs, parsed_cells))
}

fn parse_cell_content(text: &str) -> Option<CellSpec> {
    let text = text.trim();

    if text == ">" {
        return Some(CellSpec {
            text: String::new(),
            attrs: AttrSpec::default(),
            is_colspan_marker: true,
            is_rowspan_marker: false,
        });
    }
    if text == "^" {
        return Some(CellSpec {
            text: String::new(),
            attrs: AttrSpec::default(),
            is_colspan_marker: false,
            is_rowspan_marker: true,
        });
    }

    let mut content = text.to_string();
    let mut attrs = AttrSpec::default();

    let mut attr_start = None;
    for i in (0..text.len()).rev() {
        if text.as_bytes()[i] == b'[' || text.as_bytes()[i] == b'{' {
            if i == 0 || text.as_bytes()[i - 1] == b' ' || text.as_bytes()[i - 1] == b'-' {
                attr_start = Some(i);
                break;
            }
        }
    }

    if let Some(start) = attr_start {
        let (parsed_attrs, rest) = parse_attr_block(&text[start..]);
        if rest.trim().is_empty() {
            attrs = parsed_attrs;
            let mut c = text[..start].to_string();
            if c.ends_with('-') || c.ends_with(' ') {
                c.pop();
            }
            content = c.trim().to_string();
        }
    }

    Some(CellSpec {
        text: content,
        attrs,
        is_colspan_marker: false,
        is_rowspan_marker: false,
    })
}

fn is_table_row(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return false;
    }
    if !(trimmed.starts_with('|') || trimmed.contains(" | ")) {
        return false;
    }
    split_table_cells(trimmed).len() >= 1
}

fn split_table_cells(line: &str) -> Vec<String> {
    let mut s = line.trim();
    if s.starts_with('|') {
        s = &s[1..];
    }
    if s.ends_with('|') {
        s = &s[..s.len() - 1];
    }

    let mut cells = Vec::new();
    let mut current = String::new();
    let mut wiki_depth = 0i32;
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Detect wiki link opening [[
        if i + 1 < len && chars[i] == '[' && chars[i + 1] == '[' {
            wiki_depth += 1;
            current.push('[');
            current.push('[');
            i += 2;
            continue;
        }

        // DDetect wiki link closing ]]
        if i + 1 < len && chars[i] == ']' && chars[i + 1] == ']' && wiki_depth > 0 {
            wiki_depth -= 1;
            current.push(']');
            current.push(']');
            i += 2;
            continue;
        }

        // Split on | only if not inside wiki link
        if chars[i] == '|' && wiki_depth == 0 {
            cells.push(current.clone());
            current.clear();
            i += 1;
            continue;
        }

        current.push(chars[i]);
        i += 1;
    }

    if !current.is_empty() || cells.is_empty() {
        cells.push(current);
    }

    cells
}

fn collect_text_only(events: &[Event]) -> Option<String> {
    let mut out = String::new();
    for ev in events {
        match ev {
            Event::Text(t) => out.push_str(t),
            _ => return None,
        }
    }
    Some(out)
}

fn find_matching_end(events: &[Event], start_idx: usize, kind: NodeKind) -> Option<usize> {
    let mut depth = 0isize;
    for (idx, ev) in events.iter().enumerate().skip(start_idx) {
        match ev {
            Event::StartNode(k) if *k == kind => depth += 1,
            Event::EndNode(k) if *k == kind => {
                depth -= 1;
                if depth == 0 {
                    return Some(idx);
                }
            }
            _ => {}
        }
    }
    None
}
