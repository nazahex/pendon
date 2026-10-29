# Fields inventory — built-in fields per element

**Status: frozen (T4.1).** This is the contract `plugin-extract` implements: the
`only` / `omit` / `preset` vocabulary, one row per field, `preset`
decided per element (D-7 of
[`../workflows/numbering-extract-type-export.md`](../workflows/numbering-extract-type-export.md)).
Changing a `preset` mark here changes what `preset = "pendon"` extracts, so
it is a behaviour change and needs its own line in `STATUS.md`.

Scope is fixed by §8.6 of the workflow: `anchor`, `blockquote`, `cite`,
`directive`, `heading`, `image`, `marker`, `list`, `section`, `table`, `wiki`
(wikilink only), `codeblock` (vanilla), `latex`, `bind`, `micromatter`. §1 lists
the five in-scope elements that are **not** extractable yet and why; §2–§11 are
the frozen tables. Out of scope and deliberately absent: `quiz`,
`codeblock-syntect`, `vicado`, `dialog`, and `plugin-custom`'s `toml:` specs.

## 0. How to read a row

1. **`field`** — the name a config block uses (`only` / `omit`), which is also the
   key in the emitted object. Everything the element plugin writes as an
   `Event::Attribute` is addressable by its attribute name.
2. **`type`** — `string`, `number`, `string[]`, `text` (`inner_text`), or `flag`.
   A value carried as `JSON_ATTR_PREFIX` + JSON (`number`, a bound `data`) is
   emitted **structured**, not as a string.
3. **`source`** — `§N · crate · golden`, in that order: the attribute list in
   [`SYNTAX.md`](../spec/SYNTAX.md), the plugin's `emit` block, then the fixture
   that proves it. A field with no golden does not ship.
4. **`preset`** — `yes` = in the `pendon` preset, `no` = only on request.
5. **`notes`** — interactions worth pinning.

Two names exist on **every** extractable element and are not repeated per table:

| field        | type   | meaning                                                              |
| ------------ | ------ | -------------------------------------------------------------------- |
| `inner_text` | text   | the node's text content, descendants concatenated, trimmed           |
| `depth`      | number | outline nesting: the heading's own, a section's open count, else `1` |

Rules that hold for all rows:

- **Internal keys never ship.** `__layer`, `__plugin_kind`, `__number` and every
  other `__`-prefixed attribute are the event stream's own bookkeeping (D-9);
  `only = ["__layer"]` selects nothing rather than leaking it.
- **Extras pass through under their own name.** An element plugin emits the
  author's extras verbatim (`for (name, value) in attrs.values`), so
  `only = ["extraAttr1"]` works for any author-defined key. The tables list the
  fields the _plugin_ writes; a custom key is not less real, it is just not
  enumerable here.
- **A missing field is dropped, never emitted empty** — even when `only` names
  it (P.1 of [`../issues/plugin-extract.md`](../issues/plugin-extract.md)).
- **Selection order** is `preset` → `only` → `omit`, and `omit` always wins
  (P.3). Fields are emitted in **attribute order** (the order the plugin wrote
  them), not in the order of the `only` array.

## 1. In scope, not extractable yet

