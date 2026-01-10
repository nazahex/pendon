#[derive(Debug, Clone, Copy)]
pub struct Options {
    pub strict: bool,
    pub max_doc_bytes: Option<usize>,
    pub max_line_len: Option<usize>,
    pub max_blank_run: Option<usize>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            strict: false,
            max_doc_bytes: None,
            max_line_len: None,
            max_blank_run: None,
        }
    }
}
