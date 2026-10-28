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
- **IR glue** — `scan_extras_chars` / `emit_attrs` / `warning_event` (`src/emit.rs`):
  the shared **emit** pass that turns a parsed head into event attributes. It is
  not data binding — that is `plugin-bind` (§12.2, ADR-0005).
- **Decorator binder** — `parse_decorator_line` / `bind_decorators`
  (`src/decorator.rs`): a standalone extras line binds to the _next_ block node
  through a caller-supplied `target` callback. See
  [`docs/decisions/0002-decorator-binder.md`](../../docs/decisions/0002-decorator-binder.md).
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
  (unassign, OBJ-10) and `is_empty()` as the D-7 preset-voiding trigger. The
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
  head declares **two** static pieces of data in its own crate: its
  `SlotContract` and a `preset: &[Preset]` table naming the field each position
  lands in (`@[Inner Text](href "title")` → `inner_text`, `href`, `title`).
  `ElementHead` is just the pair, with `preset_for(slot, kind)` (the lookup),
  `is_consistent()` (every preset sits on a contracted position) and
  `fallback_path(slot, kind)` (D-7's bare name for an unassigned value:
  `bracket.quote`, `paren.tick`, …). The names are `FieldName`s — a
  `const fn`-validated, single-segment built-in name — so a typo in a preset
  table **fails to compile** rather than at load. Voiding a preset is the
  entry's job: one `field.*` key in the matched `custom` entry disables the whole
  table for that entry (D-7, `FieldConfig::is_empty`).
- **Positional names** — `PositionalNames` / `NamesResolver`: what each of the
  head's four positional contents is called (`bracket`, `paren`, `tick`,
  `quote`). [`PositionalNames::resolve`] is the only read side of the entry's
  `field.*` naming (`field.bracket.bare` / `field.paren.bare` /
  `field.specs.tick` / `field.specs.quote` — §6.1, D-5); an entry that configures
  nothing keeps the built-in `slug` / `title`.

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
