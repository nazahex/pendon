#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token<'a> {
    Text(&'a str),
    Newline,
    // Detectors (not yet used by parser; reserved for future)
    FenceBackticks(usize), // count of backticks at line start
    Hashes(usize),         // count of '#'
}

pub fn tokenize<'a>(input: &'a str) -> Vec<Token<'a>> {
    let bytes = input.as_bytes();
    let mut i = 0;
    let mut out = Vec::with_capacity(8 + input.len() / 8);

    let mut line_start = true;
    while i < bytes.len() {
        match bytes[i] {
            b'\r' => {
                // CRLF -> single newline
                if i + 1 < bytes.len() && bytes[i + 1] == b'\n' { i += 2; } else { i += 1; }
                out.push(Token::Newline);
                line_start = true;
            }
            b'\n' => {
                i += 1;
                out.push(Token::Newline);
                line_start = true;
            }
            b'`' if line_start => {
                // Detect fence backticks run
                let start = i;
                while i < bytes.len() && bytes[i] == b'`' { i += 1; }
                let count = i - start;
                if count >= 3 { out.push(Token::FenceBackticks(count)); }
                else { out.push(Token::Text(&input[start..i])); }
                line_start = false;
            }
            b'#' if line_start => {
                // Detect heading hashes run
                let start = i;
                while i < bytes.len() && bytes[i] == b'#' { i += 1; }
                let count = i - start;
                out.push(Token::Hashes(count));
                // consume following single space if present as text for now
                if i < bytes.len() && bytes[i] == b' ' { out.push(Token::Text(" ")); i += 1; }
                line_start = false;
            }
            _ => {
                // Consume until next newline or detector at line start
                let start = i;
                while i < bytes.len() {
                    match bytes[i] {
                        b'\r' | b'\n' => break,
                        _ => { i += 1; }
                    }
                }
                if start != i { out.push(Token::Text(&input[start..i])); }
                line_start = false;
            }
        }
    }
    out
}
