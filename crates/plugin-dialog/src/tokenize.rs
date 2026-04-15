#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogToken {
    Quote(String),
    Italic(String),
    Break,
    Plain(String),
}

pub fn tokenize_content(content: &str) -> Vec<DialogToken> {
    let bytes = content.as_bytes();
    let mut out: Vec<DialogToken> = Vec::new();
    let mut buf = String::new();
    let mut i = 0usize;

    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'\\' && bytes[i + 1] == b'\\' {
            flush_plain(&mut out, &mut buf);
            out.push(DialogToken::Break);
            i += 2;
            continue;
        }

        if i + 1 < bytes.len()
            && ((bytes[i] == b'_' && bytes[i + 1] == b'(')
                || (bytes[i] == b'*' && bytes[i + 1] == b'('))
        {
            let closer = if bytes[i] == b'_' { ")_" } else { ")*" };
            if let Some(pos) = content[i + 2..].find(closer) {
                flush_plain(&mut out, &mut buf);
                let start = i + 1;
                let end = i + 2 + pos;
                out.push(DialogToken::Italic(content[start..=end].to_string()));
                i = i + 2 + pos + 2;
                continue;
            }
        }

        if bytes[i] == b'"' {
            if let Some(end) = find_next_quote(content, i + 1) {
                flush_plain(&mut out, &mut buf);
                out.push(DialogToken::Quote(content[i + 1..end].to_string()));
                i = end + 1;
                continue;
            }
        }

        buf.push(bytes[i] as char);
        i += 1;
    }

    flush_plain(&mut out, &mut buf);
    out
}

fn find_next_quote(s: &str, from: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut i = from;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn flush_plain(out: &mut Vec<DialogToken>, buf: &mut String) {
    if buf.trim().is_empty() {
        buf.clear();
        return;
    }
    out.push(DialogToken::Plain(buf.trim().to_string()));
    buf.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_quote_break_and_italic() {
        let t = tokenize_content("\"hello\"\\\\_(note)_");
        assert_eq!(
            t,
            vec![
                DialogToken::Quote("hello".to_string()),
                DialogToken::Break,
                DialogToken::Italic("(note)".to_string())
            ]
        );
    }
}
