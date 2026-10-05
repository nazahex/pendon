# Syntax Spec: Unified Typed Extras

Status: Frozen (target grammar, Phase 0 in progress — `crates/extra` v2 landed)

Supersedes (grammar only): `docs/rfc/unified-syntax.md`, `docs/rfc/table.md`,
`docs/rfc/micomatter.md` (extras fragments only).

Normative keywords **MUST**, **MUST NOT**, **SHOULD**, **MAY** follow RFC 2119.

## 1. Scope

This document is the single normative reference for Pendon's _surface_ markup
syntax. It fixes:

1. the **typed extras head** (`@@type{…}`) shared by every construct;
2. the **decorator line** (an extras-only line that decorates the _following_
   block; generalized to blockquote, list, code fence and paragraph from day
   one);
3. the per-construct head syntax (`[…]`, `(…)`, `{{…}}`, `::…`, `==…`, `|…|`);
4. escaping and the **literal fallback** rule (when Pendon MUST _not_ treat
   text as markup);
5. the mapping from syntax to the event IR (`crates/core/src/event.rs`) and to
   per-plugin custom-component config.

Implementation units that own this grammar:

| Unit                                                         | Owns                                                            |
| ------------------------------------------------------------ | --------------------------------------------------------------- |
| `crates/extra`                                               | extras scanners, `AttrValue`, `ExtrasHead`, `parse_type_marker` |
| `crates/plugin-marker`                                       | `{{type}}` inline/block markers                                 |
| `crates/plugin-directive`                                    | `::type` inline and `==type` block directives                   |
| `crates/plugin-list`                                         | list + item extras binding                                      |
| `crates/plugin-blockquote`                                   | blockquote extras binding                                       |
| `crates/plugin-table`                                        | table head, declaration row, cell/row/section extras            |
| `crates/plugin-img`, `-anchor`, `-cite`, `-heading`, `-wiki` | their own head + extras                                         |
| `crates/plugin-custom`                                       | component rendering (`node = "Component"`)                      |

### Non-goals

- `plugin-wiki` infobox internals (reworked separately).
- `plugin-quiz` (retired here, see §12).
- Renderer template semantics beyond attribute mapping (§6).

## 2. Terminology

- **head** — the construct-specific prefix that carries the construct's own
  identity: `[slug]`, `("title")`, `[[…]]`, `[^^](…)`, `~?!…(…)`, `|-…-|`.
- **extras** — the shared `@@type{…}` block: unordered positional values
  (`` `slug` ``, `"title"`), classes, id, CSS variables, props and flags.
- **decorator** — an extras head that stands alone and applies to the next
  block instead of to an inline construct.
- **marker** — `{{type}}`, an always-typed, extras-carrying node with no
  construct-specific head.
- **directive** — `::type…::` (inline) and `==type…==` (block) containers.
- **layer** — a node kind a plugin can render (e.g. `table`, `caption`,
  `thead`, `row`, `cell` for tables; `list`, `unordered`, `ordered` for lists).
- **typed extras** — the union of head + extras, converted to attributes.

## 3. Sigil inventory

| Token                       | Meaning                            | Owner               | Status                                   |
| --------------------------- | ---------------------------------- | ------------------- | ---------------------------------------- |
| `@@type`                    | extras head type marker            | `crates/extra`      | new                                      |
| `@@{…}`                     | anonymous extras head              | `crates/extra`      | new                                      |
| `{…}`                       | extras body                        | `crates/extra`      | new                                      |
| `` `x` `` / `"x"`           | positional slug / title            | `crates/extra`      | new                                      |
| `.x` `#x` `--x:v` `k:v` `k` | class / id / CSS var / prop / flag | `crates/extra`      | extended                                 |
| `[.c,#id]{k:v}`             | legacy extras fragment             | `crates/extra`      | deprecated (removed Phase 1)             |
| `{{type}}`                  | inline & block marker              | `plugin-marker`     | new (freed from `cite.section`)          |
| `::type` … `::`             | inline directive, 2–7 colons       | `plugin-directive`  | new                                      |
| `==type` … `==`             | block directive                    | `plugin-directive`  | new                                      |
| `[k]` `("k")`               | construct head (slug / title)      | per-plugin          | existing                                 |
| `\|-…-\|`                   | table declaration line             | `plugin-table`      | new                                      |
| `\|\| cap \|\|`             | table caption line                 | `plugin-table`      | new                                      |
| `\|===\|`                   | table foot separator               | `plugin-table`      | existing (kept)                          |
| `~?!` / `~?!!`              | image / figure marker              | `plugin-img`        | existing                                 |
| `[^^](…)`                   | citation                           | `plugin-cite`       | existing                                 |
| `[[…]]`                     | wiki link                          | `plugin-wiki`       | existing                                 |
| `>`                         | blockquote                         | `plugin-blockquote` | existing (+extras)                       |
| `-` `*` `+`                 | unordered list item                | `plugin-list`       | existing (+extras)                       |
| `1.`                        | ordered list item                  | `plugin-list`       | existing (+extras)                       |
| `:::name`                   | legacy custom block                | `plugin-custom`     | superseded by `==type`                   |
| `o.`                        | ordered list offset                | —                   | **retired**                              |
| `===` (bare, below table)   | legacy table foot separator        | —                   | **retired**                              |
| `{{section}}`               | legacy section marker              | —                   | **retired** (with `[task.cite.section]`) |
| `@@`                        | _no other meaning_                 | —                   | reserved for Pendon                      |

`@@` is reserved: no plugin may claim it for another purpose.

## 4. Lexical rules

### 4.1 Adjacency

An extras head MUST be adjacent to the construct it belongs to, with **no
whitespace** between the construct and `@@`:

