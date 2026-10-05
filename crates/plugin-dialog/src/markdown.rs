use pendon_core::{parse, Options};
use pendon_plugin_latex::{process_with_options as process_latex, LatexOptions};
use pendon_plugin_markdown::process as process_markdown;

pub fn render_inline_markdown(input: &str, latex: Option<LatexOptions>) -> String {
    if input.trim().is_empty() {
        return String::new();
    }

    let parsed = parse(input, &Options::default());
    let markdown = process_markdown(&parsed);
    // Math runs after markdown so a `$` inside a link destination is not
    // mistaken for inline math.
    let markdown = match latex {
        Some(latex) => process_latex(&markdown, &latex),
        None => markdown,
    };
    let rendered = pendon_renderer_html::render_html(&markdown);
    strip_single_paragraph_wrapper(rendered.trim())
}

fn strip_single_paragraph_wrapper(html: &str) -> String {
    if let Some(inner) = html
        .strip_prefix("<p>")
        .and_then(|s| s.strip_suffix("</p>"))
    {
        return inner.trim().to_string();
    }
    html.trim().to_string()
}
