# Pendon for VS Code

TextMate grammars for Pendon (`.mdp`), so files highlight without a language
server, plus a small runtime layer that draws directive open/close aids and
feeds Pendon headings to the breadcrumbs / Outline. **What** a construct means
is defined once, in
[`docs/spec/SYNTAX.md`](../../docs/spec/SYNTAX.md) — the grammar only tags the
text (each rule carries the `§` it encodes in its `comment`), and the runtime
layer only draws on top of that.

## What is here

| File                                        | Job                                                                                      |
| ------------------------------------------- | ---------------------------------------------------------------------------------------- |
| `syntaxes/pendon.tmLanguage.json`           | `text.html.markdown.pendon`: the `.mdp` grammar                                          |
| `syntaxes/pendon-injection.tmLanguage.json` | `pendon.injection`: injects the same rules into `text.html.markdown` for `.md`           |
| `src/extension.js`                          | runtime entry point: directive match/depth decorations + heading breadcrumbs (see below) |
| `src/scan.js`                               | VS Code-free directive-marker scanner the entry point draws from                         |
| `src/headings.js`                           | VS Code-free §7.4 heading scanner the breadcrumbs / Outline are built from               |
| `icons/pendon-icon-theme.json`              | file icon theme: gives `.mdp` its own Explorer icon (see below)                          |
| `tools/grammars.mjs`                        | harness: loads the declared grammars + the bundled VS Code ones                          |
| `tools/check-scopes.mjs`                    | prints the scope stack of every token in a file                                          |
| `tools/self-test.mjs`                       | checks rule references, scope names and that every fixture tokenizes                     |
| `tools/scope-cases.mjs`                     | one case per §5.2–§5.4 / §6.1 construct: the scopes a line must (not) produce            |
| `tools/directive-scan-test.mjs`             | drives `src/scan.js` against goldens 15/16 with plain `node` (no VS Code)                |
| `tools/heading-scan-test.mjs`               | drives `src/headings.js` against goldens 06/21/22 with plain `node` (no VS Code)         |

`.mdp` files get their own Explorer icon through a **file icon theme**
(`contributes.iconThemes` → `icons/pendon-icon-theme.json`), which maps the
`mdp` extension (and the `pendon` language) to `pendon-light.svg` /
`pendon-dark.svg`. Only `.mdp` selectors are styled, so every other file and
folder keeps its built-in icon. The theme is **opt-in**: pick _Pendon_ under
**File Icon Theme** (`workbench.iconTheme`) to turn it on — until then `.mdp`
files keep the active theme's generic icon.

