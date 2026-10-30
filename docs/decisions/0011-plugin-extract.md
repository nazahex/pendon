# ADR-0011 — `plugin-extract`: field-level extraction per element

- Status: **Accepted / implemented** (Phase 4 of
  [`workflows/numbering-extract-type-export.md`](../workflows/numbering-extract-type-export.md);
  outcome below)
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

## Outcome (2026-10-25)

- `pendon-plugin-extract` is wired into the CLI: the blocks are validated for
  every task (`build_extract_options`), the pass runs as a chain stage named
  `extract`, side files are written **after** a successful render through the
  shared `cache::write_atomically`, and `export` registers through
  `SolidRenderHints.exports` (ADR-0009).
- **`plugin-extract-heading` is deleted** — the crate, both `chain.rs` arms, the
  workspace/CLI manifests, its README, and the name in the four sandbox plugin
  lists. The `headings` export did **not** disappear with it: `renderer-solid`
  builds the nested outline from the `Heading` nodes themselves
  (`metadata::extract_headings`), with the legacy id chain
  (section → `id` → `slug` → slugified text) and the same alphabetical keys, so
  `sandbox/heading` renders byte-identically (verified with the real CLI, then
  frozen as golden `32-extract-heading`).
  - Consequence worth knowing: the export used to appear **only** when
    `extract-heading` ran. It is now what §20 always said — automatic — so every
    `solid` document with headings gains one line. Eight goldens were re-frozen
    with exactly that line and nothing else (`06`, `16`, `21`, `22`, `25`, `28`,
    `30`, `31`).
- The payload rule is general now: `run_chain` skips **`extract`** for a
  `Document::Payload` because it is the only stage left that writes
  document-level outputs; every other plugin still runs for its in-document
  effects (ADR-0007's citations-in-a-payload case).
- **Side files are outputs**: `CacheEntry.side_outputs: Vec<OutputInfo>` records
  `{ path, hash }` for every file the render actually wrote, `CACHE_VERSION` is
  `2`, and `is_fresh` requires each of them to still exist with that hash. A
  path that was _not_ written (a block that matched nothing) is not recorded, so
  it cannot make a page permanently stale. The private tmp+rename in `cache.rs`
  became `write_atomically`, used by both the record writer and the side files.
- Goldens `32-extract-heading` (file **and** `export const outline`, next to the
  automatic nested `headings`) and `33-extract-scopes` (P.1.a/b/c) are frozen;
  the harness compares a fixture's side files against `NN-name.out/`.
- `sandbox/extract` runs: it gained `src/reference.yaml`, the `heading`/`extract`
  plugins and two citations, so all four outputs (`.cite.json`, `.heading.yaml`,
  `.heading.json`, the page) are produced.
- Put `extract` **last** in `plugin`. `markdown` is what turns the source lines
  into the nodes `extract` reads; the validator only rejects an element plugin or
  `number` placed after it, so a chain with `extract` before `markdown` is
  accepted and yields half-parsed rows (found while writing golden `32`).
- **Amended 2026-10-30 — a side file accumulates (D-20).** "One path = one array"
  above still holds, but the array is no longer _replaced_ by the last writer:
  blocks, pages and runs **append** to it, a row the file already holds is not
  written twice, and a block whose `format` disagrees with the file's own
  (detected) format fails the page instead of mixing two documents
  ([`D-20`](../workflows/unified-syntax-stage-2/DECISIONS.md), `SYNTAX.md` §21.6).
