# ADR-0017 — Slot modifiers merge in written order

- Status: **Accepted** — D-9 settled by the owner 2026-10-28; implementation
  lands in Phase 2 (`S2-2.5`, `S2-2.6`)
- Depends on: [`0012`](0012-universal-slots.md) (the slot parser),
  [`0014`](0014-field-model.md) (the public field map the result lands in)
- Spec: [`SYNTAX.md`](../spec/SYNTAX.md) §5.4 (modifier run), §13 (merge errors)
- Design:
  [`workflows/unified-syntax-stage-2/DECISIONS.md`](../workflows/unified-syntax-stage-2/DECISIONS.md)
  D-9
- Evidence: `crates/extra/src/slot.rs` modifier tests; the two-symbol `rel`
  fixture

## Context

`[Foo |?~uv$%]` starts a modifier run at `|`, and every symbol after it is
configuration (`bracket.modifier."?" = { rel = "nofollow" }`). Two things were
unspecified: what order the symbols contribute in, and what happens when two
symbols write the same path. The proposal's own example (`rel` from both `?` and
`!`, arriving as an array) is the only hard evidence, and it happens to be
written in source order.

## Decision

1. **Canonical order = first appearance in the bracket, left to right**, after
   deduplication: `|?y!~x` contributes `?`, `y`, `!`, `~`, `x`.
2. **Repetition collapses before merging**, so one bracket can never contribute
   the same symbol twice (`|?xx?|ee?` ≡ `|?x|e`).
3. **Merge by target path:** same path → array (`rel` becomes
   `["nofollow","noopener"]` in symbol order); array + scalar → keep the array;
   array + array → nested array; object + object → deep merge; irreconcilable
   types → `Severity::Error`, and that element falls back to **no** modifier
   contribution at all.
4. Contributions are **specs-level fields** — they land beside `specs.class`,
   never inside `field.*`. (`specs` is a field namespace, not a slot.)
5. **Modifiers are bracket-only.** One in `(…)` is literal content, not an error.
6. `|` may be escaped only in a bracket that is empty or bare-only; `[Foo "Bar"
   \|?x]` is `Severity::Error` because it would put bare after quote (slot order,
   §4).

## Rejected

- **Sorting symbols by code point.** Reproducible, but it makes `?y` and `y?`
  differ from how they read, and the proposal's extraction example is written in
  source order.
- **Last symbol wins instead of merging.** It would drop `rel` from one of the two
  symbols, contradicting the proposal's own output.

## Consequences

Merge order is observable in array contents, so the two-symbol `rel` case is a
required fixture, and §5.4 must state the merge table (it is grammar, not
implementation detail). The error case exercises D-2's "error, and the element
falls back" tier.
