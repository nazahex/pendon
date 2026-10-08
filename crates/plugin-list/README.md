# pendon-plugin-list

Binds the **extras** a list may carry to the list layers, before
`plugin-markdown` parses the Markdown (`docs/spec/SYNTAX.md` §9.3, §9.4).

Three layers, mirroring `[[task.list.<layer>.custom]]`:

| Layer       | Element          | Matched by (§9.3)                                |
| ----------- | ---------------- | ------------------------------------------------ |
| `unordered` | `<ul>` container | a container decorator `type`, marker `-`/`*`/`+` |
| `ordered`   | `<ol>` container | a container decorator `type`, marker `1.`        |
| `list`      | `<li>` item      | an item decorator `type` (L2/L3)                 |

## What it does

**L1, the container decorator**: a decorator line directly above a list
(`@@type{…}` / `@@{…}`, §9.1) decorates the container.

```text
@@unorderedA{.u, #l03}

- one          ← the marker lines stay untouched for `plugin-markdown`
- two
```

The plugin wraps the list's events in the container node —
`NodeKind::Custom(name)` when the `type` is claimed (§11 rule 3),
`NodeKind::Element("ul"|"ol")` otherwise (D8) — carrying
`__plugin_kind = "unordered" | "ordered"`, the extras, the `type` marker and the
`name`. The layer follows the **marker** (`-` → `unordered`, `1.` → `ordered`),
never the decorator's own type (§9.4). The decorators that bind to nothing are
reported as `Severity::Warning` (§9.1).

`plugin-markdown` reads that wrapper as a _container_ and builds the list
**inside** it, so the extras land on the `<ul>` / `<ol>` — or on the component
that replaces it — instead of around it:

```jsx
<UL class={"u"} type={"unorderedA"}><li>one</li><li>two</li></UL>
<ul class="u"><li>one</li></ul>             <!-- no component: the D8 element -->
<OL class={"o"} start={6}><li>Goo</li></OL> <!-- the start offset survives -->
```

A wrapper whose body turns out not to be a list stays an ordinary block node, so
a decorator's extras are never silently dropped. (The element fallback also
renders the internal `__plugin_kind` marker as an attribute; that §11 rule 3 leak
is older than this crate and is carried by the frozen CLI baselines.)

## What is not implemented yet

- **L2/L3, the item layer** (`list` / `<li>`): an item-decorator at the start of
  an item's content, and a decorator line used as an item's second line.
- **L4** the `start` offset rule: the Markdown pass reads it from the first item's
  number and the container merge keeps it; extras still cannot override it, but
  the rejection warning the spec asks for is not emitted yet.
- **L5** nesting: `plugin-markdown` parses the nesting, and a nested list's own
  decorator has to be indented to the nested list's level — which the plugin's
  §9.1 indentation check already accepts; there is no test for it yet.

## Options

```toml
[[task.list.unordered.custom]]
type = ["unorderedA"]
name = "UL"
imports = "import { UL } from '@comp/content/List'"
template = "<UL {...attrs}>{children}</UL>"

[task.list.ordered.custom] # single form = the layer default
name = "OL"
imports = "import { OL } from '@comp/content/List'"
template = "<OL {...attrs}>{children}</OL>"

[[task.list.list.custom]] # the item layer (§9.4)
name = "LI"
imports = "import { LI } from '@comp/content/List'"
template = "<LI {...attrs}>{children}</LI>"
```

`primary_layer()` is `list`; `layers()` is `["list", "unordered", "ordered"]`.

### Positional keys (§6.1 / §11 rule 5)

An entry may rename the §6.1 positional slots in its extras head: `backtick_key`
(default `slug`) for a `` `slug` `` item and `quote_key` (default `title`) for a
`"title"` item. Keys the entry leaves unset keep the built-in default, so an
entry with no overrides behaves exactly as before.

## Not yet wired

This crate is not part of the CLI plugin list yet: `apps/cli` still has to learn
`list` (options + dispatch + `solid_hints`), which another change owns — until
then the decorator line is literal text for the CLI. `cargo test -p
pendon-plugin-list` covers both levels: the crate (`src/lib.rs`) and the full
pipeline `core → plugin-list → plugin-markdown → solid` (`tests/pipeline.rs`).
