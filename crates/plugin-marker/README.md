# pendon-plugin-marker

Binds the **extras** a marker may carry to the marker node, before `plugin-markdown` parses the Markdown (`docs/spec/SYNTAX.md` §10.1).

This plugin owns the two spellings a marker may use:

```text
{{sectionA}[intro]("title"){.lead}}   ← §10.1 marker block (own line)
{{sectionA}[intro]("title")}           ← marker block without body
text {{sectionA}[intro]("title")} more ← marker inline (inside inline content)
```

When both a marker block and an inline marker of the same type appear, they are independent; each is bound separately.

## What it does

- Finds every marker (block or inline) that matches a known type.
- Parses the marker, including its positional groups (`[…]` / `(…)`) and its extras head (`{…}`).
- Resolves the groups and items through the marker type's `field.*` (§6.1) with
  `pendon_extra::to_attributes`, and reports the §13 diagnostics it resolves.
- Keeps the marker's own `type` as the routing key: an extras `@@type` is
  **silently** overridden (§10.1 — one node, one routing key, so no error and no
  warning), and `type` is the marker's only built-in field. It contracts no slot
  position, and its text is its children, never an `inner_text` attribute.
- Replaces the marker with a built-in element (`<span>` inline, `<div>` block) carrying its `type` and tagged `__layer = "marker"`; the renderer answers the layer with `[[task.marker.custom]]` (§11 rule 3, ADR-0008 `D-3a`), and with no component configured the element itself is the output.
- Marks the node `__plugin_kind = "inline"` for inline markers and `__plugin_kind = "block"` for block markers, so `plugin-markdown` re-lexes the stripped body appropriately (inline markers as inline content, block markers as block content).

## Options

```toml
[[task.marker.custom]]
type = ["sectionA"]
name = "SectionA"
imports = "import { SectionA } from '@comp/SectionA'"
template = "<SectionA {...attrs}>{children}</SectionA>"

[[task.marker.custom]] # answers every unclaimed type (§11 rule 3)
name = "MarkerDefault"
imports = "import { MarkerDefault } from '@comp/MarkerDefault'"
template = "<MarkerDefault {...attrs}>{children}</MarkerDefault>"
```

```rust
use pendon_plugin_marker::{process, solid_hints, MarkerOptions};

let events = pendon_plugin_marker::process(&parsed, &options);
let jsx = render_solid_with_hints(&markdown(&events), solid_hints(&options).as_ref());
```

`primary_layer()` is `marker` — the key `[[task.marker.custom]]` addresses.

### Specs (§6.1, D-5)

An entry renames the §6.1 positional slots through its `field.*` keys —
`field.bracket.bare` for the `[x]` group and `field.paren.bare` for `("x")`. An
entry that configures none keeps the §6.1 default paths (D-7). The table is in
[`docs/spec/SYNTAX.md` §6.1](../../docs/spec/SYNTAX.md).

## Notes

- The plugin runs **before** `plugin-markdown` (§12.1), like every other construct plugin.
- A marker inside a code fence or raw HTML is literal text (§4.3).
- Nesting is not special for markers; they are replaced in a single pass.
- Malformed positional groups (missing closing `]` or `)`) cause the marker to still render, but the group stays as literal text (§4.3).

## CLI wiring

This crate is wired into the CLI plugin list: `marker` builds its `MarkerOptions`
from `[[task.marker.custom]]` and dispatches `process` in the config runner
(`apps/cli/src/plugins.rs`, `apps/cli/src/process.rs`). Run `cargo test -p
pendon-plugin-marker` for crate-level and end-to-end coverage.
