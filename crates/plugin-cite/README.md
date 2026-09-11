# pendon-plugin-cite

Citation processing for Pendon. The plugin reads references from `micromatter` frontmatter or an external YAML file, replaces `[^^]` markers, and injects ready-to-use citation data into frontmatter.

## Syntax

```md
[^^]("suryana-2026")
[^^]("paper-smith", "hlm. 210-225")
[^^]("paper-smith", loc="hlm. 210", style="short")
```

The second positional argument is the conventional `loc` property. Additional properties are preserved in the citation object. Repeated citations receive a new index when any property other than `id` differs; identical citations reuse the same index.

## Task configuration

```toml
[[task]]
name = "Cite Demo (Solid)"
input = "./src/[slug].md"
output = "./out/[slug].jsx"
plugin = "micromatter,cite,markdown"
format = "solid"

[task.cite]
reference_source = "frontmatter" # frontmatter | internal | external
prefix = "citeref-"
class = "cite-ref"
id_prefix = "cra-"

[task.cite.custom_node]
name = "Citation"
template = "<Citation index={attrs.index} id={attrs.id} loc={attrs.loc} />"

[[task.cite.custom_node.imports]]
module = "@comp/citation"
default = "Citation"
```

## Citation sections

`plugin-cite` can also replace a configurable paragraph marker with a custom section node. This keeps footnotes, bibliographies, and reference lists fully under application control:

```toml
[task.cite.section]
marker = "{{ footnote }}"
node = "Bibliography"
template = "<Bibliography citations={attrs.cites} references={attrs.references} />"

[[task.cite.section.imports]]
module = "@comp/citation"
default = "Bibliography"
```

The section receives `cites` and `references` as JSON-valued JSX props. Multiple section markers can be used in one document, and each task may configure a different node, template, and component import.

`frontmatter` and `internal` use the `references` object supplied by `micromatter`. With `external`, `reference_file` is resolved against the current input captures. A `[chapter_id]` placeholder can be inferred from the first path segment captured by `[...slug]`.

Frontmatter references take precedence over external references with the same ID. Only cited external references are injected into the output `references` object.

The plugin adds a `cites` array to frontmatter. Default output is an inline HTML citation marker. When `custom_node` is configured, the plugin emits a custom node and the CLI adds the configured Solid renderer template/imports for that task only.
