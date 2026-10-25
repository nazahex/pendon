# Workflow — Type Selectors, Exports, Numbering, Extraction

**Status: plan of record, Phase 0 signed off (§4) — Phase 1 in flight.** This document
sequences four interdependent proposals into an executable order. It restates
**no grammar**: the normative text stays in
[`spec/SYNTAX.md`](../spec/SYNTAX.md), and the day-to-day task list lives in
[`TASKS.md`](TASKS.md).

| Input proposal                                                        | Turns into                 |
| --------------------------------------------------------------------- | -------------------------- |
| [`issues/type-upgrade.md`](../issues/type-upgrade.md)                 | Phase 1 — `pendon-types`   |
| [`issues/export-const-upgrade.md`](../issues/export-const-upgrade.md) | Phase 2 — generic exports  |
| [`issues/plugin-number.md`](../issues/plugin-number.md)               | Phase 3 — `plugin-number`  |
| [`issues/plugin-extract.md`](../issues/plugin-extract.md)             | Phase 4 — `plugin-extract` |

Supporting artefacts produced while executing this workflow:

- [`type-resolution.md`](type-resolution.md) — shared resolver design (draft).
- `spec/EXTRAS-INVENTORY.md` — Q.1: every element's extractable fields. Does
  not exist yet; T0.3 creates the skeleton, T4.1 fills it.

---

## 1. What each phase delivers

| Phase | Deliverable                                                                  | Breaks                                                  | Crates                                                                        |
| ----- | ---------------------------------------------------------------------------- | ------------------------------------------------------- | ----------------------------------------------------------------------------- |
| 1     | `pendon-types`: selectors `?` `_` `*`, specificity, override, occurrence set | nothing (additive selectors)                            | `crates/types` (new), `apps/cli`, `renderer-solid`                            |
| 2     | One ordered `export const` list, producers supply `(name, Value)`            | internal hint fields (no public syntax break)           | `renderer-solid`, `apps/cli`, `plugin-bind`                                   |
| 3     | `plugin-number`: `.number` blocks, `%%` extras, `attrs.number` array         | `plugin-heading` `auto_number`/`number_style` (removed) | `crates/plugin-number` (new), `plugin-heading`, `extra`, `apps/cli`           |
| 4     | `plugin-extract`: per-element `.extract` blocks, files + export channel      | `plugin-extract-heading` (deleted)                      | `crates/plugin-extract` (new), `plugin-extract-heading` (deleted), `apps/cli` |

Each phase ends with its own ADR and its own golden fixtures; a phase is not
"done" while its fixtures are `#[ignore]`d (`spec/golden/README.md`).

---

## 2. Order and why

```text
┌─────────────────┐
│ 1. Type Upgrade │──────────────┬──────────────┐
└─────────────────┘              │              │
         │                       ▼              ▼
         │              ┌──────────────────┐  ┌──────────────────┐
         │              │ 3. plugin-number │  │ 4. plugin-extract│
         │              └──────────────────┘  └──────────────────┘
         │                       │  attrs.number  ▲
         │                       └────────────────┘
         │                              ▲
┌──────────────────────────┐             │
│ 2. Export Const Upgrade  │─────────────┘
└──────────────────────────┘  export channel
```

1. **Type first.** `plugin-number` and `plugin-extract` both accept `type = …`
   with selectors, and `extract` _requires_ it — its `type` decides which blocks
   see which elements (an omitted `type` means `_`, i.e. the leftovers). Built
   before Phase 1, one of them would grow a private copy of the matching rules
   and drift.
2. **Exports second, not last.** Phase 4's `export = "citationsC"` channel and
   Phase 2's `[[task.bind.inject]]` share one writer (`export const`). Doing it
   before `plugin-number` keeps it a small, self-contained change, and gives
   Phase 4 a dependency-free channel.
