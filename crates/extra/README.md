# pendon-extra

Shared parsing primitives for the typed-extras surface syntax
([`docs/spec/SYNTAX.md`](../../docs/spec/SYNTAX.md) §4–§6). This crate owns the
**common** syntax so plugins never re-implement it; each plugin stays responsible
for interpreting the parsed properties.

> **Layer rule (`CONTRIBUTORS.md`):** plugin-specific meaning does **not** belong
> here. `extra` may parse `key: value`, but anchor owns the meaning of `target`,
> cite owns citation identity, and vicado owns typed property conversion.

## What it provides

- **Head parsing** — `parse_extras` (the `{…}` / `@@type{…}` head),
  `parse_type_marker` (the `@@type` run), `parse_directive_head`, and
  `parse_extras_body`. A malformed head returns `ExtrasMatch::Malformed` and the
  caller MUST render the original text verbatim (literal fallback, §4.3).
- **Slot grammar** — `parse_slots` (`src/slot.rs`) reads a `[…]` / `(…)` head
  before Markdown sees it: the tag sigil (`@` `!` `#` before the bracket, `^`
  right after it, head position only), the bracket, and the paren **only when it
  is attached**. Each slot yields `SlotParts { bare, quote, tick }` in the fixed
  bare → quote → tick order, and the bracket may carry a `|…` modifier run
  (`SlotPair::modifiers`, first-appearance order, deduped). `SlotMatch` mirrors
  `ExtrasMatch`: `Absent` (literal, no diagnostic), `Slots { pair, rest }`,
  `Malformed { error, rest }` (`Severity::Error` + literal, D-2). A plugin
  supplies only a `SlotContract { accepted, tag }`; no plugin may split a bracket
  itself (§5.2–§5.4, ADR-0012, ADR-0017). `Slot::name` / `SlotKind::name` and
  their `from_name` inverses are the `field.<slot>.<kind>` config vocabulary, so
  a typo can never be read as a position.
- **Slot modifiers** — `ModifierConfig` / `merge` / `contributions`
  (`src/modifier.rs`): one entry's `bracket.modifier."<symbol>"` tables, turned
  into the **public fields** a parsed run contributes (§5.4, D-9,
  [ADR-0017](../../docs/decisions/0017-slot-modifiers.md)). Same path → array in
  written order, object + object → deep merge, and an irreconcilable pair is a
  `ModifierError` that drops the whole run. Pendon ships no modifier, so a symbol
  with no config answers an empty list — legal, never a diagnostic.
- **IR glue** — `scan_extras_chars` / `emit_attrs` / `warning_event` (`src/emit.rs`):
  the shared **emit** pass that turns a parsed head into event attributes. It is
  not data binding — that is `plugin-bind` (§12.2, ADR-0005).
- **Decorator binder** — `parse_decorator_line` / `classify_block` /
  `bind_decorators` / `decorator_diagnostics` (`src/decorator.rs`): a standalone
  `@@`-prefixed extras line binds to the _next_ block node, and **which** plugin
  binds it is the claim table of D-10 rule 2, not the caller's choice:
  `Receiver::ORDER` is section → table → list → blockquote → paragraph,
  `classify_block` names a block's receiver from its `NodeKind` alone, and each
  pass passes a `Claim { receiver, enabled }` — `Claim::only` leaves other
  owners' blocks untouched, `Claim::sweeping` (the last binder) also drops what
  no enabled receiver will claim. Every drop is reported through the one
  `decorator_diagnostics` helper, so the four binders cannot drift. The `@@`
  sigil is mandatory (D-10, ADR-0013): a bare `{…}` line is literal text, and the
  head reads in one order (`@@type` → slot → `{specs}` → `%%numbering`). See
  [`docs/decisions/0002-decorator-binder.md`](../../docs/decisions/0002-decorator-binder.md)
  and §9.1 for the table.
- **Typed values** — `AttrValue { Str, Int, Float, Bool, Raw, Object, Array, JsxExpr, Null }` and
  `classify_scalar` (§6.3, ADR-0005). Numbers parse as numbers (a component
  receives `12`, not `"12"`) but render back to their authored literal text so
  goldens stay stable (`6.0` stays `6.0`). A nested `{…}`/`[…]` value is parsed
  with a depth-aware, quote-aware item splitter (inner commas never split the
  head) and renders as compact JSON unless `plugin-bind` transports it as a real
  JS value. `Null` is the one variant a document cannot spell: it exists so
  `AttrValue::from_json` is total over the AST (`plugin-bind` can resolve a
  `$var` to `null`), which is what makes the projection lossless.
- **Spread transport** — a `...$ref` item inside a nested `{…}` value (§19) is
  collected under `SPREAD_KEY` (`"..."`, `src/value.rs`) as a JSON array of the
  reference strings, in source order. `...` cannot be spelled as a `key`, so the
  marker is unreachable from a document; the merge itself belongs to
  `plugin-bind`, which is the only consumer. It travels as a key because a JSON
  object in this workspace has no item order to preserve.
