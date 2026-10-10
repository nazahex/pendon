# pendon-tui

A tiny terminal-UI toolkit used by the CLI's optional `--tui` spinner: a spinner
and a progress bar, with themable glyphs. It writes only to **stderr** and is
safe for pipelines (it never touches stdout).

## What it provides

- **`Spinner`** — an indeterminate spinner for stderr while input is processed.
  `Spinner::start(message: String, theme: Theme)` returns a guard; `stop(self)`
  ends it.
- **`ProgressBar`** — a determinate progress bar. `ProgressBar::new(total, prefix, theme)`
  with builder-style `set_width(..)` and `set(current)` / `inc(delta)` / `finish(self)`.
- **`Glyphs` / `Theme`** — the character set (`ascii()` and `nerd()` presets).
- **`is_interactive_stderr() -> bool`** — whether stderr is a terminal; the CLI
  uses this to decide whether a spinner is safe to draw.

## Usage

```rust
use pendon_tui::{is_interactive_stderr, Spinner, Theme};

if is_interactive_stderr() {
    let spinner = Spinner::start("Reading input…".to_string(), Theme::default());
    // … work …
    spinner.stop();
}
```

The CLI wires this behind the `--tui` flag:

```bash
pendon --tui --input ./doc.md
```

Without a terminal on stderr (a pipe), `is_interactive_stderr()` is `false` and
nothing is drawn, so output stays clean.

## License

MIT
