# ADR-0009 — Generic data exports (`DataExport`)

- Status: **Accepted** (landed in Phase 2 of
  [`workflows/numbering-extract-type-export.md`](../workflows/numbering-extract-type-export.md))
- Depends on: [`0007`](0007-mdp-subdocuments.md) (sub-documents skip
  document-level outputs)
- Spec: [`SYNTAX.md`](../spec/SYNTAX.md) §20 (module-level `export const`),
  §11 (custom components)
- Evidence: goldens `25-extras-nested`, `26-cite-scope`, `27-bind-mdp` stayed
  byte-identical; `apps/cli/tests/bind_inject_spec.rs` covers the new producer
- Supersedes: the four `*_export*` hint fields in `SolidRenderHints`

## Context

`renderer-solid` emits four kinds of `export const` today, each through its own
path: `frontmatter` and `headings` are computed from the AST inside
`render_solid_with_hints`, while `references` and `citations` ride two dedicated
hint fields (`references_export` + `references_export_name`,
`citations_export` + `citations_export_name`) that `plugin-cite` fills. Two
consequences:

1. **Every new payload needs a new field pair.** Phase 2's
   `[[task.bind.inject]]` and Phase 4's `extract.export` both need to publish a
   value, and neither is a citation.
2. **Ordering is implicit.** References must precede citations because a
   citation names a reference; the ordering is hard-coded in `lib.rs` rather
   than expressed by the list.

## Decision

**`renderer-solid` owns emission; producers register `(name, Value)`.** (`D-5`,
`D-6`.)

1. `DataExport { name, value, source }` with
   `source: ExportSource { Automatic, Plugin(String) }`; `SolidRenderHints`
   carries one ordered `exports: Vec<DataExport>` instead of the four
   `*_export*` fields.
2. Emission is one pass over the list in insertion order, then
   `export default function`; `push_data_export` stays the single writer.
3. `frontmatter` and `headings` register first, as `Automatic`, from the AST —
   before any plugin registration, so goldens keep their current order.
4. **Two exports with the same name: the last registration wins entirely** —
   value _and_ position — and only one line is emitted.
5. Only the `solid` format emits exports. A non-solid task drops the list; a
   `Payload`/`mdp` sub-document registers nothing.

## Rejected alternatives

- **Keep per-payload fields and add two more.** Grows the hint struct per
  feature and keeps the ordering knowledge in `lib.rs`.
- **A `BTreeMap<String, Value>`.** Sorts names, which would reorder goldens and
  make "last wins" ambiguous about position.
- **Emit from the plugin crate.** A plugin would own JS syntax and the
  "only for solid" rule; `plugin-cite` would need to know the renderer.
- **Emit exports in every format.** HTML/JSON have no module scope; a silent
  no-op is kinder than a JSON key nobody reads.

## Consequences

- `grep references_export_name crates/renderer-solid` is empty after the change.
- Identifier validation moves to one shared helper (`<task>.<key> '…' is not a
  valid JavaScript identifier`, reserved words rejected) reused by `cite`,
  `bind` and `extract`.
- New producers (bind inject, extract) are a `push`, not a schema change.

## Outcome (2026-10-25)

- `DataExport`/`ExportSource` live in `crates/renderer-solid/src/imports.rs` and
  are re-exported from the crate root; `SolidRenderHints.exports` is the only
  export channel — `grep references_export_name crates/` finds nothing.
- `collect_exports` registers `frontmatter` (then `headings`) from the AST only
  when the document has them, appends the hints' registrations in push order, and
  resolves duplicates by "remove the earlier one, push the later one".
- `plugin-cite` pushes `references` then `citations`, so the ordering that used to
  be hard-coded in `lib.rs` is now the order of the pushes; the
  `*_export_name` config keys stayed as sugar (goldens use them).
- `[[task.bind.inject]]` (`apps/cli/src/plugins.rs::build_bind_injects`) resolves
  `source` against the project root with the task's `[foo]` captures substituted,
  names the binding from `export` or the file stem, parses with
  `pendon_plugin_bind::parse_data_file` (the block converter), pushes
  `DataExport::plugin("bind", …)`, and adds the file to the task's cache `deps`.
  The section only applies when `bind` is in `task.plugin`, like every other
  config-driven plugin.
- `validate_export_identifier` (`apps/cli/src/utils.rs`) is the single validator;
  it rejects the ES reserved set plus `await`, the strict-mode keywords and
  `eval`/`arguments` (`of` is contextual and allowed).
- Non-Solid tasks drop the list: `process.rs` only builds inject hints on the
  `solid` branch, and no other renderer reads `exports`.
