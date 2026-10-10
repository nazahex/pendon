# Pendon — Micro TODO & Options

Status: living document for day-to-day tracking. See also: [docs/RFC-0001-pendon.md](docs/RFC-0001-pendon.md)

## Now (Focus: Parser)

- [x] Byte-level lexer skeleton (normalize CRLF, fast newline/fence/hash detection)
- [ ] Paragraph invariants doc (start/end rules, leading/trailing blanks)
- [ ] DoS guards (max line length, max blank-run, max document size gate)
- [x] Minimal error event type (for strict mode later)

## Next Up

- [ ] Strict mode toggle path (wire `Options.strict`, define unknown-block policy)
- [ ] CLI `--renderer <json|yaml|solid>` flag plumbing (no-op for yaml/solid initially)
- [ ] TUI diagnostics widget (error list/table) and localized strings
- [ ] CI: add `cargo test` task to Turborepo pipeline

## Completed (MVP baseline)

- [x] CLI produces JSON IR { type: "Document", children: <full string> } — [apps/cli/src/main.rs](apps/cli/src/main.rs)
- [x] TUI crate (spinner/progress bar; stderr-only; gated by `--tui`) — [crates/tui](crates/tui)
- [x] Core event model + parser (Document, Paragraph, Text; CRLF→LF) — [crates/core](crates/core)
- [x] JSON renderer (event→string) — [crates/renderer-json](crates/renderer-json)
- [x] Integration tests for CLI, unit tests for parser

## Parser Notes (Spec Extract)

- Paragraph open on first non-newline text; close on blank line (two consecutive newlines).
- Preserve newlines as Text("\n") for fidelity; renderer may concatenate.
- CRLF normalized to LF at ingress.
- Leading/trailing blank lines are preserved as Text("\n") but do not open paragraphs.
- `max_blank_run` emits a single Warning diagnostic when exceeded; text remains unmodified. See [docs/spec/PARSER.md](docs/spec/PARSER.md).

## Options / Recommendations

- Parser
  - [x] Introduce Event::Diagnostic { span, message, severity } for strict/permissive flows
  - [ ] Add NodeKind::CodeFence detection (still emit as Text until plugin system)
  - [ ] Byte-slice indices on spans to avoid string allocs (later zero-copy)
  - [ ] Arena or smallvec-backed buffers for temporary chunks
- Plugin System (design placeholder)
  - [ ] Macro surface md_plugin! (block/inline) — trait boundaries and ctx API
  - [ ] Deterministic ordering + conflict resolution (priority tiers)
- Renderers
  - [ ] YAML renderer crate (mirrors JSON)
  - [ ] Solid IR renderer crate (streaming-first)
- TUI
  - [ ] Nerd Fonts glyph set behind `nerd-fonts` feature
  - [ ] Diagnostics view with table templates and themes
- CLI / DX
  - [ ] `--strict` (default off) and error-exit policy
  - [ ] `--stats` to print timing/memory summary to stderr
- CI / Tooling
  - [ ] `turbo` task for `cargo test` and `cargo clippy` (non-fatal on warnings initially)

## Quick Commands

```bash
# Run all tests
cargo test

# CLI runs
cargo run --bin pendon -- --input docs/PROPOSAL.md
printf "Hello\nWorld" | cargo run --bin pendon -- --tui
```

## References

- RFC: [docs/RFC-0001-pendon.md](docs/RFC-0001-pendon.md)
- CLI: [apps/cli/src/main.rs](apps/cli/src/main.rs)
- Core: [crates/core](crates/core)
- TUI: [crates/tui](crates/tui)
- Renderer JSON: [crates/renderer-json](crates/renderer-json)
