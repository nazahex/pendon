# pendon-plugin-cite

Citation and bibliography plugin for Pendon.

`pendon-plugin-cite` transforms inline citation syntax into structured citation
nodes, numbers them across the entire document (including sub-pipelines like
image captions), and exports the citation list plus the references selected by
`references_scope` as typed JS bindings. It supports custom Solid components,
extra HTML attributes, and external reference files.

## What This Plugin Does

- Parses the tag-and-slots head `[^reference_id]` / `[^reference_id "location"]`
  (§7.3 of `docs/spec/SYNTAX.md`), including every other slot position as an
  ordinary field
- Maintains a **shared citation context** so citations in image captions, blockquotes, and other sub-pipelines share the same numbering
- Emits `export const references = {…}` and `export const citations = […]` in Solid/JS output; frontmatter is left untouched
- Supports custom Solid components for citations
- Supports extra classes, IDs, data attributes, and CSS custom properties on each citation
- Loads references from frontmatter YAML or external YAML files
- Filters the `references` export with `references_scope` (`cited` / `all`)
- Assigns `citation_number` and deduplicates it by `[task.cite] stable_fields`
  (default `["reference_id"]`; `[]` gives every occurrence its own number)
- Keeps the processor's one input (`reference_id`) independent of where the head
  puts it and of how it is published (`field.*`, D-4)
