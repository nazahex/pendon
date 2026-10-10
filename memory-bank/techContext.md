# Tech Context

## Technologies

- **Rust (stable)**, Cargo workspace (`resolver = "2"`).
- **Syntect** for fenced-code highlighting (`plugin-codeblock-syntect`), default
  grammars only (TS/TSX falls back to the JavaScript grammar).
- **Bun** for monorepo JS hygiene: `biome` (lint/format) + `dprint` (docs format),
  wired through `lefthook.yaml` pre-commit hooks.
- **Solid/JSX** is the canonical component output contract (`renderer-solid`).

## Workspace layout

- `apps/cli` — the `pendon` binary; direct CLI (`main.rs`) + config runner
  (`process.rs`, `run.rs`); shared plugin loop in `plugins.rs`.
- `crates/core` — events, parser primitives, options, pipeline.
- `crates/extra` — shared cross-plugin syntax (the `@@`-head grammar).
- `crates/plugin-*` — one transform per crate (see `Cargo.toml` members).
- `crates/renderer-*` — `solid`, `html`, `json`, `ast`, `events`.
- `crates/tui` — optional spinner on stderr.
- `sandbox/*` — executable integration examples; sources in `src/`, output in `out/`.

## Common commands

```bash
cargo build --workspace
cargo test --workspace
cargo test -p <crate>                 # focused first
cargo run --bin pendon -- run         # inside a sandbox dir
cargo run --bin pendon -- run -F      # bypass cache, re-render
bun run check                         # biome
bun run format                        # biome + dprint
```

## Constraints / gotchas

- Use `cargo run --bin pendon` (not an installed `pendon`) during development.
- `sandbox/**` is excluded from biome/dprint (fixture data); `docs/spec/golden/**`
  is **not**, so its generated `.jsx` may still surface lint noise — pre-existing.
- Renderers keep attributes on fallback nodes; custom components get typed
  `AttrValue` via the `attrs` map.
