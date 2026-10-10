# Decisions (ADRs)

Append-only records of _why_ the syntax is the way it is. The **what** lives in
[`../spec/SYNTAX.md`](../spec/SYNTAX.md); this folder holds the rationale, the
rejected alternatives, and the evidence that pins each decision.

| ADR                                         | Title                                               | Status              |
| ------------------------------------------- | --------------------------------------------------- | ------------------- |
| [0001](0001-typed-extras-reconciliation.md) | Typed Extras Reconciliation (D1–D10)                | Accepted / complete |
| [0002](0002-decorator-binder.md)            | Decorator binder: wrapper node with `__plugin_kind` | Accepted            |
| [0003](0003-positional-groups.md)           | Positional groups `@@type[…](…){…}` (Option 2)      | Accepted / done     |

## Conventions

- One decision (or one tightly-coupled set) per file, numbered, never edited after
  acceptance — supersede with a new ADR instead.
- State the context, the decision, the rejected alternatives, and the evidence
  (tests/fixtures that pin it).
- Cross-link to the spec section (`SYNTAX.md §N`) it justifies.
