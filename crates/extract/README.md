# pendon-extract

One extraction pass for **every** extractable element (P.1 of
[`docs/archive/fulfilled/plugin-extract.md`](../../docs/archive/fulfilled/plugin-extract.md)). The
fields an element can carry are frozen in
[`docs/spec/FIELDS-INVENTORY.md`](../../docs/spec/FIELDS-INVENTORY.md) and
**machine-checked**: `PRESETS` in `src/fields.rs` is §12 of that file, and
`apps/cli/tests/fields_inventory_spec.rs` parses the document to compare both —
a rename on either side fails the suite instead of silently drifting (S2-7.4).
The `type = …` selector vocabulary is §11 of
[`SYNTAX.md`](../../docs/spec/SYNTAX.md) (ADR-0008). This crate only turns
`[[task.<element>.extract]]` blocks into rows and hands them to a destination.

```text
plugin = "micromatter,heading,cite,number,extract,markdown"
```

`extract` **reads** the event stream and returns it untouched (P.2), so adding a
block never changes a rendered byte. It must therefore run **after** every plugin
whose attributes it reads — the element plugins, `number`, and `markdown`, which
is what turns the source lines into nodes in the first place. **Put it last**:
the CLI rejects an element plugin (or `number`) placed after it with a config
error naming both plugins (`validate_extract_order` in `apps/cli/src/chain.rs`),
but nothing stops `markdown` from running late, and a stream that is still
unstructured yields half-parsed rows.

## Configuration

A block names one destination or both, plus the field selection:

```toml
# P.1.a — a type-scoped file destination
[[task.cite.extract]]
type = ["CiteA", "CiteB"]
output = ["./out/[slug].cite.json", "../../project-2/[category].yaml"]
format = ["json", "yaml"]
only = ["reference_id", "citation_number"]

# P.1.b — an in-module binding, no file
[[task.cite.extract]]
export = "citationsC"
omit = ["locator"]

# P.1.c — both, and it overrides the earlier `citationsC`
[[task.cite.extract]]
output = "./out/[category]-cite-c.yaml"
format = "yaml"
export = "citationsC"
omit = ["locator"]

# D-6 — a preset plus one field the preset does not list
[[task.cite.extract]]
output = "./out/[category]-cite-preset.json"
format = "json"
preset = "pendon"
include = ["class"]
```

- `output` is a path (or a list) **relative to the project root**, with the same
  `[segment]` substitution as a task `output`. One path is one document of rows,
  and it **accumulates**: several blocks — or several pages — resolving to one
  path append to it instead of replacing it. Appending is idempotent (a row the
  file already holds is not written twice), so re-rendering a page never
  duplicates its data. JSON and YAML keep one array, TOML one array of tables per
  element (§21.6,
  [D-20](../../docs/workflows/unified-syntax-stage-2/DECISIONS.md)).
- `format` is mandatory and one per `output`: `json`, `yaml` or `toml`. The
  format wins over the extension, so `format = "json"` may write `out/x.yaml` —
  but it must also match the file that is already there: the format is detected
  from the content (JSON, then TOML, then YAML), and appending to a file written
  in another format is a task error naming both, never a mixed document. TOML has
  no top-level array, so its rows become `[[<element>]]` tables.
- `export` registers `export const <name> = [ … ]` through the Solid renderer's
  data exports (ADR-0009) and is a **no-op for every other format**; a name that
  is not a JavaScript identifier is a config error (§20).
- `type` is a §11 selector list; an absent `type` is `_` — the types of that
  element no block names, i.e. P.1.b's "safety net". `*` opts into the named
  types too.
- A block that matches nothing writes nothing: no empty file, no empty binding.

## Field selection

Every emitted object is one occurrence of the element: its attributes in
emission order, plus `inner_text` and `depth`. Selection is one **preset** plus
three modifiers, applied in this order: **`preset` (the starting set) →
`include` (adds names) → `only` (keeps nothing else) → `omit` (always wins)**.

- `preset = "pendon"` is the element's own field list — §12 of
  [`FIELDS-INVENTORY.md`](../../docs/spec/FIELDS-INVENTORY.md), the same table as
  [`pendon_preset`]. `preset = "all"` is every field the occurrence carries, and
  it is what an **absent** `preset` means today (S2-8.6 flips the default to
  `"pendon"`). Any other name is a config error listing both values.
- `include` adds names to the preset instead of replacing it; on `"all"` it is
  redundant but valid. `only` keeps nothing but its own names, so `only` **and**
  `include` in one block is a config error — it would be one command with two
  readings. `include` with `omit` is fine: the omitted names simply never ship.
- A field the occurrence does not carry is dropped, never emitted empty, even when
  `only` names it, and an unmatched `only` yields an empty row rather than a fall
  back to the preset. `__`-prefixed internal keys are never extractable.

`recommended = <bool>` was the pre-D-6 spelling of this (`true` = `"pendon"`,
`false` = `"all"`). It is **gone**; a config that still has it fails to load with
an error naming `preset = "pendon"` (D-16 — no compatibility shims).

## Scope

`apps/cli` rejects a block for the five in-scope-but-not-extractable elements
(`wiki`, `codeblock`, `latex`, `bind`, `micromatter`) with an error naming the
element — see §1 of the inventory for why each has no field yet.

`plugin-extract-heading` no longer exists (T4.5): the **nested** outline it used
to inject is now the automatic `headings` export of `renderer-solid`, built from
the `Heading` nodes of the AST (§20 of
[`SYNTAX.md`](../../docs/spec/SYNTAX.md)), so it needs no plugin at all. A
`[[task.heading.extract]]` block is the flat, configurable alternative — same
headings, one row each, with `depth` (golden `32-extract-heading`).

A `cite` block sees citations only when the task configures the citation layer
(`[task.cite.custom]`), because that is what turns a citation into a structured
node with attributes. Without it the citations render as inline `<sup>` markup
and carry nothing to read — their data is `plugin-cite`'s own
`citations`/`references` export.

## License

MIT
