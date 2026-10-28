# Golden Fixtures — the executable source of truth

`docs/spec/golden/` is Pendon's strongest documentation: **input + config +
expected output**, verified byte-for-byte by tests. When prose and a golden
disagree, **the golden wins** — fix the doc or the code, never the test to match a
guess.

## Format

Each fixture is a stem `NN-name` with up to four files:

```text
docs/spec/golden/NN-name.mdp          input markdown (the source, hand-authored)
docs/spec/golden/NN-name.toml         pendon.toml for the fixture (plugins + custom sets)
docs/spec/golden/NN-name.jsx          expected Solid output (GENERATED — do not hand-edit)
docs/spec/golden/NN-name.events.json  expected event IR (optional, for parser bugs)
docs/spec/golden/NN-name.data/        payload files a fixture reads from a path (optional; copied to `data/`)
docs/spec/golden/NN-name.out/         side files the render writes beside the page (optional; mirrors `out/`)
```

`NN-name.out/` exists for fixtures whose task writes files — an
`[[task.<element>.extract]]` block (§21 of `SYNTAX.md`). Every file in it is
compared byte-for-byte with the file the render wrote under `out/`, so the side
file is as frozen as the page.

The `.mdp` files are written to **double as prose**: they carry short comments
explaining the rule the fixture exercises (e.g. `01-extras-head.mdp` explains that
`class` accumulates while every other duplicate is an `Error` keeping the first
value). Read them as living examples.

## How they are tested

`apps/cli/tests/syntax_spec.rs` — one `#[test]` per fixture. The harness
(`write_project`) copies the fixture into a temp project, runs the real CLI
(`pendon run -F`), and compares the output byte-for-byte. This means the goldens
cover config loading and the full CLI path, not just the parsers.

Every render is also checked for **well-formedness before comparison**
(`assert_no_dotted_attribute_name`): a `.jsx` may not spell a dotted field as an
attribute name (`specs.tick="x"`), because `a.b=` is a member expression in JSX.
The one-key spread `{...{ "specs.tick": "x" }}` is the only spelling (§6.3), so a
fixture cannot freeze invalid output and pass.

```bash
cargo test -p pendon --test syntax_spec      # 30 fixtures, none #[ignore]d
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
| 03 | `image`            | `!` tag + preset `alt`/`src`, bare slot fields, retired marker run |
| 04 | `anchor`           | `@` tag vs untagged, six positions → preset, removed modifiers/external defaults |
| 05 | `cite`             | `[^^](ref "loc")` + extras, retired forms stay literal        |
| 06 | `heading`          | `[slug]` + `("title")` + extras, an authored `number:` passes through (numbering is `plugin-number`) |
| 07 | `wiki`             | `[[…]]` + extras, `href` not overridable                      |
| 08 | `table-decl`       | `\|-…-\|`, `\|\| caption \|\|`, decl head `[slug]`            |
| 09 | `table-layers`     | column/th/td/tr/tbody/tfoot extras routing                    |
| 10 | `decorator-blocks` | paragraph + code fence decorators, typed / bare / `@@[…]`     |
| 11 | `blockquote`       | decorator above only; an inner `@@` stays literal             |
| 12 | `list-container`   | L1 decorator → `ul`/`ol` + `start`                            |
| 13 | `list-item`        | L2 item-decorator → `li` (open — blocked by L37)              |
| 14 | `marker`           | `{{type}}` forms + positional groups                          |
| 15 | `directive-inline` | `::type…::`                                                    |
| 16 | `directive-block`  | `==type…==`                                                    |
| 21 | `extras-forms`     | bare head, type-only head, empty head, `@@type {…}` literal   |
| 22 | `section`          | §9.5 outline, section decorator + id chain                    |
| 23 | `bind`             | §19 data blocks → real props: JSON/JSONC/YAML/TOML/CSV, nested `$var`, unbound → literal |
| 24 | `bind-paths`       | §2.5–§2.7 paths, spread/merge, external file via `../data/`  |
| 25 | `extras-nested`    | nested objects/arrays + JSX brackets (`{valueA}` → `name={valueA}`), bind + non-bind   |
| 26 | `cite-scope`       | `references_scope`: frontmatter defaults to `all`, uncited entry still exported (§2.6) |
| 27 | `bind-mdp`         | `mdp` is a sub-document: the payload runs the page's chain (heading, wiki, cite, math, figure, table) and reaches the prop as a fragment |
| 28 | `type-selectors`   | `type` as selectors — `*` / literal / `_` / `?` against one document (§11, ADR-0008 `D-3a`) |
| 30 | `number-heading`   | `[[task.heading.number]]` — one counter for `HeadingA`+`HeadingB`, styles per depth, undefined depth defaults to decimal, unselected types stay bare (ADR-0010, P.1) |
| 31 | `number-custom`    | `%%numberType` markers: marker depth (`%`−1), marker beats element numbering, `[[task.number.custom]]` blocks, numbers on headings/anchors/images (ADR-0010, P.2) |
| 32 | `extract-heading`  | `[[task.heading.extract]]` — a file **and** `export const outline`; the automatic nested `headings` survives the deletion of `plugin-extract-heading` (ADR-0011, T4.5/T4.7) |
| 33 | `extract-scopes`   | P.1.a/b/c — two `type`-scoped blocks (two paths, two formats), the `_` remainder, and one `export` name where the later block wins (D-6) |
| 34 | `specs-assignment` | `field.specs.*` as a **move** — one entry assigning all five positions next to an entry that assigns nothing (§6.1, §6.3) |

Fixtures 17–20 are covered by unit tests in the renderer/CLI crates rather than as
golden files.

## Three findings to keep in mind

- A `Link` node produced by the core lexer inside an extras head is rebuilt by
  `plugin-table` before the table is parsed, so a long `("title")` in the
  declaration line still works (fixture 08).
- The cite head's location is the quoted string after the unquoted reference
  (`[^^](book "hlm. 45")`); it fills the same `loc` slot as an extras `loc:` prop,
  with the head winning (fixture 05).
- `extract` reads the stream, so it must run **last**: with `extract` before
  `markdown` the source lines are not nodes yet and the rows are half-parsed
  (headings swallow the rest of the document). The validator only rejects an
  element plugin or `number` placed after it (fixture 32).
