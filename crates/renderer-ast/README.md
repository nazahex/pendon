# pendon-renderer-ast

Renders an event stream into a hierarchical JSON **AST** — the structured form
downstream transforms consume (`--format ast`).

## API

```rust
use pendon_renderer_ast::{render_ast_to_string, render_ast_to_string_pretty};

let compact = render_ast_to_string(&events)?;
let pretty  = render_ast_to_string_pretty(&events)?;
```

Both return `Result<String, serde_json::Error>`.

## Node shape

Each node serializes as an object with:

- `type` — the `NodeKind` name;
- `text` — present for leaf/text nodes;
- `attrs` — a map of attribute name → value (present only when non-empty);
- `children` — nested nodes.

Diagnostics are collected separately (severity + message) rather than as tree
children.

## Value typing

Extras reach the AST as strings from `Event::Attribute`; bare flags render as
`{ "name": "k", "kind": "flag" }`. Custom components receive the typed
`AttrValue` (see [`SYNTAX.md`](../../docs/spec/SYNTAX.md) §6.3).

## Usage

```bash
pendon --plugin markdown --format ast --input ./doc.md
pendon --plugin markdown --format ast --pretty --input ./doc.md
```

## License

MIT
