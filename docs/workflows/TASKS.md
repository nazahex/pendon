# TASKS — numbering / extract / type / export

Atomic, ordered checklist for
[`numbering-extract-type-export.md`](numbering-extract-type-export.md).
**Work top to bottom.** A box is only ticked when its `verify` line passes and
its docs are updated; the phase's _Freeze_ task is always last.

Legend — **C** = crates touched, **R** = read before editing, **Do** = change,
**Done** = completion test.

---

## Phase 0 — decisions and scaffolding (blocks all code)

### [x] T0.1 — Accept decisions D-1…D-5

- **R**: §4 of the workflow doc; `docs/decisions/README.md`.
- **Do**: mark each of D-1…D-5 accepted (or flip it) in §4 and record the verdict
  in the corresponding ADR (T0.2). Do not start T1.1 before this.
- **Done**: §4 has no "awaiting" note for D-1…D-5.

### [x] T0.2 — ADR skeletons 0008–0011

- **C**: `docs/decisions/`.
- **Do**: create `0008-type-selectors.md`, `0009-generic-data-exports.md`,
  `0010-plugin-number.md`, `0011-plugin-extract.md` — Status/Context/Decision/
  Consequences/Migration sections only, no grammar restatement (link `SYNTAX.md`
  §N instead); add the four rows to `README.md`'s index.
- **Done**: `docs/decisions/README.md` lists 0008–0011, each file has a Status line.

### [x] T0.3 — Inventory skeleton

- **C**: `docs/spec/`.
- **Do**: create `EXTRAS-INVENTORY.md` per the workflow doc §8.1: one row per
  in-scope element (§8.6), columns `field | source | recommended | notes`, and a
  "how to fill a row" note pointing at `SYNTAX.md` §N, the plugin's `emit` block
  and the element's golden `.jsx`.
- **Done**: every in-scope element has a row with `source` filled and
  `recommended` marked `TODO` (Phase 4 fills it).

---

## Phase 1 — Type Upgrade

### [x] T1.1 — Scaffold `crates/types`

- **C**: new `crates/types`, root `Cargo.toml`.
- **R**: `docs/workflows/type-resolution.md` §1–§3; an existing small crate
  (`crates/core/Cargo.toml`) for the manifest/edition conventions.
- **Do**: `Cargo.toml` (`name = "pendon-types"`, workspace edition, `serde`
  behind the same feature style as `core` if it derives anything), `src/lib.rs`
  with module layout and re-exports, `README.md` stub linking
  `type-resolution.md`.
- **Verify**: `cargo test -p pendon-types` (empty suite passes), `cargo check --workspace`.
- **Done**: crate is a workspace member and nothing depends on it yet.

### [x] T1.2 — `Selector` + `parse`/`Display`

- **C**: `pendon-types`.
- **Do**: `Selector::{Exact(String), Question, Underscore, Star}`; a parse that
  maps a single non-empty token and rejects everything else; `Display` round-trips
  the source token (`?`, `_`, `*`, literal). Unit tests: round-trip, whitespace
  trimming, empty token rejected, `??`/`**` rejected (not special).
- **Verify**: `cargo test -p pendon-types selector`.
- **Done**: parsing a config `type` array never panics and never silently maps a
  typo to a wildcard.

### [x] T1.3 — `winner` + specificity

- **C**: `pendon-types`.
- **R**: `type-resolution.md` §4–§5.
- **Do**: `Rule { selectors, payload, target }`; `winner(rules, types) ->
  Option<&Rule>` scoring each rule (`?`/literal = 2, `_`/`*` = 1, no match = 0),
  highest score wins, ties broken by later registration. Tests: exact beats `_`,
  `_` beats nothing, later `_` beats earlier `_`, no match → `None`.
- **Verify**: `cargo test -p pendon-types winner`.
- **Done**: a rule with no matching selector never wins.

### [x] T1.4 — `matching` (additive, deduped)

- **C**: `pendon-types`.
- **Do**: `matching(rules, types) -> impl Iterator<Item = (&Rule, &str)>` yielding
  one entry per (rule, matched type), deduped per `(target, type)`, in
  registration order. Tests: two `_` rules with different targets both yielded;
  two `_` rules with the same target and the same type yielded once.