- Emits diagnostics when a referenced ID is not found
- Declares its slot contract and preset assignment as data (`pub const HEAD`,
  `crates/extra`'s `ElementHead`) — the single copy of §7.3's
  `bracket.bare → reference_id` mapping, plus `citation_number` as the one field
  the processor **owns** (D-12 rule 5)

## Recommended Plugin Order

```text
frontmatter,image,cite,wiki,anchor,markdown
```

**Why this order matters:**
`

1. `frontmatter` **must** run first — it parses the `---` delimited frontmatter block into a `Frontmatter` node that cite needs to read references from
2. `img` runs before cite so image captions can use the shared citation context
3. `cite` runs before `wiki` and `anchor` because `[^…]` is more specific than `[text](url)` or `[[wiki]]`
4. `markdown` runs last to handle remaining inline formatting

Example task config:

```toml
[[task]]
input = "./src/[slug].md"
output = "./out/[slug].jsx"
plugin = "frontmatter,image,cite,wiki,anchor,markdown"
format = "solid"
```

## Citation Syntax

### Basic Citation

```md
According to recent research [^suryana-2026], software engineering is evolving.
```

### Citation with Location

A location is an ordinary field: it sits in whichever position the document names
for it (`field.bracket.quote = "locator"` in golden `35-cite-fields`, which also
lists it in `stable_fields`), and the bracket's quote keeps it as `bracket.quote`
when nothing is configured.

```md
As noted in the literature [^suryana-2026 "hlm. 45"], this trend is accelerating.
```

### Citation with Extra Attributes

```md
See [^suryana-2026 "hlm. 123"](paperback){.highlight, .urgent, #my-cite, foo: "bar", --color: "red"}
```

The citation head is the bracket, whose tag is the caret: `[^reference_id]`.
`reference_id` is **mandatory**; every other position (the bracket's quote and
tick, the whole paren) is an ordinary field of the node. There is no built-in
location field — the form of one varies per author, so a document names it itself
(see [Stable Fields](#stable-fields-73)). The extras head that may follow the
bracket uses the shared `{…}` syntax:

- `.class` — appended to the class list; `#id` — the extra ID
- `key: "value"` — extra props; `--css-var: "value"` — CSS custom properties
- Keys starting with `--` become inline style entries
- Values can be quoted or unquoted

The extras head must touch the closing `]` (or `)`, when the paren is present).
A head that assigns no `reference_id` — `[^]`, `[^"ref"]`, or an entry whose
`field.*` voided the preset without redefining it — is an **`Error`** and the
text stays as written (§7.3). The retired `[^^](ref "loc")` spelling is not a
citation either: its second caret becomes the reference (§14).

`class`, `id` and `style` are the universal DOM fields (§6.3), so they keep those
names everywhere; the tick is a field of the author's (`specs.tick`), never a
slug or an id. How the remaining keys are emitted depends on the render target:

- **Default HTML** (no custom component): non-`--` keys become `data-{key}`
  attributes, and a dotted path is hyphenated (`data-specs-tick`).
- **Custom component** (`[task.cite.custom]`): keys are passed through verbatim as
  component props (`key={...}`), since Solid components expect plain props
  without a `data-` prefix.

### The Node's Fields

A citation node carries the preset field plus the number the processor owns:

| Field             | Meaning                                                                  |
| :---------------- | :----------------------------------------------------------------------- |
| `reference_id`    | The reference the head names. Mandatory (§7.3).                          |
| `citation_number` | Display ordinal `[1], [2]`, deduplicated by `stable_fields` (see below). |

```md
First mention [^book] and second mention [^book] both get [1],
but they are anchors `cite-1` and `cite-2`.
Another reference [^paper] gets [2].
```

Every other position is the author's own field (`bracket.quote`, `paren.tick`, …),
so a location is whatever the document calls it — `plugin-cite` has no opinion.

`reference_id` is not fixed either: `field.bracket.tick = "reference_id"` moves it
(§6.1), and `field.override.reference_id = "ref_id"` renames it publicly (D-4). The
processor reads the name, so numbering is unaffected by either — see golden
`35-cite-fields`.

## Stable Fields (§7.3)

`citation_number` is stable for a repeated **identity**, and the identity is
whatever `[task.cite] stable_fields` lists — public field paths, in the order they
are compared:

```toml
[task.cite]
stable_fields = ["reference_id", "locator_type", "locator_value", "author"]

[[task.cite.custom]]
type = ["citeX"]
name = "Cite"
imports = "import { Cite } from '@comp/Cite'"
template = "<Cite reference={references["{attrs.ref_id}"]} {...attrs} />"

field.bracket.bare = "reference_id" # one slot key voids the preset: re-declare it
field.paren.bare = "locator_type"
field.paren.quote = "locator_value"
field.specs.quote = "author"        # the extras `"Smith"`
field.override.reference_id = "ref_id"
```

With that config `[^book "p. 1"](paper "2001")@@citeX{"Smith"}` publishes
`reference_id`, `locator_type`, `locator_value` and `author`, and all four decide
its number:

- same four values -> same `citation_number`
- any of them different -> **different** `citation_number`
- a listed field the citation does not carry (say `citeY` has no `locator_value`)
  counts as absent, so two citations that both leave it out still match

The names are the paths **before** `field.override` (D-4), which is what the
processor reads. Both kinds of position are addressable: a head position
(`reference_id`, `bracket.quote`, `paren.tick`, or whatever `field.<slot>.<kind>`
renamed it to) and an extras position (`specs.quote`, or the name
`field.specs.<k>` gave it). The universal DOM fields `class`, `id` and `style` are
addressable too.

The defaults matter:

| `stable_fields`         | Meaning                                                  |
| :---------------------- | :------------------------------------------------------- |
| unset                   | `["reference_id"]` — one number per reference            |
| `[]`                    | nothing is stable — every occurrence gets a fresh number |
| `["reference_id", "…"]` | the listed values, in order                              |

A name that is not a field path (`.a`, `a..b`, `1x`) is a **config error**; golden
`48-cite-stable-fields` runs the example above end to end.

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

| Key                 | Default            | Description                                                  |
| :------------------ | :----------------- | :----------------------------------------------------------- |
| `reference_source`  | `frontmatter`      | `frontmatter` / `internal` / `external`                      |
| `reference_file`    | —                  | YAML file read when `reference_source = "external"`          |
| `references_export` | `references`       | Name of the exported reference map (must be a JS identifier) |
| `citations_export`  | `citations`        | Name of the exported citation list (must be a JS identifier) |
| `references_scope`  | see below          | `cited` / `all` — which references the export carries (§2.6) |
| `stable_fields`     | `["reference_id"]` | Field paths that make `citation_number` stable (§7.3)        |

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
spreads the remaining props typed (`citation_number={1}`, plus the extras and
any other slot field as the one-key spread
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
| `{attrs.citation_number}` | Display ordinal (1-based, deduplicated by `stable_fields`)           |
| `{attrs.bracket.tick}`    | Any other claimed position, under its (moved) field name (D-7)       |
| `{attrs.class}`           | Space-separated extra classes                                        |
| `{attrs.id}`              | Extra `#id` from the attribute block (universal, §6.3)               |
| `{attrs.<key>}`           | Extra prop passed verbatim (e.g. `{attrs.foo}` for `{ foo: "val" }`) |
| `{attrs.style}`           | Inline style string from `{ --var: "val" }`                          |

### Default HTML Output (No Custom Node)

Without a custom node, citations render as standard HTML:

```html
<sup class="highlight urgent" data-foo="bar" id="my-cite" style="--color:red;">
  <a href="#1-suryana-2026" id="cite-1" data-bracket-tick="p. 2">[1]</a>
</sup>
```

The markup ships no class and no `href` / `id` prefix (`citation_class`,
`ref_id_prefix` and `citation_id_prefix` are gone, §14): a site that wants its own
spelling configures a `Cite` component. `class` and `id` are the universal DOM
fields, so they appear on the `<sup>` only when the head supplied them, and the
`cite-<n>` backlink anchor is the markup's own, not a field.

The markup is fixed, but the head is not (§7.3, D-7): every position beyond
`reference_id` — `bracket.tick` in the example above — is written on the `<a>` as
`data-<slot>-<kind>`, because a dotted path is not an attribute name in JSX (§6.3)
and raw markup cannot spread one. `reference_id` stays the `href` and
`citation_number` the label; a `citations` row (§20) is that pair. A head
therefore never loses a value for want of a component (fixture
`37-head-slots-inline`).

## Shared Citation Context

This plugin uses a `CitationContext` that persists across the entire document
processing lifecycle. This means:

- Citations inside image captions (`plugin-image`) share the same numbering as
  citations in regular paragraphs
- Identity deduplication works globally — the same citation in a caption and a
  paragraph gets the same `citation_number`
- The context is built once per file, from the reference map the caller passes
  (§2.4: references come from `[task.cite] references`, never from frontmatter)

`CitationContext` is the plugin's only entry point — there is no free-function
shim (`process()` was removed with the frontmatter carrier, D-16):

```rust
let mut ctx = CitationContext::new(references, options);

// Process main document
let events = ctx.process_events(&events);

// Process sub-pipeline (e.g., caption) — numbering continues where the main doc left off
let caption_events = ctx.process_events(&caption_events);

// Finalize
let citations = ctx.get_citations();
let refs = ctx.get_exported_references(); // honours `references_scope` (§2.6)
let diagnostics = ctx.drain_diagnostics();
```

## Data Exports

Citation and reference data never touches frontmatter (§2.4). Solid/JS output
carries it as typed module-level exports, written after `frontmatter`/`headings`:

```jsx
export const references = {
  "suryana-2026": { "title": "...", "authors": [...] }
};

export const citations = [
  { "citation_number": 1, "reference_id": "suryana-2026" },
  { "citation_number": 2, "reference_id": "other-2021" }
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
`reference_id` assignment, or on diagnostics.

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
field.bracket.bare = "locator"        # …so a position the entry wants is named here
field.override.reference_id = "ref_id"
field.override.citation_number = "no"
field.override.locator = "loc"
```

One slot key voids the whole preset for that entry, so a position it leaves out
falls back to its qualified name (`bracket.bare`, `paren.bare`, …) and a head
that then carries no `reference_id` is an `Error` (§7.3). `field.override.*`
only renames the public field: the renderer applies it to the projection, and the
processor keeps reading `reference_id` (D-4).

The complete example is golden `35-cite-fields`; the unconfigured preset is
`05-cite`.

## Slot modifiers (§5.4, D-9)

The bracket's `|…` run contributes configured fields to the citation (§5.4,
[ADR-0017](../../docs/decisions/0017-slot-modifiers.md)). The run is **pure
configuration**, so it contributes last: a field the node already carries wins
with a `Warning`, `class` accumulates (§6.4) and a `true` contribution becomes a
bare attribute. The run never writes the reference, so citation identity and
numbering are unaffected. A symbol with no
`bracket.modifier."<symbol>"` config contributes nothing, and a config-only entry
(no `name` / `template`) is enough to define one (D-8) — the built-in `<sup>`
keeps rendering. Golden `43-slot-modifiers` is the end-to-end example.

## Behavioral Notes

- Citation syntax `[^…]` must appear as raw text — it is processed **before** markdown parsing
- Citations inside code fences, inline code, HTML blocks, and HTML inline elements are ignored
- Citations inside frontmatter blocks are ignored
- Unresolvable reference IDs emit an `Error` diagnostic and the raw syntax is preserved as-is
- Extra attributes are optional — citations work without them

## Scope and Limitations

- A head is a slot bracket (`[^id]`, `[^id "loc"]`), not a paren call: the
  retired `[^^](id)` form is literal text (§14)
- A location (`locator`) is not a field of its own: name a position for it
  (`field.bracket.quote = "locator"`) or use an extras prop
- Reference files must be valid YAML with string keys at the top level
- The plugin does not format bibliography entries — that responsibility belongs to the custom bibliography component

## License

MIT
