# STATUS — Live Plan of Record

**This is the single live source of truth for "what is happening now."**
It is intentionally small and changes every commit. History and rationale live
elsewhere — do **not** restate them here:

- **Normative grammar** → [`docs/spec/SYNTAX.md`](docs/spec/SYNTAX.md)
- **Why a decision was made (D1–D10)** → [`docs/decisions/`](docs/decisions/)
- **Completed work / historical essays** → [`docs/archive/`](docs/archive/)
- **How to run tests, goldens, sandboxes** → [`CONTRIBUTORS.md`](CONTRIBUTORS.md) & [`docs/guides/testing.md`](docs/guides/testing.md)

Update this file whenever you start, block, or finish a piece of work. When a
line is done, delete it (the git history is the record) — keep this file short.

---

## Current focus

**Stage 2 (Unified Syntax) is landed.** All 40 tasks in
[`docs/workflows/unified-syntax-stage-2/TASKS.md`](docs/workflows/unified-syntax-stage-2/TASKS.md)
are checked and the gate is green (`cargo test --workspace` = 79 targets,
`cargo clippy --workspace --all-targets`, `cargo fmt --all -- --check`,
`dprint check`). The grammar the rework settled on is
[`docs/spec/SYNTAX.md`](docs/spec/SYNTAX.md) — §5 heads/slots/modifiers, §6 fields
and presets, §14 what is gone; the reasons are ADR-0012…ADR-0018, and D-16 is
executable now (`apps/cli/tests/clean_break_spec.rs`: a retired config key stops
the build with one `Error:` naming its replacement).

What is left: the **L37 blocker** below, the maintainer's sandbox replay (see
"Deferred"), and the two follow-up issues `S2-10.5` files.

## Open blockers

- **L37 truncation (`sandbox/ultimate`).** A heading whose line carries an extras
  head inside a list item over-scans past the line and re-emits the swallowed
  text inside the `<h*>`; the frozen baseline currently encodes a truncated body.
  Minimal repro:
  ```text
  ##[s-a] V1

  - item

    # [x]("T"){.d}body

  D01 after
  ```
  Owner: shared head reader (`crates/extra`) + the plugin that scans it
  (`plugin-anchor` / `plugin-heading`). After the fix: re-render, review the newly
  visible tail sections, re-freeze, add a regression case to the E section.
  **Blocks:** Step 6b (list-container sandbox cases L53–L55), the D/B/N/E sections
  of `sandbox/ultimate`, and golden `13-list-item`.

## In progress

- [ ] **Editor grammars (`editors/vscode/`).** The Stage-2 grammar (slots, tags,
      modifier runs, `@@` decorators) and the harness (`tools/self-test.mjs`,
      `tools/scope-cases.mjs`) are green; breadcrumbs / Outline
      (`src/headings.js`) and directive highlighting (`src/scan.js`) exist. Left:
      package and publish the extension; the frontmatter gap, line-bounded display
      math, blockquote inline fallback and the other approximations are listed in
      [`editors/vscode/README.md`](editors/vscode/README.md).
- [ ] **RFCs: diagnostics + VS Code plugin (drafts).**
      `docs/rfc/advanced-diagnostic.md` and `docs/archive/fulfilled/plugin-vsc.md` authored; no
      code yet. Note: today diagnostics are silent (the CLI uses them only for the
      exit code, `--strict` covers only core-parser limit guards, and
      `Event::Diagnostic.span` is never populated).
- [ ] **CLI report: contract spec + upgrade RFC (docs).**
      `docs/spec/CLI-REPORT.md` formalizes the stderr report; `-q` / `--quiet`
      landed (it silences the `run` report and the `--tui` spinner only). The rest
      of `docs/rfc/cli-report-upgrade.md` (plain report for CI, `NO_COLOR`,
      verbosity ladder, `--report=json`) is unstarted.
