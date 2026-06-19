# pendon-plugin-micromatter

Micromatter is a frontmatter parser for Pendon that uses full YAML parsing. It emits a `Frontmatter` node with a single `data` attribute containing normalized JSON, so renderers can expose the data without re-parsing.

## Format (full YAML)

- Any valid YAML document inside the leading and trailing `---` fences is accepted.
- Nested mappings, sequences, block scalars, booleans, numbers, nulls, anchors, aliases, and quoted strings are supported by the YAML parser.
- Parsed YAML is converted to JSON and stored in `attrs.data`.
- Mapping keys must still be scalar values so they can be represented in JSON.

Example:

```text
---
title: "Demo"
draft: false
score: 95.5
tags: ["specification", "metadata", "performance"]
---
```

## Behavior

- Detects frontmatter only at the very start of the document, bounded by `---` fences that the core parser emits as `ThematicBreak` nodes.
- On success, injects a `Frontmatter` node under `Document` with `attrs.data` holding a JSON string.
- On error (invalid grammar, mixed array types, missing closing fence), emits a `Diagnostic::Error` with a `[micromatter]` prefix and leaves the original events intact.
- Downstream renderers:
  - AST/JSON: `Frontmatter` node is present with `attrs.data` JSON.
  - Solid: exports `export const frontmatter = {...}` before the component.
  - HTML: frontmatter is skipped (no rendered output).

## CLI usage

Run micromatter before markdown so the block is stripped and normalized:

```bash
pendon --plugin micromatter,markdown --format ast --input ./doc.md
pendon --plugin micromatter,markdown --format solid --input ./doc.md
```

In `pendon.toml` tasks, list `micromatter` first in `plugin` values:

```toml
plugin = "micromatter,markdown"
```

## Diagnostics

Typical error messages:

- `[micromatter] missing closing ---`
- `[micromatter] invalid YAML frontmatter: ...`
- `[micromatter] YAML mapping keys must be scalar values`

Use `--strict` to treat these diagnostics as errors for the CLI exit code.
