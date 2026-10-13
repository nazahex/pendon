// Validate this extension's TextMate grammars without launching VS Code:
//
//   node tools/self-test.mjs
//   NODE_PATH=/path/to/node_modules node tools/self-test.mjs
//
// Needs `vscode-textmate` + `vscode-oniguruma` (see tools/grammars.mjs). It
// checks that both grammars parse, that every `#rule` a grammar refers to exists
// and no rule is orphaned, that the declared scope names match the files, and
// that every fixture in the repository tokenizes without the engine failing.
import fs from "node:fs"
import path from "node:path"

const problems = []
const problem = (message) => problems.push(message)

let harness
try {
  harness = await import("./grammars.mjs")
} catch (error) {
  console.error(`cannot load tools/grammars.mjs: ${error.message}`)
  console.error("install vscode-textmate + vscode-oniguruma, or point NODE_PATH at a copy")
  process.exit(2)
}

const { EXT_DIR, INITIAL, ROOT_SCOPE, createRegistry, declaredGrammars, readLines } = harness
const ROOT = path.resolve(EXT_DIR, "..", "..")

/** Every `include` value in a grammar document, wherever it sits. */
function includes(node, out = []) {
  if (Array.isArray(node)) {
    for (const item of node) includes(item, out)
  } else if (node && typeof node === "object") {
    for (const [key, value] of Object.entries(node)) {
      if (key === "include" && typeof value === "string") out.push(value)
      else includes(value, out)
    }
  }
  return out
}

// --- grammars ------------------------------------------------------------------

const grammars = declaredGrammars()
const documents = new Map()
for (const [scope, { file }] of grammars) {
  try {
    documents.set(scope, JSON.parse(fs.readFileSync(file, "utf8")))
  } catch (error) {
    problem(`${scope}: ${error.message}`)
  }
}

const local = new Map()
const external = new Map()
for (const [scope, doc] of documents) {
  const refs = includes(doc)
  local.set(
    scope,
    refs.filter((ref) => ref.startsWith("#")).map((ref) => ref.slice(1)),
  )
  external.set(
    scope,
    refs.filter((ref) => !ref.startsWith("#")),
  )
}

/** Rules of a declared grammar that another declared grammar includes. */
const includedElsewhere = new Map([...documents.keys()].map((scope) => [scope, new Set()]))
for (const refs of external.values()) {
  for (const ref of refs) {
    const [target, rule] = ref.split("#")
    if (rule && documents.has(target)) includedElsewhere.get(target).add(rule)
  }
}

for (const [scope, doc] of documents) {
  if (doc.scopeName !== scope) problem(`${scope}: the file declares scopeName ${doc.scopeName}`)
  const rules = new Set(Object.keys(doc.repository ?? {}))
  const own = new Set(local.get(scope))
  for (const rule of own) {
    if (!rules.has(rule)) problem(`${scope}: no rule for #${rule}`)
  }
  for (const rule of rules) {
    if (!own.has(rule) && !includedElsewhere.get(scope).has(rule)) {
      problem(`${scope}: #${rule} is never included`)
    }
  }
}

// --- fixtures ------------------------------------------------------------------

const files = []
const addDir = (dir, keep) => {
  if (!fs.existsSync(dir)) return
  for (const name of fs.readdirSync(dir).sort()) {
    if (keep(name)) files.push(path.join(dir, name))
  }
}

addDir(
  path.join(ROOT, "docs", "spec", "golden"),
  (name) => name.endsWith(".md") || name.endsWith(".mdp"),
)
const sandbox = path.join(ROOT, "sandbox")
if (fs.existsSync(sandbox)) {
  for (const name of fs.readdirSync(sandbox).sort()) {
    addDir(path.join(sandbox, name, "src"), (n) => n.endsWith(".md") || n.endsWith(".mdp"))
  }
}

const { registry, missing } = await createRegistry()
for (const scope of ["text.html.markdown", "text.html.derivative"]) {
  if (missing.has(scope)) {
    problem(`${scope} is missing: set VSCODE_EXTENSIONS to a VS Code install`)
  }
}

const grammar = await registry.loadGrammar(ROOT_SCOPE.mdp)
for (const file of files) {
  try {
    let stack = INITIAL
    for (const line of readLines(file)) stack = grammar.tokenizeLine(line, stack).ruleStack
  } catch (error) {
    problem(`${path.relative(ROOT, file)}: ${error.message}`)
  }
}

// `injectTo` only lives in the manifest, so the harness has to add the injection
// grammar to its target: if that breaks, `.md` files silently lose every rule.
const markdown = await registry.loadGrammar(ROOT_SCOPE.md)
const injected = markdown
  .tokenizeLine('Wiki [[Wireless]] and cite [^^](book "hlm. 45") and ::note x::', INITIAL)
  .tokens.map((token) => token.scopes.join(" "))
  .join(" ")
for (const scope of [
  "meta.wikilink.pendon",
  "meta.citation.pendon",
  "meta.directive.inline.pendon",
]) {
  if (!injected.includes(scope)) problem(`not injected into text.html.markdown: ${scope}`)
}

// --- result --------------------------------------------------------------------

if (problems.length) {
  console.error(`pendon grammar self-test failed (${problems.length}):`)
  for (const item of problems) console.error(`  - ${item}`)
  process.exitCode = 1
} else {
  console.log(
    `ok: ${documents.size} grammars, ${files.length} fixtures, ${missing.size} unbundled scopes`,
  )
}
