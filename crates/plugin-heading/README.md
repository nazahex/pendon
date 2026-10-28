# pendon-plugin-heading

Heading parsing and custom-component plugin for Pendon.

`pendon-plugin-heading` turns a heading line into a structured `Heading` node with
an id, extras attributes and a `raw_title`, and routes it to a §11 component. It
runs **before** `markdown` (§7.4 of [`SYNTAX.md`](../../docs/spec/SYNTAX.md));
numbering is not part of it any more — [`plugin-number`](../plugin-number/README.md)
owns `attrs.number` (ADR-0010).

## What This Plugin Does

- Extracts custom IDs from the `[id]` prefix syntax in heading text
- Parses extra classes, IDs, key-value attributes and flags from a `{…}` extras head
- Emits the §11 layer tag `__layer = "heading"` in both modes
- Auto-generates URL-safe slugs as fallback IDs when no custom ID is provided
- Supports custom Solid components with full attribute access, including `raw_title`
- Strips all extras syntax from visible heading text in both standard and custom modes
- Declares its slot contract and preset assignment as data (`pub const HEAD`,
  `crates/extra`'s `ElementHead`) — the single copy of §7.4's
  `bracket.bare`/`paren.quote` mapping; `depth` and `inner_text` are fields of the
  element but are not slot-derived, so they are not in the table

## Recommended Plugin Order

```text
micromatter,heading,anchor,image,number,markdown
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
plugin = "micromatter,heading,anchor,image,number,markdown"
format = "solid"

[[task.heading.number]]
type = ["HeadingA"]
style = ["decimal", "decimal", "lower-alpha"]
```

`[[task.heading.number]]` belongs to `plugin-number`, not to this crate; see its
README for styles, depth and the `%%numberType` marker.

## Heading Extras Syntax

A heading head touches the `#` run; the extras head is a single `{…}` block
further items may follow inside it.

### Custom ID

```md
##[introduction] Introduction to the Topic
```

Produces `<h2 id="introduction">` (or custom component with `id="introduction"`).

### Classes and ID in the extras head

```md
##{.hero, .featured} Featured Section

##[intro]{.hero, .dark} Introduction
```

The first heading has `class="hero featured"`; the second has `id="intro"` and
`class="hero dark"` (§6.2: the `[slug]` head beats a positional slug).

### Key-Value Attributes

```md
##[intro]{data-section: "overview", --accent: "blue"} Overview
```

Produces heading with `data-section="overview"` and `style="--accent: blue;"`.

### Full Combined Syntax

```md
##[intro]("Heading X"){.hero, .dark, data-section: "overview", --accent: "blue"} Introduction
```

`[slug]`, `("title")` and the extras head are all optional and must be adjacent,
in that order. The retired `[.class]` bracket and `{key: value}` blocks written
apart from the head are literal text.

## Numbering

This plugin does **not** number headings. `[[task.heading.number]]` and the
`%%numberType` marker are `plugin-number`'s (`crates/plugin-number/README.md`,
ADR-0010): it writes `attrs.number` as an array of strings per depth, and a
component decides how to display it. An authored `number:` prop still passes
through as an ordinary extras prop when no `number` block selects the element
(golden `06-heading`).

The retired `[task.heading]` keys `auto_number` and `number_style` fail the build
with a §14 migration message pointing at `[[task.heading.number]]`.

## Custom Solid Component

Replace the default `<hN>` output with a custom Solid component:

```toml
[task.heading.custom.heading]
name = "DocHeading"
# `{...attrs}` spreads every attribute, so the component receives `number` as a
# real array when `plugin-number` wrote one (golden `30-number-heading`).
template = "<DocHeading {...attrs}>{children}</DocHeading>"

[[task.heading.custom.heading.imports]]
module = "@/components/DocHeading"
default = "DocHeading"
```

`imports` also accepts a raw import line
(`imports = ["import DocHeading from '@/components/DocHeading';"]`). Entries
coming from the same module are merged into a single `import` statement.

### Available Template Attributes

| Attribute           | Description                                                                  |
| :------------------ | :--------------------------------------------------------------------------- |
| `{attrs.level}`     | Heading level as string (`"2"`, `"3"`, etc.)                                 |
| `{attrs.id}`        | Custom ID from `[id]` syntax                                                 |
| `{attrs.slug}`      | Auto-generated slug (only when no custom ID is provided)                     |
| `{attrs.number}`    | Written by `plugin-number` when a `number` block selects the heading (array) |
| `{attrs.raw_title}` | Heading text after the head, extras stripped                                 |
| `{attrs.class}`     | Space-separated class list from the extras head                              |
| `{attrs.data-*}`    | Any extra data attributes from `{key: val}`                                  |
| `{attrs.style}`     | Inline style string from `{--var: val}`                                      |
| `{children}`        | Heading inline content (the head stripped, nothing prepended)                |

### Standard Mode Behavior

Without a custom component the plugin emits a `Heading` node with `level`, the
optional `id` (or fallback `slug`), `raw_title` and the extras attributes
(§6.4). The heading text is the node's content; nothing is prepended to it.

### Specs ((§6.1, D-5))

An entry renames the §6.1 positional slots through its `field.*` keys —
`field.specs.tick` / `field.specs.quote` for a `` `slug` `` / `"title"` item, and
`field.specs.class` / `.id` / `.style` for a `.x` / `#x` / `style:` item. An
entry that configures none keeps `slug` / `title` / `class` / `id` / `style`;
once it configures one the preset is void as a whole (D-7). An assignment also
**moves** the position: the assigned path is the author's own public field,
which this element neither reads nor renames (`Specs::element_path` answers
`None`, §6.1). The table is in
[`docs/spec/SYNTAX.md` §6.1](../../docs/spec/SYNTAX.md).

Only the **extras** head is affected: the heading's own `[slug]` bracket (§7.4
`id`) and `("title")` parentheses are fixed slots and are not renamed.

## Behavioral Notes

- This plugin parses heading lines itself — it must run **before** `plugin-markdown`
- Extras syntax (the `[id]` / `("title")` head and the `{…}` extras head) is stripped from visible text in all modes
- The head must be adjacent: `## Title {.x}`, `##[id]{.class}` and `{key: val}` written apart are literal text
- Unquoted values in `{key: val}` are accepted (numbers, booleans stored as strings)
- Slug generation uses lowercase alphanumeric characters and hyphens only
- §9.5: with `section_owns_id = true` no `id`/`slug` is emitted (the section owns it)
- Empty headings (after extras stripping) still receive an ID/slug

## Scope and Limitations

- Extras parsing is positional: `[slug]`, then `("title")`, then the `{…}` extras head
- Punctuation that does not start `(`, `{` or `@@` right after the head is literal text
- Nested braces inside `{key: val}` are not supported
- Comma-separated values inside attribute values are not supported (commas delimit pairs)
- The plugin does not generate anchor links or permalink icons — use a custom component for those patterns
- `raw_title` does not include inline formatting markers — it contains the plain text content after extras removal

## License

MIT
