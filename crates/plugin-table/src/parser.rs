use crate::attrs::{attr_block, extras_to_layer, LayerAttrs};
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
    /// `true` for the §8 `|| … ||` form (its extras belong to `<caption>`), and
    /// `false` for the pre-§8 `[…]` form (whose extras historically configure
    /// the `<table>` itself).
    pub new_form: bool,
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

/// Parses the caption line: the §8 `|| extras? content ||` form, or the
/// pre-§8 `[content][.class,#id]{key: value}` form.
///
/// Returns the caption and how many lines it consumed (0 when the line is not a
/// caption).
fn parse_caption_line(line: &str) -> Option<(CaptionSpec, usize)> {
    let trimmed = line.trim();

    // §8: `|| @@type{…} content ||`.
    if let Some(body) = trimmed.strip_prefix("||") {
        let body = body.strip_suffix("||").unwrap_or(body);
        let (attrs, _marker, content) = parse_front_extras(body);
        return Some((
            CaptionSpec {
                text: content.trim().to_string(),
                attrs,
                new_form: true,
            },
            1,
        ));
    }

    if !trimmed.starts_with('[') {
        return None;
    }

    // Search for the matching closing bracket, taking into account nested brackets
    let mut depth = 0;
    let mut close_br = None;
    let chars: Vec<char> = trimmed.chars().collect();

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
    let caption_text = &trimmed[1..close_br];
    let rest = &trimmed[close_br + 1..];

    // The line may carry a §11 extras head right after the legacy block.
    let mut layer = LayerAttrs::default();
    let rest = match scan_extras_chars(&rest.chars().collect::<Vec<char>>(), 0) {
        Some((head, next)) if rest.chars().count() >= next => {
            let bytes: usize = rest.chars().take(next).map(char::len_utf8).sum();
            crate::attrs::merge_layer(&mut layer, &extras_to_layer(&head));
            &rest[bytes..]
        }
        _ => rest,
    };
    let (attrs, remaining) = crate::attrs::parse_attr_block(rest);
    if !remaining.trim().is_empty() {
        return None;
    }
    layer.attrs = attrs;

    Some((
        CaptionSpec {
            text: caption_text.to_string(),
            attrs: layer,
            new_form: false,
        },
        1,
    ))
}

/// Splits a leading §11 `@@type{…}` head (or `[.c]{k:v}` block) off a cell,
/// returning the layer attributes, the marker and the remaining content.
fn parse_front_extras(text: &str) -> (LayerAttrs, Option<String>, String) {
    let trimmed = text.trim_start();
    if trimmed.starts_with("@@") {
        let chars: Vec<char> = trimmed.chars().collect();
        if let Some((head, next)) = scan_extras_chars(&chars, 0) {
            let bytes: usize = chars[..next].iter().map(|ch| ch.len_utf8()).sum();
            let layer = extras_to_layer(&head);
            let marker = layer.type_marker.clone();
            return (layer, marker, trimmed[bytes..].to_string());
        }
    }
    // Pre-§11 cell form: a leading `[.class,#id]{k:v}` block.
    if trimmed.starts_with('[')
        && trimmed
            .chars()
            .nth(1)
            .is_some_and(|ch| ch == '.' || ch == '#')
    {
        let (attrs, rest) = crate::attrs::parse_attr_block(trimmed);
        if attrs.id.is_some() || !attrs.classes.is_empty() || !attrs.properties.is_empty() {
            let consumed = trimmed.len().saturating_sub(rest.len());
            return (attr_block(&trimmed[..consumed]), None, rest.to_string());
        }
    }
    (LayerAttrs::default(), None, text.to_string())
}

/// Splits the trailing `@@type{…}` / `[.c,#id]{k:v}` block off a table line.
///
/// The block must sit **after** the last `|` of the line and be the only thing
/// there (`| a | b |@@rowB{.x}`), so cell content ending in braces is untouched.
/// Returns the line without the block and the parsed layer attributes.
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
        Some(attrs) if !attrs.is_empty() || attrs.type_marker.is_some() => {
            (&line[..pipe + 1], Some(attrs))
        }
        _ => (line, None),
    }
}

/// Parses one layer's attribute block: a §11 `@@type{…}` head, or the pre-§11
/// `[.class,#id]{key: value}` form. `None` when `input` is neither.
fn parse_layer_block(input: &str) -> Option<LayerAttrs> {
    let input = input.trim();
    if input.starts_with("@@") {
        let chars: Vec<char> = input.chars().collect();
        return match scan_extras_chars(&chars, 0) {
            Some((head, next)) if next == chars.len() => Some(extras_to_layer(&head)),
            _ => None,
        };
    }
    if input.starts_with('{') || input.starts_with('[') {
        let (attrs, rest) = crate::attrs::parse_attr_block(input);
        let has_attrs =
            attrs.id.is_some() || !attrs.classes.is_empty() || !attrs.properties.is_empty();
        return (has_attrs && rest.trim().is_empty()).then_some(attr_block(input));
    }
    None
}