- **Verify**: `cargo test -p pendon-types matching`.
- **Done**: additive consumers (numbering, extraction) get a deterministic list.

### [x] T1.5 — `Group` with `defined` computed up front

- **C**: `pendon-types`.
- **R**: `type-resolution.md` §2.2, §6.
- **Do**: `Group::new(rules)` computes `defined: BTreeSet<String>` from every
  literal selector in the group, before any resolution; `_` = occurring −
  defined, `*` = all occurring; `_`/`*` add nothing to `defined`. Tests: a rule
  registered later still contributes to `defined` for an earlier resolution;
  empty document → no matches.
- **Verify**: `cargo test -p pendon-types group`.
- **Done**: resolution is order-independent w.r.t. registration.

### [x] T1.6 — `TypeSet` for a document

- **C**: `pendon-types`.
- **Do**: `TypeSet::from_types(iter)` → sorted, deduped occurrence set, plus the
  untyped marker; `occurring()`, `contains()`; helpers used by `_`/`*`.
- **Verify**: `cargo test -p pendon-types type_set`.
- **Done**: a document with no types yields an empty set, not a wildcard.

### [x] T1.7 — CLI: `type` becomes selectors

- **C**: `apps/cli`.
- **R**: `apps/cli/src/components.rs` (whole file — `ComponentEntry`,
  `ComponentSet::select`, the duplicate check), `apps/cli/src/process.rs`
  (`build_*_options`), §11 rules 1/2/3/6 of `SYNTAX.md`.
- **Do**: `ComponentEntry.types: Vec<String>` → `selectors: Vec<Selector>`;
  `load` parses each entry through `Selector`; `is_default()` stays "no `type`
  key" (an explicit `_`/`*` entry is a _rule_, not the default slot, so two
  entries without `type` remain the hard error); keep the exact-literal duplicate
  error and allow `?`/`_`/`*` duplicates (D-4).
- **Verify**: `cargo test -p pendon components`, plus `cargo test -p pendon --test custom_spec`.
- **Done**: an invalid selector is a `ConfigError` naming the key and the layer.

> Landed. `ComponentEntry.selectors`, `TypedComponent.selectors` and
> `Selector::parse_all` are in place, and the duplicate check is literal-only.
> **A wildcard parses and validates but does not resolve yet** — that is T1.8,
> which is why `_` / `*` must not ship in a user config before it lands. (Both
> halves have since landed: T1.8 resolves per document, T1.10 pins it in the
> `28-type-selectors` golden.)

### [x] T1.8 — Resolve components at render time (D-3a)

- **C**: `renderer-solid`, then the ten element plugins.
- **R**: `crates/renderer-solid/src/template.rs` (`select_template`),
  `node.rs` (the `type` dispatch), `lib.rs` (`render_solid_with_hints`,
  `collect_used_nodes`), `imports.rs` (`template_imports` gating).
- **Do**: `ComponentTemplate` carries `selectors: Vec<Selector>`; build one
  `pendon_types::Group` per layer from the document's occurring types (computed
  once in `render_solid_with_hints` from the AST's `type` attributes);
  `select_template` picks literal → `?` → `_`/`*` (last wins) → default slot;
  gate `template_imports` on the winners. Then delete the
  `options.custom.select(marker)` + `Custom(name)` swap from each element plugin
  (they keep emitting the built-in node and the `type` attribute).
- **Verify**: `cargo test -p pendon-renderer-solid` and
  `cargo test -p pendon --test solid_spec --test nested_component_spec`.
- **Done**: no plugin imports `ComponentSet::select`; type dispatch is one place.

> Landed as `crates/renderer-solid/src/resolve.rs` rather than in
> `template.rs`: `ComponentTemplate` carries `layer` + `selectors`, the plugin
> builds its hints with `layer_templates(layer, &set, …)`, and `Resolution::build`
> groups them into one `pendon_types::Group` per layer, collects the document's
> occurring types (fragments included) and stores the winner per occurring type.
> Plugins tag their built-in nodes with `__layer` and keep the `type` marker;
> `__`-prefixed attributes are stripped by the Solid and HTML renderers before a
> component sees `attrs`. `NodeKind::Custom` survives only where there is no
> layer to resolve: `plugin-custom`'s `toml:` specs, `plugin-quiz` and
> `plugin-vicado` (identity templates) — and **none of those three needs a
> migration**: `quiz`, `vicado` and the wiki infobox are deleted in the next
> workflow (see the scope note in T1.11), so the identity path only has to keep
> working until then and their sandbox output is not a regression signal. Goldens
> `08-table-decl` and `09-table-layers` re-rendered byte-identical, so the
> cutover moved the decision without moving the output.

