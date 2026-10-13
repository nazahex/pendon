"use strict"

// Pendon directive visual-assist entry point.
//
// Two non-intrusive aids, both VS Code-only (the `::`/`==` syntax is untouched):
//
//   B — *active pair*: when the caret sits inside (or on) a directive marker,
//       its matching open/close run is outlined, exactly like native bracket
//       matching. Uses the `editorBracketMatch.*` theme colours so it blends in.
//   C — *depth rainbow*: opt-in (`pendon.directive.depthColors`). Every matched
//       marker is tinted by its nesting depth, the way bracket-pair colorization
//       tints nested brackets.
//
// It also registers a `DocumentSymbolProvider` for the `pendon` language so
// Pendon headings appear in the breadcrumbs and Outline, just like standard
// Markdown (see `headingSymbols` below).
//
// The matching and heading scanning are `scan.js` / `headings.js`, VS Code-free
// re-implementations of the §10.2 / §10.3 / §7.4 rules; the TextMate grammar
// remains the source of truth for what *is* a directive or heading, and this
// layer only draws on top of it.

const vscode = require("vscode")
const { scanDirectives } = require("./scan")
const { scanHeadings } = require("./headings")

// Seven depth hues, echoing VS Code's own bracket-pair palette. Depth is capped
// at 7 by the grammar (colons/equals = 2..7 → nesting depth 1..6 here), so the
// last colour covers anything deeper.
const DEPTH_COLORS = ["#ffd700", "#da70d6", "#179fff", "#ff8c00", "#00d7af", "#98c379", "#c678dd"]

// Cache the last scan per document URI + version so cursor moves don't re-scan.
const cache = new Map()

/** Scan a document, memoized on its version. */
function scanFor(document) {
  const key = document.uri.toString()
  const hit = cache.get(key)
  if (hit && hit.version === document.version) return hit.result
  const result = scanDirectives(document.getText())
  cache.set(key, { version: document.version, result })
  return result
}

/** True when the caret is inside the marker run `[startCol, endCol)` on `line`. */
function withinMarker(marker, pos) {
  return (
    marker.line === pos.line && pos.character >= marker.startCol && pos.character <= marker.endCol
  )
}

/** True when `pos` lies anywhere in the span a pair covers (open .. close). */
function pairContains(pair, pos) {
  const { open, close } = pair
  if (open.line === close.line) {
    return pos.line === open.line && pos.character >= open.startCol && pos.character <= close.endCol
  }
  return pos.line >= open.line && pos.line <= close.line
}

/**
 * The pair to outline for a caret at `pos`: the pair a marker belongs to if the
 * caret is on a marker, else the innermost (deepest) pair whose span contains it.
 */
function activePair(pairs, pos) {
  let best = null
  for (const pair of pairs) {
    if (withinMarker(pair.open, pos) || withinMarker(pair.close, pos)) return pair
    if (pairContains(pair, pos) && (!best || pair.depth >= best.depth)) best = pair
  }
  return best
}

/** Turn a marker into the editor Range its run occupies. */
function markerRange(marker) {
  return new vscode.Range(marker.line, marker.startCol, marker.line, marker.endCol)
}

/** Build the seven depth decoration types plus the shared active-pair type. */
function createDecorations() {
  const depth = DEPTH_COLORS.map((color) =>
    vscode.window.createTextEditorDecorationType({
      // Tint the marker glyphs themselves; the underline reinforces it subtly.
      color,
      textDecoration: "underline",
      rangeBehavior: vscode.DecorationRangeBehavior.ClosedClosed,
    }),
  )
  const active = vscode.window.createTextEditorDecorationType({
    border: "1px solid",
    borderRadius: "2px",
    borderColor: new vscode.ThemeColor("editorBracketMatch.border"),
    backgroundColor: new vscode.ThemeColor("editorBracketMatch.background"),
    rangeBehavior: vscode.DecorationRangeBehavior.ClosedClosed,
  })
  return { depth, active }
}

