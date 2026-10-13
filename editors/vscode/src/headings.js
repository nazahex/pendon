"use strict"

// Heading scanner for the Pendon VS Code extension.
//
// This is an *outline* scanner, not a parser: it finds the ATX headings of a
// `.mdp` document and computes the plain-text label VS Code shows in the
// breadcrumbs / Outline, the way the built-in Markdown extension does. The
// rules it mirrors live in `docs/spec/SYNTAX.md` and the engine:
//
//   - §7.4 ATX heading: a run of `#` at line start, then the optional Pendon
//     head `[slug]` / `("title")` / `@@type{…}` / `{…}` (§7.4 + §11), then the
//     visible title. The head is stripped; the visible title is the label.
//   - Detection is *column-0 only*: the `core` lexer only emits a heading
//     marker at the start of a line, and every fixture is written that way.
//     (`plugin-markdown` also accepts an indented marker with a following
//     space, but that secondary path is left out — it never appears in a real
//     Pendon document and would diverge from the column-0 rule.)
//   - There is no 6-level cap: Pendon accepts `#######` (level 7+) as a real
//     heading, unlike CommonMark.
//   - A space after the `#` run is *not* required (`#NoSpace` is a heading).
//   - Setext headings never existed (§14); a `----` line is a thematic break.
//   - The breadcrumb label reduces inline markdown to plain text (`**bold**` →
//     `bold`, `` `code` `` → `code`, `[t](u)` → `t`), mirroring the built-in
//     Markdown outline which concatenates a heading's text children.
//   - A trailing closing `#` run is *not* stripped by the engine (verified:
//     `## Trailing ##` keeps its title), so it stays in the label.
//   - A `#` inside a region the engine keeps out of the outline is skipped:
//     fenced code (info strings included), HTML comments, closed frontmatter
//     and `plugin-bind` data blocks — see `scanHeadings`.
//
// It stays VS Code-free on purpose so `tools/heading-scan-test.mjs` can drive
// it with plain `node`.

