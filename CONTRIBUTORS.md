# Contributing to Pendon

Pendon is an event-driven, plugin-first Markdown-as-DSL engine. It is still alpha software, but changes should follow explicit contracts so the parser, plugins, renderers, and CLI remain predictable.

## Core Principles

1. Prefer one shared implementation over plugin-local copies.
2. Preserve the event IR as the boundary between parsing, transformation, and rendering.
3. Keep plugin-specific semantics inside the owning plugin.
4. Make document state explicit and scoped to one document.
5. Keep direct CLI and config execution behaviorally equivalent where their options are equivalent.
6. Validate behavior with focused tests before broad refactors.
7. Avoid unrelated formatting, generated-file churn, and speculative abstractions.

## Repository Structure

- `crates/core`: events, parser primitives, options, and pipeline abstractions.
- `crates/extra`: shared cross-plugin syntax and support primitives.
- `crates/plugin-*`: document transforms and syntax-specific behavior.
- `crates/renderer-*`: output renderers; Solid output is the JSX/TSX contract.
- `apps/cli`: direct CLI and config-driven document processing.
- `sandbox`: executable integration examples and generated output.
- `docs/spec`: language & renderer specifications and golden fixtures.

## Event IR Contract

Plugins consume and produce `Event` values. The normal structural form is:

```text
StartNode(kind)
Attribute(...)
Text(...) / child nodes
EndNode(kind)
```

Rules:

- Attributes must appear after their owning `StartNode` and before content.
- Every transformed node must preserve balanced start/end structure.
- Do not emit renderer-specific markup when a structured event or custom node is available.
- Preserve text unless the plugin explicitly owns the syntax being removed.
- Do not treat diagnostics as ordinary document content.
- Use `pendon_core::validate_events` for focused structural tests, but remember that legacy plugin ranges may require normalization before validation.

## Plugin Design

Every plugin should have a narrow responsibility and a clear phase:

- preprocessing: frontmatter or document metadata;
- block transformation: headings, tables, images, lists, blocks;
- inline transformation: links, citations, wiki links, inline custom syntax;
- finalization: document-wide state such as citation numbering and frontmatter updates.

A plugin must not silently run another plugin unless it receives the official inline pipeline. Nested content such as image captions and table cells must use the same enabled-plugin policy as the parent document.

When adding a new syntax:

1. Define the syntax owner.
2. Define the event or custom-node output.
3. Define behavior when the plugin is disabled.
4. Add parser and output tests.
5. Add at least one integration example if the syntax crosses plugin boundaries.

## Shared Support Code

Use `crates/extra` for functionality shared by multiple plugins, such as:

- class/id and property parsing;
- quote-aware CSV splitting;
- shared configuration representations;
- small syntax-independent helpers.

Do not add plugin-specific rules to `crates/extra`. For example, `extra` may parse `key: value`, but anchor owns the meaning of `target`, citation owns citation identity, and Vicado owns typed property conversion.

Do not reimplement:

- attribute block parsing;
- quote-aware comma splitting;
- HTML/JSX escaping;
- plugin-list parsing;
- custom import parsing;
- renderer hint merging.

Before adding a helper, search the workspace for an existing equivalent.

## Pipeline and Document Context

`Pipeline` is the compatibility API for stateless or legacy processors. New stateful work should use `ContextPipeline<C>` and `InlinePipeline<C>`.

The preferred shape is:

```rust
pipeline.add(|context: &mut DocumentContext, events: Vec<Event>| {
    // transform events using explicit document state
    events
});
```

Rules:

- State belongs to one document context, never to global mutable state.
- Citation numbering and reference collection belong to the document context.
- Finalization runs once after all citation-producing phases and nested content.
- Do not add a new `Arc<Mutex<_>>` merely to pass state into a closure.
- If a legacy API must remain, add a context-aware API beside it and migrate consumers incrementally.
- Do not remove compatibility APIs until all workspace consumers and sandboxes have migrated.

## JSX and Renderer Consistency

Solid output must be deterministic and consistent across built-in nodes and plugins.

- Use shared JSX attribute emission and escaping.
- Keep attribute names and ordering stable.
- Do not leak internal attributes such as custom-node `name` unless the template explicitly needs them.
- Preserve canonical identities. Extra IDs must use a namespaced attribute such as `data-cite-id` instead of overwriting semantic IDs.
- Prefer structured nodes and renderer hints over arbitrary raw HTML.
- Raw HTML is allowed only when the owning plugin explicitly produces an HTML block/inline contract.
- Custom templates must declare required imports through renderer hints.

When output changes intentionally, update the relevant golden or sandbox output and explain the contract change in the change description.

## Diagnostics and Errors

Diagnostics are document-level events and should be consistent across plugins. Do not use `eprintln!` deep inside a plugin for recoverable syntax problems.

When adding an error path:

- preserve the input location when possible;
- distinguish warning from error;
- preserve strict-mode escalation;
- include enough context to identify the plugin and syntax;
- add a regression test for both recovered and strict behavior.

Runtime failures such as unreadable files, invalid configuration, and renderer failures should remain runtime errors, not be silently converted into document text.

## CLI Runtime Rules

There are two entry paths:

- direct CLI invocation in `apps/cli/src/main.rs`;
- config-driven `run` processing in `apps/cli/src/process.rs`.

Keep their shared behavior in `apps/cli/src/plugins.rs` or a future `PluginRuntime`. Do not copy plugin loops into both files.

The config runner may additionally own cache, file discovery, task interpolation, custom registries, and renderer overrides. Those concerns should not leak into core plugin APIs.

When changing plugin order, verify:

- `micromatter` runs before frontmatter-dependent context construction;
- inline plugins run only when enabled;
- Markdown runs at the intended phase;
- citation finalization runs after all citation-producing transforms;
- quiz and other order-sensitive plugins retain their documented behavior.

## Testing and Validation

Before submitting a change:

```bash
cargo fmt --all -- --check
cargo test --workspace
git diff --check
```

For changes affecting CLI or generated output, run the relevant sandbox directly:

```bash
cd sandbox/custom && cargo run --bin pendon -- run -F
cd sandbox/syntect && cargo run --bin pendon -- run -F
cd sandbox/universal && cargo run --bin pendon -- run -F
```

Inspect generated files under `out/`. A successful process is not enough if the JSX/HTML structure is wrong.

Focused validation is preferred after each edit. Use the smallest relevant package test first, then widen to workspace and sandbox validation.

## Generated Files and User Changes

- Never revert unrelated user changes.
- Do not commit generated output merely because a command rewrote it unless the repository convention or task requires it.
- When generated output is part of an integration contract, inspect it and include only intentional changes.
- Do not commit build artifacts from `target/`.

## Documentation and Handoffs

Update documentation when architecture or public behavior changes.

For incomplete multi-step work, update the relevant file under `docs/todo/` with:

- completed work;
- current architecture;
- remaining tasks;
- known blockers;
- exact validation commands.

The active plugin refactor is tracked in [`docs/todo/plugins-refactor.md`](docs/todo/plugins-refactor.md).

## Pull Request Checklist

- [ ] The change has a focused test.
- [ ] Shared behavior was reused instead of duplicated.
- [ ] Direct CLI and config paths were considered.
- [ ] Plugin enablement and ordering are explicit.
- [ ] Event structure and JSX/HTML output were inspected.
- [ ] `cargo fmt --all -- --check` passes.
- [ ] Relevant package tests pass.
- [ ] `cargo test --workspace` passes for cross-cutting changes.
- [ ] Relevant sandbox output was verified.
- [ ] Documentation and handoff notes are updated when needed.
