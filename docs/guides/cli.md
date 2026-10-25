# CLI Guide

The CLI binary is `pendon`. During development always use
`cargo run --bin pendon` so you run the current code, not an old build.

## Input

Reads from stdin by default, or from a file with `--input`.

```bash
echo "Hello" | cargo run --bin pendon --
cargo run --bin pendon -- run                 # full run with pendon.toml config
cargo run --bin pendon -- run -F              # bypass cache and re-render
cargo run --bin pendon -- --input ./README.md
```

## Formats (`--format`)

| Format   | Output                                                     |
| -------- | ---------------------------------------------------------- |
| `json`   | concatenated text IR for quick preview (default)           |
| `events` | raw event stream (Start/End/Text/Diagnostic) for debugging |
| `ast`    | hierarchical JSON AST with nodes and aggregated text       |
| `html`   | HTML                                                       |
| `solid`  | JSX/TSX (the component contract)                           |

`--pretty` selects the indented mode of a format that has one (`json`, `events`,
`ast`, `html`). Same as `pretty = true` on a task.

## Flags

- `--input <path>` — read from a file instead of stdin.
- `--format <name>` — `json`, `events`, `ast`, `html`, `solid`.
- `--pretty` — indented output where supported.
- `--strict` — escalate diagnostics to errors; the CLI still prints output and
  exits non-zero if any error is present.
- `--tui` — minimal spinner on stderr (safe for pipelines; does not touch stdout).
- `-q`, `--quiet` — suppress the report (the `run` summary and the `--tui`
  spinner). stdout, exit codes, and diagnostics are unaffected. See
  [`../spec/CLI-REPORT.md`](../spec/CLI-REPORT.md) §5.
- `--max-doc-bytes <n>` — warn/error when input exceeds `n` bytes.
- `--max-line-len <n>` — warn/error when a line exceeds `n` characters.
- `--max-blank-run <n>` — warn/error when consecutive blank lines exceed `n`.
- `--plugin <name>` — apply a plugin transform before rendering (e.g. `markdown`,
  `markdown,syntect`). Multi-plugin lists are comma-separated.

## `run` flags

`pendon run` reads `./pendon.toml` from the current directory. Its flags are
separate from the pipeline flags above:

| Flag            | Effect                                                                            |
| --------------- | --------------------------------------------------------------------------------- |
| `-F`, `--force` | ignore the cache, re-render everything, and refresh every record                  |
| `--no-cache`    | ignore the cache for this run **and** record nothing, so the next run starts cold |
| `--clean-cache` | delete `.pendon/cache` first, then rebuild                                        |
| `-q`, `--quiet` | silence the run report (see [`../spec/CLI-REPORT.md`](../spec/CLI-REPORT.md) §5)  |

`--no-cache` is a whole-run bypass; `-F` is its sibling that leaves a warm cache
behind. To keep one task out of the cache permanently, use `no_cache` in
`pendon.toml` below.

## Cache

`pendon run` records one small TOML file per rendered `(task, input)` pair:

```text
.pendon/cache/<first 2 hex>/<sha256(task + input)>.toml
```

A record holds the task, its output, the task-config hash, the input and output
hashes, and the hash of every dependency. A file is skipped only while all of
them still match **and** its output is still on disk at the path the task would
write — deleting `out/`, or editing an `output` template, rebuilds it. Because
the key contains the task, two tasks may render one input and stay cached
independently. Records are written atomically (temp file + `rename`), so an
interrupted run loses at most the record it was writing.

The run walks `./src`-style trees from the project root, and that walk prunes
`.pendon/`: a cache record can never be matched by an `input` glob, counted, or
rendered as a document.

- A warm run writes nothing; only files that actually re-rendered get a new record.
- A record from another schema version is dropped and rebuilt — upgrading Pendon
  costs one cold run — and a pre-v1 `.pendon/cache.toml` is removed automatically.
- `--clean-cache` deletes `.pendon/cache`. Nothing prunes records for inputs or
  tasks that no longer exist; they are never read again and cost only disk.

See [`../decisions/0006-per-file-cache-records.md`](../decisions/0006-per-file-cache-records.md)
for why the cache is shaped this way.

## Task config (`pendon.toml`)

Each `[[task]]` names `input` (a glob with `[slug]`-style placeholders),
`output`, `plugin`, and `format`, plus optional keys such as `name`, `pretty`
and `[task.<plugin>]` layers (see each `crates/plugin-*/README.md`). Unknown
keys are reported as warnings and otherwise ignored (see `docs/spec/SYNTAX.md`
§13); they are never a silent no-op.

### `no_cache`

`no_cache = true` removes the task from the cache entirely: its records are
neither read nor written, so every file it matches renders on every run. The
rest of the pipeline is unaffected — other tasks stay cached, including ones
that render the same `input`.

