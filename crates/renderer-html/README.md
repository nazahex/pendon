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
- A **fragment** value (`{"$fragment": [ … ]}` — a bound `mdp` payload,
  [ADR-0007](../../docs/decisions/0007-mdp-subdocuments.md)) is markup, not data:
  its nodes are rendered to HTML and escaped into the attribute value
  (`data="&lt;p&gt;…"`), in compact _and_ pretty mode. A fragment nested inside a
  structured value renders in place, so the surrounding JSON stays readable.
- HTML passthrough is opt-in via `plugin-markdown`'s `allow_html` option.
- A **`__`-prefixed** attribute (`__layer`, `__plugin_kind`, `__number`) is
  internal transport, never output (§11 rule 3, ADR-0005/ADR-0010). The generic
  attribute writer drops it, and so does `Link` — which writes `href`/`title`
  itself and then falls back to the same skip.

## Usage

```bash
pendon --plugin markdown --format html --input ./doc.md
pendon --plugin markdown,syntect --format html --pretty --input ./doc.md
```

## License

MIT
