# Pendon Usage Guide

Concise, public-facing guide for using the Pendon CLI and plugins.

## Quickstart

- Build the workspace:

```bash
cargo build --workspace
```

- Basic runs:

```bash
# Read from stdin (default)
echo "Hello" | pendon

# Read from a file
pendon --input ./doc.md

# JSON renderer (AST JSON)
pendon --format json --input ./doc.md

# Events renderer (debug-friendly)
pendon --format events --input ./doc.md

# AST renderer (structured output)
pendon --format ast --input ./doc.md
```

## Plugins

- Micomatter frontmatter (run it before markdown):

```bash
pendon --plugin micromatter,markdown --format ast --input ./doc.md
pendon --plugin micromatter,markdown --format solid --input ./doc.md
```

- Enable the Markdown plugin to normalize common structures:

```bash
# Apply markdown plugin and render AST
pendon --plugin markdown --format ast --input ./doc.md

# Apply markdown plugin and render events
pendon --plugin markdown --format events --input ./doc.md
```

### What the Markdown plugin does today

- Headings: Strips leading `#` and the trailing newline; yields `Heading` nodes with inner text.
- Code fences: Suppresses opening/closing backtick lines; preserves inner content as `CodeFence` nodes.
- Thematic breaks: Suppresses hyphen line; yields `ThematicBreak` nodes.
- Blockquote: Converts `>` prefixes into nested `Blockquote` nodes.
- Tables: Parses pipe tables into `Table`/`TableHead`/`TableBody`/`TableRow`/`TableCell` with header attr on the first row.

Unrecognized or unsupported Markdown patterns remain as plain text inside `Paragraph` nodes. Coverage will expand over time.

### Micomatter frontmatter plugin

- Parses a flat, YAML-inspired subset between leading `---` fences into a `Frontmatter` node with `attrs.data` JSON.
- Rendering:
  - AST/JSON: frontmatter node is present with `attrs.data`.
  - Solid: `export const frontmatter = {...}` is emitted above the component.
  - HTML: frontmatter is skipped.
- Use `--plugin micromatter,markdown` (micromatter first) so the block is stripped before markdown parsing.

## Syntax Highlighting (code fences)

Use the `syntect` plugin to render fenced code blocks into minimal HTML with class-based styles.

```bash
# Render HTML with markdown normalization + syntax highlighting
pendon --plugin markdown,syntect --format html --input ./doc.md
```

Behavior:

- Uses Syntect's built-in default syntaxes at runtime (no disk I/O).
- No external grammar loading.
- TypeScript/TSX is highlighted via JavaScript grammar; otherwise falls back to plain text if unsupported.

Environment toggles:

- `PENDON_SYNTECT_DEBUG=classes`: emit raw classed HTML spans (useful for debugging mapping rules).

Styling:

- The output uses minimal tags mapped from Syntect classes; provide a stylesheet to style tokens.
- See `sandbox/syntect/static/style.css` for a lightweight example.

### AST Attributes

The AST includes simple node attributes for better usability:

- Headings: `attrs.level` is set to `"1".."6"` based on `#` count.
- Code fences: `attrs.lang` captures the info string (e.g., `"ts"`).
- Ordered lists: `attrs.start` on the list node is set to the first item number.

Example:

```json
{
  "type": "Heading",
  "attrs": { "level": "2" },
  "text": "Section Title",
  "children": []
}
```

## Output Formats

- `json`: Structured AST JSON (same schema as `ast`), optionally pretty-printed with `--pretty`.
- `events`: Low-level event stream (`StartNode`, `EndNode`, `Text`, `Diagnostic`); best for debugging and plugin development.
- `ast`: Hierarchical JSON AST with nested nodes and aggregated text; suitable for downstream processing.
  - Nodes may include `attrs` (object) and `diagnostics` at document-level.

## Lists

- Bullet lists: lines starting with `-`, `*`, or `+` become a `BulletList` containing `ListItem` children.
- Ordered lists: lines starting with `<digits>.` or `<digits>)` become an `OrderedList` with `ListItem` children; `attrs.start` is set from the first number.

### Nested and Multiline Items

- Nesting: Indentation (spaces at line start) controls nesting depth. A deeper indent opens a nested list under the current `ListItem`; a shallower indent closes deeper lists.
- Continuation lines: Non-marker lines at the same or deeper indent are treated as continuation content within the current `ListItem`.
- Mix and match: You can nest `BulletList` under `OrderedList` (and vice versa) by switching markers at any indent level.

Examples:

```bash
# Nested bullets
printf "- a\n  - b\n    - c\n- d\n" | pendon --plugin markdown --format ast

# Multiline item (continuation)
printf "- first line\n  continuation line\n- second item\n" | pendon --plugin markdown --format ast

# Mixed nesting (ol → ul)
printf "1. one\n  - detail\n2) two\n" | pendon --plugin markdown --format ast
```

Examples:

```bash
# Bullet list
printf "- one\n- two\n" | pendon --plugin markdown --format ast

# Ordered list starting at 3
printf "3. three\n4) four\n" | pendon --plugin markdown --format ast
```

## Strict Mode & Guards

- Turn diagnostics into errors with `--strict`. CLI still prints output but exits non-zero if any errors occurred.

```bash
# Limit blank runs to 1 and escalate to error
pendon --strict --max-blank-run 1 --input ./doc.md

# Limit line length to 120
pendon --strict --max-line-len 120 --input ./doc.md

# Limit document size to 1MB
pendon --max-doc-bytes 1048576 --input ./doc.md
```

## TUI

- `--tui` shows a lightweight spinner on stderr while reading input; safe for pipelines and does not affect stdout JSON.

```bash
pendon --tui --format ast --input ./doc.md > doc.ast.json
```

## Sandbox & Examples

- Generate demo outputs into `.temp/`:

```bash
bash sandbox/run_files.sh
```

- Inspect example inputs in `sandbox/examples/`.

## Troubleshooting

- File not found: CLI reports an error and exits non-zero.
- Strict mode: Any emitted error causes non-zero exit; check stderr and consider relaxing guards.
- Large files: Use `--max-doc-bytes` and `--max-line-len` thoughtfully to avoid excessive diagnostics.

## Contributing

- See the root `README.md` for development commands and project status.
