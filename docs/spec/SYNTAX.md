# Syntax Spec: Unified Typed Extras

Status: **Frozen normative grammar.** This file is the target reference for the
surface markup syntax and stays free of per-phase progress (that lives in
[`STATUS.md`](../../STATUS.md) and [`decisions/`](../decisions/)). Rationale for
the rules is in [ADR-0001](../decisions/0001-typed-extras-reconciliation.md).

Supersedes (grammar only): the archived `docs/rfc/{unified-syntax,table,micomatter}.md`
(extras fragments).

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
| `crates/plugin-list`                                         | list container + item extras binding (item layer L2/L3 pending) |
| `crates/plugin-blockquote`                                   | blockquote extras binding                                       |
| `crates/plugin-bind`                                         | `{{{lang[var] … }}}` data blocks + `$var` binding (§19)         |
| `crates/plugin-table`                                        | table head, declaration row, cell/row/section extras            |
| `crates/plugin-section`                                      | heading-driven `Section` outline + section decorator / markers  |
| `crates/plugin-img`, `-anchor`, `-cite`, `-heading`, `-wiki` | their own head + extras                                         |
| `crates/plugin-custom`                                       | component rendering (`node = "Component"`)                      |

### Non-goals

- `plugin-wiki` infobox internals (reworked separately).
- `plugin-quiz` (retired here, see §12).
- Renderer template semantics beyond attribute mapping (§6).

## 2. Terminology

- **head** — the construct-specific prefix that carries the construct's own
  identity: `[slug]`, `("title")`, `[[…]]`, `[^^](ref "loc")`, `~?!…(…)`, `|-…-|`.
- **extras** — the shared `{…}` head (optionally `@@`-prefixed, optionally typed):
  unordered positional values (`` `slug` ``, `"title"`), classes, id, CSS
  variables, props and flags.
- **decorator** — an extras head that stands alone and applies to the next
  block instead of to an inline construct.
- **marker** — `{{type}}`, an always-typed, extras-carrying node with no
  construct-specific head.
- **directive** — `::type…::` (inline) and `==type…==` (block) containers.
- **layer** — a node kind a plugin can render (e.g. `table`, `caption`,
  `thead`, `row`, `cell` for tables; `list`, `unordered`, `ordered` for lists).
- **typed extras** — the union of head + extras, converted to attributes.

## 3. Sigil inventory

| Token                       | Meaning                            | Owner               | Status                          |
| --------------------------- | ---------------------------------- | ------------------- | ------------------------------- |
| `{…}`                       | extras head (canonical)            | `crates/extra`      | **current**                     |
| `@@type{…}`                 | typed extras head                  | `crates/extra`      | **current**                     |
| `@@type`                    | type-only head                     | `crates/extra`      | **current**                     |
| `@@{…}`                     | untyped extras head (= `{…}`)      | `crates/extra`      | **current**                     |
| `{}` / `@@{}`               | empty head                         | `crates/extra`      | **current**                     |
| `` `x` `` / `"x"`           | positional slug / title            | `crates/extra`      | new                             |
| `.x` `#x` `--x:v` `k:v` `k` | class / id / CSS var / prop / flag | `crates/extra`      | extended                        |
| `{{type}}`                  | inline & block marker              | `plugin-marker`     | new (freed from `cite.section`) |
| `::type` … `::`             | inline directive, 2–7 colons       | `plugin-directive`  | new                             |
| `==type` … `==`             | block directive                    | `plugin-directive`  | new                             |
| `[k]` `("k")`               | construct head (slug / title)      | per-plugin          | existing                        |
| `\|-…-\|`                   | table declaration line             | `plugin-table`      | new                             |
| `\|\| cap \|\|`             | table caption line                 | `plugin-table`      | new                             |
| `\|===\|`                   | table foot separator               | `plugin-table`      | existing (kept)                 |
| `~?!` / `~?!!`              | image / figure marker              | `plugin-img`        | existing                        |
| `[^^](ref "loc")`           | citation                           | `plugin-cite`       | **current** (§7.3)              |
| `[[…]]`                     | wiki link                          | `plugin-wiki`       | existing                        |
| `>`                         | blockquote                         | `plugin-blockquote` | current (+extras)               |
| `-` `*` `+`                 | unordered list container           | `plugin-list`       | current (+extras)               |
| `1.`                        | ordered list container             | `plugin-list`       | current (+extras)               |
| `{{{lang[var] … }}}`        | data block                         | `plugin-bind`       | new (§19)                       |
| `$var`                      | bound data reference (extras only) | `plugin-bind`       | new (§19)                       |
| `$a.b[0]`                   | path into a bound value            | `plugin-bind`       | new (§19)                       |
| `...$var`                   | spread/merge inside a `{…}` value  | `plugin-bind`       | new (§19)                       |
| `{{{lang[var](../src/x)}}}` | external data block                | `plugin-bind`       | new (§19)                       |
| `:::name`                   | legacy custom block                | `plugin-custom`     | superseded by `==type`          |
| `@@`                        | _no other meaning_                 | —                   | reserved for Pendon             |

`@@` is reserved: no plugin may claim it for another purpose.

## 4. Lexical rules

### 4.1 Adjacency

An extras head MUST be adjacent to the construct it belongs to, and to its own
parts:

