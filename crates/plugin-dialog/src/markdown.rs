use pendon_core::{parse, Options};
use pendon_plugin_markdown::process as process_markdown;

pub fn render_inline_markdown(input: &str) -> String {
    if input.trim().is_empty() {
        return String::new();
    }

    let parsed = parse(input, &Options::default());
    let rendered = pendon_renderer_html::render_html(&process_markdown(&parsed));
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
