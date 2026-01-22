#[derive(Clone, Copy)]
pub struct Glyphs {
    pub tick: char,
    pub cross: char,
    pub info: char,
    pub warn: char,
    pub bullet: char,
}

pub fn ascii() -> Glyphs {
    Glyphs {
        tick: '✔',
        cross: '✖',
        info: '◆',
        warn: '▲',
        bullet: '-',
    }
}

pub fn nerd() -> Glyphs {
    // Keep same defaults; can be extended when nerd-fonts feature is used
    ascii()
}

pub const SPINNER_FRAMES_ASCII: &[&str] = &["-", "\\", "|", "/"];
pub const SPINNER_FRAMES_DOTS: &[&str] = &["⠁", "⠂", "⠄", "⠂"];
