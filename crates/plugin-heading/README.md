# pendon-plugin-heading

Heading parsing and custom-component plugin for Pendon.

`pendon-plugin-heading` turns a heading line into a structured `Heading` node with
its depth, slug, title and extras attributes, and routes it to a §11 component. It
runs **before** `markdown` (§7.4 of [`SYNTAX.md`](../../docs/spec/SYNTAX.md));
numbering is not part of it any more — [`number`](../number/README.md)
owns `attrs.number` (ADR-0010).

## What This Plugin Does

- Reads the §7.4 head `#[slug]("title")` and the §11 `{…}` / `@@type{…}` extras
  head that follows it, in one pass over the raw heading line
- Publishes `depth`: the `#` run **minus the one hash the tag spends** (D-11)
- Publishes `slug` (the head's `[slug]`, else a slug of the heading text),
  `title` (the head's `("title")`) and `id` (the universal extras `#id`, §6.3) —
  the plugin derives no id at all, and the tick carries no default meaning (§6.2)
- Emits the §11 layer tag `__layer = "heading"` in both modes, so
  `number` and `plugin-extract` can tell the element kind from the stream
- Strips the head from the visible text in both standard and custom modes; the
  visible text is the node's **content** (`inner_text`), never an attribute
- Routes the node to a §11 component through `@@type{…}`
- Declares its slot contract and preset assignment as data (`pub const HEAD`,
  `crates/extra`'s `ElementHead`) — the single copy of §7.4's
  `bracket.bare`/`paren.quote` mapping; `depth` and `inner_text` are fields of
  the element but are not slot-derived, so they are not in the table

## Recommended Plugin Order

```text
frontmatter,heading,anchor,image,number,markdown
```

**Why this order matters:**

1. `heading` runs **before** `markdown`: it rewrites heading lines itself, so it
   must see the raw text
2. `number` follows every element plugin that produces nodes it numbers and
   precedes `markdown`; the CLI rejects any other position
3. `markdown` converts whatever is left into the remaining core nodes

Example task config:

```toml
[[task]]
input = "./src/[slug].mdp"
output = "./out/[slug].jsx"
plugin = "frontmatter,heading,anchor,image,number,markdown"
format = "solid"

[[task.heading.number]]
type = ["HeadingA"]
style = ["decimal", "decimal", "lower-alpha"]
```

`[[task.heading.number]]` belongs to `number`, not to this crate; see its
README for styles, depth and the `%%numberType` marker.

## The Heading Head

```ebnf
heading = "#"(1..6) [ "[" bracket "]" [ "(" paren ")" ] ] extras? text
```

The head is one unit that touches the `#` run (§4.1). **The `#` immediately before
the bracket is the tag** (D-11), so the rest of the run is the depth:

| source                 | depth | rendered (standard mode)   |
| ---------------------- | ----- | -------------------------- |
| `## Title`             | 2     | `<h2 id="title">`          |
| `#[slug] Title`        | 1     | `<h1 id="slug">`           |
| `###[slug]("T") Title` | 2     | `<h2 id="slug" title="T">` |
| `######[]() Title`     | 5     | `<h5 id="title">`          |

A run with no bracket claims nothing and keeps its whole length; a claimed head
never goes below depth 1. `("Title")` written without a bracket is **text**, not a
head (§4.3). The extras head follows the slots adjacently, and the whitespace
between the head and the text belongs to the head (it is not part of the text).

### Slug and title

```md
##[introduction]("Heading X")@@type{`slug-extras`, .hero} Introduction
```

`slug="intro"`-style fields come from the preset: `bracket.bare` → `slug`,
`paren.quote` → `title` (§6.6). The extras tick is a plain `specs.tick` field: it
neither loses to nor fills in for the `[slug]` (§6.2). With no `[slug]`, `slug`
falls back to a slug of the heading text (`pendon_core::slugify`):

```md
##@@headingX{`slug-extras`, .fancy} Extras tick
```

`<h2 class="fancy" id="slug-extras" type="headingX">Extras tick</h2>` — no
bracket, so no tag: the depth stays 2.

### Classes, id, style and props

```md
##[intro]{.hero, .dark, #explicit, --accent: "blue", data-section: "overview"} Introduction
```

`<h1 class="hero dark" data-section="overview" id="explicit" style="--accent: blue">`:
`class` accumulates (§6.4), an extras `#id` beats the head's slug (§6.2), `--x`
merges into `style`, and any other pair is an attribute of its own. A bare item is
an attribute flag (§6.3), not an attribute with an empty value.

### Removed forms

The `[.class]` bracket and a `{key: value}` block written apart from the head are
literal text (§14).

## Numbering

This plugin does **not** number headings. `[[task.heading.number]]` and the
`%%numberType` marker are `number`'s (`crates/number/README.md`,
ADR-0010): it writes `attrs.number` as an array of strings per depth, and a
component decides how to display it. An authored `number:` prop still passes
through as an ordinary extras prop when no `number` block selects the element
(golden `06-heading`).

The retired `[task.heading]` keys `auto_number` and `number_style` fail the build,
and the error names `[[task.heading.number]]` (§13, §14).

## Custom Solid Component

Replace the default `<hN>` output with a custom Solid component:

```toml
[task.heading.custom.heading]
name = "DocHeading"
# `{...attrs}` spreads every attribute, so the component receives `number` as a
# real array when `number` wrote one (golden `30-number-heading`).
template = "<DocHeading {...attrs}>{children}</DocHeading>"

[[task.heading.custom.heading.imports]]
module = "@/components/DocHeading"
default = "DocHeading"
```

`imports` also accepts a raw import line
(`imports = ["import DocHeading from '@/components/DocHeading';"]`). Entries
coming from the same module are merged into a single `import` statement.

### Available Template Attributes

| Attribute        | Description                                                           |
| :--------------- | :-------------------------------------------------------------------- |
| `{attrs.depth}`  | The depth as a string (`"1"`…`"6"`, §7.4)                             |
| `{attrs.id}`     | The universal extras `#id` (§6.2, §6.3)                               |
| `{attrs.slug}`   | The head's `[slug]`, else a slug of the heading text                  |
| `{attrs.title}`  | The head's `("title")`                                                |
| `{attrs.number}` | Written by `number` when a `number` block selects the heading (array) |
| `{attrs.class}`  | Space-separated class list from the extras head                       |
| `{attrs.type}`   | The `@@type` marker (§11 rule 3)                                      |
| `{attrs.*}`      | Any other extras pair, under its own name                             |
| `{attrs.style}`  | Inline style string from `{--var: val}`                               |
| `{children}`     | The visible heading text (the head stripped, nothing prepended)       |

### Standard Mode Behavior

Without a custom component the plugin emits a `Heading` node with `depth`, `id`,
`slug`, `title` (when the head has one) and the extras attributes and flags
(§6.3). The built-in `<hN>` reads `depth` and prints every other attribute except
`slug`, which is a template-only read (§11 rule 3). The heading text is the
node's content; nothing is prepended to it.

### Specs (§6.1, D-5)

An entry renames the §6.1 positional slots through its `field.*` keys —
`field.specs.tick` / `field.specs.quote` for a tick / quote item, and
`field.specs.class` / `.id` / `.style` for a `.x` / `#x` / `style:` item. An
entry that configures none keeps the §6.1 default paths; once it assigns a
`field.<slot>.<kind>` row the preset is void as a whole (D-7), a `field.specs.*`
key alone voids nothing. An assignment also
**moves** the position: the assigned path is the author's own public field,
which this element neither reads nor renames (`Specs::element_path` answers
`None`, §6.1). The table is in
[`docs/spec/SYNTAX.md` §6.1](../../docs/spec/SYNTAX.md).

Only the **extras** head is affected: the `[slug]` bracket and `("title")`
parentheses are §7.4's fixed slots and are not renamed. The element reads the
`specs.tick` and `specs.quote` positions into `slug` / `title` only while the
matched entry leaves them on their default paths — move one and it becomes a field
of the author's own (D-7).

## Slot modifiers (§5.4, D-9)

The bracket's `|…` run contributes configured fields to the heading's `<hN>`
(§5.4, [ADR-0017](../../docs/decisions/0017-slot-modifiers.md)). The run is
**pure configuration**, so it contributes last: a field the decorator or the
specs items already produced wins with a `Warning`, and `class` accumulates
(§6.4). A symbol with no `bracket.modifier."<symbol>"` config contributes
nothing, and a config-only entry (no `name` / `template`) is enough to define one
(D-8) — the heading keeps its built-in `<hN>` and only gains the fields. Golden
`43-slot-modifiers` renders such an entry.

## Behavioral Notes

- This plugin parses heading lines itself — it must run **before** `plugin-markdown`
- The head (`[slug]`, `("title")`, `{…}` / `@@type{…}`) is stripped from the visible text in all modes
- The head must be adjacent: `## Title {.x}` and `{key: val}` written apart are literal text
- Unquoted values in `{key: val}` are accepted (numbers, booleans stored as strings)
- `slug` is the `[slug]` slot, or `slugify` of the heading text when the head has
  none (text with no alphanumeric character publishes no slug at all)
- No id is derived: `id` is the universal extras `#id`, so a `[slug]` is never an id
- `plugin-section` does not suppress any of this; the section keeps its own id
  chain and reports a heading extras `#id` instead of consuming it (§9.5)

## Scope and Limitations

- The head is positional: `[slug]`, then `("title")`, then the `{…}` extras head
- A `("title")` needs its bracket; on its own it is text (§4.3)
- Punctuation that does not start `(`, `{` or `@@` right after the head is literal text
- Nested braces inside `{key: val}` are not supported
- Comma-separated values inside attribute values are not supported (commas delimit pairs)
- The plugin does not generate anchor links or permalink icons — use a custom component for those patterns
- `title` is the head's string, not the text: inline markup stays in `{children}`

## License

MIT
