# Active Context

## Current work focus

The docs restructure (this effort) plus **Phase 4** wiring of `plugin-list` L2/L3
(the `<li>` item layer). Container layers (`list`/`unordered`/`ordered`,
blockquote) have landed; the item layer is the next code task.

## Recent changes

- **VS Code heading breadcrumbs / Outline (`editors/vscode/src/headings.js` +
  `src/extension.js`, VS Code-only).** `.mdp` files (language id `pendon`) had no
  breadcrumbs because VS Code's built-in Markdown `DocumentSymbolProvider` only
  serves the `markdown` language. Added a VS Code-free §7.4 heading scanner
  (`src/headings.js`, mirroring `src/scan.js`) and registered a
  `DocumentSymbolProvider` for `pendon` in `src/extension.js`, so `.mdp` headings
  now appear in the breadcrumbs / Outline just like standard Markdown. The symbol
  shape **mirrors the built-in provider exactly** (read out of the installed
  bundle's `vscode-markdown-languageservice`): `name` = `"#".repeat(level) + " " +
  title` so marker depth stays visible, `range` = the heading's whole _section_
  (heading line → line before the next same-or-higher heading, else EOF) — what
  makes the breadcrumb chain follow the cursor's depth — and `selectionRange` =
  `range`. An earlier line-scoped `range` produced flat/odd breadcrumbs (a `...`
  in the bar as soon as the cursor left the heading line); section ranges +
  `#`-prefixed names fix that. `#` inside the four regions the engine never
  promotes to document-level headings is now skipped (each verified with
  `pendon run --format ast` on `micromatter,markdown,bind`): fenced code —
  including info-string openers like `` ```yaml ``, which the old bare-run regex
  missed — HTML comments, closed `---` frontmatter pairs at doc start, and
  `plugin-bind` `{{{lang[var] … }}}` blocks. Unclosed spans keep their
  headings (engine leaves them as text), matching micromatter/§19 exactly.
  Label = the visible title with the Pendon head
  (`[slug]` / `("title")` / `@@type{…}` /
  `{…}`, §7.4/§11) stripped and inline markdown reduced to plain text;
  `SymbolKind.String`, children nested by level. Behaviour was **verified against
  the real CLI**
  (`pendon run --format ast`), not guessed: ATX-only (no setext, §14), **no
  6-level cap** (`#######` = level 7), **no space required** (`#NoSpace`),
  trailing `#` **not** stripped, column-0 only (the `plugin-markdown` indented
  marker path is out of scope), head must be adjacent. `package.json` gained
  `onLanguage:pendon` (also fixes the pre-existing gap where the extension never
  activated for `.mdp`-only sessions, so the directive aids were inert there too)
  and `test` now also runs the new guard `tools/heading-scan-test.mjs` (18 checks
  over goldens 06/21/22 + edge cases, all green; Biome-clean). Scoped to `pendon`
  only — registering for `markdown` would run beside VS Code's built-in provider
  and duplicate symbols. See `editors/vscode/README.md` § "Heading breadcrumbs".

- **VS Code grammar coloring pass (`editors/vscode/syntaxes/pendon.tmLanguage.json`).**
  Fixed three real gaps found by tokenizing the goldens with the real engine
  (`tools/check-scopes.mjs`). (1) **Heading `captures` off-by-one** — all six
  `heading1..6` rules labelled the title group `4` and the trailing `#` run `5`,
  but the `(?=…)` before the title is a _non-capturing_ lookahead, so the title
  is really group `3` and the run `4`; the title (extras head, backtick slug,
  emphasis, trailing `##`) therefore never tokenized and shared the heading's
  `innerText` colour. Renumbered to `3`/`4` in all six. (2) **CSV bind body**
  (`§19`) had `"patterns": []`; added field scopes (comma, quoted / numeric /
  boolean / unquoted) under `meta.embedded.block.csv.bind.pendon`. (3) **Inline
  LaTeX** had no rules; added `#mathInline` (`$…$`) and `#mathDisplay` (`$$…$$`)
  to `#inlinePendon`, mirroring `plugin-latex`'s scanner (`\$` escape, no-space
  adjacency, `$100` stays literal). Verified img/anchor URLs and anchor URL
  modifiers were **already** scoped (`markup.underline.link[.image].pendon`,
  `keyword.operator.url-modifier.pendon`) — no change needed. `npm run check`
  green (72 fixtures); display math is line-bounded and blockquote inline rules
  fall back to Markdown — both noted in `editors/vscode/README.md` "Known gaps".

