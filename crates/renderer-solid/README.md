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
  one default per layer, and `layer_templates`, which turns a layer's set into
  renderer hints tagged with the §11 layer they answer (§11 rule 3). The plugin
  registers **what each entry answers**; nothing is chosen here.
- **`resolve.rs`** — §11 resolution, decided **once per document**
  ([ADR-0008](../../docs/decisions/0008-type-selectors.md) `D-3a`). It groups the
  hints by layer, collects the `type` markers the AST occurs (fragments included)
  and asks `pendon_types::Group` for the winner of each occurring type, so `_`
  (remainder) and `*` (everything) see the whole document. Nodes are routed by
  the internal `__layer` attribute a plugin tags them with; a component that is
  not a §11 layer (`plugin-custom`, `plugin-quiz`, `plugin-vicado`) keeps the
  older node identity. Only winning entries contribute imports.
- **`node.rs`** — the built-in element renderers. `Paragraph`, `Blockquote`,
  `Heading`, `Section`, `CodeFence`, `BulletList` and `OrderedList` all call
  `render_attrs`, so extras reach the fallback element (no silent data loss,
  §12.3). `ListItem` (`<li>`) is the one gap (pending L2/L3, §9.3).
  `render_attrs` emits the node's **projection** — `pendon_extra::project(node,
  None).attrs()`, ADR-0014 rule 2 — rather than the raw `attrs` map. The map is
  the public vocabulary: `__`-prefixed internals are gone, `specs.*` is in it, and
  the universal positions are already renamed to `class`/`id`/`style` (§6.3).
  Phase 4 fills in each element's preset assignment. Per-element reads
  (`level` on a heading, `header` on a cell, the skip lists) still name their own
  field — Phase 4 moves them as the element's preset lands — while `name` is
  skipped everywhere because it is the `Element` node's tag.
- **`template.rs`** — expands a component `template` (`{children}`, `{attrs.x}`,
  `{text}`) and validates it: a non-void element opened without `{children}` or
  `{text}` is a **hard error** at config load (§11 rule 6). Both `{...attrs}`
  and `{attrs.k}` read the same projection, so a component sees public fields
  only.
- **`imports.rs`** — emits deduplicated import lines from renderer hints, and owns
  the module's `export const` lines (§20): `SolidRenderHints::exports` is an
  ordered list of `DataExport { name, value, source }`; `frontmatter`/`headings`
  are registered automatically from the AST first, then the hints in push order,
  and a repeated name keeps the **last** value and position. Only `solid` has a
  module scope, so a non-Solid task drops the list
  ([ADR-0009](../../docs/decisions/0009-generic-data-exports.md)).
- **`metadata.rs`** — the two automatic registrations. `frontmatter` is the JSON
  text `micromatter` put on its node; `headings` is the **nested** outline of
  every `Heading` node, built here from the AST (id chain: enclosing section →
  `id` → `slug` → slugified text; `subheadings` nested by level, omitted when
  empty, keys alphabetical). It used to be injected by `plugin-extract-heading`;
  the renderer now owns it, so no plugin has to run for the export to exist
  ([ADR-0011](../../docs/decisions/0011-plugin-extract.md), golden
  `32-extract-heading`).

## Attribute typing (§6.3)

| Item             | Solid output                                                                                       |
| ---------------- | -------------------------------------------------------------------------------------------------- |
| bare flag        | `k` (bare JSX shorthand, verbatim)                                                                 |
| typed string     | `k="v"`                                                                                            |
| `Int`/`Float`    | `k="12"` (fallback path only)                                                                      |
| custom component | typed value via the `attrs` map (`12`, `true`)                                                     |
| fragment         | `k={(<>…</>)}` — a bound `mdp` payload ([ADR-0007](../../docs/decisions/0007-mdp-subdocuments.md)) |

A fragment is a node list (`pendon_renderer_ast::FRAGMENT_KEY`), not a string, so
it is emitted as a JSX expression that keeps the markup parseable. Its nodes are
rendered exactly as page nodes are — including their own attrs, and their
components' imports: `collect_fragment_nodes` walks every fragment in the stream
so a payload's `~?!!…` figure still contributes its `Figure` import.

### Attribute names

The fields come from the projection, so a name is a **path**, not automatically a
JSX attribute name. `specs.class`/`specs.id`/`specs.style` are already renamed to
`class`/`id`/`style` by `pendon_extra::project` (§6.3), and a dotted path
(`specs.tick`, `bracket.bare`, an assigned `heading.anchor`) is emitted as a
one-key spread — `<p {...{ "specs.tick": "x" }}>` — because `a.b=` would parse as a
member expression. The path survives verbatim into the DOM, matching what
`renderer-html` prints and what JSON/AST keep.

## Usage

```bash
pendon --plugin markdown --format solid --input ./doc.md
pendon --plugin anchor,markdown --format solid --pretty --input ./doc.md
```

## License

MIT