```text
~?!h300w800[alt](url)@@type{`slug`, .hero}   correct
~?!h300w800[alt](url) @@type{`slug`}         wrong → literal text
```

Whitespace _inside_ `{…}` is free. Exception: list, blockquote and decorator
forms (§9) MAY be written on their own line.

### 4.2 Escaping

A backslash immediately before an opening sigil removes that sigil's meaning.
The backslash is consumed and MUST NOT be rendered.

```text
\@@type{…}   → literal text  @@type{…}
\{{type}}    → literal text  {{type}}
\===         → literal text  ===
```

Escapable sigils: `@@`, `{`, `}`, `{{`, `::`, `==`, `` ` ``, `[`, `]`, `(`, `)`,
`#`, `.`, `--`, `|`, `~`, `\`. A backslash before any other character is
literal (and preserved). Inside inline code spans and code blocks no sigil is
processed at all.

### 4.3 Literal fallback

A malformed head MUST fall back to literal text; it MUST NOT abort the build:

- `@@type{` without a closing `}` on the same line → literal text;
- `@@` followed by a character that is not an ASCII letter and not `{` →
  literal text;
- extras whose type is missing and whose body is empty (`@@{}`) → literal text;
- an item that is _malformed_ (unterminated `` ` `` or quote, a trailing
  character after a quoted value, an empty `key:` value, an invalid `key`)
  makes the **whole head** literal text — items are never dropped silently;
- a head separated from its construct by whitespace → literal text.

### 4.4 Type names

```ebnf
type = ALPHA ( ALPHA | DIGIT )*
```

ASCII only, case-sensitive, no space, no `-`, no `_`, no other symbols
(`fooBar` is valid, `foo-bar` is not). A non-matching name makes the whole head
literal text. Unicode lookalikes (`＠`, `｛`, `．`) MUST NOT be treated as
sigils. Widening this charset is a breaking change and requires a spec update.

## 5. Extras head grammar

```ebnf
extras-head = "@@" [ type ] "{" [ ws items ] ws "}"
items       = item { ws "," ws item } [ ws "," ]
item        = slug | title | class | id | cssvar | prop | flag
slug        = "`" { char } "`"
title       = dquote | squote
class       = "." name
id          = "#" name
cssvar      = "--" css-name ws ":" ws value
prop        = key ws ":" ws value
flag        = key
value       = dquote | squote | bquote | number | bool | bare
bquote      = "`" { char } "`"
key         = ( ALPHA | "_" ) { ALPHA | DIGIT | "_" | "-" }
css-name    = key
name        = 1*( ALPHA | DIGIT | "_" | "-" | ":" )
ws          = 1*( " " | "\t" )
```

- Item order is free; extras are completely optional.
- Duplicate handling: `class` accumulates; `id`, `slug`, `title`, `prop`,
  `flag` and `cssvar` are last-wins; a duplicate `id` emits `Severity::Warning`.
  A duplicate key collapses to one attribute (§6.4), and a `class:` prop
  accumulates into `class` exactly like a `.x` item.
- `value` classification (produces `AttrValue`, §6.3): quoted or backticked →
  `Str`; `true`/`false` → `Bool`; integer → `Int`; decimal → `Float`; otherwise
  `Raw`. A quoted value is unquoted on the way in; `Int`/`Float`/`Bool` render
  back to the authored literal text.
- An empty item (`a,,b`, leading/trailing comma) is ignored.
- Escape inside a value: `\,` `\"` `\'` `\` ``\\`.

### 5.1 Worked example

```text
@@type{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar",
      bar: 12, isFoo: true, --style-var: "2rem", isBar}
```

> The head is shown wrapped for readability only: a head is always a single
> line (§4.3). This exact example is asserted by
> `crates/extra/tests/extras_spec.rs::spec_worked_example`.

| item                  | kind   | attribute               |
| --------------------- | ------ | ----------------------- |
| `` `slug-foo` ``      | slug   | `slug = "slug-foo"`     |
| `"Title Foo"`         | title  | `title = "Title Foo"`   |
| `.extra` `.class`     | class  | `class = "extra class"` |
| `#id`                 | id     | `id = "id"`             |
| `foo: "bar"`          | prop   | `foo = "bar"`           |
| `bar: 12`             | prop   | `bar = 12` (Int)        |
| `isFoo: true`         | prop   | `isFoo = true` (Bool)   |
| `--style-var: "2rem"` | cssvar | merged into `style`     |
| `isBar`               | flag   | bare attribute `isBar`  |

## 6. Extras → attribute mapping

### 6.1 Positional names

Positional items are named through per-component config (§11):

| item      | config key     | default | meaning         |
| --------- | -------------- | ------- | --------------- |
| `` `x` `` | `backtick_key` | `slug`  | construct slug  |
| `"x"`     | `quote_key`    | `title` | construct title |

Directive **heads** keep their own keys: `bracket_key` (default `slug`) for
`[x]`, `parentheses_key` (default `title`) for `("x")`.

### 6.2 head vs extras priority

When a construct head and an extras positional fill the same slot, **the head
wins**; the extras value is dropped silently (a `Severity::Warning` MAY be
emitted):

```text
[Foo](url "Head title")@@type{"Extras title"}   → title = "Head title"
###[slug-a]("Head")@@type{`slug-b`} Foo         → id = "slug-a", title = "Head"
```

Additionally, a plugin that owns an element id (`heading`, `img`, `anchor`,
`cite`, table/cell layers) MUST use `slug` as `id` when no explicit `#id` is
present: `#id` > head slug > extras slug.

### 6.3 Value typing (IR mapping)

`crates/extra` exposes:

```rust
enum AttrValue { Str(String), Int(i64), Float(f64), Bool(bool), Raw(String) }
```

Mapping to the event IR (`Event::Attribute { name, value: String }`):

- `Str` / `Raw` / `Int` / `Float` → the literal text (no quotes) in `value`;
  number literals keep their authored shape, so `6.0` stays `6.0` and `12`
  stays `12`;
- `Bool` prop (`k: true`) → `value = "true"`;
- **flag** (`k` bare) → `Event::AttributeFlag { name }` (new variant, §13
  OPEN-IR-1);
- `--x: v` → merged into a single `style` attribute, `--x: v; --y: w;` and
  merged with an explicit `style:` prop. Raw `--x` keys MUST NOT be emitted as
  attributes. `v` is unquoted like any other value, so `--tone: "red"` becomes
  `--tone: red`;

Renderers:

| Renderer                 | flag                               | typed string | Int/Float                     |
| ------------------------ | ---------------------------------- | ------------ | ----------------------------- |
| `renderer-solid`         | `k` (bare JSX shorthand, verbatim) | `k="v"`      | `k="12"` (fallback path only) |
| `renderer-html`          | `k`                                | `k="v"`      | `k="12"`                      |
| `renderer-json` / `-ast` | `{ "name": "k", "kind": "flag" }`  | string       | string                        |

Custom components receive the **typed** form through the `attrs` map
(`{...attrs}` spread), because that path serialises `AttrValue` as JSON
(`12`, `true`) rather than as text.

> Rationale: bare flags render exactly as authored (`isFoo`) in every renderer;
> the DOM fallback path stays stringly-typed by design.

### 6.4 Emission order (deterministic)

Attributes MUST be emitted in this canonical order so golden fixtures are
stable:

1. `class` (single joined value, head classes then `.x` items in source order);
2. `id`;
3. positional keys (`slug`, then `title`, using `backtick_key`/`quote_key`);
4. all remaining props and flags in source order, with `style` where the first
   `style:` prop or `--var` item appeared.

Duplicates collapse to one attribute: the position of the first occurrence
wins, the value of the last occurrence wins. `class` accumulates instead
(a `class:` prop joins the `.x` items). A positional item and a same-named prop
are such a duplicate: the positional slot keeps its place and the later value
wins. Whitespace-only differences MUST NOT change the output.

## 7. Construct grammars

Notation: `head?` = optional, `extras?` = optional `@@type{…}`, `∘` = no space.

### 7.1 Image and figure (`plugin-img`)

```ebnf
image  = "~?" "?"? size* "[" alt "]" "(" url title? ")" extras? text?
size   = ( "w" | "h" ) DIGIT+
```

- `~?!` → `Image` node; `~?!!` → `Image` wrapped in a `Figure`.
- Extras attach to the **outermost** node (`Figure` when `~?!!`, else `Image`).
- `w800` / `h300` map to `width` / `height` on the inner image (unchanged).
- `#id`/`slug` set the outer node id; `class` goes to the outer node.
- Trailing text on the same line is the figure caption (`children` of `Figure`).
- Legacy `[.c,#id]{k:v}` after `(…)` is deprecated; Phase 1 removes it.

### 7.2 Anchor (`plugin-anchor`)

```ebnf
anchor = "[" text "]" "(" url title? ")" extras?
```

- Extras attach to the `<a>` element.
- The head `("title")` wins over extras `"…"` (§6.2).
- URL modifier suffixes stay part of the URL token and are parsed **before**
  extras (unchanged behaviour): `^` → `target="_blank"`, `~` → `target="_self"`,
  `!` → `rel="nofollow"`, `$` → `rel="sponsored"`, `;;` → `rel="ugc"`,
  `--` → `rel="noreferrer"`; conflicting `^`/`~` is last-wins with a
  Warning. `rel:`/`target:` extras merge with (do not replace) the modifier
  result, as today.
- `slug` → `id` when `#id` is absent.
- Legacy `[.c,#id]{k:v}` after `)` is deprecated; Phase 1 removes it.

### 7.3 Cite (`plugin-cite`)

```ebnf
cite = "[^^]" "(" ref [ "," loc ] ")" extras?
```

- First head positional → `id`; second (or `loc=`) → `loc`.
- Extras merge into the citation node; head values win.
- `{{section}}` and `[task.cite.section]` are **gone** (removed in Phase 1;
  §12, §14). The bibliography section returns as a `plugin-marker` block
  (Phase 3).

### 7.4 Heading (`plugin-heading`)

```ebnf
heading = "#"(1..6) [ "[" slug "]" ] [ "(" title ")" ] extras? text
```

- `[slug]` and `("title")` are both optional and independent.
- `("title")` is **not** the `innerText`; it is a separate attribute.
- Whitespace between the attr block and the heading text is consumed by the
  block (it MUST NOT become part of the title).
- Extras attach to the heading element; extras `slug` is ignored when `[slug]`
  exists.

### 7.5 Wiki (`plugin-wiki`)

```ebnf
wikilink = "[[" target [ "|" label ] "]]" extras?
```

- Unchanged apart from adjacent extras support (`[[Anim Esta]]@@type{…}`).
- Extras attach to the wiki `<a>`; `href` is produced by the plugin
  (`link_prefix`) and MUST NOT be overridable by extras (it wins).
- Infobox syntax is out of scope for this spec.

## 8. Table (`plugin-table`)

```ebnf
table       = decl? caption? header-row row* [ foot ]
decl        = "|-" [ head ] extras? ["-"] "|"        # declares table extras
caption     = "||" extras? content "||"
header-row  = "|" cell ( "|" cell )* "|" extras?     # alignment/width row
row         = "|" cell ( "|" cell )* "|" extras?      # body row
foot        = ("|===|" | "===") extras? row*
cell        = "|" extras? content
```

Mapping to table layers:

| Source                               | Layer / node                                 |
| ------------------------------------ | -------------------------------------------- |
| `decl` line                          | `<table>` (`table`)                          |
| extras after closing `-\|`           | `<table>`                                    |
| `\|\| … \|\|` line                   | `<caption>` (`caption`)                      |
| alignment-row cell extras            | the `<th>` of that column (`thead` / column) |
| alignment-row **end-of-line** extras | `<tbody>` (`tbody`)                          |
| body-row cell-front extras           | that `<td>` (`cell`)                         |
| body-row end-of-line extras          | that `<tr>` (`row`)                          |
| `\|===\|` line extras                | `<tfoot>` (`tfoot`)                          |
| foot-row cell-front extras           | that `<td>` (`cell`)                         |
| foot-row end-of-line extras          | that `<tr>` (`row`)                          |

Rules:

- The declaration line is **childless**: any non-extras/head text inside it is
  discarded (a `Severity::Warning` MAY be emitted).
- The declaration line is optional; when absent the table is untyped.
- `head` inside `decl` uses the standard `[slug]` / `("title")` form:
  `|-[slug-foo]@@type{…}-|`.
- Legacy bare `===` as a foot separator is **retired**; only `|===|` remains.
- Alignment/width tokens (`:---:`, `(200px)`) and the `>` (colspan `+1`) / `^`
  (rowspan `+1`) cell markers are unchanged and are parsed **before** extras.
- Cell content may be empty; empty cells become `<td></td>`.

## 9. Decorators, blockquotes and lists

### 9.1 The decorator rule (general)

A **decorator line** is a line whose entire content is `@@type{…}` or `@@{…}`
(optional leading indentation, no trailing text other than whitespace). It
decorates the **next block node** at the same nesting level and is consumed
(MUST NOT be rendered):

````text
@@{`intro`, .lead}
First paragraph with extras on the paragraph node.

@@aside{"Aside title"}
> quoted text                 ← decorates the blockquote

@@codeBlock{`listing-1`}
```rust                        ← decorates the code fence node

@@{`my-list`, .compact}
- Alpha                       ← decorates the list container
- Beta
````

- The decorator's `type` (when present) is the **type marker** of the decorated
  node and drives per-type component selection (§11).
- A decorator applies to a blockquote, list, code fence, paragraph, table or
  heading. Anything else (inline constructs) uses the adjacent form (§4.1).
- If no following block exists (end of input, or the next sibling is not a
  block), the decorator is dropped with a `Severity::Warning`.
- Consecutive decorators: only the **last** one applies; earlier ones are
  dropped with a `Severity::Warning` (MAY be an error later).
- Indentation MUST be `<=` the indentation of the decorated block; a decorator
  indented deeper than the following block is literal text.

### 9.2 Blockquote (`plugin-blockquote`)

```ebnf
blockquote = ">" ws [ extras ] content
```

- Extras written immediately inside the quote (`> @@type{…} text`) attach to the
  **blockquote** node, not to the paragraph inside it.
- A decorator line directly above `>` attaches to the blockquote as well; when
  both exist the inner one wins.
- Nested quotes follow the same rule for their own node.

### 9.3 List (`plugin-list`)

Design A: extras are carried by **decorator lines**, never by a swallowed
carrier list.

```ebnf
list-decorator = "@@" type? "{" … "}"                 # own line, above the list
item           = marker ws [ item-decorator ws ] content
item-decorator = "@@" type? "{" … "}"
marker         = "-" | "*" | "+" | DIGIT+ "."
```

Rules:

- **L1 (list container).** A decorator line directly above a list decorates the
  container: layer `unordered` for `-`/`*`/`+`, layer `ordered` for `1.`.
- **L2 (item).** An item-decorator at the very start of an item's content
  decorates that `li`; the remainder of the line is the item's first block.

  ```text
  - @@liItem{`alpha`} Alpha text
    - Nested item
  ```

- **L3 (block inside item).** A decorator line on its own, as the _second_ line
  of an item, decorates the following block inside that `li`.
- **L4 (`o.` retired).** List start offsets use the first item's number only
  (CommonMark behaviour): `6. Goo` starts the ordered list at 6.
- **L5 (nesting).** Nested lists follow L1–L3 at their own indentation; a
  nested list's decorator must be indented to the nested list's level.
- Ordered and unordered markers may not be mixed inside one list.
- A "task list" checkbox (`- [ ]`) is content, not a marker; the item extras
  come after the checkbox.

### 9.4 Layer mapping for lists

| Layer       | Element          | Matched by                                          |
| ----------- | ---------------- | --------------------------------------------------- |
| `unordered` | `<ul>` container | container decorator `type` (L1), marker `-`/`*`/`+` |
| `ordered`   | `<ol>` container | container decorator `type` (L1), marker `1.`        |
| `list`      | `<li>` item      | item-decorator `type` (L2)                          |

Layer keys mirror `task.table.<layer>.custom`; `list` is the layer named after
the plugin and owns the **item** element (`<li>`), while `unordered`/`ordered`
own the containers (`<ul>`/`<ol>`). Config: `[task.list.list.custom]`,
`[task.list.unordered.custom]`, `[task.list.ordered.custom]`.

`start` for `ol` comes from the first item's number and MUST NOT be
overridable by extras (`start:` prop is rejected with a `Severity::Warning`).

## 10. Markers and directives

### 10.1 Marker (`plugin-marker`)

```ebnf
marker-inline = "{{" type "}}" extras?            # inside inline content
marker-block  = "{{" type "}}" extras?            # own line (block context)
```

- `type` is **mandatory** for markers; `{{}}` is literal text.
- Extras are optional but must be adjacent.
- Inline form emits an inline node; block form emits a block node.
- The block form carries the same line's trailing text as its `children` and
  does not absorb the following block.
- With no custom component, the inline form renders `<span>` and the block form
  renders `<div>` (both without children).
- The `{{…}}` sigil is free: `[task.cite.section]` and the legacy
  `{{section}}` usage are removed (§12).

### 10.2 Inline directive (`plugin-directive`)

```ebnf
inline-directive = colons type [ "[" bracket "]" ] [ "(" paren ")" ] extras? text colons
colons           = "::" up to ":::::::" (2..7)
```

- `[bracket]` key = `bracket_key` (default `slug`), `("paren")` key =
  `parentheses_key` (default `title`).
- The closing run MUST be `>=` the opening run (nesting depth = colon count).
  An inner directive with fewer colons closes implicitly at the outer's close.
- Content is parsed as inline content.
- No custom component → `<span>`.

### 10.3 Block directive (`plugin-directive`)

```ebnf
block-block  = fences type [ "[" bracket "]" ] [ "(" paren ")" ] extras? body fences
fences       = "==" up to "=======" (2..7)
close        = fences (alone on its line)
```

- A fence **with** a type opens a directive; a fence **alone** closes the
  innermost open directive (LIFO; the width need not match, SHOULD match).
- A bare fence with no directive open is literal text. There is no way to open
  an anonymous directive: use a typed one (e.g. `==note`) — its fallback
  renderer is `<div>`.
- Content is parsed as block content; directives nest to 3 levels (`==`, `===`,
  `====`) and MAY nest deeper (limit 7).
- Unclosed at end of input → closed implicitly with a `Severity::Warning`.

### 10.4 Priority summary

| Slot                     | Winner                                                |
| ------------------------ | ----------------------------------------------------- |
| element `id`             | `#id` > head slug > extras slug                       |
| `title`                  | head `("…")` > extras `"…"`                           |
| `class`                  | union of head classes and `.x` items, in source order |
| `href` / `src` / `start` | construct-owned, never overridable                    |
| any other key            | head prop > extras prop                               |

## 11. Custom component config

Every plugin that owns one or more **layers** exposes component sets:

```toml
# primary layer: array of tables
[[task.anchor.custom]]
type = ["anchorA", "anchorB"] # string or array of strings
name = "AnchorAB"
imports = ["import { AnchorAB } from '@comp/shared/Anchor'"]
template = "<AnchorAB type={\"{attrs.type}\"} {...attrs}>{children}</AnchorAB>"

[[task.anchor.custom]] # empty/absent `type` = default
name = "AnchorDefault"
imports = ["import { AnchorDefault } from '@comp/shared/Anchor'"]
template = "<AnchorDefault {...attrs}>{children}</AnchorDefault>"

# layered plugins: one `custom` key per layer
[task.table.custom.table] # single table = that layer's default
[task.table.custom.caption]
[[task.table.custom.thead]] # array of tables = typed + default
[[task.table.custom.tbody]]
[[task.table.custom.tfoot]]
[[task.table.custom.row]]
[[task.table.custom.cell]]

[task.list.custom.list] # <li> item
[task.list.custom.unordered] # <ul> container
[task.list.custom.ordered] # <ol> container

[task.img.custom.img] # <img>
[task.img.custom.figure] # <figure>
[task.wiki.custom.anchor] # <a> produced by plugin-wiki
[task.wiki.custom.infobox] # infobox
```

A plugin that declares **one** layer may address that layer directly, using the
plugin's own key as the layer name (the _primary layer_): `[[task.anchor.custom]]`
above is shorthand for `[[task.anchor.custom.anchor]]`. A bare
`task.<plugin>.custom` is also accepted when the table holds one entry
(`[task.anchor.custom]` with `name`/`template`) instead of a map of layers.

Rules:

1. A layer key MAY be a **single table** (that layer's default component) or an
   **array of tables** (`[[task.table.custom.thead]]`, typed entries plus at
   most one default). Both forms are equivalent to "the default component for
   that layer". The layer name is the element the layer emits (`table`, `thead`,
   `caption`, `img`, `figure`, `list`, `unordered`, `ordered`, …).
2. `type` accepts a single string or an array; `[]` is equivalent to an absent
   `type`. At most one default per layer; two defaults is a **hard error** at
   config load.
3. Unmatched type selection order: exact `type` match → layer default → built-in
   fallback node (which MUST still carry all extras as attributes).

   > Until a plugin's §11 cutover lands (OPEN-C3) the CLI requires a layer to be
   > **routable**: exactly one component, or a layer default that no `type` entry
   > competes with. A layer with several `type` entries is a config error rather
   > than a silent fall back to the default.
4. `imports` is canonical. The legacy singular `import` is accepted with a
   deprecation `Severity::Warning` (see §14 OPEN-C2).
5. Positional naming (per component, all layers): `backtick_key`, `quote_key`
   (§6.1). Directive heads additionally use `bracket_key`, `parentheses_key`.
6. `template` validation is a **hard error** when:
   - a non-void element is opened (`<X …>`) but the template contains neither a
     `{children}` nor a `{text}` token — children would be silently dropped; or
   - the template's element is unbalanced.
     Self-closing templates (`<X … />`) are exempt (they declare a childless
     node, e.g. markers and cite leaves).
7. `{attrs.key}` interpolation stays as implemented in `plugin-custom`.

## 12. Pipeline strategy

Pendon's pipeline runs pre-markdown plugins (text → text) and post-markdown
plugins (events → events). The new syntax therefore needs two stages:

### 12.1 Pre-markdown (protect)

Every construct parser strips its head + extras from the source text and
replaces them with a **sentinel** so the removed information survives
micromatter/markdown untouched. Sentinels are line-scoped and self-delimiting:

```text
<U+E000>pendon:1:<payload><U+E000>
```

- `U+E000` (Private Use Area) MUST NOT be produced by any other stage; a literal
  `U+E000` in the source is stripped and a `Severity::Warning` emitted.
- `<payload>` is a URL-safe base64/JSON encoding of the parsed extras +
  construct metadata, so no escaping of `"`/`}`/`|` inside the payload is
  needed.
- HTML comments are **not** used, because `markdown_strip_comments = true`
  removes them.
- Sentinels are emitted as a plain paragraph/HTML-level token that markdown
  passes through unchanged.

### 12.2 Post-markdown (bind)

A single binder pass (shared by `plugin-list`, `plugin-blockquote`,
`plugin-directive`, `plugin-table`, `plugin-marker` and decorators) walks the
event stream once:

1. Resolves sentinels back into `ExtrasHead` values.
2. Attaches them to the correct target node:
   - _next-block_ decorators → the next `StartNode` at the same nesting level;
   - _construct_ sentinels (img, anchor, cite, heading, wiki, table layers,
     markers, directives) → the sentinel's own position.
3. Removes the sentinel nodes from the output.
4. Emits diagnostics for dangling sentinels (§13).

This keeps the markdown plugin unaware of Pendon extras and avoids
per-plugin text hacking.

### 12.3 Fallback paths (critical)

The built-in renderers for `BlockQuote`, `List`, `ListItem`, `CodeBlock` and
`Paragraph` currently **ignore attributes** (`renderer-solid/src/node.rs`
~L32–36, ~L83–108). This MUST be fixed: extras parsed but discarded are a
silent data-loss bug. Minimum requirement: the fallback path emits `id`,
`class` and `style`, and passes the remaining keys as attributes.

## 13. Diagnostics

| Condition                                                | Severity                 |
| -------------------------------------------------------- | ------------------------ |
| malformed head / missing `}` / whitespace-separated head | none (literal text)      |
| duplicate `#id` in one extras block                      | Warning                  |
| extras `id` overriding a head slug                       | Warning                  |
| decorator with no following block                        | Warning                  |
| dropped decorator (consecutive decorators)               | Warning                  |
| dropped extras value because a head value won            | Warning                  |
| unclosed block directive at EOF                          | Warning                  |
| template without `{children}` (non-void element)         | Error (config load)      |
| unbalanced template element                              | Error (config load)      |
| two default components for one layer                     | Error (config load)      |
| `start:` prop on `ol` extras                             | Warning (ignored)        |
| literal `U+E000` sentinel character in source            | Warning (stripped)       |
| unknown key outside a known layer                        | Warning (passed through) |

Config-load errors MUST abort the build before any file is processed.

## 14. Retirement and collisions

| Removed                       | Replaced by                          | Note                                                                                                                                                                                                                                                                                                                                                                                                           |
| ----------------------------- | ------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `[task.cite.section]`         | `plugin-marker` (`{{bibliography}}`) | **removed** in Phase 1: `CiteOptions.section` and `CitationContext::replace_section_markers` are gone. The section component returns as a marker once `plugin-marker` lands (Phase 3)                                                                                                                                                                                                                          |
| `{{section}}` in sources      | marker block                         | **removed**: the `{{ footnote }}` lines in `sandbox/cite` and `sandbox/img` are gone; `{{…}}` is free for `plugin-marker`                                                                                                                                                                                                                                                                                      |
| `:::name` custom blocks       | `==type` block directive             | keep reading `:::` for one release? §15 OPEN-R1                                                                                                                                                                                                                                                                                                                                                                |
| bare `===` table footer       | `\|===\|`                            | Phase 2                                                                                                                                                                                                                                                                                                                                                                                                        |
| `o.` ordered-list offset      | first item's number                  | Phase 3                                                                                                                                                                                                                                                                                                                                                                                                        |
| `plugin-quiz` (`:::quiz`)     | —                                    | retired from the pipeline; plugin code stays, deep rework later                                                                                                                                                                                                                                                                                                                                                |
| legacy `[.c,#id]{k:v}`        | `@@type{…}`                          | Phase 1, deprecated not silent                                                                                                                                                                                                                                                                                                                                                                                 |
| `import = "…"` (string)       | `imports = ["…"]`                    | done: accepted with a Warning (§15 OPEN-C2)                                                                                                                                                                                                                                                                                                                                                                    |
| `[task.<plugin>.custom_node]` | `[task.<plugin>.custom.<layer>]`     | **removed**: `custom.<layer>` is read for `anchor`, `img` (`figure`), `heading`, `cite` and all seven `table` layers (`apps/cli/src/plugins.rs`) and every sandbox/fixture config is migrated. The stale key is still parsed so it fails the build with a migration message instead of being ignored. Unwired layers (`task.img.custom.img`) and layers that need per-type routing (OPEN-C3) are config errors |

## 15. Open decisions

Each item records the recommendation to implement **unless the maintainer
objects**; the spec text above already assumes the recommendation.

- **OPEN-IR-1 (flags) — resolved.** `Event::AttributeFlag { name }` is in the IR
  and rendered verbatim by the AST, JSON, HTML (compact + pretty) and Solid
  renderers.
- **OPEN-L1 (item extras placement).** An item-decorator at the very start of
  an item's line decorates the `li` (L2); a decorator on its own line _inside_
  the item decorates the following block (L3).
  _Alternative:_ both always decorate the following block (then `li` extras
  need a different spelling). **Recommendation: as specified.**
- **OPEN-L2 (list layers) — resolved.** Layer keys are `list` (= `<li>` item),
  `unordered` (= `<ul>` container) and `ordered` (= `<ol>` container), mirroring
  `task.table.custom.<layer>`. Config: `[task.list.custom.list]`,
  `[task.list.custom.unordered]`, `[task.list.custom.ordered]`.
- **OPEN-B1 (bare fence).** A bare `==…==` fence only closes; anonymous block
  directives are impossible. **Recommendation: as specified** (removes the
  open/close ambiguity of `===`).
- **OPEN-C1 (config shape) — resolved.** Layered plugins use `custom.<layer>`
  (single table or array of tables); single-layer plugins use the primary layer
  shorthand (`[task.anchor.custom]`, `[[task.anchor.custom]]`). Implemented by
  the loader in `apps/cli/src/components.rs`.
- **OPEN-C2 (`import` alias) — resolved (warn).** The loader accepts the
  singular `import` and emits a Warning, then hard-errors one release later.
- **OPEN-C3 (layer routing before the plugin cutover) — resolved.** Each plugin
  keeps its single-component option struct until its own cutover, so a `custom`
  layer must stay _routable_: exactly one component, or a layer default that no
  `type` entry competes with. A layer holding several `type` entries (or a
  default next to typed entries) is a §13 config error instead of a silent
  downgrade to the default — `components::resolve_single` enforces it and
  `apps/cli/src/plugins.rs` maps the resolved entry into the existing option
  structs. Per-type routing (`type` marker → entry) lands with the plugin
  cutovers in Phases 1–3.
- **OPEN-E1 (table column extras).** Alignment-row cell extras apply to that
  column's `<th>` only. **Recommendation: yes** (cascading to `<td>`s would
  duplicate attributes on every body cell).
