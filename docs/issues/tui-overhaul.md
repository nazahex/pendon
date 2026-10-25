# Pendon TUI Overhaul RFC

`crates/tui` (`pendon-tui`) is the only thing standing between `pendon` and a
terminal, and it has not been touched since the project's inception. It shows: the
crate is eight free functions with no model, half of it is dead, its two colour
APIs disagree with each other, its "glyph sets" do not switch, its spinner ignores
the theme it is handed, and its `README.md` documents an API the CLI never calls.

This RFC proposes a ground-up overhaul. The target is a terminal experience that
is **fancy, modern and genuinely practical** — aligned blocks, real progress,
correct colour, a summary you can read in a CI log — without ever risking the one
thing Pendon is actually for: byte-exact output on **stdout**.

Three documents divide the concern; this one owns only the toolkit:

| Concern                                 | File                                                        |
| --------------------------------------- | ----------------------------------------------------------- |
| What the report _is_ (normative)        | [`docs/spec/CLI-REPORT.md`](../spec/CLI-REPORT.md)          |
| What the report should _become_         | [`docs/rfc/cli-report-upgrade.md`](cli-report-upgrade.md)   |
| Diagnostics rendering                   | [`docs/rfc/advanced-diagnostic.md`](advanced-diagnostic.md) |
| The toolkit that draws all of it (here) | `docs/rfc/tui-overhaul.md`                                  |

> Status: proposal; no code yet. `pendon-tui` is **internal** — `apps/cli` is its
> only consumer, verified by `grep` across the workspace — so its Rust API may
> break freely. Decisions marked **Agreed** were confirmed by the maintainer when
> reviewing the CLI report RFC, and the same policy is applied here.

## Current State

### The crate as it exists

```text
crates/tui/
  Cargo.toml            # crossterm 0.27, is-terminal 0.4, once_cell 1, unicode-width 0.1
  README.md             # documents an API the CLI does not call
  src/
    lib.rs              # the real lib root ([lib] path); `pub use` re-exports everything
    mod.rs              # ORPHAN — never compiled; declares a `locales` module that does not exist
    theme.rs            # `Theme` (5 colours) + `trait Paint`      (unused by the CLI)
    assets.rs           # `Glyphs` + `ascii()`/`nerd()` + two spinner frame sets
    templates.rs        # the free `render_*` functions + a second, private colour trait
    widgets/
      mod.rs
      spinner.rs        # `Spinner::start/stop` — background thread + AtomicBool
      progress_bar.rs   # `ProgressBar`                            (unused)
```

`Cargo.toml` declares `default = ["ascii"]`, `ascii` and `nerd-fonts`, but no
`src/` file contains a single `#[cfg(feature = …)]`, so every feature is inert.

### What the CLI actually calls

| Symbol                                  | Call site                 | Notes                              |
| --------------------------------------- | ------------------------- | ---------------------------------- |
| `is_interactive_stderr()`               | `main.rs` ×1, `run.rs` ×3 | the only gate                      |
| `Theme::default()`                      | `main.rs`, `run.rs`       | passed but mostly ignored          |
| `widgets::spinner::Spinner`             | `main.rs`                 | `--tui` only                       |
| `render_status_line`                    | `run.rs`                  | in-place `\r` line                 |
| `render_severity_line` + `SeverityLine` | `run.rs` ×2               | `Found: n` / `All tasks completed` |
| `render_kv_list`                        | `run.rs` ×2               | the `› Wrote:` block               |

Exported but **never called**: `trait Paint`, `ProgressBar`, `render_progress_bar`,
`render_bullet_list`, `nerd()`, `SPINNER_FRAMES_DOTS`. `Glyphs` is exported but only
ever constructed internally, always as `ascii()`.

### Why it feels dated

1. **No model.** The crate is eight free functions and no state. Rendering is five
   of them, each of which re-opens `io::stderr()`, re-constructs `ascii()`, and
   decides nothing. Nothing owns the destination, the style, or the gating, so the
   CLI has to ask `is_interactive_stderr()` at four call sites and no part of the
   output can be tested.