// Fenced-code opener: three or more backticks/tildes at line start, with any
// info string after them (```` ```yaml ````); the core lexer emits the fence on
// the run itself and markdown-it keeps the info string as `lang`.
const FENCE_OPEN = /^\s*(`{3,}|~{3,})/
// A *closer* must be a bare run of the same char, at least as long (CommonMark;
// a ` ```js ` line inside a block does not close it).
const FENCE_CLOSE = /^\s*(`{3,}|~{3,})\s*$/

// Column-0 ATX marker: a run of `#`. The `core` lexer emits a heading marker for
// any `#` run at line start, so a space after it is *not* required (`#NoSpace`
// is a heading) and there is no level cap; a lone `#` is a blank heading.
const ATX = /^(#+)(.*)$/

// Strip a trailing carriage return so `\r\n` files line up with VS Code.
function normalize(line) {
  return line.endsWith("\r") ? line.slice(0, -1) : line
}

/** Index of the `)` matching the `(` at `start`, counting nested pairs. */
function findMatchingParen(chars, start) {
  let depth = 0
  for (let i = start; i < chars.length; i++) {
    if (chars[i] === "(") depth++
    else if (chars[i] === ")") {
      depth--
      if (depth === 0) return i
    }
  }
  return -1
}

/**
 * Consume the Pendon head off the front of `text` (§7.4 / §11) and return how
 * many characters it spans. Mirrors `plugin-heading::parse_heading_head`:
 *
 *   - an optional `[slug]` bracket (a `.`/`#`-leading bracket is literal);
 *   - an optional `("title")` parenthesis (a separate attribute, never title);
 *   - an optional adjacent `@@type{…}` / `@@type` / `{…}` extras head.
 *
 * A malformed extras head consumes nothing (§4.3) and stays literal, exactly
 * like `scan_extras_chars` returning `None`.
 */
function consumeHead(text) {
  const chars = [...text]
  let cursor = 0

  // §7.4: an optional `[slug]` head.
  if (chars[cursor] === "[") {
    const close = chars.indexOf("]", cursor + 1)
    if (close !== -1) {
      const content = text.slice(cursor + 1, close).trim()
      if (content !== "" && !content.startsWith(".") && !content.startsWith("#")) {
        cursor = close + 1
      }
    }
  }

  // §7.4: an optional `("title")` head (never part of the title).
  if (chars[cursor] === "(") {
    const close = findMatchingParen(chars, cursor)
    if (close !== -1) cursor = close + 1
  }

  // §7.4/§11: an adjacent `@@type{…}` / `@@type` / `{…}` extras head.
  cursor += consumeExtras(text.slice(cursor))

  return cursor
}

/**
 * Span of an extras head (`@@type{…}`, `@@{…}`, `@@type`, `{…}`) at the front
 * of `text`, or 0 when it is absent or malformed. Mirrors
 * `crates/extra::parse_extras` closely enough to strip a head from a title:
 * both spellings are heads, adjacency is strict, and a head that does not close
 * on the line consumes nothing (§4.3).
 */
function consumeExtras(text) {
  if (text.startsWith("@@")) {
    const after = consumeAtHead(text.slice(2))
    return after > 0 ? after + 2 : 0
  }
  if (text.startsWith("{")) return consumeBraceBody(text)
  return 0
}

/** After the `@@` prefix: `type? […]? (…)? {…}?`, each part touching the last. */
function consumeAtHead(after) {
  let cursor = 0
  // Optional type name: an ASCII letter then alphanumerics.
  if (/[A-Za-z]/.test(after[0] ?? "")) {
    cursor = 1
    while (/[A-Za-z0-9]/.test(after[cursor] ?? "")) cursor++
  }
  const hasType = cursor > 0

  // Optional positional groups: an immediate `[…]` then an immediate `(…)`.
  const groups = consumePositionalGroups(after.slice(cursor))
  cursor += groups

  // An adjacent `{…}` body.
  if (after[cursor] === "{") {
    const body = consumeBraceBody(after.slice(cursor))
    if (body > 0) return cursor + body
    return 0 // an unterminated body makes the whole head literal (§4.3)
  }

  // No body: a type or a positional group carries the head on its own; a bare
  // `@@` does not. `@@type {…}` (space before the brace) is not a head (§4.1).
  if (!hasType && groups === 0) return 0
  const rest = after.slice(cursor)
  if (/^[ \t]/.test(rest) && /^[ \t]*\{/.test(rest)) return 0
  return cursor
}

/** Span of an immediate `[…]` then an immediate `(…)`, or 0. */
function consumePositionalGroups(after) {
  let cursor = 0
  if (after[cursor] === "[") {
    const close = after.indexOf("]", cursor + 1)
    if (close === -1) return 0
    cursor = close + 1
  }
  if (after[cursor] === "(") {
    const close = after.indexOf(")", cursor + 1)
    if (close === -1) return 0
    cursor = close + 1
  }
  return cursor
}

/**
 * Span of a `{…}` body (including both braces), or 0 when it never closes on
 * this line. Mirrors `find_group_close`, which scans the body *after* the
 * opening brace at depth 0: the first `}` at depth 0 closes the head. Honours
 * nested `{ }` / `[ ]`, quotes and `\` escapes so a nested value does not close
 * it early.
 */
function consumeBraceBody(text) {
  let depth = 0
  let quote = null
  let escaped = false
  for (let i = 1; i < text.length; i++) {
    const ch = text[i]
    if (escaped) {
      escaped = false
      continue
    }
    if (ch === "\\") {
      escaped = true
      continue
    }
    if (ch === "'" || ch === '"' || ch === "`") {
      if (quote === null) quote = ch
      else if (quote === ch) quote = null
      continue
    }
    if (quote !== null) continue
    if (ch === "{" || ch === "[") depth++
    else if (ch === "}" || ch === "]") {
      if (depth === 0) return ch === "}" ? i + 1 : 0
      depth--
    }
  }
  return 0
}

/**
 * Reduce a heading's visible text to the plain-text label the built-in Markdown
 * outline shows: inline markdown becomes its text content. Handles the inline
 * constructs the grammar scopes (emphasis, code, links, wiki, citation) so a
 * breadcrumb reads `Bold` rather than `**Bold**`.
 */
function plainLabel(text) {
  return (
    text
      // Inline code spans first: their content is literal, so it is protected
      // from the emphasis/link passes below.
      .replace(/(`+)([^`]|[^`][\s\S]*?[^`])\1(?!`)/g, "$2")
      // Links `[text](url)` → text.
      .replace(/\[([^\]]*)\]\([^)]*\)/g, "$1")
      // Wiki links `[[target|label]]` → the label, `[[target]]` → the target.
      .replace(/\[\[[^\]|]*\|([^\]]*)\]\]/g, "$1")
      .replace(/\[\[([^\]]*)\]\]/g, "$1")
      // Citations `[^^](ref "loc")` → the reference.
      .replace(/\[\^\^\]\(\s*([^)\s]*)[^)]*\)/g, "$1")
      // Bold / italic / strikethrough: drop the delimiters, keep the text.
      .replace(/(\*\*\*|___)([\s\S]+?)\1/g, "$2")
      .replace(/(\*\*|__)([\s\S]+?)\1/g, "$2")
      .replace(/(\*|_)([\s\S]+?)\1/g, "$2")
      .replace(/~~([\s\S]+?)~~/g, "$1")
      // Backslash escapes: `\*` → `*`.
      .replace(/\\([\\`*_{}[\]()#+.!~|-])/g, "$1")
      .trim()
  )
}

/**
 * Scan `text` and return every ATX heading.
 *
 * Lines inside the four regions the engine never turns into document-level
 * headings are skipped (each verified with `pendon run --format ast` on the
 * full chain `micromatter,markdown,bind`):
 *
 *   - fenced code blocks, including ones opened with an info string
 *     (```` ```yaml ````);
 *   - HTML comments (`<!-- … -->`), which markdown-it nests inside a paragraph
 *     and `markdown_strip_comments` drops from output — a `#` there is comment
 *     text, exactly as in vanilla Markdown;
 *   - frontmatter (`---` at the document start … closing `---`): `micromatter`
 *     removes the whole span; without it the content stays a paragraph and the
 *     `#` still never reaches the document level. Either way it is not an
 *     outline heading. Only a *closed* span counts — micromatter reports
 *     `missing closing ---` and leaves the content as ordinary markup;
 *   - `plugin-bind` data blocks (`{{{lang[var] … }}}`, §19), removed by the
 *     pre-parse stage. An *unterminated* block is "left as text" (§19), so its
 *     body keeps scanning, mirroring the engine.
 *
 * @returns {Array<{ level: number, line: number, startCol: number, endCol: number,
 *                  titleStartCol: number, label: string }>}
 *   level           — the `#` run length (uncapped, as the engine allows).
 *   line            — 0-based line index of the marker.
 *   startCol/endCol — the marker run `[startCol, endCol)` (for the symbol range).
 *   titleStartCol   — column where the visible title begins (for selectionRange).
 *   label           — the plain-text title (head + inline markdown stripped).
 */
function scanHeadings(text) {
  const out = []
  const lines = text.split("\n").map(normalize)

  // Frontmatter: a `---` fence pair at the very start of the document
  // (micromatter's rule). A span with no closing `---` is not frontmatter.
  let frontmatterEnd = -1
  if (lines[0] !== undefined && lines[0].trim() === "---") {
    for (let i = 1; i < lines.length; i++) {
      if (lines[i].trim() === "---") {
        frontmatterEnd = i
        break
      }
    }
  }

  let fence = null // { char, len } while inside a fenced code block
  let comment = false // inside an `<!-- … -->` HTML comment
  let bind = false // inside a `{{{ … }}}` data block

  for (let line = 0; line < lines.length; line++) {
    const raw = lines[line]

    if (line <= frontmatterEnd) continue

    if (comment) {
      // The comment ends on the first line carrying `-->`; whatever follows it
      // on that line is ordinary text again (and never a column-0 heading).
      if (raw.includes("-->")) comment = false
      continue
    }
    if (fence) {
      // A bare run of the same char, at least as long, closes the block.
      const close = raw.match(FENCE_CLOSE)
      if (close && close[1][0] === fence.char && close[1].length >= fence.len) fence = null
      continue
    }
    if (bind) {
      if (raw.trim() === "}}}") bind = false
      continue
    }

    // --- region openers (a line that opens a region is never a heading) ------
    const trimmed = raw.trimStart()
    if (trimmed.startsWith("<!--")) {
      // One-line comments (`<!-- x -->`) open and close at once.
      if (!raw.includes("-->")) comment = true
      continue
    }
    const fenceOpen = raw.match(FENCE_OPEN)
    if (fenceOpen) {
      fence = { char: fenceOpen[1][0], len: fenceOpen[1].length }
      continue
    }
    if (trimmed.startsWith("{{{")) {
      // A head line that already carries its `}}}` (a `(path)` payload or a
      // dropped one-line body) is a whole block; else the body runs to the
      // next trimmed-`}}}` line. No terminator at all → the block stays text
      // and every following line keeps scanning (§19's drop rule).
      if (raw.includes("}}}")) continue
      const end = lines.findIndex((l, i) => i > line && l.trim() === "}}}")
      if (end !== -1) {
        line = end
        continue
      }
    }

    const m = raw.match(ATX)
    if (!m) continue

    const marker = m[1]
    const rest = m[2]
    const endCol = marker.length

    // The head touches the marker; the single separating space (if any) is part
    // of the head region and never the title (§7.4).
    const after = rest.startsWith(" ") || rest.startsWith("\t") ? rest.slice(1) : rest
    const sepLen = rest.length - after.length
    const headLen = consumeHead(after)
    const headEnd = endCol + sepLen + headLen

    // The whitespace between the head and the title is not part of the title.
    const tail = after.slice(headLen)
    const title = tail.replace(/^[ \t]+/, "")
    const titleStartCol = headEnd + (tail.length - title.length)

    out.push({
      level: marker.length,
      line,
      startCol: 0,
      endCol,
      titleStartCol,
      label: plainLabel(title),
    })
  }

  return out
}

module.exports = { scanHeadings }
