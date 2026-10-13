# ADR-0005 — Structured extras values and the `U+E001` transport

- Status: **Accepted / implemented**
- Depends on: [`0001-typed-extras-reconciliation.md`](0001-typed-extras-reconciliation.md)
  (the `AttrValue` model), [`0004-pre-parse-bind-stage.md`](0004-pre-parse-bind-stage.md)
  (who consumes this)
- Spec: [`SYNTAX.md`](../spec/SYNTAX.md) §5 (`value`), §6.3 (`AttrValue`), §19
- Evidence: `crates/extra/tests/extras_spec.rs` (`mod nested_values`), golden
  [`23-bind`](../spec/golden/README.md), `pendon-renderer-ast` / `-solid` tests

## Context

`plugin-bind` (ADR-0004) binds a declared payload into an **extras value** with
`$var`. The motivating case in
[`sandbox/bind/bind-main.sb.md`](../../sandbox/bind/bind-main.sb.md) is a nested
value:

```md
===directiveCSV("Tabel Karyawan"){ tabel: { karyawan: $tabel21-karyawan4, foo: { fooYi: $foo_yi } }, --color: "#ffa"}
```

(`sandbox/bind/bind-main.sb.md` line 155. `--color` is a **top-level** head item,
i.e. a cssvar; nested object keys are prop `key`s only — `is_key` rejects a
leading `-` or digit, so `--x` inside a `{…}` value is `InvalidItem` and falls
back to literal text.)

Three facts of the pre-existing design blocked it:

1. **The grammar had no structured value.** §5 said
   `value = dquote | squote | bquote | number | bool | bare` — a `{` or `[` in a
   value position was literal text.
2. **The splitter was depth-blind.** `extra::split_items` split the head body on
   every top-level comma, so the inner `karyawan: …, foo: …` commas would have
   ended the outer item rather than the nested one.
3. **The IR is stringly typed.** `Event::Attribute { value: String }`
   (`crates/core/src/event.rs`), and the AST builder stored every attribute as
   `Value::String`, so a bound object would have collapsed into a quoted string
   before `renderer-solid` could see it.

The render side already had the contract we wanted: the Solid spread path emits
an object/array as `name={…}`.

## Decision

**`crates/extra` owns structured extras values; `plugin-bind` owns their
transport to a real JS value.**

1. `AttrValue` gains two variants (`crates/extra/src/value.rs`):
   `Object(Vec<(String, AttrValue)>)` and `Array(Vec<AttrValue>)`. Entries keep
   source order; nested keys are last-wins, matching §6.4.
2. The head's item splitter (`extra/src/typed.rs::split_items`) is **depth-aware**
   and quote-aware: a `,` only splits an item at nesting depth 0, so
   `{a: {b: 1, c: 2}, d: 3}` is one item. `}`/`[` inside a quoted string are
   inert. Nesting is arbitrary (a value recurses through `parse_value`).
3. `AttrValue::literal()` renders `Object`/`Array` as **compact JSON** — the
   stringly-typed form the IR, interpolations (`{attrs.key}`) and the DOM
   fallback path consume. `AttrValue::to_json()` is the JSON projection.
4. The **transport** for a real JS value is `JSON_ATTR_PREFIX` (`U+E001`) +
   compact JSON on `Event::Attribute.value`. `renderer-ast` strips the prefix and
   re-hydrates a real `serde_json::Value`; `renderer-solid` then emits `name={…}`.
   **Only `plugin-bind` may emit `U+E001`** — `crates/extra` never does, so
   enabling the plugin is what turns a structured value into a JS value.
   Without it the value stays a quoted compact-JSON string (verified with the
   real CLI: `[a](/u)@@anchorA{a: {b: 1}}` → `a="&#123;&quot;b&quot;:1&#125;"`).
5. A malformed structured value (unbalanced group, trailing characters,
   unquoted whitespace) rejects the **whole head**, which then falls back to
   literal text per §4.3 — extras are never partially applied.

### Terminology: the _emit_ pass, not "bind"

The extras → `Event` glue module `crates/extra/src/bind.rs` is renamed to
**`crates/extra/src/emit.rs`**, and the post-markdown pass that resolves
sentinels and attaches attributes is called the **emit pass** (§12.2).

Reusing "bind" for two things was a live hazard: `plugin-bind` _binds data into
extras_ (pre-parse extraction, §19), while the shared pass merely _emits parsed
extras as events_. The overloaded word made a pre-parse text stage and a
post-parse event pass look like the same mechanism. Only `plugin-bind` and its
`$var` references may be called "binding".

## Rejected alternatives

- **Keep values scalar; stringify objects per plugin.** Every extras-consuming
  plugin (anchor, heading, img, wiki, cite, marker, directive, table, list)
  would need its own JSON parser, and `$var` could never bind a real prop.
- **A second, "JSON" flavour of extras head.** Duplicates the head grammar and
  the typed-extras model of ADR-0001 for no gain; nesting is orthogonal to
  typing.
- **Emit the `U+E001` sentinel from `crates/extra`.** Then _every_ structured
  value would become a JS expression in every renderer, changing output for
  documents that never opted into `plugin-bind` and for renderers whose contract
  is a string (HTML/JSON).
- **Teach `renderer-ast` to guess: parse any attribute that looks like JSON.**
  A legitimate string attribute `a="{b: 1}"` would silently become an object.

## Consequences

- §5's `value` production and §6.3's `AttrValue` mapping are extended; the spec
  previously contradicted the code.
- Structured values are available to **every** extras head, not just the bind
  example — a re-freeze touched `extras_spec` and added golden `23-bind`.
- A `...$ref` **spread** item inside a nested `{…}` value is transported under
  `SPREAD_KEY` (`"..."`) as a JSON array of reference strings; the merge is
  `plugin-bind`'s (RFC §2.6) — the reserved key is this crate's half of that
  protocol, and `...` can never be spelled as a `key`.
- The DOM fallback stays stringly typed by design; only Solid gets a JS value,
  and only for a `$var`-bearing attribute.
- `U+E001`/`U+E002` are reserved in `crates/core/src/event.rs`, distinct from the
  `U+E000` pre-markdown protect sentinel (§12.1).
