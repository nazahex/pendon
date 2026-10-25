# Extras inventory — extractable fields per element

**Status: skeleton (Phase 0).** One row per in-scope element is seeded here; the
`recommended` column is filled by **T4.1** of
[`../workflows/TASKS.md`](../workflows/TASKS.md) before `plugin-extract` is
written ([ADR-0011](../decisions/0011-plugin-extract.md)). `T4.1` also expands
each seed row into one row per field.

Scope is fixed by
[`../workflows/numbering-extract-type-export.md`](../workflows/numbering-extract-type-export.md)
§8.6: `anchor`, `blockquote`, `cite`, `directive`, `heading`, `image`, `marker`,
`list`, `section`, `table`, `wiki` (wikilink only), `codeblock` (vanilla),
`latex`, `bind`, `micromatter`. Out of scope and deliberately absent: `quiz`,
`codeblock-syntect`, `vicado`, `dialog`, and `plugin-custom`'s `toml:` specs.
`quiz`, `vicado` and the wiki infobox are not merely unscoped — they are
**deleted entirely in the next workflow**, so no Phase-4 row will ever be filled
for them and no code should be written against them.

## How to fill a row

1. **`field`** — the attribute name as it appears on the node, i.e. the `only`
   / `omit` name a config block would use (`slug`, `loc`, `rel`, `number`, …).
   It must already have a defined "only" name; a value with no stable name does
   not ship.
2. **`source`** — where the field is defined, in this order: the element's §N in
   [`../spec/SYNTAX.md`](../spec/SYNTAX.md) (the attribute list), the plugin's
   `emit` block (`crates/plugin-<element>/src/lib.rs`), then the golden `.jsx`
   that shows it (`docs/spec/golden/NN-*.jsx`).
3. **`recommended`** — `yes` (in the `recommended` preset), `no` (only on
   request), or leave `TODO` while unreviewed. Nothing is recommended until it
   has been reviewed against a real fixture.
4. **`notes`** — interactions worth pinning: a reserved key (e.g. `number` once
   `plugin-number` runs, D-10), a field only present in one form (figure vs
   image), or a value that is structured (`U+E001`).

A row whose `source` cannot name a §N, an `emit` site **and** a golden stays
`TODO` — that is the signal that the field is not yet extractable.

## anchor

| field        | source                                                                                       | recommended | notes                                                        |
| ------------ | -------------------------------------------------------------------------------------------- | ----------- | ------------------------------------------------------------ |
| _all extras_ | [`SYNTAX.md` §7.2](../spec/SYNTAX.md); `crates/plugin-anchor/src/lib.rs`; golden `04-anchor` | TODO        | seed row — T4.1 splits it per field (`href`, `rel`, `id`, …) |

## blockquote

| field        | source                                                                                               | recommended | notes                                             |
| ------------ | ---------------------------------------------------------------------------------------------------- | ----------- | ------------------------------------------------- |
| _all extras_ | [`SYNTAX.md` §9.2](../spec/SYNTAX.md); `crates/plugin-blockquote/src/lib.rs`; golden `11-blockquote` | TODO        | seed row; `type` mirrors the extras marker (§6.2) |

## cite

| field        | source                                                                                   | recommended | notes                                                                  |
| ------------ | ---------------------------------------------------------------------------------------- | ----------- | ---------------------------------------------------------------------- |
| _all extras_ | [`SYNTAX.md` §7.3](../spec/SYNTAX.md); `crates/plugin-cite/src/lib.rs`; golden `05-cite` | TODO        | seed row; `number` arrives from `plugin-cite` today — Phase 3 keeps it |
| `ref`        | [`SYNTAX.md` §7.3](../spec/SYNTAX.md); golden `26-cite-scope`                            | TODO        | reference id, the join key into `export const references`              |

## directive

| field        | source                                                                                          | recommended | notes                                             |
| ------------ | ----------------------------------------------------------------------------------------------- | ----------- | ------------------------------------------------- |
| _all extras_ | [`SYNTAX.md` §10.2](../spec/SYNTAX.md); `crates/plugin-directive/src/lib.rs`; goldens `15`/`16` | TODO        | seed row; inline and block forms share the fields |

## heading

