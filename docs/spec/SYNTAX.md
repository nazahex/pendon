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

| Unit                                                           | Owns                                                            |
| -------------------------------------------------------------- | --------------------------------------------------------------- |
| `crates/extra`                                                 | extras scanners, `AttrValue`, `ExtrasHead`, `parse_type_marker` |
| `crates/plugin-marker`                                         | `{{type}}` inline/block markers                                 |
| `crates/plugin-directive`                                      | `::type` inline and `==type` block directives                   |
| `crates/plugin-list`                                           | list container + item extras binding (item layer L2/L3 pending) |
| `crates/plugin-blockquote`                                     | blockquote extras binding                                       |
| `crates/plugin-bind`                                           | `{{{lang[var] … }}}` data blocks + `$var` binding (§19)         |
| `crates/plugin-table`                                          | table head, declaration row, cell/row/section extras            |
| `crates/plugin-section`                                        | heading-driven `Section` outline + section decorator / markers  |
| `crates/plugin-image`, `-anchor`, `-cite`, `-heading`, `-wiki` | their own head + extras                                         |
| `crates/plugin-custom`                                         | component rendering (`node = "Component"`)                      |

### Non-goals

- `plugin-wiki` infobox internals (reworked separately).
- `plugin-quiz` (retired here, see §12).
- Renderer template semantics beyond attribute mapping (§6).

## 2. Terminology

- **head** — the construct-specific prefix that carries the construct's own
  identity: `@[slug](…)`, `[^ref "loc"]`, `[slug]`, `("title")`, `[[…]]`,
  `~?!…(…)`, `|-…-|`.
- **slot** — the `[…]` and `(…)` groups of a head, each carrying at most one
  _bare_, one _quoted_ and one _ticked_ content, in that order (§7.2, ADR-0012).
  Slots are universal: an element claims the positions it wants by declaring a
  contract, and a slot content no position claims is kept as a bare field.
- **slot tag** — the sigil that says which element a slot belongs to (`@` for
  the anchor, §7.2, D-11). A slot that is not attached to another head needs one;
  without a tag the text is nobody's head.
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

| Token                       | Meaning                                         | Owner               | Status                          |
| --------------------------- | ----------------------------------------------- | ------------------- | ------------------------------- |
| `{…}`                       | extras head, **adjacent** to its construct      | `crates/extra`      | **current**                     |
| `@@type{…}`                 | typed extras head (adjacent or decorator line)  | `crates/extra`      | **current**                     |
| `@@type`                    | type-only head                                  | `crates/extra`      | **current**                     |
| `@@{…}`                     | untyped head on a **decorator line**            | `crates/extra`      | **current**                     |
| `{}` / `@@{}`               | empty head                                      | `crates/extra`      | **current**                     |
| `` `x` `` / `"x"`           | positional slug / title                         | `crates/extra`      | new                             |
| `.x` `#x` `--x:v` `k:v` `k` | class / id / CSS var / prop / flag              | `crates/extra`      | extended                        |
| `{{type}}`                  | inline & block marker                           | `plugin-marker`     | new (freed from `cite.section`) |
| `::type` … `::`             | inline directive, 2–7 colons                    | `plugin-directive`  | new                             |
| `==type` … `==`             | block directive                                 | `plugin-directive`  | new                             |
| `@[k]("k")`                 | anchor head (tag + slots)                       | `plugin-anchor`     | current (§7.2)                  |
| `[k]` `("k")`               | construct head (slug / title)                   | per-plugin          | existing                        |
| `\|-…-\|`                   | _retired_ table declaration line (literal text) | —                   | **removed** (D-10)              |
| `\|\| cap \|\|`             | table caption line                              | `plugin-table`      | new                             |
| `\|===\|`                   | table foot separator                            | `plugin-table`      | existing (kept)                 |
| `~?!` / `~?!!`              | image / figure marker                           | `plugin-image`      | existing                        |
| `[^ref "loc"]`              | citation (tag + slots)                          | `plugin-cite`       | current (§7.3)                  |
| `[[…]]`                     | wiki link                                       | `plugin-wiki`       | existing                        |
| `>`                         | blockquote                                      | `plugin-blockquote` | current (decorator only, §9.2)  |
| `-` `*` `+`                 | unordered list container                        | `plugin-list`       | current (+extras)               |
| `1.`                        | ordered list container                          | `plugin-list`       | current (+extras)               |
| `{{{lang[var] … }}}`        | data block                                      | `plugin-bind`       | new (§19)                       |
| `$var`                      | bound data reference (extras only)              | `plugin-bind`       | new (§19)                       |
| `$a.b[0]`                   | path into a bound value                         | `plugin-bind`       | new (§19)                       |
| `...$var`                   | spread/merge inside a `{…}` value               | `plugin-bind`       | new (§19)                       |
| `{{{lang[var](../src/x)}}}` | external data block                             | `plugin-bind`       | new (§19)                       |
| `:::name`                   | legacy custom block                             | `plugin-custom`     | superseded by `==type`          |
| `@@`                        | _no other meaning_                              | —                   | reserved for Pendon             |

`@@` is reserved: no plugin may claim it for another purpose.

## 4. Lexical rules

### 4.1 Adjacency

An extras head MUST be adjacent to the construct it belongs to, and to its own
parts:

```text
~?!h300w800[alt](url)@@type{`slug`, .hero}   correct
@[text](url){`slug`}                         correct (bare spelling)
@[text](url)@@anchorA.                       correct: type-only head, `.` stays text
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
marker, the head and the content (`- {.x} item`, §9). Every other construct —
including table cells — requires strict adjacency. Decorator lines (§9.1) stand on
their own line instead.

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
@[t](/a)@@anchorA.   → <a type="anchorA" …>t</a> + literal "."
@@type-x{.x}         → type `type` + literal "-x{.x}"
@@anchorA            → type-only head (end of the line)
```

## 5. Extras head grammar

```ebnf
extras-head = ( "@@" )? [ type ] "{" [ ws items ] ws "}"    # typed / untyped / empty
            | "@@" type                                     # type-only (§4.4)
            | number-mark                                   # bare `%%numberType` (§10.5)
items       = item { ws "," ws item } [ ws "," ]
item        = slug | title | class | id | cssvar | prop | flag | number-mark
number-mark = "%"(2..7) name                            # depth = run − 1
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
- `@@` is optional where the head is **adjacent** to its construct (a table cell,
  a caption, a row's end-of-line head): `|{.x} text |` and `|@@cellA{.x} text |`
  are the same head. A head that stands **alone on a line** is a decorator line
  (§9.1), and there the sigil is mandatory: `{.x}` on its own line is literal
  text, `@@{.x}` decorates the block below it (D-10, §14).
- Duplicate handling ([D-12](../workflows/unified-syntax-stage-2/DECISIONS.md)):
  two sources for one public path inside one head is a bug, not a merge, so it
  is a `Severity::Error` and the **first** value wins. That covers a repeated
  `slug`, `title` or `#id`, and a repeated `prop` or `flag`. Two exceptions:
  `class` accumulates and dedupes, keeping the first position (a `class:` prop
  joins the `.x` items exactly like a `.x` item), and a repeated `style:`
  property is CSS — last value wins, no diagnostic.
- A `k` that names a **built-in field** of the element the head attaches to
  (`href` on an anchor, `src` on an image) is `Severity::Error`: the construct
  keeps its own value and the public name is chosen with
  `field.override.<k>` (§11). A malformed value in either case never partially
  applies (§4.3).
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
- A `number-mark` is the `%%numberType` **numbering marker** (§10.5,
  [ADR-0010](../decisions/0010-plugin-number.md)). It is one whole item — no
  separator and nothing trailing — and it has three legal positions: alone as
  the head (`@[link](/u)%%numberA`, `##%%numberB`), the **last** item of a `{…}`
  body (`@@headingA{foo: "bar", %%%numberB}`), or directly after a head
  (`@@anchorA%%numberB`, `@@headingA{…}%%%numberB`, `@@%%%numberB`). `depth` is
  normalized once, here (`%%` → 1 … `%%%%%%%` → 6), and the item reaches the
  plugin as the internal attribute `__number` with
  `{"depth":1,"type":"numberB"}`; the type keeps the authored spelling, because
  resolving it is `number`'s job (§10.5). `%` alone, `%%` with no name and
  a run of 8+ `%` are literal text with no diagnostic (`D-9`).

### 5.1 Worked example

```text
@@type{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar",
      bar: 12, isFoo: true, --style-var: "2rem", isBar}
```

> The head is shown wrapped for readability only: a head is always a single
> line (§4.3). This exact example is asserted by
> `crates/extra/tests/extras_spec.rs::spec_worked_example`.

| item                  | kind   | public field (§6.1)           |
| --------------------- | ------ | ----------------------------- |
| `` `slug-foo` ``      | slug   | `specs.tick = "slug-foo"`     |
| `"Title Foo"`         | title  | `specs.quote = "Title Foo"`   |
| `.extra` `.class`     | class  | `specs.class = "extra class"` |
| `#id`                 | id     | `specs.id = "id"`             |
| `foo: "bar"`          | prop   | `foo = "bar"`                 |
| `bar: 12`             | prop   | `bar = 12` (Int)              |
| `isFoo: true`         | prop   | `isFoo = true` (Bool)         |
| `--style-var: "2rem"` | cssvar | merged into `specs.style`     |
| `isBar`               | flag   | bare attribute `isBar`        |

> Three of those paths are universal, so a DOM renderer prints them as the fields
> they name — `class="extra class" id="id" style="--style-var: 2rem"` — while what
> the tick and the quote become is the element's own decision (§6.1, §6.3,
> §7.x). This exact map is asserted by
> `crates/extra/tests/extras_spec.rs::spec_worked_example`.

## 6. Extras → attribute mapping

### 6.1 Positional names

Every positional content of a head lands in exactly one **public field**, and the
entry names it with a `field.*` key (§7, §11, D-5). The five `specs.*` rows are
the **positions** a construct can attach meaning to (`SPECS_KEYS` in
`crates/extra::field`); the two slot rows carry the bracket and parentheses
groups of an element head:

