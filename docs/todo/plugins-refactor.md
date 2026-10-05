# Plugin Refactor

Status: in progress

This document tracks the plugin and runtime refactor started in October 2026. It is an execution handoff document, not a general architecture reference. Permanent rules belong in [`CONTRIBUTIONS.md`](../../CONTRIBUTIONS.md).

## Goal

Make Pendon plugins predictable, consistent, DRY, and safe to evolve while preserving the event-driven IR and consistent Solid/JSX output.

## Completed

- [x] Added `crates/extra` for shared attribute parsing and CSV/property primitives.
- [x] Migrated shared attribute parsing across image, table, anchor, cite, heading, and Vicado paths.
- [x] Centralized built-in Solid JSX attribute emission.
- [x] Centralized plugin-list parsing and stateless plugin execution.
- [x] Centralized custom plugin spec loading and caching.
- [x] Added `DocumentContext` for per-document citation ownership and finalization.
- [x] Added `ContextPipeline<C>` and `InlinePipeline<C>` to support explicit borrowed state.
- [x] Migrated image, HTML table, custom table, and custom inline processing to context-aware APIs.
- [x] Removed the legacy inline pipeline from the config runtime.
- [x] Migrated direct CLI processing to the shared document context pipeline.
- [x] Fixed duplicate heading lifecycle events and heading marker leakage.
- [x] Fixed false Markdown table detection for pipes inside `[[wiki | labels]]`.
- [x] Fixed citation extra IDs overwriting canonical citation IDs.

## Current Architecture

### Runtime paths

Both direct CLI execution (`apps/cli/src/main.rs`) and config execution (`apps/cli/src/process.rs`) now use the shared context-aware inline pipeline builders in `apps/cli/src/plugins.rs`.

The config runtime still owns additional concerns that direct CLI does not have:

- task configuration and external references;
- custom plugin registry;
- cache entries and output writes;
- renderer hint overrides.

These concerns should remain outside the core document context.

### State

`DocumentContext` owns the per-document citation state. Its current implementation uses an internal `Arc<Mutex<CitationContext>>` because the legacy `Pipeline` API requires `Send + Sync + 'static` closures. `ContextPipeline<C>` is the migration path toward direct borrowing.

Do not remove the mutex by changing lifetimes casually. First migrate all stateful processors to `ContextPipeline<DocumentContext>`, then simplify ownership and verify nested caption/table processing.

## Remaining Work

### 1. Remove the internal citation mutex

- [ ] Confirm all stateful inline processing uses `ContextPipeline<DocumentContext>`.
- [ ] Make `DocumentContext` own `CitationContext` directly.
- [ ] Replace `Arc<Mutex<_>>` access with `&mut DocumentContext`.
- [ ] Keep citation finalization single-owner and after all plugin phases.
- [ ] Run universal, cite, image-caption, and table integration checks.

### 2. Introduce a final `PluginRuntime`

- [ ] Extract the shared parse, plugin execution, finalization, and render lifecycle.
- [ ] Keep file discovery, cache, task interpolation, and output writes in the config runner.
- [ ] Keep argument parsing and stdout/exit-code policy in the direct CLI.
- [ ] Delete duplicated plugin loops only after both paths produce equivalent outputs.

### 3. Standardize diagnostics and errors

- [ ] Define a shared diagnostic code/category convention.
- [ ] Introduce a runtime/plugin error type without hiding user-facing context.
- [ ] Decide which diagnostics are returned as events and which are fatal runtime errors.
- [ ] Preserve strict-mode behavior.

### 4. Add output contracts

- [ ] Add golden tests for representative JSX from image, link, citation, heading, table, custom, and LaTeX combinations.
- [ ] Assert stable attribute ordering and escaping.
- [ ] Assert that disabled plugins do not transform syntax.
- [ ] Assert both direct CLI and config runtime paths.

### 5. Complete validation

- [ ] Run `cargo test --workspace`.
- [ ] Run `cargo run --bin pendon -- run -F` in `sandbox/custom`.
- [ ] Run it in `sandbox/syntect`.
- [ ] Run it in `sandbox/universal`.
- [ ] Run it in `sandbox/table`.
- [ ] Run it in `sandbox/anchor` and `sandbox/img/inline`.
- [ ] Inspect generated files under each relevant `out/` directory.

## Important Decisions

- Keep legacy public APIs temporarily when adding context-aware APIs; remove them only after all consumers migrate.
- Keep plugin semantics local. Shared crates may provide parsing, pipeline, diagnostics, and rendering primitives, but must not encode plugin-specific meaning.
- Do not make disabled plugins run implicitly through nested caption or table processing.
- Run citation finalization once, after all document and nested inline processing.
- Treat generated sandbox output as an integration contract, not as an incidental build artifact.

## Validation Notes

The following focused checks have passed during this refactor:

- `cargo test -p pendon-core`
- `cargo test -p pendon-plugin-img`
- `cargo test -p pendon-plugin-table`
- `cargo test -p pendon-plugin-custom`
- `cargo test -p pendon`
- `cargo test --workspace`
- `cargo fmt --all -- --check`
- `git diff --check`
- CLI `run -F` checks for custom, syntect, universal, and table sandboxes.

When a validation fails, fix the smallest owning abstraction first and rerun the same focused check before widening the refactor.
