# ADR-0012 — Slots are universal and parsed before Markdown

- Status: **Accepted** — D-1 / D-11 / D-12 settled by the owner 2026-10-28;
  implementation lands in Phase 1 (`S2-1.1`–`S2-1.6`) and Phase 3
  (`S2-3.2`, `S2-3.3`)
- Depends on: [`0003`](0003-positional-groups.md) (positional keys),
  [`0001`](0001-typed-extras-reconciliation.md) (typed extras)
- Spec: [`SYNTAX.md`](../spec/SYNTAX.md) §5.2 (slots), §5.3 (tags), §6.1 (slot
  contracts), §6.5 (the `specs.*` projection)
- Design:
  [`workflows/unified-syntax-stage-2/DECISIONS.md`](../workflows/unified-syntax-stage-2/DECISIONS.md)
  D-1, D-11, D-12
- Evidence: `crates/extra/src/slot.rs` unit tests; the golden named by each task
  in `TASKS.md` as it lands

## Context

Bracket syntax was split four ways: `plugin-anchor` owned `@[x](y)`,
`plugin-cite` owned `[^^ref]`, `plugin-image` owned `![alt](src)`, and
`plugin-heading` owned `#[slug](title)`, while `crates/extra` knew only the
_names_ of the positional keys (`PositionalKeys`). Stage 2 makes `[…]` and `(…)`
universal, so keeping the split would have meant a fifth hand-written bracket
splitter with its own escaping rules.

## Decision

**One slot parser, in `crates/extra/src/slot.rs`, called before the Markdown
parser sees the line** (the pre-Markdown point of §12.1).

1. A slot holds up to three _parts_ in fixed order — `Bare`, `Quote`, `Tick` —
   each of `[…]` and `(…)`, and a slot's bracket may carry a modifier run
   ([`0017`](0017-slot-modifiers.md)).
2. **Tags** (`@` anchor, `^` cite, `!` image, `#` heading) are matched only in
   head position — start of the remaining line, after indent. `foo#[bar]` is
   literal. `#` is a heading slot only when immediately followed by `[`, so
   `# Word` stays an ATX heading.
3. **Tags are hard-coded, owners are not required.** A tag whose owner plugin is
   disabled is literal text with **no** diagnostic (the user did not opt into the
   syntax). If the owner is enabled but the element does not accept slots, that
   is `Severity::Error` with literal fallback.
4. `^` is available because `crates/core` has **no** footnote extension, so
   `[^ref]` is never claimed before us (verified: `OBJ-1`, 2026-10-28).
5. **`specs.*` duplication rules** live here because they are the same head:
   `tick`/`quote`/`id` may appear once (second = `Severity::Error`, first kept);
   `class` accumulates and dedupes keeping first position; `style` is
   last-wins per property; `k: v` claims the bare field `k` and a `k` that
   collides with a built-in field is an error (the user must use
   `field.override.<path>`).

A plugin supplies only its **slot contract** — which slots it accepts, in what
order, and which tag claims an untagged standalone slot. No plugin may
re-implement bracket splitting.

## Rejected

- **A `slot` plugin.** Parsing must happen before Markdown, and intra-plugin
  ordering would become a race.
- **Post-parse AST rewriting.** Markdown has already destroyed the nesting
  (links, images, emphasis) by then.
- **Owner-disabled tags becoming errors.** It punishes a user for turning a
  plugin off.

## Consequences

`plugin-heading`'s `#[` and `plugin-cite`'s `[^^` shrink to a contract plus a tag
check, and "one head parser per element" (`CONTRIBUTORS.md`) survives one layer
down. `crates/extra` gains `slot.rs`, and §5.2–§5.3 + §6.1 become the normative
statement of everything above.
