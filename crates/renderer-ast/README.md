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

## Relation to the projection

This crate _produces_ the `attrs` map that `pendon_extra::project` reads
(ADR-0014): the public field map is the identity projection of these attributes
minus the `__`-prefixed internals. That is why this renderer has no
attribute-emission path to reroute — its `attrs` output **is** the projection in
its AST form, and the identity is asserted in `pendon-extra`
(`the_projection_is_an_identity_pass_over_public_fields`).

## Usage

```bash
pendon --plugin markdown --format ast --input ./doc.md
pendon --plugin markdown --format ast --pretty --input ./doc.md
```

## License

MIT