> If D-3b was chosen instead, replace this task with: add
> `select(marker, &TypeSet)` to both `ComponentSet`s, build the set in
> `chain.rs` from `pendon_extra`'s scan, and update the ten call sites.

### [x] T1.9 — Positional keys stay literal-only

- **C**: `renderer-solid`, `apps/cli`.
- **Do**: `keys_for`/`keys_for_opt` keep matching exact literals only; document
  the rule in the code comment and in `SYNTAX.md` §11 (a `_`/`*` entry
  contributes no `slug`/`title` overrides). Add a unit test that a `_`-only entry
  yields `PositionalKeys::default()`.
- **Verify**: `cargo test -p pendon-renderer-solid keys_for`.
- **Done**: no silent positional change for wildcard entries.

### [x] T1.10 — `28-type-selectors` golden

- **C**: `docs/spec/golden/`, `apps/cli/tests/syntax_spec.rs` (only if the harness
  needs a new entry).
- **R**: `docs/spec/golden/README.md` (fixture format), `docs/guides/testing.md`,
  an existing fixture (`06-heading.*`) for the file trio.
- **Do**: `.mdp` with `TypeA`, `TypeB`, `TypeC`, untyped and one uncovered
  `TypeD`; `.toml` with R.1–R.4 verbatim; freeze the `.jsx` with
  `cargo run --bin pendon -- run` inside the fixture sandbox, then add the README
  row.
- **Verify**: `cargo test -p pendon --test syntax_spec type_selectors`.
- **Done**: the frozen `.jsx` shows `HeadingTypeA`, `HeadingFallback`,
  `HeadingUntyped`, and no import line for `HeadingDefault`.

> Landed as `docs/spec/golden/28-type-selectors.{mdp,toml,jsx}` +
> `golden_28_type_selectors` in `apps/cli/tests/syntax_spec.rs` and a README row.
> The document occurs `{TypeA…TypeD, untyped}`; R.1 `*` loses everywhere, so
> `HeadingDefault`'s import is absent, `TypeA` → `HeadingTypeA`, `TypeB`/`TypeC`/
> `TypeD` → `HeadingFallback`, and both untyped headings → `HeadingUntyped`.

### [x] T1.11 — Docs for Phase 1

- **C**: `docs/`.
- **Do**: `SYNTAX.md` §11 (selectors, specificity, `defined` group-wide,
  positional limitation) + cross-refs in §5/§15; ADR-0008 with the rejected
  alternatives and the `defined`-up-front rule; `crates/types/README.md` finished.
- **Verify**: `grep -rn "TODO" docs/spec/SYNTAX.md` clean for the touched §.
- **Done**: the four numbers (2/1 specificity, last-wins, `_` = remainder,
  `defined` group-wide) appear **once** in `SYNTAX.md`.

> Landed: §11 rule 2 carries specificity + last-wins, rule 3 carries `_` =
> remainder and the up-front `defined` group; the cutover note inside rule 3
> points at `ADR-0008` and OPEN-C3.`

> **Scope note (out of bounds).** `quiz`, `vicado` and the wiki infobox are
> **deleted entirely in the next workflow**. Ignore them here and in Phase 2:
> no `__layer` / `layer_templates` migration, no golden, no sandbox triage, no
> new feature built on the identity path. They keep working only because they
> are untouched; nothing is planned around them. The same note sits in
> `ADR-0008`, `SYNTAX.md` §11, `docs/guides/plugin-authoring.md` and
> `STATUS.md`, so a reader who lands on any one of them sees it.

### [x] T1.12 — Phase 1 gate

- **Do**: `cargo test --workspace`, `cargo clippy --workspace --all-targets`,
  then `cargo test -p pendon --test syntax_spec` with every fixture enabled.
- **Done**: every fixture that configures `[task.*.custom]`
  (`03, 05, 06, 08, 09, 11, 12, 14, 15, 16, 25, 27`) is byte-identical;
  `STATUS.md` gets one line for the in-progress phase.

> Landed. `cargo test --workspace --no-fail-fast` — 75 test binaries, 0 failures;
> `cargo test -p pendon --test syntax_spec` — 23/23 fixtures, none `#[ignore]`d;
> `cargo fmt --all --check` clean; `cargo clippy --workspace --all-targets`
> exits 0 (only the pre-existing style lints: collapsible `if`, derivable
> `Default`, doc lazy continuation). Nothing was re-frozen except
> `28-type-selectors`, which is new: every cutover was byte-identical, which is
> the evidence the move from "plugin chooses" to "renderer resolves" did not
> change output.

