# Pendon-LaTeX Walkthrough

This walkthrough covers the built-in `pendon-plugin-latex` SSR flow.

## What it does

- Scans `Event::Text` nodes for inline math `$...$` and display math `$$...$$`.
- Skips text inside code fences and HTML blocks or inline HTML.
- Renders formulas with `katex::render_with_opts`.
- Replaces formulas with `HtmlInline` or `HtmlBlock` event wrappers containing KaTeX HTML.

## Verify locally

Run the plugin tests:

```bash
cargo test -p pendon-plugin-latex
```

Render the example document through the CLI:

```bash
cargo run --bin pendon -- --plugin markdown,latex --format html < sandbox/examples/math.md
```

The output should contain KaTeX markup such as `<span class="katex">...` for inline math and `<span class="katex-display">...` for display math.
