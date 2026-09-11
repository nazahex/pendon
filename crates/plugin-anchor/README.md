# pendon-plugin-anchor

Extended link processing for Pendon. The plugin applies external-link defaults, parses link modifiers, merges `rel` values, and optionally emits a custom Solid node.

## Syntax

```md
[docs](https://example.com)
^--$![docs](https://example.com "Example")
[docs](https://example.com--!){rel: "prefetch", hreflang: "en"}
```

Modifiers may appear before the label or at the end of the URL. The canonical form is before the label:

- `^`: `target="_blank"` and `noopener`
- `~`: `target="_self"`
- `!`: `nofollow`
- `--`: `noreferrer`
- `$`: `sponsored`
- `;;`: `ugc`

External links (`http://`, `https://`, `//`, `www.`, and hostname-like URLs) default to `target="_blank" rel="noopener"`. `rel` values from modifiers and extra attributes are deduplicated. If `^` and `~` are both present, the last one wins and a warning diagnostic is emitted.

## Task configuration

```toml
[[task]]
name = "Anchor Custom Node Demo (Solid)"
input = "./src/custom.md"
output = "./out/custom.jsx"
plugin = "anchor,markdown"
format = "solid"

[task.anchor.custom_node]
name = "Anchor"
template = "<Anchor href=\"{attrs.href}\">{children}</Anchor>"

[[task.anchor.custom_node.imports]]
module = "@comp/shared/Anchor"
default = "Anchor"
```

Without `custom_node`, the plugin emits the standard `Link` node and built-in renderers produce normal anchor elements. With `custom_node`, each link becomes `Custom("Anchor")`; the configured template and imports apply only to that task.
