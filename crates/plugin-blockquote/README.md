# pendon-plugin-blockquote

Binds the **extras** a blockquote carries to the quote node, before
`plugin-markdown` parses the Markdown (`docs/spec/SYNTAX.md` §9.1, §9.2).

`>` stays plain Markdown, and a quote has **no head of its own**: the only way to
configure one is a decorator line directly above it.

```text
@@quoteA[intro]("title"){.x}    ← §9.1 decorator line with positional groups
> quoted text
```

Everything inside the `>` is content (§9.2): `> @@quoteA{.x} text` reads no head,
so the `@@` and the braces stay literal text, and a slot written in the body
belongs to whichever element claims it.

## What it does

- Finds every paragraph that opens a quote (`> …`), including the lines a lazy
  continuation adds, and strips exactly one `>` level from each of them.
- Reads **no** head from the body (§9.2): the decorator line is the whole of a
  quote's extras, so the element declares no slot contract, no preset and no
  owned field.
- Binds a decorator line directly above the quote through
  [`pendon_extra::bind_decorators`](../extra/src/decorator.rs) (§9.1) and reports
  the decorators that bind to nothing as `Severity::Warning`.
- Replaces the paragraph the core parser wrapped the quote in with the quote
  node: always the built-in `NodeKind::Element("blockquote")` (D8 — it carries
  every extra), tagged `__layer = "blockquote"` so the renderer can answer that
  layer with `[[task.blockquote.custom]]` (§11 rule 3, ADR-0008 `D-3a`). The
  plugin never swaps the node for a component itself.
- Marks the node `__plugin_kind = "block"`, so `plugin-markdown` re-lexes the
  stripped body as block content: headings, lists, tables, fences and nested
  quotes inside the quote follow the normal rules.

## Options

```toml
[[task.blockquote.custom]]
type = ["quoteA"]
name = "QuoteA"
imports = "import { QuoteA } from '@comp/Quote'"
template = "<QuoteA {...attrs}>{children}</QuoteA>"

[[task.blockquote.custom]] # answers every unclaimed type (§11 rule 3)
name = "QuoteDefault"
imports = "import { QuoteDefault } from '@comp/Quote'"
template = "<QuoteDefault {...attrs}>{children}</QuoteDefault>"
```

```rust
use pendon_plugin_blockquote::{process, solid_hints, BlockquoteOptions};

let events = pendon_plugin_blockquote::process(&parsed, &options);
let jsx = render_solid_with_hints(&markdown(&events), solid_hints(&options).as_ref());
```

`primary_layer()` is `blockquote` — the key `[[task.blockquote.custom]]`
addresses.

### Specs (§6.1, D-5)

An entry renames the §6.1 positional items through its `field.*` keys —
`field.specs.tick` / `field.specs.quote` for a `` `slug` `` / `"title"` item, and
`field.specs.class` / `.id` / `.style` for a `.x` / `#x` / `style:` item. An
assignment **moves** the item: the assigned path is the author's own public
field, which this element neither reads nor renames (`Specs::element_path`
answers `None`, §6.1). The table is in
[`docs/spec/SYNTAX.md` §6.1](../../docs/spec/SYNTAX.md).

## Notes

- The plugin runs **before** `plugin-markdown` (§12.1), like every other
  construct plugin.
- A decorator line inside a code fence or raw HTML is literal text (§4.3).
- Nesting is preserved one level at a time: `>> inner` leaves `> inner` for the
  Markdown pass, which then builds the second `<blockquote>`.

## CLI wiring

This crate is wired into the CLI plugin list: `blockquote` builds its
`BlockquoteOptions` from `[[task.blockquote.custom]]` and dispatches `process`
in the config runner (`apps/cli/src/plugins.rs`, `apps/cli/src/process.rs`).
Run `cargo test -p pendon-plugin-blockquote` for crate-level and end-to-end
coverage.
