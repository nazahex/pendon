# plugin-custom

Technical proposal: `plugin-custom` — a TOML-driven runtime plugin format

Status: Draft

## Summary

`plugin-custom` is a runtime plugin bridge that lets end users declare custom Markdown-like syntax and rendering rules using a declarative TOML format. The goal is to give users "absolute freedom" to define block and inline syntaxes, map them to AST node kinds, and attach rendering hooks (including Solid auto-imports and custom renderer templates) without writing Rust code.

## Design goals

- Declarative: express syntax, attributes, AST mapping, and renderer hints in TOML.
- Runtime-loadable: the CLI and library can load a TOML spec and apply it to the event stream at runtime.
- Safe: validations and deterministic behaviour; no arbitrary code execution by default.
- Extensible: allow future custom renderer adapters (JS/TS/WASM) as opt-in.
- Low friction DX: examples, useful errors, and a small test harness.

## Non-goals

- Execute arbitrary user JS/TS on the host by default.
- Replace compiled Rust plugins for maximum performance; this is primarily for configurability and DX.

## High-level architecture

1. `PluginSpec` TOML → parsed into an immutable `PluginSpec` struct (serde + validation).
2. `plugin-toml` crate exposes `apply_spec(events: &[Event], spec: &PluginSpec) -> Vec<Event>`.
3. CLI support: `--plugin toml:path/to/foo.toml` or `--plugin-custom path/to/foo.toml`.
4. Renderer hooks:
   - Solid renderer reads `spec.renderer.solid.imports` and injects imports at top of generated TSX. Imports may be provided as raw lines or as structured entries (module/names/default) — the renderer will normalize, merge named imports per module, and deduplicate imports once per document.
   - Renderer-specific templates in spec map AST -> output (fallback to AST JSON when absent). Templates should be string templates expanded by the renderer (no arbitrary eval by default).
5. Security: templates are sanitized strings (no direct runtime eval). Advanced hooks (WASM/JS) must be explicitly enabled and run in a sandbox.

## Integration points (where to modify codebase)

- `apps/cli/src/main.rs`: load `--plugin toml:...` entries and call `plugin_toml::process(&events, &spec)` in the plugin loop.
- `crates/plugin-markdown/src/lib.rs`: use as reference for event-transform behavior; `plugin-toml` will follow same Event semantics.
- `crates/renderer-solid/src/lib.rs`: add small API to accept `extra_imports: &[String]` or a `SpecRenderHints` structure.
- `crates/plugin-toml` (new crate): implement TOML parsing, validation and runtime event transformations.

## Risks and mitigations

- Incorrect specs causing surprising ASTs: mitigate with strict validation and a `--dry-run` mode that prints diagnostics.
- Malicious templates: disallow arbitrary code execution; provide opt-in WASM executor later.
- Performance: applying many runtime plugins may add allocations; recommend compiled Rust plugins for hotspots.

## Next steps for implementers

1. Implement `crates/plugin-toml` with TOML schema and loader.
2. Add CLI parsing for `toml:` plugin entries and register specs.
3. Add renderer hook in `renderer-solid` to accept import lines and renderer hints.
4. Provide integration tests and an example `plugins/alert.toml`.

See also: [`SPEC.md`](SPEC.md) and [`USAGE.md`](USAGE.md) for the TOML schema and usage examples.

## Index-based plugin registry (recommended)

Large projects benefit from an explicit index file that lists plugins and their order. The index (commonly `plugins/index.toml`) is a static manifest that the engine reads and orchestrates deterministically. This avoids the unpredictability of globs and makes review/PRs and CI easier.

High-level flow:

1. Project adds `plugins/index.toml` (or another named index path).
2. `pendon.toml` references that index via the `plugin-custom.source` key (single path or array of paths). Example:

```toml
[plugin-custom]
source = ["./plugins/index.toml"]
plugins = ["foo"] # list of plugin ids that tasks may consume
enable_unsafe_hooks = false
```

3. `plugins/index.toml` contains an explicit ordered list of `[[plugin]]` entries. Each entry can either be an inline `PluginSpec` or a `path` referring to a separate TOML file. The index is authoritative for ordering and presence.

Benefits:

- Explicit ordering (reviewable diffs) and atomic updates.
- No filesystem nondeterminism; CI and devs see the same applied order.
- Easier to reason about overrides and policy (override flags in the index).

Index entry schema (conceptual):

[[plugin]]
id = "alert" # stable identifier used by tasks and CLI
name = "Alert" # optional; human-friendly name (for CLI, logging)
enabled = true
path = "alert.toml" # optional; if omitted, `inline` must be supplied
override = false

[plugin.inline]
kind = "block"
[plugin.inline.matcher]
start = ":::alert"
end = ":::"

Rules & semantics:

- The index file is read and parsed deterministically. The order of `[[plugin]]` entries is the application order unless `order` in `pendon.toml` further constrains it.
- `path` is resolved relative to the index file's directory.
- `inline` lets teams keep small specs centralized without adding extra files.
- Duplicate `name` across the index is an error unless `override = true` is explicitly set on exactly one entry.
- The engine emits helpful diagnostics when an index references a missing `path` or contains invalid inline specs.

Integration with CLI tasks and `pendon run`:

- When `pendon run` executes tasks from `pendon.toml`, the task's `plugin` list may include plugin ids that resolve against the index. For example, a task can say `plugin = "alert,markdown"` where `alert` comes from `plugins/index.toml` and `markdown` is built-in. The resolver will expand `alert` to the loaded `PluginSpec`.
- CLI `--plugin` explicit entries still work and override discovery for that invocation.

Security and operational policies:

- The index should be version-controlled. The engine will validate the index on load and offer `--plugin-validate` and `--plugin-dryrun` modes that print normalized specs and resolved import lines.
- Advanced/unsafe hooks (WASM/JS) remain opt-in and only enabled when `enable_unsafe_hooks = true` is set in `pendon.toml` and the runtime allows it.

Caching and performance:

- Parse and normalize each referenced file once per run. Cache by content hash for repeated runs; invalidate on file change.
- The engine should load index files in parallel when multiple sources are specified, then apply ordering/merging serially.
