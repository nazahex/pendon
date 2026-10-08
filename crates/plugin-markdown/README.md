# pendon-plugin-markdown

Markdown-to-Pendon transformer that turns raw parser events into structured blocks, lists, tables, headings, inline formatting, and optional HTML. Use it after `pendon_core::parse` (or the CLI) to normalize Markdown into consistent `Event` streams for downstream plugins/renderers.

## What it does

- Builds block structure: paragraphs, headings (with `level` attrs), blockquotes, bullet/ordered lists (with `start` attr), table head/body from pipe rows, code fences (keeps `lang`), and thematic breaks.
- §9.1 decorators: a decorator line above a **paragraph** or a **code fence** is bound here and consumed. The decorator line may carry positional groups (`[…]` / `(…)`) between its type and body, which are resolved via the built-in §6.1 keys. A heading's decorator belongs to `plugin-section` (§9.5) and a table parses its own heads (§8), so both are declined; a list/quote opener is declined too, because `plugin-list` / `plugin-blockquote` own it.
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

### Positional keys (§6.1 / §11 rule 5)

`plugin-markdown` exposes no `[task.markdown.custom]` component set — its
`MarkdownOptions` only carries `allow_html` / `strip_comments` — so the blocks
it owns always use the built-in §6.1 keys (`slug` / `title`). The
`bind_decorators` resolver this plugin passes therefore returns the defaults.

Decorator lines bound by this plugin (above paragraphs and code fences) may
carry positional groups (`[…]` / `(…)`) that are resolved via these same
built-in keys.

## Notes

- §11 rule 3 (placement): a `Custom` **or** `Element` node declares how the Markdown
  pass must treat its content through the hidden `__plugin_kind` attribute the
  emitting plugin writes first (`plugin-custom`, `plugin-directive`,
  `plugin-table`). `block` / `codefence` / `blockquote` re-lexes the body as block
  content, `inline` as inline content, and `element` — or no attribute at all —
  passes the subtree through verbatim (the pre-rendered `figure` / `table`
  containers of `plugin-img` and `plugin-table`). This is why an unclaimed block
  directive (`<div>`) renders the lists and emphasis inside its body instead of
  leaking the raw Markdown.
- §9.3/§9.4 (list container merge): a node that declares `unordered` or `ordered`
  is a container wrapper from `plugin-list`. Its body is re-lexed as blocks and
  the list it decorates is built **inside** that node instead of in a `<ul>`/`<ol>`
  of its own, so the wrapper's extras — and the component that replaces the
  container — land on the list rather than around it. The `start` offset of an
  ordered list is kept, a nested list still opens its own element, and a wrapper
  whose body turns out not to be a list stays an ordinary block node (its extras
  are never dropped).
- Tables: first pipe row becomes `TableHead` until a separator row of dashes, then `TableBody` rows follow.
- Lists: ordered lists emit a `start` attribute on the first item when numbering begins at a value other than 1.
- Lists: an open item is closed through the same stack bookkeeping as every other
  node, so a block wrapper around a list (a directive, a decorated container) no
  longer emits a second `EndNode(ListItem)` for a stale entry.
- Inline code is recognised when the span opens at column 0 (a paragraph or a table cell that starts with `` `code` ``).
- An indented fence (the fence of a list item, for example) removes up to its own indentation from every content line, so fenced code inside a list keeps only the indentation it declares.
- HTML passthrough is deliberately opt-in to keep Markdown safe by default.

## Known gaps

- The §9.4 item layer (`list` / `<li>`) has no merge rule yet: an item-decorator
  at the start of an item's content (`- @@liItem{…} Alpha text`) reaches the
  pass as literal text. The `unordered` / `ordered` container layers are merged
  (see _Notes_).
- Fenced code blocks inside blockquotes (``> ```lang``) are not supported: the `>` prefix is kept as fence content and the closing fence is not detected, so the rest of the document is swallowed by the fence.
