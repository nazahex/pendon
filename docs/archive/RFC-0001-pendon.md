# RFC-0001: Pendon — High-Performance Modular Markdown Engine

Status: Draft
Date: 2026-01-10
Authors: Core Team

## Summary

Pendon is a programmable, event-driven Markdown-as-DSL engine designed for modern frontend workflows. It prioritizes developer experience (DX), small binary size, and limitless syntax customization. The engine separates parsing, semantics, and output, enabling domain-specific renderers (e.g., SolidJS IR, JSON/YAML) and plugin-defined syntax without enforcing CommonMark compliance.

## Goals

- Provide a byte-level, streaming parser producing a semantic event stream.
- Offer a safe, declarative plugin DSL for custom block/inline syntax.
- Enable renderer plugins to transform events into output-native formats.
- Ensure excellent DX via a clear `Context` API and friendly diagnostics.
- Achieve predictable performance with minimal allocation and small footprint.
- Supply a TUI module for progress, diagnostics, and developer ergonomics.

## Non-Goals

- Full CommonMark compatibility.
- HTML-first rendering model or WYSIWYG editor.
- Forcing a single AST model; tree building is optional and renderer-driven.

## Architecture

Layers are strictly separated and communicate via stable interfaces:

1. Input Validation
   - Validate UTF-8 once; parsing uses byte indices.
2. Lexer (Byte-level FSM)
   - Emits tokens with spans; avoids premature string allocation.
3. Parser Core (State Machine)
   - Consumes tokens; produces semantic events without building an AST by default.
4. Event Stream
   - Core event types: `StartNode`, `EndNode`, `Text`, `Component`, `Attribute`.
5. Plugins
   - Syntax plugins extend parsing semantics; renderer plugins consume events.

### Event Model

Events are emitted in-order with source spans for diagnostics. Renderers may stream-consume or build trees on demand. Unknown or custom blocks are handled by plugins; strict mode toggles error vs passthrough.

### Context API

The `Context` provides high-level, safe operations to plugin authors:

- `ctx.text()` — access current inline text segment.
- `ctx.children()` — retrieve nested event slice for block scopes.
- `ctx.attr(name)` — access attribute values with type-safe retrieval.
- `ctx.component(name)` — begin an output-native component.
- `ctx.emit_text()` — push text to event stream.

Context enforces isolation, deterministic behavior, and controlled capabilities.

## Plugin DSL & API

Provide a declarative macro `md_plugin!` for block and inline syntax:

```rust
md_plugin! {
    name: "alert",

    block {
        start: ":::alert",
        end: ":::",

        attrs {
            type: String,
            title?: String,
        }

        render |ctx| {
            ctx.component("Alert")
               .prop("type", ctx.attr("type"))
               .children(ctx.children());
        }
    }
}
```

Inline definition example:

```rust
md_plugin! {
    name: "highlight",

    inline {
        marker: "==",

        render |ctx| {
            ctx.component("Highlight")
               .child(ctx.text());
        }
    }
}
```

### Safety & Isolation

- Plugins operate through `Context` only; no direct lexer/parser mutation.
- Attribute schemas validated at compile-time where possible, runtime otherwise.
- Deterministic plugin execution order; explicit priority and conflict resolution.

## Renderer Contracts

Renderers implement a trait to consume events in streaming mode, with optional tree building. Targets include JSON, YAML, and SolidJS-like IR.

Key design points:

- Streaming-first with backpressure support (bounded buffers, sink signals).
- Feature flags control build-time output modules: `json`, `yaml`, `solid`.
- Clear lifecycle hooks: `begin()`, `on_event(ev)`, `end()`.
- Error surfaces include source spans and plugin names.

## TUI Module

Purpose: improve DX during parsing, plugin authoring, and rendering.

Structure:

- `tui/mod.rs` — module registration.
- `tui/assets.rs` — icon/glyph definitions (ASCII/Nerd fonts fallback).
- `tui/locales.rs` — strings, error messages, static text with i18n.
- `tui/templates.rs` — table/list rendering for outputs and diagnostics.
- `tui/theme.rs` — consistent color/style definitions.
- `tui/widgets/` — spinners, progress bars, status indicators.

Usage examples:

- Progress for large inputs or multi-pass renders.
- Diagnostics view for unknown blocks/unclosed regions.
- Bench visualization (latency/throughput charts via textual templates).

## Performance & Memory Plan

- Byte-level lexer; streaming parser; avoid global AST allocation.
- Minimize allocations: `SmallVec`, arenas for transient nodes.
- Build flags: `panic = abort`, LTO, symbol strip.
- Predictable perf: avoid hidden recursion, prefer iterative state machines.
- Bench harnesses: micro (lexer, parser), macro (end-to-end render).

Targets:

- Latency: sub-millisecond for small docs; linear scaling for large.
- Memory: bounded buffers; avoid unbounded growth on malformed input.

## Error Handling & Diagnostics

- Human-friendly errors with spans and annotated context:

```
Unclosed :::alert block
  ┌─ input.md:12:1
  │
12│ :::alert type=warning
  │ ^^^^^^^^^^^^^^^^^^^^
```

- Policies: strict vs permissive, unknown blocks handling.
- Recovery strategies: best-effort continuation, error events to renderer.
- i18n via `tui/locales.rs`.

## Security & Safety Notes

- UTF-8 validation at ingress; all indices are byte-based.
- Plugin sandboxing: capability-limited `Context` only.
- Denial-of-service mitigation: limits on nesting depth, token lengths.
- Deterministic behavior; no hidden global state.

## Testing & Benchmarking Plan

- Unit tests per layer (lexer, parser, events, context).
- Plugin tests: block/inline, attributes, conflict resolution.
- Renderer golden files for JSON/YAML/Solid IR outputs.
- Benchmarks: micro and macro; CI runs minimal perf checks.
- CI matrix: stable + nightly, feature flags `json|yaml|solid`.

## Milestones & Timeline

M0 — Repository Skeleton (1 week)

- Create core crates layout; wire basic modules and feature flags.
- Acceptance: compiles with no-op renderer; basic TUI skeleton.

M1 — Lexer + Event Stream (2 weeks)

- Implement byte-level FSM lexer and event emission for baseline Markdown.
- Acceptance: unit tests for tokens/events; streaming demo.

M2 — Plugin System (3 weeks)

- Implement `md_plugin!` macro, `Context`, block/inline support.
- Acceptance: sample plugins (`Alert`, `Tabs`, `Highlight`) working.

M3 — Renderers (2 weeks)

- Implement JSON/YAML/Solid IR renderers; streaming consumption with backpressure.
- Acceptance: golden files; interchange tests across renderers.

M4 — TUI Module (2 weeks)

- Implement assets/locales/templates/theme/widgets for progress/diagnostics.
- Acceptance: interactive/dev-mode views for parsing and errors.

M5 — Performance & Bench (2 weeks)

- Optimize allocations; enable `panic=abort`, LTO, strip; finalize benches.
- Acceptance: perf targets met on representative docs.

## Risks & Mitigations

- Plugin complexity: keep DSL minimal; provide strong examples and linting.
- Streaming edge cases: formalize event invariants; fuzz testing.
- Renderer divergence: clear trait contracts; versioned capabilities.

## Open Questions

- Best default policies for unknown/third-party blocks?
- Optional AST builder utility for users who prefer tree APIs?
- Capability model granularity for plugins (e.g., file IO restrictions)?

## References

- Proposal overview: see `docs/PROPOSAL.md`.
- Project structure intent reflects `crates/` and `apps/` layout.