- **VS Code directive visual aids (`editors/vscode/src/`, VS Code-only).** The
  `::`/`==` syntax stays as-is; two aids draw on top of the existing
  `punctuation.definition.directive.begin/.end.pendon` scopes. `src/scan.js` is a
  VS Code-free decoration scanner that re-implements §10.2 (inline colon-run,
  close ≥ open, LIFO, line-bounded) and §10.3 (block `==type` … lone `==`, LIFO),
  honouring §4.2 escaping and inert code spans/fences; it returns matched marker
  runs + pairs with nesting depth. `src/extension.js` turns that into (B) a
  caret-tracking open/close **match highlight** using `editorBracketMatch.*`
  colours (always on, innermost pair wins) and (C) an **opt-in depth rainbow**
  (`pendon.directive.depthColors`). Rationale: VS Code's bracket matching /
  `editor.bracketPairColorization` / `editor.guides.bracketPairs` only work for
  _distinct_ single-character open/close pairs, so `::`/`==` can never use them —
  the only way to get the matching-hover feature (which a cancelled `::[[…]]::`
  bracket-fence proposal was trying to buy) is extension-code decoration.
  `package.json` gained `main`/`activationEvents`/`contributes.configuration` and
  `type: "commonjs"` (the `.mjs` tools stay ESM). Guarded by
  `tools/directive-scan-test.mjs` (`npm test`) — 20 checks over goldens 15/16 +
  escaping/code-span, all green; Biome-clean. See `editors/vscode/README.md` §
  "Directive visual aids".

- **VS Code TextMate grammars completed (`editors/vscode/`).** The `.mdp`
  grammar (`syntaxes/pendon.tmLanguage.json`) and the `.md` injection now cover
  every block **and** inline construct: inline directive (§10.2 — nested, closing
  run ≥ opening run, expressed with a `\1` back-reference in `end`), citation
  (§7.3), wiki (§7.5), figure/image (§7.1) and the anchor link with its §7.2 URL
  modifiers. Shapes were re-locked against the plugins
  (`plugin-img::parse_marker`/`has_modifiers`, `plugin-cite::parse_citation`,
  `plugin-wiki`, `extra::typed`) and verified two ways: token scope dumps
  (`tools/check-scopes.mjs`) against the goldens' `.jsx`, and live renders
  (`/tmp/pdimg` — a bare `!` plus an adjacent head → `<AdvancedImage>`, a
  mid-line `!!` → text `!` + an advanced image, `p!`/`d!` → `container`), which
  match the grammar's claims. New `tools/self-test.mjs` (`npm run check`) checks
  rule-reference integrity, declared scope names, and that every golden + sandbox
  fixture tokenizes; `editors/vscode/README.md` documents the harness and the
  known gaps (the harness/`\A` frontmatter gap, line-bounded inline directives,
  2-level URL parentheses, unscoped `p`/`d` container letters). The `$more` stub
  is gone, both syntaxes JSON are biome-expanded (repo JSON policy) and all
  touched Markdown is dprint-clean. **Bug found and fixed on the way:**
  `injectTo` is a VS Code _manifest_ concept that vscode-textmate does not know
  (it takes `injections[selector]` on the target), so the harness had to build
  that mapping — which promptly exposed that the manifest injected the rules into
  `text.html.markdown.pendon` as well, i.e. into the grammar that already
  includes them (infinite recursion, and the `.md` path was dead). `injectTo` is
  now `["text.html.markdown"]` and the selector excludes the Pendon scope; the
  self-test asserts the injection still applies.

- **TUI overhaul RFC (docs only — no code).** New
  `docs/rfc/tui-overhaul.md` is the design record for rewriting `crates/tui`: a
  single `Report<W>` sink (replacing the eight free `render_*` functions and
  `trait Paint`/`StyleExt`), one colour decision (`--color` → `NO_COLOR` →
  `CLICOLOR_FORCE` → `TERM=dumb` → TTY), capability-detected `Glyphs`
  (`ascii`/`unicode`/`nerd` so `--features nerd-fonts`/`LC_ALL=C` change real
  bytes), width-aware aligned blocks, a **synchronous** `Progress`/`Spinner` (no
  threads, no cursor races), a first-class `plain` mode for non-TTY runs, and a
  `Report::buffer(w)` test seam so the crate finally gets snapshot tests. All
  open-question recommendations are accepted/recorded. It **supersedes** the
  "`pendon-tui` hygiene pass" list in `cli-report-upgrade.md` (now a link), and
  `CLI-REPORT.md` §6, `AGENTS.md`, `INDEX.md` and `STATUS.md` were updated to route
  to it.

