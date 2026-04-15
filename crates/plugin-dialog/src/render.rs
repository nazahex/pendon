use crate::markdown::render_inline_markdown;
use crate::tokenize::{tokenize_content, DialogToken};

pub fn render_dt(speaker: &str, class: Option<&str>) -> String {
    let mut out = String::from("<dt");
    if let Some(cls) = class {
        out.push_str(" class=\"");
        out.push_str(&escape_html(cls));
        out.push('"');
    }
    out.push('>');
    out.push_str(&escape_html(speaker));
    out.push_str("</dt>");
    out
}

pub fn render_dd(content: &str, class: Option<&str>) -> String {
    let mut out = String::from("<dd");
    if let Some(cls) = class {
        out.push_str(" class=\"");
        out.push_str(&escape_html(cls));
        out.push('"');
    }
    out.push('>');

    let tokens = tokenize_content(content);
    let mut wrote_non_break = false;
    let mut last_was_break = false;

    for token in tokens {
        match token {
            DialogToken::Break => {
                out.push_str("<br />");
                last_was_break = true;
            }
            DialogToken::Quote(inner) => {
                let rendered = render_quote(&inner);
                if !rendered.is_empty() {
                    if wrote_non_break && !last_was_break {
                        out.push(' ');
                    }
                    out.push_str(&rendered);
                    wrote_non_break = true;
                    last_was_break = false;
                }
            }
            DialogToken::Italic(inner) => {
                let rendered = render_italic(&inner);
                if !rendered.is_empty() {
                    if wrote_non_break && !last_was_break {
                        out.push(' ');
                    }
                    out.push_str(&rendered);
                    wrote_non_break = true;
                    last_was_break = false;
                }
            }
            DialogToken::Plain(inner) => {
                let rendered = render_plain(&inner);
                if !rendered.is_empty() {
                    if wrote_non_break && !last_was_break {
                        out.push(' ');
                    }
                    out.push_str(&rendered);
                    wrote_non_break = true;
                    last_was_break = false;
                }
            }
        }
    }

    out.push_str("</dd>");
    out
}

fn render_quote(text: &str) -> String {
    let inner = render_inline_markdown(text.trim());
    if inner.is_empty() {
        return String::new();
    }
    format!("<q>{}</q>", inner)
}

fn render_italic(text: &str) -> String {
    let inner = render_inline_markdown(text.trim());
    if inner.is_empty() {
        return String::new();
    }
    format!("<i>{}</i>", inner)
}

fn render_plain(text: &str) -> String {
    let inner = render_inline_markdown(text.trim());
    if inner.is_empty() {
        return String::new();
    }
    format!("<p>{}</p>", inner)
}

fn escape_html(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}
