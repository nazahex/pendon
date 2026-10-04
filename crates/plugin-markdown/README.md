# pendon-plugin-markdown

Markdown-to-Pendon transformer that turns raw parser events into structured blocks, lists, tables, headings, inline formatting, and optional HTML. Use it after `pendon_core::parse` (or the CLI) to normalize Markdown into consistent `Event` streams for downstream plugins/renderers.

## What it does

- Builds block structure: paragraphs, headings (with `level` attrs), blockquotes, bullet/ordered lists (with `start` attr), table head/body from pipe rows, code fences (keeps `lang`), and thematic breaks.
- Parses inline formatting: `*em*`, `__bold__`, `**strong**`, `` `code` ``, links `[text](href)`, and line breaks from trailing double spaces or `\\` → emits `<br />` as `HtmlInline`.
- Handles code fences: preserves fenced content verbatim; the leading newline after the fence is skipped to match Markdown expectations.
- Optional HTML passthrough: when enabled, copies HTML blocks/inline segments as `HtmlBlock`/`HtmlInline` nodes; otherwise HTML-like text is treated as plain text.
- Resets paragraph/list state around blockquotes and tables to avoid malformed nesting.

## Options

```rust
use pendon_plugin_markdown::{process_with_options, MarkdownOptions};

let opts = MarkdownOptions { allow_html: true };
let events = process_with_options(&parsed, opts);
```

- `allow_html` (default `false`): pass raw HTML blocks/inline through instead of leaving them as plain text.

## Usage

CLI (with micromatter frontmatter parser first):

```bash
pendon --plugin micromatter,markdown --format json --input ./doc.md
```

Library:

```rust
use pendon_core::parse;
use pendon_plugin_markdown::process;

let parsed = parse("# Title\n\nText.", &Default::default());
let normalized = process(&parsed);
```

## Notes

- Tables: first pipe row becomes `TableHead` until a separator row of dashes, then `TableBody` rows follow.
- Lists: ordered lists emit a `start` attribute on the first item when numbering begins at a value other than 1.
- Inline code is recognised when the span opens at column 0 (a paragraph or a table cell that starts with `` `code` ``).
- An indented fence (the fence of a list item, for example) removes up to its own indentation from every content line, so fenced code inside a list keeps only the indentation it declares.
- HTML passthrough is deliberately opt-in to keep Markdown safe by default.

## Known gaps

- Fenced code blocks inside blockquotes (``> ```lang``) are not supported: the `>` prefix is kept as fence content and the closing fence is not detected, so the rest of the document is swallowed by the fence.
