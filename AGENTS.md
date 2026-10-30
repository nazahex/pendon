# AGENTS.md — routing for AI coding agents (Cline, Copilot, …)

This file tells an agent **what to read before acting** and **what to update
after**, so no single giant file has to be loaded. Pendon's old `TODO.md` /
`SYNTAX.md` were too large; docs are now split by concern. Follow this router.

## Read before acting (task → context)

| Task                                                   | Read first (in order)                                                                                                                                                                                        |
| ------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Any task at all                                        | [`STATUS.md`](STATUS.md) (what's live), then this table                                                                                                                                                      |
| Touch markup grammar / a `@@`-head rule                | [`docs/spec/SYNTAX.md`](docs/spec/SYNTAX.md); if the change is part of the rework, also [`docs/workflows/unified-syntax-stage-2/`](docs/workflows/unified-syntax-stage-2/README.md)                          |
| Understand _why_ a grammar rule exists                 | [`docs/decisions/0001-typed-extras-reconciliation.md`](docs/decisions/0001-typed-extras-reconciliation.md)                                                                                                   |
| Touch `type = …` / custom component selection          | §11 of [`docs/spec/SYNTAX.md`](docs/spec/SYNTAX.md) + [`ADR-0008`](docs/decisions/0008-type-selectors.md) + `crates/types/README.md`                                                                         |
| Change or add a fixture / golden                       | [`docs/spec/golden/README.md`](docs/spec/golden/README.md) + [`docs/guides/testing.md`](docs/guides/testing.md)                                                                                              |
| Work on a specific plugin                              | `crates/<plugin>/README.md` **and** its §N in `docs/spec/SYNTAX.md`                                                                                                                                          |
| Touch `[[task.<element>.extract]]` / extraction        | `crates/extract/README.md` + `docs/spec/FIELDS-INVENTORY.md` (the frozen field tables) + P.1 of `docs/archive/fulfilled/plugin-extract.md`                                                                   |
| Touch `plugin-bind` data blocks (`{{{lang[var] … }}}`) | `crates/plugin-bind/README.md`, §19 in `docs/spec/SYNTAX.md`, ADR-0004 + ADR-0005, the RFC's §2.5–§2.8                                                                                                       |
| Add a new plugin / syntax                              | [`docs/guides/plugin-authoring.md`](docs/guides/plugin-authoring.md) + `CONTRIBUTORS.md`; during the rework also [`docs/workflows/unified-syntax-stage-2/`](docs/workflows/unified-syntax-stage-2/README.md) |
| Run / change the CLI                                   | [`docs/guides/cli.md`](docs/guides/cli.md)                                                                                                                                                                   |
| Touch CLI report output / styling                      | [`docs/spec/CLI-REPORT.md`](docs/spec/CLI-REPORT.md)                                                                                                                                                         |
| Touch `crates/tui` / how the report is drawn           | [`docs/rfc/tui-overhaul.md`](docs/rfc/tui-overhaul.md) + [`docs/spec/CLI-REPORT.md`](docs/spec/CLI-REPORT.md)                                                                                                |
| Run tests / validate output                            | [`CONTRIBUTORS.md`](CONTRIBUTORS.md) + [`docs/guides/testing.md`](docs/guides/testing.md)                                                                                                                    |
| Touch the editor grammar (`editors/vscode/`)           | [`editors/vscode/README.md`](editors/vscode/README.md) + the §N it names in `docs/spec/SYNTAX.md`                                                                                                            |
| Renderer output contract (Solid/HTML/JSON/AST)         | `crates/renderer-*/README.md` if present, else `docs/spec/SYNTAX.md` §20 (module-level exports) / §6.3 (values)                                                                                              |

Do **not** read `docs/archive/**` unless asked — it is historical and frozen.

## Update after acting (change → file to touch)

| You changed…                            | Update                                                                                  |
| --------------------------------------- | --------------------------------------------------------------------------------------- |
| Behaviour verified by a golden          | Re-freeze the `.jsx` (see golden/README); the test is the record — don't hand-edit docs |
| The normative grammar                   | `docs/spec/SYNTAX.md` (+ a new ADR in `docs/decisions/` if it is a _decision_)          |
| Started / blocked / finished work       | `STATUS.md` (one or two lines; delete done lines)                                       |
| A plugin's public behaviour             | that `crates/<plugin>/README.md` (reference the §N; never restate the grammar)          |
| The type resolver (`Selector`, `Group`) | `crates/types/README.md` + ADR-0008 (grammar stays in §11)                              |
| CLI flags                               | `docs/guides/cli.md` (and `README.md` only if it is a headline flag)                    |
| The CLI report format / styling         | `docs/spec/CLI-REPORT.md`                                                               |
| `crates/tui`'s API / drawing behaviour  | `crates/tui/README.md` (and, until the overhaul lands, `docs/rfc/tui-overhaul.md`)      |
| The editor grammars / scope harness     | `editors/vscode/README.md` (link the §N; never restate the grammar)                     |
| The repo's architecture / conventions   | `CONTRIBUTORS.md` (principles) — not SYNTAX.md                                          |

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
6. **No compatibility shims.** Pendon is alpha and every rework is a clean
   replacement: no `#[serde(alias)]`, no `#[deprecated]`, no dual parsing, no
   migration notes. Removed config keys are a load **error**; removed syntax is
   plain text. See D-16 in
   [`docs/workflows/unified-syntax-stage-2/DECISIONS.md`](docs/workflows/unified-syntax-stage-2/DECISIONS.md).
7. **Goldens are the contract; sandboxes are scratch.** Some `sandbox/*` baselines
   predate the current pipeline and are **maintainer-owned** (live list: the
   "Deferred" item in [`STATUS.md`](STATUS.md)). You may still render one to
   inspect real output (rule 2), but never commit its regenerated `out/` as part of
   a change, never freeze it into a golden, and never read its diff as a
   regression — `cargo test --workspace` decides. When output changes
   intentionally, re-freeze the fixture under `docs/spec/golden/`.