2. **Two colour APIs.** `theme::Paint` (public) and `templates::StyleExt` (private)
   both define `fn with(self, color) -> StyledContent<String>`. `theme.rs` imports
   `crossterm::style::Stylize`; `templates.rs` does not — so the same method name
   resolves to two different traits in two files.
3. **Glyph sets that do not switch.** `ascii()` returns `✔ ✖ ◆ ▲` — Unicode, not
   ASCII — and `nerd()` is literally an alias for it. `--features nerd-fonts` is a
   no-op and `TERM=dumb` still gets UTF-8 glyphs.
4. **The theme is thrown away.** `Spinner::start(message, _theme)` accepts a
   `Theme` and drops it; `SPINNER_FRAMES_ASCII` is hard-wired regardless of
   capability.
5. **The animation has no owner.** `Spinner` runs its own thread writing to stderr
   while `render_status_line` / `render_severity_line` write to stderr from the
   main thread, with no shared cursor state and no `Hide`/`Show`. It only works
   today because the CLI happens to stop the spinner before drawing the report.
6. **`\r` with no width awareness.** Every pretty line assumes the terminal will
   overwrite the previous one, but there is no width source, no
   `Clear(UntilNewLine)` and no truncation, so one long path smears the frame.
7. **Almost no capability detection.** Colour and glyph choice are decided by a
   single fact (is stderr a TTY?). There is no `NO_COLOR`, no `TERM=dumb`, no
   `CI`, no UTF-8 fallback, no `--color`, no width.
8. **Untestable.** Everything writes to the real `io::stderr()`; there is no buffer
   target, no fixed width and no injected clock, so no widget can be asserted on.
   This repo's entire culture is byte-frozen fixtures — and the TUI has zero tests.
9. **Dead weight.** `src/mod.rs` is orphaned; `once_cell` is declared and never
   used; `unicode-width` is declared and never used either (this RFC puts it to
   work — Must-Have 4); the `ascii` / `nerd-fonts` features select nothing; and
   `README.md` describes `Spinner`/`ProgressBar`/`Glyphs` as if they were the public
   API.

## Goals

1. **Fancy and modern** — one coherent visual language: aligned columns, boxed
   sections, a real progress bar, tasteful colour that degrades gracefully.
2. **Practical** — the toolkit must serve the jobs the CLI actually has: per-file
   progress, per-task sections, a final tally, diagnostics, and eventually a
   watch/rebuild view.
3. **Readable** — width-aware layout that never wraps or smears, humanised units,
   and meaning that survives without colour.
4. **Testable and deterministic** — rendering to a buffer at a fixed width, with no
   clock and no TTY, so every block is a freezable, diffable fixture.
5. **Small and honest** — one colour API, one glyph API, no dead code, no dead
   features, a README that matches the code.
6. **Safe** — stdout untouched and exit codes unchanged; `--quiet` still silences
   exactly the report; pipelines and CI stay clean.

## Non-Goals

- **Not a full-screen TUI.** No alternate screen, no keybindings, no `ratatui`
  event loop, no mouse. Pendon is a one-shot CLI; a full-screen app would break
  pipelines and every consumer that pipes stderr.
- **Never writes to stdout.** The document is the only thing on stdout
  (CLI-REPORT §1). If a future feature needs stdout, that is a separate decision.
- **Not a logging framework.** No timestamps, log levels, or log-file rotation.
- **Not a colour-only design.** Colour is always redundant with a glyph and a
  bracketed label (CLI-REPORT §3).
- **No new rendering dependency** unless it buys determinism we cannot build
  ourselves (see Open Question 2).

## Design Overview

### Proposed crate layout

