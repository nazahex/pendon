// Drives `src/headings.js` against the heading goldens with plain `node`, no VS
// Code and no npm deps. It asserts the scanner finds the same headings — level,
// line and plain-text label — that `docs/spec/golden/06-heading`,
// `21-extras-forms` and `22-section` say the engine renders (§7.4), and pins the
// edge cases verified against the real CLI (no level cap, `#NoSpace`, head
// stripping, inline-markdown reduction, no setext, column-0 only). Run it with
// `npm test`.
import assert from "node:assert/strict"
import fs from "node:fs"
import path from "node:path"
import { fileURLToPath } from "node:url"

import scan from "../src/headings.js"

const { scanHeadings } = scan

const HERE = path.dirname(fileURLToPath(import.meta.url))
const ROOT = path.resolve(HERE, "..", "..", "..")
const golden = (name) => fs.readFileSync(path.join(ROOT, "docs", "spec", "golden", name), "utf8")

let checks = 0
const ok = (label, cond) => {
  assert.ok(cond, label)
  checks++
}
const eq = (label, actual, expected) => {
  assert.deepEqual(actual, expected, `${label}: got ${JSON.stringify(actual)}`)
  checks++
}
// The `(level, label)` pairs of a document, in order.
const outline = (text) => scanHeadings(text).map((h) => [h.level, h.label])

// --- 06-heading: heads are stripped, the visible title is the label ---------
eq("06: four ATX headings, heads stripped to plain titles", outline(golden("06-heading.mdp")), [
  [3, "Extras-routed heading"],
  [3, "Routed by the second marker"],
  [3, "An unclaimed marker keeps the layer default"],
  [2, "A plain heading without extras"],
])

// --- 21 / 22: a typed head and a section heading ----------------------------
const h21 = scanHeadings(golden("21-extras-forms.mdp"))
ok("21: exactly one heading", h21.length === 1)
eq("21: typed head stripped", [h21[0].level, h21[0].label], [3, "A typed heading"])

eq("22: section heading + bare heading", outline(golden("22-section.mdp")), [
  [2, "Section A"],
  [2, "Bare Section B"],
])

// --- §7.4 head forms --------------------------------------------------------
eq('head: [slug] + ("title") + @@type{…}', outline('##[slug]("T")@@h{.lead} Visible Title'), [
  [2, "Visible Title"],
])
eq("head: bare {…} after a space", outline("## {.x} TitleD"), [[2, "TitleD"]])
eq("head: adjacent {…}", outline("##{.x} TitleE"), [[2, "TitleE"]])
// A head that is not adjacent to the marker is literal text and stays.
eq("head: mid-text {.x} stays", outline("## Title {.x} kept"), [[2, "Title {.x} kept"]])
// A `[slug]` with no space before the title still strips.
eq("head: [slug] then title", outline("##[islug] NoSpaceHead"), [[2, "NoSpaceHead"]])

// --- detection edge cases (verified against the real engine) ----------------
eq("edge: no space after the marker", outline("#NoSpace"), [[1, "NoSpace"]])
eq("edge: no 6-level cap (level 7)", outline("####### Seven"), [[7, "Seven"]])
// A trailing closing `#` run is NOT stripped by the engine.
eq("edge: trailing ## kept", outline("## Trailing ##"), [[2, "Trailing ##"]])
// Column-0 only: an indented marker is not a heading here.
eq("edge: indented marker ignored", outline("   ## Indented"), [])
// Setext never existed; an underline is a thematic break.
eq("edge: setext underline is not a heading", outline("Title\n===="), [])

// --- inline markdown is reduced to plain text -------------------------------
eq(
  "inline: **bold**, `code`, [link](url) → plain",
  outline("## **bold** and `code` and [link](http://x)"),
  [[2, "bold and code and link"]],
)
eq("inline: wiki [[target|label]] → label", outline("## See [[Page|the label]]"), [
  [2, "See the label"],
])

