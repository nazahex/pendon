# `task.include` — modular `pendon.toml` configuration

**Status: implemented.** `task.include` lets a `[[task]]` pull in additional
TOML files, so a single `pendon.toml` does not have to grow to hundreds of
lines in medium-to-large projects.

## Problem

A task's configuration (`input`, `output`, `plugin`, `format`, plus per-element
`[task.<element>.custom]` blocks, `field.*` assignments, `post_hook`, etc.) can
easily run to dozens of lines. Across many tasks, `pendon.toml` becomes hard to
navigate and hard to share between projects.

## Objective

Enable users to split their `pendon.toml` into multiple files and include them
from a `[[task]]`, keeping the configuration modular while preserving Pendon's
strict config contract (§13: unknown keys are load errors, not warnings).

## Recommended design (implemented)

### Syntax

`include` is a `[[task]]`-level key holding an array of strings:

```toml
[[task]]
name = "Include Demo"
input = "./src/[slug].md"
output = "./out/[slug].jsx"
plugin = "all"
format = "solid"

include = [
  "../../foo.toml",
  "../../../baz/bar.toml",
  "./wax.toml",
  "./soo/sas.toml",
]
```

Each path is relative to `pendon.toml`'s directory (the project root / CWD).

### Merge semantics

An included file is parsed as raw TOML `toml::Value`. Its `task` content is
deep-merged into the including task **before** deserialisation into
`PendonConfig`; the `include` key is then dropped so it never surfaces as an
unknown key (§13).

Two shapes are supported in an included file:

1. **Table form** — `[[task.anchor.custom]]`, `[[task.heading.extract]]`, etc.
   These produce `task` as a table (`{ anchor = { custom = [...] } }`), whose
   fields are deep-merged into the including task. Useful for contributing
   per-element custom component config without repeating the task header.

2. **Array form** — `[[task]]` in the included file produces `task` as an array
   (`[{ ... }]`). Each element's fields are deep-merged into the including task
   in order. Useful for contributing scalar task fields
   (`markdown_strip_comments`, `pretty`, etc.).

Merge rules (deep merge):

- **Tables** merge recursively; source keys override target keys.
- **Arrays** concatenate (e.g. repeated `[[task.anchor.custom]]` entries accumulate).
- **Scalars** — source wins.

### Resolution order

1. `pendon.toml` is parsed as `toml::Value`.
2. For each `[[task]]` with `include`:
   - `include` is removed from the task.
   - Each included file is read, parsed, and its own `include` directives are
     resolved recursively first (nested includes).
   - The included file's `task` content is deep-merged into the including task.
3. The merged `Value` is deserialised into `PendonConfig`.
4. §13 unknown-key check runs on all tasks (including merged content).

### Errors

- **Missing included file** → load error, exit 2, one `Error:` line.
- **Invalid TOML in an included file** → load error, exit 2.
- **Circular include** (`a.toml` → `b.toml` → `a.toml`) → load error, exit 2.
- **Maximum include depth exceeded** (16) → load error, exit 2.
- **`include` not an array of strings** → load error, exit 2.

## Non-goals

- **Including at the top level** (`include` outside `[[task]]`) is not supported.
  `include` is strictly a `[[task]]` key.
- **Included tasks are not appended as new tasks.** Included content is merged
  into the task that declares `include`. To add a new task, write a new
  `[[task]]` entry in `pendon.toml` or in an included file's array form.
- **No `plugin-custom` merging.** Included files contribute `task` content only;
  `[plugin-custom]` belongs in `pendon.toml` itself.
- **No compatibility shims.** `include` is new syntax; there is no alias,
  no `#[serde(alias)]`, no migration path.

## Files changed

- `apps/cli/src/run.rs` — parse `pendon.toml` as `toml::Value`, resolve
  `task.include` before deserialisation, deep-merge helpers, unit tests.
- `apps/cli/tests/cli_spec.rs` — integration tests (array form, table form,
  missing file, circular include).
- `docs/issues/config-upgrade.md` — this document.

## Open questions (post-implementation)

- Should `include` also be allowed at the `pendon.toml` top level (config-wide
  include, not per-task)?
- Should `plugin-custom` from included files be merged too?
- Should included files be able to declare `[[task]]` entries that become new
  tasks (rather than merging into the including task)?