/// Parses the §8 declaration line: `|-` `[slug]` `("title")` `extras?` `-|`.
///
/// Returns the `<table>` attributes and how many lines it consumed (0 when the
/// line is not a declaration). The declaration is childless: text that is not
/// part of a head or the extras is discarded.
fn parse_decl_line(line: &str) -> Option<(LayerAttrs, usize)> {
    let body = line.trim().strip_prefix("|-")?;
    let mut rest = body;

    let mut layer = LayerAttrs::default();

    // §8: the head uses the standard `[slug]` / `("title")` form.
    if let Some(stripped) = rest.strip_prefix('[') {
        if let Some(close) = stripped.find(']') {
            let slug = stripped[..close].trim();
            if !slug.is_empty() && !slug.starts_with('.') {
                layer.attrs.id = Some(slug.to_string());
            } else if slug.starts_with('.') {
                layer.attrs.classes.extend(
                    slug.trim_start_matches('.')
                        .split('.')
                        .filter(|token| !token.is_empty())
                        .map(str::to_string),
                );
            }
            rest = &stripped[close + 1..];
        }
    }
    if let Some(stripped) = rest.trim_start().strip_prefix('(') {
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

    // The trailing `-|` closes the declaration; the extras head precedes it.
    let rest = rest.trim();
    let extras_part = rest.strip_suffix("-|").unwrap_or(rest);
    if !extras_part.trim().is_empty() {
        let (head, marker, _content) = parse_front_extras(extras_part.trim());
        let mut head = head;
        if head.type_marker.is_none() {
            head.type_marker = marker;
        }
        crate::attrs::merge_layer(&mut layer, &head);
    }

    Some((layer, 1))
}

fn parse_delimiter_cells(
    delim_cells: &[String],
    header_cells: &[String],
) -> Option<Vec<ColumnSpec>> {
    let mut columns = Vec::new();

    for (i, delim) in delim_cells.iter().enumerate() {
        let text = delim.trim();
        let header_text = header_cells.get(i).cloned().unwrap_or_default();

        // §8: cell-front extras (`| @@type{…} :--- |`).
        let (layer, _marker, cell) = parse_front_extras(text);
        let mut core = cell.trim();

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

        // 6. Parse the deprecated trailing attribute block (§14).
        let (trailing, remaining) = crate::attrs::parse_attr_block(rest);
        if !remaining.trim().is_empty() {
            return None;
        }
        // The column attributes are the front extras followed by the legacy
        // block: classes accumulate in source order (§6.4).
        let mut merged = layer;
        merged.attrs.classes.extend(trailing.classes);
        for (key, value) in trailing.properties {
            merged.attrs.properties.retain(|(name, _)| *name != key);
            merged.attrs.properties.push((key, value));
        }
        if trailing.id.is_some() {
            merged.attrs.id = trailing.id;
        }

        columns.push(ColumnSpec {
            header_text,
            align,
            width,
            attrs: merged,
        });
    }

    Some(columns)
}

fn parse_body_cells(
    cells: &[String],
    _expected_cols: usize,
) -> Option<(LayerAttrs, Vec<CellSpec>)> {
    let mut parsed_cells = Vec::new();
    let mut row_attrs = LayerAttrs::default();

    for (i, cell) in cells.iter().enumerate() {
        let mut text = cell.trim();

        if i == cells.len() - 1 {
            if text.starts_with('-') {
                text = text[1..].trim();
            }
            let (attrs, content) = crate::attrs::parse_attr_block(text);
            if content.trim().is_empty()
                && (!attrs.classes.is_empty() || !attrs.properties.is_empty() || attrs.id.is_some())
            {
                row_attrs = attr_block(text);
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

    // §8: cell-front extras (`| @@cellA{.x} content |`).
    let (front, _marker, rest) = parse_front_extras(text);
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

    let mut content = rest.to_string();
    let mut attrs = front;

    // Pre-§11 trailing form: `content [.class]{key: value}`.
    let mut attr_start = None;
    for i in (0..rest.len()).rev() {
        if rest.as_bytes()[i] == b'[' || rest.as_bytes()[i] == b'{' {
            if i == 0 || rest.as_bytes()[i - 1] == b' ' || rest.as_bytes()[i - 1] == b'-' {
                attr_start = Some(i);
                break;
            }
        }
    }

    if let Some(start) = attr_start {
        let (parsed_attrs, tail) = crate::attrs::parse_attr_block(&rest[start..]);
        // Only treat the trailing block as attributes when it actually carries
        // an id, class, or property. A bare `{foo}` / `[]` is literal text.
        let has_attrs = parsed_attrs.id.is_some()
            || !parsed_attrs.classes.is_empty()
            || !parsed_attrs.properties.is_empty();
        if has_attrs && tail.trim().is_empty() {
            // The canonical (front) extras win over the deprecated block.
            let mut legacy = attr_block(&rest[start..]);
            legacy.attrs.classes.extend(attrs.attrs.classes);
            for (key, value) in attrs.attrs.properties {
                legacy.attrs.properties.retain(|(name, _)| *name != key);
                legacy.attrs.properties.push((key, value));
            }
            if attrs.attrs.id.is_some() {
                legacy.attrs.id = attrs.attrs.id;
            }
            for flag in attrs.flags {
                if !legacy.flags.contains(&flag) {
                    legacy.flags.push(flag);
                }
            }
            legacy.type_marker = legacy.type_marker.or(attrs.type_marker);
            attrs = legacy;

            let mut c = rest[..start].to_string();
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

    #[test]
    fn keeps_bare_braces_as_literal_text() {
        let cell = parse_cell_content("{foo}").unwrap();
        assert_eq!(cell.text, "{foo}");
        assert!(cell.attrs.attrs.properties.is_empty());
        assert!(cell.attrs.attrs.classes.is_empty());
        assert!(cell.attrs.attrs.id.is_none());
    }

    #[test]
    fn still_strips_valid_attribute_blocks() {
        let cell = parse_cell_content("[.text-red]").unwrap();
        assert_eq!(cell.text, "");
        assert_eq!(cell.attrs.attrs.classes, vec!["text-red".to_string()]);

        let cell = parse_cell_content("Nilai { rox: \"rox\" }").unwrap();
        assert_eq!(cell.text, "Nilai");
        assert_eq!(
            cell.attrs.attrs.properties,
            vec![("rox".to_string(), "rox".to_string())]
        );
    }
}
