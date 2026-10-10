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
- **IR glue** — `scan_extras_chars` / `emit_attrs` / `warning_event`: the shared
  code that binds a parsed head to the event IR.
- **Decorator binder** — `parse_decorator_line` / `bind_decorators`
  (`src/decorator.rs`): a standalone extras line binds to the _next_ block node
  through a caller-supplied `target` callback. See
  [`docs/decisions/0002-decorator-binder.md`](../../docs/decisions/0002-decorator-binder.md).
- **Typed values** — `AttrValue { Str, Int, Float, Bool, Raw }` and
  `classify_scalar` (§6.3). Numbers parse as numbers (a component receives `12`,
  not `"12"`) but render back to their authored literal text so goldens stay
  stable (`6.0` stays `6.0`).
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
