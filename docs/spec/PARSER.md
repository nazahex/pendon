# Parser & IR Contract

The parser (`crates/core`) turns input text into a stream of `Event` values. This
is the boundary every plugin and renderer speaks. (This file is the short contract;
the surface syntax that feeds it is [`SYNTAX.md`](SYNTAX.md).)

## Event IR

```text
StartNode(kind)
Attribute(name, value) / AttributeFlag(name)
Text(...) / child nodes
EndNode(kind)
```

Rules (from `CONTRIBUTORS.md`):

- Attributes appear after their owning `StartNode` and before content.
- Every transformed node preserves balanced start/end structure.
- Preserve text unless the plugin explicitly owns the syntax being removed.
- Do not emit renderer-specific markup when a structured event or custom node is
  available.
- Do not treat diagnostics as ordinary document content.

## Behaviour & invariants

- Newlines are preserved as text (`"\n"`) for fidelity.
- Paragraphs open on the first non-blank line and close on a blank run ≥ 2.
- In strict mode, diagnostics become errors; the CLI exits non-zero if any errors
  occurred.

## Node kinds (representative)

`Paragraph`, `Heading`, `CodeFence`, `ThematicBreak`, `Blockquote`, `BulletList`,
`OrderedList`, `ListItem`, `Table`, `TableHead`, `TableBody`, `TableRow`, `Cell`,
`Custom(name)` (a configured component), `Element(name)` (a built-in HTML element),
plus `HtmlBlock` / `HtmlInline`. The `__plugin_kind` attribute on a `Custom` /
`Element` node tells `plugin-markdown` how to re-lex its body.

## Value typing

Extras map to `AttrValue { Str, Int, Float, Bool, Raw }` (`crates/extra`), then to
`Event::Attribute { name, value: String }`; bare flags emit `Event::AttributeFlag`.
See `SYNTAX.md` §6.3 for the full mapping and per-renderer behaviour.

## Not in this contract

- Setext headings (never existed; `---` is micromatter / `<hr />` only).
- The retired legacy `[.class,#id]{k:v}` extras form.
