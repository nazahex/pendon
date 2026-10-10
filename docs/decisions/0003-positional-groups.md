# ADR-0003 — Positional groups `@@type[…](…){…}` (Option 2)

- Status: **Accepted / done**
- Spec: [`docs/spec/SYNTAX.md`](../spec/SYNTAX.md) §6.1, §10.1

## Context

Add `[…]` / `(…)` positional slots to heads and markers without breaking the
existing rule that a bare `[x](y)` line is a link.

## Decision — Option 2

The `[…]` / `(…)` positional groups sit **between the type and the body**, so a
decorator line still starts with `@@` and a plain `[x](y)` paragraph line can never
be mistaken for one (omitting `@@` ⇒ link, by design). Markers take their groups
directly after `}}`:

```text
@@type[…](…){extras}      @@[…](…){extras}     untyped decorator
{{type}}[…](…){extras}    {{type}}[…](…)@@head{extras}
```

- Groups parse in the **shared `read_head`** (`crates/extra/src/typed.rs`), so every
  `@@` head accepts them (uniform grammar; a repo grep found zero existing
  `@@type[` / `{{type}}[` patterns — no meaning changes).
- Marker order: groups right after `}}`, **before** any extras head. They map
  through `keys_for(marker type)` → `bracket_key` / `parentheses_key` (§6.1).
- **Key collision:** the positional group wins over a same-key `{…}` item; the
  dropped value reports an §13 `Overridden` warning (§6.2 head-wins style).
- **Malformed group** (`[` or `(` without its closer) ⇒ `UnterminatedHead` ⇒ the
  head/line stays literal text (§4.3), matching directive behaviour.
- **Emission order** (§6.4): `class`, `id`, then the positional keys — `bracket_key`,
  `parentheses_key`, `backtick_key`, `quote_key` — then remaining props/flags in
  source order.
- **Layer order** (now §6.1): the groups belong to the layer that runs first. Under
  the canonical order (`anchor` before `marker`) a marker keeps no attribute from an
  adjacent `[x]("y")` pair (the anchor claims it); swapping the entries hands the
  marker both groups. Single-group and type-only forms parse in either order.

## Evidence

`crates/extra/tests/extras_spec.rs` (`heads_carry_positional_groups`,
`groups_must_be_adjacent`, `groups_map_through_the_positional_keys`),
`decorator.rs` (`positional_groups_make_a_decorator_line`,
`a_group_decorator_binds_its_attributes`), `plugin-marker`
(`groups_follow_the_marker_type`,
`marker_groups_win_and_malformed_groups_stay_literal`); golden fixtures 10 and 14;
`sandbox/ultimate` M13–M19 and Q25/Q26.
