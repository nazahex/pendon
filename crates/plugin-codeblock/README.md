# pendon-plugin-syntect

Syntax-highlights code fences using `syntect` and rewrites them into raw HTML payloads for renderers. Keeps the fence node shape but injects highlighted HTML and a `raw_html` marker so renderers can bypass escaping.

## Behavior

- Detects `CodeFence` blocks, reads `lang` (and optional `syntect_debug`) attributes, and collects the inner text until the closing fence.
- Highlights with syntect’s built-in syntax set; no external `.sublime-syntax` files are loaded.
- Language resolution:
  - Directly matches `lang` tokens (e.g., `rust`, `python`, `html`).
  - `ts`/`tsx`/`typescript` fall back to JavaScript grammar to ensure coverage.
  - Unknown/absent `lang` → plain-text highlighting.
- Output fence retains the `lang` attribute (if present), adds `raw_html=1`, and sets the fence text to the highlighted HTML.
- Each output line is wrapped in `<p>`; empty lines get a zero-width space to preserve height.
- Debug mode: set `syntect_debug="classes"` on the fence or `PENDON_SYNTECT_DEBUG=classes` to emit raw classed spans instead of minimized tags.
- Advanced info-string controls (inline + line highlights):
  - Inline highlights: add quoted or bare patterns after the language, e.g. `html "<b>" "indah" "b/>"` or `js log function*`. Patterns support `*` wildcards and are wrapped with `<strong>` in the emitted HTML (outside of existing tag boundaries).
  - Line highlights: `{n}` / `{n-m}` marks lines with `<p class="mark">`; `ins={n}` / `ins={n-m}` uses `<p class="ins">`; `del={n}` / `del={n-m}` uses `<p class="del">`. Unmarked lines stay `<p>` without class.

## Usage

CLI (after Markdown so code fences exist):

```bash
pendon --plugin micromatter,markdown,syntect --format html --input ./doc.md
```

Library:

```rust
use pendon_plugin_codeblock_syntect::process;
let highlighted = process(&events);
```

## Renderer expectations

- Renderers should detect `raw_html=1` on `CodeFence` and inject the `Text` content directly without escaping.
- The generated HTML uses minimal tags mapped from syntect classes; ensure your CSS targets the emitted tags or enable debug class output to style manually.
- Style `.mark`, `.ins`, `.del` in CSS to render line states. Unmarked lines are bare `<p>`.