| content                 | config key           | default path   | meaning                              |
| ----------------------- | -------------------- | -------------- | ------------------------------------ |
| `` `x` `` (tick)        | `field.specs.tick`   | `specs.tick`   | the construct's slug (short id)      |
| `"x"` (quote)           | `field.specs.quote`  | `specs.quote`  | the construct's title                |
| `.x` / `class: "x"`     | `field.specs.class`  | `specs.class`  | the class list (accumulates, D-12)   |
| `#x`                    | `field.specs.id`     | `specs.id`     | the element id                       |
| `--x: v` / `style: "…"` | `field.specs.style`  | `specs.style`  | inline style declarations            |
| `[x]` (bracket)         | `field.bracket.bare` | `bracket.bare` | the bracket group's bare content     |
| `("x")` (paren)         | `field.paren.bare`   | `paren.bare`   | the parentheses group's bare content |

The `meaning` column is the one piece of this table that exists **only** here: an
assigned field carries no semantics of its own, so a rename never changes what
the content _is_. The `default path` column is what the map carries when the
entry configures nothing — it is the machine-readable
`SpecsKey::default_path` (asserted by
`crates/extra/tests/extras_spec.rs::specs_keys_cover_the_specs_vocabulary`), and
the same seven rows are the doc of `Specs` in `crates/extra/src/typed.rs`.

