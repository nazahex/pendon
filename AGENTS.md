# AGENTS.md — routing for AI coding agents (Cline, Copilot, …)

This file tells an agent **what to read before acting** and **what to update
after**, so no single giant file has to be loaded. Pendon's old `TODO.md` /
`SYNTAX.md` were too large; docs are now split by concern. Follow this router.

## Read before acting (task → context)

| Task                                           | Read first (in order)                                                                                           |
| ---------------------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| Any task at all                                | [`STATUS.md`](STATUS.md) (what's live), then this table                                                         |
| Touch markup grammar / a `@@`-head rule        | [`docs/spec/SYNTAX.md`](docs/spec/SYNTAX.md)                                                                    |
| Understand _why_ a grammar rule exists         | [`docs/decisions/0001-typed-extras-reconciliation.md`](docs/decisions/0001-typed-extras-reconciliation.md)      |
| Change or add a fixture / golden               | [`docs/spec/golden/README.md`](docs/spec/golden/README.md) + [`docs/guides/testing.md`](docs/guides/testing.md) |
| Work on a specific plugin                      | `crates/<plugin>/README.md` **and** its §N in `docs/spec/SYNTAX.md`                                             |
| Add a new plugin / syntax                      | [`docs/guides/plugin-authoring.md`](docs/guides/plugin-authoring.md) + `CONTRIBUTORS.md`                        |
| Run / change the CLI                           | [`docs/guides/cli.md`](docs/guides/cli.md)                                                                      |
| Run tests / validate output                    | [`CONTRIBUTORS.md`](CONTRIBUTORS.md) + [`docs/guides/testing.md`](docs/guides/testing.md)                       |
| Renderer output contract (Solid/HTML/JSON/AST) | `crates/renderer-*/README.md` if present, else `docs/spec/SYNTAX.md` §6.3                                       |

Do **not** read `docs/archive/**` unless asked — it is historical and frozen.

## Update after acting (change → file to touch)

| You changed…                          | Update                                                                                  |
| ------------------------------------- | --------------------------------------------------------------------------------------- |
| Behaviour verified by a golden        | Re-freeze the `.jsx` (see golden/README); the test is the record — don't hand-edit docs |
| The normative grammar                 | `docs/spec/SYNTAX.md` (+ a new ADR in `docs/decisions/` if it is a _decision_)          |
| Started / blocked / finished work     | `STATUS.md` (one or two lines; delete done lines)                                       |
| A plugin's public behaviour           | that `crates/<plugin>/README.md` (reference the §N; never restate the grammar)          |
| CLI flags                             | `docs/guides/cli.md` (and `README.md` only if it is a headline flag)                    |
| The repo's architecture / conventions | `CONTRIBUTORS.md` (principles) — not SYNTAX.md                                          |

## Hard rules

1. **Single source of truth.** A fact lives in exactly one file; everywhere else
   links. If you find the same rule in two files, keep the spec/ADR one and turn
   the other into a link.
2. **Never invent output.** Render with the real CLI
   (`cargo run --bin pendon -- run` inside a `sandbox/*`) and inspect `out/`
   before writing an example. Sandbox sources live in `sandbox/*/src/`.
3. **Keep `STATUS.md` short.** Long rationale belongs in `decisions/` or
   `archive/`, not the status file.
4. **Respect the layer model.** A plugin only owns its own syntax; shared
   `@@`-head parsing lives in `crates/extra`. See `CONTRIBUTORS.md`.
5. When done, verify: `cargo test -p <affected>` first, then widen.
