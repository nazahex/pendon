# Pendon — MVP Status & Review Tracker

Last updated: 2026-01-21

## What’s Implemented (MVP)

- Core event stream with `Event::{StartNode, EndNode, Text, Diagnostic}` and rich `NodeKind` (`Document`, `Paragraph`, `Heading`, `CodeFence`, `Blockquote`, list types, `Table*`, `Frontmatter`, inline emphasis/bold/italic/link/code, `ThematicBreak`).
- Lexer: CRLF normalization; tokens for `Text`, `Newline`, backtick runs, hash runs.
- Parser:
  - Paragraph open/close on non-blank/blank runs.
  - Newlines preserved as `Text("\n")` for fidelity.
  - Guards: `max_doc_bytes`, `max_line_len`, `max_blank_run` → emit Diagnostic.
  - Strict mode: Diagnostics escalate to `Error` when `Options.strict = true`.
  - Hints: emits Start/End for `Heading` (line-start `#`) and `CodeFence` (line-start backticks≥3); text unchanged.
- Plugins:
  - Markdown: normalizes headings, code fences, thematic breaks; parses blockquotes, lists, and pipe tables into structured nodes.
  - Micomatter: parses flat YAML-inspired frontmatter into a `Frontmatter` node with JSON payload; emits diagnostics on errors.
  - Codeblock-syntect: highlights fenced code blocks to minimal HTML spans.
- Renderers:
  - JSON: emits structured AST JSON (same shape as the AST renderer), including diagnostics and attrs.
  - AST: hierarchical JSON with attrs; pretty and compact variants.
  - HTML: renders structural HTML, skips frontmatter, supports blockquote/table/list/code fence.
  - Solid: renders JSX and exports `frontmatter` const when present.
- CLI:
  - Input via stdin or `--input <path>`.
  - Formats: `json` (default), `events`, `ast`, `html`, `solid`.
  - Plugins: `--plugin micromatter,markdown[,syntect]`; config-driven tasks in `pendon.toml` honor plugin order.
  - `--tui` optional (stderr-only spinner, no impact on stdout/IR).
  - Strict/guards flags: `--strict`, `--max-doc-bytes`, `--max-line-len`, `--max-blank-run` with non-zero exit on Errors.
- TUI crate: spinner/progress-bar, themes, helpers (inactive unless `--tui`).
- Docs: RFC-0001, parser invariants in `docs/spec/PARSER.md`, usage in `docs/USAGE.md`.

## Tests

- Core parser: newline segmentation, CRLF normalization, paragraph counts, blank-run diagnostics, strict escalation.
- Lexer: CRLF and fence/hash detection.
- CLI: stdin/file inputs, multiline, unknown block preserved as text, file-not-found, strict+guard exit behavior; AST/JSON/Solid/HTML cover headings, lists, blockquote, tables, code fences, and micromatter frontmatter.

## Developer Flags (CLI)

- `--strict`: escalate Diagnostics to Errors.
- `--max-doc-bytes <n>`: warn/error if input bytes > n.
- `--max-line-len <n>`: warn/error if line length > n.
- `--max-blank-run <n>`: warn/error on blank line run > n.
- `--tui`: show spinner on stderr while reading input.

## Invariants

- Preserve all newlines in `Text("\n")`.
- Leading/trailing/extra blank lines preserved as text; don’t auto-open paragraphs.
- Paragraph opens on first non-blank, closes on blank run ≥ 2.

## Open Items / Next Steps

- Additional renderers (e.g., MDX/YAML) without breaking IR stability.
- CI integration via Turborepo tasks.
- TUI diagnostics view (stderr) for Errors/Warnings.

## Recent Changes

- JSON renderer now emits full AST shape (attrs, diagnostics, structured nodes).
- Micomatter frontmatter plugin added; Solid exports `frontmatter` when available.
- Markdown plugin handles blockquote and pipe tables; renderers support them across HTML/AST/Solid.
