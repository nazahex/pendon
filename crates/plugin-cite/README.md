# pendon-plugin-cite

Citation and bibliography plugin for Pendon.

`pendon-plugin-cite` transforms inline citation syntax into structured citation
nodes, numbers them across the entire document (including sub-pipelines like
image captions), and exports the citation list plus the references selected by
`references_scope` as typed JS bindings. It supports custom Solid components,
extra HTML attributes, and external reference files.

## What This Plugin Does

- Parses the tag-and-slots head `[^reference_id]` / `[^reference_id "locator"]`
  (§7.3 of `docs/spec/SYNTAX.md`), including every other slot position as an
  ordinary field
- Maintains a **shared citation context** so citations in image captions, blockquotes, and other sub-pipelines share the same numbering
- Emits `export const references = {…}` and `export const citations = […]` in Solid/JS output; frontmatter is left untouched
- Supports custom Solid components for citations
- Supports extra classes, IDs, data attributes, and CSS custom properties on each citation
- Loads references from frontmatter YAML or external YAML files
- Filters the `references` export with `references_scope` (`cited` / `all`)
- Assigns `citation_number` and deduplicates it by the `(reference_id, locator)`
  tuple only
- Keeps the processor's two inputs (`reference_id`, `locator`) independent of
  where the head puts them and of how they are published (`field.*`, D-4)
