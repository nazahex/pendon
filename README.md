# Pendon

Event-driven, plugin-first Markdown-as-DSL engine with a predictable IR.

## Installation

- Requires Rust (stable). Clone the repo and build:

```bash
cargo build --workspace
```

## CLI Usage

The CLI binary is `pendon`. It reads from stdin by default or from a file via `--input`.

```bash
# From stdin
echo "Hello" | pendon

# From a file
pendon --input ./README.md

# JSON renderer (default)
pendon --format json < ./doc.md

# Events renderer (debug-friendly)
pendon --format events --input ./doc.md

# AST renderer (structured output)
pendon --format ast --input ./doc.md
```

### Flags

- `--input <path>`: Read input from file instead of stdin.
- `--format <name>`: Output format. Supports `json`, `events`, `ast`.
- `--strict`: Escalate diagnostics into errors. The CLI still prints output and exits with a non-zero code if any error is present.
- `--tui`: Show a minimal spinner on stderr while reading input (safe for pipelines).
- `--max-doc-bytes <n>`: Warn/error when input size exceeds `n` bytes.
- `--max-line-len <n>`: Warn/error when a line exceeds `n` characters.
- `--max-blank-run <n>`: Warn/error when consecutive blank lines exceed `n`.
- `--plugin <name>`: Apply a plugin transform before rendering. Currently supports `markdown`.

### Examples

```bash
# Strict mode with blank-line guard
pendon --strict --max-blank-run 1 < input.md

# Limit line length to 120 characters
pendon --strict --max-line-len 120 --input ./doc.md

# Limit document size to 1MB
pendon --max-doc-bytes 1048576 --input ./doc.md
```

## Behavior & Invariants

- Newlines are preserved as text ("\n") for fidelity.
- Paragraphs open on first non-blank and close on a blank run ≥ 2.
- In strict mode, diagnostics become errors; CLI exits non-zero if any errors occurred.

See `docs/spec/PARSER.md` for more details.

## Plugins

- `markdown`: Normalizes a subset of Markdown blocks:

  - Headings: `#` prefix removed, newline dropped; represented as `Heading` nodes.
  - Code fences: Marker lines suppressed; inner content retained as `CodeFence` nodes.
  - Thematic breaks: Hyphen lines suppressed; represented as `ThematicBreak` nodes.

- `syntect`: Syntax highlighting for fenced code blocks using Syntect's built-in grammars.
  - Uses default syntax set bundled with Syntect (no external grammar loading).
  - TypeScript/TSX fallback: highlights using JavaScript grammar; otherwise plain text if unsupported.

Examples:

```bash
# Render structured AST with markdown plugin
pendon --plugin markdown --format ast --input ./doc.md

# Inspect low-level events with plugin applied
pendon --plugin markdown --format events --input ./doc.md

# HTML with syntax highlighting
pendon --plugin markdown,syntect --format html --input ./doc.md
```

Note: Plugin coverage is evolving. Non-recognized structures remain as text inside `Paragraph` nodes.

## Formats

- `json`: Concatenated text IR for quick preview.
- `events`: Raw event stream (Start/End/Text/Diagnostic) for debugging and plugin development.
- `ast`: Hierarchical JSON AST with nodes and aggregated text, suitable for downstream transforms.

## TUI

- Optional `--tui` shows a spinner on stderr during input processing. Safe for pipelines; does not interfere with stdout JSON.

## Sandbox

- Quick demos and file outputs:
  - Run `bash sandbox/run_files.sh` to write JSON and AST outputs into `.temp/`.
  - See example inputs in `sandbox/examples/`.

## Usage Guide

For a more comprehensive, public-facing usage guide, see `docs/USAGE.md`.

## Development

Run tests:

```bash
cargo test --workspace
```

Basic lint/format (JS tooling present for monorepo hygiene):

```bash
bun run check
bun run format
```

## Status

See `docs/STATUS.md` for MVP progress and review tracking.
