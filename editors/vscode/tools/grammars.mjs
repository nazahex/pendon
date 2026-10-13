// Shared harness for the CLI helpers in this folder.
//
// The grammar is validated without launching VS Code: the extension's own
// `contributes.grammars` drives the registry, so what is checked is exactly what
// the editor loads. Needs `vscode-textmate` + `vscode-oniguruma` (install them
// here, or point NODE_PATH at an existing install).
import fs from "node:fs"
import { createRequire } from "node:module"
import path from "node:path"
import { fileURLToPath } from "node:url"

const HERE = path.dirname(fileURLToPath(import.meta.url))
export const EXT_DIR = path.resolve(HERE, "..")

const require = createRequire(import.meta.url)
const vsctm = require("vscode-textmate")
const oniguruma = require("vscode-oniguruma")

/** VS Code's bundled grammars, reused for every `include` we delegate to. */
const BUILTIN = {
  "text.html.markdown": "markdown-basics/syntaxes/markdown.tmLanguage.json",
  "text.html.derivative": "html/syntaxes/html-derivative.tmLanguage.json",
  "source.json": "json/syntaxes/JSON.tmLanguage.json",
  "source.json.comments": "json/syntaxes/JSONC.tmLanguage.json",
  "source.yaml": "yaml/syntaxes/yaml.tmLanguage.json",
}

const EXTENSION_ROOTS = [
  process.env.VSCODE_EXTENSIONS,
  "/usr/share/code/resources/app/extensions",
  "/usr/lib/code/resources/app/extensions",
  "/usr/local/share/code/resources/app/extensions",
  "/Applications/Visual Studio Code.app/Contents/Resources/app/extensions",
  "C:/Program Files/Microsoft VS Code/resources/app/extensions",
].filter(Boolean)

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"))
}

function findBuiltin(relPath) {
  for (const root of EXTENSION_ROOTS) {
    const file = path.join(root, relPath)
    if (fs.existsSync(file)) return file
  }
  return null
}

/** scopeName -> { file, injectTo } from this extension's package.json. */
export function declaredGrammars() {
  const pkg = readJson(path.join(EXT_DIR, "package.json"))
  const out = new Map()
  for (const g of pkg.contributes.grammars ?? []) {
    out.set(g.scopeName, {
      file: path.resolve(EXT_DIR, g.path),
      injectTo: g.injectTo ?? [],
    })
  }
  return out
}

/** scopeName -> file for every grammar this harness can load; `missing` holds
 * the `include`s no grammar was found for (e.g. `source.toml`, `source.csv`). */
export function grammarSources() {
  const declared = declaredGrammars()
  const sources = new Map()
  for (const [scope, { file }] of declared) sources.set(scope, file)
  const missing = new Set()
  for (const [scope, rel] of Object.entries(BUILTIN)) {
    const file = findBuiltin(rel)
    if (file) sources.set(scope, file)
    else missing.add(scope)
  }
  return { declared, sources, missing }
}

/** `injectTo` is a VS Code *manifest* concept: the editor hands each injection
 * grammar to its target as `injections[selector]`, so the harness does the same
 * (vscode-textmate itself only knows the `injections`/`injectionSelector` pair). */
function injectionRules(declared) {
  const byTarget = new Map()
  for (const { file, injectTo } of declared.values()) {
    if (!injectTo.length) continue
    const raw = vsctm.parseRawGrammar(fs.readFileSync(file, "utf8"), file)
    if (!raw.injectionSelector) continue
    for (const target of injectTo) {
      byTarget.set(target, [...(byTarget.get(target) ?? []), [raw.injectionSelector, raw]])
    }
  }
  return byTarget
}

/** A registry over the sources above, with every grammar preloaded so that
 * cross-grammar `include`s (and the injection into Markdown) resolve. */
export async function createRegistry() {
  const { declared, sources, missing } = grammarSources()
  const injections = injectionRules(declared)
  const wasm = fs.readFileSync(require.resolve("vscode-oniguruma/release/onig.wasm"))
  await oniguruma.loadWASM(wasm.buffer.slice(wasm.byteOffset, wasm.byteOffset + wasm.byteLength))

  const cache = new Map()
  const registry = new vsctm.Registry({
    onigLib: Promise.resolve({
      createOnigScanner: (patterns) => new oniguruma.OnigScanner(patterns),
      createOnigString: (s) => new oniguruma.OnigString(s),
    }),
    loadGrammar: async (scopeName) => {
      if (cache.has(scopeName)) return cache.get(scopeName)
      const file = sources.get(scopeName)
      if (!file) {
        missing.add(scopeName)
        cache.set(scopeName, null)
        return null
      }
      const raw = vsctm.parseRawGrammar(fs.readFileSync(file, "utf8"), file)
      const own = injections.get(scopeName)
      if (own) raw.injections = Object.fromEntries([...(raw.injections ?? []), ...own])
      cache.set(scopeName, raw)
      return raw
    },
  })

  for (const scope of sources.keys()) await registry.loadGrammar(scope)
  return { registry, missing }
}

/**
 * Tokenize `lines` with `scopeName` and return, per line, the scope stack of
 * every token: `{ lines: [{ text, startIndex, scopes }][], missing }`.
 */
export async function tokenize(scopeName, lines) {
  const { registry, missing } = await createRegistry()
  const grammar = await registry.loadGrammar(scopeName)
  if (!grammar) throw new Error(`no grammar for ${scopeName}`)
  let stack = vsctm.INITIAL
  const out = []
  for (const line of lines) {
    const res = grammar.tokenizeLine(line, stack)
    stack = res.ruleStack
    out.push(
      res.tokens.map((t) => ({
        text: line.slice(t.startIndex, t.endIndex),
        startIndex: t.startIndex,
        scopes: t.scopes,
      })),
    )
  }
  return { lines: out, missing }
}

export const ROOT_SCOPE = {
  mdp: "text.html.markdown.pendon",
  md: "text.html.markdown",
}

/** The state a `tokenizeLine` call needs for the first line of a document. */
export const INITIAL = vsctm.INITIAL

export function readLines(file) {
  return fs.readFileSync(file, "utf8").replace(/\r\n?/g, "\n").split("\n")
}