- **OPEN-T1 (tenant of `plugin-blockquote`/`plugin-list`).** Both plugins are
  **new crates**; the existing markdown-plugin behaviour stays as fallback when
  they are not enabled. **Recommendation: new crates.**
- **OPEN-R1 (`:::` compatibility).** Keep parsing `:::name` blocks as a
  deprecated alias of `==name` for one release (Warning), then hard-error.
  **Recommendation: alias with Warning** — `sandbox/universal`,
  `sandbox/custom` and `apps/cli/tests/fixtures` still use `:::`.
- **OPEN-S1 (sectionize/`[task.cite.section]` removal order) — resolved
  (done).** Verified `plugin-sectionize` is heading/icon based and never
  references `cite.section`, so the removal landed with the cite change:
  `CiteOptions.section`, `CitationContext::replace_section_markers`, the
  section branch of `cite::solid_hints` and the `[task.cite.section]` blocks in
  `sandbox/cite`, `sandbox/cite/custom`, `sandbox/universal` and
  `sandbox/img/regular` are gone, together with the `{{ footnote }}` lines in
  the two sources that used them.

## 16. Golden set and tests

Phase ordering says "spec first": expectations for the new grammar are frozen as
data **before** the implementation lands, so no downstream rewrite happens.

### 16.1 Fixture format

