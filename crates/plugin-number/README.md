# pendon-plugin-number

One numbering pass for **every** element (ADR-0010). The grammar of a marker
lives in `crates/extra` (§5 + §10.5 of [`SYNTAX.md`](../../docs/spec/SYNTAX.md));
this crate only maps a marker or an element to a counter and a style, and writes
the result as the `number` **attribute**.

```text
plugin = "micromatter,heading,anchor,image,number,markdown"
```

`number` must run **after** every element plugin that produces the nodes it
numbers and **before** `markdown`; the CLI rejects any other position with a
config error naming both plugins.

## What it writes

`attrs.number` is an array of exactly `depth` formatted strings — never text, and
never a display format:

```jsx
<Heading level={2} number={["1", "2", "a"]}>…</Heading>
```

A client decides how to show `1.2.a`. The value travels as ADR-0005's structured
transport (`U+E001` + compact JSON), so Solid hands the component a real array.
Nothing is written to the inner text, and an element whose number is only used
for an `id`/`class` never has to display it.

## Configuration

Two kinds of block, both repeatable and order-sensitive (a block **is** a
counter):

```toml
# 1. an element's own numbering, scoped by §11 `type` selectors (ADR-0008)
[[task.heading.number]]
type = ["HeadingA", "HeadingB"]
style = ["decimal", "decimal", "lower-alpha"]

# 2. a task-level counter named by a `%%numberType` marker
[[task.number.custom]]
type = ["NumberA"]
style = "upper-alpha"
```

- `type` is a selector list: `*` / a literal / `_` (the remainder) / `?` (the
  untyped nodes), resolved with `pendon-types` (`Group`, ADR-0008). No `type` key
  means "every element of this element".
- `style` is a number system (below). A **string** applies to every depth; an
  **array** maps entry _n−1_ to the _n_-th visible depth. A depth the array does
  not reach is `decimal`.
- Two blocks that match the same element are decided by selector specificity,
  then config order — the same rule as `[[task.<layer>.custom]]`.

Tier-1 systems (D-8): `decimal`, `lower-roman`, `upper-roman`, `lower-alpha`,
`upper-alpha`, `lower-latin`, `upper-latin`, `lower-greek`. Anything else (or a
value a system cannot represent, e.g. roman past 3999) falls back to `decimal`.

## Depth

- **Element numbering**: the element's own depth — a heading's `depth` (§7.4), 1
  for everything else that has no natural depth.
- **Marker numbering**: `%`-count − 1, computed **once** by `crates/extra`
  (`%%` → 1 … `%%%%%%%` → 6) and carried in `__number`; this crate never counts
  `%`. A marker **always** beats the element's own numbering, and its depth is
  absolute (a `%%%` marker is depth 2 even on a `##` heading), so a missing
  parent counts as the style's first value (`I`, `A`, `1`).

The **smallest depth is decided across the whole document** for an element block:
a `##` at the end of the document renumbers the `###` headings before it. That is
why the pass runs twice — collect members and the group minimum, then assign —
and why `style[0]` is the group's smallest occurring depth, not necessarily `1`.
A **marker** group instead counts from `1` always (see below).

## Interaction with authored `number:` (D-10)

While this plugin runs, an authored `number:` extras item is **reserved**: it is
replaced with a diagnostic instead of competing with the computed attribute. To
give an element a fixed number, turn the plugin off for that element (drop its
`number` block) — the authored value then passes through like any other prop
(golden `06-heading`).

## Layering

- The `%%numberType` grammar (marker shape, `depth` normalization, literal-text
  cases) is `crates/extra`'s (§5, D-9); this plugin reads `__number` and never
  re-parses text.
- `__number` itself never reaches output: every renderer strips `__`-prefixed
  keys, and `plugin-image`'s HTML fallback passes it through unprefixed instead
  of turning it into `data-__number`.
- A plain markdown image only carries a marker when the head ends its line
  (`![alt](/a.png)%%numberA`); §7.1 requires the extras head to own the rest of
  the line, so `%%` with trailing prose stays literal text.

## Scope

Numbering of **citations** stays in `plugin-cite` (ADR-0010 rule 7); this plugin
never touches them. Ordering, `start` and list markers (§9.4) are unrelated.

## License

MIT