- **CLI report contract + upgrade RFC + `--quiet` (docs + one flag).** New
  `docs/spec/CLI-REPORT.md` is the normative contract for the stderr report
  (streams, `Theme`/glyph tokens, the four block shapes, TTY/`--quiet` gating,
  known gaps); new `docs/rfc/cli-report-upgrade.md` proposes the upgrade path
  (plain non-interactive report so CI stops losing the summary, `NO_COLOR`,
  verbosity ladder, `--report=json`, `pendon-tui` hygiene). Implemented `-q` /
  `--quiet` in `cli.rs` / `main.rs` / `run.rs` — it silences the `run` report and
  the `--tui` spinner only (stdout, exit codes and diagnostics untouched).
  Also confirmed `crates/tui` (`pendon-tui`) **is** used: `Theme`,
  `is_interactive_stderr`, `Spinner`, `render_status_line`,
  `render_severity_line`, `render_kv_list`. Dead/stale parts: `ProgressBar`,
  `render_progress_bar`, `render_bullet_list`, `Paint`, `Glyphs` are exported but
  unused; `src/mod.rs` is orphaned (declares a missing `locales` module);
  `unicode-width`/`once_cell` deps and the `ascii`/`nerd-fonts` features are
  declared but unused; the crate README documents a different API.

- **RFCs authored (diagnostics + VS Code plugin) — docs only, no code.**
  `docs/rfc/advanced-diagnostic.md` proposes the diagnostic system (stderr
  printer, producer-attached byte spans resolved to line/column in the CLI,
  `--strict` extended to plugin diagnostics, `P####` codes) and
  `docs/rfc/plugin-vsc.md` the VS Code extension. Key finding recorded in both:
  diagnostics are **silent** today — the CLI uses them only for the exit code and
  `Event::Diagnostic.span` is never populated (see the progress.md "Known issues"
  entry). Prerequisite for the VS Code plugin's Problems-pane support.
- **`plugin-bind` paths + merge + external files (latest — decided, implemented,
  frozen).** The RFC's §2.5–§2.8 opened four questions; the user accepted every
  recommendation, asked for the decisions to be recorded in the RFC, and then for
  the implementation. Landed: `is_valid_var` requires a leading `ALPHA`
  (`OPEN-BIND-1`, so `$100` is text); `Ref`/`Segment`/`parse_ref`/`lookup` give a
  reference a **path** (`$a.b`, `$rows[0].n`, `$m["k"]`, `OPEN-BIND-2`); a
  **spread** `{...$ref, k: v}` (`OPEN-BIND-3`) whose reference strings travel under
  `pendon_extra::SPREAD_KEY` (`"..."`) — an authored key wins over a spread, the
  later spread wins; and a **file payload** `{{{csv[rows](../data/rows.csv)}}}`
  (`OPEN-BIND-4`) via `extract_with_base(source, dir)` — base = the **source
  file's** directory at any depth, stdin = `Error`, and every file read is added
  to the CLI's cache `deps`. Goldens `23-bind` + new `24-bind-paths` (harness now
  copies `NN-name.data/` → `data/`); RFC §2.5–§2.8/§5/§6/§7 rewritten as decided,
  §15's `OPEN-BIND-1..4` moved to "Resolved decisions", `SYNTAX.md` §5/§19 made
  normative, READMEs + ADR-0004/-0005 updated, sandbox example added and rendered.
- **`plugin-bind` doc-debt pass (previous, **docs only** — no code touched):**
  ADR-0005 (structured extras values + the `U+E001` transport + the _emit_-pass
  rename); `SYNTAX.md` §5 (`object`/`array` values + depth-aware split rules),
  §6.3 (`AttrValue` + structured renderer column), §12.2 renamed
  "Post-markdown (emit)" with a not-`plugin-bind` note, §19 (`var` starts
  `ALPHA`, no-`$`-escape rationale, non-goals, proposals), §15
  `OPEN-BIND-1..4`; the RFC gained §2.5 paths, §2.6 merge/override, §2.7
  external data (deep discussion), §2.8 declined; `PARSER.md`,
  `crates/extra/README.md`, `crates/plugin-bind/README.md`, `AGENTS.md` aligned.
- **`plugin-bind` phase 1 (previous pass):** new crate + ADR-0004 + spec §19 + golden
  `23-bind`; CLI wired in both paths; transport foundation (JSON/JSX attr
  sentinels, nested extras, AST re-hydration, Solid `={…}`).
- **Docs split (earlier):** the oversized `TODO.md` and `SYNTAX.md` were split
  by concern. `STATUS.md` is now the live plan of record; `docs/decisions/` holds
  the D1–D10 rationale as ADRs; `docs/spec/golden/README.md` documents the
  executable-truth mechanism; `docs/guides/` holds how-tos; `memory-bank/` is
  Cline's working memory. `SYNTAX.md` keeps only normative grammar + fixture ref.
- **Prior work (frozen/landed):** unified typed-extras `@@type{…}` heads (D1–D10),
  decorator binder, `plugin-section` rework, §11 positional keys + empty-attr
  omission, positional groups `@@type[…](…){…}`, `plugin-list`/`plugin-blockquote`
  container layers.
