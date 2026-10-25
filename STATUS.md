# STATUS — Live Plan of Record

**This is the single live source of truth for "what is happening now."**
It is intentionally small and changes every commit. History and rationale live
elsewhere — do **not** restate them here:

- **Normative grammar** → [`docs/spec/SYNTAX.md`](docs/spec/SYNTAX.md)
- **Why a decision was made (D1–D10)** → [`docs/decisions/`](docs/decisions/)
- **Completed work / historical essays** → [`docs/archive/`](docs/archive/)
- **How to run tests, goldens, sandboxes** → [`CONTRIBUTORS.md`](CONTRIBUTORS.md) & [`docs/guides/testing.md`](docs/guides/testing.md)

Update this file whenever you start, block, or finish a piece of work. When a
line is done, delete it (the git history is the record) — keep this file short.

---

## Current focus

Four proposals, one sequenced plan ([`docs/workflows/`](docs/workflows/), 40 tasks
in [`TASKS.md`](docs/workflows/TASKS.md)). **Phases 0–3 are landed**; Phase 4
(`plugin-extract`) is next.

- **Phase 0/1 — shared types.** `pendon-types` (`Selector`, `TypeSet`, `Rule`,
  `Group`, `winner`, `matching`) answers `type = …` selectors; `D-3a`: the
  renderer resolves each layer once per document and plugins stop choosing
  (ADR-0008). Every `[task.*.custom]` golden stayed byte-identical.
- **Phase 2 — exports.** `SolidRenderHints.exports: Vec<DataExport>`, written by
  `renderer-solid` alone; `frontmatter`/`headings` are automatic, `plugin-cite`
  pushes `references` then `citations`, `[[task.bind.inject]]` publishes a file
  (SYNTAX §20, ADR-0009).
- **Phase 3 — numbering.** `plugin-number` owns all numbering (ADR-0010):
  `[[task.<element>.number]]` + `[[task.number.custom]]`, the `%%numberType`
  marker parsed in `crates/extra` (§5, §10.5) and carried as `__number`,
  `attrs.number` written as an array. `plugin-heading` numbers nothing (its
  `auto_number`/`number_style` are a §14 migration error); `plugin-image` no
  longer `data-`prefixes `__` keys. Goldens `30-number-heading`,
  `31-number-custom` frozen, `06-heading` re-frozen (numbers gone). Found while
  re-rendering: `renderer-html` leaked `__layer` on a `Link` (it writes
  `href`/`title` itself, bypassing the `__` skip) — fixed + tested.

**Next:** Phase 4 — `plugin-extract` (T4.1). The only open blockers are the
`plugin-list` item layer (L37) and the sandbox/docs cleanup listed below.

## Open blockers

- **L37 truncation (`sandbox/ultimate`).** A heading whose line carries an extras
  head inside a list item over-scans past the line and re-emits the swallowed
  text inside the `<h*>`; the frozen baseline currently encodes a truncated body.
  Minimal repro:
  ```text
  ##[s-a] V1

  - item

    # [x]("T"){.d}body

  D01 after
  ```
  Owner: shared head reader (`crates/extra`) + the plugin that scans it
  (`plugin-anchor` / `plugin-heading`). After the fix: re-render, review the newly
  visible tail sections, re-freeze, add a regression case to the E section.
  **Blocks:** Step 6b (list-container sandbox cases L53–L55), the D/B/N/E sections
  of `sandbox/ultimate`, and golden `13-list-item`.

## In progress

- [ ] **Editor grammars (`editors/vscode/`).** Block + inline rules landed
      (`syntaxes/pendon.tmLanguage.json`, plus the injection that adds them to
      `.md`), and the harness (`tools/self-test.mjs`) is green on every golden
      and sandbox fixture. Coloring pass: fixed the heading `captures`
      off-by-one (title/extras/slug now tokenize; was lumped with `innerText`),
      added CSV bind-body field scopes, and added inline `$…$` + display
      `$$…$$` math (mirrors `plugin-latex`). Directive visual aids landed
      VS Code-side (`src/extension.js` + `src/scan.js`, guarded by
      `tools/directive-scan-test.mjs`): caret-tracking open/close match highlight
      (always on) and an opt-in `pendon.directive.depthColors` depth rainbow —
      the `::`/`==` syntax is unchanged. **Heading breadcrumbs/Outline landed**
      (`src/headings.js` + a `DocumentSymbolProvider` in `src/extension.js`,
      guarded by `tools/heading-scan-test.mjs`): `.mdp` headings now appear in
      the breadcrumbs with the Pendon head stripped and inline markdown reduced
      to plain text; symbol shape (section-scoped ranges, `#`-prefixed names)
      mirrors the built-in Markdown provider, so the breadcrumb chain follows
      the cursor's depth. `#` inside HTML comments, fenced code (info-string
      openers included), closed `---` frontmatter and `plugin-bind` data blocks
      is skipped — each region verified inert with `pendon run --format ast`.
      `package.json` gained
      `onLanguage:pendon` so the extension activates for `.mdp`. Left:
      package/publish the extension; the frontmatter gap, line-bounded display
      math, blockquote inline fallback and the other approximations are listed in
      [`editors/vscode/README.md`](editors/vscode/README.md).
