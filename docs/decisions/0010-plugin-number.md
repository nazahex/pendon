# ADR-0010 — `plugin-number`: one numbering plugin for every element

- Status: **Accepted** (landed in Phase 3 of
  [`workflows/numbering-extract-type-export.md`](../workflows/numbering-extract-type-export.md))
- Depends on: [`0005`](0005-structured-extras-values.md) (structured attribute
  values), [`0008`](0008-type-selectors.md) (selector-scoped blocks)
- Spec: [`SYNTAX.md`](../spec/SYNTAX.md) §5 (`%%` item), §10.5 (`%%` markers and
  the `number` config), §15 open decision D-10
- Evidence: goldens `30-number-heading`, `31-number-custom`; `crates/number` unit
  tests
- **Amended 2026-10-30 — the crate moved and the field split ([0016](0016-number-promotion.md)).**
  `crates/plugin-number` is now `crates/number` (`pendon-number`), and the single
  `attrs.number` array is published as the three public fields `number.raw` /
  `number.depth` / `number.type` (`SYNTAX.md` §10.5, `FIELDS-INVENTORY.md` §0).
  Nothing else above changed: `__number` is still the marker's internal transport,
  one pass still numbers every element, and a `number` block is still a counter.

## Context

Numbering is per-element and hard-coded: `plugin-heading` counts headings with
its own `counters` and style formatting, `plugin-cite` numbers citations, and
anchors and images have no numbering at all. Every new "number this element"
request either edits a plugin that should not know about numbering or copies
`plugin-heading`'s counter code.

`%%` is the author-facing notation proposed for "give this element a number
sequence", and it is an **extras item**, so its grammar belongs in `crates/extra`
(shared `@@`-head parsing), never in the plugin.

## Decision

1. **`plugin-number` owns numbering.** Config is
   `[[task.<element>.number]]`, repeatable and order-sensitive, scoped by
   `type = …` selectors resolved with ADR-0008's `matching`.
2. **The grammar of `%%` lives in `crates/extra`**; the plugin only maps it to
   a group/style and a counter.
3. **`__number` is the internal transport** (`D-9`): a double-underscore
   attribute is stripped by every renderer, exactly like `__plugin_kind`. The
   value is a structured array carried through ADR-0005's `U+E001` transport,
   so `attrs.number` is a real JS value in Solid.
4. **`depth` is parse-time normalized** — `%`-count − 1, computed once by
   `crates/extra`. The plugin never re-derives it.
5. **The heading extras prop `number:` becomes reserved** (`D-10`): while
   `plugin-number` runs, an authored `number:` item is a diagnostic instead of a
   competing attribute.
6. **Number systems** ship tier 1: `decimal`, `lower/upper-roman`,
   `lower/upper-alpha` (+ `latin` aliases), `lower-greek`; the rest stay in the
   enum for a follow-up (`D-8`).
7. `plugin-cite` keeps its own citation numbering (out of scope) — extraction
   must be able to read `attrs.number` from headings, anchors and images.
8. **Marker types match case-insensitively.** The marker is authored CamelCase
   (`%%numberB`) while a `[[task.number.custom]]` selector is written however the
   config author likes; both sides of that one comparison are lowercased.
   Element selectors keep ADR-0008's case-sensitive literals, so a `type = …`
   over `HeadingA` still means exactly `HeadingA`.
9. **A marker's depth is absolute**, a grouped element's depth is relative to the
   group minimum. `%%%` is depth 2 on any element (a missing parent counts as
   the style's first value); `[[task.heading.number]]` reads `style[n−1]` for the
   group's _n_-th smallest occurring depth, because the document decides which
   depth is deepest.

## Rejected alternatives

- **Extend `plugin-heading`.** Numbering is not heading-specific; the config
  would still be `[[task.heading.number]]`, which cannot number an anchor.
- **Number in the renderer.** The counter state is document-global and must be
  identical for HTML/JSON/AST; a renderer-local counter would diverge.
- **Write the number into the text.** §6 and ADR-0005: plugins write attributes
  only; a consumer decides how to display `1.2.3`.
- **Re-derive `depth` from the marker string in the plugin.** Two parsers of
  `%%` is exactly the layering violation `CONTRIBUTORS.md` forbids.

## Consequences

- `plugin-heading`'s counter/style code moves out (its `auto_number` /
  `number_style` options become sugar over `[[task.heading.number]]`).
- New crates: `crates/plugin-number`; new grammar in `crates/extra`.
- `EXTRAS-INVENTORY.md` gains a `number` row per in-scope element, which Phase 4
  then extracts unchanged.