| field        | source                                                                                         | recommended | notes                                                             |
| ------------ | ---------------------------------------------------------------------------------------------- | ----------- | ----------------------------------------------------------------- |
| _all extras_ | [`SYNTAX.md` §7.4](../spec/SYNTAX.md); `crates/plugin-heading/src/lib.rs`; golden `06-heading` | TODO        | seed row; `level`, `slug`, `title`, `number` are the known ones   |
| `number`     | `crates/plugin-number/src/lib.rs` (Phase 3); golden `29-number`                                | TODO        | reserved once `plugin-number` runs (D-10); `__number` on the wire |

## image

| field        | source                                                                                            | recommended | notes                                                                  |
| ------------ | ------------------------------------------------------------------------------------------------- | ----------- | ---------------------------------------------------------------------- |
| _all extras_ | [`SYNTAX.md` §7.1](../spec/SYNTAX.md); `crates/plugin-image/src/lib.rs`; golden `03-image-figure` | TODO        | seed row; figure-only fields (`alt`, caption) differ from bare `<img>` |

## list

| field        | source                                                                                             | recommended | notes                                                       |
| ------------ | -------------------------------------------------------------------------------------------------- | ----------- | ----------------------------------------------------------- |
| _all extras_ | [`SYNTAX.md` §9.4](../spec/SYNTAX.md); `crates/plugin-list/src/lib.rs`; golden `12-list-container` | TODO        | seed row; container vs item layers (`unordered`, `ordered`) |

## marker

| field        | source                                                                                        | recommended | notes                                    |
| ------------ | --------------------------------------------------------------------------------------------- | ----------- | ---------------------------------------- |
| _all extras_ | [`SYNTAX.md` §10.1](../spec/SYNTAX.md); `crates/plugin-marker/src/lib.rs`; golden `14-marker` | TODO        | seed row; `{{type}}` marker is mandatory |

## section

| field        | source                                                                                         | recommended | notes                                         |
| ------------ | ---------------------------------------------------------------------------------------------- | ----------- | --------------------------------------------- |
| _all extras_ | [`SYNTAX.md` §9.5](../spec/SYNTAX.md); `crates/plugin-section/src/lib.rs`; golden `22-section` | TODO        | seed row; decorator + id chain (§9.5 outline) |

## table

| field        | source                                                                                            | recommended | notes                                                             |
| ------------ | ------------------------------------------------------------------------------------------------- | ----------- | ----------------------------------------------------------------- |
| _all extras_ | [`SYNTAX.md` §8](../spec/SYNTAX.md); `crates/plugin-table/src/{lib,custom}.rs`; goldens `08`/`09` | TODO        | seed row; per-layer fields (`table`, `thead`, `caption`, `tfoot`) |

## wiki

| field        | source                                                                                   | recommended | notes                                                                                    |
| ------------ | ---------------------------------------------------------------------------------------- | ----------- | ---------------------------------------------------------------------------------------- |
| _all extras_ | [`SYNTAX.md` §7.5](../spec/SYNTAX.md); `crates/plugin-wiki/src/lib.rs`; golden `07-wiki` | TODO        | wikilink only — infobox internals stay out of scope (§8.6) and are deleted next workflow |

## codeblock (vanilla)

| field        | source                                                                      | recommended | notes                                                    |
| ------------ | --------------------------------------------------------------------------- | ----------- | -------------------------------------------------------- |
| _all extras_ | [`SYNTAX.md` §9.1](../spec/SYNTAX.md); `crates/plugin-codeblock/src/lib.rs` | TODO        | no golden yet — T4.1 either freezes one or drops the row |

## latex

| field        | source                                                                   | recommended | notes                     |
| ------------ | ------------------------------------------------------------------------ | ----------- | ------------------------- |
| _all extras_ | [`SYNTAX.md` §10.3](../spec/SYNTAX.md); `crates/plugin-latex/src/lib.rs` | TODO        | no golden yet — see above |

## bind

| field        | source                                                                                  | recommended | notes                                                               |
| ------------ | --------------------------------------------------------------------------------------- | ----------- | ------------------------------------------------------------------- |
| _all extras_ | [`SYNTAX.md` §19](../spec/SYNTAX.md); `crates/plugin-bind/src/lib.rs`; golden `23-bind` | TODO        | bound values are structured (`U+E001`); extraction emits them as-is |

## micromatter

| field        | source                                                                       | recommended | notes                                                                |
| ------------ | ---------------------------------------------------------------------------- | ----------- | -------------------------------------------------------------------- |
| _all extras_ | [`SYNTAX.md` §12](../spec/SYNTAX.md); `crates/plugin-micromatter/src/lib.rs` | TODO        | frontmatter-derived; `headings`/`references` exports already read it |
