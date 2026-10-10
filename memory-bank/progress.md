# Progress

## What works (verified green)

- Workspace builds clean: `cargo build --workspace`.
- `cargo test --workspace` green.
- Golden fixtures `01–12`, `14–16`, `21`, `22` frozen and green; none `#[ignore]`d.
  Gates: `cargo test -p pendon --test syntax_spec` (17 fixtures) and
  `cargo test -p pendon --test ultimate_freeze_spec`.
- `cargo fmt --check` clean for touched files; `bun run check` / `format` for JS/docs.

## What is left to build

- **L2/L3 list item layer** (blocked by L37 bug): item wrapper in `plugin-list`,
  `CustomPlacement::ListItem` in `plugin-markdown`, L4 `start:` warning, L5 nesting
  test, golden `13-list-item`.
- **L37 truncation fix** in `crates/extra` + `plugin-anchor`/`plugin-heading`; then
  re-render, review the newly visible tail, re-freeze `sandbox/ultimate`, add a
  regression case.
- **Phase-4 cleanup**: OPEN-R1 `:::` deprecated alias (warn → hard-error), archive
  `docs/rfc/unified-syntax.md`, rewrite `sandbox/unified`, trim
  `docs/todo/syntaxes.md`.
- **§11 threading gaps**: `plugin-table` layer options, `plugin-wiki` `ComponentSet`,
  `extract-heading` options.

## Known issues

- `sandbox/ultimate` frozen baseline encodes a **truncated** body (L37 bug): the
  rendered `ultimate.jsx` stops after the L37 list item; L40–L46, D, B, N and the
  whole E section never render, though `headings` still lists them.
- `plugin-markdown`: fenced code inside a blockquote (Q14) unsupported;
  `parse_blockquote_prefix` over-consumes indentation; a blank `>` splits one list
  into two.

## Evolution of decisions (pointers)

- D1–D10 rationale → `docs/decisions/0001-typed-extras-reconciliation.md`.
- Progress notes that used to live in `SYNTAX.md` §17 → moved to `STATUS.md` /
  `archive/`; the spec now stays normative only.
- Completed-phase essays from the old `TODO.md` → `docs/archive/`.
