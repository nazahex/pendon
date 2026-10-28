# Unified Syntaxes — Stage 2 (workflow)

**Status: planning. Nothing implemented.** This folder is the plan of record for
the rework proposed in
[`docs/issues/unified-syntax-stage-2.mdp`](../../issues/unified-syntax-stage-2.mdp).

| file                           | job                                                                |
| ------------------------------ | ------------------------------------------------------------------ |
| `README.md` (this)             | scope, vocabulary→code map, phase ladder, gates, risks             |
| [`DECISIONS.md`](DECISIONS.md) | every open design question, with a recommendation and sign-off box |
| [`TASKS.md`](TASKS.md)         | the atomic, ordered, agent-sized task list                         |

**Replacement policy (D-16).** Stage 2 is a clean, total replacement: no backward
compatibility, no deprecation warnings, no migration notes, no legacy code paths.
Old configs fail fast; removed syntax is simply not syntax. There is deliberately
**no** migration document — read the new grammar in `spec/SYNTAX.md` and adopt it.

Authority order, unchanged: **goldens > `spec/SYNTAX.md` > ADRs > these docs**.
The proposal says _what the user wants_; this folder says _in what order and how_
we build it. Do not extend the proposal in place — amend a decision here.

The earlier Indonesian draft
[`docs/issues/draft/unified-syntax-stage-2_id.mdp`](../../issues/draft/unified-syntax-stage-2_id.mdp)
is the same text in another language. **Do not read it as canonical and do not
merge it**; the English file above is the source.

## 1. What Stage 2 changes

Eight moving parts, in dependency order:

1. **Terminology** — _parties/regions_, _elements_, _heads_, _extras_ (`@@type`,
   `{ specs }`, `%%numbering`), _slots_, _slot tag_, _slot contract_, _slot
   modifier_, _decorator_, _fields_ (bare / built-in / assign / override /
   preset assignment).
2. **Universal slots** — `[]` and `()` stop belonging to anchor/cite/image and
   become positional containers every element may contract, with three content
   kinds (**bare**, **quote**, **tick**) in a fixed order.
3. **Slot tags** — `@` `^` `!` `#` are the only way a slot that stands alone
   says which element owns it. Hard-coded, closed set.
4. **Slot modifiers** — `[Foo |?~uv$%]` becomes a _user-defined_ shortcut for
   specs; Pendon ships **no** built-in modifiers. The old per-plugin modifier
   mechanisms are deleted without nostalgia.
5. **Specs** — `{}` is renamed _specs_, its fields become `specs.tick`,
   `specs.quote`, `specs.class`, `specs.id`, `specs.style` (a `k: v` stays `k`),
   and every specs field becomes assignable.
6. **Decorator** — `@@` is mandatory and first; slot → specs → numbering follow;
   the receiver set grows to blockquote, codefence, list, paragraph, table.
7. **Fields** — one model for what an element exposes: preset assignment
   (built-in names), bare fields for everything uncontracted, `field.*`
   assignment, `field.override.*` renaming, all validated by one field-name rule.
8. **Plugin rework** — `plugin-extract` → **`extract`** and `plugin-number` →
   **`number`** (main crates), `preset = "pendon"` / `preset = "all"` /
   `include`, frontmatter extraction, `FIELDS-INVENTORY.md`, VSCode grammar.

## 2. Goals / non-goals

**Goals**

- One grammar for slots, one grammar for extras, one grammar for decorators —
  no per-element dialect (standing rule 1 of `CONTRIBUTORS.md`).
- Every hard-coded convenience that is really _policy_ leaves the engine:
  external-URL `target`, URL/prefix modifiers, figure, heading numbering,
  cite `citation_class` / `*_prefix` / `instance_index`.
- Public names become the user's choice: assignment, override and extraction
  all speak one field vocabulary.
- The preset becomes small and honest (`preset = "pendon"` = built-in fields
  only), with `include` / `only` / `omit` / `preset = "all"` for everything else.

**Non-goals (do not touch in Stage 2)**

