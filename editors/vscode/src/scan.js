"use strict"

// Directive marker scanner for the Pendon VS Code extension.
//
// This is a *decoration* scanner, not a parser: it finds the matched open/close
// marker runs of a directive and their nesting depth so the editor can colour
// them (see `extension.js`). The rules it mirrors live in `docs/spec/SYNTAX.md`:
//
//   - §10.2 inline directive: `::type … ::`, colons = 2..7, the closing run must
//     be >= the opening run, nesting depth = colon count. A `:` run followed by a
//     type letter opens; a `:` run not followed by a letter closes (LIFO). An
//     opener with no closer on its line never opens (§4.3), and a bare `::` stays
//     literal.
//   - §10.3 block directive: `==type` opens, a `==` fence alone on a line closes
//     the innermost open directive (LIFO). A fence with nothing open, or without
//     a type, stays literal; an unclosed opener is discarded (implicit EOF close).
//   - §4.2 escaping: a `\` before a `:`/`=` run removes its meaning, and inside
//     code fences / inline code spans no sigil is processed at all.
//
// It stays VS Code-free on purpose so `tools/directive-scan-test.mjs` can drive
// it with plain `node`.

// Fenced-code opener/closer: three or more backticks or tildes.
const FENCE = /^\s*(`{3,}|~{3,})\s*$/

// Block directive opener: up to 3 leading spaces, then 2..7 `=` and a type.
const BLOCK_OPEN = /^ {0,3}(={2,7})(?=[A-Za-z])([A-Za-z][A-Za-z0-9_-]*)/

// Block directive closer: up to 3 leading spaces, then 2..7 `=` and nothing else.
const BLOCK_CLOSE = /^ {0,3}(={2,7})[ \t]*$/

// Inline code spans: `…`, ``…`` … — the inner text is blanked, columns preserved.
const INLINE_CODE = /(`+)([\s\S]*?)\1/g

// Column index of the first non-space, i.e. where a `=` fence run starts.
function leadingSpaces(line) {
  let i = 0
  while (i < line.length && (line[i] === " " || line[i] === "\t")) i++
  return i
}

// Strip a trailing carriage return so `\r\n` files line up with VS Code.
function normalize(line) {
  return line.endsWith("\r") ? line.slice(0, -1) : line
}

// Replace the inside of every inline code span with spaces so a marker that
// lives inside `…` is not treated as a directive, while every column index stays
// put (the decoration layer needs real offsets).
function blankInlineCode(line) {
  return line.replace(INLINE_CODE, (_match, ticks, inner) => {
    return ticks + " ".repeat(inner.length) + ticks
  })
}

/**
 * Scan `text` and return every matched directive marker plus the pairs they form.
 *
 * @returns {{ markers: object[], pairs: object[] }}
 *   markers — flat list; each is
 *     `{ kind: "inline" | "block", role: "open" | "close", depth, line, startCol, endCol, partner }`
 *   pairs — each is `{ kind, depth, open, close }` where `open`/`close` are
 *     marker objects that also point back at each other via `partner`.
 */
function scanDirectives(text) {
  const markers = []
  const pairs = []

  const push = (marker) => {
    markers.push(marker)
    return marker
  }

  // Block directives: line-oriented, LIFO (§10.3).
  const blockStack = []
  let fence = null // { char, len } while inside a fenced code block
  const lines = text.split("\n").map(normalize)

  for (let line = 0; line < lines.length; line++) {
    const raw = lines[line]
    const fenceMatch = raw.match(FENCE)

    if (fence) {
      // A fence line of the same char, at least as long, closes the code block.
      if (fenceMatch && fenceMatch[1][0] === fence.char && fenceMatch[1].length >= fence.len) {
        fence = null
      }
      continue
    }
    if (fenceMatch) {
      fence = { char: fenceMatch[1][0], len: fenceMatch[1].length }
      continue
    }

    const close = raw.match(BLOCK_CLOSE)
    if (close && blockStack.length) {
      const open = blockStack.pop()
      const col = leadingSpaces(raw)
      const marker = push({
        kind: "block",
        role: "close",
        depth: open.depth,
        line,
        startCol: col,
        endCol: col + close[1].length,
      })
      open.partner = marker
      marker.partner = open
      pairs.push({ kind: "block", depth: open.depth, open, close: marker })
      continue
    }

    const open = raw.match(BLOCK_OPEN)
    if (open) {
      const col = leadingSpaces(raw)
      const marker = push({
        kind: "block",
        role: "open",
        depth: blockStack.length + 1,
        line,
        startCol: col,
        endCol: col + open[1].length,
      })
      blockStack.push(marker)
      continue
    }

    // Inline directives: colon runs within the line, LIFO (§10.2).
    scanInlineLine(raw, line, markers, pairs, push)
  }

  // Unclosed openers (§10.3 implicit EOF close) have no visible partner and are
  // dropped: they stay literal text for decoration purposes.
  return { markers, pairs }
}

/**
 * Match `::` … `::` runs on a single line. A maximal `:` run of length 2..7 that
 * is followed by a type letter opens; one not followed by a letter closes the
 * innermost open run (§10.2). Openers left on the stack at end of line never
 * opened (§4.3) and are removed again.
 */
function scanInlineLine(raw, line, markers, pairs, push) {
  const text = blankInlineCode(raw)
  const stack = []
  const re = /:+/g
  let m = re.exec(text)
  while (m !== null) {
    const startCol = m.index
    const len = m[0].length
    if (len < 2 || len > 7) {
      m = re.exec(text)
      continue
    }
    if (text[startCol - 1] === "\\") {
      m = re.exec(text)
      continue
    }

    const next = text[startCol + len]
    const isOpen = next !== undefined && /[A-Za-z]/.test(next)

    if (isOpen) {
      const marker = push({
        kind: "inline",
        role: "open",
        depth: stack.length + 1,
        line,
        startCol,
        endCol: startCol + len,
      })
      stack.push(marker)
    } else if (stack.length) {
      const open = stack.pop()
      const marker = push({
        kind: "inline",
        role: "close",
        depth: open.depth,
        line,
        startCol,
        endCol: startCol + len,
      })
      open.partner = marker
      marker.partner = open
      pairs.push({ kind: "inline", depth: open.depth, open, close: marker })
    }

    m = re.exec(text)
  }

  // Remove openers that never closed on this line: they are literal text (§4.3).
  for (const open of stack) {
    const i = markers.indexOf(open)
    if (i !== -1) markers.splice(i, 1)
  }
}

module.exports = { scanDirectives }