---

## Phase 2 — Export Const Upgrade

Workflow: §6. One writer, a list of registrations, `format != solid` drops them.

### [x] T2.1 — `DataExport` type + unified emission

- **C**: `renderer-solid`.
- **R**: `crates/renderer-solid/src/{imports,lib}.rs` (`SolidRenderHints`,
  `render_solid_with_hints`, `push_data_export`), §6.1 of the workflow.
- **Do**: add `DataExport { name: String, value: Value, source: ExportSource }`
  and `ExportSource { Automatic, Plugin(String) }`; replace the four ad-hoc hint
  fields (`citations_export`, `citations_export_name`, `references_export`,
  `references_export_name`) with `exports: Vec<DataExport>`; emit by iterating
  `exports` and keep `push_data_export` as the single writer.
- **Verify**: `cargo test -p pendon-renderer-solid` (the export-order tests at
  `lib.rs:287–325` keep passing after their field names change).
- **Done**: `grep -rn "references_export_name" crates/renderer-solid` is empty.

### [x] T2.2 — `frontmatter`/`headings` become registrations

- **Do**: register both from the AST at the top of `render_solid_with_hints`
  (`source: Automatic`), in that order, before any plugin registration; emission
  becomes "registrations in insertion order, then `export default function`".
- **Verify**: `25-extras-nested`, `26-cite-scope`, `27-bind-mdp` byte-identical.
- **Done**: `metadata::extract_frontmatter`/`extract_headings` are called only
  from the registration block, never from the emission block.

### [x] T2.3 — `plugin-cite` pushes instead of filling fields

- **C**: `plugin-cite`, `apps/cli`.
- **R**: `crates/plugin-cite/src/lib.rs:360–370`,
  `apps/cli/src/plugins.rs:340–352` (`references_export`/`citations_export`
  plumbing).
- **Do**: replace the four assignments with two pushes (`references`, then
  `citations`, `source: Plugin("cite")`); keep the config keys and defaults as
  sugar over the pushes.
- **Verify**: `cargo test -p pendon-plugin-cite` and
  `cargo test -p pendon --test syntax_spec cite`.
- **Done**: cite names no `*_export_name` field.

### [x] T2.4 — One identifier validator

- **R**: `apps/cli/src/plugins.rs:408` (`validate_js_identifier` — note the
  hard-coded `"cite.{key}"` prefix).
- **Do**: move it behind a task-name parameter so messages read
  `<task>.<key> '<value>' is not a valid JavaScript identifier`; reject reserved
  words (`export`, `default`, `import`, `const`, `function`, `class`, `return`,
  `let`, `var`, `new`, `this`, `null`, `true`, `false`, `await`, `static`,
  `yield`, `delete`, `typeof`, `void`, `in`, `of`); reuse from cite, bind and
  Phase 4's `extract.export`.
- **Verify**: `cargo test -p pendon export_identifier`.
- **Done**: exactly one `is not a valid JavaScript identifier` site in `apps/`.

> Landed as `utils::validate_export_identifier(task, key, name)`. The final
> reserved list is the ES2015+ set **plus** `await`, the strict-mode keywords
> (`let`, `static`, `implements`, `interface`, `package`, `private`, `protected`,
> `public`) and the two bindings strict mode forbids (`eval`, `arguments`).
> `of` is contextual, not reserved, so `export = "of"` is accepted — covered by
> `utils::tests::export_identifier_accepts_contextual_keywords`.