```text
~?!h300w800[alt](url)@@type{`slug`, .hero}   correct
[alt](url){`slug`}                           correct (bare spelling)
[alt](url)@@anchorA.                          correct: type-only head, `.` stays text
~?!h300w800[alt](url) @@type{`slug`}         wrong → literal text
~?!h300w800[alt](url)@@type {`slug`}         wrong → literal text (space before `{`)
@@type[…](…)                                 wrong → the groups must touch the type
```

Whitespace _inside_ `{…}` is free. The `@@type {…}` form (whitespace between the
type and its `{`) is **not** a head: the whole run stays literal, so a head never
silently detaches from its type. The positional groups (§6.1) follow the same
rule: `[` and `(` must touch what precedes them, so `@@type […](…)` is a
type-only head followed by literal text.

Two exceptions exist, both for block constructs whose marker must be followed by
a space anyway: a **list item** and a **blockquote** MAY write a space between the
marker, the head and the content (`> {.x} text`, `- {.x} item`, §9). Every other
construct — including table cells — requires strict adjacency. Decorator lines
(§9.1) stand on their own line instead.

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
- `@@` followed by a character that is not an ASCII letter, not `{`, not `[`
  and not `(` → literal text (`@@1anchor{.x}`, `@@.x`);
- `[…]` or `(…)` opened without its closer → the **whole head** is literal
  text (`@@type[…`, `{{type}}[…`, §4.1);
- `@@type {…}` — whitespace between the type and its `{` → literal text (§4.1);
- an item that is _malformed_ (unterminated `` ` `` or quote, a trailing
  character after a quoted value, an empty `key:` value, an invalid `key`)
  makes the **whole head** literal text — items are never dropped silently;
- a head separated from its construct by whitespace → literal text.

Note what is **not** text: `{}`, `@@{}` and `@@type{}` are valid empty heads
(§3), and `@@type` alone is a valid type-only head (§4.4).

### 4.4 Type names

```ebnf
type        = ALPHA ( ALPHA | DIGIT )*
extras-head = type-only | typed | untyped
type-only   = "@@" type group?                 # ends at the first non-alphanumeric
typed       = "@@" type group? "{" body "}"
untyped     = "@@" group+ | ( ( "@@" )? "{" body "}" )
group       = [ "[" bracket "]" ] [ "(" paren ")" ]   # each part adjacent, §6.1
```

ASCII only, case-insensitive in effect (`@@Type` and `@@TYPE` are both types),
no space, no `-`, no `_`, no other symbols (`fooBar` and `anchor2` are valid,
`foo-bar` and `foo_bar` are not). A type always starts with a letter: `@@1anchor`
is not a head. Widening this charset is a breaking change and requires a spec
update.

The alphanumeric run ends at the first symbol. What follows decides the head:

| Character after the type run             | Result                                                           |
| ---------------------------------------- | ---------------------------------------------------------------- |
| `{` (adjacent)                           | typed head — the body is parsed                                  |
| `[` or `(` (adjacent)                    | the positional group is parsed (§6.1), then `{` or the head ends |
| whitespace, EOL or EOF                   | **type-only head** — only the type is consumed                   |
| whitespace then `{`                      | literal text (§4.1)                                              |
| any other symbol (`-`, `_`, `.`, `,`, …) | **type-only head**; the symbol stays literal text                |

```text
[t](/a)@@anchorA.    → <a type="anchorA" …>t</a> + literal "."
@@type-x{.x}         → type `type` + literal "-x{.x}"
@@anchorA            → type-only head (end of the line)
```

## 5. Extras head grammar

```ebnf
extras-head = ( "@@" )? [ type ] "{" [ ws items ] ws "}"    # typed / untyped / empty
            | "@@" type                                     # type-only (§4.4)
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
            | object | array
bquote      = "`" { char } "`"
object      = "{" [ ws pairs ] ws "}"
pairs       = pair { ws "," ws pair } [ ws "," ]
pair        = key ws ":" ws value
array       = "[" [ ws values ] ws "]"
values      = value { ws "," ws value } [ ws "," ]
key         = ( ALPHA | "_" ) { ALPHA | DIGIT | "_" | "-" }
css-name    = key
name        = 1*( ALPHA | DIGIT | "_" | "-" | ":" )
ws          = 1*( " " | "\t" )
```

- Item order is free; extras are completely optional. `{}` is a valid (empty)
  head: it adds no attribute and raises no warning.
- Duplicate handling: `class` accumulates; `id`, `slug`, `title`, `prop`,
  `flag` and `cssvar` are last-wins; a duplicate `id` emits `Severity::Warning`.
  A duplicate key collapses to one attribute (§6.4), and a `class:` prop
  accumulates into `class` exactly like a `.x` item.
- `value` classification (produces `AttrValue`, §6.3): quoted or backticked →
  `Str`; `true`/`false` → `Bool`; integer → `Int`; decimal → `Float`; otherwise
  `Raw`. A quoted value is unquoted on the way in; `Int`/`Float`/`Bool` render
  back to the authored literal text.
- An empty item (`a,,b`, leading/trailing comma) is ignored.
- `value` MAY also be a nested `{…}` object or `[…]` array
  ([ADR-0005](../decisions/0005-structured-extras-values.md)). Nesting is
  arbitrary and the item splitter is depth-aware: a `,`, `}` or `]` inside a
  nested value — or inside a quoted string — never ends the item or the head.
  `{}` and `[]` are valid and empty.
- A nested object's entries are `key : value` pairs, or a `...ref` **spread** item
  (§19), using the `key` production above: `--cssvar`, `.class`, `#id`, `` `slug` ``,
  `title` and bare flags do not exist inside a `{…}` value. A duplicate nested key
  is last-wins (§6.4); a spread is transported to `plugin-bind` under the reserved
  key `...`, which no `key` can spell.
- A malformed nested value (unbalanced group, trailing characters, unquoted
  whitespace) rejects the **whole head**, which then falls back to literal text
  (§4.3). Extras are never partially applied.
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

| item      | config key        | default | meaning          |
| --------- | ----------------- | ------- | ---------------- |
| `` `x` `` | `backtick_key`    | `slug`  | construct slug   |
| `"x"`     | `quote_key`       | `title` | construct title  |
| `[x]`     | `bracket_key`     | `slug`  | bracket slot     |
| `("x")`   | `parentheses_key` | `title` | parentheses slot |

Every `@@` head may carry the `[x]` / `("x")` groups between its type and its
`{…}` body (`@@type[x]("y"){…}`, `@@type[x]("y")`, `@@[x]("y")`), each part
adjacent (§4.1). Markers place the groups directly after `}}`, before any
adjacent extras head: `{{type}}[x]("y"){…}` / `{{type}}[x]("y")@@head{…}`
(§10.1). The groups' contents behave exactly like a directive head's: `[x]`
stays raw, `("x")` is trimmed and unquoted. The keys are resolved per `type`
like every other positional key (§11 rules 3 and 5).

The groups are read by the layer that owns the construct, in the order of the
task's plugin list: more than one enabled layer can read the same `[x]("y")`
text, and **the layer that runs first wins**. With the canonical order
(`anchor` before `marker`, the order every sandbox uses) a marker therefore
keeps only a `[x]` that no `(` follows — `{{type}}[x]("y")` is already an anchor
head by the time `plugin-marker` runs, so the marker renders without those
attributes and the anchor component follows it. Listing `marker` before `anchor`
hands the marker both groups instead (`{{type}}[x]("y")` → `slug` + `title`).
The single-group and type-only forms (`{{type}}[x]`, `{{type}}("y")`,
`@@type[x]`, `@@type("y")`) parse the same way in either order: nothing else
claims them before their layer runs.

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
enum AttrValue {
    Str(String), Int(i64), Float(f64), Bool(bool), Raw(String),
    Object(Vec<(String, AttrValue)>), Array(Vec<AttrValue>),
}
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
- **empty string** (`k: ""`) → the attribute is omitted, so a node that leaves a
  slot unset does not emit `k=""`. `alt` is the single exception: an empty
  `alt=""` is meaningful (a decorative image) and always renders.
- `Object` / `Array` ([ADR-0005](../decisions/0005-structured-extras-values.md))
  → **compact JSON** text in `value` (`{"b":1}`), the stringly-typed form every
  renderer understands. Verified: with no data-binding plugin enabled,
  `[a](/u)@@anchorA{a: {b: 1}}` renders `a="&#123;&quot;b&quot;:1&#125;"`.
- **Structured transport.** With `plugin-bind` enabled (§19), a `$var`-bearing
  value is carried as `JSON_ATTR_PREFIX` (`U+E001`) + compact JSON;
  `renderer-ast` strips the prefix and re-hydrates a real JSON value, which
  `renderer-solid` spreads as `k={…}`. `crates/extra` MUST NOT emit `U+E001` —
  only `plugin-bind` may (`crates/core/src/event.rs`).

Renderers:

| Renderer                 | flag                               | typed string | Int/Float                     | structured (`{…}` / `[…]`)         |
| ------------------------ | ---------------------------------- | ------------ | ----------------------------- | ---------------------------------- |
| `renderer-solid`         | `k` (bare JSX shorthand, verbatim) | `k="v"`      | `k="12"` (fallback path only) | `k="{…}"`, or `k={…}` under `bind` |
| `renderer-html`          | `k`                                | `k="v"`      | `k="12"`                      | `k="{…}"` (escaped)                |
| `renderer-json` / `-ast` | `{ "name": "k", "kind": "flag" }`  | string       | string                        | string (compact JSON)              |

Custom components receive the **typed** form through the `attrs` map
(`{...attrs}` spread), because that path serialises `AttrValue` as JSON
(`12`, `true`) rather than as text.

> Rationale: bare flags render exactly as authored (`isFoo`) in every renderer;
> the DOM fallback path stays stringly-typed by design.

Omission scope: it applies to plain-element attributes and `{...attrs}` spreads
in `renderer-solid` and `renderer-html`. An **explicit** template binding
(`k={attrs.k}`) is untouched — the template author asked for that slot, so it
keeps rendering `k={""}`.

### 6.4 Emission order (deterministic)

Attributes MUST be emitted in this canonical order so golden fixtures are
stable:

1. `class` (single joined value, head classes then `.x` items in source order);
2. `id`;
3. positional keys — the groups first (`bracket_key`, then `parentheses_key`),
   then `slug` and `title` (using `backtick_key`/`quote_key`);
4. all remaining props and flags in source order, with `style` where the first
   `style:` prop or `--var` item appeared.

Duplicates collapse to one attribute: the position of the first occurrence
wins, the value of the last occurrence wins. `class` accumulates instead
(a `class:` prop joins the `.x` items). A positional item and a same-named prop
are such a duplicate: the positional slot keeps its place and the later value
wins. One exception: a positional **group** (`[…]` / `(…)`, §6.1) beats a
same-key body item outright — the body value is dropped with an §13 `Warning`
(§6.2 head-wins style). Whitespace-only differences MUST NOT change the output.

## 7. Construct grammars

Notation: `head?` = optional, `extras?` = optional `@@type{…}`, `∘` = no space.

### 7.1 Image and figure (`plugin-img`)

```ebnf
image    = marker "[" alt "]" "(" url ")" extras? caption?
marker   = container? marker-body               # container: `p` = <p>, `d` = <div>
marker-body = 1*( "!" | modifier )              # MUST contain >= 1 "!"; "!!" => figure
modifier = "?" | "~" | ( "w" | "h" ) DIGIT+
caption  = text
extras   = §5 extras-head                       # the shared {…} / @@type{…} head
```

- Markers must contain at least one `!`; `!!` enables figure mode. A leading `p`
  wraps in `<p>` and `d` wraps in `<div>` (container); a container prefix and `!!`
  are mutually exclusive (e.g. `p!!`, `d!!` are rejected and the line is skipped
  without error).
- Bare `!` requires at least one modifier (`?`, `~`, `w<digits>`, or `h<digits>`); otherwise the line is left for `plugin-markdown`.
- Modifiers are order-independent: `?` adds `loading="lazy"`, `~` adds `decoding="async"`, `w<digits>`/`h<digits>` set explicit dimensions on the inner `<img>`.
- Invalid marker combinations (e.g., `p!!`, `d!!`) are rejected and the line is skipped without error.
- Extras attach to the outermost node (`Figure` when `!!`, else `Image`); dimensions from modifiers always apply to the inner `<img>`.
- Trailing caption text is permitted only in figure mode (`!!`); non-figure markers ignore trailing text.
- Caption content is processed through the shared inline pipeline (wiki, cite, anchor, markdown) before rendering.

### 7.2 Anchor (`plugin-anchor`)

```ebnf
anchor   = "[" text "]" "(" url modifier* title? ")" extras?
modifier = "^" | "~" | "!" | "$" | ";;" | "--"
extras   = §5 extras-head                       # the shared {…} / @@type{…} head
```

- Modifiers are appended directly to the URL (before the title) and parsed right-to-left: `^` forces `target="_blank"`, `~` forces `target="_self"`, `!` adds `nofollow`, `$` adds `sponsored`, `;;` adds `ugc`, `--` adds `noreferrer`.
- Conflicting `^` and `~` modifiers resolve as last-wins with a warning diagnostic emitted.
- External URLs (http, https, protocol-relative, www, or non-root paths containing dots) automatically receive `target="_blank"` and `rel="noopener"` unless overridden by modifiers.
- Extras attach to the `<a>` element; `rel` values merge with modifier-generated tokens rather than replacing them, while `target` in extras overrides modifier defaults.
- Positional `` `slug` `` maps to `id` when `#id` is absent; `"title"` in extras yields to the standard link title syntax.
- Image syntax `![alt](src)` is explicitly skipped; reference-style links and nested brackets are unsupported.

### 7.3 Cite (`plugin-cite`)

```ebnf
cite     = "[^^]" "(" ref [ ws dquote loc dquote ] ")" extras?
ref      = 1*( any char except ws, ")", ",", dquote )
extras   = §5 extras-head                       # the shared {…} / @@type{…} head
```

- The reference is **unquoted** and the location is an optional **quoted**
  string. `[^^](book)` and `[^^](book "hlm. 45")` are the only two forms.
- Everything else is literal text (no citation, no warning):
  `[^^]("book")`, `[^^](book, "hlm. 45")`, `[^^](book loc="x")` and `[^^]()`.
- Extras attach to the citation node and MUST touch the closing `)`; the head
  values (`id`, `loc`) win over same-named extras props (§6.2).
- First head positional → `id`; the quoted location → `loc`. There is no `loc=`
  prop form of the head: the quoted string after the reference is the only
  location syntax.

### 7.4 Heading (`plugin-heading`)

```ebnf
heading  = "#"(1..6) [ "[" slug "]" ] [ "(" title ")" ] extras? text
extras   = §5 extras-head                       # the shared {…} / @@type{…} head
```

- `[slug]` and `("title")` are both optional and independent, and the whole head
  MUST touch the `#` run (§4.1); the extras head follows them adjacently.
- `("title")` is **not** the `innerText`; it is a separate attribute.
- Whitespace between the head and the heading text is consumed by the head (it
  MUST NOT become part of the title).
- Extras attach to the heading element; extras `slug` is ignored when `[slug]`
  exists.

### 7.5 Wiki (`plugin-wiki`)

```ebnf
wikilink = "[[" target [ "|" label ] "]]" extras?
extras   = §5 extras-head                       # the shared {…} / @@type{…} head
```

- Extras attach to the wiki `<a>`; `href` is produced by the plugin
  (`link_prefix`) and MUST NOT be overridable by extras (it wins).

## 8. Table (`plugin-table`)

```ebnf
table       = decl? caption? header-row row* [ foot ]
decl        = "|-" [ "[" slug "]" ] [ "(" title ")" ] extras? ( "-|" | "|" )
caption     = "||" extras? content "||"
header-row  = "|" cell ( "|" cell )* "|" extras?     # alignment/width row
row         = "|" cell ( "|" cell )* "|" extras?      # body row
foot        = "|===|" extras? row*
cell        = "|" extras? content

extras   = §5 extras-head                       # the shared {…} / @@type{…} head
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

- **Slot placement (§4.1).** A table has **no** adjacency exception: every head
  touches the `|` (or the `||`) it belongs to.
  - A cell head sits directly after its opening `|`: `|{.x} text |`,
    `|@@cellB{.lead} text |`; `| {.x} text |` is literal text.
  - A caption head sits directly after `||`: `||{.c} caption ||`.
  - A trailing row/section head sits directly after the line's **last** `|`:
    `| a | b |{.row}` and `|===|@@tfootX{.total}`.
  - The head of a **delimiter cell goes after the alignment code and touches it**:

    ```text
    | :---(200px)@@cellA{.v-top} | :---:{.c} | ---:(30%) | :---@@cellB{.a, #i, k: "v", n: 2} |@@tbodyX{.tb}
    ```

- The declaration line is **childless**: any non-extras/head text inside it is
  discarded (a `Severity::Warning` MAY be emitted).
- The declaration line is optional; when absent the table is untyped.
- `head` inside `decl` uses the standard `[slug]` / `("title")` form:
  `|-[slug-foo]@@type{…}-|`.
- Bare `===` as a foot separator is **retired**; only `|===|` remains.
- Alignment/width tokens (`:---:`, `(200px)`) and the `>` (colspan `+1`) / `^`
  (rowspan `+1`) cell markers are unchanged and come **before** the cell extras.
- Cell content may be empty; empty cells become `<td></td>`.
- Text before the closing `-|` is literal text (§14).

## 9. Decorators, blockquotes and lists

### 9.1 The decorator rule (general)

A **decorator line** is a line whose entire content is a head — `type?{…}` with
the `@@` sigil optional (§3, §9.3) — with optional leading indentation and no
trailing text other than whitespace. It decorates the **next block node** at the
same nesting level and is consumed (MUST NOT be rendered):

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

Positional groups (§6.1) are part of the head, between the type and the body:

```text
@@aside[intro]("Aside title"){.lead}
Decorated with the bracket / parentheses slots too.

@@[intro]("Aside title")
The untyped form: `@@` is required, so the line can never be a link.

[intro](url)
A paragraph line like this stays a link — without `@@` it is not a decorator.
```

- The decorator's `type` (when present) is the **type marker** of the decorated
  node and drives per-type component selection (§11).
- The `@@` sigil is what separates a decorator from a link at the start of a
  line: the groups need it (`@@[…]`), the bare forms do not gain it (`[…]` is
  never a decorator, §4.1).

- A decorator applies to a **paragraph** or a **code fence** (`plugin-markdown`),
  a **list container** (§9.3) or a **blockquote** (§9.2). A decorator line above
  a **heading** decorates the **section** (§9.5), and a **table** parses its own
  heads (§8); neither is a `plugin-markdown` target. Anything else (inline
  constructs) uses the adjacent form (§4.1).
- If no following block exists (end of input, or the next sibling is not a
  block), an **explicit** `@@…` decorator is dropped with a `Severity::Warning`.
  A bare `@@`-less `{…}` line is ambiguous with literal text (§4.3), so when it
  has no block to bind it stays literal instead of being dropped.
- Consecutive decorators: only the **last** one applies; earlier ones are
  dropped with a `Severity::Warning` (MAY be an error later).
- Indentation MUST be `<=` the indentation of the decorated block; a decorator
  indented deeper than the following block is literal text.

### 9.2 Blockquote (`plugin-blockquote`)

```ebnf
blockquote_inner = ">" ws "@@" type positional* extras? content
decorator_line   = "@@" type positional* extras?
```

- This plugin binds extras to the blockquote node **before** `plugin-markdown` parses content; `>` remains plain Markdown.
- Two spellings are supported: an inner head immediately inside the quote (`> @@type{…}`) or a decorator line directly above it.
- The inner head takes precedence when both forms are present for the same blockquote.
- Whitespace between the `>` marker, the head, and the content is explicitly allowed as an adjacency exception (§4.1).
- Decorator lines inside code fences or raw HTML blocks are treated as literal text and do not bind.
- The paragraph wrapping the quote is replaced with `NodeKind::Custom(name)` if claimed by `[[task.blockquote.custom]]`, otherwise `NodeKind::Element("blockquote")`.
- Nodes are marked `__plugin_kind = "block"` so stripped body content is re-lexed as block-level Markdown (headings, lists, nested quotes).
- Nesting is preserved one level at a time; `>> inner` leaves `> inner` for the subsequent Markdown pass.

### 9.3 List (`plugin-list`)

```ebnf
list_decorator = "@@" type? extras?
marker         = "-" | "*" | "+" | DIGIT+ "."
```

- This plugin binds extras to list containers **before** `plugin-markdown` parses content; markers remain plain Markdown.
- A decorator line directly above a list targets the container layer: `unordered` for `-`/`*`/`+`, `ordered` for `1.`.
- The layer is determined strictly by the **marker**, never by the decorator's own type name.
- Container nodes emit `NodeKind::Custom(name)` when claimed by config, otherwise `NodeKind::Element("ul"|"ol")` with `__plugin_kind`.
- Extras land on the wrapper element or component, ensuring attributes apply to the list itself rather than surrounding it.
- Ordered list start offsets follow CommonMark behavior (first item number); extras cannot currently override this value.
- Decorator lines that fail to bind to a valid list trigger a `Severity::Warning` diagnostic.
- Item-level decorators (L2/L3) and nesting validation (L5) are defined in spec but not yet implemented.
- Positional slots default to `` `slug` `` and `"title"` but can be renamed per entry via `backtick_key` and `quote_key`.
- Unbound decorators outside of lists remain as ordinary block nodes to prevent silent data loss.

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

### 9.5 Section (`plugin-section`)

`plugin-section` wraps each heading and the content under it in a nested
`Section` node, giving renderers a clean outline. It runs **before**
`plugin-markdown` and `plugin-heading`: the level markers collide with Markdown
(`>---<` is a blockquote), so the raw heading text, the decorator line and the
markers are read before Markdown runs. Markdown then parses the interior of
every `Section`.

- A decorator line directly above a heading — `@@type{…}` / `{…}` (optional
  indentation; a blank line before the heading is allowed) — decorates the
  **section**, not the heading: the heading already owns its extras on its `#`
  run (§7.4).

  ```text
  @@sectionA{`slug-section`, #sectionID}
  ###[slug-head]("Heading X")@@headingX{`slug-head-extras`, #headingID} Title
  ```

- The section **id** is the first available of `#sectionID` (the decorator's
  `#id`) > `` `slug-section` `` (the decorator's slug) > `` `slug-head` `` (the
  heading's `[slug]`) > `` `slug-head-extras` `` (the heading's extras slug) >
  the slug of the heading title. A heading's own extras `#id` is **not** part of
  the chain; it is ignored with a `Warning`.
- When `plugin-section` owns the outline the heading never emits an `id` (or its
  fallback `slug`): the id always transfers to the section.
- The decorator's `type` selects a §11 `section`-layer component; without one the
  built-in `<section>` element is used (D8).
- **Level markers.** A line whose trimmed content is exactly `<--->` deepens the
  outline by one nested section (capped at level 6); a line that is exactly
  `>---<` closes the innermost section (a no-op once only the preface is open).
  Both are consumed and never rendered.
- Content before the first heading is wrapped in a preface `Section` (level 0).

Config: `[task.section.custom.section]` (`name` / `template` / `imports`), the
plugin's only (primary) layer.

## 10. Markers and directives

### 10.1 Marker (`plugin-marker`)

```ebnf
marker-inline = "{{" type "}}" groups? extras?   # inside inline content
marker-block  = "{{" type "}}" groups? extras?   # own line (block context)
```

- `type` is **mandatory** for markers; `{{}}` is literal text.
- Extras are optional but must be adjacent.
- The positional groups (§6.1) sit directly after `}}`, **before** any extras
  head: `{{type}}[x]("y"){…}` and `{{type}}[x]("y")@@head{…}`. They map through
  the marker type's `bracket_key` / `parentheses_key`. A marker-level group
  beats a same-key group of the extras head (§6.2, dropped with a §13 Warning);
  a malformed group stays literal text while the marker still renders (§4.3).
  A `[x]("y")` pair is also a legal anchor head, so which layer reads it first
  decides: §6.1's layer-order rule applies (in the canonical order the anchor
  layer claims it and the marker keeps no attribute from it).
- Inline form emits an inline node; block form emits a block node.
- The block form carries the same line's trailing text as its `children` and
  does not absorb the following block.
- With no custom component, the inline form renders `<span>` and the block form
  renders `<div>`, both carrying the extras of the head (§6).
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
- No custom component → `<span>` (an `Element` node, exactly like a marker), so
  no extra of the head is dropped.

### 10.3 Block directive (`plugin-directive`)

```ebnf
block-directive = fences type [ "[" bracket "]" ] [ "(" paren ")" ] extras? body fences
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

A directive whose type no component claims — including a layer with no default —
falls back to `<span>` / `<div>` with the instance's `type` and extras, exactly
like a marker (§11 rule 3). The node is never rendered as a bare child container.

### 10.4 Priority summary

| Slot                     | Winner                                                    |
| ------------------------ | --------------------------------------------------------- |
| element `id`             | `#id` > head slug > extras slug                           |
| `title`                  | head `("…")` > extras `"…"`                               |
| `class`                  | union of head classes and `.x` items, in source order     |
| `href` / `src` / `start` | construct-owned, never overridable                        |
| positional group vs body | the group (`[…]` / `(…)`) wins; body value dropped (§6.4) |
| any other key            | head prop > extras prop                                   |

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

### 12.2 Post-markdown (emit)

A single **emitter** pass (shared by `plugin-list`, `plugin-blockquote`,
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

> **Not `plugin-bind` (§19).** The word _emit_ here means "turn a parsed extras
> payload into `Event`s and attach them". It is not data binding: there is no
> `{{{…}}}` block and no `$var` at this stage. `plugin-bind` runs _before_
> `parse` (the **extract** stage) and resolves _after_ this pass; only those
> `$var` references are ever called "binding"
> ([ADR-0005](../decisions/0005-structured-extras-values.md)).

### 12.3 Fallback paths

When no custom component claims a layer, the built-in renderers emit the element
with the node's attributes, so no extras are silently dropped. `renderer-solid`
(`crates/renderer-solid/src/node.rs`) calls `render_attrs` for `Paragraph`
(`<p>`), `Blockquote` (`<blockquote>`), `Heading` (`<h*>`, skipping the internal
`level`), `Section` (`<section>`), `CodeFence` (`<pre>`, skipping `raw_html`),
`BulletList` (`<ul>`) and `OrderedList` (`<ol>`, where `start` is emitted
numerically); the HTML/JSON/AST renderers mirror this. The fallback path emits
`id`, `class` and `style` and passes the remaining keys as attributes.

Known gap: `ListItem` (`<li>`) is rendered by the fallback path without its
attributes — this is the item layer (L2/L3, §9.3), which is still pending.

## 13. Diagnostics

| Condition                                                 | Severity                 |
| --------------------------------------------------------- | ------------------------ |
| malformed head / missing `}` / whitespace-separated head  | none (literal text)      |
| a retired legacy form (`[.c,#id]{k:v}`, `[^^]("ref")`, …) | none (literal text)      |
| duplicate `#id` in one extras block                       | Warning                  |
| extras `id` overriding a head slug                        | Warning                  |
| decorator with no following block                         | Warning                  |
| dropped decorator (consecutive decorators)                | Warning                  |
| dropped extras value because a head value won             | Warning                  |
| unclosed block directive at EOF                           | Warning                  |
| unknown plugin name in `task.plugin`                      | Warning                  |
| unknown `task.<plugin>` key in `pendon.toml`              | Warning                  |
| template without `{children}` (non-void element)          | Error (config load)      |
| unbalanced template element                               | Error (config load)      |
| two default components for one layer                      | Error (config load)      |
| `start:` prop on `ol` extras                              | Warning (ignored)        |
| literal `U+E000` sentinel character in source             | Warning (stripped)       |
| unknown key outside a known layer                         | Warning (passed through) |

Config-load errors MUST abort the build before any file is processed.

## 14. Retirement and collisions

| Removed                       | Replaced by                          | Note                                                                                                                                                                                                                                                                                                          |
| ----------------------------- | ------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `[task.cite.section]`         | `plugin-marker` (`{{bibliography}}`) | **removed**: `CiteOptions.section` and `CitationContext::replace_section_markers` are gone; the section component is now a marker.                                                                                                                                                                            |
| `{{section}}` in sources      | marker block                         | **removed**: the `{{ footnote }}` lines in `sandbox/cite` and `sandbox/img` are gone; `{{…}}` is free for `plugin-marker`.                                                                                                                                                                                    |
| `:::name` custom blocks       | `==type` block directive             | deprecated alias kept for one release (Warning), then hard-error — §15 OPEN-R1.                                                                                                                                                                                                                               |
| bare `===` table footer       | `\|===\|`                            | **removed**: only `\|===\|` remains.                                                                                                                                                                                                                                                                          |
| `o.` ordered-list offset      | first item's number                  | **removed**: an `ol`'s `start` comes from the first item and is not overridable (§9.4).                                                                                                                                                                                                                       |
| `plugin-quiz` (`:::quiz`)     | —                                    | retired from the pipeline; plugin code stays, deep rework later.                                                                                                                                                                                                                                              |
| legacy `[.c,#id]{k:v}`        | `{…}` / `@@type{…}`                  | **removed** (D3): plain literal text, no warning, no deprecation path. Gone from the code (`parse_attrs`, `ExtraAttrs`, `legacy_extras_warning`), docs, sandboxes and READMEs.                                                                                                                                |
| `import = "…"` (string)       | `imports = ["…"]`                    | **removed**: the singular form is accepted with a Warning (§15 OPEN-C2), then hard-errors.                                                                                                                                                                                                                    |
| `[task.<plugin>.custom_node]` | `[task.<plugin>.custom.<layer>]`     | **removed**: `custom.<layer>` is read for `anchor`, `img` (`figure`), `heading`, `cite` and all seven `table` layers (`apps/cli/src/plugins.rs`); every sandbox/fixture config is migrated. The stale key fails the build with a migration message. Unwired layers (`task.img.custom.img`) are config errors. |

## 15. Open decisions

Each open item records the recommendation to implement **unless the maintainer
objects**; the spec text above already assumes the recommendation.

- **OPEN-L1 (item extras placement).** An item-decorator at the very start of
  an item's line decorates the `li` (L2); a decorator on its own line _inside_
  the item decorates the following block (L3).
  _Alternative:_ both always decorate the following block (then `li` extras
  need a different spelling). **Recommendation: as specified.**
- **OPEN-B1 (bare fence).** A bare `==…==` fence only closes; anonymous block
  directives are impossible. **Recommendation: as specified** (removes the
  open/close ambiguity of `===`).
- **OPEN-E1 (table column extras).** Alignment-row cell extras apply to that
  column's `<th>` only. **Recommendation: yes** (cascading to `<td>`s would
  duplicate attributes on every body cell).
- **OPEN-T1 (tenant of `plugin-blockquote`/`plugin-list`).** Both plugins are
  **new crates** (they now exist); the existing markdown-plugin behaviour stays
  as fallback when they are not enabled. **Recommendation: new crates.**
- **OPEN-R1 (`:::` compatibility).** Keep parsing `:::name` blocks as a
  deprecated alias of `==name` for one release (Warning), then hard-error.
  **Recommendation: alias with Warning** — `sandbox/universal`,
  `sandbox/custom` and `apps/cli/tests/fixtures` still use `:::`.

### Resolved decisions

Recorded for provenance; the current behaviour is what the sections above state.
Rationale lives in [`decisions/`](../decisions/).

- **OPEN-BIND-1 (`var` first character) — resolved.** `is_valid_var` requires a
  leading `ALPHA`, so `$100` is plain text and "no `$` escape hatch is needed"
  holds. No fixture had a digit-leading name, so nothing was re-frozen.
- **OPEN-BIND-2 (path into a bound value) — resolved.** Dotted key, integer array
  index and quoted key only; a miss warns and stays literal; a non-`ref` shape
  (`$a.`, `$a[*]`) is untouched text. Implemented (`Ref`/`parse_ref`/`lookup` in
  `crates/plugin-bind`), frozen by golden `24-bind-paths`. RFC §2.5.
- **OPEN-BIND-3 (merge / override) — resolved.** `...$ref` inside a nested `{…}`
  value supplies **defaults**, shallow: an authored key wins over a spread, the
  later spread wins among spreads. The spread travels under the reserved
  `SPREAD_KEY` (`"..."`) that `crates/extra` emits, because a JSON object loses
  item order before `plugin-bind` sees it — so §6.4's "later item wins" does
  **not** carry over. RFC §2.6.
- **OPEN-BIND-4 (external data files) — resolved.** `{{{lang[var](../src/x.ext)}}}`
  is a bodyless block whose payload is a file, resolved against the **source
  file's** directory (never the CWD, so it works at any depth); stdin is an
  `Error`; the file joins the render's cache dependencies. RFC §2.7.
- **OPEN-IR-1 (flags) — resolved.** `Event::AttributeFlag { name }` is in the IR
  and rendered verbatim by the AST, JSON, HTML (compact + pretty) and Solid
  renderers (§6.3).
- **OPEN-L2 (list layers) — resolved.** Layer keys are `list` (= `<li>` item),
  `unordered` (= `<ul>` container) and `ordered` (= `<ol>` container), mirroring
  `task.table.custom.<layer>`. Config: `[task.list.custom.list]`,
  `[task.list.custom.unordered]`, `[task.list.custom.ordered]`.
- **OPEN-C1 (config shape) — resolved.** Layered plugins use `custom.<layer>`
  (single table or array of tables); single-layer plugins use the primary layer
  shorthand (`[task.anchor.custom]`, `[[task.anchor.custom]]`). Implemented by
  the loader in `apps/cli/src/components.rs`.
- **OPEN-C2 (`import` alias) — resolved (warn).** The loader accepts the
  singular `import` and emits a Warning, then hard-errors one release later.
- **OPEN-C3 (layer routing) — resolved.** Each plugin selects a layer component
  per instance with §11 rule 3 (exact `type` match → layer default → built-in
  element); the earlier "routable layer" restriction in `apps/cli` is gone
  (`apps/cli/src/plugins.rs::layer_set`).
- **OPEN-S1 (`[task.cite.section]` removal order) — resolved (done).**
  `plugin-section` is heading/icon based and never references `cite.section`, so
  the removal landed with the cite change: `CiteOptions.section`,
  `CitationContext::replace_section_markers`, the section branch of
  `cite::solid_hints` and the `[task.cite.section]` blocks in `sandbox/cite`,
  `sandbox/cite/custom`, `sandbox/universal` and `sandbox/img/regular` are gone,
  together with the `{{ footnote }}` lines in the two sources that used them.

## 16. Golden set and tests

Expectations for the grammar are frozen as **data** before implementation, so no
downstream rewrite happens. The fixtures live in `docs/spec/golden/` and are the
**executable source of truth**; see [`golden/README.md`](golden/README.md) for the
format, the harness (`apps/cli/tests/syntax_spec.rs`, one `#[test]` per fixture,
byte-for-byte against the real CLI) and the re-freeze procedure.

### 16.1 Fixture format

```text
docs/spec/golden/NN-name.md           input markdown
docs/spec/golden/NN-name.toml         pendon.toml for the fixture (plugins + custom sets)
docs/spec/golden/NN-name.jsx          expected solid output (generated — never hand-edited)
docs/spec/golden/NN-name.events.json  expected event IR (optional, for parser bugs)
docs/spec/golden/NN-name.data/        payload files read by a `(path)` block (optional)
```

### 16.2 Test gating

- A fixture whose syntax is not implemented yet is annotated
  `#[ignore = "pending Phase N"]` so `cargo test` stays green while the expectation
  is frozen. A phase is not complete while its fixtures are still ignored.
- `crates/extra` tests are **not** ignored: `src/value.rs` unit tests plus
  `tests/extras_spec.rs` (one case per rule of §4–§6) land green in Phase 0 and are
  the primary gate for the foundation.

> Live status (which fixtures are green) is tracked in [`STATUS.md`](../../STATUS.md),
> not here — this section stays normative.

## 17. Migration phases (historical)

The migration ran in phases (0: shared scanner; 1: construct plugins; 2: table
rewrite; 3: blockquote/list/marker/directive + binder; 4: list-item layer). The
completed-phase record and progress notes moved to
[`docs/archive/typed-extras-completed-phases.md`](../archive/typed-extras-completed-phases.md);
live work is in [`STATUS.md`](../../STATUS.md). This spec keeps only the target
grammar, so it does not drift as phases land.

## 18. Acceptance criteria

1. Every §3 sigil is either implemented or explicitly retired.
2. No extras data is dropped silently: any node that parses extras MUST emit them as
   attributes, with or without a custom component.
3. A `template` that would drop children fails the build at config load.
4. All §16 fixtures are green (none ignored) except fixtures explicitly deferred
   with a separate spec note.
5. Escaping and literal fallback are covered by fixtures 01/02.
6. `cargo test --workspace` is green; `sandbox/unified` produces the documented
   output.

## 19. Data blocks (`plugin-bind`)

A **data block** declares a structured value once; a `$var` reference in an
extras value binds it as a real JavaScript prop. Owner: `crates/plugin-bind`; the
pipeline placement is fixed by [ADR-0004](../decisions/0004-pre-parse-bind-stage.md).

```text
bind_block = "{{{" lang "[" var "]" body "}}}"                  ; body payload
           | "{{{" lang "[" var "]" "(" path ")" "}}}"          ; file payload
lang       = "json" | "jsonc" | "yaml" | "yml" | "toml" | "csv" | "mdp"
var        = ALPHA { ALPHA | DIGIT | "-" | "_" }
ref        = "$" var { "." var | "[" DIGIT+ "]" | "[" dquote key dquote "]" }
spread     = "..." ws ref                ; an item of a nested `{ … }` value
path       = any non-empty text up to the closing ")"
```

- A block MUST start at the beginning of a line. The body starts on the line
  after the head and ends at the first line whose trimmed content is exactly
  `}}}`.
- `lang` and `var` are REQUIRED. A block with a missing or invalid `lang`/`var`
  is dropped (never rendered) with a `Severity::Warning`; the same applies to an
  unterminated block, a payload that fails to parse, and a non-`mdp` body written
  on the head line.
- `var` MUST be unique; a duplicate warns and the **last** block wins.
- An empty body binds `null`, with no warning.
- `var` MUST start with a letter (`ALPHA`). A leading digit is never a reference
  name — `$100` is plain text. `is_valid_var` enforces it (`OPEN-BIND-1`).
- A `$var` reference is recognised only when it is the **entire** value of an
  extras item (`k: v`), quoted or unquoted; there is no free-text interpolation.
  It resolves inside a nested extras value as well.
- A `ref` MAY carry a **path** (`$config.db.host`, `$rows[0].nama`,
  `$m["a-b"]`): a dotted key, an integer index or a quoted key, walked left to
  right. A dotted key follows `var`, so a key that starts with a digit or holds
  whitespace needs the quoted form. An integer index addresses an **array** only.
  No wildcards, filters, recursive descent or slices. A miss warns and stays
  literal, exactly like an undefined `$var`; a value that is not a `ref` at all
  (`$100`, `prefix$var`, `$a.`, `$a[*]`) is untouched text with no warning.
- A **spread** item (`{...$brand, scale: 1.2}`, only inside a nested `{ … }`
  value) merges a bound object in as **defaults**, shallow. A key written in the
  object wins over a spread, and among spreads the later one wins. A spread that
  is unbound, not an object, or not a reference warns and is ignored (the other
  items still apply). A spread's own values are copied, never walked as
  references. `SYNTAX.md` §6.4's "later item wins" does **not** apply here — the
  merge is transported through a JSON object, which loses item order.
- A **file payload** replaces the body with a `(path)` group closed by `}}}` on
  the same line: `{{{csv[rows](../data/rows.csv)}}}`. `path` is resolved against
  the **source file's** directory — never the process CWD — so a relative path
  works at any depth; `..` and absolute paths are allowed (this is not a security
  boundary). A document from stdin has no such directory, so a file payload there
  is an **error**. An unreadable file, a directory, non-UTF-8 bytes, a path
  **and** a body in one block, or a missing `}}}` warn and drop the block. `lang`
  is authoritative; if a known data extension disagrees with it, warn. A block's
  file is recorded as a dependency of the render so an edit re-renders the page.
  The `( … )` group is not a path for `mdp` (its head-line body keeps its
  meaning).
- An undefined `$var` warns and stays literal text.
- **No `$` escape, and none needed.** Because a reference must start with
  `ALPHA`, `$100`, `$1.50` and `$ 5` are ordinary text, so a currency or price
  value is written as-is (`price: $100`, `price: "$100"`). There is no `\$`
  escape hatch, and **quoting is not one**: a quoted value is unquoted before the
  reference check, so today `price: "$100"` raises the same unbound warning as
  `price: $100` (verified). Quoting exists for whitespace and separators, not for
  escaping a `$`.
- **Non-goals.** No free-text interpolation (`Total: $sum` stays literal); no
  `\$` escape; no frontmatter/micromatter payload source; resolution is a
  **build-time** transform with no reactive/runtime binding.
- Values are parsed to JSON: `jsonc` strips `//` and `/* … */` comments outside
  strings; `yaml` expands merge keys (`<<`) and maps non-finite floats to `null`;
  `toml` datetimes become RFC 3339 strings; `csv` uses the header row as keys and
  parses each cell as JSON when possible (`NULL` → `null`).
- `mdp` (Pendon Markdown as a value) is specified but not implemented yet: it
  warns and drops the block.

`plugin-bind` runs **before** `pendon_core::parse` (a payload's `#`, `-` and blank
lines would otherwise be markup) and resolves **after** every other plugin. It is
Solid-first: a bound value reaches the prop as `name={…}`; other renderers keep a
stringly-typed fallback. Executable contracts: goldens [`23-bind`](golden/README.md)
(blocks, `$var`, nested values) and [`24-bind-paths`](golden/README.md) (paths,
spread/merge, a file payload resolved through `../data/`).
