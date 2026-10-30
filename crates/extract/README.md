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

# S2-8.3 — paths, not names: `include` a subtree, then cut one leaf out of it
[[task.anchor.extract]]
output = "./out/[slug].anchor.json"
format = "json"
preset = "pendon"
include = ["price.details"]
omit = ["price.details.tax"]
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

Every emitted object is one occurrence of the element: its fields, plus
`inner_text` and `depth`. Selection is one **preset** plus three modifiers,
applied in this order: **`preset` (the starting set) → `include` (adds paths) →
`only` (keeps nothing else) → `omit` (always wins)**.

- Every entry is a **dotted field path** (`price.details.amount`, `soo.baz`),
  validated at load like a `field.*` name: `price..amount` or `1x` is a config
  error naming the entry. A path whose **prefix** names it selects the whole
  subtree, so `only = ["soo"]` keeps `soo.baz` and `omit = ["soo"]` takes it
  away. `soo.baz` is one path of two segments whether the name came dotted from
  an attribute or nested from a structured value; the row nests it either way
  (§21.4).
- `preset = "pendon"` is the element's own field list — §12 of
  [`FIELDS-INVENTORY.md`](../../docs/spec/FIELDS-INVENTORY.md), the same table as
  [`pendon_preset`]. `preset = "all"` is every field the occurrence carries, and
  it is what an **absent** `preset` means today (S2-8.6 flips the default to
  `"pendon"`). Any other name is a config error listing both values.
- `include` adds paths to the preset instead of replacing it; on `"all"` it is
  redundant but valid. `only` keeps nothing but its own paths — it is a filter,
  not an addition — so `only` **and** `include` in one block is a config error:
  it would be one command with two readings. `include` with `omit` is fine: the
  omitted paths simply never ship.
- A path the occurrence does not carry is dropped, never emitted empty, even when
  `only` names it, and an unmatched `only` yields an **empty row** (`[{}]`, PE.4)
  rather than a fall back to the preset. A path that lost every child leaves no
  `{}` husk behind. `__`-prefixed internal keys are never extractable.
- A row is built from the **projection** (`pendon_extra::project`, ADR-0014) —
  the same public field map the renderers read — so `field.override.*` renames a
  field before it is extracted (`reference_id` → `ref_id`), a `specs.*` position
  lands under its DOM field, and a row can never spell a field differently from
  the element the page renders (S2-8.4). The CLI passes the `field.*` config in
  (`FieldConfigs`, resolved per layer and `type` marker with
  `ComponentSet::field_for`), because only it knows what the task configured.

`recommended = <bool>` was the pre-D-6 spelling of this (`true` = `"pendon"`,
`false` = `"all"`). It is **gone**; a config that still has it fails to load with
an error naming `preset = "pendon"` (D-16 — no compatibility shims).

## Frontmatter (`[[task.frontmatter.extract]]`)

The document's own frontmatter is extractable as **one record per document**
(D-14, §21 rule 11). Its fields are the top-level keys — a nested key is a path,
so `only = ["title", "meta.tags"]` keeps those two and drops the rest of `meta`
— and it has **no preset**: `preset`, `include` and `type` are load errors there,
which is also why the block's starting set is `all` by construction and the
S2-8.6 default flip cannot empty it.

```toml
[[task.frontmatter.extract]]
output = "./out/[slug].fm.yaml"
format = "yaml"
only = ["title", "author.meta.tags"]

[[task.frontmatter.extract]]
export = "meta"
```

A JSON/TOML file holds one row (`[ { … } ]`, `[[frontmatter]]`), an `export`
holds a one-element array. A document with no frontmatter, or an empty one,
writes no file and registers no binding. `export = "frontmatter"` is an error:
§20 already publishes that const from the same record.

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

## Layout

| file        | owns                                                                   |
| ----------- | ---------------------------------------------------------------------- |
| `lib.rs`    | the pass: one row per occurrence, one rule per destination (D-2, D-20) |
| `scan.rs`   | reading occurrences out of the event stream, and their field tree      |
| `block.rs`  | one `[[task.<element>.extract]]`: selectors, destinations, `Path`s     |
| `select.rs` | the field algebra — `preset` / `include` / `only` / `omit` over paths  |
| `fields.rs` | the frozen vocabulary: `ELEMENTS` and the §12 `PRESETS` table          |
| `format.rs` | JSON / YAML / TOML rendering, detection and appending (§21.6)          |

## License

MIT