```text
docs/spec/golden/NN-name.md          input markdown
docs/spec/golden/NN-name.toml        pendon.toml for the fixture (plugins + custom sets)
docs/spec/golden/NN-name.jsx         expected solid output
docs/spec/golden/NN-name.events.json expected event IR (optional, for parser bugs)
```

Harness: `apps/cli/tests/syntax_spec.rs`, one `#[test]` per fixture, following
the existing `assert_cmd` style (`apps/cli/tests/custom_spec.rs`,
`list_render_spec.rs`) and run through a `write_project` helper in a temp dir.

### 16.2 Test gating

- Fixtures whose syntax is not implemented yet are annotated
  `#[ignore = "pending Phase N"]` so `cargo test` stays green while the
  expectation is still frozen in the repo.
- Each phase removes the `#[ignore]` annotations it makes pass. A phase is not
  complete while its fixtures are still ignored.
- `crates/extra` tests are **not** ignored: `src/value.rs` unit tests plus
  `tests/extras_spec.rs` (one case per rule of §4–§6, including the §5.1 worked
  example) land green in Phase 0 and are the primary gate for the foundation.

### 16.3 Minimum fixture list

| #  | Fixture                   | Covers                                                            |
| -- | ------------------------- | ----------------------------------------------------------------- |
| 01 | `extras-head`             | all item kinds, ordering, duplicates, escaping                    |
| 02 | `extras-literal`          | malformed heads → literal text, adjacency failures                |
| 03 | `img-figure`              | `~?!` / `~?!!` + extras + caption                                 |
| 04 | `anchor`                  | head `("title")` vs extras `"title"` priority                     |
| 05 | `cite`                    | `[^^](ref "loc")` + extras, `loc=` prop form                      |
| 06 | `heading`                 | `[slug]` + `("title")` + extras, auto-number interaction          |
| 07 | `wiki`                    | `[[…]]` + extras, `href` not overridable                          |
| 08 | `table-decl`              | `\|-…-\|`, `\|\| caption \|\|`, decl head `[slug]`                |
| 09 | `table-layers`            | column/th/td/tr/tbody/tfoot extras routing                        |
| 10 | `decorator-blocks`        | paragraph, code fence, heading decorators                         |
| 11 | `blockquote`              | inner extras + decorator above                                    |
| 12 | `list-container`          | L1 decorator → `ul`/`ol` + `start`                                |
| 13 | `list-item`               | L2 item-decorator → `li`, nesting                                 |
| 14 | `marker`                  | inline + block forms, `<span>`/`<div>` fallback                   |
| 15 | `directive-inline`        | `::…::` nesting 2..7, `bracket_key`/`parentheses_key`             |
| 16 | `directive-block`         | `==type`/`==` LIFO closing, nesting, EOF warning                  |
| 17 | `flags`                   | bare flags through solid/html/json/ast                            |
| 18 | `fallback-attrs`          | extras on list/blockquote/paragraph/code fence without components |
| 19 | `template-children-error` | `template` without `{children}` → hard error                      |
| 20 | `component-selection`     | type match → default → fallback, two defaults error               |

