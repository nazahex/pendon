# pendon-plugin-quiz

Transforms quiz blocks into a serialized `Quiz` custom node for a downstream
Solid component.

> **Note:** `plugin-quiz` is retired from the normative surface grammar
> ([`docs/spec/SYNTAX.md`](../../docs/spec/SYNTAX.md) §14, ADR-0001). It is still
> a workspace member and **is** dispatched by the CLI, but it is driven by a
> `custom_registry` spec (`toml:` plugin) rather than a first-class
> `[task.quiz]` options block, and a deep rework is planned. See
> [`STATUS.md`](../../STATUS.md).

## What it does

- Detects the quiz block, extracts choices and feedback, and marks the correct
  choice(s).
- Emits a `Custom("Quiz")` node with a serialized JSON payload (`Choice`,
  `Feedback`, …) plus renderer hints (`ComponentTemplate`, `ImportEntry`,
  `SolidRenderHints`) so the Solid renderer can emit the component.

## Pipeline behaviour

The plugin runs **after** `markdown` (so the block already exists as structured
events). In the CLI it is keyed off a `custom_registry` spec named `quiz`; when
`markdown` has not run yet the transform is deferred until it has
(`apps/cli/src/process.rs`, `apps/cli/src/main.rs`).

## API

```rust
use pendon_plugin_quiz::{process, solid_hints};

let events = pendon_plugin_quiz::process(&parsed);
let hints = pendon_plugin_quiz::solid_hints(); // SolidRenderHints for the Quiz component
```

- `process(events: &[Event]) -> Vec<Event>` — the transform.
- `solid_hints()` — the renderer hints (component template + imports).

## License

MIT
