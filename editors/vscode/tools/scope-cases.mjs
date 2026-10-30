// One grammar case per Stage-2 construct, so the grammar cannot drift from
// `docs/spec/SYNTAX.md` without a red run:
//
//   node tools/scope-cases.mjs
//   NODE_PATH=/path/to/node_modules node tools/scope-cases.mjs
//
// A case is one line plus an ordered list of `[text, scope]` pairs the tokenizer
// must produce **in that order**; tokens no pair names are skipped, so a case
// pins the construct rather than the document. `reject` lists scopes that must
// not appear at all — that is how a retired spelling is pinned as literal text.
// What a construct *is* stays in `SYNTAX.md`; this file only checks the scopes
// the grammar emits for it.
import { ROOT_SCOPE, tokenize } from "./grammars.mjs"

const CASES = [
  {
    name: "@@ first, then slots, specs and numbering",
    spec: "§9.1, D-10",
    line: '@@quoteA[intro]("Aside title"){.q, #q1, k: "v"}%%%numberA',
    expect: [
      ["@@", "punctuation.definition.head.sigil.pendon"],
      ["quoteA", "entity.name.type.pendon"],
      ["intro", "string.unquoted.slot.bare.pendon"],
      ["Aside title", "string.quoted.double.slot.pendon"],
      [".", "punctuation.definition.class.pendon"],
      ["#", "punctuation.definition.id.pendon"],
      ["%%%", "punctuation.definition.number.pendon"],
      ["numberA", "entity.name.type.number.pendon"],
    ],
  },
  {
    name: "a decorator without a type",
    spec: "§9.1",
    line: '@@[intro2]("Another title")%%numberA',
    expect: [
      ["@@", "punctuation.definition.head.sigil.pendon"],
      ["intro2", "string.unquoted.slot.bare.pendon"],
      ["numberA", "entity.name.type.number.pendon"],
    ],
  },
  {
    name: "a head line without @@ stays literal",
    spec: "§14, D-10",
    line: "{.lead, #nope}",
    expect: [["{.lead, #nope}", "meta.paragraph.pendon"]],
    reject: ["meta.decorator.pendon", "meta.head.pendon"],
  },
  {
    name: "the anchor tag",
    spec: "§5.3, §7.2",
    line: "An @[inner text](/docs) inline.",
    expect: [
      ["@", ["meta.anchor.pendon", "keyword.other.slot.tag.anchor.pendon"]],
      ["inner", "string.unquoted.slot.bare.pendon"],
      ["/docs", "string.unquoted.slot.bare.pendon"],
    ],
  },
  {
    name: "the citation tag sits inside the bracket",
    spec: "§5.3, §7.3",
    line: '[^book "hlm. 45"](paperback "Reprint")',
    expect: [
      ["[", "meta.citation.pendon"],
      ["^", "keyword.other.slot.tag.cite.pendon"],
      ["book", "variable.other.reference.cite.pendon"],
      ["hlm. 45", "string.quoted.double.slot.pendon"],
      ["paperback", "string.unquoted.slot.bare.pendon"],
      ["Reprint", "string.quoted.double.slot.pendon"],
    ],
  },
  {
    name: "the image tag",
    spec: "§5.3, §7.1",
    line: "![Alt text](x.webp)",
    expect: [
      ["!", ["meta.image.pendon", "keyword.other.slot.tag.image.pendon"]],
      ["Alt", "string.unquoted.slot.bare.pendon"],
      ["x.webp", "string.unquoted.slot.bare.pendon"],
    ],
  },
  {
    name: "the heading tag",
    spec: "§5.3, §7.4",
    line: '##[slug]("Heading title") Heading text',
    expect: [
      ["##", "markup.heading.2.pendon"],
      ["slug", "string.unquoted.slot.bare.pendon"],
      ["Heading title", "string.quoted.double.slot.pendon"],
    ],
  },
  {
    name: "the three contents, in order",
    spec: "§5.2",
    line: "@[bare \"quote\" 'single' `tick`](/u)",
    expect: [
      ["bare", "string.unquoted.slot.bare.pendon"],
      ["quote", "string.quoted.double.slot.pendon"],
      ["single", "string.quoted.single.slot.pendon"],
      ["tick", "string.other.slot.tick.pendon"],
    ],
  },
  {
    name: "the modifier run, and a paren has none",
    spec: "§5.4",
    line: "@[two symbols |?!](/u?x|?y)",
    expect: [
      ["|", "punctuation.definition.slot.modifier.begin.pendon"],
      ["?!", "keyword.other.slot.modifier.pendon"],
      ["/u?x|?y", "string.unquoted.slot.bare.pendon"],
    ],
  },
  {
    name: "the specs positions",
    spec: "§6.1",
    line: '@@t{`tick`, "quote", .cls, #id, --v: "1"}',
    expect: [
      ["tick", "entity.name.slug.pendon"],
      ["quote", "string.quoted.double.pendon"],
      [".", "meta.specs.class.pendon"],
      ["#", "meta.specs.id.pendon"],
      ["--", "meta.specs.style.pendon"],
    ],
  },
  {
    name: "numbering alone as a head",
    spec: "§5, §10.5",
    line: "##%%numberB",
    expect: [
      ["##", "markup.heading.2.pendon"],
      ["%%", "punctuation.definition.number.pendon"],
      ["numberB", "entity.name.type.number.pendon"],
    ],
  },
  {
    name: "an untagged slot pair belongs to its head",
    spec: "§5.2, §6.1",
    line: '{{type}}[x]("y"){a: 1}',
    expect: [
      ["{{", "punctuation.definition.marker.begin.pendon"],
      ["x", "string.unquoted.slot.bare.pendon"],
      ["y", "string.quoted.double.slot.pendon"],
      ["a", "entity.other.attribute-name.pendon"],
    ],
  },
  {
    name: "a Markdown link is not an anchor",
    spec: "§7.2, §14",
    line: "[text](/u)",
    expect: [
      ["[", "meta.link.pendon"],
      ["text", "string.other.link.title.pendon"],
      ["/u", "markup.underline.link.pendon"],
    ],
    reject: ["meta.anchor.pendon"],
  },
]

const failures = []

for (const testCase of CASES) {
  const { lines } = await tokenize(ROOT_SCOPE.mdp, [testCase.line])
  const tokens = lines.flat().filter((token) => token.text.length > 0)

  for (const scope of testCase.reject ?? []) {
    if (tokens.some((token) => token.scopes.includes(scope))) {
      failures.push(`${testCase.name}: ${scope} must not be emitted`)
    }
  }

  let cursor = 0
  for (const [text, scope] of testCase.expect) {
    let found = -1
    for (let index = cursor; index < tokens.length; index++) {
      const token = tokens[index]
      const wanted = Array.isArray(scope) ? scope : [scope]
      if (token.text === text && wanted.every((w) => token.scopes.includes(w))) {
        found = index
        break
      }
    }
    if (found < 0) {
      const wanted = Array.isArray(scope) ? scope.join(" + ") : scope
      failures.push(`${testCase.name}: ${JSON.stringify(text)} never carries ${wanted}`)
      break
    }
    cursor = found + 1
  }
}

if (failures.length) {
  console.error(`pendon scope cases failed (${failures.length}/${CASES.length}):`)
  for (const failure of failures) console.error(`  - ${failure}`)
  process.exitCode = 1
} else {
  console.log(`ok: ${CASES.length} grammar cases`)
}
