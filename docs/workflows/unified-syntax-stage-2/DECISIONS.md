# Stage 2 — decisions

Every place where the proposal leaves a choice to the implementer, or where
following it literally would break something we already ship. Each entry is
`Context → Recommendation → Rejected → Consequence`. **A task that needs a
decision does not start until it is marked `settled`.** Change a recommendation
here, never in `TASKS.md`.

Status values: `proposed` (default, needs an owner's yes), `settled`, `rejected`.

## ADR map (one file per _structural_ decision)

| ADR                                                     | covers                         |
| ------------------------------------------------------- | ------------------------------ |
| [`0012`](../../decisions/0012-universal-slots.md)       | D-1, D-11, D-12                |
| [`0013`](../../decisions/0013-decorator-unification.md) | D-10                           |
| [`0014`](../../decisions/0014-field-model.md)           | D-3 – D-8                      |
| [`0015`](../../decisions/0015-extract-promotion.md)     | D-13 (extract), D-14           |
| [`0016`](../../decisions/0016-number-promotion.md)      | D-13 (number), `number.*`      |
| [`0017`](../../decisions/0017-slot-modifiers.md)        | D-9                            |
| [`0018`](../../decisions/0018-clean-replacement.md)     | D-16 (owner policy: no compat) |

All seven are written and registered in
[`docs/decisions/README.md`](../../decisions/README.md) (2026-10-28).

Register each in [`docs/decisions/README.md`](../../decisions/README.md) next to
ADR-0011 when it is created.

---

## D-1 — Where slots are parsed

**Context.** Today four plugins each own a slice of bracket syntax: `plugin-anchor`
(`@[x](y)`), `plugin-cite` (`[^^ref]`), `plugin-image` (`![alt](src)`) and
`plugin-heading` (`#[slug](title)`), while `crates/extra` only knows the _names_
of the positional keys (`PositionalKeys`). Stage 2 makes `[]`/`()` universal, so
a fifth copy is not acceptable.

**Recommendation.** One parser in **`crates/extra/src/slot.rs`**, pure and
markdown-agnostic:

```rust
pub enum SlotKind { Bare, Quote, Tick }                    // fixed order per slot
pub enum SlotTag  { None, Anchor, Cite, Image, Heading }   // `@` `^` `!` `#`
pub struct Slot   { pub tag: SlotTag, pub parts: [SlotParts; 2] }
pub fn parse_slots(rest: &str) -> (Option<Slot>, &str);
```

Called _before_ the core Markdown parser sees the line (the same point where
each element parses its head today — §12.1 "pre-markdown"), so `[` `(` `` ` ``
`"` are still ours. `crates/extra` owns the grammar; a plugin only supplies its
`SlotContract` (which slots it accepts, in what order, and the tag that claims an
untagged standalone slot). No plugin may re-implement bracket splitting.

**Rejected.** (a) A `slot` plugin — parsing must happen before Markdown, and
intra-plugin ordering would become a race. (b) Post-parse AST rewriting —
Markdown has already destroyed the bracket nesting (links, images, emphasis).

**Consequence.** `plugin-heading`'s `#[` and `plugin-cite`'s `[^^` shrink to a
`SlotContract` + tag check; "one head parser per element"
(`CONTRIBUTORS.md`) survives, moved one layer down.

---

## D-2 — Malformed markup: error, warning, or literal?

**Context.** The proposal says "an error is thrown" (duplicate `specs.tick`,
unmergeable modifiers, colliding field names) and "a warning is thrown … ignored
and removed" (decorator problems). Pendon already has both channels
(`Event::Diagnostic`, `Severity::{Error, Warning}` in `crates/core`) and a
long-standing _literal fallback_ rule (§4.3): text we cannot parse is emitted
verbatim — never swallowed, never fatal.

**Recommendation.** Four tiers, no exceptions:

| situation                                                             | channel                                                               |
| --------------------------------------------------------------------- | --------------------------------------------------------------------- |
| ambiguous/incomplete grammar (`[Foo` with no `)`)                     | literal fallback, **no** diagnostic (§4.3)                            |
| complete but illegal (two `specs.tick`, bad modifier, bad field name) | `Severity::Error` + literal fallback **for the offending item only**  |
| legal but inapplicable (decorator over a non-receiver)                | `Severity::Warning` + remove the decorator (§6)                       |
| **retired** syntax/config (Stage-1 forms)                             | literal text, **no** diagnostic (D-16); config keys `Severity::Error` |

The render never panics and never aborts: `pendon run` prints diagnostics and
still writes output (existing CLI contract). "Error is thrown" in the proposal
means `Severity::Error`, **not** a Rust panic or a failed run because one element
is wrong.

**Rejected.** Hard-failing the run; it would break the editor/watch workflow and
contradict §13.

**Consequence.** Every new error gets a diagnostics case and a line in
`SYNTAX.md` §13.

---

## D-3 — Public field names are a projection, not the IR

**Context.** The proposal's extract output is dotted (`specs.class`,
`number.raw`, `price.details.amount`). But the IR node today carries flat
attributes that _internal processors read_ (`reference_id` feeds
`citation_number`, `__number` feeds nesting). Renaming attributes in place would
force every plugin to speak the public vocabulary — and the proposal explicitly
forbids that for overrides ("overrides only apply to the final public result …
must not affect the work of Pendon's internals").

**Recommendation.** Split the two layers explicitly:

- **Internal** (`reference_id`, `__number`, …): plugin-to-plugin data, today's
  flat attributes, never rendered, never extracted. Unchanged by Stage 2.
- **Public** (the _field map_): built once per element at the end of the
  pipeline by a projection step that reads (a) the element's built-in fields,
  (b) slot values under their active assignment, (c) `specs.*` / `number.*`,
  (d) `k: v` at the top level. Dotted paths are the public truth.
- Renderers and `extract` both consume the **public** map. Neither recomputes
  names; neither reads internals.

**Rejected.** (a) Flat public names — cannot distinguish `specs.class` from a
user's `class`, and `include`/`only`/`omit` need paths. (b) Dotted names in the
IR — leaks user configuration into plugin logic and kills D-4.

**Consequence.** Phase 2 builds the map (no behaviour change); Phase 3 moves
specs into it; Phase 4 moves each element's fields into it. `.jsx`/HTML goldens
change once per phase, deliberately.

---

## D-4 — Override applies at the projection boundary only

**Context.** `field.override.<built_in>.= "<custom>"` (proposal §7) renames a
public field. If it were applied to the IR, `plugin-cite` could no longer find
`reference_id` and numbering/citation would silently break.

**Recommendation.** Overrides are a **rename table consulted when the public map
is emitted** (render + extract). They never enter `internal.*`, never reach a
plugin's own reads, and never affect sibling plugins' computations. Collisions
(two overrides producing the same path, or an override shadowing an existing
public field that is also emitted) are `Severity::Error`.

**Rejected.** Applying overrides to the AST. It looks simpler and breaks
numbering, citations, and section nesting in ways that only goldens would catch
late.

**Consequence.** One function (`project(node) -> FieldMap`) is the only place
overrides are read; both renderers call it. Ownership: Phase 2.

> **Implementation note (S2-2.7).** The function is
> `project(node, Option<&FieldConfig>)` (`crates/extra/src/field.rs`); `None` is
> "this element carries no `field.*` config", which is what both DOM renderers
> pass until Phase 4 threads the matched entry through. The identity pass also
> settles two things the decision left implicit: a `__`-prefixed key is internal
> and never enters the map (§11 rule 3 — that is why the renderers no longer test
> the prefix), and an override renames its field **as the entry is written**, so
> the renamed field keeps the replaced field's position and cannot reorder the
> output. A node key that is not a valid field name cannot be public; it is
> dropped with a `debug_assert!` rather than silently renamed.

---

## D-5 — Configuration key taxonomy

**Context.** Stage 2 introduces four _families_ of keys inside
`[[task.<element>.custom]]`, and the proposal names some of them literally
(`field.specs.tick`, `field.bracket.bare`, `field.override.number.raw`,
`bracket.modifier."?"`). They must be consistent, or users will guess wrong.

**Recommendation.** Keep exactly four prefixes, as the proposal writes them:

| prefix                  | meaning                                                       | example                                       |
| ----------------------- | ------------------------------------------------------------- | --------------------------------------------- |
| `field.<slot>.<k>`      | assign a slot's positional value to a public field            | `field.bracket.tick = "extra-class"`          |
| `field.specs.<k>`       | assign a specs position (`tick`/`quote`/`class`/`id`/`style`) | `field.specs.quote = "title"`                 |
| `field.override.<path>` | rename a public field                                         | `field.override.number.raw = "bar"`           |
| `<slot>.modifier."<c>"` | a slot modifier's contribution                                | `bracket.modifier."?" = { rel = "nofollow" }` |

`bracket`/`paren` are the **slot names**; `specs` and `number` are **field
namespaces**, not slots — a modifier never lives inside `field.*`.

**Rejected.** A single flat namespace (`field.tick`, `override.raw`). It cannot
express slot vs specs, and the proposal already fixed the dotted spelling.

**Consequence.** `backtick_key`/`quote_key` become `field.specs.tick`/`quote`,
`bracket_key`/`parentheses_key` become `field.bracket.bare`/`field.paren.bare`
when _assigned_. TOML dotted keys are already what `serde` sees, so the parser
change is mostly `#[serde(rename)]` + validation (Phase 2).

> **Implementation note (S2-2.8).** The four old keys were **live**, not dead
> config: `ExtrasOptions` named a head's four positional contents and
> `to_attributes` emitted those names, so deleting the keys without a replacement
> would have moved golden 15's bytes. The replacement is the config-side type
> itself: `Specs { bracket, paren, tick, quote }` +
> `Specs::resolve(Option<&FieldConfig>)` (`crates/extra/src/typed.rs`), named
> `PositionalNames` until `S2-3.1` renamed the "canonical extras" vocabulary to
> **specs**, resolved in the renderer's component set (`specs_for`) from the
> matched entry's `field.*`. `resolve` is the only reader of D-5 for naming:
>
> - `field.bracket.bare` / `field.paren.bare` name the two groups;
> - `field.specs.tick` / `field.specs.quote` name the two items;
> - an entry that configures **nothing** keeps `slug` / `title` (the preset is
>   alive, D-7); once it configures **anything**, an unassigned position falls back
>   to its own qualified path (`bracket.bare`, `paren.bare`, and — because specs
>   become map entries in Phase 3 — `specs.tick`, `specs.quote`).
>
> `TypedComponent` now carries the entry's `FieldConfig` (not a bespoke naming
> struct), which is also the shape Phase 4 needs. Because the naming path is the
> only consumer, no golden byte moved: fixture 15 migrates its four keys to
> `field.*` and renders identically.

---

## D-6 — Selection vocabulary and the default preset

**Context.** Today an extract rule is `recommended: bool` plus `only`/`omit`, and
the default is "everything". The proposal replaces this with
`preset = "pendon" | "all"`, adds `include`, and makes `pendon` the default.

**Recommendation.** `ExtractRule { preset: Preset, include: Vec<Path>, only:
Vec<Path>, omit: Vec<Path> }` where `Preset { Pendon, All }` defaults to
`Pendon`. Validation, all `Severity::Error` at config load:

- `only` + `include` → error (proposal says they must not be combined).
- `include` + `omit`, `only` + `omit` → allowed.
- `preset = "pendon"` + `only` → allowed, and yields `[{}]` when nothing matches
  (PE.4 — this is _correct output_, not an error; do not "helpfully" fall back).
- unknown preset name → error listing `pendon`, `all`.

**Rejected.** Keeping `recommended = true` as an alias, even once with a
diagnostic — see D-16: the clean-replacement policy deletes the key, so a config
that still writes it is an **error** at load, not a warning and not a mapping.

**Consequence.** `preset = "all"` must include _custom_ slot/extras fields but
still exclude internals (`reference_id`, `__number`) — the same exclusion list
`EXTRAS-INVENTORY.md` already encodes as "not extracted".

---

## D-7 — When the preset assignment is voided

**Context.** Proposal §7: "If there is **even one** user-made field definition
for bracket or paren, Pendon's built-in fields are completely disabled on all
fields available in the related `task.<element>.custom` (not globally)." Read
literally this means one `field.bracket.bare = …` also voids `href`.

**Recommendation.** Implement the literal reading, scoped to the **matched
custom entry**, and state it as one sentence in the spec:

> Defining any `field.*` key in a custom entry disables every preset assignment
> for that element _for that entry_; unassigned slot values fall back to their
> bare names (`bracket.bare`, `paren.quote`, …).

Consequences to encode as tests. For `@[foo "Bar"`baz`](soo "Sar"`zaz`)` with
`field.bracket.bare = "something"`:

- `something` = `foo` (the only assignment),
- `bracket.quote` = `Bar`, `bracket.tick` = `baz`, `paren.bare` = `soo`,
  `paren.quote` = `Sar`, `paren.tick` = `zaz` (bare fallbacks),
- and **no** `inner_text` / `href` / `title` — the preset is void for this entry.

That is the proposal's own example, so there is no half-preset mode.

> **Spelling note (S2-2.6).** The example above is written without a space before
> either tick, and §5.2 makes a space the thing that ends a content — so read
> literally, the quote swallows the tick. The six values listed are the ones the
> spaced head ``@[foo "Bar" `baz`](soo "Sar" `zaz`)`` produces, and that is what
> `preset::tests::one_field_key_voids_the_whole_preset` asserts.

**Rejected.** Per-slot voiding (`field.bracket.*` voids only `bracket.*`). It is
friendlier but contradicts the proposal, and the proposal's own example proves
the all-or-nothing reading.

**Consequence.** Phase 4, per element: document which preset fields vanish, and
list the bare fallbacks in `FIELDS-INVENTORY.md`.

---

## D-8 — Fields-only custom entries

**Context.** The proposal shows two shapes that TOML cannot both hold under one
key: `[task.anchor.custom]` (single table, `field.bracket.bare = "something"`)
and `[[task.anchor.custom]]` (array of tables with `name`/`template`/`type`).

**Recommendation.** `custom` is **always an array of tables**
(`[[task.anchor.custom]]`). An entry with **no** `name`/`template`/`imports` is a
_fields-only_ entry: it configures nothing but field names. Two sub-kinds:

- with `type = "typeA"` → applies to that type only (merges with the component
  entry for the same type, or stands alone if no component exists);
- without `type` → the element-wide default field config.

Validation: an entry with `name`/`template` **and** no `field.*` is a component
(unchanged); an entry with both is a component _with_ field config; an entry with
neither is an error (`Severity::Error`, "empty custom entry").

**Rejected.** Loading two different serde shapes for the same key. It is
ambiguous in TOML and would surprise every user who copies the proposal's
example.

**Consequence.** `docs/spec/SYNTAX.md` §11 examples that use
`[task.<el>.custom]` as a table must be rewritten to the array form; a config
written in the single-table form is a hard load error (D-16), not a warning and
not a converted geometry.

---

## D-9 — Slot modifiers: canonical order and merge

**Context.** `[Foo |?~uv$%]` — `|` starts the modifier run, everything after it
is a symbol, duplicates collapse (`|?xx?|ee?` ≡ `|?x|e`), and the _meaning_ is
pure configuration (`bracket.modifier."?" = { … }`). Two things are unspecified:
what order the symbols contribute in, and how contributions merge.

**Recommendation.**

- **Canonical order** = first appearance in the bracket, left to right, after
  deduplication. `|?y!~x` → `?`, `y`, `!`, `~`, `x`.
- **Repetition** collapses before merging, so a symbol can never contribute
  twice _from one bracket_.
- **Merge** exactly as the proposal's example demands: same target path → array
  (`rel` from `?` and `!` becomes `["nofollow","noopener"]`); array + scalar →
  keep the array; array + array → nested array; object + object → deep merge;
  irreconcilable types → `Severity::Error` (D-2), the element falls back to no
  modifier contribution.
- Modifier contributions are **specs-level fields** (they land beside
  `specs.class`, not inside `field.*`) — the proposal's NOTE says so explicitly.
- Modifiers are bracket-only; a modifier in `()` is literal content, not an
  error (§4.3).
- `|` escaping: only for a bracket that is empty or bare-only (proposal);
  `[Foo "Bar" \|?x]` is `Severity::Error` because it would make bare come after
  quote (slot order, §4).

**Rejected.** Sorting symbols by code point — stable and reproducible, but it
makes `?y` and `y?` behave differently from how they read, and the proposal's
own extraction example is in written order.

**Consequence.** Merge order affects array contents, so it must be covered by a
fixture with two symbols hitting one field (the proposal's `rel` example is the
canonical case).

---

## D-10 — Decorator: precedence and removal

**Context.** `@@` becomes mandatory and moves _first_ (before slot → specs →
numbering), the receiver set becomes blockquote / codefence / list / paragraph /
table, and section may steal a decorator that precedes a heading.

**Recommendation.**

1. **Lexing.** A line is a decorator line iff it starts with `@@` at column 0..
   3 (indent tolerance identical to today's rule) and contains nothing after the
   extras except whitespace. Anything else is literal text.
2. **Claiming order** (first match wins, one decorator per receiver):
   `section` (before a heading, no warning) → `table` (table head line) →
   `list` (first list item) → `blockquote` (`>` line) → `paragraph` (any block).
   Ordered from most specific to least; `crates/extra` owns the order, not the
   plugins.
3. **Rejection** (per proposal): trailing literal text on the decorator line →
   `Warning`, the text is dropped; a receiver that does not accept decorators
   (heading when `section` is off, marker, directive, frontmatter, data block) →
   `Warning`, decorator dropped.
4. The decorator is **removed from the stream** in every case (accepted or not),
   so it can never leak into rendered text.

**Rejected.** (a) Leaving an unclaimed decorator in the text as a visible
`@@…` — the proposal says remove. (b) Warning when `section` claims a heading
decorator — the proposal explicitly says no warning.

**Consequence.** `bind_decorators` grows a claim table; `plugin-section` gets a
pre-claim hook instead of special-casing inside the decorator scan. Fixtures:
one per receiver + 3 rejection cases.

---

## D-11 — Slot tag ambiguity and disabled elements

**Context.** `#` now means _heading slot_ as well as `#` heading + `#id` specs;
`^` inside `[` is also the CommonMark footnote marker (`[^ref]`). Tags are
hard-coded, and a tag whose owner plugin is disabled still appears in text.

**Recommendation.**

- Tags are matched **only** in head position (start of the remaining line, after
  indent), never mid-text, so `foo#[bar]` stays literal.
- `[^…]`: the core parser claims footnotes **before** we do. Verify with
  `OBJ-1` in Phase 1; if footnotes are enabled, cite's tag must be something
  else (proposal's `^` is then amended — this is the one place the proposal can
  lose).
- A tag whose owner is disabled → **literal text, no diagnostic** (the plugin is
  off; the user did not opt into that syntax). If the owner is enabled but the
  element does not accept slots → `Severity::Error` (D-2), literal fallback.
- `#` heading slot is only valid when `plugin-heading` is on **and** the tag is
  immediately followed by `[`; `# Word` remains an ATX heading.

**Consequence.** Phase 1 must test tags with each owner plugin disabled.

---

## D-12 — Specs duplication and accumulation

**Context.** `specs.tick`, `specs.quote`, `specs.id` may appear once (error
otherwise). `specs.class` accumulates; `specs.style` is an object. Order is free
(unlike slots).

**Recommendation.**

- `tick`/`quote`/`id` → second occurrence is `Severity::Error`; the **first**
  wins and is kept alongside the error (so output stays usable).
- `class` → append in written order, dedupe keeping first position (idempotent
  re-render matters for goldens).
- `style` → `--prop: value` accumulates; duplicate property = last wins (CSS
  semantics), no error.
- `k: v` → the field is `k` (never `specs.k`); duplicates are `Severity::Error`
  (two sources for one public path is a bug, not a merge). `k: $bind` keeps
  today's bind semantics unchanged.
- A `k` that collides with a _built-in_ field of that element (e.g. `href` on an
  anchor) → `Severity::Error`: the user must use `field.override.href` (D-4).

**Consequence.** One unit-test table per rule in `crates/extra`; the `href`
collision is the case agents will otherwise implement as "last wins".

---

## D-13 — How a "plugin" becomes a "main crate"

**Context.** `plugin-extract` and `plugin-number` are plugins in name only:
extraction is driven by `[[task.<element>.extract]]` (a _task_), and numbering is
read by every element. Their packages are `pendon-plugin-extract` /
`pendon-plugin-number`, and the CLI imports them directly
(`apps/cli/src/plugins.rs`, `process.rs`).

**Recommendation.** Pure mechanical promotion — no behaviour in the rename task:

- `crates/plugin-extract` → `crates/extract`, package `pendon-extract`;
  `crates/plugin-number` → `crates/number`, package `pendon-number`.
- **Config surface does not change**: still `[[task.<element>.extract]]` and
  `[[task.<element>.number]]`. Users never see crate names; only docs do.
- Plugin _tokens_ are not renamed (`frontmatter` etc. keep their tokens); only
  the crate/README layer moves.
- If either type only exists to satisfy the `Plugin` trait, it stops being a
  `Plugin`; if it is a chain stage today, it stays one — this task must not
  change pipeline order. Verify in `apps/cli/src/chain.rs` before editing.
- `docs/spec/EXTRAS-INVENTORY.md` → `FIELDS-INVENTORY.md` lands in the same PR as
  `extract`, so presets and inventory change together (Phase 7 → 8).

**Rejected.** Renaming the _configuration_ (`[[task.<element>.extraction]]`).
Churn for users, zero benefit.

**Consequence.** ADR-0015/0016 record the promotion; `CONTRIBUTORS.md` gains one
sentence on what "main crate" means vs "plugin crate".

---

## D-14 — Frontmatter extraction

**Context.** Frontmatter is already an element (`# frontmatter` in the head
list) and can already export `export const frontmatter`. Stage 2 says it must be
extractable like any element, with **no preset**.

**Recommendation.** Accept `[[task.frontmatter.extract]]` with `output`,
`format` (`json`/`yaml`/`toml`/`export`), `only`, `omit`. Reject `preset` and
`include` (error: "frontmatter has no preset; use only/omit"). Fields = the
top-level frontmatter keys, paths = nested keys. A pre-existing
`export const frontmatter` (§20) plus an `export`-format extract block for
frontmatter must not define the same const twice — duplicate is
`Severity::Error`.

**Rejected.** Treating frontmatter as a preset element. It has no element
fields, only user data.

**Consequence.** One fixture per format + one duplicate-const error case.

---

## D-15 — Where the new grammar lives in `SYNTAX.md`

**Context.** Slots, slot tags, modifiers and fields are new normative grammar.
New top-level sections would renumber §7–§21 and break every link in a dozen
READMEs.

**Recommendation.** **No renumbering.** Add subsections in place:

| change                                                        | section                  |
| ------------------------------------------------------------- | ------------------------ |
| retitle §5 "Heads: extras, slots"; add slots, tags, modifiers | §5.2, §5.3, §5.4         |
| replace "Positional names" with "Slot contracts"              | §6.1 (rewrite)           |
| public field projection, preset assignment, voiding (D-7)     | §6.5, §6.6               |
| per-element grammar                                           | §7.1–§7.4, §8, §9.1–§9.5 |
| `number.*` public fields                                      | §10.5                    |
| new diagnostics                                               | §13                      |
| every removal                                                 | §14                      |
| extraction rules + frontmatter                                | §21, §21.x               |
| terminology (slot, modifier, preset, field)                   | §2                       |

**Rejected.** A clean top-to-bottom rewrite in Stage 2. It reads better, but it
makes every task's diff unreviewable and every cross-link wrong; schedule it
after Phase 10 as its own change.

**Consequence.** `SYNTAX.md` grows subsections, not sections. Links stay valid;
`docs/spec/golden/README.md` and `AGENTS.md` need no edits.

---

## D-16 — Breaking-change posture: clean total replacement

**Context.** Pendon is **alpha**. The owner's policy for Stage 2 (stated
2026-10-28) is a clean, total replacement: **no** backward compatibility with old
config or output formats, **no** deprecation warnings or transitional periods,
**no** changelog/migration notes for old field names, **no** legacy code paths.
Users adopt the new API in full; old configuration must fail fast rather than
silently fall back.

**Recommendation.** One rule, applied without exception:

- **Deleted means deleted.** Every removed syntax, config key, field name and
  helper is deleted from the code in the task that removes it — no
  `#[serde(alias)]`, no `#[deprecated]`, no dual-parse branch, no
  `#[allow(dead_code)]` stub, no re-exported shim, no `// TODO: remove in 0.1`
  marker. If a golden still needs the code, the golden is wrong and changes in the
  same task.
- **Config fails fast.** Unknown keys, removed keys and removed
  `[task.<el>.*]` tables are `Severity::Error` at load (today they are §13
  warnings). The message names the offending key and the load aborts before any
  render; it may name the current key only because the user must unblock, and it
  is a _diagnostic_, not a documented old→new mapping.
- **Removed syntax is not grammar.** It falls back to literal text under the
  existing §4.3 rule, with **no** deprecation diagnostic and no warning — the same
  way a typo is text. §14 still records _that_ a form is no longer syntax (that is
  a normative grammar statement, and it keeps `SYNTAX.md` honest), but it carries
  no "write this instead" column, no dates, no versions, and no prose migration
  guidance.
- **No migration document.** This workflow ships `README.md`, `DECISIONS.md` and
  `TASKS.md` only. There is no `MIGRATION.md`, no "upgrading" guide, and no
  old→new compatibility table for Stage-2 changes anywhere in `docs/`.
  `STATUS.md` records work, not transitions.
- **One version.** `0.0.x`, no semver ceremony, no major-version note. The new
  API _is_ the API.

**Rejected.** (a) A grace release with warning shims — that is precisely what the
owner ruled out, and it doubles the fixture surface. (b) A migration guide — it
would be a standing promise that the old names still work somewhere, which they
must not. (c) Keeping the old parser behind a flag — two grammars, two fixture
sets, permanent drift.

**Consequence.** Downstream tasks lose their shim clauses
(`backtick_key`/`quote_key`/`bracket_key`/`parentheses_key` → hard deletion in
S2-2.8; `recommended` → deleted in S2-8.2; cite's `*_prefix`/`citation_class` →
deleted in S2-4.4), `MIGRATION.md` is deleted, and each removal task must also
prove the old form is now literal text or a load error (a fixture, per D-17).
Removals still land in the _same_ task that deletes the code — never a "cleanup
later" task.

---

## D-17 — Fixture and golden strategy

**Context.** Almost every task changes output. Re-freezing everything in one big
bang at the end makes the semantics unreviewable.

**Recommendation.**

- **Spec-first per task**: add or change the fixture _source_ and its expected
  `.jsx` in the same task that changes behaviour; re-freeze with
  `pendon run -F`; the diff is part of the review.
- Prefer **unit tests** for grammar (`crates/extra`, `crates/types`) — slot
  order, tag, modifiers, specs need no renderer.
- Use **goldens** for anything reaching output (`.jsx`, HTML, JSON, extract
  files).
- A phase gate needs: its fixture set green, no _unintended_ hunk in the
  re-freeze diff, `cargo test --workspace` green.

**Rejected.** Committing `#[ignore]`-ed future fixtures early — they rot, and
they hide the failure they were meant to signal.

**Consequence.** `TASKS.md` names the fixture for every behaviour-changing task.

---

## Objectives — answered at sign-off (2026-10-28)

Each answer is binding for the phase that owns it; where an answer contradicts the
proposal, the answer wins and `SYNTAX.md` records the corrected rule.

| id     | question                                                                                                                                                                                                                                                                                                                                                                                         | owner / when           |
| ------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ---------------------- |
| OBJ-1  | Does the core parser claim `[^ref]` as a footnote? Today cite is `[^^ref]` (two carets); the new tag makes it one (`[^ref]`).                                                                                                                                                                                                                                                                    | Phase 1, before S2-1.4 |
| OBJ-2  | Proposal §4 has a typo: "a slot that has no **modifier** and stands alone becomes literal" — it means **no slot tag**. Confirm while writing §5.2.                                                                                                                                                                                                                                               | Phase 1 docs           |
| OBJ-3  | Does §3's "writing order" (`@@type{specs}%%number`) apply to _every_ head, or only to decorators? Slots come _before_ extras everywhere.                                                                                                                                                                                                                                                         | Phase 1                |
| OBJ-4  | For a non-slot-based head (marker/heading/decorator), a slot may appear untagged. Does that mean marker/heading _consume_ a following `[...]`, or only decorators?                                                                                                                                                                                                                               | Phase 4 (marker)       |
| OBJ-5  | Table: does the old `@@` head line persist for tables, or is the decorator the only form?                                                                                                                                                                                                                                                                                                        | Phase 5                |
| OBJ-6  | Numbering: today `__number` internal attrs are consumed by nesting. Confirm `number.depth` is the same value (not a re-count).                                                                                                                                                                                                                                                                   | Phase 9                |
| OBJ-7  | `preset = "all"` on an element with zero extractable fields — emit `[{}]` per row or skip the row?                                                                                                                                                                                                                                                                                               | Phase 8                |
| OBJ-8  | The proposal contradicts itself on numbering's depth: §7.4 says the heading's number `depth` is renamed to `number_depth`, §9 defines `number.depth`. Pick one before S2-4.6/S2-9.2 — recommended: `number.depth` (namespaced, one rule for every element).                                                                                                                                      | Phase 4 (heading)      |
| OBJ-9  | "Preset" is used for two different things: the element's built-in **slot→field assignment** (e.g. heading `paren.quote → title`) and the **extraction preset** (`preset = "pendon"`, D-6). §7.4 even lists heading's preset as `slug, depth, inner_text` while assigning `title`. Keep both words apart and decide whether slot-contracted fields join the extraction preset (Phase 7 + S2-8.2). | Phase 7                |
| OBJ-10 | The cite example unassigns a position with `field.bracket.bare = ""`. Confirm empty string means _unassign_ (fall back to the bare field), not "a field whose name is empty" — and that it is the only documented way.                                                                                                                                                                           | Phase 2 (S2-2.3)       |
| OBJ-11 | `SYNTAX.md` §14 still carries pre-Stage-2 rows with a "write this instead" column and "migration message" prose. Under D-16, does the sweep (S2-10.2) prune those rows to removal-only statements too, or does the ban apply to Stage-2 changes only? Recommended: prune them — one §14 shape for every stage.                                                                                   | Phase 10 (S2-10.2)     |

### Answers

- **OBJ-1 — no collision, `^` stands.** `crates/core` implements no footnote
  extension (`grep -rn "footnote" crates/core/src` → no match), so `[^ref]` is
  never claimed as footnote syntax and reaches the slot parser as ordinary text.
  Cite's tag is one caret; D-11's fallback clause is dead. Pinned by the S2-1.4
  tag fixture and a `[^ref]`-in-prose case.
- **OBJ-2 — the typo is "no slot tag".** A slot with no tag standing alone is
  literal; the modifier run needs a slot next to it (`[Foo |?x]`). §5.2 states
  this; no other reading survives `[Foo]` being a link.
- **OBJ-3 — the order is per-head, not global.** A head's parts are always
  `@@type` → `{specs}` → `%%number`; slots come _before_ all three because they
  are the head's own content (`@[…]()[…]{…}`). The two orders never interleave,
  so "writing order" needs no exception for decorators — a decorator is a
  separate line and carries extras only.
- **OBJ-4 — only a decorator is claimed across lines.** Marker/heading do not
  consume a following `[...]`; a slot is read only at the position its element's
  contract puts it. An untagged block-level `[...]` is literal.
- **OBJ-5 — the decorator is the only form.** The old `@@type{…}` line _is_ the
  decorator line (same text, same position); tables simply stop accepting the
  inline `@@` head. No second rule, no deprecation notice (D-16).
- **OBJ-6 — same value, not a re-count.** `number.depth` is the depth copied from
  the `__number` metadata (D-3: the projection copies, it does not recount).
- **OBJ-7 — skip the row.** `preset = "all"` on an element with zero extractable
  fields emits nothing; PE.4's `[{}]` is reserved for "the element has fields but
  the filter matched none". Fixture: an element with no `[[task.<el>.extract]]`
  fields and `preset = "all"`.
- **OBJ-8 — `number.depth`.** The namespaced path wins; §7.4's `number_depth` is
  not part of the field model and is removed by S2-4.6.
- **OBJ-9 — two words, two meanings.** _Preset assignment_ = the element's
  built-in slot→field mapping (heading `paren.quote → title`). _Extraction
  preset_ = `preset = "pendon" | "all" | [names]` (D-6). Slot-contracted fields
  join the element's extraction preset only when the element's preset table lists
  them, so §7.4's `slug, depth, inner_text` list is corrected there, not here.
- **OBJ-10 — `""` means unassign**, and it is the only documented way: the
  position falls back to the bare field. An empty _name_ is never valid (paths are
  non-empty by construction), so the two cases cannot be confused.
- **OBJ-11 — prune, retroactively.** `SYNTAX.md` §14 becomes removal-only for
  _every_ stage: each row states that a form is no longer syntax, with no
  "write this instead" column and no migration prose. S2-10.2 does the pruning
  and S2-10.3's grep enforces it.

## Sign-off

| decision | status  | note                                                                        |
| -------- | ------- | --------------------------------------------------------------------------- |
| D-1      | settled | 2026-10-28 — universal slots, parsed before Markdown                        |
| D-2      | settled | 2026-10-28 — four tiers, incl. retired syntax/config                        |
| D-3      | settled | 2026-10-28 — single projection source; unblocks Phase 2                     |
| D-4      | settled | 2026-10-28 — override paths read in the projection; unblocks Phase 2        |
| D-5      | settled | 2026-10-28                                                                  |
| D-6      | settled | 2026-10-28 — `recommended` is a load error, not a mapping                   |
| D-7      | settled | 2026-10-28                                                                  |
| D-8      | settled | 2026-10-28 — explicit yes: one `custom` table, the proposal's TOML is wrong |
| D-9      | settled | 2026-10-28 — written order, dedupe before merge                             |
| D-10     | settled | 2026-10-28 — claim order lives in `crates/extra`                            |
| D-11     | settled | 2026-10-28 — cite keeps `^`; OBJ-1 proved `crates/core` has no footnotes    |
| D-12     | settled | 2026-10-28                                                                  |
| D-13     | settled | 2026-10-28 — mechanical promotion, config surface unchanged                 |
| D-14     | settled | 2026-10-28 — frontmatter extractable, no preset                             |
| D-15     | settled | 2026-10-28 — subsections in place, no renumbering                           |
| D-16     | settled | owner policy 2026-10-28: clean total replacement, no compat                 |
| D-17     | settled | 2026-10-28                                                                  |

**Phase 0 exit met 2026-10-28.** Every decision above is `settled` and every
objective below is answered. Baseline before Phase 1: `cargo test --workspace`
green at `244bbb9` — 645 passed, 0 failed.
