# pendon-plugin-heading

Heading processing, auto-numbering, and custom component plugin for Pendon.

`pendon-plugin-heading` transforms heading nodes with support for hierarchical auto-numbering, custom IDs, extra HTML attributes, and custom Solid components. It handles documents that skip heading levels gracefully by suppressing leading zeros in numbered output.

## What This Plugin Does

- Extracts custom IDs from `[id]` prefix syntax in heading text
- Parses extra classes, IDs, and key-value attributes from `[.class]{key: val}` blocks
- Provides hierarchical auto-numbering with two styles: nested (`1.2.3.`) and flat (`3.`)
- Suppresses leading zeros when documents start below H1 (e.g., H2-first documents produce `1.` not `0.1.`)
- Auto-generates URL-safe slugs as fallback IDs when no custom ID is provided
- Supports custom Solid components with full attribute access including raw title and computed number
- Strips all extras syntax from visible heading text in both standard and custom modes

## Recommended Plugin Order

```text
markdown,heading,extract-heading
```

**Why this order matters:**

1. `markdown` must run first to convert raw `## Title` text into structured `Heading` nodes with `level` attributes
2. `heading` processes the structured heading nodes, extracting extras and applying numbering
3. `extract-heading` collects heading metadata for table of contents generation

Example task config:

```toml
[[task]]
input = "./src/[slug].md"
output = "./out/[slug].jsx"
plugin = "markdown,heading,extract-heading"
format = "solid"

[task.heading]
auto_number = true
number_style = "nested-number"
```

## Heading Extras Syntax

### Custom ID

```md
##[introduction] Introduction to the Topic
```

Produces `<h2 id="introduction">` (or custom component with `id="introduction"`).

### Classes Only

```md
##[.hero,.featured] Featured Section
```

Produces heading with `class="hero featured"`.

### Custom ID with Classes

```md
##[intro][.hero,.dark] Introduction
```

Produces heading with `id="intro"` and `class="hero dark"`.

### Key-Value Attributes

```md
##[intro]{ data-section: "overview", --accent: "blue" } Overview
```

Produces heading with `data-section="overview"` and `style="--accent:blue;"`.

### Full Combined Syntax

```md
##[intro][.hero,.dark]{ data-section: "overview", --accent: "blue" } Introduction
```

All three parts are independent and optional. Bracket groups and brace blocks can appear in any combination.

## Auto-Numbering

### Configuration

```toml
[task.heading]
auto_number = true
number_style = "nested-number" # or "flat"
```

### Nested Number Style (Default)

```md
## First Chapter → 1. First Chapter

### Section → 1.1. Section

### Another Section → 1.2. Another Section

## Second Chapter → 2. Second Chapter

### Subsection → 2.1. Subsection
```

### Flat Number Style

```md
## First Chapter → 1. First Chapter

### Section → 1. Section

### Another Section → 2. Another Section

## Second Chapter → 3. Second Chapter
```

### Leading Zero Suppression

Documents that skip H1 produce clean numbering automatically:

```md
## First Section → 1. First Section (not 0.1.)

### Subsection → 1.1. Subsection (not 0.1.1.)
```

This works for any starting level — an H3-first document produces `1.`, `1.1.`, etc.

## Custom Solid Component

Replace the default `<hN>` output with a custom Solid component:

```toml
[task.heading.custom_node]
name = "DocHeading"
template = "<DocHeading level={{attrs.level}} id=\"{attrs.id}\" number=\"{attrs.number}\" class=\"{attrs.class}\">{children}</DocHeading>"

[[task.heading.custom_node.imports]]
module = "@/components/DocHeading"
default = "DocHeading"
```

### Available Template Attributes

| Attribute           | Description                                                     |
| :------------------ | :-------------------------------------------------------------- |
| `{attrs.level}`     | Heading level as string (`"2"`, `"3"`, etc.)                    |
| `{attrs.id}`        | Custom ID from `[id]` syntax                                    |
| `{attrs.slug}`      | Auto-generated slug (only when no custom ID is provided)        |
| `{attrs.number}`    | Formatted number string without trailing space (`"1"`, `"1.2"`) |
| `{attrs.raw_title}` | Heading text without extras prefix or number                    |
| `{attrs.class}`     | Space-separated class list from `[.class]` blocks               |
| `{attrs.data-*}`    | Any extra data attributes from `{key: val}`                     |
| `{attrs.style}`     | Inline style string from `{--var: val}`                         |
| `{children}`        | Heading inline content (extras stripped, number NOT prepended)  |

### Number Handling in Custom Mode

In custom node mode, the number is **not** prepended to children text. It is available exclusively via `{attrs.number}` so the template has full control over placement and styling:

```tsx
// DocHeading.tsx
export default function DocHeading(props) {
  return (
    <div class={`heading heading-${props.level} ${props.class || ""}`} id={props.id}>
      {props.number && <span class="heading-number">{props.number}</span>}
      <span class="heading-title">{props.children}</span>
    </div>
  );
}
```

### Standard Mode Behavior

Without a custom node, the plugin emits standard `Heading` nodes with `level` and `id` attributes. The formatted number is prepended directly to the first text child. Extra attributes are emitted as node attributes for renderer consumption.

## Behavioral Notes

- This plugin processes **structured Heading nodes** — it must run after `plugin-markdown`
- Extras syntax (`[id][.class]{attrs}`) is stripped from visible text in all modes
- Multiple bracket groups are supported: `[id][.a][.b]` merges all classes
- Unquoted values in `{key: val}` are accepted (numbers, booleans stored as strings)
- Slug generation uses lowercase alphanumeric characters and hyphens only
- Empty headings (after extras stripping) still receive numbering and ID/slug
- The counter resets per document — each file processed independently

## Scope and Limitations

- Extras parsing is positional: `[id]` must come before `[.class]` which must come before `{attrs}`
- Nested braces inside `{key: val}` are not supported
- Comma-separated values inside attribute values are not supported (commas delimit pairs)
- The plugin does not generate anchor links or permalink icons — use a custom component for those patterns
- Auto-numbering state is not shared across files; each document starts from 1
- `raw_title` does not include inline formatting markers — it contains the plain text content after extras removal

## License

MIT
