# Fields inventory — built-in fields per element

**Status: the contract `plugin-extract` implements.** One row per **built-in**
field — a field an element produces itself (§0). The vocabulary is the archived
proposal's: `preset` is decided per element (D-7 in §4 of
[`../archive/workflows/numbering-and-extract/numbering-extract-type-export.md`](../archive/workflows/numbering-and-extract/numbering-extract-type-export.md))
and the selection order is P.1–P.3 of
[`../archive/fulfilled/plugin-extract.md`](../archive/fulfilled/plugin-extract.md).
Changing a `preset` mark here changes what the preset extracts, so it is a
behaviour change and needs its own line in `STATUS.md`.

The tables are **machine-checked**: §12 is `PRESETS` in
`crates/extract/src/fields.rs`, the four slot contracts are each plugin's
`HEAD`, and `apps/cli/tests/fields_inventory_spec.rs` parses this file and fails
when a row and the code disagree — a rename, an added row, a dropped one, or a
changed order. Editing a table is therefore a two-file change, not a doc fix.

Scope is fixed by §8.6 of the workflow: `anchor`, `blockquote`, `cite`,
`directive`, `heading`, `image`, `marker`, `list`, `section`, `table`, `wiki`
(wikilink only), `codeblock` (vanilla), `latex`, `bind`, `micromatter`. §1 lists
the five in-scope elements that are **not** extractable yet and why; §2–§11 are
the tables. Out of scope and deliberately absent: `quiz`, `codeblock-syntect`,
`vicado`, `dialog`, and `plugin-custom`'s `toml:` specs.

## 0. How to read a row

1. **`field`** — the name a config block uses (`only` / `omit`), which is also the
   key in the emitted object.
2. **`type`** — `string`, `number`, `string[]`, `text` (`inner_text`), or `flag`.
   An attribute carried as `JSON_ATTR_PREFIX` + JSON (`number` once
   `plugin-number` ran) is emitted **structured**; every other attribute keeps the
   string it is in the stream, so a numeric-looking `start` or `citation_number`
   is a `string` here.
3. **`source`** — `§N · crate · golden`, in that order: the attribute list in
   [`SYNTAX.md`](SYNTAX.md), the plugin's `emit` block, then the fixture that
   proves it. A field with no golden does not ship.
4. **`preset`** — `yes` = a `preset = "pendon"` extract includes it (§12),
   `no` = only on request.
5. **`notes`** — interactions worth pinning.

### A row is a built-in field

