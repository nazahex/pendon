use crate::attrs::{extras_to_layer, LayerAttrs};
use pendon_core::{Event, NodeKind};
use pendon_extra::scan_extras_chars;

#[derive(Debug, Clone)]
pub struct TableBlock {
    pub start_index: usize,
    pub end_index: usize,
    /// §8 `decl` layer: the `<table>` attributes.
    pub attrs: LayerAttrs,
    pub caption: Option<CaptionSpec>,
    pub columns: Vec<ColumnSpec>,
    pub body_rows: Vec<RowSpec>,
    pub footer_rows: Vec<RowSpec>,
    /// §8: extras of the alignment row's end of line → the `<tbody>` (§11 layer
    /// `tbody`).
    pub tbody: LayerAttrs,
    /// §8: extras of the `|===|` line → the `<tfoot>` (§11 layer `tfoot`).
    pub tfoot: LayerAttrs,
}

#[derive(Debug, Clone)]
pub struct CaptionSpec {
    pub text: String,
    pub attrs: LayerAttrs,
}

#[derive(Debug, Clone)]
pub struct ColumnSpec {
    pub header_text: String,
    pub align: Align,
    pub width: Option<String>,
    pub attrs: LayerAttrs,
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
    pub attrs: LayerAttrs,
}

#[derive(Debug, Clone)]
pub struct CellSpec {
    pub text: String,
    pub attrs: LayerAttrs,
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
    let raw_text = collect_reconstructed_text(&events[start_idx + 1..para_end])?;

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
    let mut cursor = 0usize;

    // §8: the optional declaration line carries the `<table>` extras.
    let mut attrs = LayerAttrs::default();
    if let Some((decl_attrs, consumed)) = parse_decl_line(lines[cursor]) {
        attrs = decl_attrs;
        cursor += consumed;
    }

    let mut caption = None;
    if let Some((cap, consumed)) = parse_caption_line(lines.get(cursor)?) {
        caption = Some(cap);
        cursor += consumed;
    }

    if cursor >= lines.len() {
        return None;
    }

    let (header_line, _) = split_trailing_extras(lines.get(cursor)?.trim());
    let header_line = header_line.trim();
    if !is_table_row(header_line) {
        return None;
    }
    let header_cells = split_table_cells(header_line);

    let delim_idx = cursor + 1;
    // §8: the alignment row's end-of-line extras belong to the `<tbody>`.
    let (delim_line, tbody) = split_trailing_extras(lines.get(delim_idx)?.trim());
    let delim_line = delim_line.trim();
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
    let mut tfoot = LayerAttrs::default();

    for line in lines.iter().skip(delim_idx + 1) {
        let line = line.trim();
        if line.is_empty() {
            break;
        }

        // §8: only `|===|` opens the footer; the bare `===` separator is retired.
        let (foot_line, foot_extras) = split_trailing_extras(line);
        if foot_line.trim() == "|===|" {
            tfoot = foot_extras.unwrap_or_default();
            in_footer = true;
            continue;
        }

        let (row_line, row_extras) = split_trailing_extras(line);
        if !is_table_row(row_line.trim()) {
            return None;
        }

        let cells = split_table_cells(row_line.trim());
        let (mut row_attrs, parsed_cells) = parse_body_cells(&cells, columns.len())?;
        if let Some(extras) = row_extras {
            crate::attrs::merge_layer(&mut row_attrs, &extras);
        }

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
        attrs,
        caption,
        columns,
        body_rows,
        footer_rows,
        tbody: tbody.unwrap_or_default(),
        tfoot,
    })
}

/// Parses the caption line: the §8 `|| extras? content ||` form.
///
/// Returns the caption and how many lines it consumed (0 when the line is not a
/// caption). The retired pre-§8 `[content][.class,#id]{k:v}` form is literal
/// text (§14), so only `||…||` is a caption.
fn parse_caption_line(line: &str) -> Option<(CaptionSpec, usize)> {
    let trimmed = line.trim();
    let body = trimmed.strip_prefix("||")?;
    let body = body.strip_suffix("||").unwrap_or(body);
    // §8/D4: the extras head touches the opening `||`.
    let (attrs, _marker, content) = parse_front_extras(body);
    Some((
        CaptionSpec {
            text: content.trim().to_string(),
            attrs,
        },
        1,
    ))
}