- [ ] **TUI overhaul RFC (docs only — no code).** `docs/rfc/tui-overhaul.md`
      proposes the ground-up rewrite of `crates/tui` (one `Report<W>` sink, one
      colour decision, width-aware blocks, a plain mode, a buffer test seam); all
      open-question recommendations are accepted and recorded there.
- [ ] **Step 3 — L2/L3 item layer.** `plugin-list` item wrapper; `plugin-markdown`
      gains `CustomPlacement::ListItem` + `PendingListItem` / `arm_list_item` /
      `take_list_item`; golden `13-list-item`. (blocked by L37)
- [ ] **Step 6b — ultimate sandbox (list container).** L53/L54/L55 container
      decorator + bracket group. (blocked by L37)

## Deferred

- **Sandbox replay — maintainer-owned, not an agent task.** `S2-10.4` re-rendered
  every sandbox that has a top-level `pendon.toml` (16 render; `sandbox/number/`
  cannot — a pre-implementation sketch whose own config no longer parses and
  which has no source) and reviewed all 42 moved files: **not one is an output
  regression.** Every diff traces to a stale **source** or a pre-Stage-2
  baseline — a `!?~` image marker run, a `[^^]` citation, an `@@`-less decorator
  line, a `|…|` table declaration line, an `[.class, foo: "x"]` extras list on a
  link — all of which now render as literal text, plus the retired field names
  (`ref_id`/`loc`, a flat `number`, `level: "2"`). The replay was **reverted**, not
  committed (AGENTS.md rule 7: goldens are the contract, sandboxes are scratch),
  so `sandbox/{image,unified,universal}` no longer track `HEAD` either; nothing
  does until the maintainer re-authors those sources. One gotcha for that replay:
  an `[[task.<element>.extract]]` side file **accumulates** (D-20), so replaying
  over an `out/` that still holds old rows appends rows in the new schema after
  them — delete `sandbox/extract/out/` first. `sandbox/ultimate` additionally
  differs in the `export const headings` line (two of three are L37's swallowed
  body text, the third a heading in a blockquote that takes its enclosing
  section's id — §6.2/§9.5). No golden depends on any sandbox.

- **§11 threading gaps** (from the positional-keys pass):
  - `plugin-table`: `extras_to_layer(head)` still uses `Specs::default()`;
    thread `TableOptions` into the 7-layer head scan.
  - `plugin-wiki`: `WikiOptions` carries no `ComponentSet`, so extras stay on the
    built-in `slug`/`title`; add `[task.wiki.custom]` to lift this.
- **Phase-4 cleanup** (Step 7): archive `docs/rfc/unified-syntax.md`; rewrite
  `sandbox/unified` (part of the replay above); trim `docs/todo/syntaxes.md`.
  §15's OPEN-R1 and OPEN-C2 are resolved as _dropped_ — a removed spelling is
  literal text, never an alias with a warning (D-16).
- **plugin-markdown gaps** (Step 5): fenced code inside a blockquote (Q14);
  `parse_blockquote_prefix` indentation + blank-`>` loose-list split.
- **Decorator space exception:** a decorator on a list item / blockquote may allow
  one space between marker and head (specified §4.1); exercised once L2/L3 lands.

## Golden & freeze status (see [`docs/spec/golden/README.md`](docs/spec/golden/README.md))

- Golden fixtures `01–12`, `14–16`, `21–28`, `30–43` are frozen and green; none
  `#[ignore]`d. `cargo test -p pendon --test syntax_spec` (37 fixtures) is the gate.
- Phase 3 re-froze `06-heading` (it loses the numbers `plugin-heading` used to
  compute and keeps its authored `number:` props) and `28-type-selectors`
  (dropped `auto_number`, no output change).
- Phase 4 added one line to every golden with headings (`06`, `16`, `21`, `22`,
  `25`, `28`, `30`, `31`): `export const headings` is now automatic instead of
  depending on the deleted `plugin-extract-heading`. No other byte moved; `32`
  and `33` are new (the latter also freezes side files in `NN-name.out/`).
- Stage 2 re-froze `14-marker` twice (`S2-4.9` both type sources, `S2-3.6` the
  component-path dotted fields) and `34-specs-assignment` (`S2-3.6`, its three
  `moveA` fields). The `S2-3.6` diff is dotted attribute names becoming the
  `{...{ "path": … }}` spread, nothing else — the harness now rejects a dotted
  attribute name on every fixture, so that class of invalid JSX cannot be frozen
  again.
- `S2-4.4` moved every cite fixture: `05-cite` rewritten (`[^id "loc"]` heads,
  paren positions, the two `Error` cases), `26-cite-scope` (one citations row),
  `27-bind-mdp` and `33-extract-scopes` (its `.cite.json` / `.cite.yaml` side
  files too) — all by the field rename `ref_id`/`loc` → `reference_id`/`locator`
  and the loss of `instance_index`; `35-cite-fields` is new.
- `S2-4.7` renamed `03-image-figure` → `03-image` and rewrote it (one `<img>`
  node: bare head, bare slot fields, the retired marker run as literal text, an
  `alt:` extra erroring) and moved `27-bind-mdp` again (its payload image is a
  `[task.image.custom.img]` entry now). `31-number-custom` did **not** move: a
  `%%numberType` marker is an extras item, so those images were already the
  plugin's and the block shape is unchanged.
- `S2-4.10` moved `22-section` and `32-extract-heading` by the `depth` attribute
  (§9.5): every `<section>` now publishes the outline level it stands for, so a
  fixture that renders a section shows one extra attribute and nothing else.
- `S2-4.11` moved `09-table-layers` and `21-extras-forms` (a `<td>` no longer
  receives the column's `class`/`type` — §8/OPEN-E1) and added `36-table-slots`,
  the seven-layer proof that every table layer reads `[…]("…"){…}%%numberType`.
- `S2-5.1` moved **no** fixture: every golden already spelled its decorators with
  `@@`. `39-decorator-order` and `40-decorator-needs-at` are new and freeze the
  mandatory sigil from both sides.
- `S2-5.2` moved **no** fixture either (the three binders that already existed
  agreed with the table for every golden block) and added `41-decorator-claim` /
  `42-decorator-rejected`: the first proves each receiver takes its own line, the
  second that a `NotAReceiver` / `ReceiverAbsent` / `Overridden` line leaves no
  trace.
- `S2-5.3`/`S2-5.4` moved the three table fixtures that carried a decl line:
  `08-table-decl` → **`08-table-head`** (renamed and rewritten — the retired line
  is literal text, the decorator carries `class`/`id`/`title`/typed props, and
  the caption line now actually renders), `09-table-layers` (same output, the
  decl line's prose removed) and `36-table-slots` (the `<table>`'s `specs.*`
  slots are spreads now instead of the decl's hardcoded `id`/`title`).
  `41-decorator-claim` and `42-decorator-rejected` each gained one table case.
- `S2-6` moved **no** existing fixture: a modifier run contributes nothing until a
  config defines a symbol, and no earlier fixture defines one. `43-slot-modifiers`
  is new — array merge in written order (equal values kept), `class`
  accumulation without a repeated token, head-wins, an unconfigured symbol, a bare
  flag, `loading` → `data-loading`, and a **config-only** entry per element — and
  its `.out/` freezes the extract pair (`preset = "pendon"` leaves the modifier
  fields out, the un-preset block keeps `rel` / `soo.baz` as an array and a dotted
  path).
  The one behavioural addition behind it: `ComponentSet` carries config-only
  entries (`ConfigEntry`, D-8) and `field_for` / `modifier_for` are transparent
  across levels, so `field.*` and `bracket.modifier` reach an element that has no
  custom component (and an entry that configures nothing no longer silences the
  layer default).
