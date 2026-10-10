# pendon-plugin-directive

Inline (`::type…::`) and block (`==type…==`) directives — the two construct
containers of [`docs/spec/SYNTAX.md`](../../docs/spec/SYNTAX.md) §10.2/§10.3.

A directive is a _construct container_: the head names a **mandatory** `type`
plus the usual `[…]` / `(…)` / `@@type{…}` extras, and the content between the
fences is parsed as inline (colon sigil) or block (equal sigil) content. This
plugin runs **before** `plugin-markdown` (§12.1).

## Syntax

```text
::type[…]("…"){…} content ::        ← §10.2 inline directive (2–7 colons)
==type[…]("…"){…}                    ← §10.3 block directive opener (2–7 equals)
  body
==                                   ← closes the innermost open directive (LIFO)
```

- The closing run MUST be `>=` the opening run (nesting depth = sigil count). An
  inner directive with fewer colons closes implicitly at the outer's close.
- A fence **alone** closes the innermost open directive; a bare fence with no
  directive open is literal text. There is no anonymous block directive — use a
  typed one (e.g. `==note`); its fallback renderer is `<div>`.
- Block content nests to 3 levels (`==`, `===`, `====`) and MAY nest deeper
  (limit 7). Unclosed at end of input → closed implicitly with a `Severity::Warning`.

## What it does

Turns every directive into a `Custom` node carrying:

- `__plugin_kind` = `inline` / `block`, so `plugin-markdown` re-lexes the content
  in the matching mode (§11 rule 3 placement);
- `type`, the routing key of the §11 component set (exactly like `plugin-marker`);
- `name`, the selected component (the node name the renderer matches on);
- every extra of the head (§6.3), flags included.

With no custom component, an unclaimed type renders `<span>` (inline) / `<div>`
(block), mirroring markers (§10, D8) — no extra is dropped.

## Options

`directive` is the plugin's primary layer, so `[[task.directive.custom]]`
addresses it:

```toml
[[task.directive.custom]]
type = ["note"]
name = "Note"
imports = "import { Note } from '@comp/content/Note'"
template = "<Note {...attrs}>{children}</Note>"

[[task.directive.custom]] # answers every unclaimed type (§11 rule 3)
name = "DirectiveDefault"
imports = "import { DirectiveDefault } from '@comp/content/Directive'"
template = "<DirectiveDefault {...attrs}>{children}</DirectiveDefault>"
```

`primary_layer()` is `directive`.

### Positional keys (§6.1 / §11 rule 5)

Directive heads use all four positional keys: `backtick_key` (default `slug`),
`quote_key` (default `title`), `bracket_key` (default `slug`) and
`parentheses_key` (default `title`). Keys an entry leaves unset keep the built-in
default.

## Rust API

```rust
use pendon_plugin_directive::{process, solid_hints, DirectiveOptions};

let events = pendon_plugin_directive::process(&parsed, &options);
let jsx = render_solid_with_hints(&markdown(&events), solid_hints(&options).as_ref());
```

## CLI wiring

Wired into the CLI plugin list: `directive` builds its `DirectiveOptions` from
`[[task.directive.custom]]` and dispatches `process` in the config runner
(`apps/cli/src/plugins.rs`, `apps/cli/src/process.rs`).

## Notes

- The plugin runs **before** `plugin-markdown` (§12.1). The block pass runs first
  so a block body is collected whole; the inline pass then walks the result, so
  `::…::` inside a block body is bound too.
- `:::name` is a deprecated alias kept for one release (Warning) — §15 OPEN-R1.

## License

MIT
