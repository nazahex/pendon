# ADR-0011 — `plugin-extract`: field-level extraction per element

- Status: **Proposed** (Phase 4 of
  [`workflows/numbering-extract-type-export.md`](../workflows/numbering-extract-type-export.md))
- Depends on: [`0008`](0008-type-selectors.md) (type scoping),
  [`0009`](0009-generic-data-exports.md) (exports), [`0010`](0010-plugin-number.md)
  (`attrs.number` as an extractable field)
- Spec: [`SYNTAX.md`](../spec/SYNTAX.md) §11 contract, §13 (
  diagnostics), §15 (open decision D-7)
- Evidence: `spec/EXTRAS-INVENTORY.md`, goldens `32-extract-heading`,
  `33-extract-scopes`

## Context

`plugin-extract-heading` is the only extractor, and it is not an extractor: it
has no config, no file output and no export — it injects a `Custom("Headings")`
node whose `attrs.data` is a _nested_ outline. The `headings` export comes from
`renderer-solid`'s own metadata walk. Meanwhile every element carries
extras-authored fields (`heading.slug`, `cite.loc`, `image.alt`, `anchor.rel`,
…) that a consumer must currently re-parse from the rendered markup.

Three separate problems follow: extraction is element-specific instead of
field-specific, extraction cannot write a file, and a `payload` document can
reach document-level outputs at all (`chain.rs` hard-codes `extract-heading`).

## Decision

1. **Extraction is per element, configured per element**:
   `[[task.<element>.extract]]`, repeatable, order-sensitive, keys `type`,
   `output`, `format`, `only`, `omit`, `recommended`, `export`.
2. **Type scoping reuses ADR-0008**, resolved with `matching`, so `["CiteA",
   "CiteB"]` and an omitted `type` (which defaults to `_`) are two disjoint
   blocks over the same element kind.
3. **Field selection order is fixed**: all fields → `recommended` restricts →
   `only` intersects → `omit` subtracts. **`omit` always wins.** A field the
   element does not carry is dropped, never emitted empty.
4. **A field ships only when `EXTRAS-INVENTORY.md` names it** (its "only" name
   and its recommended preset mark are decided there, `D-7`), so the plugin
   never invents a key.
5. **Output is flat.** One destination path = one array; several blocks to the
   same path overwrite (last wins); `output`/`format` arrays must have equal
   length. Depth is a `level`-like field, not nesting.
6. **`export` registers through ADR-0009's list** and is dropped when the task
   format is not `solid`.
7. **Only a `Page` document writes document-level outputs.** A `Payload`/`mdp`
   sub-document runs every plugin for its in-document effects and writes no file
   and no export — replacing the hard-coded `extract-heading` skip.
8. **`plugin-extract-heading` is deleted.** Its outline becomes an ordinary
   `[[task.heading.extract]]` returning a flat array; the automatic
   `export const headings` keeps its nested shape so goldens stay stable.
9. **Side files are cache-tracked**: `CacheEntry.side_outputs` records
   `{ path, hash }` and freshness requires the file to still exist, so deleting
   one re-renders the page.

## Rejected alternatives

- **Grow `plugin-extract-heading`.** Element-specific by construction; a second
  element means a second crate.
- **One global `[task.extract]` section.** Selectors scope by `type`, not by
  element kind, so a heading and an image rule would collide.
- **Let the extension pick the format.** `out/x.json` would silently be JSON
  even when the task asked for YAML; `format` is explicit and required.
- **Nested output by default.** A consumer that wants a tree can build one;
  nesting makes "one path = one array" impossible.
- **Write files before the render succeeds.** A failed render would leave a
  half-truth on disk; writes happen after a successful render, atomically.

## Consequences

- `plugin-extract-heading` leaves the workspace, the CLI's `chain.rs` arms, the
  four sandbox configs and `apps/cli/Cargo.toml`.
- `CacheEntry` gains `side_outputs` and `CACHE_VERSION` bumps; the private
  tmp+rename in `cache.rs` is lifted to a shared `write_atomically`.
- `EXTRAS-INVENTORY.md` is the gate: Phase 4 cannot ship a field the inventory
  has not reviewed.
