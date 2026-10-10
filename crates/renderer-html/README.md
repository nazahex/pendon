# pendon-renderer-html

Renders an event stream to HTML (`--format html`), with a compact and a pretty
(indented, one child per line) mode.

## API

```rust
use pendon_renderer_html::{render_html, render_html_pretty};

let compact = render_html(&events);
let pretty  = render_html_pretty(&events);
```

Internally the renderer first builds the AST
(`pendon-renderer-ast`) and walks it, so the HTML always reflects the same
structure the AST renderer produces.

## Behaviour

- Built-in elements render as their HTML tags with attributes; a `Custom` node
  without a configured template falls back to the built-in element carrying all
  extras (no data loss — `SYNTAX.md` §6.3, §12.3).
- Bare flags render as bare attributes (`isFoo`); typed strings render
  `k="v"`; `Int`/`Float` render as their literal text.
- HTML passthrough is opt-in via `plugin-markdown`'s `allow_html` option.

## Usage

```bash
pendon --plugin markdown --format html --input ./doc.md
pendon --plugin markdown,syntect --format html --pretty --input ./doc.md
```

## License

MIT
