use crossterm::style::{Color, Stylize};

#[derive(Clone, Copy)]
pub struct Theme {
    pub primary: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub muted: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            primary: Color::Blue,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            muted: Color::DarkGrey,
        }
    }
}

pub trait Paint {
    fn primary(self, theme: Theme) -> crossterm::style::StyledContent<String>;
    fn success(self, theme: Theme) -> crossterm::style::StyledContent<String>;
    fn warning(self, theme: Theme) -> crossterm::style::StyledContent<String>;
    fn error(self, theme: Theme) -> crossterm::style::StyledContent<String>;
    fn muted(self, theme: Theme) -> crossterm::style::StyledContent<String>;
}

impl Paint for String {
    fn primary(self, theme: Theme) -> crossterm::style::StyledContent<String> {
        self.with(theme.primary)
    }
    fn success(self, theme: Theme) -> crossterm::style::StyledContent<String> {
        self.with(theme.success)
    }
    fn warning(self, theme: Theme) -> crossterm::style::StyledContent<String> {
        self.with(theme.warning)
    }
    fn error(self, theme: Theme) -> crossterm::style::StyledContent<String> {
        self.with(theme.error)
    }
    fn muted(self, theme: Theme) -> crossterm::style::StyledContent<String> {
        self.with(theme.muted)
    }
}
