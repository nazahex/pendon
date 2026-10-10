# Documentation Index

Start here. Each document has **one job**; every fact lives in exactly one place
and the rest link to it. If you are an AI agent, also read [`AGENTS.md`](../AGENTS.md).

## The trust hierarchy (most authoritative first)

1. **Executable truth** — golden fixtures + tests. If docs and code disagree,
   the test is right. Fix the doc or the code, never the test to match a guess.
2. **Normative spec** — [`spec/SYNTAX.md`](spec/SYNTAX.md). The grammar (RFC 2119).
3. **Decisions** — [`decisions/`](decisions/). Why the grammar is the way it is.
4. **Guides** — [`guides/`](guides/). Task-oriented how-tos.
5. **Status** — [`STATUS.md`](../STATUS.md). What is being worked on now.

## "I want to…" → read this

| I want to…                              | Read                                                                                             |
| --------------------------------------- | ------------------------------------------------------------------------------------------------ |
| Understand what Pendon is               | [`README.md`](../README.md)                                                                      |
| Look up the exact markup grammar        | [`spec/SYNTAX.md`](spec/SYNTAX.md)                                                               |
| See a worked example as executable data | [`spec/golden/`](spec/golden/) + [`spec/golden/README.md`](spec/golden/README.md)                |
| Know why a syntax rule exists (D1–D10)  | [`decisions/0001-typed-extras-reconciliation.md`](decisions/0001-typed-extras-reconciliation.md) |
| See what is being worked on / blocked   | [`STATUS.md`](../STATUS.md)                                                                      |
| Run the CLI                             | [`guides/cli.md`](guides/cli.md)                                                                 |
| Author or modify a plugin               | [`guides/plugin-authoring.md`](guides/plugin-authoring.md)                                       |
| Write/run tests, goldens, sandboxes     | [`guides/testing.md`](guides/testing.md)                                                         |
| Contribute (principles, PR checklist)   | [`CONTRIBUTORS.md`](../CONTRIBUTORS.md)                                                          |
| Read a plugin's public behaviour        | that crate's `crates/plugin-*/README.md`                                                         |
| Propose a new feature                   | [`rfc/`](rfc/) (create `rfc/<feature>.md`)                                                       |
| Read superseded/old material            | [`archive/`](archive/) (historical, frozen)                                                      |

## Layout

```
/  README.md            what/why/quickstart (entry)
   AGENTS.md            AI-agent routing: task → read → update
   STATUS.md            LIVE status (this is the small, frequently-updated file)
   CONTRIBUTORS.md      principles + PR checklist

docs/
   INDEX.md             this map
   spec/SYNTAX.md       normative grammar (§1–§15 normative, §16 fixture ref)
   spec/PARSER.md       parser/IR contract
   spec/golden/         executable fixtures (ground truth) + README
   decisions/           ADRs (append-only) + index
   guides/              how-tos: cli, plugin-authoring, testing
   rfc/                 proposals (pre-spec)
   archive/             historical, frozen

memory-bank/            Cline per-session working memory (not project truth)
crates/*/README.md      per-plugin public reference (link to spec, don't restate it)
```

## Keeping docs up to date (the mechanism)

- **Code/behaviour changed?** The failing golden forces you to re-freeze
  (`pendon run -F`). That is the enforcement point — docs follow it.
- **Grammar changed?** Update `spec/SYNTAX.md` **and** add/adjust an ADR in
  `decisions/`. Never bury a rule in a README.
- **Work started/blocked/done?** Update `STATUS.md` only (one or two lines).
- **New doc needed?** Add it here in the map. Don't grow an existing doc into a
  second concern — that is what made the old `TODO.md` / `SYNTAX.md` fat.
