# CLI Guide

The CLI binary is `pendon`. During development always use
`cargo run --bin pendon` so you run the current code, not an old build.

## Input

Reads from stdin by default, or from a file with `--input`.

```bash
echo "Hello" | cargo run --bin pendon --
cargo run --bin pendon -- run                 # full run with pendon.toml config
cargo run --bin pendon -- run -F              # bypass cache and re-render
cargo run --bin pendon -- --input ./README.md
```

## Formats (`--format`)

| Format   | Output                                                     |
| -------- | ---------------------------------------------------------- |
| `json`   | concatenated text IR for quick preview (default)           |
| `events` | raw event stream (Start/End/Text/Diagnostic) for debugging |
| `ast`    | hierarchical JSON AST with nodes and aggregated text       |
| `html`   | HTML                                                       |
| `solid`  | JSX/TSX (the component contract)                           |

`--pretty` selects the indented mode of a format that has one (`json`, `events`,
`ast`, `html`). Same as `pretty = true` on a task.

## Flags

- `--input <path>` — read from a file instead of stdin.
- `--format <name>` — `json`, `events`, `ast`, `html`, `solid`.
- `--pretty` — indented output where supported.
- `--strict` — escalate diagnostics to errors; the CLI still prints output and
  exits non-zero if any error is present.
- `--tui` — minimal spinner on stderr (safe for pipelines; does not touch stdout).
- `--max-doc-bytes <n>` — warn/error when input exceeds `n` bytes.
- `--max-line-len <n>` — warn/error when a line exceeds `n` characters.
- `--max-blank-run <n>` — warn/error when consecutive blank lines exceed `n`.
- `--plugin <name>` — apply a plugin transform before rendering (e.g. `markdown`,
  `markdown,syntect`). Multi-plugin lists are comma-separated.

## Examples

```bash
# Strict mode with blank-line guard
cargo run --bin pendon -- --strict --max-blank-run 1 < input.md

# AST with the markdown plugin
cargo run --bin pendon -- --plugin markdown --format ast --input ./doc.md

# HTML with syntax highlighting
cargo run --bin pendon -- --plugin markdown,syntect --format html --pretty --input ./doc.md

# Config-driven run inside a sandbox
cd sandbox/custom && cargo run --bin pendon -- run -F
```

## Plugins

- `markdown` — normalizes a subset of Markdown blocks (headings, code fences,
  thematic breaks).
- `syntect` — syntax highlighting for fenced code (Syntect default grammars;
  TS/TSX falls back to the JavaScript grammar).

Plugin coverage is evolving; non-recognized structures remain as text inside
`Paragraph` nodes. The full plugin set and order are documented in
`CONTRIBUTORS.md` and each `crates/plugin-*/README.md`.