- **Compatibility of any kind** — no aliases, no shims, no deprecation warnings,
  no migration doc, no dual parsing (D-16). "Removed" means the code is gone.
- `plugin-wiki` / wikilink (planned full overhaul — proposal §Notes).
- `plugin-vicado`, `plugin-dialog`, `plugin-quiz` (scheduled for deletion).
- Renumbering `spec/SYNTAX.md` (see D-15), the TUI, the CLI report, and the
  diagnostics RFC.
- Any behaviour a golden does not cover. A new rule ships with a fixture.

## 3. Vocabulary → code

Left column is proposal language; right column is what an agent actually touches.

| proposal term         | today's name / location                                                                                |
| --------------------- | ------------------------------------------------------------------------------------------------------ |
| element, head         | plugin + its head parser (`crates/plugin-*/src/lib.rs`, `read_head` in `crates/extra`)                 |
| extras                | `@@type` + `{ … }` + `%%numbering`, parsed by `crates/extra/src/lib.rs` + `typed.rs`                   |
| specs                 | §5 "extras head grammar" / §6 "extras → attribute mapping"; `to_attributes`                            |
| numbering             | §10.5 `%%numberType` → `__number` → `crates/plugin-number`                                             |
| slot                  | brackets/parentheses parsed ad hoc by `plugin-anchor`, `plugin-cite`, `plugin-image`, `plugin-heading` |
| slot tag              | today: the whole head (`@[`, `[^^]`, `![`, `#[`)                                                       |
| slot contract         | `SlotContract` in `crates/extra/src/slot.rs`, declared per element (S2-1.x)                            |
| slot modifier         | today: anchor URL suffix + image pre-bracket — **both deleted**                                        |
| decorator             | `crates/extra/src/decorator.rs` (`bind_decorators`), §9.1                                              |
| bare / built-in field | `AttrValue` on the node; rows in `docs/spec/EXTRAS-INVENTORY.md`                                       |
| assign / override     | `field.*` (D-5) + `Specs::resolve`; the four old naming keys are deleted (S2-2.8)                      |
| preset                | today: `recommended: bool` in `ExtractRule` (`apps/cli/src/config.rs`)                                 |
| extract               | `crates/plugin-extract` + `[[task.<element>.extract]]` (§21)                                           |
| number                | `crates/plugin-number` + `[[task.<element>.number]]` (ADR-0010)                                        |

## 4. Phase ladder

Each phase is independently shippable and leaves `cargo test --workspace` green.
Phases are ordered by _what the next phase reads_, not by proposal order.

| phase | outcome                                                           | gate (true before leaving the phase)                      |
| ----- | ----------------------------------------------------------------- | --------------------------------------------------------- |
| 0     | decisions signed off, ADRs staged, spec map fixed                 | `DECISIONS.md` has no `TBD`; ADR 0012+ exist              |
| 1     | slot grammar exists in `crates/extra` (parsed, not wired)         | unit tests green; no golden byte moves                    |
| 2     | field model: names, assignment, override, validation, mapping     | unit tests green; no golden byte moves                    |
| 3     | specs fields renamed (`specs.class` …) and assignable             | re-frozen goldens reviewed; renderer mapping tested       |
| 4     | elements reworked one at a time (anchor, cite, heading, image, …) | per-element fixture re-frozen + README + §7.x updated     |
| 5     | decorator unified (`@@` first, five receivers, warnings)          | decorator fixtures green; table decorator works           |
| 6     | slot modifiers (config, merge rule, extraction visibility)        | modifier fixture + collision errors                       |
| 7     | `FIELDS-INVENTORY.md` frozen (presets per element)                | tables reviewed; extract presets match the tables         |
| 8     | `extract` crate: `preset`/`include`, nested paths, frontmatter    | PE.1–PE.4 + frontmatter fixtures; PE.4 yields `[{}]`      |
| 9     | `number` crate + `number.*` fields, overridable                   | number fixtures re-frozen; extract sees `number.raw`      |
| 10    | VSCode grammar, docs sweep, full gates                            | clippy/fmt/test green, sandboxes re-rendered, `STATUS.md` |

