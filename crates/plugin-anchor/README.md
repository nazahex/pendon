# pendon-plugin-anchor

Link processing and external URL safety plugin for Pendon.

`pendon-plugin-anchor` processes standard Markdown link syntax (`[label](url)`) with support for URL suffix modifiers, extra attributes, automatic external link detection, and custom Solid components. It runs as an inline text processor before markdown parsing to intercept raw link patterns.

## What This Plugin Does

- Parses `[label](url)` link syntax from raw text events
- Automatically detects external URLs and applies `target="_blank"` with `rel="noopener"`
- Supports URL suffix modifiers for fine-grained control over link behavior
- Accepts extra attributes via `{key: "value"}` syntax after the link target
- Emits structured `Link` nodes or custom Solid component nodes
- Skips links inside code fences, inline code, HTML blocks, and HTML inline elements
- Emits warnings for conflicting modifier combinations

## Recommended Plugin Order

```text
micromatter,img,cite,wiki,anchor,markdown
```

**Why this order matters:**

1. `cite` and `wiki` must run before anchor because their syntax (`[^^](...)`, `[[...]]`) is more specific than generic `[label](url)`
2. `anchor` runs before `markdown` to intercept raw link text before it becomes AST Link nodes
3. `markdown` runs last to handle any remaining inline formatting

Example task config:

```toml
[[task]]
input = "./src/[slug].md"
output = "./out/[slug].jsx"
plugin = "micromatter,img,cite,wiki,anchor,markdown"
format = "solid"
```

## Link Syntax

### Basic Link

```md
[Visit our docs](/docs/getting-started)
```

### Link with Title

```md
[Visit our docs](/docs/getting-started "Getting Started Guide")
```

### Link with Suffix Modifiers

```md
[External resource](https://example.com^!)
```

### Link with Extra Attributes

```md
[Styled link](/page){class: "btn btn-primary", data-track: "cta-click"}
```

### Combined Syntax

```md
[Partner site](https://partner.com/path^--$! "Partner Page"){data-campaign: "q4-launch"}
```

## URL Suffix Modifiers

Modifiers are appended directly to the URL (before the title) and control link behavior:

| Modifier | Effect                  |
| :------- | :---------------------- |
| `^`      | Force `target="_blank"` |
| `~`      | Force `target="_self"`  |
| `!`      | Add `nofollow` to rel   |
| `$`      | Add `sponsored` to rel  |
| `;;`     | Add `ugc` to rel        |
| `--`     | Add `noreferrer` to rel |

Modifiers can be combined in any order:

```md
[Link](https://example.com^--$!)

<!-- Results in: target="_blank" rel="noopener noreferrer sponsored nofollow" -->
```

### Conflict Resolution

If both `^` and `~` are provided, the **last modifier wins** and a warning diagnostic is emitted:

```md
[Conflicting](https://example.com^~)

<!-- Warning: [anchor] both '^' and '~' were provided; last modifier wins -->
<!-- Result: target="_self" -->
```

## External Link Detection

Links are automatically classified as external when the URL matches any of these patterns:

- Starts with `http://` or `https://`
- Starts with `//` (protocol-relative)
- Starts with `www.`
- Contains a `.` and does not start with `/`

External links receive `target="_blank"` and `rel="noopener"` by default. These defaults can be overridden or augmented with suffix modifiers.

## Extra Attributes

An extras head may be attached immediately after the closing parenthesis:

```md
[Link](/page){class: "highlight", id: "important-link", --color: "blue"}
[Link](/page)@@anchorA{.highlight, #important-link, data-track: "cta"}
```

The head is a single `{…}` block; the `@@type` prefix is optional and, when
present, is the §11 routing marker of the instance. `.class` items accumulate in
the `class` list, `#id` (or a positional `` `slug` ``) sets the element `id`, and
`key: "value"` items set arbitrary attributes.

Special handling:

- `rel` values are **merged** with modifier-generated rel tokens (not replaced)
- `target` overrides the modifier/default target value
- Keys starting with `--` are passed through as-is (for CSS custom properties)
- All other keys become regular HTML/Solid attributes
- Values can be quoted (`"value"`) or unquoted (`value`)

The head must touch the closing `)` (no space, and no space before its own `{`):
`[Link](/page) {.x}` and `[Link](/page)@@a {.x}` both stay literal text. The
retired `[.class,#id]{key: value}` block is literal text too.

## Custom Solid Component

Replace the default `Link` node with a custom Solid component:

```toml
[task.anchor.custom_node]
name = "Anchor"
template = "<Anchor href={attrs.href} target={attrs.target} rel={attrs.rel} class=\"{attrs.class}\">{children}</Anchor>"

[[task.anchor.custom_node.imports]]
module = "@comp/shared/Anchor"
default = "Anchor"
```

`imports` also accepts a raw import line
(`imports = ["import Anchor from '@comp/shared/Anchor';"]`). Entries coming from
the same module are merged into a single `import` statement.

### Available Template Attributes

| Attribute        | Description                                     |
| :--------------- | :---------------------------------------------- |
| `{attrs.href}`   | Link URL (modifiers stripped)                   |
| `{attrs.title}`  | Link title (if provided)                        |
| `{attrs.target}` | Target attribute (`_blank`, `_self`, or absent) |
| `{attrs.rel}`    | Space-separated rel tokens                      |
| `{attrs.class}`  | Class from extra attrs (if provided)            |
| `{attrs.id}`     | ID from extra attrs (if provided)               |
| `{attrs.data-*}` | Any extra data attributes                       |
| `{children}`     | Link label text                                 |

### Default HTML Output (No Custom Node)

Without a custom node, links render as standard `<a>` tags via the built-in `Link` node type:

```html
<a href="https://example.com" target="_blank" rel="noopener noreferrer sponsored nofollow">
  External resource
</a>
```

## Behavioral Notes

- This plugin processes **raw text events** — it must run before `plugin-markdown`
- Links inside code fences, inline code, HTML blocks, and HTML inline elements are ignored
- Image syntax `![alt](src)` is explicitly skipped (handled by `plugin-img`)
- Extra attribute items (`.class`, `#id`, `{key: val}`) only support flat values — nested objects are not supported
- Unquoted values in extra attrs are treated as literal strings (no expression evaluation)
- Modifier parsing strips tokens from the URL right-to-left; malformed trailing characters that don't match known modifiers are left as part of the URL

## Scope and Limitations

- Only processes `[label](url)` inline link syntax — reference-style links (`[label][ref]`) are not supported
- Nested brackets in labels are not supported
- Parentheses inside URLs must be balanced for correct parsing
- The plugin does not validate URLs or check for broken links
- Extra attributes require at least one extras item (`.class`, `#id`, `key: val` or a bare flag) after `)`; the head must be adjacent

## License

MIT
