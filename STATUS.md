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

Four proposals, one sequenced plan ([`docs/workflows/`](docs/workflows/), 40 tasks
in [`TASKS.md`](docs/workflows/TASKS.md)). **Phases 0–4 are landed**; the gates and
the L37 blocker are what is left.

- **Phase 0/1 — shared types.** `pendon-types` (`Selector`, `TypeSet`, `Rule`,
  `Group`, `winner`, `matching`) answers `type = …` selectors; `D-3a`: the
  renderer resolves each layer once per document and plugins stop choosing
  (ADR-0008). Every `[task.*.custom]` golden stayed byte-identical.
- **Phase 2 — exports.** `SolidRenderHints.exports: Vec<DataExport>`, written by
  `renderer-solid` alone; `frontmatter`/`headings` are automatic, `plugin-cite`
  pushes `references` then `citations`, `[[task.bind.inject]]` publishes a file
  (SYNTAX §20, ADR-0009).
- **Phase 3 — numbering.** `plugin-number` owns all numbering (ADR-0010):
  `[[task.<element>.number]]` + `[[task.number.custom]]`, the `%%numberType`
  marker parsed in `crates/extra` (§5, §10.5) and carried as `__number`,
  `attrs.number` written as an array. `plugin-heading` numbers nothing (its
  `auto_number`/`number_style` are a §14 migration error); `plugin-image` no
  longer `data-`prefixes `__` keys. Goldens `30-number-heading`,
  `31-number-custom` frozen, `06-heading` re-frozen (numbers gone). Found while
  re-rendering: `renderer-html` leaked `__layer` on a `Link` (it writes
  `href`/`title` itself, bypassing the `__` skip) — fixed + tested.

**Phase 4 — `plugin-extract` (landed).** `[[task.<element>.extract]]` collects the
ten extractable elements' fields into flat rows (`type` selectors, `recommended` /
`only` / `omit` with omit always winning), writes them to side files (`json` /
`yaml` / `toml`, `[slug]`-dynamic paths, one path = one array) and registers
`export const` on Solid tasks (SYNTAX §21, ADR-0011). `extract` runs as a chain
stage that only _reads_ the stream, so it must come **last**. Only a `Page`
writes document-level outputs — a `mdp` payload skips `extract`. Side files are
`CacheEntry.side_outputs` (`CACHE_VERSION = 2`), so deleting one makes the next
run rewrite it, through the shared `write_atomically`.
`plugin-extract-heading` is **deleted**; the automatic nested `headings` export is
now built by `renderer-solid` from the AST (byte-identical for `sandbox/heading`,
but since it no longer depends on a plugin it appears on every Solid document
with headings — eight goldens re-frozen with that one line). Goldens
`32-extract-heading` and `33-extract-scopes` (`NN-name.out/` freezes side files
too); `sandbox/extract` runs and writes all four of its outputs.

**Next:** the L37 blocker below (it also unblocks Step 3 and Step 6b). The
Phase-4 gates are closed:
`cargo test --workspace`, `cargo clippy --workspace --all-targets` and
`cargo fmt --all -- --check` are green, the eight re-frozen goldens moved by
exactly one `export const headings` line each (no bytes removed), and
`sandbox/{unified,universal,multi}` re-render byte-identically.

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

