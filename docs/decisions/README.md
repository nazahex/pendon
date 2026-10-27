# Decisions (ADRs)

Append-only records of _why_ the syntax is the way it is. The **what** lives in
[`../spec/SYNTAX.md`](../spec/SYNTAX.md); this folder holds the rationale, the
rejected alternatives, and the evidence that pins each decision.

| ADR                                         | Title                                                   | Status                  |
| ------------------------------------------- | ------------------------------------------------------- | ----------------------- |
| [0001](0001-typed-extras-reconciliation.md) | Typed Extras Reconciliation (D1–D10)                    | Accepted / complete     |
| [0002](0002-decorator-binder.md)            | Decorator binder: wrapper node with `__plugin_kind`     | Accepted                |
| [0003](0003-positional-groups.md)           | Positional groups `@@type[…](…){…}` (Option 2)          | Accepted / done         |
| [0004](0004-pre-parse-bind-stage.md)        | `plugin-bind` runs a pre-parse text stage               | Accepted / building     |
| [0005](0005-structured-extras-values.md)    | Structured extras values + the `U+E001` transport       | Accepted / complete     |
| [0006](0006-per-file-cache-records.md)      | One cache record per `(task, input)`                    | Accepted / complete     |
| [0007](0007-mdp-subdocuments.md)            | `mdp` payloads are sub-documents (fragments)            | Accepted / complete     |
| [0008](0008-type-selectors.md)              | `type` is a selector list resolved per document         | Accepted / implementing |
| [0009](0009-generic-data-exports.md)        | One `export const` writer (`DataExport`)                | Accepted / complete     |
| [0010](0010-plugin-number.md)               | `plugin-number` owns numbering; `%%` grammar in `extra` | Proposed                |
| [0011](0011-plugin-extract.md)              | `plugin-extract`: field-level extraction per element    | Proposed                |
| [0012](0012-universal-slots.md)             | Slots are universal and parsed before Markdown          | Accepted / building     |
| [0013](0013-decorator-unification.md)       | `@@` is the only decorator, and it comes first          | Accepted / building     |
| [0014](0014-field-model.md)                 | Public fields are a projection, not the IR              | Accepted / building     |
| [0015](0015-extract-promotion.md)           | `extract` is a main crate; frontmatter is extractable   | Accepted / building     |
| [0016](0016-number-promotion.md)            | `number` is a main crate and exports `number.*`         | Accepted / building     |
| [0017](0017-slot-modifiers.md)              | Slot modifiers merge in written order                   | Accepted / building     |
| [0018](0018-clean-replacement.md)           | Clean total replacement: no compatibility of any kind   | Accepted / enforced     |

## Conventions

- One decision (or one tightly-coupled set) per file, numbered, never edited after
  acceptance — supersede with a new ADR instead.
- State the context, the decision, the rejected alternatives, and the evidence
  (tests/fixtures that pin it).
- Cross-link to the spec section (`SYNTAX.md §N`) it justifies.
