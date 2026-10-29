# ADR-0013 — `@@` is the only decorator, and it comes first

- Status: **Accepted** — D-10 settled by the owner 2026-10-28; all four pieces
  landed 2026-10-29: rule 1 (mandatory `@@`) in `S2-5.1`, the claim table in
  `S2-5.2`, the new receivers in `S2-5.3`, the table head in `S2-5.4`
- Depends on: [`0002`](0002-decorator-binder.md) (the binder),
  [`0003`](0003-positional-groups.md) (positional groups)
- Spec: [`SYNTAX.md`](../spec/SYNTAX.md) §5.1 (head order), §6 (decorators), §14
  (the retired inline form)
- Design:
  [`workflows/unified-syntax-stage-2/DECISIONS.md`](../workflows/unified-syntax-stage-2/DECISIONS.md)
  D-10
- Evidence: `crates/extra/src/decorator.rs` claim-order tests; one fixture per
  receiver plus the three rejection cases

## Context

`@@` used to be one optional way to give a block extras, accepted wherever the
binder happened to look, and each element decided for itself whether to look.
Stage 2 makes `@@` **the** decorator: mandatory, first in the head order, with a
closed receiver set (blockquote, codefence, list, paragraph, table), and section
may claim a decorator that precedes a heading.

## Decision

1. **Lexing.** A line is a decorator line iff it starts with `@@` within the
   existing indent tolerance and has nothing after the extras except whitespace.
   Anything else is literal text.
2. **Claiming order lives in `crates/extra`, not in the plugins**, first match
   wins, one decorator per receiver: `section` (before a heading, no warning) →
   `table` (table head line) → `list` (first item) → `blockquote` (`>` line) →
   `paragraph` (any block).
3. **Rejection is a warning, and the line always disappears.** Literal text after
   the decorator → `Warning`, text dropped. A receiver that does not accept
   decorators (heading when section is off, marker, directive, frontmatter, data
   block) → `Warning`, decorator dropped. Accepted or not, the decorator is
   removed from the stream so the `@@` can never leak into rendered output.
4. The old inline `@@type{…}` head form for tables is **removed** (§14): it is
   the same text as the decorator line, so there is nothing to migrate — a table
   simply no longer eats a decorator that sits on its own head line.

## Rejected

- **Leaving an unclaimed decorator visible as `@@…`.** It reads as a typo in the
  output; the binder already removes what it consumes.
- **Warning when `section` claims a heading's decorator.** Section claiming is the
  intended behaviour, not a mistake.
- **Per-plugin claiming order.** Five copies of a precedence table drift apart;
  the order is a grammar fact, so it lives with the grammar.

## Consequences

`bind_decorators` grows an explicit claim table, `plugin-section` gets a
pre-claim hook instead of a special case inside the scan, and the inline form's
removal appears in §14 as a removal-only row (D-16).