## 17. Migration phases

| Phase | Content                                                                                                                                                                                                                       | Gate                                                                     |
| ----- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
| 0     | `crates/extra` v2 (`AttrValue`, `ExtrasHead`, `parse_extras`, `parse_type_marker`, `parse_directive_head`), `ComponentSet` loader + `{children}` validation, `Event::AttributeFlag`, fallback-attr fix. **No syntax change.** | extras unit tests + fixtures 17/18/19/20 green; existing suite unchanged |
| 1     | Cut over `img`/`figure`, `anchor`, `cite`, `heading`, `wiki`; `[task.cite.section]` → marker; `{{…}}` freed; `imports` rename.                                                                                                | fixtures 03–07, 01, 02                                                   |
| 2     | Table rewrite: `\|-…-\|`, `\|\| caption \|\|`, cell/row/section extras, drop bare `===`.                                                                                                                                      | fixtures 08, 09                                                          |
| 3     | `plugin-blockquote`, `plugin-list`, `plugin-marker`, `plugin-directive`, decorator binder; drop `o.`.                                                                                                                         | fixtures 10–16                                                           |
| 4     | Spec/fixture/docs cleanup: RFC → archived, `sandbox/unified` rewritten, `docs/todo/syntaxes.md` trimmed.                                                                                                                      | full green, no ignored fixtures                                          |

