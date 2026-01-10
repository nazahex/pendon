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