Three of the seven are **universal**: a DOM renderer writes `specs.class`,
`specs.id` and `specs.style` as the fields they name (`class`, `id`, `style`,
§6.3 rule 1). The other four have no DOM meaning of their own — what a slug, a
title or a slot group _is_ belongs to the element, which reads it in §6.2 and, in
some cases, projects it onto its own DOM field (an anchor's quote → its `title`,
a citation's id → its `cite-id`). A default path an element does not name stays
in the map under its dotted spelling.

An entry that configures **no** `field.<slot>.<kind>` key keeps every default path
above and its element keeps its preset (§6.5, D-7). Once it configures one, the
preset is void as a whole (D-7): a position it leaves unassigned falls back to its
own qualified name (`bracket.bare`, `paren.bare`, `specs.tick`, `specs.class`, …),
never to a name the author chose for another position. A `field.specs.*` or
`field.override.*` key does not void anything — it names an extras item or a
public path, not a slot's home. There is no separate naming key (§14).

#### A moved position is the author's own field

`field.specs.<k>` moves a **position**, not just its spelling. There are five
(`tick`, `quote`, `class`, `id`, `style` — `SPECS_KEYS` in
`crates/extra::field`), and only their **default** paths carry meaning: at
`specs.tick` / `specs.quote` the content is a construct's slug or title, at
`specs.class` / `specs.id` / `specs.style` it is the element's class list, its id
and its inline style (§6.3).

Once an entry assigns a position elsewhere (`field.specs.quote = "_note"`,
`field.specs.id = "foo.id"`), that content is a **public field of the author's
own**: nothing is published at the default path any more, the element neither
reads it (the anchor's id fallback, a citation's `cite-id`, a heading's `title`)
nor renames it (`specs.class` → `class`), and no other element gains a meaning
for the assigned name. `Specs::element_path` in `crates/extra::typed` is that
boundary: it answers the default path for an unassigned position and `None` for
a moved one.

Fixture [`34-specs-assignment`](golden/34-specs-assignment.mdp) is the
executable form of both halves: one entry assigns all five positions and an
unclaimed type in the same document keeps every default.

Every `@@` head may carry the `[x]` / `("x")` groups between its type and its
`{…}` body (`@@type[x]("y"){…}`, `@@type[x]("y")`, `@@[x]("y")`), each part
adjacent (§4.1). Markers place the groups directly after `}}`, before any
adjacent extras head: `{{type}}[x]("y"){…}` / `{{type}}[x]("y")@@head{…}`
(§10.1). The groups' contents behave exactly like a directive head's: `[x]`
stays raw, `("x")` is trimmed and unquoted. The keys are resolved per `type`
like every other positional key (§11 rules 3 and 5).

The groups belong to the head they touch. Since a slot head is claimed by its
tag (§5.3), nothing else competes for them: `{{type}}[x]("y")` is a marker with
both groups under any plugin order, and an anchor is written `@[x]("y")` (§7.2).
The single-group and type-only forms (`{{type}}[x]`, `{{type}}("y")`,
`@@type[x]`, `@@type("y")`) parse the same way.

### 6.2 head vs extras priority

When a construct head and an extras positional fill the same slot, **the head
wins**; the extras value is dropped silently (a `Severity::Warning` MAY be
emitted):

```text
@[Foo](url "Head title")@@type{"Extras title"}   → title = "Head title"
###[slug-a]("Head")@@type{`slug-b`} Foo         → id = "slug-a", title = "Head"
```

Additionally, a plugin that owns an element id (`heading`, `image`, `anchor`,
`cite`, table/cell layers) MUST use `slug` as `id` when no explicit `#id` is
present: `#id` > head slug > extras slug. The **extras** slug counts only while
the tick position keeps its default path — `field.specs.tick = "x"` publishes
the item as `x`, and an element id has no `x` to read (§6.1).

### 6.3 Value typing (IR mapping)

`crates/extra` exposes:

```rust
enum AttrValue {
    Str(String), Int(i64), Float(f64), Bool(bool), Raw(String),
    Object(Vec<(String, AttrValue)>), Array(Vec<AttrValue>),
    JsxExpr(String), Null,
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
  → **`JSON_ATTR_PREFIX` (`U+E001`) + compact JSON** in `value`, so the AST — and
  therefore `plugin-extract` — keeps an object or an array instead of a string
  that merely looks like one (`AttrValue::transport`). `renderer-ast` strips the
  prefix and re-hydrates the value: `renderer-solid` prints it as `k={…}`,
  `renderer-html` as the escaped JSON text (`k="&#123;…&#125;"`), and the
  JSON/AST renderers carry the value itself. Verified: with no data-binding
  plugin enabled, `@[a](/u)@@anchorA{a: {b: 1}}` renders
  `a={{b: 1}}` in Solid and `a="&#123;&quot;b&quot;:1&#125;"` in HTML.
  A **selection path** can address a leaf of it (§21.4: `a.b` is one segment
  deep), which is the reason the value is structured and not text.
  An **internal** `__`-key is the one exception (`AttrValue::transport_for`): it
  keeps the plain literal its consumer parses — `__number`'s
  `{"depth":…,"type":…}` marker is read by `number` before any renderer
  runs (ADR-0010) and never reaches an AST, a renderer or a projection.
- **JSX expression token.** A `{ … }` group whose body is a single identifier or
  dot-separated path — e.g. `{valueA}` or `{frontmatter.fm1}` — is a JSX
  expression rather than an object. It is transported as `JSX_ATTR_PREFIX`
  (`U+E002`) + the expression text, and `renderer-solid` emits it unquoted as
  `k={valueA}` so the value is passed as a variable reference, not a string
  literal. `crates/extra` MUST NOT emit `U+E002` outside this form — only the
  JSX-expression value produces it (`crates/core/src/event.rs`).
- **Structured transport.** A value that structurally **is** an object or an
  array travels as `JSON_ATTR_PREFIX` (`U+E001`) + compact JSON, and
  `renderer-ast` strips the prefix and re-hydrates a real JSON value, which
  `renderer-solid` spreads as `k={…}`. Two producers emit it: `crates/extra`
  (an extras or modifier value — [`AttrValue::transport`]) and `plugin-bind`
  (a `$var`-bearing value, §19). Nothing else may, and an internal `__`-key is
  never prefixed (`AttrValue::transport_for`).
- **`Null` is not authorable.** No head, slot or decorator can spell `null`, so
  the value type gains it only so the public field map can carry an AST value
  the projection reads back verbatim (`crates/extra::project`,
  [ADR-0014](../decisions/0014-field-model.md)): `plugin-bind` can resolve a
  `$var` to `null`, and the DOM renderers already print it —
  `renderer-solid` as `k={null}`, `renderer-html` as `k="null"`.

Renderers:

| Renderer                 | flag                               | typed string | Int/Float                     | structured (`{…}` / `[…]`)          |
| ------------------------ | ---------------------------------- | ------------ | ----------------------------- | ----------------------------------- |
| `renderer-solid`         | `k` (bare JSX shorthand, verbatim) | `k="v"`      | `k="12"` (fallback path only) | `k={…}` (a real object/array, §6.3) |
| `renderer-html`          | `k`                                | `k="v"`      | `k="12"`                      | `k="{…}"` (escaped)                 |
| `renderer-json` / `-ast` | `{ "name": "k", "kind": "flag" }`  | string       | string                        | the value itself (object / array)   |

Custom components receive the **typed** form through the `attrs` map
(`{...attrs}` spread), because that path serialises `AttrValue` as JSON
(`12`, `true`) rather than as text.

> Rationale: bare flags render exactly as authored (`isFoo`) in every renderer;
> the DOM fallback path stays stringly-typed by design.

Omission scope: it applies to plain-element attributes and `{...attrs}` spreads
in `renderer-solid` and `renderer-html`. An **explicit** template binding
(`k={attrs.k}`) is untouched — the template author asked for that slot, so it
keeps rendering `k={""}`.

#### Public field names in the DOM renderers

A public field is a **path** ([ADR-0014](../decisions/0014-field-model.md) rule 1),
so a renderer has to decide what the attribute is _called_. Two rules:

1. **A position's _default_ path becomes its DOM field.** `specs.class` →
   `class`, `specs.id` → `id`, `specs.style` → `style` (`universal_field` in
   `crates/extra::typed`, applied by `project`). It is a rename only: the map is
   ordered by the _public_ path (§6.4), so `class` is written where `specs.class`
   sat. The rename belongs to the **default** path, not to the position: with
   `field.specs.class = "foo.class"` the item is written `foo.class` and never
   `class`, because the author already named it (§6.1). An **element assignment
   wins** the same way — a `field.specs.id` that names `cite-id` is written
   `cite-id` and never `id`, because only the element knows what its id _is_
   (§7). That is why an element projects its own head through
   `Specs::dom_fields(items, …)` instead of leaving the universal path to the
   renderer: both halves (`Specs::element_path` / `universal_field`) test the
   _path_, so a moved position falls through untouched.
2. **Every other path keeps its dotted spelling** — `specs.tick`, `specs.quote`,
   the slot groups `bracket.bare` / `paren.bare`, and an assigned
   `heading.anchor`. A cross-element position has no DOM meaning (§6.1), and the
   path is the field's public identity.

| Renderer                 | `specs.class` | `specs.tick`                                       |
| ------------------------ | ------------- | -------------------------------------------------- |
| `renderer-html`          | `class="…"`   | `specs.tick="…"` (verbatim; HTML allows that name) |
| `renderer-solid`         | `class="…"`   | `{...{ "specs.tick": "…" }}` (JSX has no `a.b=`)   |
| `renderer-json` / `-ast` | `specs.class` | `specs.tick` (the path; no rename happens there)   |

JSX reads `a.b=` as a member expression, not as an attribute name, so
`renderer-solid` emits a dotted field as a one-key spread and keeps the path as
the computed key. The compiled JSX sets the attribute the HTML renderer prints,
so both DOM targets agree and the JSON/AST keep the path untouched.

The rule is the same for a **custom component**: `{...attrs}` in a `template`
spells a dotted field as that one-key spread too
(`<Bibliography {...{ "bracket.bare": "refs" }} …>`), so a field is written
identically whether a component claims the node or the fallback element renders
it. It is never regrouped into a nested prop — `specs={{ tick: … }}` would make
Solid the only renderer that reshapes the path and would set `[object Object]` on
a built-in element; a template that wants nesting spells it itself
(`{attrs.specs.tick}` reads the flat path).

### 6.4 Emission order (deterministic)

A head's contents are assigned into the public field map in this canonical order,
so the map — and with it every golden fixture — is stable:

1. the class position (a single joined value, head classes then `.x` items in
   source order);
2. the id position;
3. the remaining positional positions — the groups first (`bracket`, then
   `paren`), then `tick` and `quote` (§6.1);
4. all remaining props and flags in source order, with the style position where
   the first `style:` prop or `--var` item appeared.

Each step names a **position**, and the step writes it at the path the entry
resolved for it: the default ones are `specs.class` / `specs.id` /
`specs.style`, `specs.tick` / `specs.quote` and `bracket.bare` / `paren.bare`
(§6.1); a moved one is written at the assigned path and never at the default.

Duplicates are [D-12](../workflows/unified-syntax-stage-2/DECISIONS.md)'s rule,
not a merge: two sources for one public path are `Severity::Error`, and the
position **and** the value of the first occurrence win. `class` accumulates
(a `class:` prop joins the `.x` items) and dedupes keeping the first position; a
repeated `style:` property is CSS, so its last value wins with no diagnostic. A
positional item and a same-named prop are such a duplicate. One exception: a
positional **group** (`[…]` / `(…)`, §6.1) beats a same-key body item outright —
the body value is dropped with an §13 `Warning` (§6.2 head-wins style).
Whitespace-only differences MUST NOT change the output.

The list above is the order a head's items are **assigned** in. The DOM renderers
emit the **projection's** order instead: `project` sorts by the public path, which
is what keeps the §6.3 rename (`specs.class` → `class`) from moving the attribute
it writes, and what keeps a moved position's output just as deterministic
([ADR-0014](../decisions/0014-field-model.md)).

### 6.5 The element preset

An element with a slot head declares two pieces of data, both in the owning
plugin's crate (`ElementHead` in `crates/extra::preset`):

- its **contract** — the positions it claims ([ADR-0012](../decisions/0012-universal-slots.md));
- its **preset** — which field each claimed position lands in when the user
  configures nothing.

**Every position is claimed.** A contract lists all six positions the grammar can
carry — `bracket`/`paren` × `bare`/`quote`/`tick` (§5.2) — for all four
slot-contracting elements (`anchor`, `cite`, `heading`, `image`). An element that
claimed only the positions it had a name for would have to drop the rest of the
head, so there is no uncontracted content at all: a position with no preset row
is an ordinary field under its slot-qualified name (D-7), and
`apps/cli/tests/slot_heads_spec.rs` fails if a contract is ever narrowed. Nor may
a _markup_ drop one: an element whose built-in output has no prop for a field
writes it anyway — the built-in citation spells it `data-<slot>-<kind>` (§7.3).

The preset is what turns the groups and items of a _built-in_ construct into its
own public fields (`@[inner](/u "t")` → `inner_text`, `href`, `title`); §7.x
states each element's table, and the crate's `HEAD` constant is the copy the code
reads. A preset names **positions**, never a `specs.*` path: the five specs
positions keep their default paths (§6.1), and what the element makes of them is
§6.2/§6.3.

**The preset is void as a whole** (D-7): one `field.<slot>.<kind>` key in the
matched `custom` entry disables every preset assignment for that entry — there is
no half-preset mode — and each position the entry left unassigned falls back to
its slot-qualified bare name (`bracket.bare`, `paren.quote`, …). The trigger is a
**slot** key because the preset maps slot positions: `field.specs.*` (an extras
item) and `field.override.*` (a public path) leave it alive (`S2-4.2`). A preset
may only sit on a position the contract accepts (`ElementHead::is_consistent`,
asserted by each plugin's own test), so the two halves cannot drift.

The same declaration carries the fields the element writes **itself**
(`ElementHead::owned`, D-12 rule 5: `href` on an anchor, `src` on an image): a
head item that spells one of them is an `Error`, and `field.override.<k>` (D-4)
is how that value gets its own public name. An element that declares nothing owns
nothing, so a construct never inherits a collision rule it did not ask for.

## 7. Construct grammars

Notation: `head?` = optional, `extras?` = optional `@@type{…}`, `∘` = no space.

### 7.1 Image (`plugin-image`)

```ebnf
image     = tag slot-pair extras?
tag       = "!"                                    # mandatory (§5.3, D-11)
slot-pair = "[" contents "]" [ "(" contents ")" ]  # §5.2, paren attached
extras    = §5 extras-head                         # the shared {…} / @@type{…} head
```

| position       | preset name | meaning              |
| -------------- | ----------- | -------------------- |
| `bracket.bare` | `alt`       | the alternative text |
| `paren.bare`   | `src`       | the image source     |

- The tag is **mandatory**, and `![alt](src)` is the whole minimum: the `!` is
  the tag, the bracket is the preset's `alt` and the paren its `src`. A `!` with
  no bracket is literal text, and only the `!` that **touches** `[` claims a head
  — `!![](src)` is the text `!` followed by an image.
- Those two rows are the whole preset (§6.5). `bracket.quote`, `bracket.tick`,
  `paren.quote` and `paren.tick` are read as bare fields under their
  slot-qualified names, so no content of a claimed head is dropped (D-7): an
  image headline goes in a quote, not out of the document.
- `alt` and `src` are **built-in** fields: an extras `alt:` / `src:` is
  `Severity::Error` naming `field.override.alt` / `field.override.src` (§13,
  D-12 rule 5) — `field.override` is the way to publish one under another name.
  `alt` renders even when the bracket is empty, because an empty `alt=""` is the
  one meaningful empty value (§6.3): `![](src)` is a decorative image.
- Extras attach to the `<img>` **itself** — there is no second node — and the
  construct head wins over a same-named extras item (§6.2). An extras
  `` `slug` `` is the `id` fallback when neither `#id` nor a head-supplied id
  exists; `--x: v` folds into `style` like everywhere else.
- The head is **one node**: a line whose whole content is a head is a block image
  (the `<img>` replaces its paragraph), while a head that shares its line with
  text stays inline inside that paragraph. Text after the head is ordinary
  content and is processed by the rest of the chain — it is never a caption.
- Everything the retired marker run used to do is an ordinary extras key (§14,
  D-16): `loading: "lazy"`, `decoding: "async"`, `width: "400"`,
  `height: "300"`. A wrapper (`<figure>`, `<div>`, `<p>`, a caption element) is a
  component's own business: configure `[task.image.custom.img]` (§11) and render
  whatever the site needs.
- §11 has exactly one layer, `img`. `[task.image.custom.figure]` no longer loads
  (§13): the two-node figure is gone with the marker that produced it.

### 7.2 Anchor (`plugin-anchor`)

```ebnf
anchor   = tag slot-pair extras?
tag      = "@"                                  # mandatory (§5.3, D-11)
slot-pair = "[" contents "]" [ "(" contents ")" ]   # §5.2, paren attached
extras   = §5 extras-head                       # the shared {…} / @@type{…} head
```

| position       | preset name  | meaning                  |
| -------------- | ------------ | ------------------------ |
| `bracket.bare` | `inner_text` | the text the `<a>` wraps |
| `paren.bare`   | `href`       | the link target          |
| `paren.quote`  | `title`      | the link title           |

- The tag is **mandatory**. Slots are universal (§5.2), so an untagged
  `[text](url)` belongs to no element: the anchor does not claim it, the text
  reaches `plugin-markdown` as an ordinary CommonMark link, and an extras head
  written after it stays literal text (§14).
- The tag is read wherever a head starts, not only at the line head: an anchor is
  inline by nature (`see @[the docs](/docs) for details`).
- Those three rows are the whole preset (§6.5). `bracket.quote`, `bracket.tick`
  and `paren.tick` are read as bare fields under their slot-qualified names, so
  no content of a claimed head is dropped (D-7).
- `href` is a **built-in** field: an extras `href:` is `Severity::Error` naming
  `field.override.href` (§13, D-12 rule 5). `title` is not — the extras
  `"quote"` is the link's title only when the head has none (§6.2).
- Extras attach to the `<a>`, and the construct head wins over a same-named extras
  item (§6.2). An extras `` `slug` `` is the `id` fallback when neither `#id` nor
  a head-supplied id exists.
- There is **no** automatic external-link handling and **no** URL-suffix modifier
  (§14, D-16): `target`, `rel`, `hreflang`, … are ordinary extras keys, and `^`,
  `!`, `$`, `;;`, `--` at the end of a target are ordinary characters of it.
- The bracket's `|…` run is §5.4's slot modifier run: each symbol contributes the
  fields its `bracket.modifier."<symbol>"` table names, merged in written order
  ([ADR-0017](../decisions/0017-slot-modifiers.md), D-9). A symbol with no config
  contributes nothing and is not a diagnostic, and Pendon ships no built-in
  modifier; the run is pure configuration, so it never overrides an authored
  field of the head (§6.2). Fixture 43 asserts the whole rule.
- Image syntax `![alt](src)` is another element's head (§7.1), and reference-style
  links and nested brackets are unsupported.

### 7.3 Cite (`plugin-cite`)

```ebnf
cite     = "[" tag bracket [ "(" slots ")" ] extras?
tag      = "^"                                  # inside the bracket (§5.3)
bracket  = slots                                # reference_id, locator
extras   = §5 extras-head                       # the shared {…} / @@type{…} head
```

```mdp
[^reference_id "locator" `bracket tick`](paren bare "paren quote" `paren tick`)
```

- The tag is the caret **inside** the bracket: `[^book]` is a citation, `[book]`
  is not (`plugin-markdown`'s text), and `[^^](book)` is not either — its second
  caret becomes the reference `^` (§14).
- `bracket.bare` → `reference_id`, `bracket.quote` → `locator`. **`reference_id`
  is mandatory**: a head that assigns it nowhere (an empty bracket, a quoted-only
  one, or an entry whose `field.*` voided the preset and did not redefine it) is
  an `Error` and the text stays as written. `locator` is optional.
- Every other claimed position (the tick, the whole paren) is an ordinary field:
  unassigned positions keep their slot-qualified path (`bracket.tick`,
  `paren.bare`, `paren.quote`, `paren.tick`) per D-7.
- Positions are movable: `field.bracket.tick = "reference_id"` +
  `field.bracket.quote = "locator"` is the documented way to put the reference in
  the tick (voiding the preset renames **every** row, so both must be assigned —
  §6.1). The processor reads `reference_id` / `locator` **by name**, so a moved or
  publicly renamed field (`field.override.reference_id = "ref_id"`) never changes
  numbering or identity (D-4).
- Extras attach to the citation node and MUST touch the closing `)`; an item
  that spells a name the head produced is dropped with a warning (§6.2). An
  extras `#id` / `` `slug` `` feeds the node's `cite-id`, and an extras quote is
  its `title` (§6.3).
- The citation's fields are `reference_id`, `locator` and `citation_number` (the
  display ordinal, assigned by the processor; a repeated
  `(reference_id, locator)` keeps its number). There is no `instance_index`
  (§14) — a per-occurrence ordinal is `number`'s job (§10.5).
- Without a `cite` component the built-in markup is
  `<sup class="cite-ref"><a href="#citeref-1-book" id="cite-1">[1]</a></sup>`:
  the class and the two prefixes are fixed, because a site that wants its own
  spelling configures a `Cite` component (§7.3 config above, §14). The markup is
  fixed, the fields are not: the four positions it has no prop for are written
  anyway, as `data-<slot>-<kind>` (`bracket.tick` → `data-bracket-tick`). A
  dotted path is not an attribute name in JSX (§6.3) and raw markup cannot spread
  one, so the `data-` prefix — the convention the built-in `<img>` already uses
  for a key it has no attribute for — is what keeps a citation head from losing
  content (fixture 37). `reference_id` stays the `href`, `citation_number` the
  label, and `locator` the `citations` binding (§20), as before.

### 7.4 Heading (`plugin-heading`)

```ebnf
heading  = "#"(1..6) [ "[" bracket "]" [ "(" paren ")" ] ] extras? text
extras   = §5 extras-head                       # the shared {…} / @@type{…} head
```

- **The tag and the depth share one run (D-11).** The `#` **immediately before the
  bracket is the tag** that claims the head, and the hashes in front of it are the
  depth: `##[slug]` is a **depth-1** heading, `######[]()` a depth-5 one. A run
  with no bracket claims nothing and keeps its whole length, so `## Title` is
  depth 2. A claimed head never goes below depth 1: `#[slug]` is still `<h1>`.
- The head MUST touch the `#` run (§4.1) and the extras head follows it
  adjacently: `###[slug]("Title")@@type{…} Title`.
- `[slug]` and `("title")` are both optional and independent — but the parentheses
  belong to the bracket: `("Title")` written without one is not a head at all
  (§4.3), it is text.
- `("title")` is **not** the `innerText`; it is a separate attribute.
- Whitespace between the head and the heading text is consumed by the head (it
  MUST NOT become part of the title, nor of `inner_text`).
- **Fields.** `depth` (`1..6`), `slug`, `title`, `id`, the extras attributes and
  `inner_text` (the visible text — the node's content, never an attribute).
  `slug` falls back to a slug of `inner_text`; `id` is the extras `#id`, else the
  `[slug]`, else the extras `` `tick` `` — which the element reads through
  `specs.tick` while that position keeps its default path (§6.1, §6.2) — else the
  auto slug.
- Extras attach to the heading element; the construct head wins over an extras
  slot filling the same field (§6.2, e.g. an extras `` `tick` `` when `[slug]`
  exists).
- With `plugin-section` owning the outline the heading emits **neither** `id` nor
  `slug`; the id transfers to the section (§9.5).
- The heading does **not** number itself: `[[task.heading.number]]` and
  `%%numberType` are `number`'s (§10.5), which reads `depth` as the
  natural depth and writes `attrs.number`. An authored `number:` prop passes
  through untouched when no block selects the heading.

### 7.5 Wiki (`plugin-wiki`)

```ebnf
wikilink = "[[" target [ "|" label ] "]]" extras?
extras   = §5 extras-head                       # the shared {…} / @@type{…} head
```

- Extras attach to the wiki `<a>`; `href` is produced by the plugin
  (`link_prefix`) and MUST NOT be overridable by extras (it wins).

## 8. Table (`plugin-table`)

```ebnf
table       = decorator? caption? header-row row* [ foot ]
decorator   = §9.1 decorator line                   # the <table> head, D-10
caption     = "||" extras? content "||"
header-row  = "|" cell ( "|" cell )* "|" extras?     # alignment/width row
row         = "|" cell ( "|" cell )* "|" extras?     # body row
foot        = "|===|" extras? row*
cell        = "|" extras? content

extras   = §5 extras-head                           # the shared {…} / @@type{…} head
```

Mapping to table layers:

| Source                               | Layer / node                                 |
| ------------------------------------ | -------------------------------------------- |
| §9.1 decorator line above the table  | `<table>` (`table`)                          |
| `\|\| … \|\|` line                   | `<caption>` (`caption`)                      |
| alignment-row cell extras            | the `<th>` of that column (`thead` / column) |
| alignment-row **end-of-line** extras | `<tbody>` (`tbody`)                          |
| body-row cell-front extras           | that `<td>` (`cell`)                         |
| body-row end-of-line extras          | that `<tr>` (`row`)                          |
| `\|===\|` line extras                | `<tfoot>` (`tfoot`)                          |
| foot-row cell-front extras           | that `<td>` (`cell`)                         |
| foot-row end-of-line extras          | that `<tr>` (`row`)                          |

The `<table>` is the one layer whose head is **outside** the table: the §9.1
decorator line directly above it, claimed by this pass under D-10 (claim order
section → table → list → blockquote → paragraph). Nothing written _inside_ a
table configures the `<table>` itself, and the retired `|-[slug](…)extras-|`
declaration line is literal text (§14).

The other six layers read the **same** head (§5) —
`@@type?[…]("…"){…}%%numberType` — so a slot pair, the specs items and a
numbering marker reach whichever layer the head sits on, and the layer keys above
are only _where_ the head is read. What a head names is not layer-specific:
`@@cellA[slug]("t"){.x}%%%numberA` puts `bracket.bare` / `paren.bare` on the
cell, `class="x"` with it, and the `__number` transport `number` reads
(§10.5). Golden `36-table-slots` is the seven-layer proof.

A column's extras stay on its `<th>`: the alignment and the width cascade into
the column's `<td>`s, the rest does not (OPEN-E1), so an attribute or a number is
never duplicated on every body cell.

Rules:

- **Slot placement (§4.1).** A table has **no** adjacency exception: every head
  touches the `|` (or the `||`) it belongs to. The one head that is not adjacent
  is the `<table>`'s own decorator line (§9.1), which belongs to the table by
  position, not by touching.
  - A cell head sits directly after its opening `|`: `|{.x} text |`,
    `|@@cellB{.lead} text |`; `| {.x} text |` is literal text.
  - A caption head sits directly after `||`: `||{.c} caption ||`.
  - A trailing row/section head sits directly after the line's **last** `|`:
    `| a | b |{.row}` and `|===|@@tfootX{.total}`.
  - The head of a **delimiter cell goes after the alignment code and touches it**:

    ```text
    | :---(200px)@@cellA{.v-top} | :---:{.c} | ---:(30%) | :---@@cellB{.a, #i, k: "v", n: 2} |@@tbodyX{.tb}
    ```

- **The `<table>` head is a decorator line** (§9.1, D-10): `@@tableX{…}` on the
  line above the table, the same head every other construct uses, with slots,
  specs and numbering. The harness hands the bound decorator to this pass only
  when the pass declares `Receiver::Table` (D-10 rule 2) — the claim order is
  section → table → list → blockquote → paragraph, and the owner follows **what
  the block is** (`|` → table), never which plugin read the stream first.
- The retired declaration line (`|-[slug](…)extras-|`) is literal text (§14); the
  table is then untyped and its layers fall back to plain elements.
- A decorator no pass can claim — including a table head when `plugin-table` is
  not in the chain — is removed with a `Warning` (D-10 rule 3).
- Bare `===` as a foot separator is **retired**; only `|===|` remains.
- Alignment/width tokens (`:---:`, `(200px)`) and the `>` (colspan `+1`) / `^`
  (rowspan `+1`) cell markers are unchanged and come **before** the cell extras.
- Cell content may be empty; empty cells become `<td></td>`.

## 9. Decorators, blockquotes and lists

### 9.1 The decorator rule (general)

A **decorator line** is a line whose entire content is an extras head **with the
`@@` sigil** — `@@type{…}` / `@@{…}` / `@@type` — with optional leading
indentation and nothing after it but whitespace (text separated by whitespace is
dropped with a warning while the head still decorates, below). The sigil is
mandatory
(D-10): it is what makes a standalone head a decorator, because without it a head
only ever attaches to a construct it touches (§4.1). The head reads in one order —
`@@type` → slot → `{specs}` → `%%numbering` (§5). A decorator decorates the
**next block node** at the same nesting level and is consumed (MUST NOT be
rendered):

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
@@aside[intro]("Aside title"){.lead}%%%numberA
Decorated with the bracket / parentheses slots, the specs, and a number marker.

@@[intro]("Aside title")
The untyped form: `@@` is required, so the line can never be a link.

{.lead, #nope}
A bare head on its own line is literal text — no `@@`, no decorator (§14).

[intro](url)
A paragraph line like this stays a link — without `@@` it is not a decorator.
```

- The decorator's `type` (when present) is the **type marker** of the decorated
  node and drives per-type component selection (§11).
- The `@@` sigil is what separates a decorator line from a link or a bare head at
  the start of a line: the groups need it (`@@[…]`), and a `@@`-less line is
  never a decorator — `[…]` stays a link (§4.1) and `{…}` stays literal text
  (§14). `@@` first is also what keeps the head order unambiguous: a type written
  after the body (`@@{.a}aside`) is not a head at all, so the line is literal.

- A decorator applies to a **paragraph** or a **code fence** (`plugin-markdown`),
  a **list container** (§9.3), a **blockquote** (§9.2), a **table** (§8) or a
  **section** — the block a decorator line sits above a **heading** decorates is
  the section, not the heading (§9.5). As a spellable rule the **block** decides
  its owner (D-10 rule 2), and the owners are ordered most specific first:

  | # | receiver     | the block       | plugin       |
  | - | ------------ | --------------- | ------------ |
  | 1 | `Section`    | a heading       | `section`    |
  | 2 | `Table`      | a `\|…\|` line  | `table`      |
  | 3 | `List`       | a `-`/`1.` line | `list`       |
  | 4 | `Blockquote` | a `>` line      | `blockquote` |
  | 5 | `Paragraph`  | anything else   | `markdown`   |

  The claim is a **classification, not a race**: `pendon_extra::classify_block`
  names the receiver from the block alone, so the pass that owns it binds the
  decorator **whichever order the chain runs the plugins in**. A pass that meets
  a decorator for another receiver leaves it in the stream untouched. Inline
  constructs never receive one — they use the adjacent form (§4.1) — and
  `heading`, `marker`, `directive` and frontmatter/data blocks own no receiver at
  all.
- A decorator no pass will ever claim is **removed with a `Severity::Warning`**
  (D-10 rule 3): the block can never receive one (a thematic break, a marker, a
  directive, frontmatter, a data block, an already-emitted element), or the block
  belongs to a receiver the chain does not have (a heading without `section`, a
  table without `table`, …). The last binder is the one that reports it.
- If no following block exists (end of input, or the next sibling is not a
  block), the decorator is dropped the same way. Every decorator line carries
  `@@`, so there is no ambiguous spelling that stays in the stream.
- Trailing text is judged by adjacency (§4.1, D-10 rule 3): text that **touches**
  the head is part of its spelling, so the line is a head-after-body and literal
  text (`@@{.a}aside`); text that is separated from the head by whitespace is
  **dropped** with a `Severity::Warning` while the head still decorates the block
  — the head is never silently re-read around it.
- Consecutive decorators: only the **last** one applies; earlier ones are
  dropped with a `Severity::Warning` (MAY be an error later).
- Indentation MUST be `<=` the indentation of the decorated block; a decorator
  indented deeper than the following block is literal text.

### 9.2 Blockquote (`plugin-blockquote`)

```ebnf
blockquote       = ">" ws content                  # `>` stays plain Markdown
decorator_line   = "…@@type{…}" above it           # §9.1 is the only head
```

- A blockquote has **no head of its own** (§9.2): no slot contract, no preset, no
  owned fields. The only way to configure one is the decorator line directly
  above it (§9.1), which binds before `plugin-markdown` parses the content.
- Everything inside the `>` is **content**: `> @@type{…} text` is literal text
  (the head is not read there), and a slot inside the body belongs to whichever
  element claims it — the quote reads none of them.
- The paragraph wrapping the quote becomes a built-in `NodeKind::Element("blockquote")` tagged
  `__layer = "blockquote"` (§11 rule 3, ADR-0008 `D-3a`): the plugin never swaps in a
  component — the renderer resolves the layer, and a quote no component answers renders as
  the built-in element.
- Nodes are marked `__plugin_kind = "block"` so the body is re-lexed as block-level Markdown (headings, lists, nested quotes).
- Nesting is preserved one level at a time; `>> inner` leaves `> inner` for the subsequent Markdown pass.

### 9.3 List (`plugin-list`)

```ebnf
list_decorator = "@@" type? extras?
marker         = "-" | "*" | "+" | DIGIT+ "."
```

- This plugin binds extras to list containers **before** `plugin-markdown` parses content; markers remain plain Markdown.
- A decorator line directly above a list targets the container layer: `unordered` for `-`/`*`/`+`, `ordered` for `1.`.
- The layer is determined strictly by the **marker**, never by the decorator's own type name.
- The container has **no head of its own and no preset**: no slot contract, no
  `ElementHead`, and nothing is read from the items — the decorator line is the
  only source of the container's fields (S2-4.8). A `@@type{…}` written at the
  start of an item is content, and the items' own constructs (an anchor, a nested
  list) are parsed by the elements that claim them.
- Container nodes stay `NodeKind::Element("ul"|"ol")` (with `__plugin_kind`) tagged
  `__layer = "unordered"|"ordered"` (§11 rule 3, ADR-0008 `D-3a`): the plugin never emits
  `NodeKind::Custom`, the renderer resolves the layer.
- Extras land on the wrapper element or component, ensuring attributes apply to the list itself rather than surrounding it.
- Ordered list start offsets follow CommonMark behavior (first item number); extras cannot currently override this value.
- Decorator lines that fail to bind to a valid list trigger a `Severity::Warning` diagnostic.
- Item-level decorators (L2/L3) and nesting validation (L5) are defined in spec but not yet implemented.
- The decorator's positional items are §6.1's: they land on the default paths
  (`specs.tick` → `specs.quote` → …) unless the entry renames them with
  `field.specs.*`, and an assignment moves the item, so this layer stops reading
  it. There is no list preset to void — the container has none.
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

**A section has no head of its own.** It contracts no slot position and declares
no `ElementHead`: there is no `[bracket]` for it to claim. Its fields are the two
it derives, which is its whole preset assignment (§6.5, D-7):

| field   | from                                                                                                                 |
| ------- | -------------------------------------------------------------------------------------------------------------------- |
| `id`    | the id chain below; absent on the preface and on a `<--->` child                                                     |
| `depth` | the outline level it stands for: the heading's own `depth` (§7.4), a `<--->` child's parent + 1, `0` for the preface |

Because the depth is the heading's, a section and the heading inside it agree,
and a `<--->` child is exactly one deeper than its parent. `depth` is a public
field, so a component receives it as a prop and the built-in `<section>` prints it
as an attribute.

- A decorator line directly above a heading — `@@type{…}` / `@@{…}` (optional
  indentation; a blank line before the heading is allowed) — decorates the
  **section**, not the heading: the heading already owns its extras on its `#`
  run (§7.4). That decorator is the section's head in every respect but spelling
  (the claim table of §9.1 gives a heading to `section`), so its `type`,
  `class` and remaining items attach to the `<section>`.

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

- `type` is **mandatory** for markers; `{{}}` is literal text. The marker's own
  type **silently** overrides an extras `@@type` (one node, one routing key: no
  error, no warning), and `type` is its only built-in field — a marker contracts
  no slot position, and its text is its children, never an `inner_text`
  attribute.
- Extras are optional but must be adjacent.
- The positional groups (§6.1) sit directly after `}}`, **before** any extras
  head: `{{type}}[x]("y"){…}` and `{{type}}[x]("y")@@head{…}`. They map through
  the marker type's `field.bracket.bare` / `field.paren.bare`. A marker-level group
  beats a same-key group of the extras head (§6.2, dropped with a §13 Warning);
  a malformed group stays literal text while the marker still renders (§4.3).
  The groups belong to the head they touch: an anchor is `@[x]("y")` (§7.2), so
  no other element competes for a marker's groups.
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

- `[bracket]`'s field = `field.bracket.bare` (default path `bracket.bare`),
  `("paren")`'s = `field.paren.bare` (default path `paren.bare`) (§6.1).
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

In both forms `type` is the directive head's own field: it is never extras-derived
and is not a slot position, so an extras head cannot set it and a decorator naming
another type cannot replace it.

### 10.4 Priority summary

| Slot                     | Winner                                                      |
| ------------------------ | ----------------------------------------------------------- |
| element `id`             | `#id` > head slug > extras slug                             |
| `title`                  | head `("…")` > extras `"…"`                                 |
| `class`                  | union of head classes and `.x` items, in source order       |
| `href` / `src` / `start` | construct-owned, never overridable                          |
| positional group vs body | the group (`[…]` / `(…)`) wins; body value dropped (§6.4)   |
| `number`                 | `%%numberType` marker > `[[task.<element>.number]]` (§10.5) |
| any other key            | head prop > extras prop                                     |

### 10.5 Numbering (`number`)

`%%numberType` and `attrs.number` are one plugin's job
([ADR-0010](../decisions/0010-plugin-number.md)); no element plugin numbers
anything itself.

```ebnf
number-type = ALPHA { ALPHA | DIGIT }                # `%%numberB` → "numberB"
style       = number-system | "[" number-system { "," number-system } "]"
number-system = "decimal" | "lower-roman"  | "upper-roman"
              | "lower-alpha" | "upper-alpha" | "lower-latin" | "upper-latin"
              | "lower-greek"
```

- **Grammar** (shape, positions, depth normalization, the literal cases) is §5
  and lives in `crates/extra`. A marker is transported as the internal attribute
  `__number = {"depth":n,"type":"…"}` and is **stripped by every renderer**,
  exactly like a `__`-prefixed key.
- **Config** is repeatable and order-sensitive; a block _is_ a counter:

  ```toml
  [[task.<element>.number]]     # `<element>` is a chain token: heading, anchor, image, …
  type = [ … ]                  # §11 selectors: `*` / literal / `_` / `?`; omit = `*`
  style = "…"                   # one system = every depth
  style = [ "…", "…" ]          # entry n−1 = the n-th visible depth, `decimal` beyond

  [[task.number.custom]]        # the task-level group a `%%numberType` names
  type = [ "NumberA" ]
  style = "upper-alpha"
  ```

- **Depth.** An element without a natural depth is depth 1 (a heading uses its
  own `depth`, §7.4). The two kinds of block differ here:
  - **Element blocks** are relative: the **smallest depth is decided across the
    whole document**, so a `##` written after `###` sections renumbers them and
    `style[0]` is the group's smallest occurring depth, not necessarily 1.
  - **Marker blocks** are absolute: a marker's depth is `%`-count − 1 and always
    counts from 1, so `%%%` is depth 2 on any element and a missing parent counts
    as the style's first value (`I`, `A`, `1`).
- **Output** is `attrs.number`: an array of exactly `depth` strings
  (`["1","2","a"]`), carried through ADR-0005's structured transport so it is a
  real array in Solid. The plugin never writes the number into the text — a
  display format (`1.2.a.`) belongs to the component.
- **Matching.** The marker's type is matched against `[[task.number.custom]]`
  selectors **case-insensitively** (the source spelling is CamelCase,
  `%%numberB`). Element types keep §11's case-sensitive literals.
- **Authored `number:` (D-10).** While a `number` block selects the element, an
  authored `number:` prop is reserved: it becomes a diagnostic instead of a
  competing attribute. With no block selecting it, the prop passes through as an
  ordinary extras value (golden `06-heading`).
- **Order.** `number` MUST follow every element plugin producing numbered nodes
  and precede `markdown`; the CLI rejects any other position with a config error
  naming both plugins.
- A construct that owns the rest of its line is required for a marker to attach:
  `![alt](/a.png)%%numberA` numbers the image (a head alone on its line is a
  block), and `![alt](/a.png)%%numberA x` numbers the same image written inline,
  with `x` as ordinary paragraph text (§7.1). Citation numbering stays in
  `plugin-cite`.
- Frozen by golden `30-number-heading` (element numbering) and
  `31-number-custom` (`%%` markers across heading/anchor/image).

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

[task.image.custom.img] # <img>
[task.wiki.custom.anchor] # <a> produced by plugin-wiki
[task.wiki.custom.infobox] # infobox (deleted in the next workflow)
```

A plugin that declares **one** layer may address that layer directly, using the
plugin's own key as the layer name (the _primary layer_): `[[task.anchor.custom]]`
above is shorthand for `[[task.anchor.custom.anchor]]`.

`custom` is **always an array of tables** (`[[task.<plugin>.custom]]`). A single
`[task.<plugin>.custom]` table is a **load error**: it reads both as one entry
and as a map of layers, so it is never guessed
([D-8](../workflows/unified-syntax-stage-2/DECISIONS.md),
[ADR-0018](../decisions/0018-clean-replacement.md)).

An entry is either a **component** (`name` / `template` / `imports`) or
**configuration** — `field.*` keys ([ADR-0014](../decisions/0014-field-model.md))
and/or `bracket.modifier."<symbol>"` tables (§5.4,
[ADR-0017](../decisions/0017-slot-modifiers.md)) — or both. A configuration-only
entry is a definition on its own ([D-8](../workflows/unified-syntax-stage-2/DECISIONS.md)):
it emits **no template**, so the layer keeps rendering its built-in element while
still reading that config. A fields-only entry with `type` configures that type
only; without `type` it is the element-wide default, and a typed entry's config
beats it. An entry with neither a component nor any configuration is a load error
("empty custom entry").

Rules:

1. A layer key MAY be a **single table** (that layer's default component) or an
   **array of tables** (`[[task.table.custom.thead]]`, typed entries plus at
   most one default). Both forms are equivalent to "the default component for
   that layer". The layer name is the element the layer emits (`table`, `thead`,
   `caption`, `image`, `figure`, `list`, `unordered`, `ordered`, …).
2. `type` accepts a single string or an array; `[]` is equivalent to an absent
   `type`. At most one default per layer; two defaults is a **hard error** at
   config load.

   Each entry is a **selector**: a literal marker name, `?` (the untyped
   element, i.e. one with no marker), `_` (every occurring type that no entry in
   the layer names) or `*` (every occurring type).
   Specificity is per selector — literal / `?` = 2, `_` / `*` = 1 — and an equal
   specificity is resolved by **later registration**. A selector list is never a
   glob: `??` / `**` are rejected at load. A literal may appear once per layer
   (twice is a hard error); `?` / `_` / `*` may repeat and stay ordered.
3. Unmatched type selection order: exact `type` match → layer default → built-in
   fallback node (which MUST still carry all extras as attributes).

   Wildcards need the **document's** occurring types, so they are resolved once
   per document by the renderer rather than per element: `_` expands to the
   occurring types that **no entry in the layer names** (naming is computed for
   the whole layer before any resolution, so registration order cannot change
   what `_` means), `*` expands to all of them, and `?` to the untyped element.
   The untyped element counts as an occurring type and only `?` **defines** it,
   so `_` answers it whenever the layer configures no `?` entry
   ([ADR-0008](decisions/0008-type-selectors.md) §3).
   Only the **winning** entry's `imports` are emitted. Positional keys (rule 5)
   stay literal-only: a `_` / `*` entry contributes no positional (`field.*`)
   override.

   > **Cutover (OPEN-C3, `D-3a`).** Every element plugin emits its own built-in
   > node, keeps the `type` marker it already emits and tags the node with the
   > layer that owns it (the internal `__layer` attribute, which every renderer
   > strips before rendering — `__`-prefixed keys are internal markers). The
   > renderer then answers the whole document at once: one `pendon_types::Group`
   > per layer, so `_` / `*` see the document rather than a single element, and
   > the plugin never picks a component itself. Components that are **not** §11
   > layers (`plugin-custom`'s `toml:` specs, `plugin-quiz`, `plugin-vicado`)
   > keep the older node identity instead. `quiz`, `vicado` and the wiki
   > infobox are scheduled for deletion, so they are deliberately **not**
   > migrated to `__layer` / `layer_templates` — ignore them when changing §11.
4. `imports` is canonical. The legacy singular `import` is accepted with a
   deprecation `Severity::Warning` (see §14 OPEN-C2).
5. Positional naming (per component, all layers): `field.specs.tick`,
   `field.specs.quote` (§6.1) — plus `field.specs.class` / `.id` / `.style` for
   the universal DOM positions. Directive heads additionally use
   `field.bracket.bare`, `field.paren.bare`. An assignment moves the position,
   so the element stops reading it (§6.1). A removed naming key is a load
   error, not a warning (§13, §14).
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
   - _construct_ sentinels (`image`, `anchor`, `cite`, `heading`, `wiki`, `table` layers,
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
(`<p>`), `Blockquote` (`<blockquote>`), `Heading` (`<h*>`, which reads `depth`
and skips the template-only `slug`), `Section` (`<section>`), `CodeFence`
(`<pre>`, skipping `raw_html`),
`BulletList` (`<ul>`) and `OrderedList` (`<ol>`, where `start` is emitted
numerically); the HTML/JSON/AST renderers mirror this. The fallback path emits
`id`, `class` and `style` and passes the remaining keys as attributes.

Known gap: `ListItem` (`<li>`) is rendered by the fallback path without its
attributes — this is the item layer (L2/L3, §9.3), which is still pending.

## 13. Diagnostics

| Condition                                                                            | Severity                                           |
| ------------------------------------------------------------------------------------ | -------------------------------------------------- |
| malformed head / missing `}` / whitespace-separated head                             | none (literal text)                                |
| a retired legacy form (`[.c,#id]{k:v}`, `[^^]("ref")`, …)                            | none (literal text)                                |
| a cite head that assigns no `reference_id` (§7.3)                                    | **Error (text stays literal)**                     |
| duplicate `#id` / `` `slug` `` / `"title"` / prop / flag in one head                 | **Error (first wins)**                             |
| extras `k` naming a built-in field of the element (`href`, `src`, `citation_number`) | **Error (construct wins; use `field.override.k`)** |
| repeated `style:` property or `--var` (last value wins, CSS)                         | none                                               |
| extras `id` overriding a head slug                                                   | Warning                                            |
| decorator with no following block                                                    | Warning                                            |
| dropped decorator (consecutive decorators)                                           | Warning                                            |
| dropped extras value because a head value won                                        | Warning                                            |
| unclosed block directive at EOF                                                      | Warning                                            |
| unknown plugin name in `task.plugin`                                                 | Warning                                            |
| unknown `task.<plugin>` key in `pendon.toml`                                         | **Error (config load)**                            |
| unknown key inside a `custom` entry                                                  | **Error (config load)**                            |
| template without `{children}` (non-void element)                                     | Error (config load)                                |
| unbalanced template element                                                          | Error (config load)                                |
| two default components for one layer                                                 | Error (config load)                                |
| `start:` prop on `ol` extras                                                         | Warning (ignored)                                  |
| literal `U+E000` sentinel character in source                                        | Warning (stripped)                                 |
| unknown key outside a known layer                                                    | Warning (passed through)                           |

Config-load errors MUST abort the build before any file is processed.

## 14. Retirement and collisions

| Removed                                                                 | Replaced by                                                                          | Note                                                                                                                                                                                                                                                                                                              |
| ----------------------------------------------------------------------- | ------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `[task.cite.section]`                                                   | `plugin-marker` (`{{bibliography}}`)                                                 | **removed**: `CiteOptions.section` and `CitationContext::replace_section_markers` are gone; the section component is now a marker.                                                                                                                                                                                |
| `{{section}}` in sources                                                | marker block                                                                         | **removed**: the `{{ footnote }}` lines in `sandbox/cite` and `sandbox/img` are gone; `{{…}}` is free for `plugin-marker`.                                                                                                                                                                                        |
| `:::name` custom blocks                                                 | `==type` block directive                                                             | deprecated alias kept for one release (Warning), then hard-error — §15 OPEN-R1.                                                                                                                                                                                                                                   |
| table declaration line (`\|-…-\|`)                                      | the §9.1 decorator line above the table (§8)                                         | **not syntax** (D-16, D-10): a table's head is the decorator, so the retired line is literal text and its extras configure nothing; golden `08-table-head` is the proof.                                                                                                                                          |
| bare `===` table footer                                                 | `\|===\|`                                                                            | **removed**: only `\|===\|` remains.                                                                                                                                                                                                                                                                              |
| `o.` ordered-list offset                                                | first item's number                                                                  | **removed**: an `ol`'s `start` comes from the first item and is not overridable (§9.4).                                                                                                                                                                                                                           |
| `plugin-quiz` (`:::quiz`)                                               | —                                                                                    | retired from the pipeline; plugin code stays, deep rework later.                                                                                                                                                                                                                                                  |
| legacy `[.c,#id]{k:v}`                                                  | `{…}` / `@@type{…}`                                                                  | **removed** (D3): plain literal text, no warning, no deprecation path. Gone from the code (`parse_attrs`, `ExtraAttrs`, `legacy_extras_warning`), docs, sandboxes and READMEs.                                                                                                                                    |
| `import = "…"` (string)                                                 | `imports = ["…"]`                                                                    | **removed**: the singular form is accepted with a Warning (§15 OPEN-C2), then hard-errors.                                                                                                                                                                                                                        |
| `[task.<plugin>.custom_node]`                                           | `[task.<plugin>.custom.<layer>]`                                                     | **removed**: `custom.<layer>` is read for `anchor`, `image` (`figure`), `heading`, `cite` and all seven `table` layers (`apps/cli/src/plugins.rs`); every sandbox/fixture config is migrated. The stale key fails the build with a migration message. Unwired layers (`task.image.custom.img`) are config errors. |
| `[task.heading]` `auto_number` / `number_style`                         | `[[task.heading.number]]`                                                            | **removed** (ADR-0010): both keys are read only to raise the §14 migration error. Numbering is `number` (§10.5), which writes `attrs.number` as an array and never prefixes the text.                                                                                                                             |
| anchor URL-suffix modifiers (`^`, `~`, `!`, `$`, `;;`, `--`)            | —                                                                                    | **not syntax** (D-16, §7.2): deleted, not replaced. A modifier-shaped tail is now part of the URL, and the `target` / `rel` tokens they produced are ordinary extras keys; the `^`/`~` conflict warning went with them. Slot modifiers are §5.4's, and Pendon ships none.                                         |
| image marker run (`?`, `~`, `w<n>`, `h<n>`)                             | extras keys `loading:` / `decoding:` / `width:` / `height:`                          | **removed** (D-16, §7.1): `~?!![](src)` is the text `~?!` followed by an empty-alt image. A flag was never a field name, so the marker is not another spelling of one.                                                                                                                                            |
| image `!!` figure, trailing caption, `<figure>` / `<figcaption>`        | the text after the head / a `[task.image.custom.img]` component                      | **removed** (D-16, §7.1): the head is one `<img>` node, so a caption is ordinary paragraph content and a wrapper is the component's own markup. `[task.image.custom.figure]` no longer loads (§13).                                                                                                               |
| image container prefixes `p!` / `d!`                                    | a component on the `img` layer                                                       | **removed** (D-16, §7.1): the prefix spelled an element name, which is config, not syntax — `p![alt](src)` now renders `p` and an image.                                                                                                                                                                          |
| automatic external-URL `target="_blank"` / `rel="noopener"`             | —                                                                                    | **not syntax** (D-16, §7.2): the anchor no longer classifies URLs, and no config key restores it — write `target` / `rel` in the extras when a link needs them.                                                                                                                                                   |
| untagged `[text](url)` anchor                                           | `@[text](url)`                                                                       | **not syntax** (D-16, §7.2): the slot tag is mandatory, because the bracket and paren are universal (§5.2). Without it the text is an ordinary CommonMark link and a trailing extras head stays literal text.                                                                                                     |
| blockquote inner head (`> @@type{…} content`)                           | decorator line above the quote                                                       | **not syntax** (D-16, §9.2): a quote has no head, so everything inside `>` is content and `@@type{…}` written there is literal text. The decorator line (§9.1) is the only way to configure a quote.                                                                                                              |
| bare / `type`-last decorator line (`{…}` on its own line, `@@{…}type`)  | `@@type` → slot → `{specs}` → `%%numbering`                                          | **not syntax** (D-16, §9.1): the `@@` sigil is mandatory on a decorator line and the type is its first part, so `{.lead}` alone is literal text and `@@{.a}aside` is not a head at all. Neither configures the block below it; the bare head is still canonical where it **touches** its construct (§5, §8).      |
| `backtick_key` / `quote_key` / `bracket_key` / `parentheses_key`        | `field.specs.tick` / `field.specs.quote` / `field.bracket.bare` / `field.paren.bare` | **removed** (D-16): the four naming keys are gone from the loader, so a config that still uses them **fails to load** with an unknown-key error (§13) — no alias, no deprecation. Naming is `field.*` only (§6.1); an entry that configures none keeps the seven default paths (`specs.tick` … `paren.bare`).     |
| `[^^](ref "loc")` cite head                                             | `[^ref "loc"]`                                                                       | **not syntax** (D-16, §7.3): the caret is the slot tag and sits inside the bracket, so the old form's second caret becomes the reference. A `cite` head without `reference_id` is an `Error`, not a citation.                                                                                                     |
| `[task.cite]` `citation_class` / `ref_id_prefix` / `citation_id_prefix` | a `Cite` component                                                                   | **removed** (D-16, §7.3): the built-in `<sup>` markup's class and prefixes are constants; a site that wants its own spelling configures the component. The three keys are unknown now, so a config that still sets them **fails to load** (§13).                                                                  |
| `citation_number`'s sibling `instance_index`                            | `number` (§10.5)                                                                     | **removed** (D-16, §7.3): the per-occurrence ordinal is no longer a field of the citation. The citations export carries `reference_id` / `locator` / `citation_number` only; the built-in markup still needs a unique backlink anchor and derives one internally.                                                 |

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
- **OPEN-E1 (table column extras) — resolved.** Alignment-row cell extras apply
  to that column's `<th>` only; the alignment and the width are what cascade into
  the column's `<td>`s. Implemented in `crates/plugin-table/src/grid.rs` and
  frozen by golden `09-table-layers` (a `class` on a delimiter cell stays on the
  `<th>`, while `<td>` keeps only `style="text-align: …"`).
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
- **OPEN-C3 (layer routing) — resolved (`D-3a`).** A plugin no longer selects a
  layer component per instance: it emits its built-in node, keeps the `type`
  marker and tags the node with the §11 layer that owns it (`__layer`), and
  `pendon-renderer-solid` resolves every layer of the document at once
  (`resolve.rs`). The earlier "routable layer" restriction in `apps/cli` is gone
  (`apps/cli/src/plugins.rs::layer_set`).
- **OPEN-S1 (`[task.cite.section]` removal order) — resolved (done).**
  `plugin-section` is heading/icon based and never references `cite.section`, so
  the removal landed with the cite change: `CiteOptions.section`,
  `CitationContext::replace_section_markers`, the section branch of
  `cite::solid_hints` and the `[task.cite.section]` blocks in `sandbox/cite`,
  `sandbox/cite/custom`, `sandbox/universal` and `sandbox/image/regular` are gone,
  together with the `{{ footnote }}` lines in the two sources that used them.

## 16. Golden set and tests

Expectations for the grammar are frozen as **data** before implementation, so no
downstream rewrite happens. The fixtures live in `docs/spec/golden/` and are the
**executable source of truth**; see [`golden/README.md`](golden/README.md) for the
format, the harness (`apps/cli/tests/syntax_spec.rs`, one `#[test]` per fixture,
byte-for-byte against the real CLI) and the re-freeze procedure.

### 16.1 Fixture format

```text
docs/spec/golden/NN-name.mdp          input markdown
docs/spec/golden/NN-name.toml         pendon.toml for the fixture (plugins + custom sets)
docs/spec/golden/NN-name.jsx          expected solid output (generated — never hand-edited)
docs/spec/golden/NN-name.events.json  expected event IR (optional, for parser bugs)
docs/spec/golden/NN-name.data/        payload files read by a `(path)` block (optional)
docs/spec/golden/NN-name.out/         side files the render writes, a mirror of `out/` (optional, §21)
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
   attributes, with or without a custom component. The same holds for a slot head —
   every position it carries becomes a field, preset-named or slot-qualified (§6.5).
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
- A whole file may also be published at module scope, without naming it in the
  Markdown: `[[task.bind.inject]]` with `source` (project-root-relative, may use
  the task's captures) and an optional `export` name (default: the file stem).
  The binding itself — order, duplicates, "Solid only" — is §20.
- Values are parsed to JSON: `jsonc` strips `//` and `/* … */` comments outside
  strings; `yaml` expands merge keys (`<<`) and maps non-finite floats to `null`;
  `toml` datetimes become RFC 3339 strings; `csv` uses the header row as keys and
  parses each cell as JSON when possible (`NULL` → `null`).
- `mdp` (Pendon Markdown as a value) is a **sub-document**, not opaque text: its
  body is parsed and run through the **same plugin chain as the page**, so every
  enabled plugin reaches inside it — wiki links, citations, figures, tables, math,
  extras heads, custom components. It therefore shares the page's state: a
  citation written in a payload joins the page's citation list and numbering, and
  the renderer hints a payload's plugins need join the page's. The block form
  (body on the line after the head) keeps the payload's block wrappers; the
  inline form (body on the head line) drops the payload's own `Paragraph`, so the
  text there contributes inline nodes only. A payload reaches the attribute as a
  **fragment** — a node list under `renderer_ast::FRAGMENT_KEY`
  (`{"$fragment": [ … ]}`) — and each renderer owns its own format: Solid emits
  `name={(<>…</>)}`, HTML renders the markup into the attribute value, and the
  AST/JSON renderers keep the structured node list.

- **No re-binding inside a payload.** Resolution is one pass over the page's
  attributes, so an `mdp` payload that itself writes an extras head
  (`{{markerX}}{data: $foo}`) is rendered by the chain but that `$foo` stays
  literal — and, because `resolve` never sees it, it raises no warning either.
  Bind the value outside the payload and pass the payload, not the reference.

`plugin-bind` runs **before** `pendon_core::parse` (a payload's `#`, `-` and blank
lines would otherwise be markup) and resolves **after** every other plugin, before
the citation export — which is why a bound payload's citations land in the page's
list. It is Solid-first: a bound value reaches the prop as `name={…}`, `html`
renders a structured value as attribute text and a fragment as escaped markup, and
the `ast`/`json` outputs keep the re-hydrated value. Executable contracts: goldens
[`23-bind`](golden/README.md) (blocks, `$var`, nested values),
[`24-bind-paths`](golden/README.md) (paths, spread/merge, a file payload resolved
through `../data/`) and [`27-bind-mdp`](golden/README.md) (`mdp` as a
sub-document: the payload runs the page's chain and reaches the prop as a
fragment).

## 20. Module-level data exports (`export const`)

A rendered **`solid`** page is a JavaScript module, and some of its data lives at
module scope as `export const <name> = <json>;`. Emission is owned by
`crates/renderer-solid`; producers only _register_ a name and a value
([ADR-0009](../decisions/0009-generic-data-exports.md)).

```text
data_export = "export const " name " = " json ";"
name        = ( ALPHA | "_" | "$" ) { ALPHA | DIGIT | "_" | "$" }
```

1. **Registrations, in order.** The renderer registers `frontmatter` then
   `headings` from the AST, automatically; producers may register more. Every
   registration is emitted in insertion order, immediately before
   `export default function`.
2. **Last wins.** Two registrations of one name emit **one** line: the value
   _and_ the position of the last registration. There is no merge and no warning
   — a later `frontmatter` registration simply replaces the automatic one.
3. **`solid` only.** `json`, `events`, `ast` and `html` have no module scope, so
   they drop the list silently instead of leaking a key nothing reads. A `mdp`
   payload registers nothing of its own (§19).
4. **`name` is a JavaScript identifier**: ASCII, `[A-Za-z_$][A-Za-z0-9_$]*`, and
   not a reserved word (the ES reserved set, plus `await`, the strict-mode
   keywords and `eval`/`arguments`; `of` is contextual and allowed). A bad name
   is a **task error** before rendering, naming the key and the value:
   `cite.citations_export 'data-x' is not a valid JavaScript identifier`.
5. **The value is compact JSON** of the same normalized shape every payload
   produces (§19), so a consumer may `import` it and use it as data.

Current producers:

| Name                                    | Producer                     | Notes                                                                                                  |
| --------------------------------------- | ---------------------------- | ------------------------------------------------------------------------------------------------------ |
| `frontmatter`                           | `renderer-solid` (automatic) | only when the document has frontmatter                                                                 |
| `headings`                              | `renderer-solid` (automatic) | built from the AST (§21); only when it has headings; nested shape                                      |
| `references`, `citations`               | `plugin-cite` (§7.3)         | names configurable; references first; a `citations` row is `{reference_id, locator?, citation_number}` |
| `[[task.bind.inject]]`'s `export`       | `plugin-bind` (§19)          | whole file payloads; Solid tasks only                                                                  |
| `[[task.<element>.extract]]`'s `export` | `plugin-extract` (§21)       | one flat row per occurrence; Solid tasks only                                                          |

The automatic `headings` outline used to be injected into the event stream by
`plugin-extract-heading`; the renderer now builds it from the `Heading` nodes
themselves (T4.5, [ADR-0011](../decisions/0011-plugin-extract.md)), so no plugin
has to run for it and the data is unchanged. A _flat_ outline is an ordinary
extract block (§21).

## 21. Data extraction (`[[task.<element>.extract]]`)

An **extract** block reads one element's occurrences and writes their fields to a
file (`output` + `format`) and/or to a module-level binding (`export`, §20). It is
a **reader**: adding one never changes a rendered byte (P.2 of
[`../issues/plugin-extract.md`](../issues/plugin-extract.md)).

```text
[[task.<element>.extract]]
type        = <selector> | [<selector>, …]   # default: `_`
output      = <path>     | [<path>, …]
format      = "json" | "yaml" | "toml"       # required, one per `output`
export      = <name>                         # one binding per block
preset      = "pendon" | "all"               # default: `pendon`
include     = [<field>, …]                   # adds to the preset; never with `only`
only        = [<field>, …]                   # keeps nothing else
omit        = [<field>, …]                   # always wins
```

1. **The plugin is `extract`.** The blocks are _validated_ for every task (a typo
   fails before rendering), but the pass only runs when `extract` is in
   `task.plugin`, and it must **follow** every plugin whose attributes it reads —
   after the element plugins and after `number`. A misordered chain is a task
   error naming both plugins.
2. **`<element>` is one of** `anchor`, `blockquote`, `cite`, `directive`,
   `heading`, `image`, `list`, `marker`, `section`, `table`. Any other name is a
   task error naming it. The extractable **fields** per element are frozen in
   [`FIELDS-INVENTORY.md`](FIELDS-INVENTORY.md).
3. **`type` is a selector list** (§11, [ADR-0008](../decisions/0008-type-selectors.md)):
   a literal, `?`, `_` or `*`, resolved per element against the types that element
   actually occurs with, so two blocks naming disjoint types cannot both claim a
   node, and an omitted `type` is `_` — the remainder no other block named.
4. **Field selection** starts from a **preset** — `"pendon"` is the element's own
   field list (§12 of [`FIELDS-INVENTORY.md`](FIELDS-INVENTORY.md)), `"all"` is
   every field the occurrence carries — then `include` adds to it, `only` keeps
   nothing but its own entries and **`omit` subtracts last, so omit always
   wins** (P.3, [D-6](../workflows/unified-syntax-stage-2/DECISIONS.md)). A
   preset name outside those two is a task error listing both; so is `only`
   together with `include`, which would be one command with two readings.
   `include` with `omit` is fine (the `omit` names simply never ship), and
   `include` on `"all"` is redundant but valid. `recommended = <bool>` is **gone**
   (D-16): it was `preset = "pendon"` and is now a load error naming the
   replacement.
   Every entry of the three lists is a **dotted field path**, validated at load
   under §7's name rules (`price..amount` is an error naming the entry, D-16), and
   a path whose **prefix** matches selects the whole **subtree**. So
   `include = ["price.details"]` ships `price.details` and everything below it but
   no sibling of `amount`, while `omit = ["price.details.tax"]` cuts one leaf out
   of an otherwise included subtree. Paths are segment-wise — `price` never
   matches `priced` — and an array is a leaf: paths address fields, never indices.
   `only` is a **filter, not an addition**: it can only keep what the preset or
   `include` already reached, so `only` naming a path the preset does not carry
   extracts nothing at all (PE.4). A path the element does not carry is dropped,
   never emitted empty, and a path that lost every child takes its container with
   it (no `{}` husk). Internal `__`-keys never ship; an author's extras key passes
   through under its own path. The names a row carries are the **projection's**
   (ADR-0014, §6.3): `field.override.*` renames a field before it is extracted
   (D-4 — `reference_id` is published as `ref_id`), a `specs.*` position lands
   under its DOM field, an internal key never reaches a row whatever
   `preset = "all"` says, and a structured value is a real object or array —
   which is what makes a nested path addressable.
5. **Two names exist on every element:** `inner_text` (the node's text,
   descendants concatenated, trimmed) and `depth` (the heading's own, a section's
   open-section count, `1` elsewhere).
6. **One destination path = one document, and it accumulates.** Blocks listing
   several `output`s write the _same_ rows to each. Several blocks — or several
   pages — resolving to one path **append** to it instead of overwriting it
   ([D-20](../workflows/unified-syntax-stage-2/DECISIONS.md)): JSON and YAML keep
   one array, TOML keeps one array of tables **per element** (`[[heading]]` next
   to `[[cite]]`). Appending is idempotent — a row the file already holds is not
   written twice, so re-rendering a page does not duplicate its data. The file's
   format is **detected from its content** (JSON, then TOML, then YAML) and a
   block whose `format` disagrees with a file that is already there is a task
   error naming both formats; so is a file of none of the three, or of the right
   format in the wrong shape. `output` and `format` arrays must have the same
   length, and the **extension never decides the format** (`format = "json"`
   writes JSON to a `.yaml` path). `paths` are project-root-relative and take the
   task's `[segment]` captures.
7. **`export` is §20's channel**: a `solid` task emits `export const <name>`, a
   non-solid task drops the binding and keeps the files. Two blocks exporting one
   name leave the later value only (D-6).
8. **One row per occurrence, in document order.** A row's fields are **nested by
   path**: a dotted attribute name (`soo.baz`) and a structured value that is an
   object (`price` → `details` → `amount`) become objects, an array stays whole,
   and the leaves a selection did not narrow keep the shape the field map gave
   them. Nothing else is nested: a heading row carries `depth` and the fields of
   its head; a consumer builds the tree. A selection that keeps nothing is still
   a **row** — `{}` (PE.4) — because the occurrence carries fields and the filter
   matched none; only an occurrence with no field at all emits no row (OBJ-7).
9. **Written after the page renders, atomically.** A failed render leaves no side
   file behind; a block that matched nothing writes no file and registers no
   binding. Side files are **outputs**: the cache records them, so deleting one
   makes the next run rewrite it.
10. **Only a `Page` extracts.** An `mdp` payload (§19) runs every plugin for its
    in-document effects, but `extract` is skipped there, so a payload writes
    neither a file nor an export.
11. **Frontmatter is extractable too, as one record** (D-14).
    `[[task.frontmatter.extract]]` takes the same destinations — `output` +
    `format` (`json` / `yaml` / `toml`) and/or `export` (§20) — with three
    differences, because frontmatter is _the author's data_ and not an element's
    fields: it is **one row per document** (the parsed map, so a JSON file holds
    `[ { … } ]`), its **fields are the top-level keys** (a nested key is a path:
    `only = ["title", "meta.tags"]` keeps those two and drops the rest of
    `meta`, and `omit` wins as in rule 4), and it has **no preset**. `preset`
    (even `"all"`), `include` and `type` are config errors there — there is
    nothing to start from and nothing to select by — so `only` / `omit` are the
    whole selection. A document with no frontmatter, or an empty one, writes no
    file and registers no binding. `export = "frontmatter"` is an error: §20
    already publishes that const from the same record. `micromatter` is what
    turns the `---` block into the node this reads, so `extract` must follow it
    (rule 1).

Executable contracts: goldens [`32-extract-heading`](golden/README.md) (a heading
block writing a file _and_ `export const outline`, next to the automatic nested
`headings`), [`33-extract-scopes`](golden/README.md) (two `type`-scoped blocks,
the `_` remainder, a last-wins `export`, and the `preset` / `preset` + `include`
pair) and [`44-extract-paths`](golden/README.md) (PE.1–PE.4 — one record selected
four ways with dotted paths); `apps/cli/tests/extract_spec.rs` adds the
dynamic/static path rules, the non-Solid drop, the cache rewrite, the projection's
public names and every D-6 selection error.