- [ ] **Stage 2 — Phases 0–3 done; Phase 4 in progress (`S2-4.1` done).** Docs live in
      [`docs/workflows/unified-syntax-stage-2/`](docs/workflows/unified-syntax-stage-2/README.md).
      **Phase 0 complete 2026-10-28**: D-1…D-17 signed (D-16 = clean total
      replacement: no compat, no warnings, no migration doc, old configs fail
      fast), OBJ-1…OBJ-11 answered, ADRs
      [0012](docs/decisions/0012-universal-slots.md)–[0018](docs/decisions/0018-clean-replacement.md)
      written and registered. Baseline `244bbb9`: 645 tests green.
      **Phase 1**: `S2-1.1` (`crates/extra/src/slot.rs`: `SlotKind`/`SlotParts`/
      `parse_slot_parts` + 13 tests), `S2-1.2` (`parse_slots` + `SlotMatch`:
      bracket + attached paren, trailing text, `[[…]]` left to wiki), `S2-1.3`
      (`SlotTag` + `read_head`, head position only), `S2-1.5` (bracket modifier
      run, dedupe in written order, parens exempt) and `S2-1.6`
      (`SlotPos`/`SlotContract`) done — 34 module tests. `S2-1.4` (OBJ-1 probe:
      `crates/core` has no footnote extension, so cite keeps `^`) done. Phase 1
      is complete: no golden byte moved.
      **Phase 2**: `S2-2.1` (name rules: `validate_field_name` +
      `FieldNameError`) and `S2-2.2` (`Path` segment-wise prefixes, `FieldMap`
      ordered dotted map with nesting + structural-clash errors) done in
      `crates/extra/src/field.rs` — 13 module tests, workspace 693 pass, still
      no golden byte moved. `S2-2.3` done: `FieldConfig` (typed
      `slot`/`specs`/`overrides` maps, `Slot::from_name` inverses) parsed from
      `field.*` dotted keys in `apps/cli/src/components.rs`, every malformed
      shape a load error. `S2-2.4` done: `custom` is array-only (a single table
      is a load error), fields-only entries work, an empty entry is an error, and
      `ComponentSet::field_config` encodes the D-8 merge order; six sandbox and
      two golden configs moved to `[[…]]` with no output change. `S2-2.5` done:
      `crates/extra/src/preset.rs` (`FieldName` — a `const fn`-validated
      single-segment built-in name, so a preset typo fails to compile — `Preset`,
      `ElementHead { contract, preset }`, `preset_for` / `is_consistent` /
      `fallback_path`), and the four slot elements declare `pub const HEAD` in
      their own crate, claiming all six positions and presetting §7.1–§7.4
      exactly (anchor `inner_text`/`href`/`title`, cite
      `reference_id`/`locator`, heading `slug`/`title`, image `alt`/`src`).
      Data only — the private `insert("href", …)` calls stay until each element's
      Phase 4 step. `S2-2.6` done: `ElementHead::assign(pair, config)` lands every
      content the contract accepts — preset name while the preset is alive, the
      entry's `field.<slot>.<kind>` name once it is void, and the slot-qualified
      bare name (`bracket.quote`, `paren.tick`) for anything unassigned, so D-7's
      example is asserted row by row. `S2-2.7` done: `project(node,
      Option<&FieldConfig>)` is the identity pass — every attribute becomes one
      `Path → AttrValue` entry in node order, `__`-prefixed internals never
      enter the map, a non-field-name key is dropped loudly, and
      `field.override.<path>` renames a field in place (the only place overrides
      are read). Both DOM renderers now emit `FieldMap::attrs()` instead of the
      raw node (Solid `render_attrs` + `{...attrs}`/`{attrs.k}`, HTML
      `render_attrs_except` + `Link`), `AttrValue` gained `Null` and a total
      `from_json` so a bound value survives the round trip, and the JSON/AST
      renderers needed no call site (their `attrs` _is_ the projection). Zero
      golden bytes moved — that is the proof. `S2-2.8` done: the four naming
      keys are gone (a leftover `backtick_key`/`quote_key`/`bracket_key`/
      `parentheses_key` is a load error, one test each) and their job is now
      `field.*` — `Specs::resolve` reads `field.bracket.bare` /
      `field.paren.bare` / `field.specs.tick` / `field.specs.quote` off the
      matched entry (which now carries its `FieldConfig` instead of a bespoke
      naming struct). Fixture 15 and `sandbox/{directive,ultimate}` moved to
      `field.*` with **byte-identical** output. Fail-fast is complete: an unknown
      `custom` entry key and an unknown `[[task]]` key are both load errors
      (§13 = Error), and an unknown entry key test pins the closed vocabulary.
      **Phase 2 complete.**
      `S2-3.1` done: "canonical extras" is now **specs** in code —
      `PositionalNames` → `Specs`, `NamesResolver` → `SpecsResolver`,
      `ComponentSet::names_for(_opt)` → `specs_for(_opt)`, and every `keys`
      binding that held one is a `specs` binding. Renames only: workspace green,
      zero output moved. The four `default`/`meaning` rows §6.1 carries now also
      live on `Specs`'s doc, because an assigned field has no semantics of its
      own — otherwise `S2-3.5`'s rewrite would delete the last copy.
      `S2-3.2` + `S2-3.4` done: the head emits `specs.class` / `specs.id` /
      `specs.style` / `specs.tick` / `specs.quote` (through `Specs`, so an
      assignment renames them) and `project` maps the three **universal** positions
      back to `class` / `id` / `style`, sorting entries by the final public path so
      nothing moves; `renderer-solid` emits a dotted path as a one-key spread
      (`{...{ "specs.tick": "x" }}`) because JSX has no `a.b=`, `renderer-html`
      prints it verbatim, JSON/AST keep it. Fixtures 10 and 14 re-frozen, one test
      per specs position in each renderer. **D-12's rule table is in** (§6.4,
      §13): a repeated `id` / `slug` / `title` / prop / flag is now an `Error`
      that keeps the first value (the `DuplicateId` warning is gone), `class`
      accumulates and dedupes keeping the first position, a repeated `style:`
      property stays CSS last-wins with no diagnostic, and
      `ExtrasIssue::owned_key` carries the built-in-field rule as an `Error`
      naming `field.override.<k>` — `plugin-anchor`'s `href` uses it, the other
      elements get theirs with their contracts (`S2-4.1`). Fixture `01` re-frozen
      (`foo="one"`, `id="dup"`, plus a new positional-duplicate line). `S2-3.3` done:
      an assignment is a **move**, not a rename — `SpecsKey` names the five
      positions, `Specs::path` where one is published, and `Specs::element_path`
      answers the default path only while it is unassigned, so every element's own
      read (`anchor`'s tick→id, a citation's `cite-id`, a heading's `title`) goes
      `None` once `field.specs.<k>` moved it and `dom_fields` (which matches on the
      **path**) leaves `foo.class` / `bar.id` / `foo.style` verbatim — no element
      meaning, no universal rename, default path empty. Verified end-to-end in a
      three-format sandbox (solid / html / json); tests in `crates/extra`,
      `plugin-anchor` and the CLI config path
      (`anchor_options_carry_the_entry_specs`).
      `S2-3.5` done — **Phase 3 is complete**: §6.1 is re-derived from `Specs`'
      doc into seven rows (content / config key / **default path** / meaning, so a
      default is visibly not an assignment), §5.1 and §14 now name the paths, §6.4
      states the map order and the projection's sort, and the missing §6.5 ("the
      element preset", D-7) that five crates already cited now exists. New golden
      `34-specs-assignment`: one entry assigns all five positions and an unclaimed
      type in the same document keeps every default. Nine plugin READMEs lose the
      copy-pasted (and wrong) "keeps `slug` / `title` / `class` / `id` / `style`"
      sentence in favour of a link to §6.1. Next: Phase 4, `S2-4.1`.
      `S2-4.1` done — **Phase 4 opens**: `apps/cli/tests/extras_heads_spec.rs` is
      the universality audit (17 rows: every element head + decorator carries the
      same five-position head and a `%` marker; two universal checks per row). No
      element failed. One anomaly recorded for Phase 5: a table **declaration's**
      `("T")` loses `title` to an extras quote (§6.2 inversion in the layer
      props). D-12 rule 5 is data now — `ElementHead.owned` (`with_owned` /
      `owns`) + `owns_field` — with `plugin-anchor` (`href`) and `plugin-wiki`
      (`href`, `title`) wired; the other elements declare theirs with their
      contracts. Next: `S2-4.2` (anchor).
      `S2-4.2` done — the anchor reads its head through the shared grammar
      (`parse_slots` + `parse_extras`), so **the `@` tag is mandatory**: an
      untagged `[x](y)` is nobody's slot and falls through to `plugin-markdown`,
      and the extras head after it stays literal. The preset drives the link —
      `HEAD.assign` gives `inner_text` (the text), `href`, `title`, while
      `bracket.quote` / `bracket.tick` / `paren.tick` are published under their
      slot-qualified names, so nothing is dropped. Deleted: external-URL
      `target`/`rel`, the six URL-suffix modifiers, the `^`/`~` conflict warning
      and the `rel`/`target` special-casing (§14, D-16). Fix found on the way:
      D-7's voiding trigger is a **slot** key, not any `field.*` key — otherwise
      renaming a specs item voids `href` (dated note on D-7). Fixture `04-anchor`
      rewritten (tag/untagged, all six positions, the removals), and the eight
      other fixtures that use the element gained their `@`: no byte moved except
      `04` and `23` (the markdown link inside an `mdp` payload loses its
      auto-`target`/`rel`). Gate: fmt + dprint clean, clippy at baseline, 759
      tests. Next: `S2-4.3` (blockquote).
      `S2-4.3` done — a quote has **no head**: the inner `> @@type{…}` form is
      gone (`emit_quote` copies the body with one `>` level stripped and reads
      nothing else), so the decorator line above the quote is the only way to
      configure one, and `> @@type{…} text` is literal text. The element declares
      no `ElementHead` at all — no contract, no preset, no owned fields — stated in
      §9.2. Fixture `11-blockquote` rewritten and re-frozen; the S2-4.1 audit's
      `quote-inner` row dropped (its `quote-decorator` row still proves the head is
      read); §14 gained the removal, §4.1's adjacency exception now names the list
      item only. Gate: fmt + dprint clean, clippy at baseline, 759 tests. Next:
      `S2-4.4` (cite).
      `S2-4.5` done — verified "no change": `::type` / `==type` still read their
      own head, `type` is never extras-derived (`plugin-extract`'s `directive`
      table keeps it), and the audit rows cover both heads. Added the missing
      regression test (`a_decorator_never_overrides_the_directive_type`: a
      `@@other{…}` line cannot change a block or inline directive's `type`) and
      one §10.3 sentence. 760 tests. Next: `S2-4.4` (cite).
      `S2-4.9` done — the marker's grammar was already right, the override was
      not: `{{type}}` now beats an extras `@@type` **silently** (the
      `ExtrasIssue::warning` is gone), `type` stays its only built-in field (no
      `inner_text` — the text is the children) and it contracts nothing. Test
      renamed to `an_extras_type_is_overridden_silently`; fixture `14-marker`
      gained both type sources and was re-frozen; §10.1's obsolete
      "layer order decides which element reads `[x]("y")`" note now points at
      `@[x]("y")`. 760 tests. Next: `S2-4.10` (section) / `S2-4.4` (cite).
      `S2-4.8` done — the list needed no code change: `plugin-list` declares no
      contract and no preset, so its preset assignment was already empty; §9.3 now
      says so and its pre-S2-3.5 `slug`/`title` bullet became §6.1's default
      paths. New test `a_head_inside_an_item_is_content`. The inventory half of
      the exit is `S2-7.2`'s (it rewrites every table anyway). 761 tests. Next:
      `S2-4.10` (section) / `S2-4.4` (cite).
      `S2-3.6` done — a gap `S2-3.4` left on the **component** path: the
      `{...attrs}` spread in `template.rs` still wrote a dotted key verbatim, so
      `14-marker.jsx` had been freezing `<Bibliography bracket.bare="refs" …
      specs.tick="…">`, which is not JSX at all (`a.b=` is a member expression).
      `node::{is_jsx_attr_name, render_dotted_attr}` are now `pub(crate)` and the
      template path calls them, so one helper spells a public path for both the
      built-in elements and a component. Goldens `14-marker` and
      `34-specs-assignment` re-frozen; every moved byte is a dotted name becoming
      a one-key spread, positions untouched. Guarded twice: a renderer unit test
      (`a_component_receives_a_dotted_field_as_a_one_key_spread`) and a harness
      check on **every** fixture render (`assert_no_dotted_attribute_name`,
      self-tested) so invalid JSX cannot be frozen again. Rejected nested props
      `specs={{ … }}` recorded as **D-18**; §6.3 + the solid README say "one rule,
      two paths". 763 tests. Next: `S2-4.4` (cite).
- [ ] **Editor grammars (`editors/vscode/`).** Block + inline rules landed
      (`syntaxes/pendon.tmLanguage.json`, plus the injection that adds them to
      `.md`), and the harness (`tools/self-test.mjs`) is green on every golden
      and sandbox fixture. Coloring pass: fixed the heading `captures`
      off-by-one (title/extras/slug now tokenize; was lumped with `innerText`),
      added CSV bind-body field scopes, and added inline `$…$` + display
      `$$…$$` math (mirrors `plugin-latex`). Directive visual aids landed
      VS Code-side (`src/extension.js` + `src/scan.js`, guarded by
      `tools/directive-scan-test.mjs`): caret-tracking open/close match highlight
      (always on) and an opt-in `pendon.directive.depthColors` depth rainbow —
      the `::`/`==` syntax is unchanged. **Heading breadcrumbs/Outline landed**
      (`src/headings.js` + a `DocumentSymbolProvider` in `src/extension.js`,
      guarded by `tools/heading-scan-test.mjs`): `.mdp` headings now appear in
      the breadcrumbs with the Pendon head stripped and inline markdown reduced
      to plain text; symbol shape (section-scoped ranges, `#`-prefixed names)
      mirrors the built-in Markdown provider, so the breadcrumb chain follows
      the cursor's depth. `#` inside HTML comments, fenced code (info-string
      openers included), closed `---` frontmatter and `plugin-bind` data blocks
      is skipped — each region verified inert with `pendon run --format ast`.
      `package.json` gained
      `onLanguage:pendon` so the extension activates for `.mdp`. Left:
      package/publish the extension; the frontmatter gap, line-bounded display
      math, blockquote inline fallback and the other approximations are listed in
      [`editors/vscode/README.md`](editors/vscode/README.md).
- [ ] **RFCs: diagnostics + VS Code plugin (drafts).**
      `docs/rfc/advanced-diagnostic.md` and `docs/archive/fulfilled/plugin-vsc.md` authored; no
      code yet. Note: today diagnostics are silent (the CLI uses them only for the
      exit code, `--strict` covers only core-parser limit guards, and
      `Event::Diagnostic.span` is never populated).
- [ ] **CLI report: contract spec + upgrade RFC + `--quiet` (docs + one flag).**
      `docs/spec/CLI-REPORT.md` formalizes the stderr report (streams, `Theme`
      tokens, block layout, gating); `docs/rfc/cli-report-upgrade.md` proposes the
      upgrade path (plain non-interactive report so CI stops losing the summary,
      `NO_COLOR`, verbosity ladder, `--report=json`; the `pendon-tui` hygiene item
      now lives in `rfc/tui-overhaul.md`). `-q` /
      `--quiet` landed — it silences the `run` report and the `--tui` spinner only
      (stdout, exit codes, and diagnostics are untouched).
- [ ] **TUI overhaul RFC (docs only — no code).** `docs/rfc/tui-overhaul.md`
      proposes a ground-up rewrite of `crates/tui`: one `Report<W>` sink replacing
      the eight free `render_*` functions, one colour decision
      (`--color`/`NO_COLOR`), glyph sets that actually switch, width-aware blocks,
      a synchronous `Progress`/`Spinner` with no threads, a plain mode for non-TTY
      runs, and a buffer test seam. All open-question recommendations are accepted
      and recorded in the RFC. Supersedes the "`pendon-tui` hygiene pass" item in
      `cli-report-upgrade.md`.
- [ ] **Step 3 — L2/L3 item layer.** `plugin-list` item wrapper; `plugin-markdown`
      gains `CustomPlacement::ListItem` + `PendingListItem` / `arm_list_item` /
      `take_list_item`; L4 `start:` rejection warning; L5 nesting test; golden
      `13-list-item`. (blocked by L37)
- [ ] **Step 6b — ultimate sandbox (list container).** L53/L54/L55 container
      decorator + bracket group, unclaimed container type, pair lost to the anchor
      layer. (blocked by L37)
- [x] **Step 7 — full gates.** `cargo test --workspace`, `cargo clippy
      --workspace --all-targets`, `cargo fmt --all -- --check` green; golden diff
      reviewed (one added `headings` line per fixture, nothing removed);
      `sandbox/{unified,universal,multi}` re-render identically. `sandbox/ultimate`
      still moves (`headings` ids only) — it is L37's baseline, not a Phase-4
      regression.

## Deferred

- **Stale sandbox baselines — maintainer-owned, not an agent task.** Take
  `sandbox/ultimate` and every other stale sandbox as untrustworthy input: do not
  re-render them, do not freeze them into goldens, and do not treat a diff there
  as a regression in your change. The maintainer replays them by hand. Concretely,
  `sandbox/anchor`, `sandbox/custom/inline`, `sandbox/img/*`, `sandbox/list/*` and
  `sandbox/table/integ` still hold older output (`"cites"` where the exports are
  now `references` + `citations`, `class={"q"}` where a typed string renders
  `class="q"`, `cra-1` where the cite id is `cite-1`), `sandbox/ultimate` differs
  from a fresh render in the `export const headings` line (two of three are L37's
  swallowed body text, the third a heading in a blockquote that takes its
  enclosing section's id — §6.2/§9.5), and `sandbox/number/` is a
  pre-implementation sketch with no source. `sandbox/{image,unified,universal,multi}`
  are the ones that still track `HEAD`. A golden is the contract; a sandbox is a
  scratch copy — the gate never depends on a sandbox staying in sync.

- **§11 threading gaps** (from the positional-keys pass):
  - `plugin-table`: `extras_to_layer(head)` still uses `Specs::default()`;
    thread `TableOptions` into the 7-layer head scan.
  - `plugin-wiki`: `WikiOptions` carries no `ComponentSet`, so extras stay on the
    built-in `slug`/`title`; add `[task.wiki.custom]` to lift this.
- **Phase-4 cleanup** (Step 7): OPEN-R1 `:::` deprecated alias with warning; archive
  `docs/rfc/unified-syntax.md`; rewrite `sandbox/unified`; trim
  `docs/todo/syntaxes.md`.
- **plugin-markdown gaps** (Step 5): fenced code inside a blockquote (Q14);
  `parse_blockquote_prefix` indentation + blank-`>` loose-list split.
- **Decorator space exception:** a decorator on a list item / blockquote may allow
  one space between marker and head (specified §4.1); exercised once L2/L3 lands.

## Golden & freeze status (see [`docs/spec/golden/README.md`](docs/spec/golden/README.md))

- Golden fixtures `01–12`, `14–16`, `21–28`, `30–34` are frozen and green; none
  `#[ignore]`d. `cargo test -p pendon --test syntax_spec` (28 fixtures) is the gate.
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
