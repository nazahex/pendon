// Print the scope stack of every token of a Pendon file, so the grammar can be
// checked without launching VS Code:
//
//   node tools/check-scopes.mjs sandbox/demo/src/demo.mdp
//   node tools/check-scopes.mjs --md some.md       # tokenized as plain Markdown
//   node tools/check-scopes.mjs --mapped file.mdp  # only pendon/markdown scopes
//
// (`--md` is how the injection into the built-in Markdown grammar is checked.)
import { ROOT_SCOPE, readLines, tokenize } from "./grammars.mjs"

const args = process.argv.slice(2)
const file = args.find((a) => !a.startsWith("--"))
if (!file) {
  console.error("usage: node tools/check-scopes.mjs [--md] [--mapped] <file>")
  process.exit(2)
}

const mapped = args.includes("--mapped")
const root = args.includes("--md") ? ROOT_SCOPE.md : ROOT_SCOPE.mdp
const { lines, missing } = await tokenize(root, readLines(file))

lines.forEach((tokens, i) => {
  const kept = tokens.filter((t) => t.text.length > 0)
  if (kept.every((t) => t.text.trim() === "")) return
  console.log(`L${String(i + 1).padStart(3)} |`)
  for (const t of kept) {
    const scopes = mapped ? t.scopes.filter((s) => /pendon|markdown/.test(s)) : t.scopes
    console.log(
      `   ${String(t.startIndex).padStart(3)} ${JSON.stringify(t.text)} ${scopes.join(" ")}`,
    )
  }
})

if (missing.size) console.error(`[no grammar found for: ${[...missing].join(", ")}]`)
