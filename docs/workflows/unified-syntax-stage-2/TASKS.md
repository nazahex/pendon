# Stage 2 — task list

One task = one commit-sized change with a testable exit. Work the **lowest-id
unblocked** task; ids are stable, so quote them in commits and `STATUS.md`
(`S2-4.2: cite slots + preset`).

Legend: `Dep` = must be done first. **Exit** is the only definition of done;
see `README.md` §5 for the shared gates. Fixtures: `G:` = golden/fixture to
add or re-freeze, `U:` = unit test. Decisions are in
[`DECISIONS.md`](DECISIONS.md).

## Phase 0 — sign-off and scaffolding

Entry: `STATUS.md` says Stage 2 is planned. Leave when every decision is
`settled` and the ADRs exist. No production code in this phase.

**Phase 0 is complete (2026-10-28)** — every task below is `[x]`.

- [x] **S2-0.1 — Settle D-1 … D-16.** Dep: none.
      Files: [`DECISIONS.md`](DECISIONS.md).
      Do: walk the sign-off table with the owner, answer OBJ-1 … OBJ-11 where they
      block a decision, flip statuses to `settled` (**D-16 is already settled** by
      the owner's clean-replacement policy).
      Exit: no `proposed`, no blocking OBJ left.
      **Done 2026-10-28** — D-1 … D-17 all signed; OBJ-1 … OBJ-11 answered
      (D-8 confirmed as "one `custom` table").
- [x] **S2-0.2 — Write ADR skeletons 0012 … 0018.** Dep: S2-0.1.
      Files: `docs/decisions/0012-universal-slots.md` … `0018-clean-replacement.md`,
      `docs/decisions/README.md`.
      Do: context / decision / rejected / consequences, status `Proposed`;
      register in the ADR index (see the map in `DECISIONS.md`).
      Exit: seven files, seven index rows, links resolve.
      **Done 2026-10-28** — written as `Accepted` (the owner signed), registered
      in `docs/decisions/README.md`.
- [x] **S2-0.3 — Route the workflow.** Dep: none.
      Files: `docs/INDEX.md`, `AGENTS.md`, `STATUS.md`.
      Do: `docs/INDEX.md` row for `docs/workflows/unified-syntax-stage-2/`;
      `AGENTS.md` "Add a new plugin / syntax" and "Touch markup grammar" rows
      point at it; `STATUS.md` gets one line.
      Exit: `grep -r unified-syntax-stage-2 docs/INDEX.md AGENTS.md` hits.
- [x] **S2-0.4 — Baseline snapshot.** Dep: none.
      Files: none (record in `STATUS.md`).
      Do: `cargo test --workspace` green at the starting commit; note which
      sandbox/golden dirs exist so later diffs are attributable.
      Exit: green main + the commit hash recorded.
      **Done 2026-10-28** — `244bbb9`: 645 passed, 0 failed. Goldens live in
      `docs/spec/golden/`; sandboxes in `sandbox/*/src/`.
- [x] **S2-0.5 — Author the workflow documents.** Dep: none.
      Files: `README.md`, `DECISIONS.md`, `TASKS.md` in this folder.
      Do: scope + phase ladder + gates (`README.md`), the decision set with
      cross-references (`DECISIONS.md`), this task list.
      Exit: no dangling links between the three files; every removal in the plan
      is owned by exactly one task with a fixture.
- [x] **S2-0.6 — Delete the migration surface.** Dep: S2-0.1 (D-16 settled).
      Files: this folder, `docs/INDEX.md`.
      Do: remove `MIGRATION.md` and every link to it — the clean-replacement
      policy forbids migration docs and compatibility notes (D-16). Confirm no
      other doc promises an "upgrade path". Done with the policy write-up.
      Exit: `grep -ri "migration" docs/workflows/ docs/INDEX.md AGENTS.md` hits
      only D-16's own policy wording — no table, guide or old→new mapping exists.

## Phase 1 — slot grammar (parse only, nothing wired)

Entry: D-1, D-9, D-11, D-12 settled (D-9 partially: canonical order only).
Leave when the parser is unit-tested and **no golden byte has moved**.
All work in `crates/extra/src/slot.rs` (+ `mod.rs` re-export).

- [x] **S2-1.1 — Slot content model and one-slot parser.** Dep: S2-0.1.
      U: `crates/extra/src/slot.rs` `#[cfg(test)]`.
      Do: `SlotKind {Bare,Quote,Tick}`, `SlotParts`, `parse_slot_parts(&str)`
      implementing order (bare → quote → tick), space separation, `"`/`'`
      quotes, backtick ticks, escapes, and the duplicate-kind error (D-2 tier 2).
      Exit: table-driven tests including `[bare"quote"`tick`]` and
      `[bare "quote"`tick`]` from §4, and the escape case.
      **Done 2026-10-28** — 13 unit tests in `slot.rs`; the §5.2 examples, the
      escape example, duplicate/out-of-order and unterminated cases are pinned.
- [x] **S2-1.2 — Both slots and adjacency.** Dep: S2-1.1.
      Do: `parse_slots` returns bracket + optional paren; paren must be attached
      (no space); `[]()` empty; `()` without `[]` is not a slot; anything left
      over is returned as trailing text.
      Exit: tests for `[]()`, `[x] (y)` (not a slot), `(x)` alone, trailing text.
      **Done 2026-10-28** — `parse_slots` returns `SlotMatch {Absent, Slots,
      Malformed}`: nothing is consumed unless the whole head parses, so an
      unterminated `[Foo` / `[Foo](bar` stays literal (D-2 tier 1). `[[…]]` is
      left to `plugin-wiki` (§7.5). Naming note: `Slot {Bracket, Paren}` is the
      _kind_ (it is what `SlotPos` pairs with a `SlotKind`) and the parsed pair is
      `SlotPair { tag, parts: [SlotParts; 2], modifiers }` — D-1's one-struct
      sketch could not name both.
- [x] **S2-1.3 — Slot tags.** Dep: S2-1.2.
      Do: `SlotTag {None,Anchor,Cite,Image,Heading}` from `@` (before `[`),
      `!` (before), `#` (before), `^` (right after `[`); tags only in head
      position; untagged standalone slot → `None` (caller decides, D-11).
      Exit: one test per tag (position asserted), one mid-text negative
      (`foo#[bar]` stays literal).
      **Done 2026-10-28** — `read_head` matches the tag in the first two
      characters; `^` sits inside the bracket and is consumed as the tag
      (`[^ref]` → bare `ref`), `!!` is **not** a slot (the figure marker is
      `plugin-image`'s, §7.1), and `foo#[bar]` / `x[^1]` / `a![b](c)` are literal.
      Whether the owner plugin is enabled is caller-side, so it is Phase 3's test.
- [x] **S2-1.4 — OBJ-1: footnote probe.** Dep: S2-1.3.
      Files: `crates/core` test only; `DECISIONS.md` note.
      Do: feed `[^ref]` / `[^ref]: x` through the core parser with no plugins;
      record whether it becomes a footnote. If yes, amend D-11's cite tag (and
      the ADR) before S2-4.2.
      Exit: a test pinning the actual behaviour + a written answer to OBJ-1.
      **Done 2026-10-28** — no footnote extension exists (`NodeKind` has no
      footnote variant); `crates/core/tests/parser_spec.rs`
      `footnote_syntax_is_not_claimed_by_the_core_parser` pins that `[^ref]`
      is neither a footnote nor a link and that the text survives. Cite keeps
      `^`; `DECISIONS.md` OBJ-1 records the answer.
- [x] **S2-1.5 — Bracket modifier run.** Dep: S2-1.2.
      Do: find the trailing `|…` run, dedupe keeping first order, allow any
      UTF-8 symbol incl. `|` and `\`, reject escapes that would reorder slots
      (D-9), ignore modifiers in parens.
      Exit: tests for `|?~uv$%`, `|?xx?|ee?` ≡ `|?x|e`, `||||` ≡ `||`, the
      `[Foo "Bar" \|?x]` error, and `[|?xu*](foo.png)`.
      **Done 2026-10-28** — the run opens at the first unescaped `|` that begins
      a word (start of the bracket, or after a space), so `a|b` and `\|` stay
      content; `collect_modifiers` dedupes in written order and drops a second
      bare `|` symbol. Parens scan with `run_allowed = false` (`bar |?x` is one
      bare content, no error). `[Foo "Bar" \|?x]` surfaces
      `SlotError::OutOfOrder { Quote, Bare }` — the ADR's reason, not a special
      case.
- [x] **S2-1.6 — `SlotContract` type.** Dep: S2-1.3.
      Do: `struct SlotContract { accepted: &'static [SlotPos], tag: SlotTag }`
      where `SlotPos` is `(Slot::Bracket|Paren, SlotKind)`. Data only; no plugin
      uses it yet. Must express "anchor contracts bracket.bare, paren.bare,
      paren.quote".
      Exit: unit test constructing the anchor/cite/image/heading contracts.
      **Done 2026-10-28** — `SlotPos` + `SlotContract` + `accepts()`; the test
      builds all four (`bracket.bare`, `paren.bare`, `paren.quote`, nothing else;
      no element reads a tick in §7.1–§7.4) and asserts each contract's tag.

## Phase 2 — field model (still byte-identical output)

Entry: D-3, D-4, D-5, D-6, D-7, D-8 settled; Phase 1 done.
Leave when a `FieldMap` is built for every element, renderers read it, and
`cargo test --workspace` passes **without re-freezing anything** (the projection
is an identity at this point).

- [x] **S2-2.1 — Field-name validation.** Dep: S2-0.1.
      Files: `crates/extra/src/field.rs` (new).
      U: name rules table.
      Do: allow `[A-Za-z0-9_-]`, first char not digit or `-`, case-sensitive,
      `.` = nesting; reject empty segments and trailing dots.
      Exit: tests for valid/invalid names, incl. `id-123`, `_extra-note`,
      `price.details.amount`, and errors for `1x`, `-x`, `a..b`.
      **Done 2026-10-28** — `validate_field_name` + `FieldNameError`
      (`Empty` / `EmptySegment` / `InvalidStart` / `InvalidChar`, 1-based
      segment index in every message; ASCII only, so `ünicode` is rejected).
      One rule table per side: 14 accepted names (every name the proposal's
      assignment block writes) and 10 rejected ones. Trailing/leading/doubled
      dots are all `EmptySegment`, so `a.` and `a..b` cannot drift.
- [x] **S2-2.2 — `Path` + `FieldMap`.** Dep: S2-2.1.
      U: path matching table.
      Do: `Path(Vec<String>)` with `from_dotted`, `starts_with`, `is_prefix_of`;
      `FieldMap` = ordered map of dotted path → `AttrValue`; insertion of a
      dotted key nests (`price.details.amount` → nested object).
      Exit: tests for nesting, prefix match, and conflict detection (scalar where
      an object is needed → error).
      **Done 2026-10-28** — `Path` is only buildable through `from_dotted` /
      `from_segments` (both run the same validator), so no invalid path exists;
      prefix tests are **segment-wise** and inclusive (`priced` is not under
      `price`; `a.b` is a prefix of itself — what `only`/`omit` need).
      `FieldMap` keeps insertion order in a `Vec` (no new dependency) and
      `to_value()` nests into `AttrValue::Object` in first-appearance order;
      re-inserting a path replaces it in place (duplicate policy stays with
      D-12), and a structural clash is `FieldMapError::ParentIsNotObject` /
      `ParentHasChildren`. 13 module tests; workspace 693 pass, 0 fail; no
      golden byte moved.
- [x] **S2-2.3 — Config: `field.*` deserialization.** Dep: S2-2.2.
      Files: `apps/cli/src/config.rs`, `apps/cli/src/components.rs` (or the
      custom-entry type's home).
      Do: `FieldConfig { slot: BTreeMap<String, BTreeMap<String, String>>, specs:
      …, r#override: BTreeMap<String, String> }` via dotted TOML keys; unknown
      `field.*` sub-key → `Severity::Error`.
      Exit: unit tests in `config.rs` parsing the proposal's assignment block
      verbatim.
      **Done 2026-10-28** — the shape landed in `pendon-extra` as `FieldConfig`
      (typed keys `BTreeMap<Slot, BTreeMap<SlotKind, Option<Path>>>`, plus
      `specs` and `overrides`), because the name rules and `Path` already live
      there; `components.rs` only maps TOML onto it and
      `ComponentEntry::field` carries it. `Slot::from_name` / `SlotKind::from_name`
      (inverses of `name()`) keep the key literals in one place. `""` is stored
      as `None` (unassign, OBJ-10); every malformed shape is a load error —
      unknown family (`field.brackett`), unknown kind (`field.bracket.tikc`),
      unknown specs position, non-string value, invalid name (via
      `validate_field_name`) and `field.override` without a table — so nothing
      is ignored (D-16). `field` also joined `ENTRY_KEYS`, so a table whose only
      keys are `field.*` is an _entry_, not a layer named `field`. Tests live in
      `components.rs` (that is where `custom` is parsed): the proposal's
      assignment block verbatim, both override spellings, the duplicate-source
      error, and one case per malformed shape. Workspace 705 pass, no golden
      byte moved.
- [x] **S2-2.4 — Fields-only custom entries (D-8).** Dep: S2-2.3.
      Do: `custom` is always an array; entries without `name`/`template` are
      field-only; typed entry beats the element-wide default; empty entry is an
      error.
      Exit: tests for merge order, the empty-entry error, and "component + field
      config in one entry".
      **Done 2026-10-28** — a single `[task.<plugin>.custom]` table is now a load
      error that names `[[task.<plugin>.custom]]` (a _layer_ table,
      `[task.table.custom.thead]`, is untouched — that is `custom.<layer>`);
      an entry with neither a component nor a `field.*` key is "empty custom
      entry"; and `ComponentSet::field_config(type)` implements the merge order
      (a typed entry's non-empty `field.*` → the layer default's → `None`, so an
      entry without fields is transparent and the preset stays alive). D-8's
      "merges with the component entry for the same type" needed one relaxation:
      `check_duplicate_type` now allows a fields-only entry to share a literal
      with a component entry (two components, or two field configs, still
      clash). `plugins.rs::entry_is_configured` is now
      `ComponentEntry::is_configured`. Fixtures moved to the array form
      (`sandbox/{anchor,cite,extract,universal,unified,img/regular}`,
      `docs/spec/golden/{05-cite,33-extract-scopes}.toml`,
      `apps/cli/tests/extract_spec.rs`); `SYNTAX.md` §11 states the rules.
      Workspace 708 pass, **zero** golden output changed.
- [x] **S2-2.5 — Declare preset assignments per element.** Dep: S2-1.6.
      Files: each `crates/plugin-*/src/lib.rs`.
      Do: replace hard-coded `insert("href", …)`-style calls with the element's
      `SlotContract` + a `preset: &[(&str, FieldName)]` table (identity mapping
      to today's names). Data only.
      Exit: every element's head values flow through the contract table; output
      unchanged (workspace green, no re-freeze).
      **Done 2026-10-28** — `crates/extra/src/preset.rs` holds the vocabulary:
      `FieldName` (a `const fn`-validated, single-segment built-in name, so a
      typo in a table is a **compile** error), `Preset { slot, kind, name }`,
      and `ElementHead { contract, preset }` with `preset_for`, `is_consistent`
      (every preset sits on a contracted position) and `fallback_path`
      (`bracket.quote`, `paren.tick` — D-7's bare name, from `Slot::name` /
      `SlotKind::name`, so the literals still live in one place). The four slot
      elements now declare `pub const HEAD` in their own crate — the only copy of
      the assignment, matching §7.1–§7.4 exactly: anchor
      `bracket.bare→inner_text`, `paren.bare→href`, `paren.quote→title`; cite
      `bracket.bare→reference_id`, `bracket.quote→locator`; heading
      `bracket.bare→slug`, `paren.quote→title`; image `bracket.bare→alt`,
      `paren.bare→src`. Each element claims **all six** positions (a claimed head
      is read whole; an unnamed content becomes a bare field, never a dropped
      one), which is what makes the D-7 fallbacks expressible. Fields that are not
      slot-derived (`depth`, `inner_text`, `citation_number`, section's `id`) stay
      out of the table by construction. `slot.rs`'s element-named contract
      fixtures became one element-neutral `FIXTURE`, so the crate no longer holds
      a second copy of an element fact. 5 new tests in `extra` + 1 per plugin, each
      pinning the table row by row as `bracket.bare -> inner_text`.
      The `insert("href", …)` calls stayed **for now** on purpose: they are not
      pure preset lookups — anchor's are entangled with the URL modifiers,
      `target`/`rel` and §6.2 head-wins-over-extras rules, image's with the figure
      container, heading's with the `slug`→DOM `id` mapping — so deleting them is
      each element's Phase 4 step (S2-4.2…), which reads `HEAD` after
      `parse_slots` replaces the private head parsers. This task's proof is
      therefore the absence of a diff: workspace 716 pass, 0 fail, **zero** golden
      output changed.
- [x] **S2-2.6 — Preset voiding (D-7).** Dep: S2-2.4, S2-2.5.
      Do: any `field.*` in the matched entry disables all preset assignments for
      that element/entry; unassigned slot values fall back to slot-qualified bare
      names (`bracket.quote`, `paren.tick`, …).
      Exit: the D-7 example asserted exactly (no `inner_text`/`href`/`title`).
      **Done 2026-10-28** — `ElementHead::assign(pair, Option<&FieldConfig>)`
      (`crates/extra/src/preset.rs`) walks the contract's positions in head order
      and lands each content present: preset alive (`config` absent or
      `FieldConfig::is_empty`) → the preset name, else the bare name; voided →
      the entry's `field.<slot>.<kind>` assignment, else the bare name. `""`
      (`Some(None)`) is an _unassign_, so it falls back like any other unassigned
      position (OBJ-10) instead of voiding the rest — the whole table is already
      void. Values are `AttrValue::Str`; `field.override` is deliberately not
      applied (that is `project`, S2-2.7). 5 tests, all parsed by the real
      grammar (`parse_slots`): D-7's example asserted row by row — and it
      **disagrees with the doc's spelling**: without a space before each tick,
      §5.2 reads the quote as swallowing it, so the six values come from
      ``@[foo "Bar" `baz`](soo "Sar" `zaz`)``. Footnote added to D-7. Also covered:
      empty entry = preset alive, `""` unassign, a position moved to another
      content (cite's "simpler version"), and a content the contract refuses is
      not read. Duplicate assignment targets are _not_ an error here (last
      position wins in the map) — a load-time check would belong to
      `components.rs`. Nothing calls `assign` yet (Phase 4 wires it), so
      workspace 721 pass, 0 fail, **zero** golden output changed.
- [x] **S2-2.7 — `project(node) -> FieldMap` identity pass.** Dep: S2-2.5.
      Files: `crates/extra/src/field.rs`, `crates/renderer-solid/src/resolve.rs`,
      the HTML/JSON/AST renderers' entry points.
      Do: renderers obtain their attributes from the projection instead of the
      raw node; overrides are read here (D-4) but no override exists yet.
      Exit: `cargo test --workspace` green with **zero** golden changes — this is
      the proof that the refactor is behaviour-free.
      **Done 2026-10-28** — `project(node, Option<&FieldConfig>)` and
      `FieldMap::attrs()` (`crates/extra/src/field.rs`). The projection is the
      identity pass: every attribute of the node becomes one `Path → AttrValue`
      entry in the node's own order, so `attrs()` (flat `(dotted name, JSON
      value)` pairs) is the exact inverse and both renderers emit the same bytes
      as before. What it decides, and the only behaviour it adds: a
      `__`-prefixed key is internal and never enters the map (§11 rule 3), so
      `renderer-solid`'s `render_attrs` / `spread` and `renderer-html`'s
      `render_attrs_except` / `Link` lost their `starts_with("__")` tests; a key
      that is not a valid field name is dropped (`debug_assert!`, so a plugin
      emitting one fails the suite rather than leaking it); and
      `field.override.<path>` renames a field **as it is written**, keeping the
      replaced field's position (D-4 — the one place overrides are read, per
      ADR-0014 rule 4). `AttrValue` gained `Null` plus the total `from_json`
      bridge, without which a `plugin-bind` value could not survive the round
      trip (`{null}` / `null` in the two DOM renderers; a `u64` above `i64::MAX`
      keeps its literal form as `Raw`). The JSON and AST renderers have no
      attribute-emission path to reroute — their `attrs` map _is_ the projection
      in AST form — so the identity is asserted in `pendon-extra`
      (`the_projection_is_an_identity_pass_over_public_fields`) and documented in
      both READMEs. 8 new tests (`from_json` round trip, identity law, empty
      node, override applies in place / is ignored when the field is absent,
      typed values through Solid and HTML), workspace 729 pass, 0 fail, **zero**
      golden and sandbox bytes changed.
- [x] **S2-2.8 — Fail fast on unknown/removed keys (D-16).** Dep: S2-2.3.
      Files: `apps/cli/src/config.rs`, `apps/cli/src/components.rs`,
      `docs/spec/SYNTAX.md` §13.
      Do: (a) delete the four positional-naming fields outright — `backtick_key`,
      `quote_key`, `bracket_key`, `parentheses_key` (`apps/cli/src/components.rs`)
      — since `field.*` replaces them in this phase; no alias, no
      `#[deprecated]`. (b) Make an **unknown key** a `Severity::Error` at load
      instead of a §13 warning, and update §13's table. Element-specific removals
      stay in their own phase task, so this one keeps output byte-identical.
      U: one test per deleted key asserting the load error (not a warning); one
      asserting a config without them still loads.
      Exit: `grep -rn "backtick_key\|quote_key\|bracket_key\|parentheses_key"`
      over `crates/`/`apps/` finds only error text and tests; §13 updated.
      **Done 2026-10-28** — the plan's premise ("`field.*` replaces them" already)
      was wrong: `ExtrasOptions` _was_ the live naming path (`to_attributes`), and
      golden 15 froze its effect (`id`/`level`/`author`/`summary`). So the
      replacement was built before the deletion — `Specs { bracket,
      paren, tick, quote }` (`PositionalNames` until `S2-3.1`) +
      `Specs::resolve(Option<&FieldConfig>)` in
      `crates/extra/src/typed.rs` (D-5 note in `DECISIONS.md`): the groups come
      from `field.bracket.bare` / `field.paren.bare`, the items from
      `field.specs.tick` / `field.specs.quote`, an entry with no `field.*` keeps
      `slug` / `title`, and an unassigned position under a configured entry falls
      back to its qualified path (`bracket.bare`, `paren.bare`, `specs.tick`,
      `specs.quote`). `TypedComponent` now carries the entry's `FieldConfig`
      (`#[serde(skip)]`) instead of a bespoke struct, and `keys_for(_opt)` became
      `names_for(_opt)`. Deleted: `PositionalKeys` (crate + CLI + renderer + the
      four TOML keys) and the `KEY`-named fields everywhere; fixture 15,
      `sandbox/directive` and `sandbox/ultimate` migrate to `field.*` with
      **byte-identical** output. Fail-fast: an unknown entry key is a load error
      (the entry vocabulary is closed) and an unknown `[[task]]` key aborts the
      run with exit 2 (§13 table now says Error; §14 records the removal). Two
      stale sandboxes lost their junk keys (`syntect`'s `css_embed`, `list/
      regular`'s `description`). New tests: one loop asserting each of the four
      removed keys is a load error, a `field.*` entry that still loads, an unknown
      entry key, and an unknown `[[task]]` key (CLI, exit 2). Workspace 733 pass,
      0 fail, **zero** golden bytes moved.

## Phase 3 — specs

Entry: Phase 2 done. Leave when `{}` produces `specs.*` in the projection, is
assignable, and the DOM renderers still emit valid `class`/`id`/`style`.

- [x] **S2-3.1 — Rename "canonical extras" to specs internally.** Dep: S2-2.7.
      Files: `crates/extra/src/typed.rs`, `lib.rs`, `decorator.rs`.
      Do: type/field renames only (`CanonicalExtras` → `Specs`, etc.); no
      behaviour, no config change.
      Exit: workspace green, no output change.
      **Done 2026-10-28** — the concept lives only as a _type_, so the rename is
      `PositionalNames` → `Specs` (+ `Specs::new`/`resolve`), `NamesResolver` →
      `SpecsResolver`, `ComponentSet::names_for(_opt)` → `specs_for(_opt)`, and
      every `keys` binding that held one is now a `specs` binding, in
      `crates/extra`, `renderer-solid` and the eight plugins that read it. The
      prose follows: "positional naming" is "specs naming", and "positional keys"
      is "specs". `Specs`'s doc now also carries the §6.1 **`default` /
      `meaning`** table (`tick`/`quote`/`bracket.bare`/`paren.bare` → `slug` /
      `title` / `slug` / `title`, and what each content _is_): an assigned field
      keeps no semantics, so without a copy in code `S2-3.5` would delete the
      last one. No behaviour, no config, no golden byte.
- [x] **S2-3.2 — Emit `specs.*`.** Dep: S2-3.1.
      Do: projection inserts `specs.tick`, `specs.quote`, `specs.class`,
      `specs.id`, `specs.style`; `k: v` stays `k`; duplicate `tick`/`quote`/`id`
      → error keeping the first (D-12); `class` accumulates and dedupes; `style`
      duplicates last-wins; `k` colliding with a built-in field → error.
      U: one table per D-12 rule.
      Note: where the paths are _produced_ is not free. `project` is called with
      `None` from the five renderer sites (`renderer-solid/src/node.rs` ×2,
      `renderer-html/src/{utils,compact,pretty}.rs`), so a rendering-time mapping
      has no `FieldConfig` and cannot tell an assigned name from a default one —
      and an attribute-name mapping is unsound anyway, because `slug` (a
      heading's auto slug), `title` (anchor head group, wiki, image) and `id`
      (section / heading / cite ids) are emitted by plugins, not by a specs
      position. So the default paths have to be produced where the config is
      known (`to_attributes` + `Specs`, parse time); the ten plugins that read
      `class` / `id` / `style` / tick / quote off the head, `plugin-extract`'s
      field tables (`EXTRAS-INVENTORY.md`) and every golden then move with it.
      **Done 2026-10-28.** `to_attributes(head, specs)` emits `specs.class` /
      `specs.id` / `specs.style` / `specs.tick` / `specs.quote` (through `Specs`,
      so an assignment renames them) and every fixture moved with it. The D-12
      table is implemented in `crates/extra/src/typed.rs` and pinned by
      `crates/extra/tests/extras_spec.rs`: a case-per-rule table
      (`duplicates_are_an_error_and_the_first_value_wins`), a class-dedupe test
      (`class_accumulates_and_dedupes`) and a colliding-assigned-names test;
      `ExtrasIssue::owned_key` carries the built-in-field rule. Rule 5 (`k`
      colliding with a built-in field) is only _partly_ landed: it needs each
      element's built-in list, so the wiring is **deferred to S2-4.1** — an
      anchor's `href:` is already an `Error`, the rest follow with their
      contracts.
- [x] **S2-3.3 — `field.specs.*` assignment.** Dep: S2-3.2.
      Do: assignment renames the paths; `backtick_key`/`quote_key` no longer exist
      (deleted in S2-2.8 — a config that still uses them fails to load, per D-16).
      Exit: the proposal's `field.specs.*` block produces its documented output;
      a test asserts the old key is a load error, not a warning.
      **Done 2026-10-28** — an assignment is a **move**, not a rename:
      `SpecsKey` (`Tick`/`Quote`/`Class`/`Id`/`Style`) names the five positions,
      `Specs::path` says where one is published and the new
      `Specs::element_path` answers the **default** path only while it is
      unassigned (`None` once moved). Every element's own read goes through
      `element_path` (`plugin-anchor` / `-cite` / `-heading` / `-image` /
      `-section` / `-table` / `-wiki`), and `Specs::dom_fields` matches the
      element pairs on the **path**, so a moved position is neither consumed as
      an id/title nor renamed to `class`/`id`/`style` — `foo.class` /
      `bar.id` / `foo.style` come out verbatim in solid, html and json while the
      default paths stay empty and the built-in `<a>` keeps `class`/`id`/`title`.
      Tests: `specs_keys_cover_the_specs_vocabulary`,
      `every_specs_position_moves_with_field_specs`,
      `dom_fields_projects_only_default_paths` (`crates/extra`),
      `an_assigned_specs_path_is_a_field_of_its_own` (`plugin-anchor`) and
      `anchor_options_carry_the_entry_specs` (CLI: the entry's assignment reaches
      `AnchorOptions`); the removed-key load error is
      `each_removed_naming_key_is_a_load_error`.
- [x] **S2-3.4 — Renderer mapping for `specs.*`.** Dep: S2-3.2.
      Files: `crates/renderer-solid`, `crates/renderer-html` (and JSON/AST as
      needed).
      Do: solid → `class`/`classList`/`style`/`id` attributes; HTML → real
      `class`/`id`/`style` attributes; JSON/AST keep the dotted paths.
      Exit: one `.jsx` + one `.html` test per specs position.
      **Done 2026-10-28** — `field::project` renames the three universal
      positions (`universal_field` in `crates/extra::typed`) and sorts entries by
      their **final** public path, because a node's `attrs` arrive name-sorted and
      `specs.class` → `class` would otherwise move the attribute it writes. An
      element assignment still wins (`cite-id` over `id`), which is why every
      plugin that emits a head field itself now projects through
      `Specs::dom_fields(items, …)`, and why `DecoratorLine`/`BoundDecorator`
      carry the `specs` the line was parsed with. `renderer-html` prints the map;
      `renderer-json`/`-ast` never rename (they keep the dotted path).
      `renderer-solid` cannot spell `specs.tick=` — `a.b` is a member expression —
      so a dotted field is emitted as a one-key spread, `<p {...{ "specs.tick":
      "x" }}>`, which sets in the DOM exactly what HTML prints instead of emitting
      invalid JSX (`is_jsx_attr_name`/`render_dotted_attr` in
      `renderer-solid/src/node.rs`). Tests: five per renderer, one per specs
      position (`specs_class_renders_as_the_dom_class`, `…_id`, `…_style`,
      `specs_tick_keeps_its_dotted_name`, `specs_quote_keeps_its_dotted_name`).
      Fixtures `10-decorator-blocks` and `14-marker` re-frozen: every moved byte is
      a rename — `slug`/`title` → `bracket.bare`/`paren.bare` (D-7: an untyped
      group has no preset), `slug`/`title` → `specs.tick`/`specs.quote` (marker
      items), and the decorator's own `class` → `specs.class` in the event stream.
      Not in this pass: `plugin-extract`'s tables and `EXTRAS-INVENTORY.md` still
      name the pre-3.2 fields for the raw-decorator elements (blockquote, list,
      marker, directive emit `specs.class` / `bracket.bare` verbatim because
      extract reads the events, not the map) — Phase 4 rewrites those elements and
      Phase 7 freezes the tables, so they move then ("Phase 4 before 7").
- [x] **S2-3.5 — Re-freeze and document specs.** Dep: S2-3.2 … S2-3.4.
      G: specs fixtures (class/style/id/tick/quote, assigned + unassigned).
      Do: `SYNTAX.md` §5/§6.3/§6.4 update, §14 entry for the old flat names.
      Exit: fixtures frozen, `crates/*/README.md` links updated, phase gate green.
      Note: §6.1 is **rewritten** here, not reworded — its `default` / `meaning`
      columns are the only prose copy of what each content is, and the code copy
      is `Specs`'s doc (`crates/extra/src/typed.rs`). Re-derive the table from
      that doc (`tick` = slug, `quote` = title, `bracket.bare` = bracket slot,
      `paren.bare` = parentheses slot) instead of deleting the columns with the
      section, and state each one's new default path (`specs.tick`,
      `specs.quote`, …) so a reader can tell a default from an assignment.
      **Done 2026-10-28.** §6.1 is the seven-row table (`content` / `config key` /
      `default path` / `meaning`), and the old `default` column is gone because it
      was wrong: `slug` / `title` are the **element preset** (§6.5, D-7 — an
      assignment), not the path, so the default column now names the path
      (`specs.tick` … `bracket.bare`) and points at `SpecsKey::default_path`.
      The §6.5 that five crates already cited did not exist and is now written;
      §5.1's worked example lists public fields instead of attributes, §6.4 names
      positions and paths and states the projection's sort (the "restate me"
      reminder is gone), and §14's removed-keys row no longer promises `slug` /
      `title`. New fixture `34-specs-assignment` is the `G:` box — one entry
      assigning all five positions, an unclaimed type keeping every default in the
      same document (`bar.id` / `foo.class` / `foo.style` / `slug` / `title`
      verbatim, no `class`/`id`/`style` rename, and no id fallback). Nine plugin
      READMEs replaced the copy-pasted "keeps `slug` / `title` / `class` / `id` /
      `style`" sentence (wrong since S2-3.2) with a link to §6.1. No other golden
      moved; gate green: fmt, clippy (no new warnings), 751 tests.
- [x] **S2-3.6 — Dotted fields on the component path.** Dep: S2-3.4. `§6.3`
      Do: make the component `{...attrs}` spread spell a dotted path the way the
      built-in elements already do.
      G: a fixture whose component carries `specs.*` / `bracket.*` fields.
      Exit: no `.jsx` in the repo contains a dotted attribute name.
      **Done 2026-10-28 — S2-3.4 fixed one of the two paths.** `node.rs` emitted
      `{...{ "specs.tick": "x" }}`, but `template.rs`'s `generate_spread_attrs`
      (the custom-component `{...attrs}` spread) still wrote `key` verbatim, so a
      component received `<Bibliography bracket.bare="refs" … specs.tick="…">` —
      **not JSX at all** (`a.b=` is a member expression), and that invalid line had
      been frozen into `14-marker.jsx` since S2-3.4. The fix is the rule, not a new
      one: `render_dotted_attr` / `is_jsx_attr_name` became `pub(crate)` and the
      template path calls them, so a field is spelled identically whether a
      component claims the node or the fallback element renders it. Fixtures
      `14-marker` (`bracket.bare`, `paren.bare`, `specs.quote`, `specs.tick`) and
      `34-specs-assignment` (`bar.id`, `foo.class`, `foo.style`) are the `G:` box;
      every moved byte is one dotted name becoming a one-key spread, positions
      unchanged. New unit test
      `a_component_receives_a_dotted_field_as_a_one_key_spread`, plus a guard in
      the golden harness (`assert_no_dotted_attribute_name`, run on **every**
      fixture render, self-tested by
      `the_dotted_attribute_guard_catches_the_invalid_spelling`) so no future
      golden can freeze an invalid name. **Rejected:** a nested props object
      (`specs={{ tick: … }}`) — it would make solid the only renderer of the three
      that regroups the path, break parity with `renderer-html`'s
      `specs.tick="…"` and JSON/AST's path, and set `[object Object]` on a
      built-in element, which shares `render_attrs` ([D-18](DECISIONS.md)). 763
      tests.

## Phase 4 — elements, one at a time

Entry: Phases 1–3 done (**except** table's decorator half, which needs Phase 5).
Leave when every element reads its head through `SlotContract` + the projection,
and each element's fixture is re-frozen.

Element tasks share a shape: **(1)** wire the contract, **(2)** move the preset
assignments into the projection, **(3)** delete the element's private head code,
**(4)** re-freeze that element's fixture, **(5)** update its `SYNTAX.md` §7.x and
README. If an element has nothing to delete, say so in the commit message.

- [x] **S2-4.1 — Extras are universal.** Dep: S2-3.5.
      Files: `crates/extra/src/lib.rs`, `decorator.rs`, every plugin head.
      Do: audit that `@@type`/`{specs}`/`%%numbering` are accepted on _every_
      element head and (via decorator) every block; keep `{{type}}`/`::type`/
      `==type` precedence rules (§10.4). Fix any element that reads extras
      privately.
      Exit: a table-driven test "extras on all element heads"; no behaviour
      regression.
      Also finish **D-12 rule 5** here: an element names the keys it writes
      itself (`href` on an anchor, `src` on an image, `start` on an ordered
      list) and reports a colliding `k: v` as `ExtrasIssue::owned_key` — an
      `Error` pointing at `field.override.<k>` (D-4) — instead of dropping it
      silently or last-wins. `plugin-anchor` is the reference; the remaining
      heads follow with their contracts (S2-4.2 … S2-4.8).
      **Done 2026-10-28.** The audit is
      `apps/cli/tests/extras_heads_spec.rs`: 17 rows — anchor, image, cite,
      heading, wiki, marker, `::`/`==` directive, table decl, table cell, quote
      inner head, quote decorator, list container, section, code fence,
      paragraph, and a `%%numbering` row — each rendered from one project with
      the **same** head (all five specs positions + `%%%numberA`) and asserted on
      its element's projection, plus two universal checks per row (no literal
      `@@`, no literal marker). No element failed: every head already funnels
      through `to_attributes` + `Specs::dom_fields` (§6.3), so "universal" was
      already true and is now proved. One anomaly recorded, not fixed: the table
      **declaration's** own `("T")` loses its `title` to an extras quote (a §6.2
      inversion in `plugin-table`'s layer props) — the decl line is Phase 5's
      (`S2-4.11`), so the row documents today's shape and names the gap.
      D-12 rule 5 is now **data**: `ElementHead.owned: &'static [FieldName]`
      (`with_owned` / `owns`) plus `owns_field(&owned, key)` for an element with
      no contract, both in `crates/extra/src/preset.rs`; `plugin-anchor` declares
      `href` and reads `HEAD.owns(key)` instead of `if key == "href"`, and
      `plugin-wiki` replaces its `WIKI_OWNED_KEYS: [&str; 2]` with
      `&[FieldName; 2]` (`href`, `title`). Tests:
      `an_element_declares_the_fields_it_writes_itself` (`crates/extra`, incl.
      "an untouched head owns nothing") and
      `title_is_not_overridable_by_extras` (`plugin-wiki`, the `Error` the
      `[&str]` list never proved); `href_stays_construct_owned` still passes.
      Docs: §6.5 states the owned list, D-12's note records the wiring, and the
      `crates/extra` README now describes the triple. Gate green: 753 tests.
- [x] **S2-4.2 — Anchor.** Dep: S2-4.1. `§7.2`
      Do: `@` tag claims `[]`/`()`; contract `bracket.bare→inner_text`,
      `paren.bare→href`, `paren.quote→title`; delete external-URL detection
      (`target="_blank"`, `rel="noopener"`), URL-suffix modifiers, and the
      conflicting-modifier warnings (D-16: §14 records both removals as "not
      syntax"; no replacement column).
      G: anchor fixture (all six positions, + removed behaviour as literal text).
      Exit: preset assignment = `inner_text`, `href`, `title`; removals covered
      by fixtures.
      **Done 2026-10-28.** The head is read by the shared grammar now:
      `parse_slots` + `parse_extras` on byte offsets replaced `parse_link` /
      `split_target_title` / `find_char` / `find_matching_paren`, so all six
      positions are parsed and the tag is enforced (`SlotTag::Anchor`; an
      untagged `[x](y)` is nobody's head and reaches `plugin-markdown`).
      `HEAD.assign(pair, matched entry's field)` is the projection — `inner_text`
      becomes the text the `<a>` wraps, `href` / `title` are attributes, and
      `bracket.quote` / `bracket.tick` / `paren.tick` are published under their
      slot-qualified names (D-7). Deleted: `strip_url_modifiers`, `is_external`,
      `add_rel`, the `^`/`~` conflict warning, and the `rel`/`target`
      special-casing (they are ordinary extras keys). New plumbing:
      `ComponentSet::field_for` (the matched entry's `FieldConfig`, one resolution
      rule shared with `specs_for_opt`) and `impl Display for SlotError`.
      **Finding — D-7's voiding trigger narrowed to slot keys** (dated note on
      D-7): reading "any `field.*` key" literally would strip `href` from every
      anchor whose entry renames a _specs_ position (fixture 34's `moveA`), so
      `ElementHead::assign` now asks `FieldConfig::is_empty_slots()`
      (`a_specs_or_override_assignment_keeps_the_preset`).
      Fixture `04-anchor` rewritten (tag vs untagged, six positions, inline tag,
      no external defaults, `^!$;;--` as URL text, `target`/`rel` as extras) and
      re-frozen; the nine fixtures that use the element gained their `@`
      (`01`, `02`, `04`, `21`, `23`, `24`, `27`, `31`, `34`), and only `04` and
      `23` moved a byte — the removal visible in `23` is the markdown link inside
      an `mdp` payload losing `target`/`rel`. Gate green: fmt + dprint clean,
      clippy at baseline, 759 tests.
- [x] **S2-4.3 — Blockquote.** Dep: S2-4.1. `§9.2`
      Do: remove slots/extras _inside_ `>`; only the decorator configures a
      blockquote; `/`-content keeps its own slots. No built-in fields.
      G: blockquote fixture — decorator form renders, `> @@type…` becomes literal
      text.
      Exit: contract table for blockquote is empty and documented as such.
      **Done 2026-10-28.** `emit_quote` lost its inner-head branch (and the
      `options`/`parse_extras`/`to_attributes`/`Attrs` imports it needed): the
      body is now copied with one `>` level stripped and nothing else, so the
      decorator line is the whole of a quote's extras. The element declares no
      `ElementHead` at all — no slot contract, no preset, no owned fields — and
      §9.2 says so. Tests: `an_inner_head_is_literal_text`, `the_body_keeps_its_
      own_slots` (a slot in the body belongs to its own element) and
      `an_inner_head_is_not_a_head` (end-to-end, solid), replacing the three
      inner-head cases; `quote-inner` left the S2-4.1 audit table (the decorator
      row still proves the head is read). Fixture `11-blockquote` rewritten
      (decorator-only + an inner `@@` that stays literal) and re-frozen; §14's
      "not syntax" list gained the inner head, §4.1's adjacency exception now
      names the list item only, and §3's `>` row says `decorator only`. Gate
      green: fmt + dprint clean, clippy at baseline, 759 tests.
- [x] **S2-4.4 — Cite.** Dep: S2-4.2 (shares slot machinery), OBJ-1 resolved.
      `§7.3`
      Do: `^` tag; `bracket.bare→reference_id`, `bracket.quote→locator`;
      `reference_id` mandatory (error when the preset is voided and nothing
      assigns it); keep citation numbering working off the _internal_ keys (D-4);
      delete the config keys `citation_class`, `ref_id_prefix`,
      `citation_id_prefix` (plus the pre-RFC-0002 `prefix`/`class`/`id_prefix`
      shim in `plugins.rs`) and the plugin props behind them, and drop the
      `instance_index` public field — all with no alias and no deprecation warning
      (D-16); support `field.bracket.bare = ""` = unassign; verify
      `[task.cite] reference_source/reference_file/references_export/
      citations_export/references_scope` exist, add what is missing.
      G: cite fixture incl. the proposal's "simpler version" config.
      Exit: preset assignment = `reference_id`, `locator`, `citation_number`;
      overrides
      (`ref_id`, `no`, `loc`) work without breaking numbering.
      **Done 2026-10-28.** The hand-rolled `[^^](ref "loc")` scanner is gone:
      `read_head` uses `parse_slots` + `parse_extras` like the anchor, so the head
      is `[^reference_id "locator"](paren…)` with `HEAD.assign` deciding where each
      position lands. `reference_id` is mandatory — a head that assigns it nowhere
      (`[^]`, `[^"book"]`, or an entry that voided the preset without redefining
      it) is an `Error` and the text stays literal; `citation_number` is declared
      as the element's **owned** field (D-12 rule 5), so an extras item spelling
      it is an `Error`. `instance_index` is deleted from the node, the record and
      the extract table (§14) — only the built-in markup's backlink anchor still
      needs a per-occurrence counter, and it stays internal. The three config keys
      and the pre-RFC-0002 shim are deleted, so a stale config is an unknown-key
      **load error**; the built-in `<sup>` class and its two prefixes are now
      constants, which is the documented reason to configure a `Cite` component.
      **A gap this closed:** `field.override.*` was documented but inert — every
      renderer called `project(v, None)` (S2-3.2's note). `ComponentTemplate`
      now carries the winning entry's `FieldConfig` (`with_field`, set by
      `layer_templates`) and the component path projects with it, so the exit
      box's `ref_id` / `no` / `loc` renames are real (new renderer test
      `an_override_renames_a_component_prop`); the processor keeps reading its own
      names (D-4, new test `an_override_does_not_break_numbering`).
      Fixtures: `05-cite` rewritten (preset, paren positions, extras, identity,
      the two error cases), `26-cite-scope` re-frozen (one citations row),
      `27-bind-mdp` + `33-extract-scopes` (and its three side files) migrated, new
      **`35-cite-fields`** is the `G:` box — the proposal's "simpler version"
      (`field.bracket.tick`/`quote` + the three overrides). `citations` rows are
      now `{reference_id, locator?, citation_number}` (D-19). Sandbox sources and
      configs migrated (no re-render). 768 tests.
- [x] **S2-4.5 — Directive: verify "no change".** Dep: S2-4.1. `§10.2–§10.3`
      Do: confirm `::type` / `==type` are untouched and `type` stays in the
      directive preset (it does not come from extras). Add the missing regression
      test if absent.
      Exit: an explicit test asserting directive `type` survives a decorator
      with `@@other`.
      **Done 2026-10-28 — no code change, as expected.** Verified: the inline and
      block forms still read their head through `resolve_head`, `type` comes from
      the directive head (`head.type_marker`, `ParsedDirective::type_name`), never
      from the extras items, and `plugin-extract`'s `directive` field table still
      lists `type` alongside `id` / `level` / `summary` / `inner_text`. The audit
      row (`extras_heads_spec`) already covers both heads. The missing test is
      `a_decorator_never_overrides_the_directive_type`
      (`crates/plugin-directive/tests/pipeline.rs`): a `@@other{.…}` line above a
      block directive leaves it `type="note"` with its own class, and the inline
      form keeps its type too. Recorded, not fixed: until `S2-5.3` makes a
      directive _reject_ a decorator, the line binds to the first block of the
      body — the invariant the test pins is the one that must survive that change.
      §10.3 gained one sentence stating that `type` is the head's, not extras'.
- [ ] **S2-4.6 — Heading.** Dep: S2-4.2. `§7.4`
      Do: `#` tag (the `#` immediately before `[` is the tag; the rest is depth,
      so today's `##[…]` becomes depth 1 — state this in §7.4);
      `bracket.bare→slug`, `paren.quote→title`; rename `level`→`depth`,
      `raw_title`→`title`; `slug` falls back to a slugified `inner_text` and
      **no longer** falls back to specs; delete all numbering from
      `plugin-heading` (`auto_number`/`number_style` already raise a §14 load
      error — keep that, and drop the now-dead fields once S2-9 lands); heading
      cannot consume a decorator.
      G: heading fixture incl. `######[]()`, numbered-via-`number` replacement.
      Exit: preset assignment = `slug`, `depth`, `inner_text`; `plugin-heading`
      has no numbering code left.
- [ ] **S2-4.7 — Image.** Dep: S2-4.2. `§7.1`
      Do: `!` tag; `bracket.bare→alt`, `paren.bare→src`; delete the marker-
      modifier run (`?`, `~`, `w<digits>`, `h<digits>`) and the `!!` figure
      doubling, with figure/figcaption and their `[task.image.custom.figure]`
      layer; `specs.*` + slots replace them. Today a bare `!` is _not_ an image
      (it needs a modifier), so state the new minimum in `SYNTAX.md` §7.1.
      G: image fixture; `~?!![](src)` renders as literal text.
      Exit: preset assignment = `src`, `alt`; no `figure`/`figcaption` in any
      fixture output.
- [x] **S2-4.8 — List.** Dep: S2-4.1. `§9.3–§9.4`
      Do: no slot contract; decorator only (wired in Phase 5); preset
      assignment empty
      (drop the extras-derived fields from the inventory rows).
      G: list fixture with a decorator.
      Exit: preset assignment is empty in code and inventory.
      **Done 2026-10-28 — the list was already there.** `plugin-list` declares no
      `ElementHead`, no `SlotContract` and no `Preset`: it binds the decorator line
      only and passes its extras through, so its preset assignment is empty in code
      (`golden/12-list-container` already exercises the decorator form). §9.3 now
      says so explicitly, and its stale bullet ("positional slots default to
      `slug` and `title`" — pre-S2-3.5 wording) became §6.1's default paths plus
      "there is no list preset to void". New test
      `a_head_inside_an_item_is_content`: a `@@type{…}` written at the start of an
      item contributes nothing to the container. The inventory half of the exit
      belongs to **`S2-7.2`**, which rewrites every element's table to "only
      built-in fields, drop everything that comes from extras" and marks preset
      membership per field — doing it here would be undone there.
- [x] **S2-4.9 — Marker.** Dep: S2-4.1. `§10.1`
      Do: `{{type}}[slot](slot)@@type{specs}%%numbering`; `{{type}}` silently
      overrides `@@type` (no error, no warning); marker contracts nothing; the
      built-in field is `type` only (no `inner_text`).
      G: marker fixture with both type sources.
      Exit: preset assignment = `type`; the override precedence is asserted.
      **Done 2026-10-28.** The head grammar was already right (groups directly
      after `}}`, then the extras head, then `%%numbering`); the one behaviour
      that was wrong is the override: `plugin-marker` reported an
      `ExtrasIssue::warning` when an extras head carried a `@@type`. That warning
      is gone — one node has one routing key, so `{{type}}` wins **silently**
      (§10.1). `an_extras_type_is_reported_and_ignored` became
      `an_extras_type_is_overridden_silently`, which also pins that `type` is the
      only built-in field (no `inner_text` attribute; the text is the children).
      No contract, no preset: the marker declares no `ElementHead`, and §10.1 says
      so in its first bullet. Fixture `14-marker` gained both type sources
      (`{{aside}}@@asideAB{.marked, isOpen}` → `type="aside"`, extras kept, no
      diagnostic) and was re-frozen; §10.1's obsolete "a `[x]("y")` pair is also a
      legal anchor head, so layer order decides" note became "an anchor is
      `@[x]("y")`, so nothing competes for a marker's groups" (S2-4.2's tag makes
      it true). Gate green: fmt + dprint clean, clippy at baseline, 760 tests.
- [ ] **S2-4.10 — Section.** Dep: S2-4.6. `§9.5`
      Do: section has no head; preset assignment = `id`, `depth` (from heading
      depth); a
      decorator above a heading is claimed by section (Phase 5 implements the
      claim; here wire section's read side).
      G: section fixture with `depth`/`id`.
      Exit: preset assignment = `id`, `depth`.
- [ ] **S2-4.11 — Table layers (non-decorator half).** Dep: S2-4.1. `§8`
      Do: keep `@@captionType`/`@@cellType`/`@@rowType` on layers; make every
      layer accept full slots **and** `{specs}` **and** `%%numbering` (today only
      type+specs); leave the table head line for Phase 5.
      G: table fixture exercising slots+numbering on a cell.
      Exit: no layer rejects a valid extras combination.
- [ ] **S2-4.12 — Bare-field universality.** Dep: S2-4.2 … S2-4.11.
      Do: assert that _any_ uncontracted slot content survives as slot-qualified
      bare fields (`bracket.tick`, `paren.quote`, …) on every element, and that
      nothing is silently dropped.
      G: one "kitchen sink" fixture per element group (inline + block).
      Exit: no element drops slot content without a diagnostic.

## Phase 5 — decorator unification

Entry: Phase 4 done except the table head; D-10 settled. Leave when `@@` is
mandatory and first, five receivers work, and both rejection paths warn.

- [ ] **S2-5.1 — `@@` mandatory, and first.** Dep: S2-4.12.
      Files: `crates/extra/src/decorator.rs`, `lib.rs`.
      Do: decorator order becomes `@@type` → slot → `{specs}` → `%%numbering`;
      the old `type`-last / no-`@@` form stops being a decorator (literal text,
      §14 entry); keep the "attached or separated by one line" tolerance.
      G: decorator ordering fixture (valid) + old form as literal (invalid).
      Exit: a decorator missing `@@` never configures the block below it.
- [ ] **S2-5.2 — Claim table.** Dep: S2-5.1.
      Do: implement D-10's order — `section` → `table` → `list` → `blockquote`
      → `paragraph`; one claim per receiver; unclaimed decorator is removed with
      `Warning`; trailing literal text on a decorator line is dropped with
      `Warning`; `section` claiming before a heading is silent.
      U: claim-order unit test; G: 3 rejection fixtures + section case.
- [ ] **S2-5.3 — New receivers.** Dep: S2-5.2.
      Do: paragraph, codefence, list and table obtain the claimed decorator's
      slots/specs/number as public fields; heading/marker/directive/frontmatter
      and data blocks reject it (D-10 §3).
      G: one fixture per receiver.
      Exit: each of the five receivers renders decorator-derived fields.
- [ ] **S2-5.4 — Table head through the decorator.** Dep: S2-5.3 (and S2-4.11).
      Do: delete `|-[slug]("Title")@@type{specs}-|`; the table head is a plain
      `|…|` line and the decorator supplies everything; confirm no table layer
      contracts slot parts; add the §14 entry.
      G: table fixture before/after (old head becomes literal text).
      Exit: table has zero head grammar of its own.

## Phase 6 — slot modifiers

Entry: Phases 1 + 5 done; D-9 settled. Leave when modifiers configure specs
through config, merge per D-9, and the legacy modifier code is gone.

- [ ] **S2-6.1 — Config for modifiers.** Dep: S2-5.4.
      Files: `apps/cli/src/config.rs`, `crates/extra/src/field.rs`.
      Do: parse `<slot>.modifier."<symbol>"` (any UTF-8 symbol, quoted) into a
      contribution map; unknown slot name → error; a symbol with no config is
      ignored (no warning — a modifier without config is legal).
      U: parsing tests incl. `||`, `Œ`, `\`, `ئ`.
- [ ] **S2-6.2 — Merge and projection.** Dep: S2-6.1.
      Do: apply contributions in canonical order (D-9) into the projection;
      same-path merge rules (array/scalar/nested-array/deep-merge); irreconcilable
      → `Severity::Error` + drop that contribution.
      G: the proposal's `rel: ["nofollow","noopener"]` + `soo.baz: ["wax","wax"]`
      example, byte-for-byte.
      Exit: extraction output matches the proposal's JSON exactly.
- [ ] **S2-6.3 — Delete the legacy modifiers.** Dep: S2-6.2.
      Do: remove the anchor URL-suffix modifiers (`^`, `~`, `!`, `$`, `;;`, `--`
      after the URL in `@[text](url…MOD)`) and the image marker modifiers
      (`?`, `~`, `w<n>`, `h<n>`); remove the conflicting-`^`/`~` warning and the
      "extras replaced the modifier" warnings; add their §14 lines (removal only —
      no replacement column, D-16).
      G: fixtures where the old symbols now render literally.
      Exit: `grep` finds no old modifier parsing in any plugin.
      Note (2026-10-28): the anchor half is already in — `S2-4.2` deleted both the
      URL-suffix modifiers and the external-URL defaults and wrote their §14 rows
      — and `S2-4.7` deletes the image marker run, so what is left here is the
      `grep` that proves neither parser survives.
- [ ] **S2-6.4 — Modifier visibility in extraction.** Dep: S2-6.2 (and S2-8.2).
      Do: modifier-produced fields are extractable (`preset = "all"`, `include`,
      `only`) and appear in the inventory as _user_ fields, never in the preset.
      U + G: extract fixture with and without a modifier.
      Exit: `preset = "pendon"` excludes them; `preset = "all"` includes them.

## Phase 7 — inventory

Entry: Phase 4–6 done (element fields are final). Leave when
`FIELDS-INVENTORY.md` is the single list of built-in fields and the presets in
code match it exactly.

- [ ] **S2-7.1 — Rename the document.** Dep: S2-6.4.
      Do: `git mv docs/spec/EXTRAS-INVENTORY.md docs/spec/FIELDS-INVENTORY.md`;
      retitle; update every inbound link (`AGENTS.md`, `docs/INDEX.md`, plugin
      READMEs, `SYNTAX.md`).
      Exit: `grep -r EXTRAS-INVENTORY` returns nothing.
- [ ] **S2-7.2 — Rewrite the tables.** Dep: S2-7.1.
      Do: per element, list only built-in fields; drop everything that comes from
      extras or from custom assignments (including `type`, except
      directive/marker); mark preset membership per field.
      Exit: an element with no built-in fields shows an explicit "none".
- [ ] **S2-7.3 — Add the contracts.** Dep: S2-7.2.
      Do: per element, a slot-contract table (which `slot.kind` is contracted to
      which field), the slot-qualified bare fallbacks, and the `specs.*` /
      `number.*` shared tables.
      Exit: every `SlotContract` in code has a matching row; no row without code.
- [ ] **S2-7.4 — Reconcile with code.** Dep: S2-7.3.
      Do: make the extract presets (`preset = "pendon"`) read from one table in
      code that mirrors the doc, so drift is a compile/test failure rather than a
      doc bug. Add the cross-check test.
      Exit: renaming a preset field without touching the doc fails a test.

## Phase 8 — `extract`

Entry: Phase 7 done; D-6, D-13, D-14 settled. Leave when `preset`/`include`/
`only`/`omit` work on dotted paths, the default is `pendon`, and PE.1–PE.4 +
frontmatter fixtures are frozen.

- [ ] **S2-8.1 — Promote the crate.** Dep: S2-7.4.
      Do: `git mv crates/plugin-extract crates/extract`; package
      `pendon-extract`; update the workspace member list, `apps/cli/Cargo.toml`
      imports (`plugins.rs`, `process.rs`, `config.rs` doc comments) and the crate
      README. The `Plugin` trait import must not change chain order.
      Exit: `cargo test --workspace` green, zero output change.
- [ ] **S2-8.2 — Preset / include / only / omit.** Dep: S2-8.1.
      Do: `preset = "pendon" | "all"` (default `pendon`), `include: String |
      [String]`, all D-6 validation. Delete the `recommended` field in this task
      (no alias, no mapping): a config that still writes it is a load
      `Severity::Error` (D-16).
      U: config validation table (every D-6 bullet) + the `recommended` load error.
      Exit: every D-6 rule has a test; `recommended` produces `Severity::Error`.
- [ ] **S2-8.3 — Nested path selection.** Dep: S2-8.2.
      Do: `include`/`only`/`omit` operate on dotted paths against the projected
      `FieldMap`; a prefix selects the whole subtree; `omit` wins over
      `include`/`only`; an unmatched path is silently empty (PE.4 ⇒ `[{}]`, not
      an error).
      U: path algebra table; G: PE.1, PE.3, PE.4.
      Exit: PE.4 produces exactly `[{}]`.
- [ ] **S2-8.4 — Extract final public fields.** Dep: S2-8.3 (needs D-3's
      projection). Do: read the projection only — `specs.*`, `number.*`,
      slot-derived, override-renamed, modifier-derived; never internals
      (`reference_id`, `__number`).
      G: PE.1, PE.2 (type-scoped) and the §9 `preset = "all"` example.
      Exit: PE.1–PE.4 byte-exact; `preset = "all"` excludes internals.
- [ ] **S2-8.5 — Frontmatter extraction.** Dep: S2-8.4.
      Do: `[[task.frontmatter.extract]]` (D-14); no preset; `only`/`omit` on
      nested keys; all four formats; duplicate `export const frontmatter` →
      error.
      G: one fixture per format + the duplicate error.
- [ ] **S2-8.6 — Flip the default and re-freeze.** Dep: S2-8.5.
      Do: every existing extract config without `preset` now yields the pendon
      preset; re-freeze the extract goldens; record the intentional shrink in the
      commit message.
      Exit: no un-attributed change in any `.json`/`.yaml`/`.toml` golden.

## Phase 9 — `number`

Entry: Phase 8 done; D-13 (number) settled. Leave when numbering is optional for
every element, exposes `number.*`, and is extractable.

- [ ] **S2-9.1 — Promote the crate.** Dep: S2-8.6.
      Do: `git mv crates/plugin-number crates/number`; package `pendon-number`;
      workspace + CLI imports + README.
      Exit: green, no output change.
- [ ] **S2-9.2 — Public `number.*`.** Dep: S2-9.1.
      Do: projection exposes `number.type`, `number.raw`, `number.depth`; keep
      `__number` internal for nesting (D-4); `number.depth` is the same value the
      nesting counter uses (OBJ-6).
      G: numbered fixture across three levels.
      Exit: internal attrs never appear in output; `number.*` does.
- [ ] **S2-9.3 — Overrides for numbering.** Dep: S2-9.2.
      Do: `field.override.number.type|raw|depth`, both per element and in
      `[[task.number.custom]]`; numbering keeps working after the rename.
      G: fixture with all three overridden.
      Exit: overridden names are correct _and_ nesting order is unchanged.
- [ ] **S2-9.4 — Extract numbering.** Dep: S2-9.3 (and S2-8.4).
      Do: `number.*` extractable from any element that uses numbering; excluded
      from `preset = "pendon"` (PE.1), reachable via `include`/`only`/
      `preset = "all"`.
      G: PE.1 re-frozen with `number.raw`.
      Exit: PE.1 shows `number.raw` only.

## Phase 10 — editor, docs, gates

Entry: Phases 0–9 done. Leave when the workspace is green, the sandboxes are
re-rendered, the editor grammar matches, and `STATUS.md` is quiet again.

- [ ] **S2-10.1 — VSCode grammar.** Dep: S2-9.4.
      Files: `editors/vscode/**`.
      Do: highlight `@@` (first), slot tags `@ ^ ! #`, the three slot content
      kinds, the modifier run, `specs.*` fields and `field.*` config keys; update
      the scope harness; link `SYNTAX.md` §5.2–§5.4 (never restate the grammar).
      Exit: harness green + one grammar case per construct.
- [ ] **S2-10.2 — Docs sweep.** Dep: S2-10.1.
      Do: `SYNTAX.md` §2 terminology, §5.2–§5.4, §6.1/§6.5/§6.6, §7.x/§8/§9.x,
      §10.5, §13, §14, §21 (D-15); every touched crate README links its §; no
      grammar restated outside the spec. §14 rows record removal only — apply
      OBJ-11 to the pre-existing rows if it is answered "prune".
      Exit: `grep` for old terms (`canonical extras`, `positional names`,
      `recommended`, `backtick_key`) returns only §14's normative lines.
- [ ] **S2-10.3 — Clean-break audit (D-16).** Dep: S2-10.2.
      Do: prove there is no compatibility surface left — `grep -rn "serde(alias)"
      crates apps`, `grep -rn "#\[deprecated\]"`, `grep -rn "TODO: remove"`,
      `grep -rni "backward compat\|deprecat\|migration" docs/ crates/ apps/`;
      every hit is either an error path or §14's normative retirement list.
      Confirm `MIGRATION.md` is gone and no doc links to it.
      Exit: the greps are clean; a deliberately old config file fails to load with
      one `Severity::Error` in a test.
- [ ] **S2-10.4 — Full gate + sandboxes.** Dep: S2-10.3.
      Do: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets`,
      `cargo test --workspace`; re-render every `sandbox/*`, review the diff;
      update `STATUS.md` and the memory bank (`progress.md`, `activeContext.md`).
      Exit: all three commands green, sandbox diffs explained, STATUS short.
- [ ] **S2-10.5 — Schedule the follow-ups.** Dep: S2-10.4.
      Do: open issues for the clean top-to-bottom `SYNTAX.md` rewrite (D-15's
      rejected option) and for `plugin-wiki`'s overhaul. Start neither.
      Exit: two issue files + `docs/INDEX.md` rows.

## Sequencing notes

- **Parallelisable:** S2-4.2 / S2-4.3 / S2-4.8 (no shared files beyond
  `crates/extra`), S2-7.1 → S2-7.3 (docs only), S2-10.1 (editor only).
- **Strictly serial:** S2-2.5 → S2-2.6 → S2-2.7 (projection), S2-3.1 → S2-3.2 →
  S2-3.4 → S2-3.5 (specs), S2-5.1 → S2-5.2 → S2-5.3 → S2-5.4 (decorator).
- **Never reorder:** Phase 2 before 3 (specs need the projection); Phase 4
  before 7 (the inventory records final field names); Phase 8 before 9
  (numbering extraction needs the new selector).
- **Landing a phase** means: all its tasks checked, gate commands green, one
  `STATUS.md` line removed, and the phase's ADR flipped from `Proposed` to
  `Accepted` once it is complete.
