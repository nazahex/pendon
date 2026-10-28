# pendon-plugin-anchor

Link processing for Pendon.

`pendon-plugin-anchor` turns a **tagged** slot head — `@[inner text](href "title")`
— into a `<a>` node, attaches the extras head to it, and lets a custom Solid
component take over through `[[task.anchor.custom]]`. It runs as an inline text
processor before markdown parsing, so it claims its own heads before the Markdown
parser sees them.

The grammar is [`docs/spec/SYNTAX.md` §7.2](../../docs/spec/SYNTAX.md); this
README is the plugin's reference, never a second copy of the grammar.

## What This Plugin Does

- Parses **`@[inner_text](href "title")`** heads (§7.2) — the tag is mandatory
- Reads both slots with the shared slot grammar (§5.2), so `bracket.quote`,
  `bracket.tick` and `paren.tick` are kept as bare fields instead of dropped
- Applies the element preset `bracket.bare → inner_text`, `paren.bare → href`,
  `paren.quote → title` (§6.5)
- Attaches an adjacent `{…}` / `@@type{…}` extras head to the `<a>` (§6.3)
- Reports a head item that names `href` as an `Error` naming
  `field.override.href` (D-12 rule 5)
- Fronts a custom Solid component per `@@type{…}` marker (§11)
- Skips links inside code fences, inline code, HTML blocks and HTML inline elements
- Declares its contract, preset and owned fields as data (`pub const HEAD`,
  `crates/extra`'s `ElementHead`) — the single copy of §7.2's table

## What This Plugin Does **Not** Do (§14, D-16)

These were removed in the Stage-2 rework and are **not syntax**:

| Removed                                        | Today                                                      |
| :--------------------------------------------- | :--------------------------------------------------------- |
| untagged `[text](url)`                         | plain text; `plugin-markdown` links it, no extras, no type |
| `^` `~` `!` `$` `;;` `--` URL-suffix modifiers | ordinary URL characters                                    |
| automatic `target="_blank"` / `rel="noopener"` | nothing — write `target:` / `rel:` in the extras head      |
| the `^`/`~` conflict warning                   | gone with the modifiers                                    |

## Recommended Plugin Order

```text
micromatter,image,cite,wiki,anchor,markdown
```

**Why this order matters:**

1. `cite` and `wiki` must run before anchor because their syntax (`[^^](...)`, `[[...]]`) is more specific than the anchor's head
2. `anchor` runs before `markdown` so its tagged heads become nodes instead of CommonMark links
3. `markdown` runs last to handle every remaining inline construct

Example task config:

```toml
[[task]]
input = "./src/[slug].md"
output = "./out/[slug].jsx"
plugin = "micromatter,image,cite,wiki,anchor,markdown"
format = "solid"
```

## Link Syntax

### Basic Link

```md
@[Visit our docs](/docs/getting-started)
```

### Link with Title

```md
@[Visit our docs](/docs/getting-started "Getting Started Guide")
```

### Link with Extra Attributes

```md
@[Styled link](/page){class: "btn btn-primary", data-track: "cta-click"}
```

### Combined Syntax

```md
@[Partner site](https://partner.com/path "Partner Page")@@anchorA{data-campaign: "q4-launch", target: "_blank", rel: "noopener"}
```

## The slot head (§7.2, §6.5)

The tag is part of the syntax, not decoration: without `@` the bracket and paren
belong to nobody (they are universal, §5.2), so `[text](url)` stays plain text
and `plugin-markdown` links it with no extras and no `type`.

Both slots are read by the shared grammar (§5.2), and the preset assigns three of
the six positions:

| position       | preset name  | lands as              |
| :------------- | :----------- | :-------------------- |
| `bracket.bare` | `inner_text` | the text inside `<a>` |
| `paren.bare`   | `href`       | `href`                |
| `paren.quote`  | `title`      | `title`               |

The other three (`bracket.quote`, `bracket.tick`, `paren.tick`) are published as
attributes under their slot-qualified names, so a content is never dropped
(D-7); a `field.<slot>.<kind>` key in the entry moves a position to a field of
the author's own and voids the three preset rows for that entry (§6.1, §6.5).

The bracket's `|…` run is the §5.4 slot-modifier run: parsed and carried, but it
contributes nothing until the modifier config lands. Pendon ships no modifier.

## Extra Attributes

An extras head may be attached immediately after the closing parenthesis:

```md
@[Link](/page){class: "highlight", id: "important-link", --color: "blue"}
@[Link](/page)@@anchorA{.highlight, #important-link, data-track: "cta"}
```

The head is a single `{…}` block; the `@@type` prefix is optional and, when
present, is the §11 routing marker of the instance. `.class` items accumulate in
the `class` list, `#id` (or a positional `` `slug` ``) sets the element `id`, and
`key: "value"` items set arbitrary attributes.

Special handling (§6.2, D-12):

- The **construct head wins**: a head item that fills a position the preset
  already filled is a `Warning` and is dropped (`@[x](/u "T")@@a{"Other"}` keeps
  `T`)
- A head item that names a field this element writes itself (`href`) is an
  `Error` naming `field.override.href`
- `.class` items accumulate; `#id` wins the `id`, and a positional `` `slug` ``
  is the `id` **fallback** when no `#id` exists
- Keys starting with `--` are passed through as-is (for CSS custom properties)
- Every other key becomes a regular HTML/Solid attribute — `target` and `rel`
  were never special and are not special now
- Values can be quoted (`"value"`) or unquoted (`value`)

The head must touch the closing `)` (no space, and no space before its own `{`):
`@[Link](/page) {.x}` and `@[Link](/page)@@a {.x}` both stay literal text. The
retired `[.class,#id]{key: value}` block is literal text too.

## Custom Solid Component

Replace the default `Link` node with a custom Solid component:

```toml
[task.anchor.custom]
name = "Anchor"
template = "<Anchor href={attrs.href} target={attrs.target} rel={attrs.rel} class=\"{attrs.class}\">{children}</Anchor>"

[[task.anchor.custom.imports]]
module = "@comp/shared/Anchor"
default = "Anchor"
```

`imports` also accepts a raw import line
(`imports = ["import Anchor from '@comp/shared/Anchor';"]`). Entries coming from
the same module are merged into a single `import` statement.

### Available Template Attributes

| Attribute       | Description                                          |
| :-------------- | :--------------------------------------------------- |
| `{attrs.href}`  | Link target                                          |
| `{attrs.title}` | Link title (if provided)                             |
| `{attrs.class}` | Class list from the extras head (if provided)        |
| `{attrs.id}`    | `#id`, or the extras slug when no `#id` exists       |
| `{attrs.type}`  | The `@@type{…}` marker, so an entry can read it back |
| `{children}`    | `inner_text` — the text the `<a>` wraps              |

`target`, `rel`, `hreflang` and every other extras key reach the template as an
attribute of the same name; nothing is generated for them.

### Default HTML Output (No Custom Node)

Without a custom node, links render as standard `<a>` tags via the built-in `Link` node type:

```html
<a href="/docs" title="Docs">Visit our docs</a>
```

### Specs (§6.1, D-5)

An entry renames the §6.1 positional items through its `field.*` keys —
`field.specs.tick` / `field.specs.quote` for a `` `slug` `` / `"title"` item, and
`field.specs.class` / `.id` / `.style` for a `.x` / `#x` / `style:` item. An
assignment **moves** the item: the assigned path is the author's own public
field, which this element neither reads nor renames (`Specs::element_path`
answers `None`, §6.1). These keys do not void the slot preset — only a
`field.<slot>.<kind>` key does (§6.5, D-7). The table is in
[`docs/spec/SYNTAX.md` §6.1](../../docs/spec/SYNTAX.md).

## Behavioral Notes

- This plugin processes **raw text events** — it must run before `plugin-markdown`
- Links inside code fences, inline code, HTML blocks, and HTML inline elements are ignored
- Image syntax `![alt](src)` is another element's head (`plugin-image`, §7.1)
- Extra attribute items (`.class`, `#id`, `{key: val}`) only support flat values — nested objects are not supported
- Unquoted values in extra attrs are treated as literal strings (no expression evaluation)
- A malformed slot (`@[a "b](/x)`) is an `Error` and stays literal text (§4.3)

## Scope and Limitations

- Only processes the tagged `@[inner text](href "title")` head — reference-style links (`[label][ref]`) are not supported
- Nested brackets in the bracket group are not supported
- Parentheses inside the paren group must be balanced for correct parsing
- The plugin does not validate URLs or check for broken links
- Extra attributes require at least one extras item (`.class`, `#id`, `key: val` or a bare flag) after `)`; the head must be adjacent

## License

MIT