/** Apply decorations for one editor. */
function update(editor, decos, config) {
  if (!editor?.document) return
  const lang = editor.document.languageId
  if (lang !== "pendon" && lang !== "markdown") {
    for (const d of decos.depth) editor.setDecorations(d, [])
    editor.setDecorations(decos.active, [])
    return
  }

  const { markers, pairs } = scanFor(editor.document)

  // C — depth rainbow (opt-in).
  if (config.get("pendon.directive.depthColors")) {
    const byDepth = decos.depth.map(() => [])
    for (const marker of markers) {
      const slot = Math.min(marker.depth, decos.depth.length) - 1
      byDepth[slot].push(markerRange(marker))
    }
    decos.depth.forEach((d, i) => {
      editor.setDecorations(d, byDepth[i])
    })
  } else {
    for (const d of decos.depth) editor.setDecorations(d, [])
  }

  // B — active pair under the caret.
  const pair = activePair(pairs, editor.selection.active)
  editor.setDecorations(decos.active, pair ? [markerRange(pair.open), markerRange(pair.close)] : [])
}

/** Re-render every visible editor (selection/document/language may differ). */
function refresh(decos, config) {
  for (const editor of vscode.window.visibleTextEditors) update(editor, decos, config)
}

// --- Breadcrumbs / Outline ---------------------------------------------------

/**
 * Build the heading symbol tree VS Code shows in the breadcrumbs and Outline.
 *
 * Mirrors the built-in Markdown extension (`vscode-markdown-languageservice`,
 * `TableOfContents` + `MdDocumentSymbolProvider`) on all three points that
 * shape how breadcrumbs read:
 *
 *   name  — `'#'.repeat(level) + ' ' + title`, e.g. `## Foo Bar` (head
 *           `[slug]("T")@@…` stripped, inline markdown reduced to text).
 *   range — the heading's whole *section*: from the heading line to the line
 *           before the next same-or-higher-level heading, else EOF. This is
 *           what makes the breadcrumb chain follow the cursor's depth — a
 *           heading stays in the chain while the cursor is anywhere inside
 *           its section, not only on the heading line.
 *   selectionRange — same as `range`, exactly like vanilla (a breadcrumb click
 *           therefore lands on the heading line).
 *
 * Children are nested by level (a deeper heading nests under the previous
 * shallower one), exactly like Markdown.
 */
function headingSymbols(document) {
  const roots = []
  // A stack of { level, symbol } for the currently-open ancestors.
  const stack = []
  const headings = scanHeadings(document.getText())

  for (let i = 0; i < headings.length; i++) {
    const heading = headings[i]
    // Section end: the line before the next heading that does not go deeper
    // than this one, else the last line of the document.
    let endLine = document.lineCount - 1
    for (let j = i + 1; j < headings.length; j++) {
      if (headings[j].level <= heading.level) {
        endLine = headings[j].line - 1
        break
      }
    }
    if (endLine < heading.line) endLine = heading.line // consecutive headings
    const range = new vscode.Range(heading.line, 0, endLine, document.lineAt(endLine).text.length)
    const symbol = new vscode.DocumentSymbol(
      `${"#".repeat(heading.level)} ${heading.label}`,
      "",
      vscode.SymbolKind.String,
      range,
      range,
    )

    while (stack.length && stack[stack.length - 1].level >= heading.level) stack.pop()
    if (stack.length) stack[stack.length - 1].symbol.children.push(symbol)
    else roots.push(symbol)
    stack.push({ level: heading.level, symbol })
  }

  return roots
}

/** The `DocumentSymbolProvider` registered for the `pendon` language. */
const pendonSymbolProvider = {
  provideDocumentSymbols(document) {
    if (document.languageId !== "pendon") return []
    return headingSymbols(document)
  },
}

function activate(context) {
  const config = vscode.workspace.getConfiguration()
  const decos = createDecorations()
  context.subscriptions.push(...decos.depth, decos.active)

  context.subscriptions.push(
    vscode.languages.registerDocumentSymbolProvider({ language: "pendon" }, pendonSymbolProvider),
    vscode.window.onDidChangeTextEditorSelection((e) => update(e.textEditor, decos, config)),
    vscode.window.onDidChangeActiveTextEditor((e) => update(e, decos, config)),
    vscode.workspace.onDidChangeTextDocument((e) => {
      cache.delete(e.document.uri.toString())
      for (const editor of vscode.window.visibleTextEditors) {
        if (editor.document === e.document) update(editor, decos, config)
      }
    }),
    vscode.workspace.onDidChangeConfiguration((e) => {
      if (e.affectsConfiguration("pendon.directive.depthColors")) refresh(decos, config)
    }),
  )

  refresh(decos, config)
}

function deactivate() {
  cache.clear()
}

module.exports = { activate, deactivate }