### [x] T2.5 — `[[task.bind.inject]]`

- **C**: `apps/cli`, `plugin-bind`.
- **R**: `apps/cli/src/config.rs` (`ConfigTask` — there is **no** bind section
  today), `apps/cli/src/plugins.rs` (`build_*_options` family),
  `crates/plugin-bind/README.md`, SYNTAX §19, ADR-0004.
- **Do**: `ConfigTask.bind: Option<BindTaskConfig>` with
  `inject: Vec<BindInject { source: String, export: Option<String> }>`; `source`
  is input-relative and may contain `[foo]`/`[bar]` segments (the segment name is
  the default export name when `export` is absent); reuse `plugin_bind`'s
  converter for payload → `Value`; push `DataExport { source: Plugin("bind") }`;
  add file sources to the task's cache `deps` exactly as
  `{{{lang[var](path)}}}` does.
- **Verify**: `cargo test -p pendon --test syntax_spec bind`.
- **Done**: `.yaml` and `.toml` sources each emit one `export const …` line above
  `export default function`, and editing a source re-renders the page.

> Landed in `apps/cli/src/plugins.rs::build_bind_injects` +
> `config::{BindTaskConfig, BindInject}` + `plugin_bind::parse_data_file`; the
> hints are appended in `process.rs`'s `solid` branch only. Two deviations from
> the sketch above: `source` resolves against the **project root** (the base
> `input`/`output` use, with the task's captures substituted) rather than the
> document's directory, and the default name is the file **stem** (not the last
> `[segment]`), so `./data/settings.toml` → `export const settings`. Tested by
> `apps/cli/tests/bind_inject_spec.rs` (yaml + toml, dynamic capture, html drop,
> invalid name, cache invalidation).

### [x] T2.6 — Non-Solid formats drop exports

- **R**: `apps/cli/src/process.rs` (confirm hints are only built for the Solid
  path) and the HTML/JSON/AST renderers.
- **Do**: if it already holds, encode it as a test rather than new code: hints
  are never built for `format != solid`, and no other renderer reads `exports`.
  If a path does build them, gate it.
- **Verify**: `cargo test -p pendon --test html_spec`.
- **Done**: the invariant is a test (workflow §3 I-6), not a comment.

### [ ] T2.7 — Docs + Phase 2 gate

