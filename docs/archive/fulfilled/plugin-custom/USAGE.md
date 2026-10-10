# USAGE.md

Quick usage and examples for the `plugin-custom` TOML plugin format.

## CLI: load a TOML plugin

The CLI will accept TOML plugin entries via the `--plugin` flag using the `toml:` prefix.

Examples:

```bash
# single toml plugin
pendon --plugin "toml:plugins/alert.toml" --format solid --input ./doc.md

# multiple plugins (comma-separated)
pendon --plugin "toml:plugins/frontmatter.toml,markdown" --format ast --input ./doc.md
```

## Development workflow

1. Create `plugins/your-plugin.toml` following `SPEC.md`.
2. Run `pendon --plugin "toml:plugins/your-plugin.toml" --format events --input example.md` to verify event transformations.
3. Iterate until emitted events/AST match expectations.

## Debugging and dry-run

Implementers should add a `--plugin-dryrun` CLI switch to validate and print the loaded `PluginSpec` and any validation diagnostics without mutating events. The `plugin-toml` crate should export a `validate_spec(&PluginSpec) -> Vec<Diagnostic>` helper.

Example plugin: `plugins/alert.toml`

```toml
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
```

## Notes for renderer authors

- `renderer.solid.imports` should be injected at the top of the TSX output only once per document. The renderer will deduplicate identical import lines.
- `component_template` tokens must be escaped or serialized appropriately by the renderer. Template expansion happens on the renderer side to reuse string escaping logic.

## Import formats and examples

`renderer.solid.imports` accepts either raw import lines (strings) or structured entries (recommended). The renderer will deduplicate and merge imports for the same module.

String form example:

```toml
[renderer.solid]
imports = ["import { createSignal } from 'solid-js'", "import Alert from './Alert'"]
```

Structured form example (recommended):

```toml
[[renderer.solid.imports]]
module = "solid-js"
names = ["createSignal", "createEffect"]

[[renderer.solid.imports]]
module = "./Alert"
default = "Alert"
```

Deduplication/merge rules:

- Multiple structured entries for the same `module` will be merged into a single import line with combined named imports.
- String import lines are deduplicated by literal equality; structured entries are normalized then deduplicated after merging.

## Testing

- Add unit tests in `crates/plugin-toml/tests/` to validate TOML parsing and example transformations.
- Provide an end-to-end example in `sandbox/examples/toml-plugin.md` showing `pendon --plugin ... --format solid` producing a valid TSX file that compiles in a Solid project.

## Project discovery example: `pendon.toml` + `plugins/index.toml`

Prefer a static index manifest (`plugins/index.toml`) that enumerates `[[plugin]]` entries in authority order. `pendon.toml` references the index file and `pendon run` orchestrates the listed plugins.

Example `pendon.toml` referencing an index:

```toml
[plugin-custom]
source = ["./plugins/index.toml"]
enable_unsafe_hooks = false
```

Example `plugins/index.toml` (ordered):

```toml
[[plugin]]
id = "alert"
path = "alert.toml"
enabled = true

[[plugin]]
id = "tabs"
inline = { name = "tabs", kind = "block", matcher = { start = ":::tabs", end = ":::" } }
enabled = true
```

## CLI ergonomics

- `pendon run` reads `pendon.toml`, loads the referenced index, validates referenced plugin files, and makes the listed plugin ids available to tasks. The order plugins are applied for a specific file is controlled by the per-task `plugin` list.
- Tasks in `pendon.toml` may reference plugin ids from the index. Example: `plugin = "alert,markdown"` where `alert` resolves to the index entry and `markdown` is built-in.
- `pendon --plugin "toml:plugins/alert.toml"` still works for ad-hoc runs and takes precedence over discovered/indexed plugins for that invocation.
- `--plugin-dryrun` prints loaded specs, normalized imports, and validation diagnostics without mutating events or outputs.

## Demo example (matching `sandbox/custom/src/demo.md`)

This repository includes a small demo in `sandbox/custom/src/demo.md` and a component in `sandbox/custom/src/Foo.tsx`.

The example plugin is located in `custom/foo.toml` and referenced by `custom/index.toml`. It matches opening lines like:

```
:::foo["bar"] { qux: "fred", waldo: 8, isBar: true }
```

Run the demo as a Solid renderer output using the plugin (index referenced by `pendon.toml` in this repo):

```bash
pendon --format solid --input sandbox/custom/src/demo.md
```

Or run `pendon run` to execute tasks in `pendon.toml` which reference the `custom` source.