**Phase 0 complete.** The `crates/extra` v2 parser is implemented and green
(`crates/extra/src/{value,typed}.rs`, `crates/extra/tests/extras_spec.rs`):
`AttrValue`, `ExtrasHead`, `parse_extras`, `parse_type_marker`,
`parse_directive_head`, `parse_extras_body`, `to_attributes` and
`Attrs::merge_with`. The §11 `ComponentSet` loader (three TOML shapes, typed
entries + one default per layer, `{children}`/balance template validation,
`import` deprecation) lives in `apps/cli/src/components.rs` and is wired into
every task-level plugin option builder in `apps/cli/src/plugins.rs`.
`Event::AttributeFlag` is part of the IR, and the fallback renderers keep
attributes on `Paragraph`/`Blockquote`/`BulletList`/`OrderedList`
(`crates/renderer-solid/src/node.rs`,
`crates/renderer-html/src/{compact,pretty}.rs`,
`crates/renderer-ast/src/builder.rs`). Fixtures 17–20 are covered by unit tests
in those crates plus `components::tests` and `plugins::tests`; the workspace
suite is green.

Remaining before Phase 1 lands the plugin cutovers: route `type` markers per
instance _inside_ the plugins (OPEN-C3) and let the §12 binder attach the parsed
extras to the constructs (img, anchor, cite, heading, wiki, table layers).

