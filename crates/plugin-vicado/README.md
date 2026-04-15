# pendon-plugin-vicado

`pendon-plugin-vicado` is a Pendon plugin that converts specific code fences into a Solid component call for `Vicado`.

It keeps the standard fenced-code syntax, so editors still treat the block as normal code and preserve syntax highlighting assistance while you write.

## What It Does

If a fence info string matches this pattern:

```text
<language> vicado [optional-class-and-id] {optional-props}
```

the plugin transforms the fence into a custom `Vicado` node. During Solid rendering, it emits:

- `import {Vicado} from "vicado"`
- `<Vicado ... />` with parsed props and code content

## Syntax

Basic:

````md
```typescript vicado
const a = 1
```
````

With class/id and props:

````md
```tsx vicado [.hero, preview, #main-editor] {mount: "visible", lineNumbers: true, tabSize: 2}
export const value = 42
```
````

## Parsed Parts

- First token: language (for example `typescript`, `tsx`, `js`, `css`)
- Second token: must be `vicado`
- Optional bracket block: classes and id
: `.class-name` adds class, `#id-name` sets id
- Optional braces block: scalar props
: supports string, number, and boolean values

## Recommended Plugin Order

Use `markdown` before `vicado`:

```text
markdown,vicado
```

If you also use syntect highlighting for regular fences:

```text
markdown,vicado,syntect
```

This allows:

- Vicado fences to become `<Vicado ... />`
- Non-vicado fences to remain regular code fences and be highlighted by syntect

## CLI Example

```bash
pendon --plugin markdown,vicado --format solid --input ./example.md
```

With syntect for regular fences:

```bash
pendon --plugin markdown,vicado,syntect --format html --input ./example.md
```

## Configure Vicado Import In pendon.toml

When using `pendon run`, you can override how Solid import lines for `Vicado` are generated.

Default import (when no override is set):

```ts
import {Vicado} from "vicado"
```

To override it, add a `plugin-vicado` section in `pendon.toml`:

```toml
[[task]]
input = "./src/[...slug].md"
output = "./out/[...slug].jsx"
plugin = "markdown,vicado"
format = "solid"

[plugin-vicado.renderer.solid]

[[plugin-vicado.renderer.solid.imports]]
module = "@/components/vicado"
default = "Vicado"
```

The `imports` field supports:

- Structured import entries (`module`, optional `default`, optional `names`)
- Raw string imports (same style as `plugin-custom`)

Example with named imports:

```toml
[plugin-vicado.renderer.solid]

[[plugin-vicado.renderer.solid.imports]]
module = "vicado"
names = ["Vicado"]
```

If `plugin-vicado.renderer.solid.imports` is omitted, Pendon keeps the built-in default import behavior.

## Notes

- The plugin is designed for Solid output integration.
- Unknown or non-matching fences are left untouched.
- Prop keys must be valid identifier-like names (`foo`, `bar_1`).

## License

MIT