// --- fenced code is inert (§4.3) --------------------------------------------
eq("fence: # inside a ``` block is skipped", outline("# Real\n\n```\n# Not a heading\n```\n"), [
  [1, "Real"],
])
// The opener may carry an info string (```` ```yaml ````) — the core lexer
// emits the fence on the run itself; verified against the real CLI.
eq("fence: info string still opens the block", outline("```yaml\n## Komentar\n```\n\ntail\n"), [])
// A closing fence must be a bare run: ```js inside a block does not close it.
eq("fence: info-string line does not close", outline("````\n```js\n## Still fenced\n````\n"), [])
// Tilde fences behave the same.
eq("fence: ~~~ block is skipped", outline("~~~\n# Not a heading\n~~~\n# Real\n"), [[1, "Real"]])

// --- HTML comments (markdown_strip_comments) --------------------------------
// Inside `<!-- … -->` the engine nests the `#` line into the comment's
// paragraph; it never becomes a document-level heading (verified with
// `--format ast`: Paragraph > [Text "<!--", Heading, Text "-->"]).
eq("comment: block comment skipped", outline("<!--\n# Communal\n-->\n\n# Hiz\n"), [[1, "Hiz"]])
eq("comment: one-line comment skipped", outline("<!-- # x -->\n\n# Real\n"), [[1, "Real"]])
// Only a `<!--` at trimmed line start opens a block; mid-line stays inline.
eq("comment: mid-line <!-- is inline", outline("text <!-- not a block\n\n# Real\n"), [[1, "Real"]])
// An unterminated comment runs to EOF (CommonMark html block).
eq("comment: unterminated runs to EOF", outline("<!--\n# Lost\n\n# Also lost\n"), [])

// --- frontmatter (plugin-micromatter) ---------------------------------------
// A closed `---` pair at the document start: micromatter removes the span
// entirely; without micromatter the `#` stays inside a paragraph. Either way
// it is not an outline heading (verified both ways with the real CLI).
eq(
  "frontmatter: closed pair skipped",
  outline('---\ntitle: "Demo"\n# FmComment\n---\n\n# After\n'),
  [[1, "After"]],
)
// An *unclosed* pair is not frontmatter (micromatter errors, content stays).
eq("frontmatter: unclosed pair keeps content", outline("---\n# Visible\n\n# Also visible\n"), [
  [1, "Visible"],
  [1, "Also visible"],
])
// The `---` must be at the very start; later thematic breaks are not it.
eq("frontmatter: mid-document --- is a thematic break", outline("# A\n\n---\n# Visible\n"), [
  [1, "A"],
  [1, "Visible"],
])

// --- bind data blocks (plugin-bind, §19) ------------------------------------
// The pre-parse stage removes the whole block (verified with --format ast on
// `--plugin micromatter,markdown,bind`).
eq("bind: body block skipped", outline("# A\n\n{{{toml[dataY]\n# Comment\n}}}\n\n# B\n"), [
  [1, "A"],
  [1, "B"],
])
// A head line carrying its own `}}}` (a `(path)` payload) is one block.
eq("bind: one-line path block skipped", outline("{{{csv[rows](../data/x.csv)}}}\n\n# Real\n"), [
  [1, "Real"],
])
// §19: an unterminated block is left as text — its body keeps scanning.
eq("bind: unterminated block stays text", outline("{{{toml[dataY]\n# Visible\n"), [[1, "Visible"]])
// Indented `{{{` still opens a block (extract uses trim_start).
eq("bind: indented head opens a block", outline("  {{{toml[x]\n# Comment\n}}}\n\n# Real\n"), [
  [1, "Real"],
])

// --- selectionRange column: the title offset is reported --------------------
const [first] = scanHeadings('##[slug]("T")@@h{.lead} Visible Title')
// "##[slug](\"T\")@@h{.lead} " is 24 columns; the title starts at column 24.
eq("column: titleStartCol points at the visible title", first.titleStartCol, 24)

console.log(`heading-scan-test: ${checks} checks passed`)