A **built-in field** is one the element itself produces: an **owned** name it
writes (`href`, `src`, `alt`, `citation_number`), a **preset assignment** it
decides (`title`, `slug`, `locator`, `reference_id`), or a value it **derives**
(`id`'s chain, `cite-id`, `number`). Four kinds of name are therefore not rows,
even though `plugin-extract` emits them:

- **Bare fields.** An extras item or a slot position the element does not assign.
  It keeps the path the stream carries: the qualified one (`specs.class`,
  `specs.tick`, `bracket.bare`, `paren.bare`) for an element that only binds a
  block's decorator (`blockquote`, `list`, `marker`, `directive`), the plain one
  (`class`, `id`, `rel`, `note`) for an element whose head projects extras onto
  the DOM (`anchor`, `image`, `heading`, `table`, `section`, `cite`) — or, where
  the built-in element has no prop for the key, the `data-`-prefixed spelling
  (`data-width`, §6.3). The set is open — any author key is addressable (§6.1).
- **`type`** — the author's routing key (§11), not a name an element defines. The
  two exceptions are the constructs that spell it in their own syntax:
  `directive` (`::name`, `==name`) and `marker` (`{{name}}`), which keep a row.
- **`name`** — the built-in tag a layer node falls back to (`div`, `span`, `img`,
  `table`, `blockquote`, `ol`, `ul`): the renderer's transport for `NodeKind`
  (D-3a), like `__layer`. Addressable, in no preset, never a row.
- **`inner_text` / `depth`** — universal (below) and computed for every
  occurrence. `inner_text` is never a row; `depth` is a row only on the two
  elements that publish it themselves (`heading`, `section`, §6/§10).

Every built-in field the tables list is in its element's preset today, so every
row reads `preset = yes`; a `no` stays legal (a built-in field the preset leaves
out) and no element needs one.

### Shared names — `inner_text`, `depth`, `specs.*`

Two fields exist on **every** extractable element. Neither is repeated per table,
except `depth` on the two elements that emit it:

| field        | type   | meaning                                                              |
| ------------ | ------ | -------------------------------------------------------------------- |
| `inner_text` | text   | the node's text content, descendants concatenated, trimmed           |
| `depth`      | number | outline nesting: the heading's own, a section's open count, else `1` |

The `specs.*` namespace is the other shared vocabulary: the five positional
contents of an extras head (§6.1), spelled the same way for every element. A
`field.specs.<k>` key moves a position, and `<k>` is the same five names:

| item      | default path  | DOM field | read as a field by                                                                                                   |
| --------- | ------------- | --------- | -------------------------------------------------------------------------------------------------------------------- |
| `` `x` `` | `specs.tick`  | —         | `slug` on `heading`; the `id` fallback on `anchor`, `image`, `table` and `section`; the `cite-id` fallback on `cite` |
| `"x"`     | `specs.quote` | —         | `title` on `anchor`, `cite`, `heading`, `image` and `table`                                                          |
| `.class`  | `specs.class` | `class`   | — (universal, §6.3)                                                                                                  |
| `#id`     | `specs.id`    | `id`      | `cite-id` on `cite`; the id chain on `section`                                                                       |
| `style:`  | `specs.style` | `style`   | — (universal, §6.3)                                                                                                  |

Only the **default path** carries meaning: once `field.specs.<k>` moves a
position, the content is the author's field under the assigned name and no
element reads it (§6.2). `class` / `id` / `style` are projected onto the DOM field
of the same name; an element that only binds a block's decorator keeps the
qualified path (`specs.class`) instead (§0).

### The slot head

A **slot head** is the `[bracket]` / `(paren)` pair that opens a line (§5.2).
Four elements claim one — `anchor`, `cite`, `heading`, `image` — and each declares
its claim as a `SlotContract` ([`crates/extra/src/slot.rs`](../../crates/extra/src/slot.rs)):
the positions it **accepts**, in head order, and the tag that claims an untagged
head. §2, §4, §6 and §7 print that contract; no other element has one, so the six
positions below are unreachable for the other six.

A claimed head is read **whole**: a content the preset does not assign is not
dropped, it keeps its slot-qualified name (D-7) — and that name _is_ the position:

| position        | content   | fallback name   |
| --------------- | --------- | --------------- |
| `bracket.bare`  | `[x]`     | `bracket.bare`  |
| `bracket.quote` | `["x"]`   | `bracket.quote` |
| `bracket.tick`  | ``[`x`]`` | `bracket.tick`  |
| `paren.bare`    | `(x)`     | `paren.bare`    |
| `paren.quote`   | `("x")`   | `paren.quote`   |
| `paren.tick`    | ``(`x`)`` | `paren.tick`    |

The fallback is the position's own spelling because a single segment cannot
contain a `.`: an unassigned content can never collide with a preset name (`href`,
`title`, `inner_text`). One `field.<slot>.<kind>` key replaces a fallback (D-5)
and, at the same time, voids the whole preset for that `custom` entry (D-7) —
there is no half-preset mode.

### The `number` namespace

`plugin-number` is the only writer of numbering attributes (ADR-0010), so one row
covers every numbered element. A `number` its own plugin wrote (`{number: "auto"}`)
is replaced by the pass, and once the pass ran the name is reserved (D-10):

| name     | type     | meaning                                                                                               |
| -------- | -------- | ----------------------------------------------------------------------------------------------------- |
| `number` | string[] | the element's formatted counters, one per level — `["1"]`, `["2","a"]`; structured, so not a `string` |

`__number` is internal (D-9): the `{"depth":1,"type":"…"}` mark `crates/extra`
writes for `%%numberType` (§5) and the numbering pass reads. It never ships, and
`number` is the whole namespace today — S2-9.2 splits it into `number.type` /
`number.raw` / `number.depth`, which is when §6's row follows.

### Rules that hold for all rows

- **Internal keys never ship.** `__layer`, `__plugin_kind`, `__number` and every
  other `__`-prefixed attribute are the event stream's own bookkeeping (D-9);
  `only = ["__layer"]` selects nothing rather than leaking it.
- **A missing field is dropped, never emitted empty** — even when `only` names
  it (P.1 of
  [`../archive/fulfilled/plugin-extract.md`](../archive/fulfilled/plugin-extract.md)).
- **Selection order** is `preset` → `only` → `omit`, and `omit` always wins
  (P.3). Fields are emitted in **attribute order** (the order the plugin wrote
  them), not in the order of the `only` array.

## 1. In scope, not extractable yet

These four are listed by §8.6 but have no row, because a field needs an `emit`
site _and_ a golden (§0's source rule) and they have neither. They are not
silently dropped: `[[task.<element>.extract]]` for one of them is a config error
naming the element.

| element     | why not                                                                                                                      | follow-up                                           |
| ----------- | ---------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------- |
| `wiki`      | `plugin-wiki` rewrites the text into a plain `Link` with **no `__layer`** tag, so a wikilink and an anchor are the same node | tag it `__layer = "wiki"` (mirrors `plugin-anchor`) |
| `codeblock` | the vanilla form is `markdown`'s `CodeFence` node; there is no `plugin-codeblock` crate to emit a layer for it               | decide the layer name, then add a golden            |
| `latex`     | `plugin-latex` emits `HtmlBlock` / `HtmlInline`, i.e. opaque HTML with **no attributes of its own**                          | none — a math node carries nothing to extract       |
| `bind`      | `plugin-bind` writes its values onto the **host** element (`<a build=… data=…>`), not onto a node of its own                 | extract the host element (`anchor`) instead         |

**`frontmatter` left this list in S2-8.5.** It is extractable (`[[task.frontmatter.extract]]`,
D-14) but it is not an element with a field vocabulary: its fields are the
**author's own keys**, a nested key is a path, and it has no preset — `only` /
`omit` are the whole selection, and `preset`, `include` and `type` are config
errors. §21 rule 11 has the rules; the automatic `export const frontmatter` (§20) is
unchanged, which is why `export = "frontmatter"` in such a block is an error.

## 2. `anchor` — `__layer = "anchor"` (`NodeKind::Link`)

Source: §7.2 · `crates/plugin-anchor` · golden `04-anchor`.

| field   | type   | preset | notes                                                                                             |
| ------- | ------ | ------ | ------------------------------------------------------------------------------------------------- |
| `href`  | string | yes    | `paren.bare`; the construct owns it, so an extras `href:` is an `Error` (D-12 rule 5)             |
| `title` | string | yes    | `paren.quote`; the head wins over an extras `"…"` (§6.2)                                          |
| `id`    | string | yes    | `#id` > extras `` `slug` `` — the tick position, read while it keeps its default path (§6.2/§6.3) |

The other three positions of the head (`bracket.quote`, `bracket.tick`,
`paren.tick`) stay bare under their slot-qualified names (D-7), and `class` /
`style` / `rel` / `target` / `hreflang` are ordinary extras (§6.3): no row (§0).

**Slot contract** — tag `@`, `HEAD` in `crates/plugin-anchor`:

| position        | lands in        |
| --------------- | --------------- |
| `bracket.bare`  | `inner_text`    |
| `bracket.quote` | `bracket.quote` |
| `bracket.tick`  | `bracket.tick`  |
| `paren.bare`    | `href`          |
| `paren.quote`   | `title`         |
| `paren.tick`    | `paren.tick`    |

`inner_text` is the one assignment to a shared name (§0) — which is why the link
text is not a row of its own — and the three bare positions are the fallbacks of
§0. The extras side feeds `id` (`specs.tick`, as a slug) and `title`
(`specs.quote`) while those positions keep their default paths (§0).

## 3. `blockquote` — `__layer = "blockquote"`

Source: §9.2 · `crates/plugin-blockquote` · golden `11-blockquote`.

**None.** A quote has no head: no slot contract, no preset, no owned field
(§9.2). The plugin writes the `__`-prefixed bookkeeping, its `name` (§0) and the
decorator line's `type` plus extras — all bare (§0), which is why a
decorator-bound quote keeps an extras class as `specs.class`, not `class`. §12's
preset for the element is a list of names a quote may carry, never fields it
defines, and `inner_text` is universal.

## 4. `cite` — `__layer = "cite"` (renders as the `Cite` component)

Source: §7.3 · `crates/plugin-cite` · golden `05-cite`.

`reference_id` and `locator` are the §7.3 preset (S2-4.4); `citation_number` is
assigned by the processor, the one field the element **owns** (D-12 rule 5).
`instance_index` is gone (§14) — a per-occurrence ordinal is `plugin-number`'s.

| field             | type   | preset | notes                                                                                                                                      |
| ----------------- | ------ | ------ | ------------------------------------------------------------------------------------------------------------------------------------------ |
| `reference_id`    | string | yes    | the reference the head names; mandatory (§7.3)                                                                                             |
| `locator`         | string | yes    | the quoted locator; part of the citation's identity                                                                                        |
| `citation_number` | string | yes    | assigned by `plugin-cite`, stable for a repeated `(reference_id, locator)`; a display ordinal, so a `string` — only `number` is structured |
| `cite-id`         | string | yes    | the node's id slot: `#id` > extras `` `slug` ``, the backlink anchor                                                                       |

`reference` is **not** a field: the renderer resolves it from `reference_id`
against `export const references`. Extract `cite-id` and join on the `references`
export — which is what golden `05`'s `citations` array does.

**Slot contract** — tag `^`, `HEAD` in `crates/plugin-cite`:

| position        | lands in       |
| --------------- | -------------- |
| `bracket.bare`  | `reference_id` |
| `bracket.quote` | `locator`      |
| `bracket.tick`  | `bracket.tick` |
| `paren.bare`    | `paren.bare`   |
| `paren.quote`   | `paren.quote`  |
| `paren.tick`    | `paren.tick`   |

The preset assigns the first two, `citation_number` is owned, and the other four
are the fallbacks of §0. Its extras side feeds `cite-id` (`specs.id`, else
`specs.tick` as a slug) and `title` (`specs.quote`), both only at their default
paths (§0).

A citation is a layer node only when a `Cite` component is configured: the
built-in `<sup class="cite-ref">` fallback is raw HTML, so a document that keeps
it has no `cite` node to extract.

## 5. `directive` — `__layer = "directive"`

Source: §10.2/§10.3 · `crates/plugin-directive` · goldens `15`/`16`.

| field  | type   | preset | notes                                                                   |
| ------ | ------ | ------ | ----------------------------------------------------------------------- |
| `type` | string | yes    | `::name` / `==name` — the head's own name, and the only field it writes |

One of §0's two exceptions: the construct spells `type` in its syntax, so it is
a field here (§10.4) and never extras-derived. A directive contracts no slot:
its `[bracket]` / `("paren")` groups stay bare (`bracket.bare`, `paren.bare`) and
so do its extras (`specs.tick`, `specs.quote`, `specs.class`), which is why §12's
`id` / `level` / `summary` are author names rather than rows.

## 6. `heading` — `__layer = "heading"` (`NodeKind::Heading`)

Source: §7.4/§11 · `crates/plugin-heading` · golden `06-heading`.

| field    | type     | preset | notes                                                                                                                                                 |
| -------- | -------- | ------ | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| `depth`  | number   | yes    | `1..6`: the `#` run minus the `#` the head's tag spends (§7.4)                                                                                        |
| `id`     | string   | yes    | the section's when `plugin-section` runs — the heading then emits neither `id` nor `slug` (§9.5); else `#id` > `[slug]` > extras tick > the auto slug |
| `slug`   | string   | yes    | `bracket.bare`, else a slug of `inner_text` (§7.4)                                                                                                    |
| `title`  | string   | yes    | `paren.quote`; the extras `"…"` is the fallback (§6.2)                                                                                                |
| `number` | string[] | yes    | written by `plugin-number` (ADR-0010) as a structured array; reserved once it runs (D-10)                                                             |

`class` is an extra and `type` the routing key (§0); `inner_text` is universal.

**Slot contract** — tag `#`, `HEAD` in `crates/plugin-heading`:

| position        | lands in        |
| --------------- | --------------- |
| `bracket.bare`  | `slug`          |
| `bracket.quote` | `bracket.quote` |
| `bracket.tick`  | `bracket.tick`  |
| `paren.bare`    | `paren.bare`    |
| `paren.quote`   | `title`         |
| `paren.tick`    | `paren.tick`    |

`depth` and `inner_text` are fields of the element but not positions of the head —
the `#` run and the Markdown body write them (§7.4) — so the preset assigns only
the two above. The extras side feeds `slug` (`specs.tick`) and `title`
(`specs.quote`), each as the fallback of its head position (§0).

`number` is **reserved** once `plugin-number` runs (D-10): an extras
`number: "auto"` is a diagnostic there, and the only `number` an extract sees is
the array.

## 7. `image` — `__layer = "img"`

Source: §7.1 · `crates/plugin-image` · golden `03-image`.

An image is **one node**: `![alt](src)` is the whole head, the extras decorate the
same `<img>`, and every position the head carries beyond the preset stays a bare
slot field. There is no `figure` layer and no caption any more (§14).

| field | type   | preset | notes                                                         |
| ----- | ------ | ------ | ------------------------------------------------------------- |
| `src` | string | yes    | `paren.bare`; owned by the construct (D-12 rule 5)            |
| `alt` | string | yes    | `bracket.bare`; owned too, and emitted even when empty (§6.3) |
| `id`  | string | yes    | `#id` > extras `` `slug` `` (§6.2)                            |

The unclaimed positions (`bracket.quote`, `bracket.tick`, `paren.quote`,
`paren.tick`) keep their slot-qualified names (D-7), and the retired modifier run
is gone (§14): `width`, `height`, `loading`, `decoding` are ordinary extras. The
built-in `<img>` has no prop for an unknown key, so such an extra reaches the
element `data-`-prefixed (`data-width`, §6.3) — a bare name, not a field of the
node.

**Slot contract** — tag `!`, `HEAD` in `crates/plugin-image`:

| position        | lands in        |
| --------------- | --------------- |
| `bracket.bare`  | `alt`           |
| `bracket.quote` | `bracket.quote` |
| `bracket.tick`  | `bracket.tick`  |
| `paren.bare`    | `src`           |
| `paren.quote`   | `paren.quote`   |
| `paren.tick`    | `paren.tick`    |

Both preset names are **owned** (D-12 rule 5) and both are optional (D-2), so
`![alt]` and `![]()` are images too. The four bare positions are the fallbacks of
§0, and the extras side feeds `id` (`specs.tick`, as a slug) and `title`
(`specs.quote`).

## 8. `list` — `__layer = "unordered"` / `"ordered"` (the container)

Source: §9.3/§9.4 · `crates/plugin-list` + `crates/plugin-markdown` · golden `12-list-container`.

| field   | type   | preset | layer     | notes                                                                                                                                    |
| ------- | ------ | ------ | --------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| `start` | string | yes    | `ordered` | the first item's number, written on the container node by `plugin-markdown`; absent on `unordered`, and extras cannot override it (§9.3) |

The items are markdown's `<li>` nodes: no layer, no field. The container contracts
nothing either — no head, no preset (§9.3) — so the decorator's `type`, `#id` and
extras are the author's (§0) and reach the node under their qualified paths
(`specs.id`, `specs.class`). `type` gets no row: it is the routing key, even when
the decorator is what writes it (§0).

## 9. `marker` — `__layer = "marker"`

Source: §10.1 · `crates/plugin-marker` · golden `14-marker`.

| field  | type   | preset | notes                                         |
| ------ | ------ | ------ | --------------------------------------------- |
| `type` | string | yes    | **mandatory** (§10.1) and the §11 routing key |

The second of §0's exceptions: `{{name}}` spells the field, and it silently wins
over an extras `@@type` (§10.1). A marker contracts nothing — no slot, no preset
— so its groups (`bracket.bare`, `paren.bare`) and extras (`specs.tick`,
`specs.quote`, `specs.class`) are bare, and its text is its children, which the
plugin never writes as an `inner_text` attribute.

## 10. `section` — `__layer = "section"` (`NodeKind::Section`)

Source: §9.5 · `crates/plugin-section` · golden `22-section`.

A section has **no head**: it declares no `ElementHead` and contracts no slot, so
the two fields it derives are its whole preset assignment (§9.5, §6.5). The
decorator line above the heading publishes everything else (`type`, `#id`,
extras — the author's, §0).

| field   | type   | preset | notes                                                                                                                                                                                      |
| ------- | ------ | ------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `id`    | string | yes    | the decorator's `#id` > its slug > the heading's `[slug]` > the heading's extras slug > the title's slug, then de-duplicated (`-2`, `-3`, …); absent on the preface and on a `<--->` child |
| `depth` | number | yes    | the outline level it stands for: the wrapped heading's own, a `<--->` child's parent + 1, `0` for the preface (§9.5)                                                                       |

## 11. `table` — `__layer = "table"`

Source: §8 · `crates/plugin-table` · goldens `08`/`09`.

**None.** The `<table>` is the one layer whose head is **outside** the table (§8):
the decorator line above it writes `type`, `#id` and the extras, so everything the
node carries is the author's (§0), and `name` is its fallback tag (§0). It does
project those extras (§6.3), so an extras `"…"` reaches it as `title` and an extras
`.published` as `class` — still the author's keys under DOM names, not fields the
element defines. The column / row / foot layers (`thead`, `tbody`, `tfoot`) and the
`<caption>` element are layers no config name maps to.

Per-cell fields (`colspan`, `rowspan`, `align`, `id`, `class`) live on `td`/`th`
nodes, which are **markdown's** nodes, not a Pendon layer: they are out of the
extract surface until a table layer owns them.

## 12. Preset table (the code's source of truth)

The `pendon` preset (`preset = "pendon"`, D-6) extracts exactly these names, in
this order; `include` adds to it, `only` keeps nothing else, `omit` subtracts
last:

| element      | preset                                                            |
| ------------ | ----------------------------------------------------------------- |
| `anchor`     | `href`, `id`, `title`, `type`\*, `inner_text`                     |
| `blockquote` | `type`\*, `id`\*, `class`\*, `inner_text`                         |
| `cite`       | `reference_id`, `locator`, `citation_number`, `cite-id`, `type`\* |
| `directive`  | `type`, `id`\*, `level`\*, `summary`\*, `inner_text`              |
| `heading`    | `depth`, `id`, `slug`, `title`, `number`, `type`\*, `inner_text`  |
| `image`      | `src`, `alt`, `width`\*, `height`\*, `type`\*, `id`               |
| `list`       | `type`\*, `id`\*, `start`, `inner_text`                           |
| `marker`     | `type`, `slug`\*, `id`\*, `title`\*, `inner_text`                 |
| `section`    | `id`, `type`\*, `class`\*, `depth`                                |
| `table`      | `type`\*, `id`\*, `caption`\*                                     |

A `*` marks a name that is not a built-in field of that element (§2–§11): it is a
bare field, the routing key, or a universal name (§0) — `inner_text` / `depth` are
universal and never starred — so the preset recommends it without any element
producing it. Some of them cannot match a name the stream carries at all today
and stay as inert recommendations: a decorator-bound element writes `specs.id` /
`specs.class`, not `id` / `class` (§3/§8/§9); the built-in `<img>` spells
`data-width` / `data-height` (§7); `table`'s `caption` is an extras key, because
the `<caption>` element is a layer of its own (§11); and `directive`'s `level` /
`summary` are extras keys its head never assigns.

A field in the preset that an instance does not carry is dropped from that row
(§0), so one preset serves a whole document.
