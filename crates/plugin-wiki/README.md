# pendon-plugin-wiki

Wiki-style links and infoboxes ([`docs/spec/SYNTAX.md`](../../docs/spec/SYNTAX.md)
§7.5). It owns its own `[.class]`-shaped spellings (infobox markers, div blocks,
the `Term =[.cill,.elit] Value` class prefix), parsed by its own code — these are
**not** the retired core `[.c,#id]{k:v}` form (ADR-0001 D3), so they are not
"legacy" and must not be migrated.

## Syntax

```text
[[Target]]                       wikilink (label defaults to the target)
[[Target | Label]]               wikilink with a label
[[Target]]@@wikiA{…}             wikilink with an extras head (§5)
:::infobox[rox]                  infobox block (own `[.class]` spelling)
::[.img] … ::                     div marker with a class
Term =[.cill,.elit] Value         class prefix on an infobox term
```

Extras attach to the wiki `<a>`; `href` is produced by the plugin from
`link_prefix` and MUST NOT be overridable by extras (it wins).

## What it does

- Resolves `[[…]]` links into anchor nodes with an `href` built from
  `WikiOptions::link_prefix`.
- Processes infobox blocks and div markers with their own class parsing.
- Re-runs math inside infobox fragments when `WikiOptions::latex` is `Some`,
  otherwise leaves `$...$` untouched (matching a task without `latex`).

## Options

```rust
use pendon_plugin_wiki::{process, process_with_options, WikiOptions};

let events = process(&parsed); // default options

let opts = WikiOptions {
    link_prefix: Some("/wiki/".to_string()),
    latex: None,
};
let events = process_with_options(&parsed, opts);
```

`WikiOptions` fields: `link_prefix: Option<String>` and `latex: Option<LatexOptions>`.

### Positional keys (§6.1 / §11 rule 5)

An entry may rename the §6.1 positional slots in its extras head: `backtick_key`
(default `slug`) for a `` `slug` `` item and `quote_key` (default `title`) for a
`"title"` item.

> **Not yet wired:** `plugin-wiki`'s `WikiOptions` carries no `ComponentSet`, so
> a `[task.wiki.custom]` key is not read yet and the extras stay on the built-in
> `slug` / `title`. Adding the component set lifts this (see `STATUS.md`).

## Recommended order

```text
micromatter,img,cite,wiki,anchor,markdown
```

`wiki` runs before `anchor` because `[[…]]` is more specific than a generic
`[label](url)` link.

## License

MIT