### 17.1 Progress after Phase 0

The §11 config cutover is **complete**:

- `custom.<layer>` is the only component-set spelling; the legacy
  `[task.<plugin>.custom_node]` key is removed and fails the build with a
  migration message (§14).
- Every sandbox config and CLI fixture is migrated; re-running all 24 sandboxes
  after the migration produced byte-identical output except for the two
  documents whose `{{ footnote }}` marker was retired (§14, `cite.section`).
- `CiteOptions.section` / `CitationContext::replace_section_markers` and the
  `[task.cite.section]` config are removed (Phase 1 item), so `{{…}}` is now
  free for `plugin-marker`.

Next: parse `@@type{…}` in `crates/plugin-{anchor,cite,heading,img,wiki}` and
attach the attributes to the construct node (fixtures 01–07).

### 17.2 Progress after the construct `@@type{…}` heads

All five construct plugins of Phase 1 read and merge the extras head:

- `plugin-anchor` (§7.2) — extras attach to the `<a>`; URL modifiers and the
  `("title")` head win, `href` is never overridable.
- `plugin-wiki` (§7.5) — extras attach to the wiki `<a>`; `href` (from
  `link_prefix`) wins.
- `plugin-heading` (§7.4) — `[slug]`, `[.class]`, `("title")` and extras;
  precedence `#id` > `[slug]` > extras slug, `class` accumulates.
