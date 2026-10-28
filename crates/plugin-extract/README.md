# pendon-plugin-extract

One extraction pass for **every** extractable element (P.1 of
[`docs/issues/plugin-extract.md`](../../docs/issues/plugin-extract.md)). The
fields an element can carry are frozen in
[`docs/spec/EXTRAS-INVENTORY.md`](../../docs/spec/EXTRAS-INVENTORY.md); the
`type = …` selector vocabulary is §11 of
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
```

- `output` is a path (or a list) **relative to the project root**, with the same
  `[segment]` substitution as a task `output`. One path is one flat array of
  rows; two blocks — or two pages — resolving to one path replace its content
  instead of merging (P.1).
- `format` is mandatory and one per `output`: `json`, `yaml` or `toml`. The
  format wins over the extension, so `format = "json"` may write `out/x.yaml`.
  TOML has no top-level array, so its rows become `[[<element>]]` tables.
- `export` registers `export const <name> = [ … ]` through the Solid renderer's
  data exports (ADR-0009) and is a **no-op for every other format**; a name that
  is not a JavaScript identifier is a config error (§20).
- `type` is a §11 selector list; an absent `type` is `_` — the types of that
  element no block names, i.e. P.1.b's "safety net". `*` opts into the named
  types too.
- A block that matches nothing writes nothing: no empty file, no empty binding.

## Field selection

Every emitted object is one occurrence of the element: its attributes in
emission order, plus `inner_text` and `depth`. Selection is four steps, in this
order: **all fields → `recommended` (the element's preset) → `only`
(intersection) → `omit` (always wins)**. A field the occurrence does not carry
is dropped, never emitted empty, even when `only` names it. `__`-prefixed
internal keys are never extractable.

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
