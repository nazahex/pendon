# pendon-plugin-micromatter

Micomatter is a safe, YAML-inspired frontmatter parser for Pendon. It accepts a small, predictable subset and emits a `Frontmatter` node with a single `data` attribute containing normalized JSON. Renderers can then expose the data without re-parsing.

## Format (flat YAML subset)

- Allowed value types: booleans (`true`/`false`), integers, floats, strings (quoted or bare), and homogeneous arrays of those scalars.
- Not allowed: indentation, nested objects, multiline strings, anchors/aliases, implicit truthy values, date auto-parsing.
- Comments start with `#` outside quoted strings and are stripped.
- Arrays must be single-type (e.g., all strings or all numbers); mixed arrays are rejected.

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
- `[micromatter] missing ':' on frontmatter line N`
- `[micromatter] mixed array types`
- `[micromatter] unterminated quoted string`

Use `--strict` to treat these diagnostics as errors for the CLI exit code.
