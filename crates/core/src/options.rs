#[derive(Debug, Clone, Copy, Default)]
pub struct Options {
    pub strict: bool,
    pub max_doc_bytes: Option<usize>,
    pub max_line_len: Option<usize>,
    pub max_blank_run: Option<usize>,
}
