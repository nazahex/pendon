use crossterm::{
    execute,
    terminal::{Clear, ClearType},
};

use crate::templates::render_progress_bar;
use crate::theme::Theme;

pub struct ProgressBar {
    total: u64,
    current: u64,
    prefix: String,
    width: usize,
    theme: Theme,
}

impl ProgressBar {
    pub fn new(total: u64, prefix: impl Into<String>, theme: Theme) -> Self {
        Self {
            total,
            current: 0,
            prefix: prefix.into(),
            width: 30,
            theme,
        }
    }

    pub fn set_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    pub fn set(&mut self, current: u64) {
        self.current = current;
        render_progress_bar(
            &self.prefix,
            self.current,
            self.total,
            self.width,
            self.theme,
        );
    }

    pub fn inc(&mut self, delta: u64) {
        self.set(self.current.saturating_add(delta));
    }

    pub fn finish(self) {
        let mut err = std::io::stderr();
        let _ = execute!(err, Clear(ClearType::CurrentLine));
    }
}
