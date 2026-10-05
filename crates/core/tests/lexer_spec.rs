use pendon_core::{tokenize, Token};

#[test]
fn crlf_normalizes_to_single_newline_token() {
    let toks = tokenize("A\r\nB");
    let kinds: Vec<_> = toks
        .iter()
        .map(|t| match t {
            Token::Text(s) => (&"T"[..], s.len()),
            Token::Newline => ("N", 0),
            Token::FenceBackticks(n) => ("F", *n),
            Token::Hashes(n) => ("H", *n),
        })
        .collect();
    // Expect: Text("A"), Newline, Text("B")
    assert_eq!(kinds, vec![("T", 1), ("N", 0), ("T", 1)]);
}

#[test]
fn detects_hashes_and_backticks_at_line_start() {
    let toks = tokenize("### Title\n```\ncode\n```");
    // Should see Hashes(3), Text(" "), Text("Title"), Newline, FenceBackticks(3), Newline, Text("code"), Newline, FenceBackticks(3)
    let mut has_hashes = false;
    let mut has_fence = 0;
    for t in toks {
        match t {
            Token::Hashes(n) if n == 3 => has_hashes = true,
            Token::FenceBackticks(n) if n == 3 => has_fence += 1,
            _ => {}
        }
    }
    assert!(has_hashes);
    assert_eq!(has_fence, 2);
}

#[test]
fn short_backtick_run_keeps_the_line_in_one_text_token() {
    // A single backtick at column 0 may open an inline code span: the whole line
    // has to stay in one token so the inline parser can see its closing backtick.
    let toks = tokenize("`code` tail\nplain\n");
    let text: Vec<&str> = toks
        .iter()
        .filter_map(|t| match t {
            Token::Text(s) => Some(*s),
            _ => None,
        })
        .collect();
    assert_eq!(text, vec!["`code` tail", "plain"]);
    assert_eq!(toks.len(), 4, "two text tokens and two newlines: {toks:?}");
}

#[test]
fn double_backtick_run_is_not_a_fence() {
    let toks = tokenize("``x``\n");
    assert!(matches!(toks[0], Token::Text("``x``")));
}
