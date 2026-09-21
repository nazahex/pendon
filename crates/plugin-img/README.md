# pendon-plugin-img

Advanced image syntax plugin for Pendon.

`pendon-plugin-img` extends standard Markdown image syntax with a compact single-line form that supports figure output, behavior flags, dimensions, custom attributes, and rich figcaptions. It integrates with the shared inline pipeline so captions can contain wiki links, citations, and anchor-processed links.

## What This Plugin Does

- Transforms advanced image syntax into structured AST nodes or raw HTML blocks
- Supports three container modes: `<figure>`, `<div>`, `<p>`, or bare `<img>`
- Renders figcaptions with **full inline markdown** including bold, italic, links, wiki links, and citations
- Supports custom Solid components via `ImgCustomNode` configuration
- Accepts extra classes, IDs, data attributes, and CSS custom properties
- Provides image behavior flags: lazy loading, async decoding, explicit dimensions
- Integrates with the shared `Pipeline` so caption sub-parsing runs through wiki, cite, and anchor plugins

## Recommended Plugin Order

```text
micromatter,img,cite,wiki,anchor,markdown
```

**Why this order matters:**

1. `micromatter` must run first if frontmatter is present
2. `img` runs before markdown to consume advanced image lines before `[alt](src)` is converted to Link nodes
3. `cite`, `wiki`, and `anchor` are registered in the inline pipeline (not in the main plugin chain order) and process caption content internally
4. `markdown` runs last to handle remaining inline formatting outside of image blocks

Example task config:

```toml
[[task]]
input = "./src/[slug].md"
output = "./out/[slug].jsx"
plugin = "micromatter,img,cite,wiki,anchor,markdown"
format = "solid"
pretty = true
```

## Syntax Overview

The core image shape is:

```text
<marker>[alt](src)[.class,#id]{key: "val"} <optional-caption>
```

Where:

- `marker` must contain at least one `!`
- `alt` is the image alt text (can be empty)
- `src` is the image URL or path
- `[.class,#id]{key: "val"}` is an optional attribute block
- `caption` is trailing text allowed only in figure mode (`!!`)

## Marker Rules

Supported marker parts:

| Token       | Effect                     |
| :---------- | :------------------------- |
| `!!`        | Enables figure mode        |
| `p!`        | Wraps in `<p>` container   |
| `d!`        | Wraps in `<div>` container |
| `?`         | Adds `loading="lazy"`      |
| `~`         | Adds `decoding="async"`    |
| `w<digits>` | Adds `width="<digits>"`    |
| `h<digits>` | Adds `height="<digits>"`   |

Marker order is flexible. Examples:

- `!!` — basic figure
- `!?` — lazy-loaded bare image
- `~?!!w320h180` — async + lazy figure with dimensions
- `d~!w300h800` — async div-wrapped image with dimensions
- `h600!!~` — figure with height and async decoding

## Supported Forms

### 1. Figure Syntax

Use a marker containing `!!`:

```md
!![Alt](https://example.com/image.webp) Caption with **bold** and [link](/x)
```

Default HTML output:

```html
<figure>
  <img alt="Alt" src="https://example.com/image.webp" />
  <figcaption>Caption with <strong>bold</strong> and <a href="/x">link</a></figcaption>
</figure>
```

Figure with full attribute block:

```md
~?!!h300w800[Alt](https://example.com/image.webp)[.hero,#cover]{foo: "bar", --rotate: "5deg"} A rich caption with [[Wiki Link]] and [^^]("ref-id")
```

### 2. Decorated Single Image with Attributes

```md
![Alt](https://example.com/image.webp)[.hero,#cover]{foo: "bar", --rotate: "5deg"}
```

Default HTML output:

```html
<img alt="Alt" id="cover" class="hero" data-foo="bar" style="--rotate:5deg;" src="https://example.com/image.webp" />
```

### 3. Single Image with Marker Modifiers Only

If the marker has modifiers (`?`, `~`, `w...`, `h...`), the attribute block is optional:

```md
!?w320h180[Alt](https://example.com/image.webp)
```

Default HTML output:

```html
<img width="320" height="180" loading="lazy" alt="Alt" src="https://example.com/image.webp" />
```

### 4. Container-Wrapped Images

Wrap a non-figure image in a `<p>` or `<div>`:

```md
p![Alt](https://example.com/image.webp)[.wrapper]
d~!w300[Alt](https://example.com/image.webp){data-section: "hero"}
```

Output:

```html
<p class="wrapper"><img alt="Alt" src="https://example.com/image.webp" /></p>
<div data-section="hero"><img width="300" decoding="async" alt="Alt" src="https://example.com/image.webp" /></div>
```

## Attribute Block Format

The attribute block has two independent parts:

