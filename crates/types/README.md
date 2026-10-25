# `pendon-types`

Type selectors and **document-scoped** type resolution. The implementation
design is
[`docs/workflows/type-resolution.md`](../../docs/workflows/type-resolution.md);
the decision is
[ADR-0008](../../docs/decisions/0008-type-selectors.md); the normative grammar is
[`SYNTAX.md` §11](../../docs/spec/SYNTAX.md).

## Why a crate

Three consumers ask the same question — "which rule owns this element's type?" —
and they must never disagree:

| Consumer         | Question                                          |
| ---------------- | ------------------------------------------------- |
| `renderer-solid` | which component renders this element              |
| `plugin-number`  | which numbering block this element belongs to     |
| `plugin-extract` | which extraction block(s) this element belongs to |

`pendon-types` depends on nothing but `std` (plus `serde` derives, because
`Selector` rides on `renderer-solid`'s `TypedComponent`), so every consumer can
share it.

## The model

```rust
Selector::{Exact(String), Question, Underscore, Star}   // "TypeA", ?, _, *
TypeSet    // the document's occurring types (literals + untyped)
Rule<P, T> { selectors, payload, target }               // P/T are the caller's
Group<P, T>::new(rules)                                 // caches `defined`
```

Resolution is per **definition group** — one element section of one task
(`[[task.heading.custom]]`, `[[task.heading.number]]`, …):

1. `defined` = every literal **any** rule in the group names, plus `untyped` if
   any rule wrote `?`. It is computed once, before any resolution, so a later
   rule cannot change what an earlier `_` meant.
2. `_` = occurring − defined; `*` = occurring; `?` = untyped.
3. Specificity: literal / `?` = 2, `_` / `*` = 1. Highest wins; equal
   specificity → **later registration** wins.

Two entry points:

- `Group::winner(...)` — exclusive consumers (one component, one counter).
  `None` means "no rule covers this type; use the default".
- `Group::matching(...)` — additive consumers (`plugin-extract`). One rule per
  destination; a destination claimed twice is resolved by the same
  specificity-then-order rule.

## Invariants pinned by tests

- `??` / `**` / empty entries are **rejected**, never mapped to a wildcard.
- A literal named `untyped` is a normal literal, distinct from the untyped
  identity.
- An empty `TypeSet` is empty: `_` and `*` cover nothing on a marker-free
  document (they never behave like "match everything").
- `winner` is always a member of `matching` for the same type.
- Iteration is `BTreeSet`-ordered (literals sorted, `untyped` last), so output
  that reaches a renderer is deterministic.

Run them with `cargo test -p pendon-types`.

## State

Phase 1 of
[`docs/workflows/TASKS.md`](../../docs/workflows/TASKS.md) builds this crate
(T1.1–T1.6). The consumers follow: the CLI parses `type` into selectors
(T1.7), `renderer-solid` resolves them per document and the element plugins stop
choosing components (T1.8), positional keys stay literal-only (T1.9).