- [ ] **RFCs: diagnostics + VS Code plugin (drafts).**
      `docs/rfc/advanced-diagnostic.md` and `docs/archive/fulfilled/plugin-vsc.md` authored; no
      code yet. Note: today diagnostics are silent (the CLI uses them only for the
      exit code, `--strict` covers only core-parser limit guards, and
      `Event::Diagnostic.span` is never populated).
- [ ] **CLI report: contract spec + upgrade RFC + `--quiet` (docs + one flag).**
      `docs/spec/CLI-REPORT.md` formalizes the stderr report (streams, `Theme`
      tokens, block layout, gating); `docs/rfc/cli-report-upgrade.md` proposes the
      upgrade path (plain non-interactive report so CI stops losing the summary,
      `NO_COLOR`, verbosity ladder, `--report=json`; the `pendon-tui` hygiene item
      now lives in `rfc/tui-overhaul.md`). `-q` /
      `--quiet` landed — it silences the `run` report and the `--tui` spinner only
      (stdout, exit codes, and diagnostics are untouched).
- [ ] **TUI overhaul RFC (docs only — no code).** `docs/rfc/tui-overhaul.md`
      proposes a ground-up rewrite of `crates/tui`: one `Report<W>` sink replacing
      the eight free `render_*` functions, one colour decision
      (`--color`/`NO_COLOR`), glyph sets that actually switch, width-aware blocks,
      a synchronous `Progress`/`Spinner` with no threads, a plain mode for non-TTY
      runs, and a buffer test seam. All open-question recommendations are accepted
      and recorded in the RFC. Supersedes the "`pendon-tui` hygiene pass" item in
      `cli-report-upgrade.md`.
- [ ] **Step 3 — L2/L3 item layer.** `plugin-list` item wrapper; `plugin-markdown`
      gains `CustomPlacement::ListItem` + `PendingListItem` / `arm_list_item` /
      `take_list_item`; L4 `start:` rejection warning; L5 nesting test; golden
      `13-list-item`. (blocked by L37)
- [ ] **Step 6b — ultimate sandbox (list container).** L53/L54/L55 container
      decorator + bracket group, unclaimed container type, pair lost to the anchor
      layer. (blocked by L37)
- [ ] **Step 7 — full gates.** `cargo test --workspace`, `cargo build --workspace`,
      `cargo fmt --check`, `git diff` review of every golden / baseline move.

## Deferred

- **Stale sandbox baselines (from Phases 1–2).** `sandbox/anchor`,
  `sandbox/custom/inline`, `sandbox/img/*`, `sandbox/list/*` and
  `sandbox/table/integ` still hold older output: `"cites"` where the exports are
  now `references` + `citations`, `class={"q"}` where a typed string now renders
  `class="q"`, `cra-1` where the cite id is `cite-1` (and
  `img/image/out/extra.html` drifts near L37). Re-rendering them is a reviewable
  separate pass — do not cite them as truth until then. `sandbox/number/` is a
  pre-implementation sketch (no source, non-runnable `pendon.toml`).

- **§11 threading gaps** (from the positional-keys pass):
  - `plugin-table`: `extras_to_layer(head)` still uses `ExtrasOptions::default()`;
    thread `TableOptions` into the 7-layer head scan.
  - `plugin-wiki`: `WikiOptions` carries no `ComponentSet`, so extras stay on the
    built-in `slug`/`title`; add `[task.wiki.custom]` to lift this.
  - `extract-heading`: `process(events)` takes no options; thread heading options
    if `[task.heading.custom]` renames `backtick_key`.
- **Phase-4 cleanup** (Step 7): OPEN-R1 `:::` deprecated alias with warning; archive
  `docs/rfc/unified-syntax.md`; rewrite `sandbox/unified`; trim
  `docs/todo/syntaxes.md`.
- **plugin-markdown gaps** (Step 5): fenced code inside a blockquote (Q14);
  `parse_blockquote_prefix` indentation + blank-`>` loose-list split.
- **Decorator space exception:** a decorator on a list item / blockquote may allow
  one space between marker and head (specified §4.1); exercised once L2/L3 lands.

## Golden & freeze status (see [`docs/spec/golden/README.md`](docs/spec/golden/README.md))

- Golden fixtures `01–12`, `14–16`, `21–28`, `30–31` are frozen and green; none
  `#[ignore]`d. `cargo test -p pendon --test syntax_spec` (25 fixtures) is the gate.
- `06-heading` was re-frozen for Phase 3: it loses the numbers `plugin-heading`
  used to compute, and keeps its authored `number:` props (they pass through now).
  `28-type-selectors` dropped its `auto_number` key with no output change.
