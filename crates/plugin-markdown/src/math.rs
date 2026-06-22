/// Math delimiter scanning — rules match plugin-latex.

/// Toggle `open` for each unescaped `$$` pair found on `line`.
pub fn toggle_display_math_on_line(line: &str, open: &mut bool) {
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut i = 0;
    while i < len {
        if chars[i] == '\\' && i + 1 < len && chars[i + 1] == '$' {
            i += 2;
            continue;
        }
        if i + 1 < len && chars[i] == '$' && chars[i + 1] == '$' {
            *open = !*open;
            i += 2;
            continue;
        }
        i += 1;
    }
}

/// If `chars[cursor..]` begins a math region, return the exclusive end index.
pub fn math_region_end(chars: &[char], cursor: usize) -> Option<usize> {
    let len = chars.len();
    if cursor >= len {
        return None;
    }

    if chars[cursor] == '\\' && cursor + 1 < len && chars[cursor + 1] == '$' {
        return None;
    }

    if cursor + 1 < len && chars[cursor] == '$' && chars[cursor + 1] == '$' {
        let mut end = cursor + 2;
        while end + 1 < len {
            if chars[end] == '$' && chars[end + 1] == '$' {
                return Some(end + 2);
            }
            end += 1;
        }
        return None;
    }

    if chars[cursor] == '$' {
        let start = cursor;
        let mut end = cursor + 1;
        if end < len && chars[end] != ' ' && chars[end] != '\n' {
            while end < len {
                if chars[end] == '$' {
                    if end > start + 1 && chars[end - 1] != ' ' && chars[end - 1] != '\n' {
                        return Some(end + 1);
                    }
                }
                end += 1;
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chars(s: &str) -> Vec<char> {
        s.chars().collect()
    }

    #[test]
    fn inline_math_region() {
        let c = chars("$O_{m}$");
        assert_eq!(math_region_end(&c, 0), Some(7));
    }

    #[test]
    fn display_math_region() {
        let c = chars("$$x = y$$");
        assert_eq!(math_region_end(&c, 0), Some(9));
    }

    #[test]
    fn dollar_amount_is_not_math() {
        let c = chars("$100");
        assert_eq!(math_region_end(&c, 0), None);
    }

    #[test]
    fn escaped_dollar_is_not_math() {
        let c = chars("\\$5");
        assert_eq!(math_region_end(&c, 0), None);
    }

    #[test]
    fn toggles_display_math_state() {
        let mut open = false;
        super::toggle_display_math_on_line("$$\\begin{aligned}", &mut open);
        assert!(open);
        super::toggle_display_math_on_line("\\end{aligned}$$", &mut open);
        assert!(!open);
    }
}
