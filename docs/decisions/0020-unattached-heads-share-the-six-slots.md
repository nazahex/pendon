# ADR-0020 — The `[…]` / `(…)` groups of an untagged head are the same six slots

- Status: **Accepted** — completes [ADR-0003](0003-positional-groups.md) and
  [ADR-0012](0012-universal-slots.md) for the heads this crate does not own
  (`::type[…]`, `==type[…]`, `{{type}}[…]`, `@@type[…]`)
- Depends on: [`0012`](0012-universal-slots.md) (slot grammar),
  [`0014`](0014-field-model.md) (public fields),
  [`0017`](0017-slot-modifiers.md) (a bracket's `|…` run)
- Spec: [`SYNTAX.md`](../spec/SYNTAX.md) §4.4, §5.2, §6.1, §6.4, §10.1, §10.2
- Evidence: `crates/extra/src/slot.rs`
  (`attaches_six_positions_with_no_tag`,
  `a_quoted_paren_is_a_quote_not_a_bare_content`,
  `the_bracket_is_optional_and_so_is_the_paren`,
  `an_attached_bracket_carries_its_modifier_run`,
  `an_unclosed_group_is_unclosed_not_absent`); `apps/cli/tests/extras_heads_spec.rs`
  (`slot_groups_reach_the_untagged_heads`); goldens `10-decorator-blocks`,
  `14-marker`, `15-directive-inline`, `36-table-slots`, `39-decorator-order`,
  and the new `49-head-slots-generic`

## Context

§5.2 says a slot holds up to three contents — one bare, one quoted, one ticked —
in that order, and §6.1 says every content lands in its own public field. That is
how the four tagged heads work: `@`, `!`, `#` and `^` read their pair with
`parse_slots` and publish `bracket.bare` … `paren.tick`.

The **untagged** heads did not. `::`/`==` directives, `{{…}}` markers and `@@…`
extras heads read their groups with a private helper
(`typed::read_positional_groups`) that stored a bracket as one verbatim string
and a paren as one trimmed, unquoted string. Three consequences, all wrong under
the spec:

1. `@@type[bare "q"`t`]{…}` published the whole `bare "q"`t`` as
   `bracket.bare`, so five of the six positions were unreachable.
2. `("A title")` was unquoted into `paren.bare` — but §5.2 decides the kind by the
   **first character**, so it is a quote. Two documents that differed only in
   quoting published the same field.
3. `field.bracket.quote` / `field.paren.tick` (and the other four) were inert on
   these heads, and so was the bracket's `|…` run, because the helper never
   produced a `SlotPair` for the run to attach to.

## Decision

1. **One grammar, one reader.** `slot::parse_attached_slots(rest) -> SlotMatch`
   reads the groups of a head the caller already owns: no tag sigil, the bracket
   optional (`::note("T")` is a paren-only pair, §6.1), `[[…]]` still
   `plugin-wiki`'s opener, contents by §5.2 (`find_close` / `scan_content`), and
   the bracket's run collected into `SlotPair::modifiers`. It is the only reader
   of those groups: `typed::read_positional_groups` is deleted.
2. **A difference in the head, never in the contents.** An unattached group keeps
   `parse_slots`' rules for what a slot _holds_; the two readers differ only in
   what a head _is_. `parse_slots` reports `Absent` for a bracket that never
   closes (the text is nobody's), while `parse_attached_slots` reports
   `SlotError::Unclosed` (the head was already committed to), which the extras
   reader wraps as `ExtrasError::InvalidSlot` and keeps literal (§4.3).
3. **`ExtrasHead.slots: SlotPair`**; `to_attributes` publishes all six positions
   in §6.4 head order (`bracket.bare` → `bracket.quote` … `paren.tick`), through
   `Specs::slot(slot, kind)` — the same six-position table a tagged head uses, so
   `field.bracket.quote = "locator"` now renames a directive's or marker's quote
   exactly as it renames a citation's. `DirectiveHead.slots` is the same field.
4. **The run is wired too.** `to_attributes_with_modifiers` and
   `apply_modifier_run` are the §5.4 entry points for these heads. Contributions
   apply **last** (D-9: configuration after authored text), `class` accumulates
   (§6.4) and any other field the head already wrote is dropped with a warning. A
   directive merges its own pair and its adjacent extras head's first, so the run
   lands after both, and `class` from an extras `.base` and a run's `class`
   becomes `base run` (fixture 49).
5. **One lookup for both halves.** `HeadConfig { specs, modifier }` and
   `HeadResolver` replace `SpecsResolver`, and `ComponentSet::head_for` builds the
   pair from **one** matched entry, so the positions and the run can never answer
   with different entries (§11 rule 3, D-7).

## Rejected alternatives

- **Keep two readers, widen the helper.** `read_positional_groups` would have to
  grow the `|…` run, the escape rules, the kind order and the duplicate errors —
  i.e. a second copy of `scan_content`. Two parsers of one grammar is exactly the
  layering violation `CONTRIBUTORS.md` forbids (and the reason `%` normalization
  lives in one place, ADR-0010).
- **Keep `("x")` → `paren.bare` for compatibility.** §5.2 has no such rule, and
  the two spellings would then publish different fields for the same content.
  Under ADR-0018 there is no compatibility layer to preserve: an entry that wants
  a directive's quoted paren in `level` writes `field.paren.quote = "level"`
  (fixture 15 re-frozen that way).
- **Leave the run inert on these heads.** A bracket is a bracket: if `@[x |?](y)`
  reads its run, `::type[x |?](y)` reading the same text differently would be a
  second dialect. The wiring is what made `HeadConfig` necessary, and it is
  smaller than the alternative (a run that silently does nothing).
- **Box the head inside `ExtrasMatch` / `DirectiveMatch`** to silence clippy's
  `large_enum_variant`: the match is a short-lived local, and a pointer between
  every caller and the head it just parsed buys nothing. The lint is allowed on
  the two enums with that rationale.

## Consequences

- Six positions per group are published everywhere: `10-decorator-blocks`,
  `14-marker`, `36-table-slots` and `39-decorator-order` re-froze `paren.bare` →
  `paren.quote`, and `15-directive-inline` re-froze because its entry now renames
  the paren's quote.
- `ExtrasError` gains `InvalidSlot(SlotError)`; `SlotError` gains `Unclosed(Slot)`.
- `Specs.bracket` / `Specs.paren` (one `String` each) become a private
  `slots: [[String; 3]; 2]` with `slot(slot, kind)` / `set_slot`, so a position
  cannot be addressed without its kind.
- `plugin-table`'s in-table layer heads (`extras_to_layer`) still resolve the
  default specs with no modifier table: the parser that reads them has no
  `CustomNode` in scope. That is a pre-existing gap (an in-table `@@type` head
  never resolved the entry's `field.*` either) and is tracked in `STATUS.md`.
