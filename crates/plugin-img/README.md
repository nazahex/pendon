# pendon-plugin-img

Advanced image syntax plugin for Pendon.

`pendon-plugin-img` adds an extended image syntax on top of normal Markdown image support. It is designed for cases where you need figure output, image behavior flags, dimensions, and custom attributes in a compact single-line form.

## What This Plugin Does

This plugin scans paragraph blocks and transforms matching advanced image lines into raw HTML blocks:

- Single image output: `<img ... />`
- Figure output: `<figure ...><img ... />...</figure>`
- Optional figcaption with inline Markdown rendering
- Optional custom id, classes, data attributes, and CSS custom property styles

## Recommended Plugin Order

Use:

- `img,markdown`

Reason:

- `img` should run first to consume advanced image lines before normal Markdown processing.
- `markdown` then handles the rest of the document as usual.

Example CLI:

```bash
pendon --plugin img,markdown --format html --input ./doc.md
```

Example task config:

```toml
[[task]]
input = "./src/[slug].md"
output = "./out/[slug].html"
plugin = "img,markdown"
format = "html"
pretty = true
```

## Syntax Overview

The core image shape is:

```text
<marker>[alt](src)<optional-attrs><optional-caption>
```

Where:

- `marker` must contain at least one `!`
- `alt` is the image alt text (can be empty)
- `src` is the image URL/path
- `optional-attrs` is an attribute block in the form `[...]{...}`
- `optional-caption` is allowed only for figure syntax

## Marker Rules

Supported marker parts:

- `!!` enables figure mode
- `?` adds `loading="lazy"`
- `~` adds `decoding="async"`
- `w<digits>` adds `width="..."`
- `h<digits>` adds `height="..."`

Marker order is flexible. Examples:

- `!!`
- `!?`
- `~?!!w320h180`
- `h600!!~`

## Supported Forms

### 1. Figure syntax

Use marker containing `!!`:

```md
!![Alt](https://example.com/image.webp) Caption with **bold** and [link](/x)
```

Output shape:

```html
<figure><img alt="Alt" src="https://example.com/image.webp" /><figcaption>Caption with <strong>bold</strong> and <a href="/x">link</a></figcaption></figure>
```

Figure may also use attribute block:

```md
~?!!h300w800[Alt](https://example.com/image.webp)[.hero,#cover]{foo: "bar", --rotate: "5deg"} Caption
```

### 2. Decorated single image with attribute block

```md
![Alt](https://example.com/image.webp)[.hero,#cover]{foo: "bar", --rotate: "5deg"}
```

Output shape:

```html
<img alt="Alt" id="cover" class="hero" data:foo="bar" style="--rotate:5deg;" src="https://example.com/image.webp" />
```

### 3. Single image with marker modifiers only

If marker has modifiers (`?`, `~`, `w...`, `h...`), attrs block is optional:

```md
!?w320h180[Alt](https://example.com/image.webp)
```

Output shape:

```html
<img width="320" height="180" loading="lazy" alt="Alt" src="https://example.com/image.webp" />
```

## Attribute Block Format

Attribute block has two parts:

```text
[.class1,.class2,#id]{key: "value", --var: "value"}
```

Class/id section:

- `.name` -> class list
- `#name` -> id
- Comma-separated

Key/value section:

- Keys starting with `--` become inline style entries
- Other keys become `data:<key>="value"`
- Comma-separated, quoted or unquoted values are accepted

## Behavioral Notes

- This plugin only transforms paragraph content that is plain text and fits a single line.
- Multi-line paragraphs are ignored by this plugin.
- Figure captions are rendered with inline Markdown support.
- Single-image mode does not accept trailing caption text.
- If input does not match advanced syntax rules, the original content is kept.

## Scope and Limitations

- The parser is intentionally strict for predictable output.
- Marker parser accepts only: `!`, `?`, `~`, `w<digits>`, `h<digits>`.
- Invalid marker combinations or malformed numbers are ignored (no transform).
- This plugin emits HTML blocks; downstream renderers should allow normal HTML handling.

## License

MIT
