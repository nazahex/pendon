# pendon-plugin-wiki

> **Deletion note (out of bounds).** The **infobox** half of this plugin is
> **deleted entirely in the next workflow**, together with `quiz` and `vicado`.
> Do not migrate infobox nodes to `__layer` / `layer_templates`, do not add
> types/selectors for them, and do not debug their output. Wikilinks stay in
> scope.

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

### Specs ((§6.1, D-5))

An entry renames the §6.1 positional slots through its `field.*` keys —
`field.specs.tick` / `field.specs.quote` / `.class` / `.id` / `.style` — and an
entry that configures none keeps `slug` / `title` / `class` / `id` / `style`
(D-7). The infobox link reads the **quote** position into its `title` while that
position keeps its default path; a `field.specs.quote` assignment publishes the
author's own field and the link stops reading it (§6.1). `plugin-wiki` does not
read the other positions yet (see below).

> **Not yet wired:** `plugin-wiki`'s `WikiOptions` carries no `ComponentSet`, so
> a `[task.wiki.custom]` key is not read yet and the extras stay on the built-in
> `slug` / `title`. Adding the component set lifts this (see `STATUS.md`).

## Recommended order

```text
micromatter,image,cite,wiki,anchor,markdown
```

`wiki` runs before `anchor` because `[[…]]` is more specific than a generic
`[label](url)` link.

## License

MIT
