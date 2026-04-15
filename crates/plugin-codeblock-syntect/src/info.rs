use self::LineKind::{Delete, Insert, Plain};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    Plain,
    Insert,
    Delete,
}

#[derive(Debug, Clone)]
pub struct LineRange {
    pub start: usize,
    pub end: usize,
    pub kind: LineKind,
}

#[derive(Debug, Clone, Default)]
pub struct ParsedInfo {
    pub lang: Option<String>,
    pub inline_patterns: Vec<String>,
    pub line_ranges: Vec<LineRange>,
    pub pre_classes: Vec<String>,
}

pub fn parse_info_string(raw: &str) -> ParsedInfo {
    let mut lang: Option<String> = None;
    let mut inline_patterns: Vec<String> = Vec::new();
    let mut line_ranges: Vec<LineRange> = Vec::new();
    let mut pre_classes: Vec<String> = Vec::new();

    let bytes: Vec<char> = raw.chars().collect();
    let mut i = 0usize;
    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }

        if bytes[i] == '"' {
            let start = i + 1;
            i += 1;
            while i < bytes.len() && bytes[i] != '"' {
                i += 1;
            }
            let end = i.min(bytes.len());
            let captured: String = bytes[start..end].iter().collect();
            inline_patterns.push(captured);
            i += 1;
            continue;
        }

        let start = i;
        while i < bytes.len() && !bytes[i].is_whitespace() {
            i += 1;
        }
        let token: String = bytes[start..i].iter().collect();
        if token.is_empty() {
            continue;
        }

        if let Some(rest) = token.strip_prefix(".") {
            pre_classes.push(rest.to_string());
            continue;
        }
        if let Some(rest) = token.strip_prefix("ins=") {
            if let Some((range, _)) = parse_range_str(rest, Insert) {
                line_ranges.push(range);
            }
            continue;
        }
        if let Some(rest) = token.strip_prefix("del=") {
            if let Some((range, _)) = parse_range_str(rest, Delete) {
                line_ranges.push(range);
            }
            continue;
        }
        if token.starts_with('{') {
            if let Some((range, _)) = parse_range_str(&token, Plain) {
                line_ranges.push(range);
            }
            continue;
        }
        if lang.is_none() {
            lang = Some(token.trim().to_string());
        } else {
            inline_patterns.push(token.trim().to_string());
        }
    }

    ParsedInfo {
        lang,
        inline_patterns,
        line_ranges,
        pre_classes,
    }
}

fn parse_range_str(input: &str, kind: LineKind) -> Option<(LineRange, usize)> {
    if !input.starts_with('{') {
        return None;
    }
    let end_idx = input.find('}')?;
    let body = &input[1..end_idx];
    let consumed = end_idx + 1;
    let mut parts = body.splitn(2, '-');
    let start = parts.next()?.trim().parse::<usize>().ok()?;
    let end = parts
        .next()
        .and_then(|p| p.trim().parse::<usize>().ok())
        .unwrap_or(start);
    let (start, end) = if start == 0 {
        (1, end.max(1))
    } else {
        (start, end)
    };
    Some((LineRange { start, end, kind }, consumed))
}

pub fn line_kind_at(line: usize, ranges: &[LineRange]) -> Option<LineKind> {
    let mut result: Option<LineKind> = None;
    for r in ranges {
        if line >= r.start && line <= r.end {
            result = Some(r.kind);
        }
    }
    result
}