- `plugin-extract-heading` (§7.4) — strips the same head from the extracted
  `text`/`id`, so the headings metadata matches the rendered heading.
- `plugin-img` (§7.1) — extras attach to the **outermost** node (`<figure>` for
  `~?!!`, else the `<img>`); `w`/`h` stay on the inner image, `--var` items
  merge into `style`.
- `plugin-cite` (§7.3) — extras merge into the citation node; the cite args
  (`loc=`) win, `#id`/`slug` feed the `cite-id` slot (the reference `id` is
  never replaced).

Shared rules implemented through `crates/extra`: `#id` > head slug > extras
slug, `class` accumulates (head first, §6.4), head wins for every other key
(dropped with a §13 Warning), bare flags become `Event::AttributeFlag`
(§6.3), and the pre-§11 `[.c,#id]{k:v}` block still works but is reported
(`legacy_extras_warning`, §14). Extras must be adjacent (§4.1); a malformed
head stays literal text (§4.3).

Still open for Phase 1: the §8 table layers (`|- … -|`, `|| caption ||`,
cell/row/section extras), the golden fixtures 01–07 (`docs/spec/golden/` +
`apps/cli/tests/syntax_spec.rs`), and per-instance `type` routing inside the
plugins (OPEN-C3).

## 18. Acceptance criteria

1. Every §3 sigil is either implemented or explicitly retired.
2. No extras data is dropped silently: any node that parses extras MUST emit
   them as attributes, with or without a custom component.
3. A `template` that would drop children fails the build at config load.
4. All §16 fixtures are green (none ignored) except fixtures explicitly
   deferred with a separate spec note.
5. Escaping and literal fallback are covered by fixtures 01/02.
6. `cargo test --workspace` is green; `sandbox/unified` produces the documented
   output.