```text
[.class1,.class2,#id]{key: "value", --var: "value"}
```

**Class/ID section** (`[...]`):

- `.name` → added to class list
- `#name` → sets the `id` attribute
- Comma-separated, whitespace-tolerant

**Key/value section** (`{...}`):

- Keys starting with `--` → inline style entries (`style="--var:value;"`)
- All other keys → `data-{key}="value"` attributes
- Comma-separated, quoted or unquoted values accepted

Both sections are optional and can appear independently.

## Custom Solid Component

Replace the default HTML output with a custom Solid component:

```toml
[task.img.custom_node]
name = "AdvancedImage"
template = "<AdvancedImage id=\"{attrs.id}\" src=\"{attrs.src}\" alt=\"{attrs.alt}\" class=\"{attrs.class}\">{children}</AdvancedImage>"

[[task.img.custom_node.imports]]
module = "@/components/AdvancedImage"
default = "AdvancedImage"
```

### Available Template Attributes

| Attribute                | Description                                   |
| :----------------------- | :-------------------------------------------- |
| `{attrs.src}`            | Image source URL                              |
| `{attrs.alt}`            | Alt text                                      |
| `{attrs.container}`      | Container type: `"figure"`, `"p"`, or `"div"` |
| `{attrs.lazy}`           | `"1"` if lazy loading is enabled              |
| `{attrs.async_decoding}` | `"1"` if async decoding is enabled            |
| `{attrs.width}`          | Explicit width (string)                       |
| `{attrs.height}`         | Explicit height (string)                      |
| `{attrs.id}`             | Custom ID from `#id`                          |
| `{attrs.class}`          | Space-separated class list                    |
| `{attrs.data-*}`         | Any extra data attributes                     |
| `{attrs.style}`          | Inline style string                           |
| `{children}`             | Rendered caption content (inline JSX nodes)   |

### Caption as Children

When using a custom component, the figcaption is rendered as **inline JSX children** (not as an attribute). This means captions support full inline markdown processing through the shared pipeline:

```tsx
// AdvancedImage.tsx
export default function AdvancedImage(props) {
  return (
    <figure id={props.id} class={props.class}>
      <img
        src={props.src}
        alt={props.alt}
        loading={props.lazy === "1" ? "lazy" : undefined}
        decoding={props.async_decoding === "1" ? "async" : undefined}
        width={props.width}
        height={props.height}
      />
      {props.children && <figcaption>{props.children}</figcaption>}
    </figure>
  );
}
```

Boolean flags (`lazy`, `async_decoding`) are emitted as `"1"` strings. Cast them in your component as needed.

## Inline Pipeline Integration

Caption content is processed through the shared inline pipeline **before** markdown rendering. This means captions can contain:

- **Wiki links**: `[[Target | Label]]` → processed by `plugin-wiki`
- **Citations**: `[^^]("ref-id", "loc")` → processed by `plugin-cite` with shared index counter
- **Anchor links**: `[text](url^--$!)` → processed by `plugin-anchor` with modifier support
- **Standard markdown**: `**bold**`, `*italic*`, `` `code` `` → processed by `plugin-markdown`

The inline pipeline is built per-file in `main.rs` and injected into `plugin-img`. Plugin authors integrating with `pendon-plugin-img` should pass their own `Pipeline` instance:

```rust
let pipeline = Pipeline::new();
pipeline.add(move |ev| pendon_plugin_anchor::process(&ev, &anchor_opts));
pipeline.add(move |ev| pendon_plugin_wiki::process_with_options(&ev, wiki_opts.clone()));

let events = pendon_plugin_img::process(&events, &img_opts, &pipeline);
```

## Behavioral Notes

- This plugin only transforms paragraph content that is **plain text** and fits on a **single line**
- Multi-line paragraphs are ignored — the original content is preserved
- Figure captions support full inline markdown; single-image mode does not accept trailing caption text
- The marker parser is intentionally strict: invalid markers or malformed dimensions cause the line to be skipped (no transform, no error)
- When no custom node is configured, output is emitted as `HtmlBlock` events containing raw HTML strings
- When a custom node is configured, output is emitted as `Custom(name)` nodes with structured attributes and inline children

## Scope and Limitations

- Marker parser accepts only: `!`, `?`, `~`, `w<digits>`, `h<digits>`, `p`, `d`
- Invalid marker combinations (e.g., `p!!` mixing explicit container with figure) are rejected
- Attribute block parsing requires well-formed `[...]` and `{...}` blocks; malformed blocks are silently skipped
- Caption inline pipeline uses default options for wiki, cite, and anchor — per-task configuration for these plugins inside captions is not yet supported
- This plugin does not handle responsive images, srcset, or picture elements — use a custom component for those patterns

## License

MIT
