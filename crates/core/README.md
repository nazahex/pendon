# pendon-core

The parser core: it turns input text into Pendon's **event IR** — a flat stream
of `Event` values that plugins transform and renderers consume. Every plugin in
the workspace speaks this IR, so `core` is the boundary between parsing,
transformation and rendering.

The IR contract is documented in [`docs/spec/PARSER.md`](../../docs/spec/PARSER.md)
and the plugin rules in [`CONTRIBUTORS.md`](../../CONTRIBUTORS.md).

## Pipeline

```text
input ─▶ pendon_core::parse ─▶ Event stream ─▶ plugins (transform) ─▶ renderer ─▶ output
```

## What it provides

- **`parse(input, &Options) -> Vec<Event>`** — the entry point. Lexes the input
  and emits the structural event stream.
- **Event IR** — `StartNode(kind)` / `Attribute{name,value}` / `AttributeFlag{name}` /
  `Text(..)` / child nodes / `EndNode(kind)` / `Diagnostic{..}`. The normal
  structural form is `StartNode` → attributes → content → `EndNode`.
- **`NodeKind`** — `Document`, `Paragraph`, `Heading`, `CodeFence`, `ThematicBreak`,
  `Blockquote`, `BulletList`, `OrderedList`, `ListItem`, `Table`, `TableHead`,
  `TableBody`, `TableRow`, `Cell`, `HtmlBlock`, `HtmlInline`, `Custom(name)`,
  `Element(name)`, and more. `Custom(name)` is a configured component; `Element(name)`
  is a built-in HTML element.
- **`Pipeline` / `InlinePipeline` / `ContextPipeline<C>`** — small combinators
  plugins use to run nested transforms (e.g. an image caption re-running the
  inline plugins).
- **Heading helpers** — `slugify`, `extract_id`, `strip_trailing_id`,
  `ensure_unique`.
- **`validate_events(events) -> Vec<Event>`** — structural self-check used in
  focused tests (balanced start/end, attributes after their node). Note: legacy
  plugin ranges may require normalization before validation.

## Usage

```rust
use pendon_core::parse;

let events = parse("# Title\n\nText.", &Default::default());
// feed `events` to plugins, then to a renderer
```

## Design rules (see `CONTRIBUTORS.md`)

- The IR is the boundary: plugins produce and consume `Event` values, not
  renderer-specific markup.
- Attributes appear after their owning `StartNode` and before content.
- Every transformed node preserves balanced start/end structure.
- Preserve text unless a plugin explicitly owns the syntax it removes.

## License

MIT
