# ADR-0014 — Public fields are a projection, not the IR

- Status: **Accepted** — D-3 / D-4 / D-5 / D-6 / D-7 / D-8 settled by the owner
  2026-10-28; implementation lands in Phase 2 (`S2-2.1`–`S2-2.8`)
- Depends on: [`0001`](0001-typed-extras-reconciliation.md) (typed extras),
  [`0008`](0008-type-selectors.md) (per-type config entries)
- Spec: [`SYNTAX.md`](../spec/SYNTAX.md) §6.1 (slot contracts), §6.5 (public
  fields), §6.6 (preset assignment and voiding), §11 (`custom`), §21 (extraction)
- Design:
  [`workflows/unified-syntax-stage-2/DECISIONS.md`](../workflows/unified-syntax-stage-2/DECISIONS.md)
  D-3 – D-8
- Evidence: `crates/extra` projection tests; the element fixtures re-frozen once
  per phase

## Context

The proposal's field names are dotted (`specs.class`, `number.raw`,
`price.details.amount`), but the IR carries flat attributes that _internal_
processors read: `reference_id` feeds `citation_number`, `__number` feeds
nesting. Renaming attributes in place would force every plugin to speak the public
vocabulary, and the proposal explicitly forbids that for overrides ("must not
affect the work of Pendon's internals"). Meanwhile `custom` had two shapes in the
proposal that TOML cannot both hold, and `field.*` needed a consistent key
taxonomy.

## Decision

1. **Two layers, explicitly.** _Internal_ (`reference_id`, `__number`, …) is
   plugin-to-plugin data: never rendered, never extracted, unchanged by Stage 2.
   _Public_ is the **field map**, built once per element at the end of the
   pipeline from (a) built-in fields, (b) slot values under their active
   assignment, (c) `specs.*` / `number.*`, (d) `k: v` at top level. Dotted paths
   are the public truth.
2. **Renderers and `extract` both read the public map.** Neither recomputes a
   name and neither reads internals.
3. **Overrides apply at the projection boundary only.** `field.override.<path>`
   is a rename table consulted when the map is emitted; it never enters
   `internal.*` and never reaches a plugin's own reads. Two overrides producing
   one path, or an override shadowing an emitted field, is `Severity::Error`.
4. **One function, `project(node) -> FieldMap`, is the only place overrides are
   read**; both renderers call it.
5. **Key taxonomy — exactly four prefixes**, as the proposal writes them:
   `field.<slot>.<k>` (assign a slot position), `field.specs.<k>`,
   `field.override.<path>`, and `<slot>.modifier."<c>"`. `bracket`/`paren` are
   slot names; `specs`/`number` are field namespaces, never slots.
6. **Selection vocabulary.** An extract rule is
   `{ preset: Pendon | All = Pendon, include, only, omit }`. `only` + `include` is
   an error; `include`/`only` + `omit` is allowed; `preset = "pendon"` + `only`
   is allowed and yields `[{}]` when nothing matches (that is correct output, not
   a fallback); an unknown preset name is an error listing `pendon` and `all`.
   `preset = "all"` includes custom slot/extras fields but still excludes
   internals.
7. **Voiding is all-or-nothing per matched entry.** Defining _any_ `field.*` in a
   custom entry disables every preset assignment for that element _for that
   entry_; unassigned slots fall back to their bare names (`bracket.bare`,
   `paren.quote`, …).
8. **`custom` is always an array of tables.** An entry without
   `name`/`template`/`imports` is a fields-only entry (with `type` → that type
   only, without → the element default). An entry with neither component keys nor
   `field.*` is an error ("empty custom entry").

## Rejected

- **Dotted names in the IR.** Leaks user configuration into plugin logic and kills
  rule 3.
- **Flat public names.** Cannot distinguish `specs.class` from a user's `class`,
  and `include`/`only`/`omit` need paths.
- **Applying overrides to the AST.** Looks simpler, breaks numbering, citations
  and section nesting — and only goldens would catch it, late.
- **Per-slot voiding.** Friendlier, but the proposal's own example proves the
  all-or-nothing reading.
- **Two serde shapes for `custom`.** Ambiguous in TOML and surprising for anyone
  who copies the proposal's example; a single-table config is a hard load error
  under [`0018`](0018-clean-replacement.md).

## Consequences

Phase 2 builds the map with **zero** output change; Phase 3 moves `specs.*` into
it; Phase 4 moves each element's fields. `backtick_key`/`quote_key` become
`field.specs.tick`/`quote` and `bracket_key`/`parentheses_key` become
`field.bracket.bare`/`field.paren.bare` when assigned — and the old keys are
deleted, not aliased (`S2-2.8`).
