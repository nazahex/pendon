# Progress

## What works (verified green)

- Workspace builds clean: `cargo build --workspace`.
- `cargo test --workspace` green.
- Golden fixtures `01–12`, `14–16`, `21–24` frozen and green; none `#[ignore]`d.
  Gates: `cargo test -p pendon --test syntax_spec` (19 fixtures) and
  `cargo test -p pendon --test ultimate_freeze_spec`.
- **`plugin-bind` — all of §2.1–§2.7** green: `cargo test -p pendon-plugin-bind`
  (32 unit tests, external files exercised in a temp dir),
  `cargo test -p pendon-extra` (39, incl. the spread transport) + goldens
  `23-bind` and `24-bind-paths`; wired into `apps/cli` both paths.
- `cargo fmt --check` clean for touched files; `dprint check` clean.
- **`--quiet` / `-q` landed.** Suppresses the `pendon run` report and the `--tui`
  spinner only (stdout, exit codes and diagnostics untouched). Contract:
  `docs/spec/CLI-REPORT.md`; upgrade plan: `docs/rfc/cli-report-upgrade.md`.
- **Editor grammars (`editors/vscode/`)** — block + inline rules complete
  (directive §10.2, citation §7.3, wiki §7.5, figure/image §7.1, anchor link
  §7.2). Coloring pass fixed the heading `captures` off-by-one (title/extras/
  slug now tokenize), added CSV bind-body field scopes (§19) and inline/display
  math (`plugin-latex`); img/anchor URLs + anchor URL modifiers were already
  scoped. `npm run check` (`tools/self-test.mjs`) is green: 2 grammars, 72
  fixtures, no dangling `#rule`s; every claimed shape was cross-checked against
  the goldens' `.jsx` and a live `pendon run` probe. Gaps (frontmatter,
  line-bounded display math, blockquote inline fallback) are listed in
  `editors/vscode/README.md`.
- **VS Code directive visual aids (`editors/vscode/src/`)** — VS Code-only, syntax
  untouched. `src/scan.js` (decoration scanner mirroring §10.2/§10.3 + §4.2) feeds
  `src/extension.js`: a caret-tracking open/close match highlight (always on,
  `editorBracketMatch.*` colours) and an opt-in `pendon.directive.depthColors`
  depth rainbow. Chosen because VS Code's bracket matching/colorization/guides
  need distinct single-char pairs, so `::`/`==` can never use them. Guarded by
  `tools/directive-scan-test.mjs` (`npm test`) — 20 checks green; Biome-clean.

## What is left to build

- **`plugin-bind` — nothing left from §2.5–§2.7.** `OPEN-BIND-1..4` are decided
  (RFC §2.5–§2.8 is the record), implemented, tested and frozen: paths
  (`$a.b[0]`, `$m["k"]`), spread/merge (`{...$base, k: v}`, shallow, an authored
  key wins over a spread), external files (`{{{csv[rows](../data/rows.csv)}}}`,
  base = the source file's directory at any depth, stdin errors, files recorded as
  cache deps).
- **`plugin-bind` phase 2:** `mdp` values (inline pipeline → `JSX_ATTR_PREFIX`
  and `prop={( … )}`), whether `mdp` gets a `(path)` payload (today it does not),
  `[task.bind]` config, an `mdp` golden.
- **L2/L3 list item layer** (blocked by L37 bug): item wrapper in `plugin-list`,
  `CustomPlacement::ListItem` in `plugin-markdown`, L4 `start:` warning, L5 nesting
  test, golden `13-list-item`.
- **L37 truncation fix** in `crates/extra` + `plugin-anchor`/`plugin-heading`; then
  re-render, review the newly visible tail, re-freeze `sandbox/ultimate`, add a
  regression case.
- **Phase-4 cleanup**: OPEN-R1 `:::` deprecated alias (warn → hard-error), archive
  `docs/rfc/unified-syntax.md`, rewrite `sandbox/unified`, trim
  `docs/todo/syntaxes.md`.
- **`pendon-tui` overhaul** — RFC landed (`docs/rfc/tui-overhaul.md`); no code yet.
  Phase 0: crossterm bump + delete `src/mod.rs`/`once_cell`/dead features. Phase 1:
  `Report<W>` sink, `--color`/`ColorChoice`, `Glyphs::detect`, width-aware blocks,
  synchronous `Progress`/`Spinner`, `plain` mode for non-TTY, `Report::buffer(w)`
  snapshots + `examples/report.rs`, README rewrite. Phases 2–3: diagnostics code
  frames, task sections + totals table, themes, OSC 8, `--watch`, presets, ETA.
- **VS Code extension**: package/publish `editors/vscode/` (`vsce package`) and
  close the gaps in its README (frontmatter, multi-line inline directives).
- **§11 threading gaps**: `plugin-table` layer options, `plugin-wiki` `ComponentSet`,
  `extract-heading` options.

## Known issues

- **`pendon-tui` is stale (in use, but partly dead).** The CLI uses `Theme`,
  `is_interactive_stderr`, `Spinner`, `render_status_line`,
  `render_severity_line`, `render_kv_list`; but `Paint`, `ProgressBar`,
  `render_progress_bar`, `render_bullet_list` are exported and unused (`Glyphs` is
  exported but only ever built internally as `ascii()`), `src/mod.rs` is orphaned
  (it declares a missing `locales` module), the `unicode-width`/`once_cell` deps
  and `ascii`/`nerd-fonts` features are unused (no `#[cfg(feature = …)]` exists),
  and the crate README documents a different API. Full plan:
  [`docs/rfc/tui-overhaul.md`](../docs/rfc/tui-overhaul.md) — the old
  `cli-report-upgrade.md` "hygiene pass" list is now a link to it. The `run` report
  is also dropped entirely when stderr is not a TTY.

- **CLI surfaces diagnostics lazily.** A `Severity::Error` sets the exit code (2)
  and appears in the `events` / `json` IR, but no renderer prints it — so a
  `solid` render of a stdin `(path)` block (the one hard error in `plugin-bind`)
  fails silently on screen. Pre-existing CLI behaviour, not `plugin-bind`'s; a
  diagnostics printer is the fix. Planned in
  [`docs/rfc/advanced-diagnostic.md`](../docs/rfc/advanced-diagnostic.md); note
  that `Event::Diagnostic.span` is never populated today, so locations require
  producer-side work first.
- `sandbox/ultimate` frozen baseline encodes a **truncated** body (L37 bug): the
  rendered `ultimate.jsx` stops after the L37 list item; L40–L46, D, B, N and the
  whole E section never render, though `headings` still lists them.
- `plugin-markdown`: fenced code inside a blockquote (Q14) unsupported;
  `parse_blockquote_prefix` over-consumes indentation; a blank `>` splits one list
  into two.

## Evolution of decisions (pointers)

- D1–D10 rationale → `docs/decisions/0001-typed-extras-reconciliation.md`.
- **Structured extras values + the `U+E001` transport → ADR-0005** (why nested
  `{…}`/`[…]` values exist, why only `plugin-bind` may emit `U+E001`, and why the
  extras glue is the _emit_ pass — not "binding").
- **Declined by decision** (RFC §2.8): `\$` escape, frontmatter/micromatter as a
  payload source, free-text interpolation, reactive/runtime binding. The
  `pendon.toml`-only external-data answer was replaced by
  `{{{lang[var](../path)}}}` (`OPEN-BIND-4`, now implemented).
- Progress notes that used to live in `SYNTAX.md` §17 → moved to `STATUS.md` /
  `archive/`; the spec now stays normative only.
- Completed-phase essays from the old `TODO.md` → `docs/archive/`.
