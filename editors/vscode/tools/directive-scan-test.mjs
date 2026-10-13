// Drives `src/scan.js` against the directive goldens with plain `node`, no VS
// Code and no npm deps. It asserts the scanner pairs open/close markers exactly
// the way `docs/spec/golden/15-directive-inline` and `16-directive-block` say the
// engine does (§10.2 / §10.3). Run it with `npm test`.
import assert from "node:assert/strict"
import fs from "node:fs"
import path from "node:path"
import { fileURLToPath } from "node:url"

import scan from "../src/scan.js"

const { scanDirectives } = scan

const HERE = path.dirname(fileURLToPath(import.meta.url))
const ROOT = path.resolve(HERE, "..", "..", "..")
const golden = (name) => fs.readFileSync(path.join(ROOT, "docs", "spec", "golden", name), "utf8")

let checks = 0
const ok = (label, cond) => {
  assert.ok(cond, label)
  checks++
}

// --- 15: inline directives ----------------------------------------------------
{
  const { pairs } = scanDirectives(golden("15-directive-inline.mdp"))

  // The headed pair on line 5: ::note[note-1]("A level")@@note{…} … ::
  const headed = pairs.find((p) => p.open.line === 4)
  ok("15: headed inline pair opens on line 5", headed)
  ok("15: headed inline pair closes on line 5", headed.close.line === 4)
  ok("15: opener is 2 colons", headed.open.endCol - headed.open.startCol === 2)

  // ::outer a ::note b:: c:: — two nested inline pairs on one line.
  const nested = pairs.filter((p) => p.open.line === 14)
  ok("15: outer + inner inline pairs both matched", nested.length === 2)
  ok(
    "15: depths are 1 and 2",
    nested
      .map((p) => p.depth)
      .sort()
      .join() === "1,2",
  )
  ok("15: inner closes before outer (LIFO)", nested[0].close.startCol < nested[1].close.startCol)

  // ::note no close here. never opens (§4.3) → no pair on line 18.
  ok("15: unterminated opener stays literal", !pairs.some((p) => p.open.line === 18))
  // :: alone, stays literal → the bare "::" is not treated as a marker.
  ok("15: bare :: stays literal", !pairs.some((p) => p.open.line === 18 && p.open.role === "open"))
}

// --- 16: block directives -----------------------------------------------------
{
  const { pairs, markers } = scanDirectives(golden("16-directive-block.mdp"))

  // ==note … == (lines 3..8 in 1-based → 2..7 zero-based).
  const single = pairs.find((p) => p.open.line === 2)
  ok("16: first block pair opens on ==note", single && single.open.role === "open")
  ok("16: first block pair closes on the lone ==", single.close.line === 7)

  // ==outer / ==note / == / == (nested, LIFO).
  const nested = pairs.filter((p) => p.open.line >= 12 && p.open.line <= 17)
  ok("16: nested block pairs matched", nested.length === 2)
  ok(
    "16: block depths are 1 and 2",
    nested
      .map((p) => p.depth)
      .sort()
      .join() === "1,2",
  )
  ok(
    "16: innermost closes first (LIFO)",
    nested.find((p) => p.depth === 2).close.line < nested.find((p) => p.depth === 1).close.line,
  )

  // A fence with nothing open (line 23) and a typeless fence (==5, line 24) stay
  // literal — neither should have produced a marker.
  ok("16: anonymous == stays literal", !markers.some((m) => m.line === 22))
  ok("16: ==5 (no type) stays literal", !markers.some((m) => m.line === 23))

  // Unclosed ==note at EOF is dropped (no close marker, no dangling open pair).
  ok("16: unclosed opener is discarded", !markers.some((m) => m.role === "close" && m.line > 28))
}

// --- escaping (§4.2) ----------------------------------------------------------
{
  const { pairs, markers } = scanDirectives("\\::note x:: and text\n==note\nbody\n==\n")
  ok(
    "esc: a backslashed :: never opens",
    !markers.some((m) => m.kind === "inline" && m.role === "open"),
  )
  ok(
    "esc: block directive still works on its own lines",
    pairs.some((p) => p.kind === "block"),
  )
}

// --- code spans / fences are inert --------------------------------------------
{
  const { markers } = scanDirectives("text `::note x::` more\n```\n==note\n==\n```\n")
  ok("code: :: inside an inline code span is ignored", !markers.some((m) => m.kind === "inline"))
  ok("code: == inside a fenced code block is ignored", !markers.some((m) => m.kind === "block"))
}

console.log(`directive-scan-test: ${checks} checks passed`)
