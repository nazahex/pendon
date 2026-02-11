# pendon-plugin-custom

TOML-driven, runtime-loadable plugin bridge for Pendon. It lets you declare Markdown-like syntaxes, map them to Pendon AST nodes, and attach renderer hints without writing Rust.

## What It Does

- Loads plugin specs from standalone TOML files or an index manifest.
- Detects custom block/inline markers using literals or regexes, then rewrites the Pendon event stream into typed nodes.
- Extracts attributes from capture groups or `key: value` pairs, with type parsing and defaults.
- Emits AST nodes/components with optional attribute remapping and renderer hints (e.g., Solid imports + JSX templates).
- Provides ready-to-use loader and processor functions for CLI and library consumers.

## Quick Start

### Single Plugin Spec

Create a TOML plugin describing the syntax you want to recognise:

```toml
name = "alert"
kind = "block"

[matcher]
start = ":::alert"
end = ":::"

[[attrs]]
name = "type"
required = true
default = "info"

[ast]
node = "Component"
node_name = "Alert"
attrs_map = { type = "type" }

[renderer.solid]
imports = ["import Alert from './Alert'"]
component_template = "<Alert type=\"{attrs.type}\">{children}</Alert>"
```

Run via CLI:

```bash
pendon --plugin "toml:plugins/alert.toml" --format ast --input ./doc.md
```

### Index Manifest (recommended)

List plugins in a deterministic manifest (commonly `plugins/index.toml`), then point `pendon.toml` to it:

```toml
# pendon.toml
[plugin-custom]
source = ["./plugins/index.toml"]
```

```toml
# plugins/index.toml
[[plugin]]
id = "alert"
path = "alert.toml"
enabled = true

[[plugin]]
id = "tabs"
inline = { name = "tabs", kind = "block", matcher = { start = ":::tabs", end = ":::" } }
```

`load_index_from_path` resolves relative `path` entries against the index directory and returns only enabled plugins (inline or file-backed).

## Matching Rules

- `matcher.start` (literal) or `matcher.start_regex` (PCRE) determines block start. `matcher.end` defaults to `:::` if omitted.
- Inline or custom behaviours can be hinted via `matcher.parse_hint`:
  - `blockquote-sigil`: scan blockquote paragraphs for the start marker; matching paragraphs become a custom component.
  - `codefence-lang`/`codefence-viewer`: if a fenced code block `lang` matches the detector, its body is wrapped as a component.
- Paragraph wrappers immediately preceding a matched block are suppressed to avoid nested paragraphs.

## Attributes

- Supported types: `string`, `int`, `bool`, `list<string>`.
- Resolution order: named regex capture → key/value map → default. Missing required attrs emit `Diagnostic` errors.
- Key/value map parsing accepts inline maps such as `{ foo: "bar", count: 3, tags: ["a", "b"] }`.

## AST Emission

- `ast.node` selects the emitted `NodeKind` (known kinds like `Heading`, `CodeFence`, etc., or falls back to `Custom(name)`).
- `ast.node_name` populates an emitted `name` attribute when present (useful for component names).
- `ast.attrs_map` remaps captured attrs to outgoing attribute keys.
- Children between start/end markers are preserved inside the emitted node; paragraph closing is skipped when appropriate.

## Renderer Hints (Solid)

- `renderer.solid.imports`: raw import lines or structured entries (`module`, `default`, `names`) to inject once per document.
- `renderer.solid.component_template`: JSX/TSX template tokens (`{children}`, `{attrs.<name>}`, `{text}`) expanded by the Solid renderer.

## Rust API Surface

- `load_spec_from_path(path) -> Result<PluginSpec, String>`: read + parse a single TOML spec.
- `load_index_from_path(path) -> Result<Vec<IndexedPlugin>, String>`: load an index manifest and resolve inline or file-backed specs.
- `process(events: &[Event], spec: &PluginSpec) -> Vec<Event>`: apply the spec to an event stream, emitting transformed events and diagnostics.

These functions are used by the Pendon CLI but can be embedded in other consumers that work with `pendon_core::Event`.

## Diagnostics & Safety

- Invalid attr parsing or missing required attrs emit `Diagnostic` events with `Severity::Error`; processing continues best-effort.
- Unknown `ast.node` values are allowed and emitted as `Custom` node kinds, enabling downstream renderers to handle them.
- No arbitrary code execution; renderer templates are plain strings processed by renderers.

## Demos

A minimal demo lives in `sandbox/custom/` (see `custom/foo.toml` and `custom/index.toml`). Run:

```bash
pendon --format solid --input sandbox/custom/src/demo.md
```

It applies the custom plugin and renders Solid output with the provided component template.
