# pendon-plugin-sectionize

Wraps documents in semantic `Section` nodes driven by heading levels. It normalizes heading IDs, moves them onto their containing section, and keeps list/other containers from leaking across sections. Run this after Markdown so renderers can walk a clean outline.

## Behavior

- Opens a preface `Section` (level 0) before the first visible content; it stays open until the first heading closes it.
- Each heading opens a new `Section` at that heading level; lower/equal levels close previous sections first.
- ID handling:
  - Pulls `{#custom}` inline IDs or `id` attributes from the heading text/attrs.
  - Otherwise slugifies the heading text via `slugify`.
  - Ensures uniqueness with numeric suffixes (`foo`, `foo-2`, ...).
  - Moves the final ID onto the `Section` node; heading nodes are left without `id`.
- Frontmatter strip: leading frontmatter fences (`---` paragraph sandwich) are removed from the stream before sectionizing.
- List safety: closes any open list containers before starting a section to avoid malformed nesting.
- Code fences and frontmatter are ignored for section logic (no accidental section creation inside code/metadata).

## Usage

Typical pipeline:

```bash
pendon --plugin micomatter,markdown,sectionize --format ast --input ./doc.md
```

Library:

```rust
use pendon_plugin_sectionize::process;
let outlined = process(&events);
```

## Notes

- Level-1 headings start a section but allow level-2+ headings to nest underneath; same or higher levels close the previous section first.
- Content before the first heading remains inside the preface section.
- Works even if Markdown was not run, but heading-derived IDs rely on the incoming `Heading` nodes having text/level attributes.
