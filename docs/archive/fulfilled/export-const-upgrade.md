# Export Const Upgrade

**Status: landed (2026-10-25).** One ordered `DataExport` list in
`renderer-solid`, `plugin-cite` and `[[task.bind.inject]]` as producers. Spec:
[`spec/SYNTAX.md` §20](../spec/SYNTAX.md); decision:
[`ADR-0009`](../decisions/0009-generic-data-exports.md); keys:
[`crates/plugin-bind/README.md`](../../crates/plugin-bind/README.md).

## Background and Goals

- I suspect that `export const data` is currently implemented in a crude way inside renderer-markdown or related crates. This rework therefore aims to make `export const data` more dynamic and elegant.
- This rework or upgrade is intended for the implementation of `plugin-extract`, which uses `export const data` as one of its main channels. See the details in the [Plugin Extract Proposal](./plugin-extract.md).

## Scope

This upgrade is only for `renderer-solid`.

`export` only takes effect when the task `format` is `solid`. For any other format, it is ignored and discarded.

## To Do

- [x] Inspect how `export const data` is currently implemented.
- [x] Make it more flexible, dynamic, and elegant (`DataExport`, one writer).
- [x] Which crate or plugin owns `export const` is to be decided by the
      development team after inspecting the source code directly: **`renderer-solid`
      owns emission**, producers only register `(name, Value)` — ADR-0009.

## plugin-bind Export

We want `plugin-bind` to get a feature like this:

```toml
[[task]]
input = "./src/[foo]/[bar].mdp"
output = "./out/[bar].jsx"
plugin = "bind,markdown"
format = "solid"

[[task.bind.inject]]
source = "./src/[foo]/[bar].yaml" # uses the dynamic dir mechanism that already exists in Pendon
export = "fooData"

[[task.bind.inject]]
source = "../../dataX.toml"
export = "dataX" # injects dataX into all jsx outputs
```

This is rendered as:

```jsx
export const fooData = ...
export const dataX = ...
```

as a JS object or array, depending on the data from the source.

The `source` path is resolved against the project root (the same base `input` and
`output` use) and may carry the task's `[foo]`/`[bar]` captures; without `export`,
the file stem names the binding. The conversion mechanism from the source data to
JS already exists in `plugin-bind` (`parse_data_file`), so both forms share it.
