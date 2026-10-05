# Code Highlight Tags

This document describes the minimal HTML tags used by Pendon’s syntect-based code fence highlighter. The goal is small HTML size, predictable styling, and raw output that stays valid inside `<pre><code>` blocks.

## Tag Set

- **b**: structural/keywords (tags, doctype, keywords, storage, support)
- **em**: names (attribute-name, generic name)
- **i**: text values (string, quoted/unquoted, attribute-value, property-value)
- **u**: numeric values (number, integer/decimal, rgb-value)
- **s**: comments (including SGML comment scopes)
- **mark**: operators/punctuation/separators/terminators and embedded/script/style markers
- **ins**: inline markers (e.g., inline blocks/notations)
- **var**: variables and parameters
- **dfn**: selectors (definition-ish tokens in CSS selectors)
- **abbr**: class/id (frequent, often abbreviated)
- **span**: fallback bucket for other/meta/entity
- **kbd**: reserved for keyboard-input-ish tokens (future use)
- **samp**: reserved for sample output tokens (future use)

These tags are presentational in this context. They do not convey semantic meaning beyond consistent, themeable styling inside code blocks.

## Design Goals

- **Small HTML**: prefer short tag names for frequently occurring tokens.
- **Predictable**: avoid UA defaults that conflict with code presentation.
- **Themeable**: one central CSS can colorize all tags.
- **Raw**: when `attrs.raw_html="1"`, HTML is emitted without escaping, preserving tags.

## CSS Normalizer / Reset

Browsers apply default styles to some inline tags (e.g., `b`, `em`, `i`, `u`, `s`, `mark`, `abbr`, `ins`, `dfn`, `kbd`, `samp`). Reset them inside code blocks to ensure consistent rendering.

```css
/* Base: apply resets only inside pre > code to avoid affecting page text */
pre code :is(b, em, i, u, s, mark, ins, abbr, var, dfn, samp, span, kbd) {
  font-style: inherit; /* cancel italics from em/i/dfn/var defaults */
  font-weight: inherit; /* cancel bold from b defaults */
  text-decoration: none; /* cancel underline/strikethrough from u/s/ins */
  background: transparent; /* cancel mark default yellow background */
  border: 0;
  outline: 0;
  font-family: inherit; /* keep monospaced font in code */
}
/* Ensure inline layout */
pre code :is(b, em, i, u, s, mark, ins, abbr, var, dfn, samp, span, kbd) {
  display: inline;
}
```

Notes:

- Reset is intentionally conservative; it avoids `all: unset` to preserve inherited code block properties (e.g., white-space, font-size).
- If your global CSS redefines any of these tags, scope your resets under a container (e.g., `.code-view pre code ...`).

## Suggested Theme CSS

Below is a compact example theme (used in the sandbox). Tune colors to your brand.

```css
pre {
  white-space: pre;
  overflow: auto;
}

b {
  color: #c792ea;
  font-weight: 600;
} /* keywords / tags */
em {
  color: #82aaff;
  font-style: normal;
} /* names */
i {
  color: #ecc48d;
} /* strings */
u {
  color: #f78c6c;
  text-decoration: none;
} /* numbers */
s {
  color: #5c6370;
  text-decoration: none;
} /* comments */
mark {
  background: none;
  color: #89ddff;
} /* operators */

/* Extended tags */
var {
  color: #c3e88d;
} /* variables */
kbd {
  color: #ffcb6b;
} /* keyboard (reserved) */
dfn {
  color: #a6accd;
  text-decoration: underline dotted;
} /* selectors */
samp {
  color: #9ccc65;
} /* sample output (reserved) */
ins {
  color: #b2ccd6;
  text-decoration: underline;
} /* inline markers */
abbr {
  color: #d4d4d4;
  border-bottom: 1px dotted currentColor;
} /* class/id */
span {
  color: #c0c0c0;
} /* fallback */
```

## Usage Example

HTML output inside a code fence (simplified):

```html
<pre><code>
<b>&lt;div</b> <em>class</em><mark>=</mark><i>"container"</i><b>&gt;</b>
  Hello <b>&lt;span</b> <em>data-id</em><mark>=</mark><i>"1"</i><b>&gt;</b>world<b>&lt;/span&gt;</b>
<b>&lt;/div&gt;</b>
</code></pre>
```

With the reset + theme CSS above, this renders with consistent colors without UA-default italics/underline/background.

## Integration Notes

- **Renderer behavior**: When a code fence node carries `attrs.raw_html="1"`, both HTML and Solid renderers inject the code fence content as raw HTML, preserving these tags.
- **Extensibility**: Per-language mappers can remap syntect classes to tags. HTML currently maps:
  - b: `tag`, `doctype`, `keyword`, `storage`, `support`
  - em: `attribute-name`, `name`
  - i: `string`, `quoted`, `unquoted`, `text`, `attribute-value`, `property-value`
  - u: `number`, `numeric`, `integer`, `decimal`, `rgb-value`
  - s: `comment`, `sgml`
  - ins: `inline`
  - mark: `operator`, `punctuation`, `separator`, `terminator`, `embedded`, `script`, `style`
  - var: `variable`, `parameter`, `parameters`
  - dfn: `selector`
  - abbr: `class-name`, `id`
  - span: `other`, `meta`, `entity`

## Best Practices

- Prefer short tags for high-frequency tokens to keep HTML small.
- Keep resets scoped to code content to avoid impacting the rest of the page.
- Evolve mapper per language using generated class lists in [docs/grammar](docs/grammar).
