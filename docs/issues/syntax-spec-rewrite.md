# Rewrite `SYNTAX.md` top to bottom

**Status: scheduled, not started.** This is a documentation change only — the
grammar must not move.

## Why this is an issue and not a task

Stage 2 rewrote the grammar, and
[D-15](../workflows/unified-syntax-stage-2/DECISIONS.md#d-15--where-the-new-grammar-lives-in-syntaxmd)
deliberately did **not** restructure the spec to match. Rewriting it in the same
change would have made every task's diff unreviewable and every cross-link wrong,
so the decision was: add subsections in place now, do the clean pass
**after Phase 10, as its own change**. Phase 10 is closed
(`S2-10.1`–`S2-10.5`), so this is due.

## What the current shape costs

`docs/spec/SYNTAX.md` is ~2070 lines over §1–§21, and the Stage-2 vocabulary is
now spread across the file instead of introduced once:

- **One concept, five places.** Slots are introduced in §2 (terminology),
  inventoried in §3 (sigils), defined in §5.2–§5.4 (slots, tags, modifier runs)
  and used in §6.1 (contracts) and §6.5–§6.6 (projection, presets). A reader
  arriving at §7 has to have read all five.
- **§5 reads backwards.** It is titled "Heads: extras and slots" but its prose
  and EBNF still open with the extras head and reach slots through a forward
  reference to §5.2.
- **§15 has become a changelog.** Three items are still open (OPEN-L1, OPEN-B1,
  OPEN-T1); the "Resolved decisions" list below them restates facts that already
  live in `docs/decisions/` and in §19/§21 — a second source of truth.
- **§17 "Build phases (historical)" and §18 "Acceptance criteria"** are process
  documents inside the normative grammar.
- **§13's diagnostics list and §14's removal table** are the two "what happens
  when it is wrong" sections, but they sit after eleven sections of grammar.

## What the rewrite would do

One linear narrative, in the order a parser meets the text:

1. Scope and terminology (§1–§2, one place per term).
2. Lexical layer: sigils, identifier/name rules, whitespace (§3–§4).
3. **The head**, once: extras items, slot parts, slot tags, modifier runs, the
   `%%` marker, and the order they are read in (§5 + §6.1 merged).
4. Fields: contracts → public projection → presets and voiding (§6).
5. Per-element grammar (§7–§10), each linking back to §3–§4 instead of restating.
6. Configuration that changes grammar: §11 (`custom`, `field.*`,
   `bracket.modifier`).
7. Order and stages: §12 (pipeline) and §21 (extraction).
8. Output contracts: §19 (data blocks) and §20 (module exports).
9. Diagnostics (§13) and removals (§14) as the closing reference.

Moves out of the spec: §15's open items become ADRs (or close), the resolved list
is deleted in favour of links to `docs/decisions/`, and §16–§18 (fixture set,
build phases, acceptance) are re-homed to `docs/spec/golden/README.md`,
`docs/archive/` and `CONTRIBUTORS.md`.

## Constraints

- **No grammar change.** Not one golden may move; `cargo test --workspace` is the
  proof (`docs/spec/golden/**` stays byte-identical).
- **No dangling links.** Every `§N` reference in `crates/*/README.md`,
  `apps/cli`, `editors/vscode`, `AGENTS.md` and `docs/INDEX.md` is updated in the
  same change; `grep -rn '§'` is the checklist, and section numbers that change
  must not be cited from outside the file afterwards.
- **One fact, one file.** After the rewrite, `SYNTAX.md` is the only normative
  statement of the grammar; ADRs stay the only place a _reason_ is recorded.
- Keep RFC-2119 wording (MUST / SHOULD / MAY) exactly as it is where the rule is
  unchanged, so diffs read as moves rather than edits.

## Exit

- The file reads front to back with no forward reference a reader cannot follow.
- `cargo test --workspace` green with **zero** re-frozen fixtures.
- `AGENTS.md`'s task → §N table and `docs/INDEX.md` point at the new numbers.
- `docs/spec/SYNTAX.md` no longer restates anything that has its own ADR.