- **Do**: SYNTAX nearest export paragraph (name rule, insertion order, last
  wins), `docs/guides/cli.md` (`[[task.bind.inject]]`), `plugin-bind/README.md`
  (link the §N, don't restate), ADR-0009.
- **Gate**: `cargo test --workspace`; `25-extras-nested`, `26-cite-scope`,
  `27-bind-mdp` byte-identical; `STATUS.md` line.

---

## Phase 3 — plugin-number

Workflow: §7. All numbering lives in one plugin; `attrs.number` is an array.

### [x] T3.1 — Parse `%%type` in `crates/extra`

- **C**: `extra`.
- **R**: `crates/extra/src/{lib,decorator,extras}.rs` (`ParsedExtras`,
  `to_attributes`, ADR-0001 transport), SYNTAX §5, golden `02-extras-literal`.
- **Do**: `NumberMark { depth: u8, number_type: String }`; accept `%`-runs of
  2…7 immediately followed by a name, canonical position = **last** in the head
  (after props, flags, `--cssvars`); attach to the head or to a preceding extras
  body; re-emit as the internal attribute `__number` with value
  `{"depth":1,"type":"numberB"}` — `depth` is stored **already normalized**
  (`%` count − 1: `%%`→1 … `%%%%%%%`→6), so `plugin-number` never re-counts. `%`
  alone, `%%` without a name and runs of 8+
  stay **literal text** with no diagnostic (D-9).
- **Verify**: `cargo test -p pendon-extra` + the `02-extras-literal` golden.
- **Done**: unit tests cover all four literal cases and both legal positions.

### [x] T3.2 — Confirm `__number` never reaches output

- **R**: `crates/renderer-html/src/utils.rs:32`,
  `crates/renderer-solid/src/{node,template}.rs` (both already skip
  `__`-prefixed keys).
- **Do**: no renderer change expected — add one assertion per renderer (AST/JSON
  included) that a `__number` attribute is absent from the rendered output, so
  the rule is tested rather than assumed.
- **Verify**: `cargo test -p pendon-renderer-solid -p pendon-renderer-html`.
- **Done**: three tests, one per renderer family.

### [x] T3.3 — `crates/plugin-number` skeleton + number systems

- **C**: new crate.
- **R**: `docs/guides/plugin-authoring.md`, `crates/plugin-heading/README.md`
  (for the pattern this replaces), issue `plugin-number.md:241–308` for the
  systems table.
- **Do**: crate with `NumberOptions`, `NumberRule` (selectors + `style`), and
  `NUMBER_SYSTEMS` = tier 1 only: `decimal`, `lower-roman`, `upper-roman`,
  `lower-alpha`, `upper-alpha`, `lower-latin`, `upper-latin`, `lower-greek`
  (D-8); formatter unit tests per system plus the out-of-range → `decimal`
  fallback.
- **Verify**: `cargo test -p pendon-plugin-number`.
- **Done**: every tier-1 system has a test at 1, 4, 9, 27, 3999 and 4000 (roman
  falls back past 3999).

### [x] T3.4 — Two passes, group minimum, counters

- **Do**: pass 1 collects member depths per group → group minimum; pass 2 assigns
  numbers. State per winning rule: `levels: Vec<u64>`, increment at the
  normalized depth, reset deeper levels, ghost-fill parents with the style's
  first value. `attrs.number` is `["1","2","a"]` with exactly
  `normalized_depth` entries. `%%` members leave their `.number` group and number
  from the task-level group using their **absolute** depth (the normalized
  `depth` in `__number`), ghosts always counted. Elements with no natural depth are depth 1.
- **Verify**: `cargo test -p pendon-plugin-number` with fixture tests for
  "minimum appears in the middle" and "initial depth does not exist".
- **Done**: both edge cases from §7.1 of the proposal are covered by tests, not
  only by goldens.

### [x] T3.5 — Styles resolve per depth

- **Do**: string style → all depths; array style → depth _n_ uses entry _n−1_;
  deeper depths and out-of-range entries fall back to `decimal`.
- **Verify**: `cargo test -p pendon-plugin-number style`.
- **Done**: the P.2 style table (`I.a`, `II.`, `A.`, …) is a unit test.

### [x] T3.6 — `plugin-heading` loses numbering

- **R**: `crates/plugin-heading/src/lib.rs` (the `NumberStyle` enum, the counter,
  `lib.rs:351`), `crates/plugin-heading/README.md`.
- **Do**: delete `NumberStyle`, the counter and both behaviours; keep
  `auto_number`/`number_style` in `HeadingTaskConfig` as `Option<toml::Value>`
  so a stale config fails with the §14 migration message; stop emitting
  `number`; update the README to point at `plugin-number`.
- **Verify**: `cargo test -p pendon-plugin-heading`, plus a CLI test asserting
  the migration message text.
- **Done**: no `number` attribute is produced by any heading code path.

### [x] T3.7 — CLI wiring

- **C**: `apps/cli`.
- **Do**: add the crate to workspace `members` and `apps/cli/Cargo.toml`;
  `NumberTaskConfig { custom: Vec<NumberRule> }` plus
  `number: Option<Vec<NumberRule>>` on every element config; `build_number_options`;
  a `"number"` arm in `chain.rs`; a pre-flight diagnostic when `number` precedes
  a producer plugin or follows `markdown`.
- **Verify**: `cargo test -p pendon chain` + `cargo test -p pendon config`.
- **Done**: a misordered `plugin = "number,heading"` fails before rendering with
  a message naming both plugins.

### [x] T3.8 — Goldens

- **Do**: re-freeze `06-heading` (the fixture loses its numbers; update the
  README row's description), add `30-number-heading` (the P.1 illustration) and
  `31-number-custom` (the P.2 illustration, task-level `[[task.number.custom]]`).
- **Verify**: `cargo test -p pendon --test syntax_spec`.
- **Done**: `30`/`31` byte-freeze the two acceptance lists in §7.4 verbatim.

### [x] T3.9 — Docs + Phase 3 gate

- **Do**: SYNTAX §5 gains the `%%` item and §N for `number` config (one place;
  link from `crates/plugin-number/README.md`), ADR for the `__number` transport
  decision, `STATUS.md`.
- **Gate**: `cargo test --workspace`, `cargo clippy --workspace --all-targets`.

> Landed with four deviations from the sketch:
>
> - The `%%` grammar has **three** positions, not two: a bare `%%name` head of its
>   own (`[link](/u)%%a`, `##%%a`), the last item of a `{…}` body, and a trailing
>   mark after a head (`@@anchorA%%a`, `@@headingA{…}%%a`).
> - `plugin-image`'s HTML fallback prefixed the internal attribute as
>   `data-__number`, so `plugin-number` never saw it. A `__`-prefixed key now
>   passes through verbatim (one test), matching the renderers' strip rule.
> - A marker can only attach to a construct that owns the rest of its line, so a
>   plain image is numbered as `![alt](/a.png)%%numberA` alone on its line; the
>   P.2 illustration's trailing prose stays literal text (§7.1). Golden `31`
>   reflects that.
> - Golden names stayed `30-number-heading` / `31-number-custom` (this file's
>   sketch, not the ADR's stale `29/30/31-…cite` placeholders); ADR-0010's
>   evidence line was corrected.

### [x] T3.10 — Editor grammar for `%%` (optional, skippable)

- **C**: `editors/vscode/`.
- **R**: `editors/vscode/README.md`, `editors/vscode/syntaxes/pendon.tmLanguage.json`
  (`:182`/`:202` are the `@@`-head rules to sit beside), SYNTAX §5.
- **Do**: one `match` rule for `(%{2,7})([A-Za-z][A-Za-z0-9]*)` inside the extras
  head scope, with a `punctuation.definition.number.pendon` capture; runs of 8+
  and a bare `%%` must **not** match (they are literal text) — express that in the
  regex, not in a comment.
- **Verify**: the scope harness named in `editors/vscode/README.md`, plus a manual
  check against golden `30-number-heading`.
- **Done**: plain `%` and `8+` `%` stay unstyled in the editor.

> Landed as `#numberMark` in both grammars (the `.md` injection shares the
> repository), scoped `meta.number-mark.pendon`; `tools/self-test.mjs` and
> `check-scopes` on golden `31` confirm `%%{2,7}name` matches while `%% text`,
> a single `%` and an 8+ run stay plain.

---

## Phase 4 — plugin-extract

Workflow: §8. Files **and** exports from any extras-capable element;
`plugin-extract-heading` is deleted.

### [ ] T4.1 — `spec/EXTRAS-INVENTORY.md` (Q.1, blocks all code)

- **C**: `docs/spec/`.
- **R**: `SYNTAX.md`'s attribute list per element, each plugin's emit block, and
  each element's golden `.jsx` — a field that appears in none of the three does
  not exist.
- **Do**: one row per element (the §8.6 in-scope list), one column per field, a
  `recommended` mark per element (D-7), the field's only-name, and its type
  (`string`/`number`/`string[]`/`object`). Out-of-scope elements get no row.
- **Verify**: every field in the table appears in that element's golden `.jsx`
  (`grep` each name).
- **Done**: `recommended` presets frozen for all 15 in-scope elements — this is
  the last blocking input for the plugin.

### [ ] T4.2 — `pendon-plugin-extract` core

- **C**: new crate + `apps/cli` wiring.
- **Do**: `ExtractBlock { selector, output: Vec<String>, format: Vec<Format>,
  only, omit, recommended: bool, export: Option<String> }` parsed per element
  from `[[task.<element>.extract]]`; validate that `output.len() == format.len()`
  (error names the element and index) and that `format` is present; resolve
  `type` through `pendon-types` with `matching` (Phase 1) so disjoint blocks over
  one element kind work; skip a block that matches nothing, silently.
- **Verify**: `cargo test -p pendon-plugin-extract` (config validation cases).
- **Done**: every config error names `task.<element>.extract[i].<key>`.

### [ ] T4.3 — Field selection pipeline

- **Do**: all fields → `recommended` restricts to the preset → `only`
  intersects (a custom attribute outside the preset is ignored when
  `recommended` is on) → `omit` subtracts last, **always winning**. A field the
  element does not carry is dropped, never emitted empty, even when `only` names
  it.
- **Verify**: a unit test replaying P.3 exactly (`recommended + only + omit` →
  exactly `citation_number`).
- **Done**: the four-step order is one function with one test per step.

### [ ] T4.4 — Writers: files and exports

- **Do**: one destination path = one flat array; several blocks to the same path
  overwrite (last wins, D-4's spirit); `[segment]`-dynamic paths resolve
  input-relative like `output` today; a file is written only after the render
  succeeds; `export` pushes through Phase 2's `exports` list (so a non-Solid task
  drops it) and is validated by T2.4's validator.
- **Verify**: `cargo test -p pendon-plugin-extract` + the P.1.a/b/c scenario as
  an integration test.
- **Done**: extract writes nothing for a `Payload` document (test, not comment).

### [ ] T4.5 — Delete `plugin-extract-heading`

- **R**: `crates/plugin-extract-heading/` (whole crate),
  `apps/cli/src/chain.rs` (both `extract-heading` arms),
  `apps/cli/Cargo.toml:19`, `Cargo.toml:14`,
  `sandbox/{heading,ultimate,unified,universal}/pendon.toml` (six plugin lists).
- **Do**: delete the crate, both chain arms, both manifest entries, the README,
  and the name from the six sandbox plugin lists; replace
  `chain.rs:232`'s `if fragment && name == "extract-heading" { continue }` with
  the general rule **only a `Page` document writes document-level outputs**
  (files + exports); a `Payload` still runs every plugin for in-document effects.
- **Verify**: `cargo build --workspace` and
  `cargo run --bin pendon -- run` inside `sandbox/heading`.
- **Done**: `grep -rn "extract-heading" --include=*.toml --include=*.rs .` is
  empty; an unknown-plugin name still warns (it is just an unknown name now).

### [ ] T4.6 — Side outputs in the cache

- **R**: `apps/cli/src/cache.rs` (`CacheEntry:48`, `CACHE_VERSION:28`,
  `create_entry:222`, `is_fresh:137`, the private tmp+rename at `:130`),
  `apps/cli/src/process.rs:249`.
- **Do**: `side_outputs: Vec<OutputInfo { path, hash }>`; bump
  `CACHE_VERSION` to 2; `create_entry` takes the side paths; `is_fresh` requires
  each side file to exist with its recorded hash; lift the tmp+rename into
  `write_atomically` and use it for side files.
- **Verify**: `cargo test -p pendon cache`.
- **Done**: deleting a side file makes the next run rewrite it; a bumped version
  silently invalidates old records (existing behaviour).

### [ ] T4.7 — Goldens `32-extract-heading`, `33-extract-scopes`

- **Do**: `32` rebuilds the outline that `sandbox/heading`'s `extract-heading`
  produced **before** deletion (capture that output first, in T4.5's branch, and
  diff against it); `33` covers P.1.a/b/c (`CiteA`, `CiteB`, `_`, overlapping
  `export`). Freeze both with the real CLI and add README rows.
- **Verify**: `cargo test -p pendon --test syntax_spec`.
- **Done**: `32` proves the deletion behaviour-preserving; `33` proves the scoping
  and last-wins rules.

### [ ] T4.8 — Docs + Phase 4 gate

- **Do**: SYNTAX's `extract` §N (keys, order, `omit` wins, flat arrays, depth
  field), `docs/guides/cli.md` if a flag moves, ADR for "only `Page` writes
  document-level outputs", `STATUS.md` cleanup (drop every line for a finished
  phase).
- **Gate**: `cargo test --workspace`, `cargo clippy --workspace --all-targets`,
  all goldens green.

---

## Done criteria for the whole sequence

- [ ] `cargo test --workspace` and `cargo clippy --workspace --all-targets` clean.
- [ ] Every acceptance box in the workflow's §5.3, §6.3, §7.4, §8.5 checked.
- [ ] `SYNTAX.md` is the only place the rules live; `docs/workflows/*` stays
      internal and can be archived once the four phases land.
- [ ] `STATUS.md` has no line about this sequence.
