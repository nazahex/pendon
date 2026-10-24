# ADR-0004 — `plugin-bind` runs a pre-parse text stage

- Status: **Accepted / implemented**
- Proposal: [`docs/archive/fulfilled/plugin-bind.md`](../archive/fulfilled/plugin-bind.md) (archived)
- Extended by: [`0007-mdp-subdocuments.md`](0007-mdp-subdocuments.md) — an `mdp`
  payload now runs the **whole** chain, so `resolve` takes a renderer
- Evidence: `crates/plugin-bind` unit tests; goldens `23-bind` (body payloads),
  `24-bind-paths` (paths, spread/merge, a file payload resolved through `../data/`)
  and `27-bind-mdp` (a payload as a sub-document)

## Context

`plugin-bind` declares data in `{{{lang[var] … }}}` blocks and binds it into
extras values with `$var`. A payload is a serialization (JSON/JSONC/YAML/TOML/CSV)
or Pendon Markdown — **not** Pendon prose. Its lines routinely start with
characters the core lexer treats as markup:

| payload line       | core lexer sees |
| ------------------ | --------------- |
| `# comment` (TOML) | Heading         |
| `- x`              | list item       |
| `> q`              | blockquote      |
| `\| a \| b \|`     | table           |
| blank line         | block separator |

Every existing plugin is **post-parse** (or a pass over parsed events — called the
**emit** pass since ADR-0005, §12.2; it is not data binding).
The panics/`U+E000` sentinel mechanism (§12.1) lets a plugin _mark_ already
parsed text, but it cannot stop the lexer from reading the payload first.
Confirmed with the real CLI: `# Head\n\n{\n "a": 1\n}\n\n# comment` renders as
one `Heading`.

## Decision

`plugin-bind` adds a **new, first-class pre-parse stage** (ADR term: the _bind
extract_ stage). When `bind` is enabled the CLI runs, in order:

1. `pendon_plugin_bind::extract(source) -> (clean_source, Registry, diagnostics)`
   **before** `pendon_core::parse`. It removes every data block and returns the
   parsed values; a removed block is replaced by exactly one blank line so the
   surrounding blocks stay separate. A block may take its payload from a file
   (`{{{csv[rows](../data/rows.csv)}}}`): `extract_with_base(source, dir)` resolves
   it against the **source file's** directory (never the CWD, so `../` works at
   any depth) and lists every file read in `Extract.external_files`; the caller
   must add those to its dependency set. Without a base (stdin) such a block is an
   `Error`. RFC §2.7.
2. `pendon_core::parse` plus the normal plugin loop (`bind` is a no-op there).
3. `pendon_plugin_bind::resolve(events, &registry) -> Vec<Event>` **after** every
   other plugin, so the extras heads have already emitted their attributes.

The `Registry` is per-document state threaded through the CLI, never global
(principle 4 in `CONTRIBUTORS.md`).

## Rejected alternatives

- **Post-parse text scan of `Event::Text`.** The lexer has already mangled the
  payload (evidence above).
- **A `U+E000` sentinel per block (protect, like §12.1).** Protects _head_ text
  that is already inside a paragraph, not a multi-line, markup-looking payload;
  the lexer would still split the payload into Heading/List/Paragraph nodes
  before the sentinel could round-trip it.
- **Requiring payloads to be escaped/prose-safe.** Shifts the burden to the
  author and breaks every real JSON/YAML/TOML file.
- **A separate CLI subcommand/preprocessor binary.** Duplicates the pipeline and
  violates "direct CLI and config execution behave identically"
  (`CONTRIBUTORS.md` principle 5).

## Consequences

- The pipeline gains one stage that owns raw text; `AGENTS.md` / `CONTRIBUTORS.md`
  must describe it so it is not confused with post-parse binders.
- `bind` must be enabled for both stages; enabling it only post-parse emits the
  "no registry" warning (§6 of the RFC).
- The stage is deterministic and side-effect free, so the golden fixtures and the
  `ultimate` freeze gate stay byte-stable when `bind` is off (no output drift).