/// Splits a leading extras head off a cell (§8/D4).
///
/// The head must touch the cell's opening `|`, so nothing is trimmed from the
/// front: `|{.x} text` and `|@@cellB{.lead} text` are heads, `| {.x} text` is
/// literal text. Returns the layer attributes, the marker and the remaining
/// content.
fn parse_front_extras(text: &str) -> (LayerAttrs, Option<String>, String) {
    let chars: Vec<char> = text.chars().collect();
    if let Some((head, next)) = scan_extras_chars(&chars, 0) {
        let bytes: usize = chars[..next].iter().map(|ch| ch.len_utf8()).sum();
        let layer = extras_to_layer(&head);
        let marker = layer.type_marker.clone();
        return (layer, marker, text[bytes..].to_string());
    }
    (LayerAttrs::default(), None, text.to_string())
}

/// Splits the trailing extras head off a table line (§8/D4).
///
/// The head must sit **after** the last `|` of the line and be the only thing
/// there (`| a | b |{.row}`), so cell content ending in braces is untouched.
/// Returns the line without the head and the parsed layer attributes.
fn split_trailing_extras(line: &str) -> (&str, Option<LayerAttrs>) {
    let trimmed = line.trim_end();
    let Some(pipe) = trimmed.rfind('|') else {
        return (line, None);
    };
    let tail = &trimmed[pipe + 1..];
    if tail.is_empty() {
        return (line, None);
    }

    match parse_layer_block(tail) {
        Some(attrs) => (&line[..pipe + 1], Some(attrs)),
        None => (line, None),
    }
}

/// Parses one layer's trailing attribute block: a `{…}` / `@@type{…}` head that
/// must touch the `|` before it. `None` when `input` is not exactly one head.
fn parse_layer_block(input: &str) -> Option<LayerAttrs> {
    let chars: Vec<char> = input.chars().collect();
    match scan_extras_chars(&chars, 0) {
        Some((head, next)) if chars[next..].iter().all(|ch| ch.is_whitespace()) => {
            Some(extras_to_layer(&head))
        }
        _ => None,
    }
}

/// Parses the §8 declaration line: `|-` `[slug]` `("title")` `extras?` `-|`.
///
/// Returns the `<table>` attributes and how many lines it consumed (0 when the
/// line is not a declaration). The declaration is childless: text that is not
/// part of a head or the extras is discarded. The retired `[.class]` bracket is
/// literal text (§14).
fn parse_decl_line(line: &str) -> Option<(LayerAttrs, usize)> {
    let body = line.trim().strip_prefix("|-")?;
    let mut rest = body;

    let mut layer = LayerAttrs::default();

    // §8: the head uses the standard `[slug]` / `("title")` form, adjacent to
    // the opening `|-`.
    if let Some(stripped) = rest.strip_prefix('[') {
        if let Some(close) = stripped.find(']') {
            let slug = stripped[..close].trim();
            if !slug.is_empty() && !slug.starts_with('.') && !slug.starts_with('#') {
                layer.attrs.id = Some(slug.to_string());
                rest = &stripped[close + 1..];
            }
        }
    }
    if let Some(stripped) = rest.strip_prefix('(') {
        if let Some(close) = stripped.find(')') {
            let title = stripped[..close]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if !title.is_empty() {
                layer
                    .attrs
                    .properties
                    .push(("title".to_string(), title.to_string()));
            }
            rest = &stripped[close + 1..];
        }
    }

    // §8/D4: the extras head touches the closing `-|`.
    let chars: Vec<char> = rest.chars().collect();
    if let Some((head, next)) = scan_extras_chars(&chars, 0) {
        let bytes: usize = chars[..next].iter().map(|ch| ch.len_utf8()).sum();
        crate::attrs::merge_layer(&mut layer, &extras_to_layer(&head));
        rest = &rest[bytes..];
    }

    // The trailing `-|` closes the declaration (a bare `|` is accepted too).
    match rest.trim() {
        "-|" | "|" | "" => Some((layer, 1)),
        _ => None,
    }
}

fn parse_delimiter_cells(
    delim_cells: &[String],
    header_cells: &[String],
) -> Option<Vec<ColumnSpec>> {
    let mut columns = Vec::new();

    for (i, delim) in delim_cells.iter().enumerate() {
        let header_text = header_cells.get(i).cloned().unwrap_or_default();

        // §8/D4: a delimiter cell is only alignment, an optional width and (after
        // them) the extras head; a head in front of the alignment code is text.
        let mut core = delim.trim();

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

        // 6. §8/D4: the extras head goes **after** the alignment/width code and
        // touches it (`:---(200px)@@cellA{.v-top}`). Anything left over is not
        // rendered, so it is ignored.
        let attrs = if rest.trim().is_empty() {
            LayerAttrs::default()
        } else {
            let (layer, _marker, _leftover) = parse_front_extras(rest);
            layer
        };

        columns.push(ColumnSpec {
            header_text,
            align,
            width,
            attrs,
        });
    }

    Some(columns)
}

