use crate::info::{line_kind_at, LineKind, LineRange};

pub fn wrap_lines(s: &str, ranges: &[LineRange]) -> String {
    let mut lines: Vec<&str> = s.split('\n').collect();
    // Highlighter output commonly ends with a single trailing newline.
    // Dropping one terminal empty segment prevents a phantom blank row.
    if matches!(lines.last(), Some(last) if last.is_empty()) {
        lines.pop();
    }
    let mut out = String::new();

    for (idx, line) in lines.iter().enumerate() {
        let line_no = idx + 1;
        let class = match line_kind_at(line_no, ranges) {
            Some(LineKind::Insert) => Some("ins"),
            Some(LineKind::Delete) => Some("del"),
            Some(LineKind::Plain) => Some("mark"),
            None => None,
        };
        out.push_str("<p");
        if let Some(cls) = class {
            out.push_str(" class=\"");
            out.push_str(cls);
            out.push('"');
        }
        out.push('>');
        if line.is_empty() {
            out.push_str("&#8203;");
        } else {
            out.push_str(line);
        }
        out.push_str("</p>");
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drops_single_trailing_empty_line() {
        let out = wrap_lines("a\nb\n", &[]);
        assert_eq!(out, "<p>a</p><p>b</p>");
        assert!(!out.contains("&#8203;"));
    }

    #[test]
    fn keeps_intentional_internal_empty_line() {
        let out = wrap_lines("a\n\nb\n", &[]);
        assert_eq!(out, "<p>a</p><p>&#8203;</p><p>b</p>");
    }
}