3. **Number before extract.** `plugin-number` is a _producer_: extraction must be
   able to read `attrs.number` (§Integration with plugin-number). Building the
   producer first means Phase 4 tests against a real structured attribute
   instead of a stub.
4. **Nothing else moves.** `plugin-cite` numbering stays as it is (explicitly out
   of scope for Phase 3), so no phase blocks on a cite rework.

---

## 3. Invariants that hold across all four phases

1. **Attributes, not text.** A plugin writes only `Event::Attribute` /
   `AttributeFlag` values (`spec/SYNTAX.md` §6). `attrs.number` is a structured
   value and travels the `U+E001` transport of
   [ADR-0005](../decisions/0005-structured-extras-values.md). Nothing writes
   numbers or extracted data into the rendered text.
2. **Layer model.** Shared `@@`-head parsing stays in `crates/extra`; a plugin
   owns only its own syntax (`CONTRIBUTORS.md`). `%%` is an extras item, so its
   _grammar_ lives in `extra`, not in `plugin-number`.
3. **Alpha = no shims, but loud removals.** Removed keys render nothing and warn
   nothing (proposals are explicit: "without nostalgia"), **except** that the
   config loader still _parses_ the removed key into a `toml::Value` field and
   fails with a migration message — the §14 pattern already used for
   `[task.image.custom_node]` and `cite.prefix`.
4. **Config typos never pass silently.** Any new section must be added to
   `ConfigTask` (otherwise it lands in `unknown` and only warns, §13). Every new
   key gets a §13 test that `unknown` stays empty (`config.rs`'s
   `new_keys_are_not_reported_as_unknown`).
5. **Determinism.** Iterate `BTreeMap`/`BTreeSet`, never `HashMap`, wherever the
   order reaches the output (exports, imports, numbering).
6. **Goldens are the record.** Re-freeze with `pendon run -F`, review the diff,
   then update `spec/golden/README.md`'s fixture table; never hand-edit a `.jsx`.
7. **One chain, two entry points.** New stages go through
   `apps/cli/src/chain.rs` (`run`) or `apps/cli/src/plugins.rs`
   (`prelude`/stateless). Sub-documents (`Document::Payload`) must skip
   document-level exports — today hard-coded for `extract-heading`
   (`chain.rs:232`), generic after Phase 4.

---

## 4. Phase 0 — the decision gate (blocks everything else)

Each decision has a recommendation and a "cost to flip" so the team can accept
them in one sitting. **No phase-1 code starts before D-1…D-5 are marked
accepted**; D-6…D-10 may be settled inside their own phase.

