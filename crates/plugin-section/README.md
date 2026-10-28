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
- **A section has no head.** It declares no `ElementHead` and contracts no slot
  position — there is no `[bracket]` for it to claim. Its two own fields are the
  ones it derives (§9.5): `id` (the chain below) and `depth` (the outline level it
  stands for — the heading's own `depth`, a `<--->` child's parent + 1, `0` for
  the preface). Both are public fields, so a component receives them as props and
  the built-in `<section>` prints them as attributes. Every section carries the
  `section` layer tag (§11 rule 3), the preface included.
- **Section decorator.** A decorator line directly above a heading
  (`@@sectionA[intro]("title"){.x}` / `@@[intro]("title"){.x}` / `{…}`, optional
  indentation, blank line allowed) decorates the **section**, not the heading —
  the heading already owns its extras on its `#` run. The decorator is bound
  through `pendon-extra` like the other block binders, and may carry positional
  groups (`[…]` / `(…)`) that are resolved through the section type's
  `field.bracket.bare` / `field.paren.bare` (§6.1).
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

### Specs (§6.1, D-5)

An entry renames the §6.1 positional slots through its `field.*` keys —
`field.specs.tick` / `field.specs.quote` for a `` `slug` `` / `"title"` item, and
`field.specs.class` / `.id` / `.style` for a `.x` / `#x` / `style:` item. An
entry that configures none keeps the §6.1 default paths;
once it configures one the preset is void as a whole (D-7). An assignment also
**moves** the position: the assigned path is the author's own public field,
which this element neither reads nor renames (`Specs::element_path` answers
`None`, §6.1). The table is in
[`docs/spec/SYNTAX.md` §6.1](../../docs/spec/SYNTAX.md).