These five are listed by §8.6 but have no row, because a field needs an `emit`
site _and_ a golden (§0's source rule) and they have neither. They are not
silently dropped: `[[task.<element>.extract]]` for one of them is a config error
naming the element.

| element       | why not                                                                                                                      | follow-up                                           |
| ------------- | ---------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------- |
| `wiki`        | `plugin-wiki` rewrites the text into a plain `Link` with **no `__layer`** tag, so a wikilink and an anchor are the same node | tag it `__layer = "wiki"` (mirrors `plugin-anchor`) |
| `codeblock`   | the vanilla form is `markdown`'s `CodeFence` node; there is no `plugin-codeblock` crate to emit a layer for it               | decide the layer name, then add a golden            |
| `latex`       | `plugin-latex` emits `HtmlBlock` / `HtmlInline`, i.e. opaque HTML with **no attributes of its own**                          | none — a math node carries nothing to extract       |
| `bind`        | `plugin-bind` writes its values onto the **host** element (`<a build=… data=…>`), not onto a node of its own                 | extract the host element (`anchor`) instead         |
| `micromatter` | `frontmatter` is not an extras-capable element; its data is already the automatic `frontmatter` export (ADR-0009)            | none — `plugin-extract` would duplicate it          |

## 2. `anchor` — `__layer = "anchor"` (`NodeKind::Link`)

Source: §7.2 · `crates/plugin-anchor` · golden `04-anchor`.

| field        | type   | recommended | notes                                                     |
| ------------ | ------ | ----------- | --------------------------------------------------------- |
| `href`       | string | yes         | the construct owns it; an extras `href` is ignored (§7.2) |
| `id`         | string | yes         | `#id` > `[slug]` head > extras `slug` (§6.2)              |
| `title`      | string | yes         | from the head's `("title")`, never part of the label      |
| `type`       | string | yes         | the `@@type` marker                                       |
| `rel`        | string | no          | merged, not replaced, by the `rel:` modifier              |
| `target`     | string | no          | `_blank` from a `!` / `?` modifier, or extras             |
| `inner_text` | text   | yes         | the link label                                            |

## 3. `blockquote` — `__layer = "blockquote"`

Source: §9.2 · `crates/plugin-blockquote` · golden `11-blockquote`.

| field  | type   | recommended | notes                         |
| ------ | ------ | ----------- | ----------------------------- |
| `type` | string | yes         | the decorator's marker (§6.2) |

The quote writes nothing else: `id` / `class` / `title` reach it as extras under
their own names (§0) and `inner_text` is universal, so §12's preset can name them
without a row here.

## 4. `cite` — `__layer = "cite"` (renders as the `Cite` component)

Source: §7.3 · `crates/plugin-cite` · golden `05-cite`.

`reference_id` and `locator` are the §7.3 preset (S2-4.4); `citation_number` is
assigned by the processor and is the one field the element **owns** (D-12 rule 5).
`instance_index` is gone (§14) — a per-occurrence ordinal is `plugin-number`'s.

| field             | type   | recommended | notes                                                            |
| ----------------- | ------ | ----------- | ---------------------------------------------------------------- |
| `reference_id`    | string | yes         | the reference the head names; mandatory (§7.3)                   |
| `locator`         | string | yes         | the quoted locator; part of the citation's identity              |
| `citation_number` | number | yes         | assigned by `plugin-cite`; stable for a repeated `(id, locator)` |
| `cite-id`         | string | yes         | the node's `id` slot (`#id` / slug) — the backlink anchor        |
| `type`            | string | yes         | the `@@type` marker                                              |
| `note`            | string | no          | extras only                                                      |
| `class`           | string | no          | extras only                                                      |
| `inner_text`      | text   | no          | the citation's display text (the `[n]` marker)                   |

`reference` is **not** a field: the renderer resolves it from `reference_id`
against `export const references`. Extract `cite-id` and join on the `references`
export — which is what golden `05`'s `citations` array does.

## 5. `directive` — `__layer = "directive"`

Source: §10.2/§10.4 · `crates/plugin-directive` · goldens `15`/`16`.

| field  | type   | recommended | notes                                                 |
| ------ | ------ | ----------- | ----------------------------------------------------- |
| `type` | string | yes         | `::name` — the routing key and the marker's only name |

## 6. `heading` — `__layer = "heading"` (`NodeKind::Heading`)

Source: §7.4/§11 · `crates/plugin-heading` · golden `06-heading`.

| field        | type     | recommended | notes                                                     |
| ------------ | -------- | ----------- | --------------------------------------------------------- |
| `depth`      | number   | yes         | `1..6`: the `#` run, minus the `#` the head's tag spends  |
| `id`         | string   | yes         | section id > `#id` > `[slug]` > extras slug (§6.2)        |
| `slug`       | string   | yes         | the heading's own slug (`[slug]` head, extras tick, auto) |
| `title`      | string   | yes         | the head's `("title")` (or the extras `"…"` fallback)     |
| `number`     | string[] | yes         | written by `plugin-number` (ADR-0010); structured array   |
| `type`       | string   | yes         | the `@@type` marker                                       |
| `class`      | string   | no          | extras only                                               |
| `inner_text` | text     | yes         | the visible heading text, never an attribute              |

`number` is **reserved** once `plugin-number` runs (D-10): an extras
`number: "auto"` is a diagnostic there, and the only `number` an extract sees is
the array.

## 7. `image` — `__layer = "img"`

Source: §7.1 · `crates/plugin-image` · golden `03-image`.

An image is **one node**: `![alt](src)` is the whole head, the extras decorate the
same `<img>`, and every position the head carries beyond the preset stays a bare
slot field. There is no `figure` layer and no caption any more (§14).

| field           | type   | recommended | notes                                              |
| --------------- | ------ | ----------- | -------------------------------------------------- |
| `src`           | string | yes         | `paren.bare`; owned by the construct (D-12 rule 5) |
| `alt`           | string | yes         | `bracket.bare`; owned by the construct (D-12)      |
| `width`         | number | yes         | an extras `width:` (the `w` modifier is gone)      |
| `height`        | number | yes         | an extras `height:` (the `h` modifier is gone)     |
| `type`          | string | yes         | the `@@type` marker                                |
| `id`            | string | yes         | `#id` / extras `` `slug` `` (§6.2)                 |
| `class`         | string | no          | extras only, accumulates (§6.4)                    |
| `style`         | string | no          | extras only; `--x:` folds in (§6.3)                |
| `loading`       | string | no          | an extras `loading:` (the `?` modifier is gone)    |
| `decoding`      | string | no          | an extras `decoding:` (the `~` modifier is gone)   |
| `bracket.quote` | text   | no          | a claimed position the preset does not name (D-7)  |
| `inner_text`    | —      | no          | **none**: the head owns no children (§14)          |

## 8. `list` — `__layer = "list"` (item), `"unordered"` / `"ordered"` (container)

Source: §9.3/§9.4 · `crates/plugin-list` · golden `12-list-container`.

| field        | type   | recommended | layer     | notes                                       |
| ------------ | ------ | ----------- | --------- | ------------------------------------------- |
| `type`       | string | yes         | container | the marker; the layer decides, not the type |
| `id`         | string | yes         | container | `#id` / slug of the decorator               |
| `start`      | number | yes         | `ordered` | the `<ol start>`; absent on `<ul>`          |
| `class`      | string | no          | both      | extras only                                 |
| `inner_text` | text   | yes         | item      | the item's text                             |

## 9. `marker` — `__layer = "marker"`

Source: §10.1 · `crates/plugin-marker` · golden `14-marker`.

| field        | type   | recommended | notes                                              |
| ------------ | ------ | ----------- | -------------------------------------------------- |
| `type`       | string | yes         | **mandatory** (§10.1) and the §11 routing key      |
| `slug`       | string | yes         | `{{type}}` groups between `}}` and the extras head |
| `id`         | string | yes         | `#id` / extras `id`                                |
| `title`      | string | yes         | extras `title:`                                    |
| `isOpen`     | flag   | no          | bare extras flag                                   |
| `class`      | string | no          | extras only                                        |
| `inner_text` | text   | yes         | the block form's trailing text (the caption)       |

## 10. `section` — `__layer = "section"` (`NodeKind::Section`)

Source: §9.5 · `crates/plugin-section` · golden `22-section`.

A section has **no head**: it declares no `ElementHead` and contracts no slot.
Its own two fields are the ones it derives — `id` from the chain and `depth` from
the heading it wraps (or its parent `<--->` section + 1, `0` for the preface) —
and the decorator line above the heading publishes everything else.

| field        | type   | recommended | notes                                                              |
| ------------ | ------ | ----------- | ------------------------------------------------------------------ |
| `id`         | string | yes         | `#sectionID` > `[slug]` > extras slug > title slug                 |
| `type`       | string | yes         | the decorator's marker (§6.2)                                      |
| `class`      | string | yes         | extras only                                                        |
| `title`      | string | no          | extras `title:`                                                    |
| `inner_text` | text   | no          | the whole section body; prefer the child headings                  |
| `depth`      | number | yes         | the heading's own depth; a `<--->` child's parent + 1; `0` prehead |

## 11. `table` — `__layer = "table"` (+ `thead` / `tbody` / `tfoot` layers)

Source: §8 · `crates/plugin-table` · goldens `08`/`09`.

| field        | type   | recommended | layer   | notes                                          |
| ------------ | ------ | ----------- | ------- | ---------------------------------------------- |
| `type`       | string | yes         | `table` | the `@@type` marker                            |
| `id`         | string | yes         | `table` | `#id` / slug (§6.2)                            |
| `class`      | string | no          | `table` | extras only                                    |
| `caption`    | string | yes         | `table` | the construct's caption slot                   |
| `inner_text` | text   | no          | `table` | flattened cell text — prefer the per-cell rows |

Per-cell fields (`colspan`, `rowspan`, `align`, `id`, `class`) live on `td`/`th`
nodes, which are **markdown's** nodes, not a Pendon layer: they are out of the
extract surface until a table layer owns them.

## 12. Preset table (the code's source of truth)

`recommended = true` extracts exactly these names, in this order, intersect
`only`, minus `omit`:

| element      | preset                                                          |
| ------------ | --------------------------------------------------------------- |
| `anchor`     | `href`, `id`, `title`, `type`, `inner_text`                     |
| `blockquote` | `type`, `id`, `class`, `inner_text`                             |
| `cite`       | `reference_id`, `locator`, `citation_number`, `cite-id`, `type` |
| `directive`  | `type`, `id`, `level`, `summary`, `inner_text`                  |
| `heading`    | `depth`, `id`, `slug`, `title`, `number`, `type`, `inner_text`  |
| `image`      | `src`, `alt`, `width`, `height`, `type`, `id`                   |
| `list`       | `type`, `id`, `start`, `inner_text`                             |
| `marker`     | `type`, `slug`, `id`, `title`, `inner_text`                     |
| `section`    | `id`, `type`, `class`, `depth`                                  |
| `table`      | `type`, `id`, `caption`                                         |

A field in the preset that an element does not carry in a given instance is
dropped from that row (§0), so one preset serves a whole document.
