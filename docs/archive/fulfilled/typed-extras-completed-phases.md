# Archived: Typed Extras Reconciliation — completed phases

> **Historical / frozen.** This is the completed-work record moved out of the old
> root `TODO.md` when docs were split by concern. It is kept for provenance only.
> Live work is [`../../STATUS.md`](../../STATUS.md); rationale is
> [`../../decisions/`](../../decisions/). Do not update this file.

The old `TODO.md` tracked D1–D10 as a phased plan (Phases A–I, then Phase M for the
list/blockquote decorator work). All of the phases below are **complete**; the
rationale now lives in ADR-0001 and the grammar in
[`../../spec/SYNTAX.md`](../../spec/SYNTAX.md).

## Completed phases (summary)

- **Phase A** — `crates/extra`: one unified head scanner (`AttrValue`, `ExtrasHead`,
  `parse_type_marker`, `parse_extras`, `to_attributes`, `Attrs::merge_with`).
  `crates/extra/tests/extras_spec.rs` is the primary gate.
- **Phase B** — construct plugins: strict adjacency, no legacy forms.
- **Phase C** — cite grammar (`[^^](ref "loc")` only; retired forms literal).
- **Phase D** — directive fallback: unclaimed type renders `<span>`/`<div>` (D8).
- **Phase E** — docs: `SYNTAX.md` §2/§3/§4/§5/§7/§8/§10/§13/§14/§16/§17 written;
  plugin READMEs re-swept for legacy `[.class]`/spaced-head examples, every example
  output re-generated with the real CLI.
- **Phase F** — fixtures and sandboxes: `golden/02` and `05` rewritten, `21` added,
  `09` re-frozen; all `sandbox/*/src/*.md` swept and rebuilt, then diffed against
  `HEAD` output to prove no attribute was lost.
- **Phase G** — config/plugin hygiene: unknown plugin name and unknown `task.*` key
  warn once and are ignored.
- **Phase H** — baseline: `sandbox/ultimate/out.frozen/` re-frozen;
  `ultimate_freeze_spec` gate added (fails if a frozen file is touched).
- **Phase I** — gates: workspace tests, build, fmt, dprint, `git diff` review.
- **Phase M (list/blockquote + decorator binder)** — `crates/extra` binder,
  `plugin-blockquote`, `plugin-list` (L1 container end-to-end), `plugin-markdown`
  placement via `__plugin_kind`, CLI wiring, goldens `10–12`. See ADR-0002.
- **§11 positional keys + empty-attr omission** — `PositionalKeys`/`ExtrasOptions`,
  `KeyResolver`, wired through blockquote/section/list/anchor/heading/marker/img/
  cite/directive; empty string omitted except `alt`.
- **Decorator & marker positional groups** — `@@type[…](…){…}` (Option 2). See
  ADR-0003.

## Known-bug caveat recorded at freeze time

The `sandbox/ultimate` frozen baseline encodes a **truncated** body because of the
L37 head-scan bug (see [`../../STATUS.md`](../../STATUS.md) "Open blockers"). This
was accepted deliberately so the freeze gate could land; the fix re-renders and
re-freezes the tail.
