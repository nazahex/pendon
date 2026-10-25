# Pendon CLI Report Upgrade RFC

This RFC proposes how to make `pendon`'s **stderr report** — chiefly the summary
printed by `pendon run` — more useful, human-friendly, modern, and visually
appealing, without disturbing the rendered document on stdout.

The behaviour as it exists today is written down normatively in
[`docs/spec/CLI-REPORT.md`](../spec/CLI-REPORT.md); that file is the contract,
this file is the change proposal. Diagnostics are a separate concern — see
[`docs/rfc/advanced-diagnostic.md`](advanced-diagnostic.md).

## Current State

- The report has two producers, both in `apps/cli`, both rendered by
  `crates/tui` (`pendon-tui`): the per-task `run` summary and the `--tui`
  spinner.
- `crates/tui` consists of very old initial code that has not been touched since Pendon was first created; its overhaul is [`rfc/tui-overhaul.md`](tui-overhaul.md).
- Styling is a single `Theme` (five colours) plus a `Glyphs` set
  (`✔ ✖ ◆ ▲` and bullet `-`); severity lines pair a glyph with a bracketed label
  (`⟦done⟧ ⟦info⟧ ⟦warn⟧ ⟦error⟧`) so colour is redundant.
- **The report is dropped when stderr is not a TTY** (`is_interactive_stderr()`),
  so in CI/pipelines the only signal is the exit code.
- `-q` / `--quiet` exists and suppresses the report and the spinner.
- Known gaps (`spec/CLI-REPORT.md` §6): `ascii()` is really Unicode and `nerd()`
  is an alias (the `ascii` / `nerd-fonts` features are dead); two colour helpers
  coexist (`theme::Paint` and a private `StyleExt`); `render_severity_line`
  double-spaces after the glyph; `ProgressBar` / `render_progress_bar` /
  `render_bullet_list` are exported but unused; there is no `NO_COLOR` support;
  `crates/tui/src/mod.rs` is orphaned dead code (it declares a `locales` module
  that does not exist, and `Cargo.toml` points the lib at `lib.rs`); the
  `unicode-width` and `once_cell` deps are declared but unused; and the crate
  README documents a different API than the CLI actually calls.

## Goals

1. **Useful everywhere** — the same run must be readable interactively _and_
   legible in a CI log.
2. **Human-friendly and consistent** — one block grammar, predictable spacing
   and units, no surprise one-off shapes.
3. **Modern and visually appealing** — tasteful colour, alignment, and progress,
   with the option to turn decoration off.
4. **Accessible** — no colour-only meaning; honour `NO_COLOR` and friends.
5. **Machine-friendly** — an opt-in structured summary for tooling.
6. **Backwards compatible** — stdout and exit codes unchanged.

## Must-Have Features

1. **Always emit a plain report when stderr is not a TTY.**
   Today the report vanishes in CI. Instead, when `is_interactive_stderr()` is
   false, print a **plain, undecorated, append-only** summary to stderr — no
   colours, no `\r` in-place rewrites:

   ```
   pendon: docs -> out/docs.html (html)  3 files, 1.2 kB, 3 processed
   pendon: done — 1 task, 3 files, 0 failed, 12 ms
   ```

   Decoration stays for a TTY. A `--report=<auto|pretty|plain|json|none>` knob
   selects the mode explicitly; `auto` is the TTY heuristic.

2. **Respect the colour environment.**
   Centralise the colour decision in one place in `pendon-tui`:
   `--color=<auto|always|never>`, then `NO_COLOR` (set ⇒ never), then
   `CLICOLOR_FORCE` (set ⇒ always), then `TERM=dumb` / non-TTY (⇒ never). Colour
   is never the only carrier of meaning (§3 of the contract already requires
   this).

3. **Normalise the layout.**
   One space (not two) between glyph and label; uniform two-space indentation;
   a single bullet glyph; align `key:` columns within a block. Fix the
   `render_severity_line` double space.

4. **A verbosity ladder.**
   Define exactly what each level adds, and make `--quiet` the bottom:
   `-q` (silent report) < default (report) < `-v` (per-file lines) <
   `-vv` (also cache decisions). `--quiet` stays diagnostic-safe (it never
   silences diagnostics, warnings, or errors).

5. **A final tally.**
   End every `run` with one summary line that stands alone: tasks, files
   scanned / written / skipped / failed, total bytes, and elapsed time — plus a
   non-zero `Failed:` count when anything failed, so the failure is visible
   rather than only implied by the exit code.

6. **Host diagnostics.**
   The report is where the diagnostics printer from
   `rfc/advanced-diagnostic.md` should live (same `Theme`, same block grammar),
   with `--strict` escalation reflected in the final tally.

## Should-Have Features

