# pendon-plugin-bind

Binds data declared in `{{{lang[var] … }}}` blocks into extras values as real
JavaScript props. Grammar: [`docs/spec/SYNTAX.md` §19](../../docs/spec/SYNTAX.md).
Rationale: [`docs/archive/fulfilled/plugin-bind.md`](../../docs/archive/fulfilled/plugin-bind.md) and
[ADR-0004](../../docs/decisions/0004-pre-parse-bind-stage.md).

```md
{{{json[card]
{ "id": "usr_01", "tags": [ "admin" ] }
}}}

[Card](/docs)@@anchorA{data: $card}
```

```jsx
<a data={{"id":"usr_01","tags":["admin"]}} href="/docs" type="anchorA">Card</a>
```

## Two stages, no middle

This is the only plugin with a **pre-parse** stage, because a payload's lines look
like markup (`#` → heading, `-` → list, a blank line → a block separator):

- `extract(source) -> Extract` runs **before** `pendon_core::parse`. It removes
  every block, replaces it with one blank line, parses the payload and returns a
  `Registry` of `var -> serde_json::Value`. `extract_with_base(source, dir)` is
  the form to use for a document read from a file: a `(path)` payload resolves
  against **that directory**, so `../` works at any depth and never means the
  process CWD.
- `resolve(events, &registry) -> Vec<Event>` runs **after** every other plugin,
  once the extras heads have emitted their literal `$var` attribute values.

The CLI enables both whenever `bind` appears in `task.plugin` (or `--plugin`);
`bind` does nothing inside the plugin loop.

## What it does

- `json`, `jsonc`, `yaml`/`yml`, `toml`, `csv` payloads → one normalized
  `serde_json::Value`. A payload may also come from a **file**
  (`{{{csv[rows](../data/rows.csv)}}}`); every file read is listed in
  `Extract.external_files`, and the CLI adds those to the task's cache
  dependencies so editing one re-renders the page.
- `mdp` (Pendon Markdown) payloads → the body is Pendon Markdown, and it is a
  **sub-document**: it is parsed and run through the **same chain of plugins as
  the page**, so wiki links, citations, figures, tables, math, extras heads and
  custom components all render inside it. The plugin does not know those plugins:
  the caller hands it a renderer (`resolve_with(events, &registry, &mut render)`),
  which is what lets a bound payload share the host's citation numbering and
  renderer hints. Its tree travels as a **fragment** — a node list under
  `renderer_ast::FRAGMENT_KEY` (`{"$fragment": […]}`) — and every renderer owns
  its own format: Solid emits `name={(<>…</>)}`, HTML renders the markup into the
  attribute value, and the AST/JSON renderers keep the structured node list.
  Block form (body on the line after the head) keeps the payload's block
  wrappers; inline form (body on the head line) drops the payload's own paragraph.
  A `(path)` file payload is **not** legal for `mdp` (§2.7 decision 10) — the
  head-line body keeps its meaning. `resolve(events, &registry)` is the
  Markdown-only wrapper (payloads rendered by `plugin-markdown` alone).
- A `$var` that is the **entire** non-`mdp` extras value is replaced with
  `JSON_ATTR_PREFIX` + compact JSON; `renderer-ast` re-hydrates it, so a renderer
  sees a real value (`name={…}` in Solid, the same structured value in the AST).
- A **path** reaches one leaf: `$config.db.host`, `$rows[0].nama`,
  `$m["a-b"]`. A miss warns and stays literal.
- A nested extras value (`{rows: $rows, n: 2}`) is walked and each string leaf
  that is exactly a reference is replaced. A `...$var` **spread** in such a value
  (`{...$brand, scale: 1.2}`) merges a bound object in as defaults — an authored
  key wins over the spread. The spread travels under `pendon_extra::SPREAD_KEY`
  (`"..."`), a key no authored extras key can spell; its values are copied, never
  walked as references.
- Diagnostics: an invalid/unclosed/unparseable block is dropped with a
  `Severity::Warning`; a duplicate `var` warns (last wins); an undefined `$var` or
  an unresolvable path warns and stays literal; a `(path)` block with no source
  directory (stdin) is a `Severity::Error`.

## Options

`[[task.bind.inject]]` publishes a whole **file** as a module-level export, so a
Solid page can `import { fooData }` instead of naming the data in the Markdown:

```toml
[[task]]
input = "./src/[foo]/[bar].mdp"
output = "./out/[bar].jsx"
plugin = "bind,markdown"
format = "solid"

[[task.bind.inject]]
source = "./src/[foo]/[bar].yaml" # project-root-relative, same captures as `input`
export = "fooData"

[[task.bind.inject]]
source = "./data/settings.toml" # no `export`: the file stem names the binding
```

```jsx
export const fooData = [{"name":"Alpha"}];
export const settings = {"count":2};
export default function PendonView() { … }
```

- `source` is resolved against the **project root** (the directory `pendon run`
  walks), never the document's directory, and may use any capture of the task's
  `input` glob (`[foo]`, `[bar]`, `[...rest]`) — so one task can publish a
  different file per page.
- `export` is optional; without it the file's stem is used (`settings.toml` →
  `settings`). It must be a JavaScript identifier and not a reserved word; a bad
  name is a task error before anything renders (§20 of SYNTAX).
- The payload is parsed by the block converter (`json`/`jsonc`/`yaml`/`yml`/
  `toml`/`csv`; anything else is an error), so the value has the same shape a
  `{{{lang[var] … }}}` block would produce.
- Emission, ordering, duplicate names and the "Solid only" rule belong to
  `renderer-solid` (SYNTAX §20, ADR-0009); this plugin only registers `(name,
  value)`.
- The injected file is a cache **dependency**, so editing it re-renders the page.
- The section applies only when `bind` is in `task.plugin`, like every other
  config-driven plugin. Exports exist only for `format = "solid"`; other formats
  drop them silently.

## Decided against

`\$` escape (unnecessary — a reference starts with a letter, so `$100` is text),
frontmatter/micromatter as a payload source, free-text interpolation,
reactive/runtime binding, and type-checking a bound value against a component's
props. Rationale: RFC §2.8.

`var` must start with a letter (`OPEN-BIND-1`): `is_valid_var` enforces it, so
`price: $100` is plain text and raises nothing. Quoting is not an escape — a
quoted value is unquoted before the reference check.