| #    | Decision                                                      | Recommendation                                                                                                                                                                                                                                                                                | Cost to flip                                                  | Needed by |
| ---- | ------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------- | --------- |
| D-1  | Where the resolver lives                                      | new crate `crates/types` (`pendon-types`), `no_std`-free, depends on nothing but `std`                                                                                                                                                                                                        | low (one `use`)                                               | T1.1      |
| D-2  | Scope of "defined" for `_`                                    | one element section per task (see `type-resolution.md` §2.2)                                                                                                                                                                                                                                  | low (group construction)                                      | T1.5      |
| D-3  | When components are resolved (they need the document's types) | **render time (D-3a)**: plugins stop choosing components, they emit the built-in node + its `type` attribute, and the renderer resolves each layer against the document's type set. Fallback **D-3b**: the CLI pre-scans markers and passes a type set into `ComponentSet::select`. See §5.4. | medium — D-3a touches every element plugin once, mechanically | T1.8      |
| D-4  | Duplicate markers within one layer                            | exact literal duplicates stay a hard error; `?`/`_`/`*` duplicates are allowed and ordered (later wins)                                                                                                                                                                                       | low                                                           | T1.7      |
| D-5  | Who owns `export const`                                       | `renderer-solid` owns _emission_ (one ordered list on the hints); producers only supply `(name, Value)`                                                                                                                                                                                       | medium                                                        | T2.1      |
| D-6  | Two exports with the same name                                | last registration wins entirely (value **and** position); one line per name                                                                                                                                                                                                                   | low                                                           | T2.3      |
| D-7  | `recommended` preset contents                                 | decided per element in `spec/EXTRAS-INVENTORY.md`, starting with `cite`; nothing recommended until reviewed                                                                                                                                                                                   | low                                                           | T4.1      |
| D-8  | `NUMBER_SYSTEMS` breadth                                      | tier 1 ships: `decimal`, `lower/upper-roman`, `lower/upper-alpha` (+`latin` aliases), `lower-greek`; the rest is an enum-only follow-up                                                                                                                                                       | low                                                           | T3.3      |
| D-9  | Internal key for `%%` on the event stream                     | `__number` attribute (double-underscore = internal, like `__plugin_kind`), stripped by every renderer                                                                                                                                                                                         | low                                                           | T3.1      |
| D-10 | Collision with today's extras `number:` prop on headings      | `number` becomes reserved by `plugin-number`; an extras `number:` item is a diagnostic when `number` runs                                                                                                                                                                                     | low                                                           | T3.6      |

**Signed off (2026-10-25).** D-1…D-5 are accepted as recommended; none is
flipped.

- **D-1** — new crate `pendon-types` (`crates/types`), `std` only, no
  dependency on `pendon-core` or `crates/extra`.
- **D-2** — a definition group is one element section of one task
  (`type-resolution.md` §2.2).
- **D-3 → D-3a** — resolution happens at render time. Plugins keep emitting
  their built-in node plus the `type` attribute; `renderer-solid` resolves each
  layer against the document's type set. **D-3b is not used**, so no code path
  pre-scans heads outside `crates/extra`.
- **D-4** — exact literal duplicates stay a hard `ConfigError`; `?`/`_`/`*`
  duplicates are allowed and ordered (later wins).
- **D-5** — `renderer-solid` owns `export const` emission through one ordered
  list (`DataExport`); producers register `(name, Value)` only.

D-6…D-10 stay open until their own phase, as the paragraph below already says.

---

## 5. Phase 1 — Type Upgrade

**Goal.** One resolver, three consumers, zero grammar change.

### 5.1 Deliverables

1. `crates/types` with `Selector`, `TypeSet`, `Rule`, `Group`, `winner`,
   `matching`, per [`type-resolution.md`](type-resolution.md).
2. `apps/cli/src/components.rs` parses `type = …` into selectors (today it
   splits strings) and keeps the exact-duplicate check for literals.
3. Renderer hints carry selectors; `renderer-solid` resolves once per document
   and selects components (and only the winning components' imports) from it.
4. `spec/SYNTAX.md` §11 documents the selector set; §5/§15 cross-reference it;
   new ADR `docs/decisions/0008-type-selectors.md`.
5. Golden `28-type-selectors` reproducing R.1–R.4 end to end.

### 5.2 Files to touch (read before editing)

| File                                                                                                  | Change                                                                                     |
| ----------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| `crates/types/{Cargo.toml,src/lib.rs}`                                                                | new crate + unit tests (§11 of the design doc)                                             |
| `Cargo.toml`                                                                                          | add `crates/types` to `members`                                                            |
| `apps/cli/src/components.rs`                                                                          | `ComponentEntry.types: Vec<String>` → `selectors: Vec<Selector>`; duplicate check          |
| `apps/cli/src/chain.rs`, `process.rs`                                                                 | pass selectors into hints (no behaviour change yet)                                        |
| `crates/renderer-solid/src/imports.rs`                                                                | `ComponentTemplate` gains `selectors`; keep `node_name` for literals                       |
| `crates/renderer-solid/src/template.rs`                                                               | `select_template` resolves against the document type set                                   |
| `crates/renderer-solid/src/{node,lib}.rs`                                                             | compute the type set once, thread it into templates + imports                              |
| `crates/renderer-solid/src/components.rs`                                                             | `TypedComponent.selectors`; `select` takes a resolution (D-3b only)                        |
| `crates/plugin-{anchor,blockquote,cite,directive,heading,image,list,marker,section,table}/src/lib.rs` | delete the `custom.select(marker)` call and the `Custom(name)` swap (D-3a)                 |
| `crates/plugin-custom/src/specs.rs`                                                                   | `type` parsing reuses `Selector` (custom `toml:` specs accept plain literals only for now) |

### 5.3 Acceptance criteria

- [ ] A document with `TypeA…TypeD` and config R.1–R.4 renders `TypeA` with
      `HeadingTypeA`, `TypeB`/`TypeC` with `HeadingFallback`, untyped with
      `HeadingUntyped`, and `HeadingDefault`'s import never appears.
- [ ] `type` omitted still behaves exactly as today (untyped **and** unknown
      markers reach the layer default) — the four existing golden suites stay
      byte-identical.
- [ ] `type = ["TypeA","_"]` and `type = "*"` behave as `type-upgrade.md` D.4/D.5.
- [ ] A duplicate literal `type = ["TypeA","TypeA"]` in one layer still errors
      with the current message.
- [ ] `cargo test -p pendon-types -p pendon-renderer-solid -p pendon --test syntax_spec`
      green; no fixture `#[ignore]`d.
- [ ] `cargo test -p pendon --test custom_spec` still green (custom components
      are the main regression risk).

### 5.4 Where resolution happens (D-3)

Today an element plugin calls `options.custom.select(marker)` while it builds its
nodes and, on a hit, swaps `NodeKind::Heading`/… for `NodeKind::Custom(name)`
(`plugin-heading/src/lib.rs:360`, and one call site in each of anchor,
blockquote, cite, directive, image, list, marker, section, table). That is only
correct for literal selectors: `_` and `*` need the _document's_ occurring types,
which a plugin never sees.

- **D-3a (recommended)** — the plugin stops choosing. It emits the built-in
  element with its `type` attribute (already emitted today), and
  `renderer_solid::template::select_template` resolves the layer's rules against
  the document type set (`pendon-types::Group` built from the layer's
  selectors). Imports are gated by the _winning_ rule, so a losing component's
  `import` line never appears. Consequences: ten small plugin deletions, one
  resolver call per document per layer, and `keys_for`/`keys_for_opt`
  (positional keys) becomes literal-only — `_`/`*` contribute no positional
  keys, documented in §11.
- **D-3b (fallback)** — the CLI scans the pre-chain text for markers and passes a
  `TypeSet` into `ComponentSet::select`. Cheaper (call sites only), but it
  re-parses extras heads outside `crates/extra` — exactly the layering rule
  `CONTRIBUTORS.md` forbids.

Pick one in T0.1; do not mix (mixed resolution is how "why did TypeB get the
wrong component" bugs are born).

---

## 6. Phase 2 — Export Const Upgrade

**Goal.** One `export const` writer; producers register `(name, Value)`; `format
!= solid` drops them.

Today's shape: `SolidRenderHints` carries four ad-hoc fields
(`citations_export`, `citations_export_name`, `references_export`,
`references_export_name`) and `render_solid_with_hints` hard-codes the emission
order (frontmatter → headings → references → citations) through
`metadata::extract_frontmatter`/`extract_headings` and `push_data_export`
(`crates/renderer-solid/src/lib.rs:19–78`).

### 6.1 Deliverables

1. `SolidRenderHints.exports: Vec<DataExport>` with
   `DataExport { name: String, value: Value, source: ExportSource }`
   (`source` = `Automatic` | `Plugin(&str)`, for reporting only).
2. `renderer-solid` registers `frontmatter` and `headings` **itself**, first,
   from the AST — they stay automatic but become ordinary registrations, so
   `export = "frontmatter"` (P.2) can override them by registering later.
3. `plugin-cite` pushes into `exports` instead of filling hint fields; the two
   `*_export_name` options become sugar that pushes `(name, value)` under the
   existing config keys (kept — the goldens use them).
4. `plugin-bind`: `[[task.bind.inject]]` with `source` (project-root-relative with
   the task's `[foo]`/`[bar]` captures substituted) and an optional `export`
   (default: the file stem); conversion reuses `parse_data_file`, i.e. the same
   converter as a `{{{lang[var] … }}}` block.
5. Name validation: `[A-Za-z_$][A-Za-z0-9_$]*`, not a reserved word; a bad
   `export` is a task error before rendering.
6. `spec/SYNTAX.md` (nearest export paragraph) + new ADR
   `docs/decisions/0009-generic-data-exports.md`; `crates/plugin-bind/README.md`
   and `docs/guides/cli.md` for the new keys.

### 6.2 Non-goals

- No merge of duplicate names (D-6: last wins, one line).
- No sub-document exports: a `Payload` document contributes none (`chain.rs`).
- No change to the `citations`/`references` payload shapes.

### 6.3 Acceptance criteria

- [x] `25-extras-nested`, `26-cite-scope`, `27-bind-mdp` stay byte-identical —
      the refactor is output-neutral. (`syntax_spec`: 23/23 fixtures.)
- [x] `[[task.bind.inject]]` with a `.yaml` and a `.toml` source emits
      `export const fooData = …` / `export const settings = …` before
      `export default function` (`apps/cli/tests/bind_inject_spec.rs`).
- [x] Two producers registering the same name emit one line, last value wins
      (`renderer-solid::tests::the_last_registration_of_a_name_wins`).
- [ ] `export = "frontmatter"` overrides the automatic `frontmatter` export
      (P.2's rule, tested in Phase 4). The mechanism is the "last wins" test
      above; the end-to-end config case belongs to Phase 4.
- [x] An invalid identifier is rejected naming the key and the value
      (`bind.inject 'data-x' is not a valid JavaScript identifier`).
- [x] An edited inject source invalidates the cache (it is a `deps` entry).

---

## 7. Phase 3 — plugin-number

**Goal.** All numbering in one plugin; `attrs.number` is an array of strings.

### 7.1 Grammar first (`crates/extra`)

`%%numberType` is an extras item (§5), so its _parse_ belongs in `extra`, not in
`plugin-number`:

- `%`-runs of 2…7 followed by a type name; canonical position = **last** in the
  head (after props, flags, `--cssvars`); attached directly to the head or to a
  preceding extras body, else literal.
- Parsed into `ParsedExtras::number = Some(NumberMark { depth, number_type })`
  and re-emitted as the internal attribute `__number` (§4 D-9), value
  `{"depth":1,"type":"numberB"}` through the ADR-0005 transport. `depth` is
  already normalized (`%` count − 1, so `%%` → 1 … `%%%%%%%` → 6); `plugin-number`
  never re-derives it from the source text.
- `%` alone, `%%` with no name, and 8+ `%` are **literal text** (the policy every
  malformed head follows, golden `02-extras-literal`); no diagnostic.
- Construct plugins already forward unknown extras, so `__number` reaches
  `plugin-number` untouched; renderers strip `__*` attributes (existing rule for
  `__plugin_kind`).

### 7.2 Plugin internals

- Two group families: per-element `.number` (`<element>:auto`) and task-level
  `[[task.number.custom]]` (`numberType:<name>`), both resolved by `pendon-types`.
- Two passes over the document: (1) collect member depths → group minimum;
  (2) assign numbers. Required by "the smallest depth is determined across the
  whole file" (a later `#` renumbers earlier sections).
- Counter state per winning rule: `levels: Vec<u64>`, increment at normalized
  depth, reset deeper levels, ghost-fill parents with their style's first value.
- `attrs.number` = `["1","2","a"]`, always exactly `normalized_depth` entries.
- `%%` members leave their `.number` group and are numbered from the task-level
  group with its **absolute** depth (the normalized `depth` carried in
  `__number`) and ghosts always counted.
- Element kinds without a natural depth (image, table, marker, …) are depth 1
  unless `%%` says otherwise.
- Styles: string → all depths; array → depth _n_ uses entry _n−1_, deeper depths
  and out-of-range numbers fall back to `decimal`.

### 7.3 Removals and wiring

- `plugin-heading`: delete the counter, `NumberStyle`, and the
  `auto_number`/`number_style` behaviour; keep both keys in `HeadingTaskConfig`
  as `Option<toml::Value>` so stale configs fail with the §14 migration message.
  Drop the emitted `number` attribute; update `crates/plugin-heading/README.md`.
- `apps/cli`: add `crates/plugin-number` to `members` and
  `apps/cli/Cargo.toml`; `NumberTaskConfig { custom: Option<Vec<NumberRule>> }`
  plus `number: Option<Vec<NumberRule>>` on every element config;
  `build_number_options`; a `"number"` arm in `chain.rs`; a pre-flight diagnostic
  when `number` precedes a producer plugin or follows `markdown`.
- Renderers: `attrs.number` stays structured — Solid prints
  `number={["1","2"]}`, HTML prints the JSON text (golden 25's array rule),
  AST/JSON keep the array.

### 7.4 Acceptance criteria

- [x] The P.1 illustration renders 1, 1.1, 1.2, 1.2.a, 1.2.a.1.1, nothing for
      `HeadingC`/untyped, then 1.3.
- [x] Both "initial depth does not exist" cases match the proposal, including the
      minimum appearing in the middle.
- [x] The P.2 illustration renders `I.a`, `II.`, `II.a`, `II.b`, `II.b.1`,
      `II.b.1.1.1`, `II.c`, `III.`, `A.`, `A.A.A.`, `A.B.`, `B.`, `B.A.A.A.`,
      `A.a.I.i.1.1.`, `IV`, `IV.a.1`, `A`, `A.a`, `B`.
- [x] `auto_number`/`number_style` produce the §14 migration error, not silence.
- [x] Golden `06-heading` re-frozen without numbers (fixture table updated); new
      goldens `30-number-heading`, `31-number-custom`.

---

## 8. Phase 4 — plugin-extract

**Goal.** Every extras-capable element can hand its fields to a file and/or to an
`export const`; `plugin-extract-heading` disappears.

### 8.1 Field inventory first (Q.1)

`spec/EXTRAS-INVENTORY.md` is filled in **before** the plugin's code is written:
one row per element, one column per extractable field, with a `recommended`
mark. Sources of truth for a row: the element's attribute list in `SYNTAX.md`,
its plugin's `emit` block, and its golden `.jsx`. A field with no defined
"only"-name does not ship.

### 8.2 Plugin shape

- Plugin name is `extract`; config is `[[task.<element>.extract]]` per element
  (`cite`, `heading`, `image`, …), repeatable, order-sensitive.
- Keys per block: `type` (selector list, defaults to `_`), `output`
  (string or array), `format` (string or array, **required**), `only`, `omit`,
  `recommended`, `export` (string only).
- **Type scoping reuses Phase 1**: `type` is a `pendon-types` selector list and is
  resolved with `matching` against the document's occurring types, so `["CiteA",
  "CiteB"]` and an omitted `type` (`_`) are two disjoint blocks over the same
  element kind — this is why Phase 4 comes after Phase 1.
- Field selection: start from _all_ fields, `recommended` restricts to the preset,
  `only` intersects (a custom attribute absent from the preset is ignored when
  `recommended` is on), `omit` subtracts last — **omit always wins**.
- A field the element does not carry is dropped, never emitted empty — even when
  `only` names it.
- Output identity: one destination path = one array; several blocks resolving to
  the same path overwrite (last wins). `output` and `format` arrays must have the
  same length or it is a task error; the extension never decides the format.
- `export` registers through Phase 2's list and is dropped when the task format is
  not `solid`.
- A block that matches no element is skipped silently.
- Destination paths are input-relative and `[segment]`-dynamic like `output`
  today; a file is written only after a successful render.

### 8.3 Removal of `plugin-extract-heading`

Today that crate has **no config and no file or export output**: it injects a
`Custom("Headings")` node with `attrs.data` (a _nested_ outline). The `headings`
export itself comes from `renderer-solid`'s `metadata::extract_headings`. So the
removal is three moves:

1. Delete the crate and its `"extract-heading"` arms in `apps/cli/src/chain.rs`
   (stateful loop + `process_stateless_plugin`), its entry in
   `apps/cli/Cargo.toml` and the workspace `Cargo.toml`, its
   `crates/plugin-extract-heading/README.md`, and the `extract-heading` plugin
   name in the four sandbox configs (`sandbox/{heading,ultimate,unified,universal}`).
2. Make the outline an ordinary `[[task.heading.extract]]` with `to` and/or
   `export`, whose value is a **flat array** of `{ level, id, text, … }` —
   nesting disappears (a consumer that wants a tree builds it; the automatic
   `export const headings` keeps its current nested shape so goldens stay
   stable).
3. Replace the hard-coded `if fragment && name == "extract-heading" { continue }`
   (`chain.rs:232`) with the general rule: **only a `Page` document writes
   document-level outputs** (files + exports); a `Payload` runs every plugin for
   its in-document effects only.

Note there is **no golden for `extract-heading` today** (`spec/golden/README.md`
has no 22-extract row) — Phase 4 closes that gap with `32-extract-heading`
built from `sandbox/heading`'s config, so the deletion is provably
behaviour-preserving for the outline data.

### 8.4 Cache and files

- Side files are outputs, not logs: extend `CacheEntry` (`cache.rs:48`) with
  `side_outputs: Vec<OutputInfo { path, hash }>`, bump `CACHE_VERSION`
  (`cache.rs:28`), extend `create_entry` (`cache.rs:222`), and make `is_fresh`
  (`cache.rs:137`) also require every side file to still exist with its recorded
  hash.
- Writing: the only atomic write today is the private tmp+rename inside the cache
  record writer (`cache.rs:130`). Lift it to a shared `write_atomically(path,
  text)` and use it for side files; leave the main output's `fs::write`
  (`process.rs:249`) alone unless the lift is free. A failed side write is a task
  error, never a half-written JSON.

### 8.5 Acceptance criteria

- [ ] `[[task.heading.extract]]` with `output = ["out/headings.json"],
      format = ["json"]` and `export = "citationsC"` produces the file **and** the
      export.
- [ ] P.1.a/P.1.b/P.1.c reproduce: two blocks with disjoint `type` scopes write
      disjoint data, and the third block's `export = "citationsC"` wins over the
      second's while both `_`-scoped blocks still write their own files.
- [ ] P.3 reproduces exactly: `recommended` + `only` + `omit` leaves one field
      (`citation_number`).
- [ ] An `extract` on a `Payload`/`mdp` sub-document writes neither file nor
      export.
- [ ] Two blocks resolving to the same `output` leave only the later data; a
      `output`/`format` length mismatch is a task error.
- [ ] A field list including an absent attribute emits no empty value.
- [ ] Deleting a side file makes the next run rewrite it (cache freshness).
- [ ] `extract-heading` is gone as a plugin name and warns like any unknown one.
- [ ] `cargo test -p pendon --test syntax_spec` green with new `32-extract-heading`,
      `33-extract-scopes` goldens.

### 8.6 Scope of elements

In scope: `anchor`, `blockquote`, `cite`, `directive`, `heading`, `image`,
`marker`, `list`, `section`, `table`, `wiki` (wikilink only), `codeblock`
(vanilla), `latex`, `bind`, `micromatter`. Out of scope, and left out of
`EXTRAS-INVENTORY.md`: `quiz`, `codeblock-syntect`, `vicado`, `dialog`, and
`plugin-custom`'s `toml:` specs.

### 8.7 Depth field

Extraction is a **flat** array; elements with a notion of depth carry it (a
`level`-like field), so a consumer builds its own tree. `plugin-number`'s
`attrs.number` array is emitted as-is, which is why Phase 3 lands first.

---

## 9. Risk register (cross-phase)

| Risk                                                                 | Phase | Mitigation                                                                                       |
| -------------------------------------------------------------------- | ----- | ------------------------------------------------------------------------------------------------ |
| `renderer-solid` resolution changes break the four component goldens | 1     | acceptance requires byte-identical output for existing fixtures before any selector golden lands |
| Custom `toml:` components silently lose `type` semantics             | 1     | `cargo test -p pendon --test custom_spec` is a gate; literals keep working unchanged             |
| Two-pass numbering makes the plugin order-sensitive with markdown    | 3     | pre-flight diagnostic for a wrong `number` position; goldens pin the recommended order           |
| `__number` leaks into output                                         | 3     | every renderer already strips `__*`; a golden asserts the absence of `__` keys                   |
| Deleting `extract-heading` breaks the four sandboxes' TOCs           | 4     | sandbox configs updated in the same PR; the automatic `headings` export is unchanged             |
| Side files confuse the cache                                         | 4     | `side_outputs` + `CACHE_VERSION` bump + freshness test                                           |
| Scope creep into `plugin-cite` numbering                             | 3     | explicitly out of scope; any cite change needs its own ADR                                       |

---

## 10. Verification cadence

Per task: `cargo test -p <crate>` (unit tests written with the code).
Per phase, in this order:

1. `cargo test -p pendon --test syntax_spec` — every golden, none `#[ignore]`d.
2. `cargo test --workspace` — catch `apps/cli` and `renderer-*` fallout.
3. `cargo clippy --workspace --all-targets` clean.
4. Render one sandbox for a human check:
   `cd sandbox/heading && cargo run --bin pendon -- run` (or the relevant one)
   and diff `out/` by eye before re-freezing.
5. Freeze the new/changed goldens **last**, after the review above.

---

## 11. Documentation obligations per phase

| Phase | `spec/SYNTAX.md`                               | ADR                            | Crate README                      | Other                                     |
| ----- | ---------------------------------------------- | ------------------------------ | --------------------------------- | ----------------------------------------- |
| 1     | §11 selectors, §5/§15 cross-refs               | `0008-type-selectors.md`       | —                                 | `crates/types/README.md`                  |
| 2     | export paragraph                               | `0009-generic-data-exports.md` | `plugin-bind` (`inject`)          | `docs/guides/cli.md`                      |
| 3     | §5 `%%` item, `.number` blocks                 | `0010-plugin-number.md`        | `plugin-number`, `plugin-heading` | `STATUS.md` line removal                  |
| 4     | per-element `.extract`, `extract-heading` gone | `0011-plugin-extract.md`       | `plugin-extract`, `plugin-bind`   | `spec/EXTRAS-INVENTORY.md`, golden README |

Every phase also updates `STATUS.md` (add while in progress, delete when done)
and records the migration note for removed keys in the ADR, not in `SYNTAX.md`.

---

## 12. Out of scope here

- `plugin-cite`'s own numbering and reference ordering.
- `quiz`, `vicado`, `dialog`, `codeblock-syntect`, `plugin-custom` extraction.
- A per-task `[task.extract]` global default block (only per-element blocks ship).
- Any change to the parser's event model: everything rides existing
  `Event::Attribute` / `Event::Diagnostic` channels.
