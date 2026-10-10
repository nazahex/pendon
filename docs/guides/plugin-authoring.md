# Plugin Authoring Guide

How to add or modify a plugin so it fits Pendon's contracts. The principles are in
[`../../CONTRIBUTORS.md`](../../CONTRIBUTORS.md); this is the working checklist.

## Anatomy

A plugin is a crate under `crates/plugin-*` (also add it to the workspace
`members` in `Cargo.toml`) that transforms an `Event` stream:

```text
pre-markdown (text → text, "protect")   OR   post-markdown (events → events, "bind")
```

Every plugin has a narrow responsibility and a clear phase:

- preprocessing — frontmatter / document metadata (`plugin-micromatter`);
- block transformation — headings, tables, images, lists, blocks;
- inline transformation — links, citations, wiki links, inline custom syntax;
- finalization — document-wide state (citation numbering, frontmatter updates).

## Adding a new syntax

1. **Define the syntax owner** — one plugin owns one syntax. Shared `@@`-head
   parsing lives in `crates/extra`; do not re-implement it.
2. **Define the output** — the `Event` / custom-node shape it emits.
3. **Define the disabled behaviour** — what happens when the plugin is off (usually
   the syntax stays literal text).
4. **Add tests** — parser + output tests; at least one integration example if the
   syntax crosses plugin boundaries (see [`testing.md`](testing.md)).
5. **Document** — a `crates/<plugin>/README.md` that references the spec section
   (`SYNTAX.md §N`) rather than restating the grammar.

## Shared support code (`crates/extra`)

Use `crates/extra` for functionality shared by multiple plugins (class/id/property
parsing, quote-aware CSV splitting, shared config representations). Do **not** add
plugin-specific rules there — e.g. `extra` may parse `key: value`, but anchor owns
the meaning of `target` and cite owns citation identity.

Do not re-implement: attribute block parsing, quote-aware comma splitting,
HTML/JSX escaping, plugin-list parsing, custom import parsing, renderer hint
merging.

## Custom components (layers)

A plugin that owns **layers** exposes a component set per layer via config
(`SYNTAX.md` §11). A single-layer plugin may address its primary layer directly:

```toml
[[task.anchor.custom]] # primary-layer shorthand
type = ["anchorA", "anchorB"]
name = "AnchorAB"
imports = ["import { AnchorAB } from '@comp/shared/Anchor'"]
template = "<AnchorAB type={{attrs.type}} {...attrs}>{children}</AnchorAB>"

[task.table.custom.table] # layered: one key per layer
```

Rules (see `SYNTAX.md` §11): unmatched type selection is exact `type` match → layer
default → built-in element (which MUST still carry all extras as attributes);
`imports` is canonical (singular `import` warns); `template` validation is a hard
error when children would be silently dropped or the element is unbalanced.

## Diagnostics

Diagnostics are document-level events, consistent across plugins. Do not use
`eprintln!` inside a plugin for recoverable syntax problems. Preserve the input
location, distinguish warning from error, preserve strict-mode escalation, and add
a regression test for both recovered and strict behaviour. Runtime failures
(unreadable files, invalid config, renderer failures) stay runtime errors — never
convert them silently into document text.

## Placement (`__plugin_kind`)

A `Custom` / `Element` node tells `plugin-markdown` how to treat its body through
the hidden `__plugin_kind` attribute the emitting plugin writes first:
`block` / `codefence` / `blockquote` re-lex as block content, `inline` as inline,
`element` (or nothing) passes the subtree through verbatim, and
`unordered` / `ordered` mark a list-container wrapper. This is why an unclaimed
block directive (`<div>`) renders the Markdown inside its body instead of leaking
raw text.