Dependency sketch (`a → b` means _b needs a_):

```text
0 ─┬─> 1 ─┬─> 4 ─┬─> 5
   │      │      └─> 6
   └─> 2 ─┴─> 3 ─┬─> 7 ─> 8 ─> 9 ─> 10
```

Phase 3 (specs) is the widest break: it changes what _every_ element emits, so it
lands as one pass with one coordinated re-freeze, **before** the per-element
reworks.

## 5. Definition of done (every task)

1. `cargo test -p <crate>` first, then `cargo test --workspace`.
2. `cargo clippy --workspace --all-targets` and `cargo fmt --all -- --check`.
3. Output that changed on purpose is **re-frozen**, never hand-edited:
   `cargo run --bin pendon -- run -F` inside a copy of the fixture project
   ([golden README](../../spec/golden/README.md)).
4. `spec/SYNTAX.md` §N updated (grammar) — never a README.
5. The owning `crates/*/README.md` links that §N; it does not restate it.
6. `STATUS.md` gets one line while in flight; delete it when done.
7. A behaviour with no fixture is not done. Add/extend a golden or unit test.

## 6. Working a task (recipe)

```bash
# 1. read the task + the decision it depends on
less docs/workflows/unified-syntax-stage-2/TASKS.md

# 2. prove current behaviour before touching it
cd sandbox/<closest> && cargo run --bin pendon -- run -F && git diff out/

# 3. implement, then
cargo test -p <crate>            # focused
cargo test --workspace           # widened
cargo run --bin pendon -- run -F # in the fixture copy; re-freeze only what moved
```

## 7. Doc-update matrix

| you changed                     | update                                                               |
| ------------------------------- | -------------------------------------------------------------------- |
| slot grammar / tags / modifiers | `spec/SYNTAX.md` §5.2–§5.4 (new), ADR-0012                           |
| a slot contract                 | `spec/SYNTAX.md` §6.1 (rewritten) + that element's §7.x/§8/§9.x      |
| fields (assign/override/preset) | `spec/SYNTAX.md` §6.5 (new), `spec/FIELDS-INVENTORY.md`, ADR-0014    |
| decorator rule                  | `spec/SYNTAX.md` §9.1 + ADR-0013                                     |
| an element's behaviour          | `crates/plugin-*/README.md` (link the §N) + a golden                 |
| `extract` / `number` promotion  | `crates/extract/README.md`, `crates/number/README.md`, ADR-0015/0016 |
| removed syntax or config key    | `spec/SYNTAX.md` §14 (removal only), the §13 error row, `STATUS.md`  |
| editor scopes                   | `editors/vscode/README.md` (never restate the grammar)               |

## 8. Risks

| risk                                      | mitigation                                                                  |
| ----------------------------------------- | --------------------------------------------------------------------------- |
| **D-3** breaks every custom template      | decide in Phase 0; land in Phase 3 with one test per renderer output        |
| slots parsed _after_ markdown ate `[`/`(` | parse slots pre-parse from raw text, like `plugin-bind` (D-1)               |
| `[^ref]` collides with Markdown footnotes | OBJ-1: verify the core lexer first; if it collides, change the marker       |
| removals delete code a golden still needs | delete code + fixture in the _same_ task; the old form becomes literal text |
| someone "helpfully" adds a compat shim    | D-16 + the S2-10.3 greps: no `serde(alias)`, no `#[deprecated]`, no shim    |
| goldens churn in dozens of files at once  | one re-freeze per task/phase; never batch unrelated fixtures                |
| agents implement invented behaviour       | hard rule: no fixture → not done (README §5.7, `AGENTS.md`)                 |

## 9. Reading order for an agent

1. `STATUS.md` → [`TASKS.md`](TASKS.md) (find your id)
2. the decision(s) your task names in [`DECISIONS.md`](DECISIONS.md)
3. the current `spec/SYNTAX.md` §N you are about to change
4. the owning `crates/*/README.md` + its goldens
5. only then code. Deleted means deleted (D-16) — never leave a shim behind.