- **Public field names and the field map** (`src/field.rs`) —
  `validate_field_name` enforces the proposal's name rules (`A-Z a-z 0-9 _ -`,
  a segment starts with a letter or `_`, case-sensitive, `.` nests, no empty
  segment); `Path` is a validated dotted name whose prefix tests are
  **segment-wise** and inclusive (`priced` is not under `price`); `FieldMap` is
  the ordered `path → AttrValue` map the projection fills, and `to_value()`
  nests it (`price.details.amount` → `price` → `details` → `amount`) in
  insertion order. A path cannot be a scalar _and_ have children, so both
  structural clashes are errors (`FieldMapError`). `FieldConfig` is the config
  side of the same vocabulary — one `custom` entry's `field.<slot>.<kind>` /
  `field.specs.<k>` / `field.override.<path>` keys, with `""` stored as `None`
  (unassign, OBJ-10) and `is_empty_slots()` as the D-7 preset-voiding trigger
  (a `field.specs.*` or `field.override.*` key does not void a preset — the
  preset maps slot positions, `S2-4.2`). The
  field names never reach the IR — see
  [`docs/decisions/0014-field-model.md`](../../docs/decisions/0014-field-model.md).
- **The projection** (`project(node, Option<&FieldConfig>) -> FieldMap`,
  `src/field.rs`) — the **one** function that turns an AST node into public
  fields, and the only place `field.override.<path>` is read (ADR-0014 rules
  1–4). Phase 2 is its identity pass: a node's attributes _are_ its public
  fields, so the map mirrors them key for key, in node order, with the same
  values — which is why rerouting the renderers through it changed no byte.
  What it decides: a `__`-prefixed key is internal and never enters the map
  (§11 rule 3); every other key is a `Path`, so a key that breaks the name rules
  cannot be a public field (dropped, with a `debug_assert!` that fails the suite
  rather than leaking a key); an override renames its field **as it is written**,
  keeping the replaced field's position. `FieldMap::attrs()` is the inverse the
  renderers emit from — `(dotted name, JSON value)` pairs, flat, in insertion
  order; `to_value()` nests the same entries for the JSON/AST view).
- **Preset assignment** (`src/preset.rs`) — each element that contracts a slot
  head declares **three** static pieces of data in its own crate: its
  `SlotContract`, a `preset: &[Preset]` table naming the field each position
  lands in (`@[Inner Text](href "title")` → `inner_text`, `href`, `title`), and
  the `owned: &[FieldName]` list of fields the element writes itself (D-12 rule
  5, `with_owned` / `owns`). `ElementHead` is the triple, with
  `preset_for(slot, kind)` (the lookup),
  `is_consistent()` (every preset sits on a contracted position) and
  `fallback_path(slot, kind)` (D-7's bare name for an unassigned value:
  `bracket.quote`, `paren.tick`, …). The names are `FieldName`s — a
  `const fn`-validated, single-segment built-in name — so a typo in a preset
  table **fails to compile** rather than at load. Voiding a preset is the
  entry's job: one `field.<slot>.<kind>` key in the matched `custom` entry
  disables the whole table for that entry (D-7, `FieldConfig::is_empty_slots`).
  `owns_field(&owned, key)`
  is the same rule for an element with no contract (`plugin-wiki`'s link); the
  collision itself is reported by the element as `ExtrasIssue::owned_key`.
- **Specs** — `Specs` / `SpecsResolver`: _where_ each of the head's contents
  lands — the slot groups `bracket` / `paren` and the five specs positions
  [`SpecsKey`]: `tick` / `quote` (a construct's slug and title) and the universal
  `class` / `id` / `style`. [`Specs::resolve`] is the only read side of the
  entry's `field.*` naming (`field.bracket.bare` / `field.paren.bare` /
  `field.specs.<k>` — §6.1, D-5); an entry that configures nothing keeps the
  built-in default paths (`specs.tick` / `specs.quote` / `specs.class` /
  `specs.id` / `specs.style` and `bracket.bare` / `paren.bare`).
  [`Specs::path`] says where a position is _published_; [`Specs::element_path`]
  is the only view an element
  may use — it answers the default path while the position is unassigned and
  `None` once `field.specs.<k>` moved it, because the assigned path is the
  author's own public field (§6.1). The type's doc carries the
  **`default path` / `meaning`** table, because only a default path has semantics.
- **The DOM projection** (`Specs::dom_fields(items, element)` +
  `universal_field`, `src/typed.rs`) — the §6.3 boundary: the three universal
  positions (`specs.class` / `specs.id` / `specs.style`) are written as the DOM
  fields they name, the element's own assignment wins when it renames one (a
  cite's `#id` → `cite-id`), and everything else keeps its dotted path
  (`specs.tick`, `bracket.bare`). Both halves match the **path**, not the
  position, so a `field.specs.<k>` assignment that moved a position (`foo.class`,
  `bar.id`) is left alone — it is the author's field, with no DOM meaning and no
  rename (§6.1). `dom_fields` returns borrowed `(field, value)` pairs, and
  `project` applies the universal half on its own, so a renderer that emits
  whatever the map holds is on the §6.3 side of the contract.

## The retired form

The pre-§11 `[.class,#id]{key: value}` block is **removed** (§14, ADR-0001 D3):
it is plain literal text and this crate has no parser for it.

## Tests

`crates/extra/tests/extras_spec.rs` pins every rule of §4–§6 (one case per rule,
including the §5.1 worked example). It is the primary gate for the foundation and
is never `#[ignore]`d.

```bash
cargo test -p pendon-extra
```

## License

MIT
