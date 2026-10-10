# pendon-plugin-bind

Binds data declared in `{{{lang[var] … }}}` blocks into extras values as real
JavaScript props. Grammar: [`docs/spec/SYNTAX.md` §19](../../docs/spec/SYNTAX.md).
Rationale: [`docs/rfc/plugin-bind.md`](../../docs/rfc/plugin-bind.md) and
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
- A `$var` that is the **entire** extras value is replaced with
  `JSON_ATTR_PREFIX` + compact JSON; `renderer-ast` re-hydrates it and
  `renderer-solid` emits `name={…}`.
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

None. `[task.bind]` is reserved; the plugin needs no configuration.

## Not implemented yet

- `mdp` (Pendon Markdown as a value): recognised, warns, drops the block. Phase 2
  needs the inline-pipeline entry point and the `U+E002` policy pinned — and the
  question whether `mdp` gets a `(path)` payload, which for now it does not (its
  head-line body keeps its meaning).

## Decided against

`\$` escape (unnecessary — a reference starts with a letter, so `$100` is text),
frontmatter/micromatter as a payload source, free-text interpolation,
reactive/runtime binding, and type-checking a bound value against a component's
props. Rationale: RFC §2.8.

`var` must start with a letter (`OPEN-BIND-1`): `is_valid_var` enforces it, so
`price: $100` is plain text and raises nothing. Quoting is not an escape — a
quoted value is unquoted before the reference check.
