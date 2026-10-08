# pendon-plugin-blockquote

Binds the **extras** a blockquote may carry to the quote node, before
`plugin-markdown` parses the Markdown (`docs/spec/SYNTAX.md` §9.1, §9.2).

`>` stays plain Markdown. This plugin only owns the two spellings a quote's
extras may use:

```text
@@quoteA[intro]("title"){.x}          ← §9.1 decorator line with positional groups, directly above the quote
> @@quoteA[intro]("title"){.x} quoted ← §9.2 inner head with positional groups, immediately inside the quote
```

When both are present the **inner** head wins (§9.2).

## What it does

- Finds every paragraph that opens a quote (`> …`), including the lines a lazy
  continuation adds.
- Parses the head immediately inside the quote (`> @@type{…} content`); the space
  between the marker, the head and the content is allowed here (§4.1 —
  blockquote is one of the two adjacency exceptions, with list).
- Binds a decorator line directly above the quote through
  [`pendon_extra::bind_decorators`](../extra/src/decorator.rs) (§9.1) and reports
  the decorators that bind to nothing as `Severity::Warning`.
- Replaces the paragraph the core parser wrapped the quote in with the quote
  node: `NodeKind::Custom(name)` when `[[task.blockquote.custom]]` claims the
  `type` (§11 rule 3), `NodeKind::Element("blockquote")` otherwise (D8 — the
  built-in element still carries every extra).
- Marks the node `__plugin_kind = "block"`, so `plugin-markdown` re-lexes the
  stripped body as block content: headings, lists, tables, fences and nested
  quotes inside the quote follow the normal rules.

## Options

```toml
[[task.blockquote.custom]]
type = ["quoteA"]
name = "QuoteA"
imports = "import { QuoteA } from '@comp/content/Quote'"
template = "<QuoteA {...attrs}>{children}</QuoteA>"

[[task.blockquote.custom]] # answers every unclaimed type (§11 rule 3)
name = "QuoteDefault"
imports = "import { QuoteDefault } from '@comp/content/Quote'"
template = "<QuoteDefault {...attrs}>{children}</QuoteDefault>"
```

```rust
use pendon_plugin_blockquote::{process, solid_hints, BlockquoteOptions};

let events = pendon_plugin_blockquote::process(&parsed, &options);
let jsx = render_solid_with_hints(&markdown(&events), solid_hints(&options).as_ref());
```

`primary_layer()` is `blockquote` — the key `[[task.blockquote.custom]]`
addresses.

### Positional keys (§6.1 / §11 rule 5)

An entry may rename the §6.1 positional slots in its extras head: `backtick_key`
(default `slug`) for a `` `slug` `` item and `quote_key` (default `title`) for a
`"title"` item. Keys the entry leaves unset keep the built-in default, so an
entry with no overrides behaves exactly as before.

## Notes

- The plugin runs **before** `plugin-markdown` (§12.1), like every other
  construct plugin.
- A decorator line inside a code fence or raw HTML is literal text (§4.3).
- Nesting is preserved one level at a time: `>> inner` leaves `> inner` for the
  Markdown pass, which then builds the second `<blockquote>`.

## Not yet wired

This crate is not part of the CLI plugin list yet: `apps/cli` still has to learn
`blockquote` (options + dispatch + `solid_hints`). Run `cargo test -p
pendon-plugin-blockquote` for the crate-level and end-to-end coverage.
