# SPEC.md

TOML schema reference for `plugin-custom` (PluginSpec)

This document defines the canonical keys and types for the TOML plugin specification supported by `plugin-custom`.

## Top-level fields

- `name` (string, required): plugin name, used for CLI registration and diagnostics.
- `kind` (string, required): `"block"` or `"inline"` (or `"both"`).
- `matcher` (table, required): how to recognise the syntax.
- `attrs` (array of tables, optional): attribute declarations extracted from the opening marker.
- `ast` (table, optional): mapping to NodeKind and AST attribute names.
- `renderer` (table, optional): renderer hints and imports for supported renderers.
- `meta` (table, optional): free-form metadata (description, examples).

## Index-based discovery via `plugins/index.toml`

The preferred mechanism is an explicit index manifest (`plugins/index.toml`) that lists plugins in deterministic order. `pendon.toml` points to this manifest using `plugin-custom.source`.

`pendon.toml` example (references index):

```toml
[plugin-custom]
source = ["./plugins/index.toml"]
# enable advanced hooks explicitly
enable_unsafe_hooks = false
```

Index manifest rules:

- The index file contains an ordered list of `[[plugin]]` entries. The index author controls presence and optional default ordering.
- Each `[[plugin]]` entry can be:
  - `path` (string): relative path to a plugin TOML file (resolved relative to the index file), or
  - `inline` (table): an inline `PluginSpec` value embedded in the index.
- Each plugin must have an `id` field (stable identifier used by tasks and CLI). A plugin may also include `props` for human metadata (for example `props.name`). Duplicate `id` entries are rejected unless exactly one entry sets `override = true`.
- The engine validates each referenced file and reports diagnostics. By default `pendon run` fails on any validation error; CI can opt-in to `--continue-on-error`.

Ordering & resolution:

- The index provides the available plugin `id`s and an optional default order. However, the application order for a specific task is determined by the task's `plugin` list in `pendon.toml` (the order in the task's `plugin` string). If a task omits a plugin list, the engine falls back to the index order.
- When a task in `pendon.toml` lists plugin ids, the resolver expands those ids against loaded index entries and built-in plugin ids. Missing ids produce diagnostics.

Caching & performance

- Parse & normalize each referenced plugin only once per run. Cache normalized `PluginSpec` values keyed by content hash.
- Load multiple index files in parallel, then resolve and merge serially to produce a final ordered `Vec<PluginSpec>`.

## Matcher table

- `start` (string): literal start marker (e.g. ":::alert").
- `start_regex` (string): alternative PCRE-style regex for start matching (anchored at line start unless otherwise specified).
- `end` (string): literal end marker. If omitted, plugin can be single-line or inline.
- `inline_marker` (string): for inline syntaxes like `==` or `~`.
- `parse_hint` (string, optional): when specified the engine can execute specialized detection logic without emitting a new AST node. Common values include `blockquote-sigil`, `codefence-viewer`, and the new `html-inline`, which uses `start_regex` or `inline_marker` to know when an inline HTML fragment is present.
- `capture` (string, optional): how to capture attribute payloads. Values: `"space"` (attrs separated by spaces), `"keyvals"` (key=value pairs), `"rest"` (remaining raw string).

Attrs array (example)

[[attrs]]
name = "type"
type = "string"
required = true
default = "info"

Supported types: `string`, `int`, `bool`, `list<string>`

## AST mapping

- `node` (string): canonical NodeKind to emit (e.g., `"Component"`, `"CodeFence"`, `"Heading"`).
- `node_name` (string, optional): custom value for `attrs.name` when producing a component name.
- `attrs_map` (table): map spec attr names to emitted AST attr keys.

## Renderer hints

The `renderer` table provides renderer-specific guidance.

[renderer.solid]
imports = ["import { createSignal } from 'solid-js'", "import Alert from './Alert'"]
component_template = '''
<Alert type="{type}">{children}</Alert>
'''

Fields:

- `imports` (array<string>): source lines to inject at top of TSX output for `solid` renderer.
- `component_template` (string): template used by Solid renderer when mapping the AST node to JSX. Tokens: `{children}`, `{attrs.<name>}`, `{text}`.
- `raw_html` (bool): if true, the renderer may consider `text` as raw HTML fragment (use with caution).

## Import forms

`renderer.*.imports` accepts two flavors to make common import patterns ergonomic and safe:

- Simple string lines (legacy friendly):

```toml
imports = ["import { createSignal } from 'solid-js'", "import Alert from './Alert'"]
```

- Structured import entries (recommended): an array of tables that the engine will normalize and merge.

```toml
[[renderer.solid.imports]]
module = "solid-js"
names = ["createSignal"]

[[renderer.solid.imports]]
module = "./Alert"
default = "Alert"
```

Rules for structured imports:

- `module` (string, required): module specifier.
- `names` (array<string>, optional): named imports to include (e.g., `Foo`, `Bar`).
- `default` (string, optional): default import identifier.
- The renderer will deduplicate imports per document and merge named imports for the same module. For example, two entries importing `createSignal` and `createEffect` from `solid-js` will be merged into a single `import { createSignal, createEffect } from 'solid-js'` line.
- Structured imports allow the engine to canonicalize quoting style and prevent accidental duplicate side-effects from identical literal lines.

## Renderer fallback

If no `component_template` is provided, the engine will fall back to emitting an AST node (preserving events) and allow the downstream renderer to render as usual.

## Validation rules

- `name`, `kind`, and at least one matcher field must be present.
- `attrs` names must be valid identifiers (^[A-Za-z_][A-Za-z0-9_-]*$).
- `node` must be one of the engine's known NodeKinds. Unknown values are allowed but will be treated as `Custom` and emitted as-is.

Example minimal TOML

name = "alert"
kind = "block"

[matcher]
start = ":::alert"
end = ":::"
capture = "keyvals"

[[attrs]]
name = "type"
type = "string"
default = "info"

[ast]
node = "Component"
node_name = "Alert"
attrs_map = { type = "type" }

[renderer.solid]
imports = ["import Alert from './Alert'"]
component_template = "<Alert type=\"{attrs.type}\">{children}</Alert>"
