# pendon-renderer-solid

Renders an event stream to **Solid / JSX** — the canonical component output
contract (`--format solid`). This is the renderer custom `[task.<plugin>.custom]`
templates target.

## API

```rust
use pendon_renderer_solid::{render_solid, render_solid_with_hints, SolidRenderHints};

let jsx = render_solid(&events);

// with renderer hints (component templates + imports) from a plugin:
let hints: Option<&SolidRenderHints> = solid_hints(&options).as_ref();
let jsx = render_solid_with_hints(&events, hints);
```

## How it works

- **`components.rs`** — the `ComponentSet<T>` loader: typed entries plus at most
  one default per layer, selected per instance (exact `type` match → layer
  default → built-in element, §11 rule 3).
- **`node.rs`** — the built-in element renderers. `Paragraph`, `Blockquote`,
  `Heading`, `Section`, `CodeFence`, `BulletList` and `OrderedList` all call
  `render_attrs`, so extras reach the fallback element (no silent data loss,
  §12.3). `ListItem` (`<li>`) is the one gap (pending L2/L3, §9.3).
- **`template.rs`** — expands a component `template` (`{children}`, `{attrs.x}`,
  `{text}`) and validates it: a non-void element opened without `{children}` or
  `{text}` is a **hard error** at config load (§11 rule 6).
- **`imports.rs`** — emits deduplicated import lines from renderer hints.

## Attribute typing (§6.3)

| Item             | Solid output                                   |
| ---------------- | ---------------------------------------------- |
| bare flag        | `k` (bare JSX shorthand, verbatim)             |
| typed string     | `k="v"`                                        |
| `Int`/`Float`    | `k="12"` (fallback path only)                  |
| custom component | typed value via the `attrs` map (`12`, `true`) |

## Usage

```bash
pendon --plugin markdown --format solid --input ./doc.md
pendon --plugin anchor,markdown --format solid --pretty --input ./doc.md
```

## License

MIT
