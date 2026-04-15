pub fn mark_inline(input: &str, patterns: &[String], open: &str, close: &str) -> String {
    if patterns.is_empty() {
        return input.to_string();
    }

    let mut out = String::new();
    let mut cursor = 0usize;
    while cursor < input.len() {
        if let Some((start, end)) = find_next_match(input, cursor, patterns) {
            out.push_str(&input[cursor..start]);
            out.push_str(open);
            out.push_str(&input[start..end]);
            out.push_str(close);
            cursor = end;
        } else {
            out.push_str(&input[cursor..]);
            break;
        }
    }

    out
}

pub fn replace_markers(input: &str, open: &str, close: &str) -> String {
    if !input.contains(open) && !input.contains(close) {
        return input.to_string();
    }

    input.replace(open, "<strong>").replace(close, "</strong>")
}

fn find_next_match(hay: &str, from: usize, patterns: &[String]) -> Option<(usize, usize)> {
    let mut best: Option<(usize, usize)> = None;
    for pat in patterns {
        if let Some((s, e)) = find_pattern(hay, from, pat) {
            best = match best {
                None => Some((s, e)),
                Some((bs, be)) => {
                    if s < bs || (s == bs && (e - s) > (be - bs)) {
                        Some((s, e))
                    } else {
                        Some((bs, be))
                    }
                }
            };
        }
    }
    best
}

fn find_pattern(hay: &str, from: usize, pat: &str) -> Option<(usize, usize)> {
    if pat.is_empty() || pat.chars().all(|c| c == '*') {
        return None;
    }

    if !pat.contains('*') {
        let idx = hay[from..].find(pat)? + from;
        return Some((idx, idx + pat.len()));
    }
    let parts: Vec<&str> = pat.split('*').collect();
    let mut search_start = from;
    while search_start <= hay.len() {
        let Some(pos) = hay[search_start..].find(parts[0]) else {
            break;
        };
        let mut idx = search_start + pos + parts[0].len();
        let mut ok = true;
        for part in parts.iter().skip(1) {
            if part.is_empty() {
                continue;
            }
            if let Some(p) = hay[idx..].find(*part) {
                idx = idx + p + part.len();
            } else {
                ok = false;
                break;
            }
        }
        if ok {
            if search_start + pos == idx {
                return None;
            }
            return Some((search_start + pos, idx));
        }
        search_start = search_start + pos + 1;
    }
    None
}