/// Parses the cells of one body/footer row.
///
/// Row extras come from the trailing head after the last `|` (§8), never from a
/// cell, so the returned [`LayerAttrs`] is always empty here.
fn parse_body_cells(
    cells: &[String],
    _expected_cols: usize,
) -> Option<(LayerAttrs, Vec<CellSpec>)> {
    let mut parsed_cells = Vec::new();

    for cell in cells {
        parsed_cells.push(parse_cell_content(cell)?);
    }

    Some((LayerAttrs::default(), parsed_cells))
}

fn parse_cell_content(cell: &str) -> Option<CellSpec> {
    // §8/D4: the extras head touches the cell's opening `|`, so the raw cell is
    // passed in untrimmed and the head is scanned at the very front.
    let (front, _marker, rest) = parse_front_extras(cell);
    let rest = rest.trim();

    // The `>` (colspan) / `^` (rowspan) markers are still recognised behind an
    // extras head; their extras decorate the cell they merge into.
    if rest == ">" {
        return Some(CellSpec {
            text: String::new(),
            attrs: front,
            is_colspan_marker: true,
            is_rowspan_marker: false,
        });
    }
    if rest == "^" {
        return Some(CellSpec {
            text: String::new(),
            attrs: front,
            is_colspan_marker: false,
            is_rowspan_marker: true,
        });
    }

    Some(CellSpec {
        text: rest.to_string(),
        attrs: front,
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

/// Reconstructs the source text of a table paragraph.
///
/// Most table paragraphs are text-only, which [`collect_text_only`] covers. A
/// construct head can still be lexed as a `Link` node — the §8 declaration line
/// `|-[slug]("A longer title")@@type{…}-|` does exactly that — so links are
/// written back as `[label](href "title")` and the head is re-parsed from the
/// reconstructed text. Any other inline node means the paragraph is not a table.
fn collect_reconstructed_text(events: &[Event]) -> Option<String> {
    if let Some(text) = collect_text_only(events) {
        return Some(text);
    }

    let mut out = String::new();
    let mut link: Option<(String, Option<String>)> = None;
    for event in events {
        match event {
            Event::Text(text) => out.push_str(text),
            Event::StartNode(NodeKind::Link) if link.is_none() => {
                link = Some((String::new(), None));
                out.push('[');
            }
            Event::Attribute { name, value } => {
                if let Some((href, title)) = link.as_mut() {
                    match name.as_str() {
                        "href" => *href = value.clone(),
                        "title" => *title = Some(value.clone()),
                        _ => {}
                    }
                }
            }
            Event::EndNode(NodeKind::Link) => {
                let (href, title) = link.take()?;
                out.push(']');
                out.push('(');
                out.push_str(&href);
                if let Some(title) = title {
                    out.push(' ');
                    out.push('"');
                    out.push_str(&title);
                    out.push('"');
                }
                out.push(')');
            }
            // Anything else (raw HTML, inline code, emphasis) is not
            // reconstructible: the paragraph is not a table.
            _ => return None,
        }
    }

    link.is_none().then_some(out)
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

#[cfg(test)]
mod tests {
    use super::*;

    /// §8/D2: a bare `{…}` cell head is parsed; the retired `[.class]` form and
    /// a trailing block stay literal text (§14).
    #[test]
    fn bare_heads_are_parsed_and_retired_blocks_are_text() {
        let cell = parse_cell_content("{foo}").unwrap();
        assert_eq!(cell.text, "");
        assert_eq!(cell.attrs.flags, vec!["foo".to_string()]);

        let cell = parse_cell_content("{.text-red}").unwrap();
        assert_eq!(cell.text, "");
        assert_eq!(cell.attrs.attrs.classes, vec!["text-red".to_string()]);

        let cell = parse_cell_content("[.text-red]").unwrap();
        assert_eq!(cell.text, "[.text-red]");
        assert!(cell.attrs.attrs.id.is_none());
        assert!(cell.attrs.attrs.classes.is_empty());
        assert!(cell.attrs.attrs.properties.is_empty());

        // A trailing block is not a head: the head touches the `|`.
        let cell = parse_cell_content("Nilai { rox: \"rox\" }").unwrap();
        assert_eq!(cell.text, "Nilai { rox: \"rox\" }");
        assert!(cell.attrs.attrs.properties.is_empty());
    }
}