- **Extras nested values + JSX brackets (this session):** `parse_value` now
  treats a `{ … }` group holding a single identifier/dotted path
  (`{valueA}`, `{frontmatter.fm1}`) as `AttrValue::JsxExpr` instead of rejecting
  the whole head (§4.3 literal fallback). Transported via `JSX_ATTR_PREFIX`
  (`U+E002`); `renderer-solid` emits `name={valueA}` unquoted in both the
  direct-node and `{...attrs}` template paths, `renderer-html` emits the escaped
  string, and object/array/`$var` bind behaviour (incl. `$theme` inside a nested
  value combined with non-bind values) is preserved. New unit tests in
  `crates/extra/tests/extras_spec.rs` (`nested_values`), golden fixture
  `25-extras-nested`, and regenerated `sandbox/extras/out/extras-basic.jsx`.

## Next steps

0. **`plugin-bind` — decide, then code.** DONE: `OPEN-BIND-1..4` are decided,
   implemented, tested and frozen (paths, spread/merge, external files; the RFC's
   §2.5–§2.8 is the decision record). What remains is **phase 2 `mdp`**
   (`U+E002` + the inline pipeline, and whether `mdp` gets a `(path)` payload) and
   the optional `[task.bind]` config table.
1. Fix the **L37 truncation** bug (shared head reader) — it blocks Step 3, Step 6b,
   golden `13`, and the D/B/N/E sections of `sandbox/ultimate`.
2. Land L2/L3 item layer + golden `13-list-item`.
3. Phase-4 cleanup: OPEN-R1 `:::` alias, archive `docs/rfc/unified-syntax.md`,
   rewrite `sandbox/unified`, trim `docs/todo/syntaxes.md`.
4. **CLI report upgrades** (`docs/rfc/cli-report-upgrade.md`, Phase 1): always emit
   a plain non-interactive report so CI keeps the summary, and honour `NO_COLOR` /
   `--color`. Then the diagnostics printer (`docs/rfc/advanced-diagnostic.md`).
5. **TUI overhaul** (`docs/rfc/tui-overhaul.md`) — the RFC landed; implementation
   awaits go-ahead. Cheapest first step is its **Phase 0**: bump `crossterm`, delete
   `crates/tui/src/mod.rs` + `once_cell` + the dead feature flags (no behaviour
   change), then Phase 1's `Report<W>` + `--color`.

## Active design threads

- **`plugin-bind` — DONE through §2.7** (ADR-0004, RFC `docs/rfc/plugin-bind.md`,
  spec §19). `crates/plugin-bind` runs a **pre-parse** `extract()` (removes
  `{{{lang[var] … }}}` blocks, parses JSON/JSONC/YAML/TOML/CSV **or reads a
  `(path)` file** to `serde_json::Value`) and a post-markdown `resolve()` (rewrites
  references — bare or a **path** — plus **spread** items in nested extras values
  to `JSON_ATTR_PREFIX` + compact JSON). Wired into `main.rs` + `process.rs`, with
  external files appended to the cache `deps`; goldens `23-bind` + `24-bind-paths`
  green. Remaining: `mdp` (phase 2, `JSX_ATTR_PREFIX` via the inline pipeline) and
  `[task.bind]` config.
- **Spread transport is a reserved key, not a new `AttrValue`.** `crates/extra`
  collects `...$ref` items into `SPREAD_KEY` (`"..."`, `src/value.rs`) as a JSON
  array of reference strings; `plugin-bind` expands it. `...` cannot be spelled as
  a prop `key`, so the marker is unreachable from a document. Because a JSON object
  here is key-sorted (no `preserve_order`), item order is gone before `resolve`
  runs — hence "an authored key wins over a spread" instead of §6.4's
  "later item wins" (RFC §2.6 records the revision).
- **Transport foundation (done).** `JSON_ATTR_PREFIX` (`U+E001`) /
  `JSX_ATTR_PREFIX` (`U+E002`) in `core/event.rs`; `AttrValue::Object`/`Array`
  - `to_json` in `extra/value.rs`; depth-aware extras splitting in
    `extra/typed.rs`; AST re-hydration in `renderer-ast/builder.rs`; Solid
    `name={…}` / `name={( … )}` in `renderer-solid/node.rs`.
- **Extraction must precede `parse`:** a payload's `#`, `-` and blank lines are
  otherwise lexed as Heading/List/block separators. Evidence in ADR-0004.

## Active decisions & considerations

- Docs **trust hierarchy**: executable fixtures > normative spec > ADRs > guides >
  STATUS. A fact lives in exactly one file; the rest link.
- `STATUS.md` must stay short — delete done lines; history is git + `archive/`.

## Learnings / insights

- Giant mixed-concern files caused AI agents to miss details; splitting by concern
  - a routing file (`AGENTS.md`) + an index (`docs/INDEX.md`) is the fix.
- The golden set is the strongest "always up to date" lever: a failing test forces
  a re-freeze, so docs can't silently drift from behaviour.