```text
crates/tui/src/
  lib.rs        # re-exports: `Report`, `Style`, `ColorChoice`, `Glyphs`, widgets
  style.rs      # `Style` + the single colour API + capability degradation
  glyphs.rs     # capability-detected glyph sets (ascii / unicode / nerd)
  width.rs      # display width, truncation, column alignment, humanised units
  report.rs     # `Report<W>` — the one sink + the block grammar
  line.rs       # the line/cursor manager shared by status, spinner and progress
  widgets/
    progress.rs # `Progress` (determinate bar) and `Spinner` (indeterminate)
```

Every current file is replaced; `templates.rs`, `assets.rs`, `theme.rs` and
`mod.rs` disappear. This is deliberately a rewrite rather than a patch, because the
defects above are architectural (no state) rather than cosmetic.

### The `Report` type

One type owns the destination, the style, the glyphs, the width and the gating.

```rust
pub enum ColorChoice { Auto, Always, Never }

pub struct Report<W: Write> {
    out: W,
    style: Style,
    glyphs: Glyphs,
    color: bool,   // resolved once, at construction
    width: usize,  // resolved once, at construction
    pretty: bool,  // in-place updates allowed?
}

impl Report<io::Stderr> {
    /// Pretty on a usable TTY, plain otherwise — the mode CLI-REPORT §5 describes.
    pub fn stderr(color: ColorChoice) -> Self;
}
impl Report<io::Sink> {
    /// `--quiet`: every method becomes a no-op. One gate instead of four.
    pub fn null() -> Self;
}
impl Report<Vec<u8>> {
    /// Deterministic test target: fixed width, no colour, no clock.
    pub fn buffer(width: usize) -> Self;
}
impl<W: Write> Report<W> {
    pub fn is_pretty(&self) -> bool;
    pub fn status(&mut self, msg: &str);                     // in-place `\r` line
    pub fn severity(&mut self, sev: Severity, title: &str);  // `✔ done  title`
    pub fn kv(&mut self, head: &str, items: &[(&str, &str)]);
    pub fn bullets(&mut self, head: &str, items: &[&str]);
    pub fn section(&mut self, title: &str);                  // Phase 2
    pub fn frame(&mut self, diag: &Diagnostic, src: &str);   // Phase 2 (diagnostics)
    pub fn finish(&mut self);                                // clear line, flush
}
```

The CLI then reads as one stream instead of eight detached calls:

```rust
let mut report = if quiet {
    Report::null()
} else {
    Report::stderr(args.color)
};
report.status("Scanning source files...");
report.kv("› Wrote:", &items);
report.severity(Severity::Done, "All tasks completed");
```

`Report::null()` is the whole of `--quiet`: the four `!quiet &&` guards in `run.rs`
and the `!args.quiet` in `main.rs` collapse into one constructor.

### Colour resolution — one place

| Order | Signal                           | Result |
| ----- | -------------------------------- | ------ |
| 1     | `--color=never`                  | plain  |
| 2     | `--color=always`                 | colour |
| 3     | `NO_COLOR` set (non-empty)       | plain  |
| 4     | `CLICOLOR_FORCE` set (non-empty) | colour |
| 5     | `TERM=dumb`                      | plain  |
| 6     | stderr is not a TTY              | plain  |
| 7     | otherwise                        | colour |

`--color=auto` means steps 3–7. This is the _implementation_ of the gating that
[`CLI-REPORT.md`](../spec/CLI-REPORT.md) §5 states normatively — the table stays
there, this is where it is executed. `Style` carries the five semantic roles
(`primary`/`success`/`warning`/`error`/`muted`) as today, plus true-colour →
256-colour → 16-colour degradation driven by `COLORTERM` / `TERM`.

### Glyph capability — actually selected

| Preset      | tick       | cross      | info       | warn       | bullet     | bar fill / track        |
| ----------- | ---------- | ---------- | ---------- | ---------- | ---------- | ----------------------- |
| `ascii()`   | `+`        | `x`        | `i`        | `!`        | `-`        | `#` / `-`               |
| `unicode()` | `✔`        | `✖`        | `◆`        | `▲`        | `•`        | `█` / `░`               |
| `nerd()`    | `\u{f00c}` | `\u{f00d}` | `\u{f05a}` | `\u{f071}` | `\u{f0da}` | `\u{2588}` / `\u{2591}` |

