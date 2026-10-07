# pendon-plugin-section

`plugin-section` turns heading levels into a nested `Section` outline. It runs
**before** `plugin-markdown` (and before `plugin-heading`): the level markers
collide with Markdown — `>---<` is a blockquote — so the raw heading text, the
section decorator line and the markers are read before Markdown runs. Markdown
then parses the interior of every `Section` this plugin emits.

## Behavior

- Each heading opens a `Section` at its level; a deeper heading nests, an equal
  or shallower heading closes the open sections first.
- Content before the first heading is wrapped in a preface `Section` (level 0).
- **Section decorator.** A decorator line directly above a heading
  (`@@sectionA{…}` / `{…}`, optional indentation, blank line allowed) decorates
  the **section**, not the heading — the heading already owns its extras on its
  `#` run. The decorator is bound through `pendon-extra` like the other block
  binders.
- **ID priority.** The section id is the first available of `#sectionID`
  (decorator `#id`) > `` `slug-section` `` (decorator slug) > `` `slug-head` ``
  (heading `[slug]`) > `` `slug-head-extras` `` (heading extras slug) > the slug
  of the heading title. A heading's own extras `#id` is not part of the chain and
  is ignored with a warning. When `plugin-section` owns the outline the heading
  never emits an `id` or fallback `slug` (see `HeadingOptions::section_owns_id`).
- **Level markers.** A line that is exactly `<--->` deepens the outline by one
  nested section (capped at level 6); a line that is exactly `>---<` closes the
  innermost section. Both are consumed and never rendered.

## Usage

Typical pipeline (the order matters — `section` before `heading` before
`markdown`):

```bash
pendon --plugin micromatter,section,heading,markdown --format solid --input ./doc.md
```

Library:

```rust
use pendon_plugin_section::{process, SectionOptions};
let outlined = process(&events, &SectionOptions::default());
```

## Config

```toml
[task.section.custom.section]
name = "DocSection"
template = "<DocSection {...attrs}>{children}</DocSection>"
```

`section` is the plugin's only (primary) layer, so `[task.section.custom]` and
`[task.section.custom.section]` are equivalent. Without a component the built-in
`<section>` element is used.