```toml
[[task]]
name = "Cite Sandbox (Solid)"
input = "./src/[slug].mdp"
output = "./out/[slug].jsx"
plugin = "micromatter,cite,markdown"
format = "solid"
no_cache = true
```

Use it for cheap renders, or ones whose output feeds a side effect (formatting,
previews), where a stale file is worse than the work of rendering again.

### `post_hook`

`post_hook` runs shell command(s) after each file of the task renders
successfully — formatting, asset optimisation, notifying another tool. A string
is one command; an array runs in declared order. They run through the system
shell (`sh -c` on Unix, `cmd /C` on Windows) with the project root as the
working directory, so relative paths resolve there.

```toml
[[task]]
name = "Site (HTML)"
input = "./src/[slug].mdp"
output = "./out/[slug].html"
plugin = "markdown"
format = "html"
# Sequential commands; the second sees the file the first rewrote.
post_hook = [
  "prettier --write ./out/[slug].html",
  "html-minifier-terser -o ./out/[slug].min.html ./out/[slug].html",
]
```

Placeholders are expanded with the same rules as `input` / `output`, using the
captures of this file's `input` match — so `[slug]` in a hook names the file that
was just rendered. An unknown placeholder is an error, not literal text.

Behaviour:

- Hooks run only for files this run actually rendered. A file served from the
  cache does not re-run them; records are keyed by `(task, input)`, so one input
  may stay cached for several tasks at once.
- `post_hook` is part of the task's config hash, so editing it invalidates the
  task once and the new command reaches already-cached files.
- Hook stdout and stderr are forwarded to stderr as `[post_hook:<task>] <line>`
  (see [`../spec/CLI-REPORT.md`](../spec/CLI-REPORT.md) §2). `--quiet` does not
  suppress them: they are the tool's own output, not the run report.
- A non-zero exit prints an error, aborts the remaining commands for that file,
  and marks the task failed (exit code `2`, as for a render error). Its cache
  record is not refreshed, so the next run renders and hooks it again.
- A task that fails to render skips its hooks.

### `[[task.bind.inject]]`

Publishes a data **file** as a module-level export on a `format = "solid"` task,
without naming it in the Markdown (requires `bind` in `plugin`):

```toml
[[task]]
name = "API docs"
input = "./src/[foo]/[bar].mdp"
output = "./out/[bar].jsx"
plugin = "bind,markdown"
format = "solid"

[[task.bind.inject]]
source = "./src/[foo]/[bar].yaml" # `[foo]`/`[bar]` come from this file's match
export = "fooData"

[[task.bind.inject]]
source = "./data/settings.toml" # no `export`: the stem names the binding
```

`source` resolves against the project root (never the document's directory) and
accepts the task's captures; `export` must be a JavaScript identifier and not a
reserved word, and a bad one is a task error naming the key and the value. The
payload goes through the same converter as a `{{{lang[var] … }}}` block, the file
joins the task's cache dependencies, and a duplicate name keeps the **last**
registration. Format, ordering and the "Solid only" rule are
[`SYNTAX.md` §20](../spec/SYNTAX.md); the config keys are detailed in
[`crates/plugin-bind/README.md`](../../crates/plugin-bind/README.md).

## Examples

```bash
# Strict mode with blank-line guard
cargo run --bin pendon -- --strict --max-blank-run 1 < input.md

# AST with the markdown plugin
cargo run --bin pendon -- --plugin markdown --format ast --input ./doc.md

# HTML with syntax highlighting
cargo run --bin pendon -- --plugin markdown,syntect --format html --pretty --input ./doc.md

# Config-driven run inside a sandbox
cd sandbox/custom && cargo run --bin pendon -- run -F
```

## Plugins

- `markdown` — normalizes a subset of Markdown blocks (headings, code fences,
  thematic breaks).
- `syntect` — syntax highlighting for fenced code (Syntect default grammars;
  TS/TSX falls back to the JavaScript grammar).

Plugin coverage is evolving; non-recognized structures remain as text inside
`Paragraph` nodes. The full plugin set and order are documented in
`CONTRIBUTORS.md` and each `crates/plugin-*/README.md`.

## Report output

Progress and status lines (`pendon run`, `--tui`) are written to **stderr**,
never stdout, and are suppressed by `--quiet`. The format, styling tokens, and
gating rules are normative in [`../spec/CLI-REPORT.md`](../spec/CLI-REPORT.md);
proposed changes are in [`../rfc/cli-report-upgrade.md`](../rfc/cli-report-upgrade.md).

`[post_hook:<task>]` lines are forwarded tool output, not the report: they go to
stderr and stay visible under `--quiet` (see [`../spec/CLI-REPORT.md`](../spec/CLI-REPORT.md) §2).