1. **`--report=json` — a structured summary.**
   Emit the same fields as the tally (§Must-Have 5) as one JSON object on
   stderr for tooling and the future VS Code extension
   ([`archive/fulfilled/plugin-vsc.md`](../archive/fulfilled/plugin-vsc.md)):
   ```json
   {
     "duration_ms": 12,
     "tasks": [
       { "name": "docs", "format": "html", "input": "src/*.md",
         "output": "out/{id}.html", "scanned": 3, "written": 2,
         "skipped": 1, "failed": 0, "bytes": 1228 }
     ],
     "totals": { "scanned": 3, "written": 2, "skipped": 1, "failed": 0,
                 "bytes": 1228 }
   }
   ```

2. **`pendon-tui` hygiene pass → owned by the toolkit RFC.**
   The crate rewrite (dead code, the duplicate colour traits, the inert features,
   the README) now has its own design record:
   [`rfc/tui-overhaul.md`](tui-overhaul.md). Nothing about `crates/tui`'s internals
   is specified here any more — see that file's §Current State and Must-Have 8.

3. **Width-aware layout → owned by the toolkit RFC.**
   Aligning `key:` columns and `…`-truncating long paths is a `pendon-tui`
   concern: [`rfc/tui-overhaul.md`](tui-overhaul.md) Must-Have 4.

4. **Per-task sections.**
   For multi-task `pendon.toml` runs, print a heading per `[[task]]`
   (its `name`, else `task-N`) so the output reads as addressable sections
   rather than one run-on list. Rendered by `Report::section`, designed in
   [`rfc/tui-overhaul.md`](tui-overhaul.md) Should-Have 2.

5. **Progress for long runs → owned by the toolkit RFC.**
   The determinate bar, the indeterminate spinner, and the removal of the
   exported-but-unused `ProgressBar` / `render_bullet_list` are
   [`rfc/tui-overhaul.md`](tui-overhaul.md) Must-Have 5.

## Nice-to-Have Features

1. **Output hyperlinks (OSC 8)** — link the written `out/…` path so supporting
   terminals make it clickable. Rendering:
   [`rfc/tui-overhaul.md`](tui-overhaul.md) Should-Have 4.
2. **`--report-file <path>`** — tee the report to a file for CI artifacts.
3. **Spinner frame sets** — a `dots`/`line` choice behind the (currently dead)
   `nerd-fonts` feature; [`rfc/tui-overhaul.md`](tui-overhaul.md) Nice-to-Have 1.
4. **Locale-safe glyph fallback** — fall back to pure-ASCII glyphs when the
   terminal encoding is not UTF-8; `Glyphs::detect()` in
   [`rfc/tui-overhaul.md`](tui-overhaul.md) Must-Have 3.

## Open Questions (resolved)

All four recommendations below were **accepted by the maintainer**, so they are
decisions rather than open threads. They stay listed as the decision record — the
same convention as [`plugin-bind`](../archive/fulfilled/plugin-bind.md) §2.5–§2.8.

1. **Is the plain non-interactive report on by default, or opt-in?**
   **Decided: on by default** — CI silently losing the summary is the bug — with
   `--quiet` / `--report=none` to silence it. The plain mode that makes this
   possible is [`rfc/tui-overhaul.md`](tui-overhaul.md) Must-Have 6.
2. **Does `--quiet` also silence diagnostics?**
   **Decided: no.** Diagnostics are actionable; `--quiet` only silences the
   report. Diagnostic verbosity is a separate knob
   ([`rfc/advanced-diagnostic.md`](advanced-diagnostic.md), Should-Have
   "Diagnostic Suppression").
3. **Where does the format live?**
   **Decided:** keep [`docs/spec/CLI-REPORT.md`](../spec/CLI-REPORT.md) normative
   and this RFC as the proposal; the spec is updated when a shape lands.
4. **stderr only, or a file too?**
   **Decided: stderr now**; add `--report-file` later without changing the
   default.

## Adoption Strategy

- **Phase 1 (Must-Have):** plain non-interactive report, `NO_COLOR` / `--color`
  centralisation, layout normalisation; `--quiet` already landed. This alone
  fixes "CI sees nothing".
- **Phase 2 (Should-Have):** `--report=json`, width-aware layout, per-task
  sections. The `pendon-tui` internals these depend on have their own RFC:
  [`rfc/tui-overhaul.md`](tui-overhaul.md).
- **Phase 3 (Nice-to-Have):** hyperlinks, `--report-file`, frame sets, watch
  reprint.

## Backwards Compatibility

- **stdout is untouched**, so the rendered document and every piped workflow are
  unaffected; exit codes are unchanged.
- Interactive TTY output stays visually recognisable; the spacing normalisation
  (§Must-Have 3) is the only visible change.
- New flags (`--report`, `--color`, `-v`) are additive. `--quiet` is opt-in.
- The one _behavioural_ change is that a non-TTY run will now print a plain
  summary where it previously printed nothing. Scripts that parse stderr should
  be pinned with `--report=none` (or `--quiet`); this is called out here because
  it is the only case where existing stderr consumers could notice a difference.
