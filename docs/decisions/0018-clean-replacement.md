# ADR-0018 — Clean total replacement: no compatibility of any kind

- Status: **Accepted** — owner policy, stated 2026-10-28 (D-16 settled); applies
  to **every** task from Phase 1 on, and is enforced by `S2-10.3`
- Depends on: nothing — it constrains every other ADR in this folder
- Spec: [`SYNTAX.md`](../spec/SYNTAX.md) §13 (load errors), §14 (removal-only
  record), §4.3 (literal fallback)
- Design:
  [`workflows/unified-syntax-stage-2/DECISIONS.md`](../workflows/unified-syntax-stage-2/DECISIONS.md)
  D-16
- Evidence: `S2-10.3`'s greps (`serde(alias)`, `#[deprecated]`, `TODO: remove`,
  compat/deprecation/migration wording) and a fixture asserting that an old config
  aborts with one `Severity::Error`

## Context

Pendon is alpha and Stage 2 replaces its markup and configuration wholesale. The
temptation in a change of that size is a compatibility layer: a serde alias so old
keys keep loading, a `#[deprecated]` warning, a `MIGRATION.md`, a "write this
instead" column in §14. Each one is a standing promise that the old form works
somewhere — and each doubles the fixture surface and lets the two grammars drift.

## Decision

One rule, applied without exception:

1. **Deleted means deleted.** Every removed syntax, config key, field name and
   helper is deleted in the task that removes it: no `#[serde(alias)]`, no
   `#[deprecated]`, no dual-parse branch, no `#[allow(dead_code)]` stub, no
   re-exported shim, no `TODO: remove in 0.1`. If a golden still needs the code,
   the golden is wrong and changes in the same task.
2. **Configuration fails fast.** Unknown keys, removed keys and removed
   `[task.<el>.*]` tables are `Severity::Error` at load (previously §13
   warnings). The message names the offending key and the load aborts before any
   render. Naming the current key is allowed — the user has to unblock — but it is
   a diagnostic, not a documented old→new mapping.
3. **Removed syntax is not grammar.** It is literal text under the existing §4.3
   rule, with **no** diagnostic: the same treatment a typo gets. §14 still records
   _that_ a form is no longer syntax (a normative statement that keeps the spec
   honest) with no "write this instead" column, no dates, no versions, no prose
   guidance.
4. **No migration document.** No `MIGRATION.md`, no upgrade guide, no old→new
   compatibility table for Stage-2 changes anywhere in `docs/`. `STATUS.md`
   records work, not transitions.
5. **One version.** `0.0.x`, no semver ceremony. The new API _is_ the API.

## Rejected

- **A grace release with warning shims.** Explicitly ruled out by the owner, and
  it doubles the fixture surface.
- **A migration guide.** It is a standing promise that the old names still work
  somewhere, which they must not.
- **Keeping the old parser behind a flag.** Two grammars, two fixture sets,
  permanent drift.

## Consequences

The task list carries the policy: `S2-2.8` (unknown/removed keys are load
errors), `S2-0.6` (delete the migration surface), `S2-10.3` (clean-break audit).
`AGENTS.md` hard rule 6 states it for agents, so a "helpful" shim is a review
finding rather than a judgement call.
