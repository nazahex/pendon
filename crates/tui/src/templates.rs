use crossterm::{execute, style::Attribute, style::Print, terminal::Clear, terminal::ClearType};
use std::io::{self};

use crate::assets::{ascii, Glyphs};
use crate::theme::Theme;

pub fn render_status_line(msg: &str, theme: Theme) {
    let mut err = io::stderr();
    let styled = msg.to_string().with(theme.primary);
    let _ = execute!(
        err,
        Clear(ClearType::CurrentLine),
        Print(styled),
        Print("\r")
    );
}

pub fn render_progress_bar(prefix: &str, current: u64, total: u64, width: usize, theme: Theme) {
    let mut err = io::stderr();
    let width = width.max(10);
    let pct = if total == 0 {
        0.0
    } else {
        current as f64 / total as f64
    };
    let filled = ((width as f64) * pct).round() as usize;
    let mut bar = String::with_capacity(width + 2);
    bar.push('[');
    for i in 0..width {
        bar.push(if i < filled { '#' } else { ' ' });
    }
    bar.push(']');
    let pct_text = format!(" {:>3}%", (pct * 100.0).round() as u64).with(theme.muted);
    let line = format!("{} {}", prefix, bar);
    let _ = execute!(
        err,
        Clear(ClearType::CurrentLine),
        Print(line),
        Print(pct_text),
        Print("\r")
    );
}

trait StyleExt {
    fn with(self, color: crossterm::style::Color) -> crossterm::style::StyledContent<String>;
}

pub enum SeverityLine {
    Done,
    Info,
    Warn,
    Error,
}

pub fn render_severity_line(sev: SeverityLine, title: &str, theme: Theme) {
    let glyphs: Glyphs = ascii();
    let mut err = io::stderr();
    let (icon, label, color) = match sev {
        SeverityLine::Done => (glyphs.tick, "⟦done⟧", theme.success),
        SeverityLine::Info => (glyphs.info, "⟦info⟧", theme.primary),
        SeverityLine::Warn => (glyphs.warn, "⟦warn⟧", theme.warning),
        SeverityLine::Error => (glyphs.cross, "⟦error⟧", theme.error),
    };
    let icon_part = format!("{} ", icon).with(color);
    let mut label_bold = label.to_string().with(color);
    label_bold = crossterm::style::Stylize::attribute(label_bold, Attribute::Bold);
    let title_part = title.to_string().with(color);
    let _ = execute!(
        err,
        Clear(ClearType::CurrentLine),
        Print(icon_part),
        Print(" "),
        Print(label_bold),
        Print(" "),
        Print(title_part),
        Print("\n")
    );
}

pub fn render_kv_list(prefix: &str, items: &[(&str, &str)], theme: Theme) {
    let glyphs = ascii();
    let mut err = io::stderr();
    let _ = execute!(err, Print(format!("{}\n", prefix).with(theme.muted)));
    for (k, v) in items {
        let line = format!("  {} {}: {}", glyphs.bullet, k, v);
        let _ = execute!(err, Print(line), Print("\n"));
    }
}

pub fn render_bullet_list(title: &str, bullets: &[String], theme: Theme) {
    let glyphs = ascii();
    let mut err = io::stderr();
    let _ = execute!(err, Print(format!("{}\n", title).with(theme.muted)));
    for b in bullets {
        let line = format!("  {} {}", glyphs.bullet, b);
        let _ = execute!(err, Print(line), Print("\n"));
    }
}
impl StyleExt for String {
    fn with(self, color: crossterm::style::Color) -> crossterm::style::StyledContent<String> {
        crossterm::style::Stylize::with(self, color)
    }
}
