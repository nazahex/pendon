# CLI Report Contract

`pendon` writes two things to the terminal: the **rendered document** and the
**report**. This file is the normative contract for the report — what it may
write, where, in what shape, and with which styling tokens. It is deliberately
separate from [`guides/cli.md`](../guides/cli.md), which lists the _flags_, and
from [`rfc/cli-report-upgrade.md`](../rfc/cli-report-upgrade.md), which proposes
changes to the shapes below.

> Status: this describes the behaviour as implemented. `crates/tui`
> (`pendon-tui`) is the only writer of _decorated_ report output. Where this file
> and the code disagree, the code wins — update this file (see
> [`../INDEX.md`](../INDEX.md)).

## 1. Streams

| Stream        | Carries                                                       | Piped-safe                      |
| ------------- | ------------------------------------------------------------- | ------------------------------- |
| **stdout**    | the rendered document only (JSON / events / AST / HTML / JSX) | yes — never decorated           |
| **stderr**    | the report, diagnostics, warnings, errors                     | yes — decorated only when a TTY |
| **exit code** | `0` success · `2` error/failure                               | —                               |

Rules:

- **The report never touches stdout.** Redirecting stdout must always yield the
  exact rendered bytes, byte for byte.
- **Diagnostics are not "the report".** They are messages _about_ the document
  and are governed by [`rfc/advanced-diagnostic.md`](../rfc/advanced-diagnostic.md).
  They may share the styling tokens below, but they are not suppressed by
  `--quiet`.
- **All decorated output is written by `pendon-tui`.** Plain
  `eprintln!("Error: …")` / `eprintln!("Warning: …")` messages are not part of
  the report; no other code may emit decorative output.

## 2. What counts as the report

Two producers, both in `apps/cli`:

1. **The `run` report** — the summary `pendon run` prints per task and once at
   the end. Sections: a status line, a per-task "Wrote" block, a final severity
   line.
2. **The `--tui` spinner** — an indeterminate spinner on stderr while input is
   read (`pendon --tui`).

Sink gating (`apps/cli/src/run.rs`, `main.rs`):

- Every `run` report line is gated on `pendon_tui::is_interactive_stderr()`.
  When stderr is **not** a terminal (a pipe, a CI log) the report is not emitted
  at all — see the known gaps in §6.
- The `--tui` spinner is gated on both `--tui` and `is_interactive_stderr()`.
- `-q` / `--quiet` suppresses the `run` report and the `--tui` spinner
  regardless of terminal-ness.

## 3. Styling tokens

The report is styled from a single `Theme`:

| Token     | Default   | Meaning                     |
| --------- | --------- | --------------------------- |
| `primary` | blue      | status / info accents       |
| `success` | green     | completed                   |
| `warning` | yellow    | warnings                    |
| `error`   | red       | errors                      |
| `muted`   | dark grey | secondary text, block heads |

Glyphs come from a `Glyphs` set (today `ascii()`; see §6):

| Glyph    | Char | Used by                    |
| -------- | ---- | -------------------------- |
| `tick`   | `✔`  | `Done`                     |
| `cross`  | `✖`  | `Error`                    |
| `info`   | `◆`  | `Info`                     |
| `warn`   | `▲`  | `Warn`                     |
| `bullet` | `-`  | key/value and bullet lists |

Severity lines pair a glyph with a bracketed **label**, so meaning is never
carried by colour alone:

| Severity | Glyph | Label     | Colour  |
| -------- | ----- | --------- | ------- |
| `Done`   | `✔`   | `⟦done⟧`  | success |
| `Info`   | `◆`   | `⟦info⟧`  | primary |
| `Warn`   | `▲`   | `⟦warn⟧`  | warning |
| `Error`  | `✖`   | `⟦error⟧` | error   |

## 4. Layout grammar

The report is built from four block shapes.

**Status line** (`render_status_line`) — one line in `primary`, terminated by a
carriage return (`\r`) so a later line overwrites it in place:

```
Scanning source files...
```

**Severity line** (`render_severity_line`) — `glyph label title`, one line,
terminated by `\n`:

```
✔ ⟦done⟧ All tasks completed
```

**Key/value list** (`render_kv_list`) — a `muted` prefix line, then two-space
indented `bullet key: value` lines:

```
› Wrote:
  - name: docs
  - format: html
  - size: 1.2 kB
  - total: 3
  - cache: 3 processed
```

**Bullet list** (`render_bullet_list`) — like a key/value list, but each item is
free text:

```
› Notes:
  - some note
```

Numbers and units:

- Sizes are humanised to `B` / `kB` / `MB` (`1024`-based, one decimal place).
- Counts are bare integers — no thousands separators, no trailing period.

## 5. Gating & quiet

| Condition           | Report | Spinner      |
| ------------------- | ------ | ------------ |
| stderr is a TTY     | yes    | `--tui` only |
| stderr is not a TTY | no     | no           |
| `-q` / `--quiet`    | no     | no           |

`--quiet` never changes stdout or the exit code, and never suppresses
diagnostics, warnings, or errors.

## 6. Known gaps (not yet normative)

Recorded so the contract is honest about today's limits. The behavioural upgrade
path is [`rfc/cli-report-upgrade.md`](../rfc/cli-report-upgrade.md); the toolkit
that has to implement these gaps (the dead code, the duplicate colour traits, the
inert glyph features) is [`rfc/tui-overhaul.md`](../rfc/tui-overhaul.md); most
items below are `pendon-tui` defects, the rest are the CLI's gating around them.

- `ascii()` returns Unicode glyphs and `nerd()` is an alias — the `ascii` /
  `nerd-fonts` Cargo features are declared but never select a glyph set.
- When stderr is not a TTY the report is dropped entirely, so CI logs lose the
  summary and see only the exit code.
- Two colour helpers coexist: the public `theme::Paint` trait and a private
  `StyleExt` in `templates.rs`.
- `render_severity_line` emits two spaces between glyph and label.
- `ProgressBar`, `render_progress_bar`, and `render_bullet_list` are exported but
  unused by the CLI.
- No `NO_COLOR` handling: colour is decided only by TTY-ness.

## 7. Non-goals

- Not a logging framework — no timestamps, log levels, or log files.
- Not the diagnostics format — see
  [`rfc/advanced-diagnostic.md`](../rfc/advanced-diagnostic.md).
- Not the rendered document format — see §6.3 of [`SYNTAX.md`](SYNTAX.md) and the
  `crates/renderer-*/README.md` files.
