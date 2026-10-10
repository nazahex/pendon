# ADR-0002 — Decorator binder: a wrapper node carrying `__plugin_kind`

- Status: **Accepted**
- Spec: [`docs/spec/SYNTAX.md`](../spec/SYNTAX.md) §9.1

## Context

§9.1 says a decorator line "decorates the **next block node**", §9.3 gives lists
three layers (`unordered` = `<ul>`, `ordered` = `<ol>`, `list` = `<li>`) and §9.4
mirrors them in config. The spec did not fix **how** a pre-Markdown plugin
attaches attributes to a node `plugin-markdown` is about to build.

## Decision

The binder lives in `crates/extra/src/decorator.rs` (extras glue, next to
`bind.rs`); the plugins pass a `target` callback. A decorator binds via a
**wrapper node carrying `__plugin_kind = <layer>`** (`blockquote`, `unordered`,
`ordered`, `list`), whose attributes are merged onto the block `plugin-markdown`
emits for the body. The plugin owns _what_ the decorator attaches to.

- `parse_decorator_line` recognises a decorator line (its entire content is a head;
  §4.3 keeps the rest literal). `bind_decorators` binds each line to the next
  block node; the run's **last** line wins (earlier ones drop as `Overridden`), a
  decorator indented deeper than the block stays literal, a decorator with no
  following block drops as `NoFollowingBlock`, and the core-wrapped paragraph is
  consumed with it.
- A paragraph whose leading run is a decorator is split off first
  (`split_leading_decorator_paragraphs`) so the touching and blank-line spellings
  behave the same.

## Rejected

A `__plugin_kind = block` variant carrying the target kind — it duplicates what the
node's own element already says.

## Container placement

`unordered` / `ordered` are a **container** placement in `plugin-markdown`: the
wrapper becomes the container of the list built inside it (`plugin-list` L1 is
complete end-to-end). The `start` offset of an ordered list is preserved, a nested
list opens its own element, a decorator in front of non-list content keeps its node
(extras never dropped), and a decorated list never leaks into the next.

## Evidence

12 binder unit tests; `plugin-blockquote` (8 unit + 4 e2e), `plugin-list` (6 unit +
7 pipeline). Golden `10-decorator-blocks`, `11-blockquote`, `12-list-container`.
