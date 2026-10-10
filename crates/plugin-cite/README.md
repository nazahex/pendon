# pendon-plugin-cite

Citation and bibliography plugin for Pendon.

`pendon-plugin-cite` transforms inline citation syntax into structured citation nodes, manages a global citation index across the entire document (including sub-pipelines like image captions), and generates bibliography metadata in the frontmatter. It supports custom Solid components, extra HTML attributes, and external reference files.

## What This Plugin Does

- Parses `[^^](ref-id)` and `[^^](ref-id "location")` inline citation syntax
- Maintains a **shared citation context** so citations in image captions, blockquotes, and other sub-pipelines share the same global index counter
- Injects `cites` and `references` arrays into the document frontmatter
- Supports custom Solid components for citations
- Supports extra classes, IDs, data attributes, and CSS custom properties on each citation
- Loads references from frontmatter YAML or external YAML files
- Deduplicates identical citations (same ID + same location = same index)
- Emits diagnostics when a referenced ID is not found

## Recommended Plugin Order

```text
micromatter,img,cite,wiki,anchor,markdown
```

**Why this order matters:**
`

1. `micromatter` **must** run first — it parses the `---` delimited frontmatter block into a `Frontmatter` node that cite needs to read references from
2. `img` runs before cite so image captions can use the shared citation context
3. `cite` runs before `wiki` and `anchor` because `[^^](...)` is more specific than `[text](url)` or `[[wiki]]`
4. `markdown` runs last to handle remaining inline formatting

Example task config:

```toml
[[task]]
input = "./src/[slug].md"
output = "./out/[slug].jsx"
plugin = "micromatter,img,cite,wiki,anchor,markdown"
format = "solid"
```

## Citation Syntax

### Basic Citation

```md
According to recent research [^^](suryana-2026), software engineering is evolving.
```

### Citation with Location

```md
As noted in the literature [^^](suryana-2026 "hlm. 45"), this trend is accelerating.
```

### Citation with Extra Attributes

```md
See [^^](suryana-2026 "hlm. 123"){.highlight, .urgent, #my-cite, foo: "bar", --color: "red"}
```

The citation head is the adjacent `[^^](ref "loc")` call: an **unquoted**
reference and an optional **quoted** location. The extras head that may follow it
uses the shared `{…}` syntax:

- `.class` — appended to the class list; `#id` — the extra ID
- `key: "value"` — extra props; `--css-var: "value"` — CSS custom properties
- Keys starting with `--` become inline style entries
- Values can be quoted or unquoted

The extras head must touch the closing `)` of the call. Retired spellings
(`[^^]("ref")`, `[^^](ref, "loc")`, `loc=…`, `[^^]()`) are not citations and stay
literal text.

How the remaining keys are emitted depends on the render target:

- **Default HTML** (no custom component): non-`--` keys become `data-{key}`
  attributes, and `#id` is emitted as `data-cite-id`.
- **Custom component** (`[task.cite.custom]`): keys are passed through verbatim as
  component props (`key={...}`), since Solid components expect plain props
  without a `data-` prefix. `#id` is emitted as `cite-id` so it stays distinct
  from the citation reference `id`.

### Duplicate Deduplication

Identical citations (same reference ID and same location) receive the same index:

```md
First mention [^^](book "p. 1") and second mention [^^](book "p. 1") both get [1].
Different location [^^](book "p. 2") gets [2].
```

## References Configuration

### Frontmatter References (Default)

Define references directly in the document frontmatter:

```yaml
---
title: "My Document"
references:
  suryana-2026:
    id: suryana-2026
    type: book
    title: Masa Depan Rekayasa Perangkat Lunak
    authors:
      - firstName: Eko
        lastName: Suryana
    publisher: TechPress Indonesia
    issuedDate:
      year: 2026
    isbn: 978-602-0000-00-0
---
```

### External Reference File

Load references from an external YAML file:

```toml
[task.cite]
reference_source = "external"
reference_file = "./references/[...slug].yaml"
```

The `reference_file` path supports the same capture group substitution as `input`/`output` patterns. When both frontmatter and external references exist, they are merged with **frontmatter taking precedence**.

## Custom Components

### Custom Citation Component

Replace the default `<sup><a>...</a></sup>` output with a custom Solid component:

```toml
[task.cite.custom]
name = "Citation"
imports = "import Citation from '@comp/citation';"
template = "<Citation index={attrs.index} id={attrs.id} loc={attrs.loc} class=\"{attrs.class}\" />"
```

Available template attributes:

| Attribute         | Description                                                          |
| :---------------- | :------------------------------------------------------------------- |
| `{attrs.index}`   | Global citation index (1-based)                                      |
| `{attrs.id}`      | Reference ID                                                         |
| `{attrs.loc}`     | Location string (if provided)                                        |
| `{attrs.class}`   | Space-separated extra classes                                        |
| `{attrs.cite-id}` | Extra `#id` from the attribute block                                 |
| `{attrs.<key>}`   | Extra prop passed verbatim (e.g. `{attrs.foo}` for `{ foo: "val" }`) |
| `{attrs.style}`   | Inline style string from `{ --var: "val" }`                          |

### Default HTML Output (No Custom Node)

Without a custom node, citations render as standard HTML:

```html
<sup class="cite-ref highlight urgent" data-foo="bar" style="--color:red;">
  <a href="#citeref-1-suryana-2026" id="cra-1">[1]</a>
</sup>
```

Extra classes are appended to the base `cite-ref` class. Data attributes and styles are placed on the `<sup>` wrapper.

## Shared Citation Context

This plugin uses a `CitationContext` that persists across the entire document processing lifecycle. This means:

- Citations inside image captions (`plugin-img`) share the same index counter as citations in regular paragraphs
- Identity deduplication works globally — the same citation in a caption and a paragraph gets the same index
- The context is built once per file after `micromatter` extracts frontmatter references

For plugin authors integrating with `pendon-plugin-cite`, use the `CitationContext` API instead of the legacy `process()` function:

```rust
let ctx = CitationContext::new(references, options);

// Process main document
events = ctx.process_events(&events);

// Process sub-pipeline (e.g., caption) — index continues from where main doc left off
caption_events = ctx.process_events(&caption_events);

// Finalize
let cites = ctx.get_cites();
let refs = ctx.get_used_references();
```

## Frontmatter Output

After processing, the plugin injects or updates the document frontmatter with:

```json
{
  "cites": [
    { "id": "suryana-2026", "index": 1 },
    { "id": "suryana-2026", "index": 2, "loc": "hlm. 45" }
  ],
  "references": {
    "suryana-2026": { "title": "...", "authors": [...], ... }
  }
}
```

- `cites` contains every unique citation in document order
- `references` contains only the references that were actually cited (not all references defined in frontmatter)
- When using external references, only cited external references are included in the output

### Positional keys (§6.1 / §11 rule 5)

An entry may rename the §6.1 positional slots in its extras head: `backtick_key`
(default `slug`) for a `` `slug` `` item and `quote_key` (default `title`) for a
`"title"` item. Keys the entry leaves unset keep the built-in default, so an
entry with no overrides behaves exactly as before.

## Behavioral Notes

- Citation syntax `[^^](...)` must appear as raw text — it is processed **before** markdown parsing
- Citations inside code fences, inline code, HTML blocks, and HTML inline elements are ignored
- Citations inside frontmatter blocks are ignored
- Unresolvable reference IDs emit an `Error` diagnostic and the raw syntax is preserved as-is
- Extra attributes are optional — citations work without them

## Scope and Limitations

- Only supports the `[^^](id)` and `[^^](id "loc")` citation forms
- Named parameters beyond `loc` are not supported (use extra attrs instead)
- Reference files must be valid YAML with string keys at the top level
- The plugin does not format bibliography entries — that responsibility belongs to the custom bibliography component

## License

MIT
