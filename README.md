# Pendon

Event-driven, plugin-first Markdown-as-DSL engine with a predictable IR.

> **Documentation map:** [`docs/INDEX.md`](docs/INDEX.md) (humans) and
> [`AGENTS.md`](AGENTS.md) (AI agents). Live work: [`STATUS.md`](STATUS.md).
> Normative grammar: [`docs/spec/SYNTAX.md`](docs/spec/SYNTAX.md).

## Install & build

Requires Rust (stable). Clone and build:

```bash
cargo build --workspace
```

## Quick start

```bash
# From stdin
echo "Hello" | cargo run --bin pendon --

# Full run with pendon.toml config (inside a sandbox/ dir)
cargo run --bin pendon -- run

# Bypass cache and renew it
cargo run --bin pendon -- run -F

# From a file, pretty HTML with the markdown plugin
cargo run --bin pendon -- --plugin markdown --format html --pretty --input ./doc.md
```

Reminder: use `cargo run --bin pendon` during development to run the current code
rather than an old build.

## Formats

`--format` selects the output: `json` (default), `events`, `ast`, `html`, `solid`.
`--pretty` picks the indented mode where a format has one.

## Plugins & renderers

- Plugins live under `crates/plugin-*`; each has its own `README.md`.
- Renderers live under `crates/renderer-*`; Solid/JSX is the component contract.
- The unified typed-extras surface syntax is specified in
  [`docs/spec/SYNTAX.md`](docs/spec/SYNTAX.md) and demonstrated by the executable
  goldens in [`docs/spec/golden/`](docs/spec/golden/README.md) — the source of
  truth, verified byte-for-byte by tests.

## Development

Run tests and the docs format:

```bash
cargo test --workspace
bun run check
bun run format
```

For the full CLI reference, plugin authoring, and testing procedures, see
[`docs/guides/`](docs/guides/) and [`CONTRIBUTORS.md`](CONTRIBUTORS.md).
