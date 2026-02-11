# pendon-plugin-micomatter

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
- On error (invalid grammar, mixed array types, missing closing fence), emits a `Diagnostic::Error` with a `[micomatter]` prefix and leaves the original events intact.
- Downstream renderers:
  - AST/JSON: `Frontmatter` node is present with `attrs.data` JSON.
  - Solid: exports `export const frontmatter = {...}` before the component.
  - HTML: frontmatter is skipped (no rendered output).

## CLI usage

Run micomatter before markdown so the block is stripped and normalized:

```bash
pendon --plugin micomatter,markdown --format ast --input ./doc.md
pendon --plugin micomatter,markdown --format solid --input ./doc.md
```

In `pendon.toml` tasks, list `micomatter` first in `plugin` values:

```toml
plugin = "micomatter,markdown"
```

## Diagnostics

Typical error messages:

- `[micomatter] missing closing ---`
- `[micomatter] missing ':' on frontmatter line N`
- `[micomatter] mixed array types`
- `[micomatter] unterminated quoted string`

Use `--strict` to treat these diagnostics as errors for the CLI exit code.
