# pendon-plugin-image

Image processing for Pendon.

`pendon-plugin-image` turns a **tagged** slot head — `![alt](src)` — into an
`<img>` node, attaches the extras head to that same node, and lets a custom Solid
component take over through `[[task.image.custom.img]]`. It runs before markdown
parsing, so it claims its own heads before the Markdown parser sees them.

The grammar is [`docs/spec/SYNTAX.md` §7.1](../../docs/spec/SYNTAX.md); this
README is the plugin's reference, never a second copy of the grammar.

## What This Plugin Does

- Parses **`![alt](src)`** heads (§7.1) — the `!` tag is mandatory and the plain
  CommonMark image is the minimum, not a special case
- Reads both slots with the shared slot grammar (§5.2), so `bracket.quote`,
  `bracket.tick`, `paren.quote` and `paren.tick` are kept as bare fields instead
  of dropped
- Applies the element preset `bracket.bare → alt`, `paren.bare → src` (§6.5)
- Attaches an adjacent `{…}` / `@@type{…}` extras head to the `<img>` (§6.3)
- Reports a head item that names `alt` or `src` as an `Error` naming
  `field.override.alt` / `field.override.src` (D-12 rule 5)
- Emits a **block** `<img>` when the head owns its line and an inline one inside
  the paragraph when it does not