There is deliberately **no** `icon` on the `pendon` language contribution. A
language `icon` is only a _fallback_: VS Code shows it for **any** file whose
language has no specific icon in the active file icon theme, so it leaks onto
unrelated files (the behaviour behind
[microsoft/vscode#206494](https://github.com/microsoft/vscode/issues/206494)).
The scoped file icon theme above is the supported way to give `.mdp` its icon
without touching any other file.

The `.mdp` grammar keeps the built-in Markdown block/inline dispatch and puts
the Pendon rules in front of it, so anything Pendon does not override (§4.3)
keeps its Markdown highlighting. All Pendon scopes are suffixed `.pendon` under
a `meta.<construct>.pendon` root scope (e.g. `meta.anchor.pendon`,
`meta.citation.pendon`), so a theme can target them without touching Markdown.

## Scopes

The grammar tags text and nothing else; **what** each construct means is defined
once, in [`docs/spec/SYNTAX.md`](../../docs/spec/SYNTAX.md). This table is the
mapping from construct to scope — the grammar's `comment` on each rule names the
same `§`, and `tools/scope-cases.mjs` asserts one case per row.

| Construct                                  | Scope                                                                                                      | §          |
| ------------------------------------------ | ---------------------------------------------------------------------------------------------------------- | ---------- |
| slot tag `@` `^` `!` `#`                   | `keyword.other.slot.tag.{anchor,cite,image,heading}.pendon`                                                | §5.3, D-11 |
| slot bracket / paren delimiters            | `punctuation.definition.slot.{bracket,paren}.{begin,end}.pendon`                                           | §5.2       |
| bare / quote / tick content                | `string.unquoted.slot.bare.pendon`, `string.quoted.{double,single}.slot…`, `string.other.slot.tick.pendon` | §5.2       |
| modifier run \|?~uv$%                      | `punctuation.definition.slot.modifier.begin.pendon` + `keyword.other.slot.modifier.pendon`                 | §5.4       |
| `[x]` / `("x")` groups on a claimed head   | `meta.slot.bracket.pendon` / `meta.slot.paren.pendon` (no tag)                                             | §5.2, §6.1 |
| `specs.class` / `specs.id` / `specs.style` | `meta.specs.{class,id,style}.pendon`                                                                       | §6.1       |
| numbering marker `%%type`                  | `meta.number-mark.pendon` (`punctuation.definition.number.pendon`, `entity.name.type.number.pendon`)       | §5, §10.5  |
| `@@` head / decorator line                 | `meta.head.pendon` inside `meta.decorator.pendon`                                                          | §9.1, D-10 |
| anchor / image / citation head             | `meta.anchor.pendon` / `meta.image.pendon` / `meta.citation.pendon`                                        | §7.1–§7.3  |

`specs.tick` and `specs.quote` keep the text scopes they share with a head's
values (`entity.name.slug.pendon`, `string.quoted.double.pendon`), because the
same rule reads both an item and a `k: v` value; a value never claims a
`meta.specs.*` scope. Config keys (`field.specs.tick`, `bracket.modifier."?"`, …)
live in `pendon.toml`, which is TOML's business — the grammar colours the markup
those keys name, not the config file.

The §5/§10.5 numbering marker gets one rule of its own (`#numberMark`,
`meta.number-mark.pendon`): the `%` run is `punctuation.definition.number.pendon`
and the type name `entity.name.type.number.pendon`. The regex bounds the run on
both sides (`(?<!%)(%{2,7})(?!%)`), so the three literal cases of D-9 — a single
`%`, `%%` without a name, and a run of 8+ `%` — stay unstyled rather than being
half-highlighted.

## Checking a change

Both tools run the real engine (`vscode-textmate` + `vscode-oniguruma`), so what
they report is what the editor loads; they need those two packages reachable —
install them beside this `package.json`, or point `NODE_PATH` at a copy:

```sh
NODE_PATH=/path/to/node_modules node tools/self-test.mjs
NODE_PATH=/path/to/node_modules node tools/scope-cases.mjs
NODE_PATH=/path/to/node_modules node tools/check-scopes.mjs sandbox/demo/src/demo.md
NODE_PATH=/path/to/node_modules node tools/check-scopes.mjs --md --mapped some.md
```

`npm run check` runs the first two (the structural one and the construct one).
`--md` tokenizes as plain Markdown (how the injection is checked) and `--mapped`
keeps only the Pendon/Markdown scopes. Compare against the fixture's `.jsx`
(`docs/spec/golden/`) to see what the engine really renders.

`cargo run --bin pendon -- run -F` (see [`docs/guides/cli.md`](../../docs/guides/cli.md))
is the other half: the grammar must agree with what the plugins emit.

`npm test` runs the two VS Code-free scanners with plain `node` — no VS Code, no
npm deps, so it works anywhere `node` does. `directive-scan-test.mjs` drives
`src/scan.js` against goldens 15/16 plus escaping/code-span cases (the guard for
the visual aids below); `heading-scan-test.mjs` drives `src/headings.js` against
goldens 06/21/22 plus the edge cases verified against the real CLI (the guard for
the breadcrumbs below).

## Directive visual aids (`src/`)

The grammar already scopes the open and close runs of a directive differently
(`punctuation.definition.directive.begin.pendon` / `.end.pendon`, §10.2/§10.3), so
a theme can colour them apart. What it cannot do — because TextMate has no nesting
state and VS Code's bracket matching/colorization needs _distinct_ single-character
open/close pairs, which `::`/`==` are not — is match a same-character pair or tint
by depth. `src/extension.js` adds both, VS Code-side only; the `::`/`==` syntax is
untouched:

- **Match highlight (always on).** With the caret inside (or on) a directive, its
  matching open/close run is outlined using the `editorBracketMatch.*` theme
  colours — the same cue as native bracket matching. The innermost pair wins.
- **Depth rainbow (opt-in).** `pendon.directive.depthColors` tints every matched
  marker by nesting depth, echoing VS Code's own bracket-pair palette.

`src/scan.js` is the VS Code-free engine behind both: it re-implements the §10.2
inline (colon-run, close ≥ open, LIFO) and §10.3 block (`==type` … lone `==`, LIFO)
rules, and honours §4.2 escaping plus inert code spans/fences. It is a
**decoration** scanner, not a parser — the grammar stays the source of truth for
what _is_ a directive; this layer only draws on top. Its approximations mirror the
grammar's own (line-bounded inline pairs, ≤7-wide runs), and `npm test` pins the
pairing against the goldens.

## Heading breadcrumbs (`src/headings.js`)

VS Code builds the breadcrumbs (and the Outline / "Go to Symbol") from a
`DocumentSymbolProvider`. The built-in Markdown provider only serves the
`markdown` language, so `.mdp` files (language id `pendon`) had **no** breadcrumbs
at all. `src/extension.js` registers a provider for `pendon` so Pendon headings
appear there just like standard Markdown. `package.json` also gained
`onLanguage:pendon` so the extension activates when a `.mdp` file is opened.

`src/headings.js` is the VS Code-free engine behind it, mirroring the §7.4 rules
and the real engine (verified against `pendon run --format ast`):

- **ATX only.** A `#` run at column 0 is a heading; setext (`Title` + `----`)
  never existed in Pendon (§14).
- **Head stripped.** The optional `[slug]` / `("title")` / `@@type{…}` / `{…}`
  head (§7.4 + §11) is removed, so `##[slug]("T")@@h{.lead} Visible Title` shows
  as `Visible Title` — the same text the engine renders. A head that is _not_
  adjacent to the marker (`## Title {.x}`) stays literal, as in the engine.
- **No 6-level cap.** Pendon accepts `#######` (level 7+) as a real heading,
  unlike CommonMark; the scanner reports the raw `#` count as the level.
- **No space required.** `#NoSpace` is a level-1 heading.
- **Plain-text label.** Inline markdown is reduced to its text (`**bold**` →
  `bold`, `` `code` `` → `code`, `[t](u)` → `t`, `[[a|b]]` → `b`), matching how
  the built-in Markdown outline concatenates a heading's text children.
- **Fenced code is inert.** A `#` inside a `` ``` `` / `~~~` block is skipped;
  the opener may carry an info string (`` ```yaml ``), the closer must be a
  bare run.
- **Comments, frontmatter and data blocks are inert too.** `#` inside an HTML
  comment (`<!-- … -->`), inside a closed `---` frontmatter pair at the document
  start, or inside a `plugin-bind` `{{{lang[var] … }}}` block (§19) never reaches
  the engine's document level — verified with `pendon run --format ast` on
  `micromatter,markdown,bind` — so the scanner skips those spans. An _unclosed_
  comment runs to EOF (CommonMark), while an unclosed frontmatter pair or bind
  block stays ordinary text (micromatter/§19 leave it intact) and its headings
  keep counting.

Each heading becomes a `DocumentSymbol` (`SymbolKind.String`, like Markdown),
mirroring the built-in provider (`vscode-markdown-languageservice`, read from
the installed VS Code bundle) on all three points that shape the breadcrumbs:

- `name` is `"#".repeat(level) + " " + title` — e.g. `## Foo Bar` — so the
  marker depth stays visible in the breadcrumb bar.
- `range` is the heading's whole **section**: the heading line to the line
  before the next same-or-higher-level heading, else EOF. This is what makes
  the breadcrumb chain follow the cursor's depth — a heading stays in the
  chain while the cursor is anywhere inside its section, exactly like vanilla
  Markdown (`# A > ## B` while you're in `## B`'s body, not just on its line).
- `selectionRange` equals `range`, like vanilla, so a breadcrumb click lands
  on the heading line.

Symbols are nested by level (a deeper heading nests under the previous
shallower one), exactly like Markdown. Only the `pendon` language is served —
registering for `markdown` too would run alongside VS Code's built-in provider
and duplicate the symbols.

## Known gaps

- **Heading breadcrumbs are column-0 only.** `src/headings.js` treats a `#` run
  at the start of a line as a heading (the `core` lexer's rule, and how every
  fixture is written). The `plugin-markdown` re-lexer also accepts an indented
  marker **with a following space** (`## x`), but that secondary path is not
  modelled here — an indented marker shows no breadcrumb. This matches the
  column-0 rule and never fires on a real Pendon document.
- **Frontmatter.** `text.html.markdown#frontMatter` (the built-in rule, included
  by `#block`) does not fire in the harness — plain Markdown shows the same, so
  it is a harness gap rather than a Pendon rule. Unknown in VS Code itself.
- **Inline directives are line-bounded.** A `::type … ::` pair must open _and_
  close on one line; a directive closing further down a paragraph stays literal.
- **Inline directive closers.** `\1` (the opening run) is used for the
  "closing run at least as long" rule, so an unterminated opener never opens
  (§4.3); a closer longer than seven colons is not modelled.
- **Inline directive head groups (`::note[slug]("level") …`)**. Inside `.md` the
  bracket/paren groups wear the inline link scopes instead of the `#slotGroup`
  ones; the `.mdp` grammar scopes them correctly.
- **Heading heads are scoped leniently.** `#headingN` scopes any run of
  `[x]` / `(x)` groups touching the `#` run, and the breadcrumb depth is the `#`
  count. §7.4 is stricter: one bracket plus _its_ paren (a lone `("title")` is
  text, §4.3), and a claimed head spends one `#` on the tag (D-11), so
  `###[slug]("Title")` is depth **2** although it scopes as `markup.heading.3`.
  Colouring never depends on the depth, so this is cosmetic — the scanner reports
  what the `#` run says.
- **URLs** nest parentheses two levels deep; deeper nesting is not matched.
- **A delimiter only closes at a boundary** (§5.2). `"x"` closes a quote content
  only when a space, `]` or `)` follows, exactly like the parser — so
  `[bare"quote"]` is _one_ bare content to the engine but colours as bare text
  plus an unstyled `"`, because a slot's kind is decided by its first character
  and TextMate cannot see that the delimiter never closed.
- **A `|` inside a table cell still splits the cell.** The table rules cut a row
  at every `|` before the inline rules run, so a modifier run (`|?x`) written in
  a cell loses its cell; a slot inside a cell without a run is unaffected. Fixing
  it means teaching `#table` the slot grammar, which waits for the table rework.
- **Display math is line-bounded.** `plugin-latex` renders `$$…$$` across lines
  (its `toggle_display_math_on_line`), but the grammar's `mathDisplay` rule
  (`meta.math.display.pendon`) matches a `$$…$$` pair on one line only; a
  display block that opens on one line and closes on another stays literal,
  like the line-bounded inline directives above. Inline `$…$`
  (`meta.math.inline.pendon`) is unaffected. Mirrors the plugin's own scanner:
  `\$` is a literal dollar, a `$` touching a space neither opens nor closes, and
  an unclosed `$` (e.g. `$100`) stays literal.
- **Pendon inline rules inside a blockquote.** A `>` line is delegated to the
  built-in `text.html.markdown#blockquote`, whose content uses Markdown's own
  `#inline` — so citation / wiki / anchor / math fall back to Markdown scopes
  there (the same for `.md` and `.mdp`). This predates the math/CSV work and
  affects every inline construct equally, not just these.
