# ADR-0015 — `extract` becomes a main crate; frontmatter is extractable

- Status: **Accepted** — D-13 (extract half) and D-14 settled by the owner
  2026-10-28; the promotion itself is `S2-7.3`, the frontmatter rule `S2-7.4`
- Depends on: [`0014`](0014-field-model.md) (the field map it consumes),
  [`0011`](0011-plugin-extract.md) (field-level extraction)
- Spec: [`SYNTAX.md`](../spec/SYNTAX.md) §21 (extraction), §21.x (frontmatter),
  §20 (module exports)
- Design:
  [`workflows/unified-syntax-stage-2/DECISIONS.md`](../workflows/unified-syntax-stage-2/DECISIONS.md)
  D-13, D-14
- Evidence: `crates/extract` tests; one fixture per output format; the duplicate
  `export const frontmatter` error case

## Context

`plugin-extract` is a plugin in name only: extraction is driven by
`[[task.<element>.extract]]`, which is _task_ configuration, not a markup
construct — the CLI imports the crate directly (`apps/cli/src/plugins.rs`,
`process.rs`). Frontmatter is already an element in the head list and can already
export `export const frontmatter`, so "it cannot be extracted" is an artefact of
the crate boundary, not a design choice.

## Decision

1. **Pure mechanical promotion**: `crates/plugin-extract` → `crates/extract`,
   package `pendon-extract`. No behaviour change in that task, and the config
   surface does not change — `[[task.<element>.extract]]` is what users write;
   they never see a crate name.
2. **A promotion changes the crate layer, nothing else.** If a type existed only
   to satisfy the `Plugin` trait it stops being a `Plugin`; if it is a chain stage
   it stays one. Pipeline order is verified in `apps/cli/src/chain.rs` before
   editing, never changed by the rename.
3. **Frontmatter is extractable like any element**, with `output`, `format`
   (`json`/`yaml`/`toml`/`export`), `only` and `omit`. Its fields are the
   top-level frontmatter keys and its paths are nested keys.
4. **Frontmatter has no preset**: `preset` and `include` are errors ("frontmatter
   has no preset; use only/omit"), because it has no element fields — only user
   data.
5. A pre-existing `export const frontmatter` (§20) and an `export`-format extract
   block for frontmatter must not define the same const twice; the duplicate is
   `Severity::Error`.

## Rejected

- **Renaming the configuration** to `[[task.<element>.extraction]]`. Churn for
  users, zero benefit.
- **Treating frontmatter as a preset element.** It has user data, not element
  fields, so a preset list would be a lie.

## Consequences

`docs/spec/EXTRAS-INVENTORY.md` becomes `FIELDS-INVENTORY.md` in the same PR as
the crate move, so presets and inventory change together. `CONTRIBUTORS.md` gains
one sentence distinguishing a main crate from a plugin crate.