- Fronts a custom Solid component per `@@type{…}` marker (§11)
- Declares its contract, preset and owned fields as data (`pub const HEAD`,
  `crates/extra`'s `ElementHead`) — the single copy of §7.1's table

## What This Plugin Does **Not** Do (§14, D-16)

The whole marker run is gone, and none of it was deprecated first:

| Removed                                                | Today                                                                             |
| :----------------------------------------------------- | :-------------------------------------------------------------------------------- |
| `?` (lazy) / `~` (async) flags                         | extras `loading: "lazy"` / `decoding: "async"`                                    |
| `w<digits>` / `h<digits>`                              | extras `width:` / `height:`                                                       |
| `!!` figure mode, caption, `<figure>` / `<figcaption>` | text after the head is paragraph content; a wrapper is the component's own markup |
| `p!` / `d!` container prefixes                         | a component on the `img` layer renders its own wrapper                            |
| `[task.image.custom.figure]`                           | `[task.image.custom.img]` — the `figure` layer no longer loads (§13)              |

`~?!![](src)` therefore renders the text `~?!` followed by an empty-`alt` image:
only the `!` that touches `[` claims a head.

## Recommended Plugin Order

```text
micromatter,image,cite,wiki,anchor,markdown
```

**Why this order matters:**

1. `micromatter` must run first if frontmatter is present
2. `image` runs before markdown so its heads become `<img>` nodes instead of
   CommonMark images, and so their extras are not lost
3. `cite`, `wiki` and `anchor` process the text around an image; an image head
   never contains children, so nothing is nested inside it
4. `markdown` runs last for every remaining inline construct

Example task config:

```toml
[[task]]
input = "./src/[slug].md"
output = "./out/[slug].jsx"
plugin = "micromatter,image,cite,wiki,anchor,markdown"
format = "solid"
```

## Image Syntax

### Basic image

```md
![Alt text](https://example.com/image.webp)
```

### With extras

```md
![Alt text](https://example.com/image.webp){.hero, #cover, width: "800", loading: "lazy"}
![Alt text](https://example.com/image.webp)@@imageX{con: "jux"}
```

### Inline, with text around it

```md
Ad ex tempor ![Alt text](https://example.com/image.webp) consectetur.
```

## The slot head (§7.1, §6.5)

The tag is part of the syntax: only the `!` that touches `[` claims the head, so
`!![](src)` is one literal `!` and an image. Both slots are read by the shared
grammar (§5.2), and the preset assigns two of the six positions:

| position       | preset name | lands as |
| :------------- | :---------- | :------- |
| `bracket.bare` | `alt`       | `alt`    |
| `paren.bare`   | `src`       | `src`    |

The other four are published as attributes under their slot-qualified names, so a
content is never dropped (D-7); a `field.<slot>.<kind>` key in the entry moves a
position to a field of the author's own and voids the two preset rows for that
entry (§6.1, §6.5).

`alt` and `src` are **owned**: `<img>` writes them itself, so an extras item that
spells one is an `Error` rather than a silent override (D-12 rule 5).

## Extra Attributes

An extras head may be attached immediately after the closing parenthesis:

```md
![Cover](https://example.com/a.webp){class: "hero", id: "cover", --rotate: "5deg"}
![Cover](https://example.com/a.webp)@@imageX{.hero, #cover, data-track: "hero"}
```

The head is a single `{…}` block; the `@@type` prefix is optional and, when
present, is the §11 routing marker of the instance. `.class` items accumulate in
the `class` list, `#id` (or a positional `` `slug` ``) sets the element `id`, and
`key: "value"` items set arbitrary attributes. `--x: v` folds into the `style`
attribute (§6.3).

Special handling (§6.2, D-12):

- The **construct head wins**: an item that fills a position the preset already
  filled is a `Warning` and is dropped (`![a "B"](u)@@i{"C"}` keeps `B`)
- An item that names a field this element writes itself (`alt`, `src`) is an
  `Error` naming `field.override.<field>`
- `.class` items accumulate; `#id` wins the `id`, and a positional `` `slug` ``
  is the `id` **fallback** when no `#id` exists
- Every other key is an ordinary prop: a component receives it verbatim, the
  built-in `<img>` fallback carries it under the `data-` prefix

The head must touch the closing `)` (no space, and no space before its own `{`):
`![a](u) {.x}` and `![a](u)@@i {.x}` both stay literal text. The retired
`[.class,#id]{key: value}` block is literal text too.

## Block or inline

- A paragraph whose whole content is a head becomes a **block** `<img>`: the
  element replaces the paragraph.
- A head that shares its line with text stays **inline** inside the paragraph.
  The text around it is ordinary content and is processed by the rest of the
  chain, so an inline plugin there still runs.

## Custom Solid Component

Replace the `<img>` with a custom Solid component:

```toml
[task.image.custom.img]
name = "AdvancedImage"
imports = "import AdvancedImage from '@comp/AdvancedImage'"
template = "<AdvancedImage {...attrs} />"
```

`imports` also accepts a raw import line
(`imports = ["import AdvancedImage from '@comp/AdvancedImage';"]`). Entries coming
from the same module are merged into a single `import` statement. Several entries
address one `type` each:

```toml
[[task.image.custom.img]]
type = ["imageX", "thumb"]
name = "ImageThumb"
template = "<ImageThumb {...attrs} />"
```

### Available Template Attributes

| Attribute       | Description                                          |
| :-------------- | :--------------------------------------------------- |
| `{attrs.alt}`   | `bracket.bare` — the alternative text                |
| `{attrs.src}`   | `paren.bare` — the image source                      |
| `{attrs.class}` | Class list from the extras head (if provided)        |
| `{attrs.id}`    | `#id`, or the extras slug when no `#id` exists       |
| `{attrs.type}`  | The `@@type{…}` marker, so an entry can read it back |

Every other extras key reaches the template as a prop of the same name; a dotted
field (`bracket.quote`) arrives as the §6.3 one-key spread. The element has no
children, so a template needs no `{children}` token.

### Default HTML Output (No Custom Node)

```html
<img alt="Cover" class="hero" src="https://example.com/a.webp" />
```

## Specs (§6.1, D-5)

An entry renames the §6.1 positional slots through its `field.*` keys —
`field.specs.tick` / `field.specs.quote` for a `` `slug` `` / `"title"` item, and
`field.specs.class` / `.id` / `.style` for a `.x` / `#x` / `style:` item. An
assignment **moves** the position: the assigned path is the author's own public
field, which this element neither reads nor renames (`Specs::element_path`
answers `None`, §6.1). These keys do not void the slot preset — only a
`field.<slot>.<kind>` key does (§6.5, D-7). The table is in
[`docs/spec/SYNTAX.md` §6.1](../../docs/spec/SYNTAX.md).

## Behavioral Notes

- This plugin processes **raw text events** — it must run before `plugin-markdown`
- A `!` that is not a head (a lone `!`, the first `!` of a retired `!!`) stays
  text
- A head written inside a **code span** (`a` ![a](b) `b`) stays text: the span is
  consumed verbatim, so documentation about the syntax does not become images
- `%%numberType` attaches to the image like to every other element: it becomes the
  `__number` transport `plugin-number` reads (§10.5)
- A malformed slot (`![a "b](/x)`) is an `Error` and stays literal text (§4.3)
- An unterminated extras head after a complete one stays literal text (§4.3)

## Scope and Limitations

- Reference-style images (`![alt][ref]`) and nested brackets are not supported
- Parens inside the paren group must be balanced for correct parsing
- The plugin does not validate URLs, resolve `srcset` or check for broken links —
  a component that needs those patterns renders them itself
- Empty `alt` / `src` are legal (D-2): the document decides what a sourceless
  image means

## License

MIT
