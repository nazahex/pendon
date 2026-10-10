# pendon-plugin-latex

Renders inline and display math with KaTeX, emitting MathML HTML that keeps its
namespace under Solid. Runs as an inline transform over the event stream.

## Syntax

- **Inline** math `$…$` → a `<span class="latex latex-inline" …>` wrapper.
- **Display** math `$$…$$` → a `<span class="latex" style="display: block;" …>`
  wrapper.

Invalid math emits a `<span class="latex-error" style="display: block;">…</span>`
so the failure is visible in the output rather than swallowed.

## Output target

`LatexTarget` chooses how the wrapper is emitted:

| Target  | Behaviour                                                                                                                                              |
| ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `Solid` | a JSX `innerHTML={...}` expression (default) — required so the browser's HTML parser assigns the correct MathML namespace when Solid claims the markup |
| `Html`  | plain, valid HTML (no `innerHTML`)                                                                                                                     |

## Options

```rust
use pendon_plugin_latex::{process, process_with_options, LatexOptions, LatexTarget};

let events = process(&parsed); // Solid target, the default

let opts = LatexOptions { target: LatexTarget::Html };
let events = process_with_options(&parsed, &opts);
```

`LatexOptions { target: LatexTarget }` — `LatexTarget` is `Solid` (default) or
`Html`.

## Notes

- The plugin merges adjacent text before scanning so a math span split across
  text events is still recognised.
- `plugin-wiki` embeds math inside infobox fragments; it renders them only when
  its `latex` option is `Some`.

## License

MIT
