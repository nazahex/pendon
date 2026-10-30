# ADR-0016 — `number` becomes a main crate and exports `number.*`

- Status: **Accepted** — D-13 (number half) settled by the owner 2026-10-28, plus
  `OBJ-6`/`OBJ-8` (`number.depth`); promotion is `S2-9.3`, the public projection
  `S2-9.1`/`S2-9.2`
- Depends on: [`0010`](0010-plugin-number.md) (`number` owns numbering),
  [`0014`](0014-field-model.md) (the public field map)
- Spec: [`SYNTAX.md`](../spec/SYNTAX.md) §10 (numbering), §10.5 (`number.*`
  public fields)
- Design:
  [`workflows/unified-syntax-stage-2/DECISIONS.md`](../workflows/unified-syntax-stage-2/DECISIONS.md)
  D-13, `OBJ-6`, `OBJ-8`
- Evidence: `crates/number` tests; the re-frozen `PE.1` preset with `number.raw`;
  the heading fixture with `number.depth`

## Context

Numbering is read by every element — headings nest, tables and figures carry the
current counters — yet it shipped as `pendon-plugin-number` and is imported
directly by the CLI. Its values reach the rest of the system as internal attributes
(`__number`), which today is also what nesting reads. Extraction needs them as
_public_ fields, and the two must not become the same thing: nesting must keep
reading the internal value even when the public name is renamed.

## Decision

1. **Pure mechanical promotion**: `crates/plugin-number` → `crates/number`,
   package `pendon-number`, in a task that changes no behaviour. The config
   surface stays `[[task.<element>.number]]`; plugin tokens are untouched.
2. **The public fields are `number.*`** (`number.raw`, `number.depth`, …),
   produced by the projection from the internal `__number` metadata. The
   projection **copies**, it never recounts: `number.depth` _is_ the depth nesting
   assigned, so a plugin cannot observe two different depths for one element.
3. **`number_depth` does not exist.** The proposal's §7.4 spelling is not part of
   the field model; the namespaced `number.depth` is the only name, one rule for
   every element.
4. **Internals stay internal.** `__number` remains the plugin-to-plugin attribute;
   nothing renders or extracts it, and `preset = "all"` still excludes it.
5. `number.raw` is extractable from any element that uses numbering, is excluded
   from `preset = "pendon"` (`PE.1`), and is reachable via `include`/`only`/
   `preset = "all"`.

## Rejected

- **Keeping the crate named `plugin-number`.** It is not a plugin in any
  observable sense, and the name is what makes agents look for a `Plugin` impl
  that should not exist.
- **Emitting `number.depth` by re-counting nesting in the projection.** Two
  counters that can disagree, and goldens would only catch it at the leaves.
- **Renaming the internal attribute to `number.*`.** It would make renaming a
  public field (`field.override.number.raw`) silently change nesting.

## Consequences

`PE.1` is re-frozen with `number.raw` when the projection lands, and `§10.5`
states the field list and the "copy, don't recount" rule. `field.override` on a
`number.*` path affects extraction and render only — never numbering.

## Implementation note (S2-9.1/S2-9.2, 2026-10-30)

The promotion is `crates/plugin-number` → `crates/number`, package
`pendon-number`, with no behaviour change (S2-9.1).

Rule 2's _producer_ is the numbering pass, not `project`: `process` owns the
counter, so it writes `number.raw`, `number.depth` and `number.type` into the
node's attribute block and the projection copies them like any other attribute.
That is the only shape that works for **element** numbering, which has no
`__number` at all — and it keeps the two counters from existing, because the
published depth is the value `Counter::increment` was given. `field.override` in
an element's own `[[task.<element>.custom]]` entry renames a `number.*` field at
the projection boundary (D-4, unchanged); an override declared in a `number`
block is applied by the pass itself (S2-9.3), because no `custom` entry answers a
`%%` marker's `[[task.number.custom]]` counter.