| Preset      | Spinner frames                                   |
| ----------- | ------------------------------------------------ |
| `ascii()`   | `-` `\` `\|` `/`                                 |
| `unicode()` | braille `⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏`                             |
| `nerd()`    | a Nerd Font frame set (gated behind the feature) |

`Glyphs::detect()` chooses `nerd` when the `nerd-fonts` feature is on, `unicode`
when the locale and `TERM` say UTF-8, and `ascii` otherwise (`TERM=dumb`,
`LC_ALL=C`, `LANG=C`, non-UTF-8). This removes the current lie where `ascii()`
returns Unicode.

### Progress and the line model

A single `Line` owns the status row: `set(text)` clears and rewrites it in place,
`commit()` ends it with `\n`, `clear()` erases it. `Spinner` and `Progress` render
_through_ the `Report`, never around it, so they can never fight over the cursor —
the race in today's design disappears by construction.

Critically, **no widget owns a thread**. `Progress` advances when the caller
advances it, which is exactly what the CLI's existing `for entry in WalkDir…` loop
already does; `Spinner` is an indeterminate mode of the same widget, ticked by the
caller. Rendering stays synchronous, which is what makes it deterministic,
testable, and impossible to interleave badly with the report.

### Phase 0

Bump `crossterm` to the current release in a separate, mechanical commit before the
rewrite starts, so the overhaul targets today's API rather than a 2023-era one.
`crossterm::terminal::size()` already provides the width source, so **no new
dependency is required for Phase 1** and `unicode-width` (already declared and
currently unused) becomes the display-width primitive.

## Must-Have Features (Phase 1 — the overhaul)

1. **One sink: `Report<W>`.**
   Replace the eight free functions with the type above. It owns the destination,
   so the CLI stops re-opening `io::stderr()`, and it owns the gating, so `--quiet`
   becomes `Report::null()` instead of five scattered `if !quiet`.

2. **One colour decision.**
   `ColorChoice` implemented exactly once, in `pendon-tui`, and consumed by the
   `--color=<auto|always|never>` flag proposed in
   [`cli-report-upgrade.md`](cli-report-upgrade.md) Must-Have 2. `trait Paint` and
   `trait StyleExt` both disappear.

3. **One glyph API that actually switches.**
   `Glyphs::detect()` + the `ascii`/`unicode`/`nerd` presets, so `--features
   nerd-fonts`, `TERM=dumb` and `LC_ALL=C` change real bytes. Severity always pairs
   a glyph with a bracketed label (`[done]` / `[warn]`), never colour alone.

4. **Width-aware layout.**
   Width from `crossterm::terminal::size()` → `COLUMNS` → 80; display width from
   `unicode-width`; `key:` columns aligned inside a block; long values truncated
   with an ellipsis; in-place updates clear to end of line. The result is a report
   that looks deliberate at 40 columns and at 200, and never smears.

5. **A real progress model.**
   `Progress::new(total)` → `set`/`inc` → `finish`, plus `Spinner` as its
   indeterminate mode, both driven synchronously by the caller's own loop. This is
   what the config runner's per-file loop has been missing: today it prints
   `Scanning source files...` once and then goes quiet for the whole run.

6. **A `plain` mode that is a first-class citizen.**
   Append-only, colourless, glyph-safe, byte-stable, and produced automatically
   when stderr is not a usable TTY. `cli-report-upgrade.md` Must-Have 1 (CI must
   stop losing the summary) is entirely dependent on this and cannot ship without
   it.

7. **A deterministic test seam.**
   `Report::buffer(width)` with fixed width, no colour and no clock, plus
   `crates/tui/tests/report.rs` snapshotting every block in `pretty`/`plain` ×
   widths `40`/`80`/`120` × `ascii`/`unicode`. This would be the crate's first
   test; the repo's "fixtures are the truth" rule then applies to the TUI like it
   does everywhere else.

8. **Delete the dead weight and tell the truth.**
   Remove `src/mod.rs`, `theme.rs`, `templates.rs`, `assets.rs`, `trait Paint`,
   `trait StyleExt` and the `once_cell` dependency; either wire `ProgressBar` /
   `render_bullet_list` into the new API or drop them; make the `ascii` /
   `nerd-fonts` features real or remove them; rewrite `crates/tui/README.md` as an
   accurate API reference that links to `spec/CLI-REPORT.md` for the contract
   instead of restating it.

## Should-Have Features (Phase 2)

1. **Diagnostics code frames.**
   `Report::frame()` — a rustc-shaped `error:`/`warning:` header, an
   `--> file:line:col` line, a gutter, the source snippet and an underlined span.
   This is the printer
   [`advanced-diagnostic.md`](advanced-diagnostic.md) Must-Have 1 asks for, and it
   must never be silenced by `--quiet` (CLI-REPORT §5 silences the report only).

2. **Task sections and a totals table.**
   `Report::section(name)` per `[[task]]`, and a final right-aligned table of
   files, bytes, cache hit-rate and duration. This is the "modern" part users
   actually notice, and it is the natural home for
   [`cli-report-upgrade.md`](cli-report-upgrade.md) Must-Have 5.

3. **Themes.**
   `Style::{auto, dark, light, plain}` with correct 24-bit → 256 → 16-colour
   degradation, best-effort dark/light detection via `COLORFGBG`, and an optional
   `[cli.theme]` table in `pendon.toml` for accent colour and colour blindness
   presets.

4. **Hyperlinks (OSC 8).**
   Clickable `out/…` paths in pretty mode; the bare path in plain mode, so piping
   still yields usable text.

5. **A `--watch` live view.**
   Rebuild on change while keeping one screen-stable summary — a status line and a
   repainted report — instead of appending a whole new report per rebuild. The
   synchronous `Progress` model is what makes this safe to build.

6. **Shared formatting utilities.**
   Humanised bytes and durations (`1.2 kB`, `12 ms`) and `n/total` counters in
   `width.rs`, so every block agrees on units and no call site formats its own.

## Nice-to-Have Features (Phase 3)

1. **Spinner presets** (`line`, `dots`, `braille`, `nerd`) selected by
   `--spinner=<name>`, replacing the two hard-coded frame constants.
2. **Throughput and ETA** (`files/s`, `MB/s`) once a usable sample exists,
   suppressed below a threshold so short runs do not jitter.
3. **Sparkline / histogram** of per-item bytes or timing, with an ASCII-safe
   fallback in `ascii()` mode.
4. **Multi-column layout** on very wide terminals, degrading to one column.
5. **A tee target** (`--report-file <path>`) so a plain report can be archived
   while a pretty one is shown; pairs with
   [`cli-report-upgrade.md`](cli-report-upgrade.md) Nice-to-Have 2.
6. **`--report=json`** (the structured summary in
   [`cli-report-upgrade.md`](cli-report-upgrade.md) Must-Have 3) rendered from the
   same data model as the human report, so the two can never drift.
7. **Locale / i18n** — only if the `locales` idea hinted at by the orphaned
   `src/mod.rs` ever returns; it would live in a `Locale` value, not in `Report`.
   Explicitly **not** now.

## API Migration Map

| Today                                               | After                                           | Notes                                     |
| --------------------------------------------------- | ----------------------------------------------- | ----------------------------------------- |
| `is_interactive_stderr()`                           | `Report::stderr(ColorChoice)`                   | gating moves inside; CLI stops asking     |
| `render_status_line(msg, theme)`                    | `report.status(msg)`                            |                                           |
| `render_severity_line(sev, title, theme)`           | `report.severity(sev, title)`                   | the double space is fixed here            |
| `render_kv_list(head, items, theme)`                | `report.kv(head, items)`                        | aligned `key:` columns                    |
| `render_bullet_list(head, items, theme)`            | `report.bullets(head, items)`                   |                                           |
| `render_progress_bar(prefix, cur, total, w, theme)` | `Progress::new(total).set(cur)`                 |                                           |
| `Spinner::start(msg, theme)` / `stop()`             | `Spinner::new(msg)` + caller ticking            | the theme is no longer dropped; no thread |
| `Theme` / `trait Paint` / `StyleExt`                | `Style` + `ColorChoice`                         | one colour API                            |
| `Glyphs` + `ascii()` / `nerd()`                     | `Glyphs::{ascii, unicode, nerd}()` + `detect()` | actually selected                         |
| `ProgressBar`                                       | `Progress`                                      | renamed, wired, not dead                  |
| `src/mod.rs`                                        | _deleted_                                       | orphan                                    |

## Verification & Tests

The overhaul is verifiable precisely because Phase 1 makes the toolkit renderable
into a buffer.

- **New:** `crates/tui/tests/report.rs` — render the same logical report in
  `pretty` and `plain`, at widths `40` / `80` / `120`, in `ascii` and `unicode`,
  for empty and populated inputs, via `Report::buffer(w)`. Snapshots are committed
  as plain text under `crates/tui/tests/snapshots/` and diffed like the goldens,
  consistent with "the test is the record".
- **New:** `crates/tui/examples/report.rs` — a demo so a human can eyeball every
  block without running the CLI.
- **Byte-check:** `cargo run --bin pendon -- --format solid < doc.md > before.jsx`
  before and after the migration must be identical (CLI-REPORT §1).
- **Manual matrix before merging Phase 1:** pseudo-TTY (`script`) vs pipe;
  `NO_COLOR=1`; `TERM=dumb`; `--color=always` into a pipe; `COLUMNS=40`; a
  non-UTF-8 locale (`LC_ALL=C`); `--quiet` in both.
- **Gates:** `cargo fmt --all -- --check`, `cargo test -p pendon-tui`,
  `cargo test -p pendon`, then `cargo test --workspace`, plus one `sandbox/*`
  re-render inspected by eye.

## Open Questions

Every recommendation below is **Agreed**: the maintainer accepted all
recommendations in the CLI report RFC, and that policy was confirmed for this
draft, so these are decisions rather than open threads. They are recorded here as
the decision record, following the precedent of
[`plugin-bind`](../archive/fulfilled/plugin-bind.md) §2.5–§2.8.

1. **Replace the free `render_*` API, or keep it alongside the new one?**
   _Recommendation_: replace it. `apps/cli` is the only consumer, so the migration
   is one commit with no external breakage; keeping both would immediately
   recreate the "two colour APIs" defect this RFC exists to remove.
   **Agreed.**

2. **Adopt an ecosystem crate (`indicatif`, `console`, `anstyle`, `comfy-table`) or
   stay on `crossterm`?**
   _Recommendation_: stay dependency-light — `crossterm` (present), `unicode-width`
   (present, unused), `crossterm::terminal::size()` (no new dep). `indicatif` and
   `console` own stdout/stderr policies we must control ourselves, and their
   animated output is not byte-stable enough for this repo's fixture discipline.
   **Agreed.**

3. **Where does the normative contract live — `docs/spec/CLI-REPORT.md` or a new
   `docs/spec/TUI.md`?**
   _Recommendation_: `CLI-REPORT.md` stays the single normative contract for the
   _observable report_; this RFC is the design record for the _toolkit_, and
   `crates/tui/README.md` documents the Rust API (linking to the spec rather than
   restating it). No `TUI.md` until a second consumer of the crate exists.
   **Agreed.**

4. **Does `--tui` stay opt-in?**
   _Recommendation_: yes. `--tui` opts into animation; a non-TTY run gets the plain
   report automatically (`cli-report-upgrade.md` Must-Have 1). Animation is never
   the default. **Agreed.**

5. **Animation inside CI pseudo-terminals (`script`, some CI runners)?**
   _Recommendation_: detect `CI` and force plain unless `--color=always` (or
   `--tui`) is explicit; a CI log full of `\r` frames is noise. **Agreed.**

6. **What does `Progress` do when the total is unknown (streaming stdin)?**
   _Recommendation_: degrade to the indeterminate spinner and never invent a
   percentage — a `--tui` run over stdin shows `Reading input…` and nothing more.
   **Agreed.**

7. **Keep the name `pendon-tui`?**
   _Recommendation_: keep it. The crate is internal, the name is already load-bearing
   in `Cargo.toml`, docs and the CLI's dependency list, and no real interactive mode
   exists yet; revisit only if `--watch` grows into one. **Agreed.**

## Adoption Strategy

- **Phase 0 — mechanical.** Bump `crossterm` to the current release; delete
  `src/mod.rs`, `once_cell`, and the dead features' declarations. No behaviour
  change, trivially reviewable, unblocks the rewrite.
- **Phase 1 — foundation (Must-Have 1–8).** One PR: add `style.rs`, `glyphs.rs`,
  `width.rs`, `report.rs`, `line.rs`, `widgets/progress.rs`; delete `theme.rs`,
  `templates.rs`, `assets.rs`, `widgets/{spinner,progress_bar}.rs` as their
  replacements land; migrate `apps/cli` (`main.rs`, `run.rs`) to `Report`; ship the
  `--color` flag; add `tests/report.rs` + `examples/report.rs`; rewrite
  `crates/tui/README.md`. This is the phase the user feels — aligned blocks, real
  progress, honest colour, and a CI summary where there is currently silence.
- **Phase 2 — depth (Should-Have 1–6).** Diagnostics code frames (with
  [`advanced-diagnostic.md`](advanced-diagnostic.md)), task sections + totals,
  themes, hyperlinks, `--watch`, shared formatting.
- **Phase 3 — polish (Nice-to-Have).** Spinner presets, ETA, sparklines, the JSON
  report, tee, locale.
- Every phase ends with the same gate list as §Verification & Tests, and updates
  `crates/tui/README.md`, `docs/guides/cli.md` (for new flags),
  [`docs/spec/CLI-REPORT.md`](../spec/CLI-REPORT.md) (for the observable shape) and
  `STATUS.md` — one fact, one file.

## Backwards Compatibility

- **stdout and exit codes are untouched.** The toolkit writes only to stderr today
  and keeps doing so (CLI-REPORT §1); a stdout path would be a separate decision.
- **The report's shape may change.** CLI-REPORT §4 documents the behaviour as
  implemented, and [`cli-report-upgrade.md`](cli-report-upgrade.md) already proposes
  the spacing and alignment normalisation. Phase 1's only change visible to a TTY is
  that normalisation (notably `render_severity_line`'s double space).
- **New flags are additive** (`--color`, later `--spinner`, `--report`,
  `--report-file`); `--quiet` and `--tui` keep their current meaning, with `--quiet`
  reimplemented as the null sink rather than as scattered guards.
- **CI consumers see exactly one new thing:** the plain summary on a non-TTY run,
  which [`cli-report-upgrade.md`](cli-report-upgrade.md) §Backwards Compatibility
  already calls out — and which is the point of that RFC.
- **Rust API.** `pendon-tui` is unpublished (`version = "0.0.1"`) with a single
  in-workspace consumer, so the break is internal and needs no shims.
  `CONTRIBUTORS.md`'s CLI-runtime rule (direct CLI and config runner behave
  identically for equivalent options) is _reinforced_: both paths route through one
  `Report`, instead of both re-deciding gating independently.
- **Determinism.** The golden set (`syntax_spec`) must stay
  byte-identical, and the new `crates/tui` snapshots become the second executable
  record. Docs follow the tests, not the other way round.
