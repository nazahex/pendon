# Golden Fixtures — the executable source of truth

`docs/spec/golden/` is Pendon's strongest documentation: **input + config +
expected output**, verified byte-for-byte by tests. When prose and a golden
disagree, **the golden wins** — fix the doc or the code, never the test to match a
guess.

## Format

Each fixture is a stem `NN-name` with up to four files:

```text
docs/spec/golden/NN-name.md           input markdown (the source, hand-authored)
docs/spec/golden/NN-name.toml         pendon.toml for the fixture (plugins + custom sets)
docs/spec/golden/NN-name.jsx          expected Solid output (GENERATED — do not hand-edit)
docs/spec/golden/NN-name.events.json  expected event IR (optional, for parser bugs)
docs/spec/golden/NN-name.data/        payload files a fixture reads from a path (optional; copied to `data/`)
```

The `.md` files are written to **double as prose**: they carry short comments
explaining the rule the fixture exercises (e.g. `01-extras-head.md` explains that
`class` accumulates and other keys are last-wins). Read them as living examples.

## How they are tested

`apps/cli/tests/syntax_spec.rs` — one `#[test]` per fixture. The harness
(`write_project`) copies the fixture into a temp project, runs the real CLI
(`pendon run -F`), and compares the output byte-for-byte. This means the goldens
cover config loading and the full CLI path, not just the parsers.

```bash
cargo test -p pendon --test syntax_spec      # 19 fixtures, none #[ignore]d
```

The `sandbox/ultimate` frozen baseline is gated separately:

```bash
cargo test -p pendon --test ultimate_freeze_spec
```

## Re-freezing (when output intentionally changes)

Never hand-edit a `.jsx`. Re-render it, review the diff, then re-freeze:

```bash
# in a temp copy of the fixture, or the fixture dir itself:
cargo run --bin pendon -- run -F
# review the diff of NN-name.jsx — only intentional changes should appear
```

A fixture whose syntax is not implemented yet may be annotated
`#[ignore = "pending Phase N"]`; `cargo test` stays green while the expectation is
frozen. A phase is not complete while its fixtures are still ignored.

## Fixture list

| #  | Fixture            | Covers                                                        |
| -- | ------------------ | ------------------------------------------------------------- |
| 01 | `extras-head`      | all item kinds, ordering, duplicates, escaping                |
| 02 | `extras-literal`   | malformed heads → literal text, adjacency failures            |
| 03 | `img-figure`       | `~?!` / `~?!!` + extras + caption                             |
| 04 | `anchor`           | head `("title")` vs extras `"title"` priority                 |
| 05 | `cite`             | `[^^](ref "loc")` + extras, retired forms stay literal        |
| 06 | `heading`          | `[slug]` + `("title")` + extras, auto-number interaction      |
| 07 | `wiki`             | `[[…]]` + extras, `href` not overridable                      |
| 08 | `table-decl`       | `\|-…-\|`, `\|\| caption \|\|`, decl head `[slug]`            |
| 09 | `table-layers`     | column/th/td/tr/tbody/tfoot extras routing                    |
| 10 | `decorator-blocks` | paragraph + code fence decorators, typed / bare / `@@[…]`     |
| 11 | `blockquote`       | inner extras + decorator above                                |
| 12 | `list-container`   | L1 decorator → `ul`/`ol` + `start`                            |
| 13 | `list-item`        | L2 item-decorator → `li` (open — blocked by L37)              |
| 14 | `marker`           | `{{type}}` forms + positional groups                          |
| 15 | `directive-inline` | `::type…::`                                                    |
| 16 | `directive-block`  | `==type…==`                                                    |
| 21 | `extras-forms`     | bare head, type-only head, empty head, `@@type {…}` literal   |
| 22 | `section`          | §9.5 outline, section decorator + id chain                    |
| 23 | `bind`             | §19 data blocks → real props: JSON/JSONC/YAML/TOML/CSV, nested `$var`, unbound → literal |
| 24 | `bind-paths`       | §2.5–§2.7 paths, spread/merge, external file via `../data/`  |

Fixtures 17–20 are covered by unit tests in the renderer/CLI crates rather than as
golden files.

## Two findings to keep in mind

- A `Link` node produced by the core lexer inside an extras head is rebuilt by
  `plugin-table` before the table is parsed, so a long `("title")` in the
  declaration line still works (fixture 08).
- The cite head's location is the quoted string after the unquoted reference
  (`[^^](book "hlm. 45")`); it fills the same `loc` slot as an extras `loc:` prop,
  with the head winning (fixture 05).
