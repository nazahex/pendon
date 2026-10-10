# Active Context

## Current work focus

The docs restructure (this effort) plus **Phase 4** wiring of `plugin-list` L2/L3
(the `<li>` item layer). Container layers (`list`/`unordered`/`ordered`,
blockquote) have landed; the item layer is the next code task.

## Recent changes

- **Docs split (this pass):** the oversized `TODO.md` and `SYNTAX.md` were split
  by concern. `STATUS.md` is now the live plan of record; `docs/decisions/` holds
  the D1–D10 rationale as ADRs; `docs/spec/golden/README.md` documents the
  executable-truth mechanism; `docs/guides/` holds how-tos; `memory-bank/` is
  Cline's working memory. `SYNTAX.md` keeps only normative grammar + fixture ref.
- **Prior work (frozen/landed):** unified typed-extras `@@type{…}` heads (D1–D10),
  decorator binder, `plugin-section` rework, §11 positional keys + empty-attr
  omission, positional groups `@@type[…](…){…}`, `plugin-list`/`plugin-blockquote`
  container layers.

## Next steps

1. Fix the **L37 truncation** bug (shared head reader) — it blocks Step 3, Step 6b,
   golden `13`, and the D/B/N/E sections of `sandbox/ultimate`.
2. Land L2/L3 item layer + golden `13-list-item`.
3. Phase-4 cleanup: OPEN-R1 `:::` alias, archive `docs/rfc/unified-syntax.md`,
   rewrite `sandbox/unified`, trim `docs/todo/syntaxes.md`.

## Active decisions & considerations

- Docs **trust hierarchy**: executable fixtures > normative spec > ADRs > guides >
  STATUS. A fact lives in exactly one file; the rest link.
- `STATUS.md` must stay short — delete done lines; history is git + `archive/`.

## Learnings / insights

- Giant mixed-concern files caused AI agents to miss details; splitting by concern
  - a routing file (`AGENTS.md`) + an index (`docs/INDEX.md`) is the fix.
- The golden set is the strongest "always up to date" lever: a failing test forces
  a re-freeze, so docs can't silently drift from behaviour.