- Emits diagnostics when a referenced ID is not found
- Declares its slot contract and preset assignment as data (`pub const HEAD`,
  `crates/extra`'s `ElementHead`) — the single copy of §7.3's
  `bracket.bare → reference_id` / `bracket.quote → locator` mapping, plus
  `citation_number` as the one field the processor **owns** (D-12 rule 5)

## Recommended Plugin Order

```text
micromatter,image,cite,wiki,anchor,markdown
```

**Why this order matters:**
`

1. `micromatter` **must** run first — it parses the `---` delimited frontmatter block into a `Frontmatter` node that cite needs to read references from
2. `img` runs before cite so image captions can use the shared citation context
3. `cite` runs before `wiki` and `anchor` because `[^…]` is more specific than `[text](url)` or `[[wiki]]`
4. `markdown` runs last to handle remaining inline formatting

Example task config:

```toml
[[task]]
input = "./src/[slug].md"
output = "./out/[slug].jsx"
plugin = "micromatter,image,cite,wiki,anchor,markdown"
format = "solid"
```

## Citation Syntax

### Basic Citation

```md
According to recent research [^suryana-2026], software engineering is evolving.
```

### Citation with Location

```md
As noted in the literature [^suryana-2026 "hlm. 45"], this trend is accelerating.
```

### Citation with Extra Attributes

```md
See [^suryana-2026 "hlm. 123"](paperback){.highlight, .urgent, #my-cite, foo: "bar", --color: "red"}
```

The citation head is the bracket, whose tag is the caret: `[^reference_id "locator"]`.
`reference_id` is **mandatory** and `locator` optional; every other position (the
tick, the whole paren) is an ordinary field of the node. The extras head that may
follow it uses the shared `{…}` syntax:

- `.class` — appended to the class list; `#id` — the extra ID
- `key: "value"` — extra props; `--css-var: "value"` — CSS custom properties
- Keys starting with `--` become inline style entries
- Values can be quoted or unquoted

The extras head must touch the closing `]` (or `)`, when the paren is present).
A head that assigns no `reference_id` — `[^]`, `[^"ref"]`, or an entry whose
`field.*` voided the preset without redefining it — is an **`Error`** and the
text stays as written (§7.3). The retired `[^^](ref "loc")` spelling is not a
citation either: its second caret becomes the reference (§14).

How the remaining keys are emitted depends on the render target:

- **Default HTML** (no custom component): non-`--` keys become `data-{key}`
  attributes, and `#id` is emitted as `data-cite-id`.
- **Custom component** (`[task.cite.custom]`): keys are passed through verbatim as
  component props (`key={...}`), since Solid components expect plain props
  without a `data-` prefix. `#id` is emitted as `cite-id` so it stays distinct
  from the citation reference `id`.

### The Node's Fields

A citation node carries the preset fields plus the number the processor owns:

| Field             | Meaning                                                                          |
| :---------------- | :------------------------------------------------------------------------------- |
| `reference_id`    | The reference the head names. Mandatory (§7.3).                                  |
| `locator`         | The optional quoted location; part of the identity.                              |
| `citation_number` | Display ordinal `[1], [2]`, deduplicated by the `(reference_id, locator)` tuple. |

```md
First mention [^book "p. 1"] and second mention [^book "p. 1"] both get [1],
but they are anchors `cite-1` and `cite-2`.
Different location [^book "p. 2"] gets [2].
```

Deduplication follows academic practice (CSL, IEEE, Vancouver): the tuple is
`(reference_id, locator)`, never `reference_id` alone.

- same `reference_id` + same `locator` -> same `citation_number`
- same `reference_id` + different `locator` -> **different** `citation_number`
- `locator` present vs. absent -> **different** `citation_number`

Neither position is fixed: `field.bracket.tick = "reference_id"` moves it (§6.1),
and `field.override.reference_id = "ref_id"` renames it publicly (D-4). The
processor reads the two names, so numbering is unaffected by either — see golden
`35-cite-fields`.

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

### Options

| Key                 | Default       | Description                                                  |
| :------------------ | :------------ | :----------------------------------------------------------- |
| `reference_source`  | `frontmatter` | `frontmatter` / `internal` / `external`                      |
| `reference_file`    | —             | YAML file read when `reference_source = "external"`          |
| `references_export` | `references`  | Name of the exported reference map (must be a JS identifier) |
| `citations_export`  | `citations`   | Name of the exported citation list (must be a JS identifier) |
| `references_scope`  | see below     | `cited` / `all` — which references the export carries (§2.6) |

## Custom Components

### Custom Citation Component

Replace the default `<sup><a>...</a></sup>` output with a custom Solid component:

```toml
[task.cite.custom]
name = "Citation"
imports = "import Citation from '@comp/citation';"
template = "<Citation reference={references[\"{attrs.reference_id}\"]} {...attrs} />"
```

The `reference` prop binds the _resolved_ entry from the module-level reference
export (§2.3), so the component never has to look the id up itself; `{...attrs}`
spreads the remaining props typed (`citation_number={1}`, `locator="hlm. 45"`,
plus the extras and any other slot field as the one-key spread
`{...{ "bracket.tick": "…" }}`, §6.3). Omit the `template` key and the same shape is
used as a fallback, with the configured `references_export` name in place of
`references`.

Only `{children}`, `{text}`, `{attrs.<key>}` and `{...attrs}` are interpreted
(§6.3) — a template is a string substitution, never an expression evaluator.
A bare `{attrs.X}` interpolates raw text, so it is only safe inside a quoted
slot (`class="{attrs.class}"`) and empty slots are dropped unless the key is
`alt`; use `{...attrs}` when the component needs typed values.

Available template attributes:

| Attribute                 | Description                                                          |
| :------------------------ | :------------------------------------------------------------------- |
| `{attrs.reference_id}`    | The reference the head names (§7.3) — or its public override name    |
| `{attrs.locator}`         | Location string (if provided)                                        |
| `{attrs.citation_number}` | Display ordinal (1-based, deduplicated by `(reference_id, locator)`) |
| `{attrs.bracket.tick}`    | Any other claimed position, under its (moved) field name (D-7)       |
| `{attrs.class}`           | Space-separated extra classes                                        |
| `{attrs.cite-id}`         | Extra `#id` from the attribute block                                 |
| `{attrs.<key>}`           | Extra prop passed verbatim (e.g. `{attrs.foo}` for `{ foo: "val" }`) |
| `{attrs.style}`           | Inline style string from `{ --var: "val" }`                          |

### Default HTML Output (No Custom Node)

Without a custom node, citations render as standard HTML:

```html
<sup class="cite-ref highlight urgent" data-foo="bar" style="--color:red;">
  <a href="#citeref-1-suryana-2026" id="cite-1" data-bracket-tick="p. 2">[1]</a>
</sup>
```

Extra classes are appended to the base `cite-ref` class. Data attributes and styles are placed on the `<sup>` wrapper.

The markup is fixed, but the head is not (§7.3, D-7): a position beyond
`reference_id` / `locator` — `bracket.tick` in the example above — is written on
the `<a>` as `data-<slot>-<kind>`, because a dotted path is not an attribute name
in JSX (§6.3) and raw markup cannot spread one. `reference_id` stays the `href`,
`citation_number` the label, and `locator` the `citations` binding (§20). A head
therefore never loses a value for want of a component (fixture
`37-head-slots-inline`).

## Shared Citation Context

This plugin uses a `CitationContext` that persists across the entire document processing lifecycle. This means:

- Citations inside image captions (`plugin-image`) share the same numbering as citations in regular paragraphs
- Identity deduplication works globally — the same citation in a caption and a paragraph gets the same `citation_number`
- The context is built once per file after `micromatter` extracts frontmatter references

For plugin authors integrating with `pendon-plugin-cite`, use the `CitationContext` API instead of the legacy `process()` function:

```rust
let ctx = CitationContext::new(references, options);

// Process main document
events = ctx.process_events(&events);

// Process sub-pipeline (e.g., caption) — numbering continues where the main doc left off
caption_events = ctx.process_events(&caption_events);

// Finalize
let citations = ctx.get_citations();
let refs = ctx.get_exported_references(); // honours `references_scope` (§2.6)
```

## Data Exports

Citation and reference data never touches frontmatter (§2.4). Solid/JS output
carries it as typed module-level exports, written after `frontmatter`/`headings`:

```jsx
export const references = {
  "suryana-2026": { "title": "...", "authors": [...] }
};

export const citations = [
  { "reference_id": "suryana-2026", "citation_number": 1 },
  { "reference_id": "suryana-2026", "locator": "hlm. 45", "citation_number": 2 }
];
```

- `citations` holds **every physical occurrence** in document order
- `references` holds the entries selected by `references_scope` (see below)
- Both names are configurable with `references_export` / `citations_export`
  (values must be valid JavaScript identifiers); a template that binds the map
  must use the same name, e.g. `<Cite reference={refs_custom_var["{attrs.reference_id}"]} />`

### Reference Export Scope

`references_scope` decides how much of the reference map reaches the
`references` binding. It has no effect on `citations`, on `citation_number` /
`reference_id` / `locator` assignment, or on diagnostics.

| Value   | Export                                                                         |
| :------ | :----------------------------------------------------------------------------- |
| `cited` | Only the references the document actually cites                                |
| `all`   | Every known reference, cited or not (frontmatter ∪ external, frontmatter wins) |

**The default follows the reference source** when the key is omitted:

| `reference_source`         | Default |
| :------------------------- | :------ |
| `frontmatter` / `internal` | `all`   |
| `external`                 | `cited` |

Frontmatter references are authored for that specific document, so exporting all
of them is the natural default. An external catalog is often a large shared file,
and shipping its uncited entries would bloat every output. An explicit value
always wins, whichever source is configured:

```toml
[task.cite]
reference_source = "external"
reference_file = "./references/[...slug].yaml"
references_scope = "all" # export the whole catalog anyway
```

```toml
[task.cite]
reference_source = "frontmatter"
references_scope = "cited" # only cited frontmatter references
```

Any other value is a config error that names the two valid options.

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

### Slot positions (§7.3, D-7)

An entry configures where the head's contents land with `field.<slot>.<kind>`:

```toml
[[task.cite.custom]]
type = ["citeShort"]
name = "CiteShort"
imports = "import { CiteShort } from '@comp/CiteShort'"
template = "<CiteShort reference={references["{attrs.ref_id}"]} {...attrs} />"
field.bracket.tick = "reference_id"   # the preset is void as a whole (D-7)…
field.bracket.quote = "locator"       # …so both fields are redefined here
field.override.reference_id = "ref_id"
field.override.citation_number = "no"
field.override.locator = "loc"
```

One slot key voids the whole preset for that entry, so a position it leaves out
falls back to its qualified name (`bracket.bare`, `paren.bare`, …) and a head
that then carries no `reference_id` is an `Error` (§7.3). `field.override.*`
only renames the public field: the renderer applies it to the projection, and the
processor keeps reading `reference_id` / `locator` (D-4).

The complete example is golden `35-cite-fields`; the unconfigured preset is
`05-cite`.

## Behavioral Notes

- Citation syntax `[^…]` must appear as raw text — it is processed **before** markdown parsing
- Citations inside code fences, inline code, HTML blocks, and HTML inline elements are ignored
- Citations inside frontmatter blocks are ignored
- Unresolvable reference IDs emit an `Error` diagnostic and the raw syntax is preserved as-is
- Extra attributes are optional — citations work without them

## Scope and Limitations

- A head is a slot bracket (`[^id]`, `[^id "loc"]`), not a paren call: the
  retired `[^^](id)` form is literal text (§14)
- Named parameters beyond `locator` are not supported (use extra attrs instead)
- Reference files must be valid YAML with string keys at the top level
- The plugin does not format bibliography entries — that responsibility belongs to the custom bibliography component

## License

MIT
