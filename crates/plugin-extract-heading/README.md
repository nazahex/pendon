# pendon-plugin-extract-heading

Builds a nested outline of document headings and injects it as a `Custom("Headings")` node with a `data` JSON attribute. Use it to power TOCs, sidebars, or SEO metadata without re-scanning the document.

## Output

- Emits a single `Headings` custom node under `Document` (right after frontmatter if present).
- The node carries `attrs.data` with JSON of `[{ id, text, level, subheadings: [...] }]`.
- Headings nest according to their levels; lower/equal levels close higher ones.
- If there are no headings, the original events are returned unchanged.

## ID and text rules

- Preferred ID sources (first match wins):
  1. Surrounding `Section` node `id` (works best when `section` runs earlier).
  2. Explicit heading `id` attribute.
  3. Inline `{#custom}` marker inside heading text.
  4. The heading component's auto-generated `slug` attribute.
  5. Slugified heading text.
- Heading text is stripped of inline `{#id}` markers before serialization.

## Custom heading components

`plugin-heading` emits `NodeKind::Heading` whether or not a
`[task.heading.custom]` template is configured — it tags the node
`__layer = "heading"` and the renderer picks the component (§11 rule 3,
ADR-0008 `D-3a`) — so the outline is built from the built-in node.

A `Custom(name)` node carrying a numeric `level` and no `__plugin_kind` marker is
still recognised as a heading: that is the identity fallback for a component
the heading plugin itself used to emit (e.g. `sandbox/heading`'s `<DocHeading>`),
and §13 `Diagnostic` warnings are skipped while looking ahead for it.

## Placement

- Inserted immediately after the opening `Document`. If a frontmatter block exists, insertion happens right after the block.
- Frontmatter itself is preserved in the stream; this plugin only adds the outline node.

## Usage

Recommended pipeline:

```bash
pendon --plugin micromatter,section,markdown,extract-heading --format json --input ./doc.md
```

Library:

```rust
use pendon_plugin_extract_heading::process;
let with_outline = process(&events);
```

## Notes

- Section IDs are reused so TOC links stay aligned with rendered anchors.
- Serialization uses `serde_json`; if serialization fails, events are passed through untouched.
- The injected `Headings` node contains only attributes, no text children; renderers can parse `attrs.data` as needed.
