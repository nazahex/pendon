# pendon-plugin-dialog

Dialog screenplay plugin for Pendon.

`pendon-plugin-dialog` transforms paragraph blocks that look like dialog scripts into semantic HTML dialog lists using `<dl>`, `<dt>`, and `<dd>`.

This plugin is useful when writing story/script style content in Markdown without losing regular Markdown features.

## What It Does

For lines inside one paragraph block that follow `Speaker: content`, this plugin emits:

- `<dl>` wrapper per dialog block
- `<dt>` for speaker name
- `<dd>` for speaker content

Inside each content body, it supports these inline dialog markers:

- `"..."` -> `<q>...</q>`
- `_(...)_` or `*(...)*` -> `<i>(...)</i>`
- `\\` -> `<br />`
- plain fragments -> `<p>...</p>`

The plugin keeps Markdown formatting inside fragments (for example `*emphasis*`, `` `code` ``, links), because each fragment is re-rendered through Pendon Markdown processing.

## Plugin Order

Recommended order:

- `micromatter,dialog,markdown`

Reason:

- `dialog` needs frontmatter data produced by `micromatter` (for `charmap` classes).
- `dialog` emits HTML blocks that should pass through subsequent Markdown processing.

## Input Rules

A dialog line is considered valid when:

- The line is not empty.
- It contains `:` separator.
- The speaker part before `:` is not empty.

If a paragraph contains any non-empty line that fails those rules, the paragraph is not transformed and remains as normal Markdown paragraph output.

## Character Map From Frontmatter

Optional class mapping can be defined in frontmatter using `charmap` array pairs:

```yaml
---
charmap: ["Revan Juan", "a", "Stevano", "b"]
---
```

This produces:

- `<dt class="a">Revan Juan</dt>` and `<dd class="a">...`
- `<dt class="b">Stevano</dt>` and `<dd class="b">...`

Speakers not present in `charmap` are rendered without class.

## Example

Input:

```md
Revan Juan: "Consectetur eu minim *aute* deserunt." _(stage note)_
Stevano: *(whispering)* "Aliquip occaecat ipsum."\\...\\"Enim velit anim sunt qui mollit."
```

Output (simplified):

```html
<dl>
<dt>Revan Juan</dt> <dd><q>Consectetur eu minim <em>aute</em> deserunt.</q> <i>(stage note)</i></dd>
<dt>Stevano</dt> <dd><i>(whispering)</i> <q>Aliquip occaecat ipsum.</q><br /><p>...</p><br /><q>Enim velit anim sunt qui mollit.</q></dd>
</dl>
```

## Configuration Example

In `pendon.toml` task config:

```toml
[[task]]
name = "Dialog Demo (HTML)"
input = "./demo.md"
output = "./out/demo.html"
plugin = "micromatter,dialog,markdown"
format = "html"
pretty = true
markdown_allow_html = true
```

## Scope and Limitations

- Current parsing is paragraph-based: one contiguous paragraph can become one `<dl>` block.
- The plugin is intentionally strict for line shape to avoid accidental conversion of regular text.
- Quote parsing uses paired `"` in one line segment; unmatched quotes are treated as plain text.
- Line break marker for dialog body is double backslash sequence `\\`.

## Development Notes

- Crate entry point: `process(events: &[Event]) -> Vec<Event>`
- Internal modules are split by concern:
	- `pipeline.rs`: block detection and event transformation
	- `render.rs`: HTML rendering for `<dt>/<dd>` and dialog tokens
	- `tokenize.rs`: dialog tokenization (`Quote`, `Italic`, `Break`, `Plain`)
	- `charmap.rs`: frontmatter `charmap` extraction/parsing
	- `markdown.rs`: inline Markdown fragment rendering helper

## License

MIT
