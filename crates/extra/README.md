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
- **IR glue** — `scan_extras_chars` / `emit_attrs` / `warning_event` (`src/emit.rs`):
  the shared **emit** pass that turns a parsed head into event attributes. It is
  not data binding — that is `plugin-bind` (§12.2, ADR-0005).
- **Decorator binder** — `parse_decorator_line` / `bind_decorators`
  (`src/decorator.rs`): a standalone extras line binds to the _next_ block node
  through a caller-supplied `target` callback. See
  [`docs/decisions/0002-decorator-binder.md`](../../docs/decisions/0002-decorator-binder.md).
- **Typed values** — `AttrValue { Str, Int, Float, Bool, Raw, Object, Array }` and
  `classify_scalar` (§6.3, ADR-0005). Numbers parse as numbers (a component
  receives `12`, not `"12"`) but render back to their authored literal text so
  goldens stay stable (`6.0` stays `6.0`). A nested `{…}`/`[…]` value is parsed
  with a depth-aware, quote-aware item splitter (inner commas never split the
  head) and renders as compact JSON unless `plugin-bind` transports it as a real
  JS value.
- **Spread transport** — a `...$ref` item inside a nested `{…}` value (§19) is
  collected under `SPREAD_KEY` (`"..."`, `src/value.rs`) as a JSON array of the
  reference strings, in source order. `...` cannot be spelled as a `key`, so the
  marker is unreachable from a document; the merge itself belongs to
  `plugin-bind`, which is the only consumer. It travels as a key because a JSON
  object in this workspace has no item order to preserve.
- **Positional keys** — `ExtrasOptions` / `PositionalKeys` / `KeyResolver` for
  the §6.1 per-component `backtick_key` / `quote_key` (and, for directive heads,
  `bracket_key` / `parentheses_key`).

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
