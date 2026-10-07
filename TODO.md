# Pendon — Typed Extras Reconciliation (D1–D10)

Plan of record for reconciling the parser, the fixtures and the docs with the ten
syntax decisions below. **Status: complete.** D1–D10 are implemented,
documented, frozen and gated; the only open work is explicitly deferred to
Phase 4 and listed under _Deferred_ at the bottom.

The normative text is `docs/spec/SYNTAX.md`; this file is the checklist, the
rationale and the record of what evidence pins each decision.

## Decisions

| #   | Decision                                                                                                                                                                                                                                                                                                                  | Status |
| --- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------ |
| D1  | A type is `ALPHA ( ALPHA \| DIGIT )*` (ASCII, any case, no symbols). After the type run: `{` ⇒ typed head; whitespace/EOL/EOF ⇒ **type-only head**; any other symbol ⇒ type-only head with the symbol left as literal text. `@@type {…}` (space then `{`) ⇒ **literal**. `@@type-x{.x}` ⇒ type `type` + literal `-x{.x}`. | done   |
| D2  | `{}` and `@@{}` are both a valid **empty head**, same output, no warning. `@@{…}` stays a valid untyped head.                                                                                                                                                                                                             | done   |
| D3  | The legacy `[.class,#id]{k:v}` form is **removed completely** (code, docs, sandboxes, READMEs): no warning, no deprecation path. `legacy_extras_warning`, `parse_attrs`, `ParsedAttrs` and the core `ExtraAttrs` are gone.                                                                                                | done   |
| D4  | Strict adjacency everywhere; `@@type {…}` stays literal. Only **list** and **blockquote** allow a space between marker / head / content (reserved for Phase 4). Tables get no exception: a head touches its cell's `\|`, and in a delimiter cell it goes _after_ the alignment code (and width) and touches it.           | done   |
| D5  | Cite is only `[^^](ref "loc")`: unquoted ref, optional quoted loc. `[^^]("book")`, `[^^]("book", "loc")`, `loc=` and `[^^]()` are literal text.                                                                                                                                                                           | done   |
| D6  | Setext headings never existed; `---` is micromatter / `<hr />` only.                                                                                                                                                                                                                                                      | done   |
| D7  | Drop `list`/`blockquote` from the plugin lists and the TOML until Phase 4 (freeze the Q/L sections as plain text). Unknown plugin names and unknown `task.*` keys warn instead of being silent.                                                                                                                           | done   |
| D8  | An unclaimed/default directive renders `<span>` (inline) / `<div>` (block), mirroring markers.                                                                                                                                                                                                                            | done   |
| D9  | Commit `out.frozen/`, add a real regression test that regenerates the three ultimate tasks into a temp dir and diffs `out.frozen/`.                                                                                                                                                                                       | done   |
| D10 | New fixture `21-extras-forms` (bare, type-only, empty, adjacency across anchor/heading/table); rewrite `02` to the new rules.                                                                                                                                                                                             | done   |

## Phase A — `crates/extra`: one unified head scanner

- [x] `typed.rs` — `read_head` reads `ALPHA (ALPHA|DIGIT)*`, then `{` ⇒ typed
      head; whitespace + `{` on the same line ⇒ `InvalidHead` (literal, §4.1);
      otherwise a **type-only head** that consumes only the type run.
- [x] `parse_extras` accepts both spellings: `@@…` via `read_head`, bare `{…}`
      via `read_body`.
- [x] `@@{}` / `{}` ⇒ valid empty head (the old `InvalidItem` escape is gone).
- [x] `parse_type_marker` kept in sync with the type rule (`@@anchorA. tail`
      scans the marker without consuming the `.`).
- [x] `bind.rs` — `legacy_extras_warning` dropped.
- [x] `lib.rs` — `parse_attrs`, `ParsedAttrs`, `ExtraAttrs`, `parse_property_block`,
      `parse_class_block`, `parse_properties`, `split_csv` dropped; only the extras
      API is re-exported and the module doc points at §14.
- [x] `extra/tests/extras_spec.rs` — `head_must_be_adjacent`,
      `malformed_heads_fall_back_to_literal_text`,
      `a_type_only_head_carries_the_type_and_nothing_else`, `empty_heads_are_valid`,
      `the_bare_spelling_is_a_head`.
- [x] `crates/extra` unit tests cover the bare/prefixed equivalence and the
      empty head.

## Phase B — construct plugins: strict adjacency, no legacy

- [x] `plugin-anchor` — dropped `parse_link_extra_attrs`/`ExtraAttrs`; the head
      is parsed directly after `)` and merged via `ExtrasHead`.
- [x] `plugin-heading` — dropped the `[...]`/`(…)` legacy head, the `{…}`
      property block, the whitespace skips and the deprecation warning; the head
      touches `#`.
- [x] `plugin-extract-heading` — mirrors the heading prefix rules.
- [x] `plugin-img` — dropped the legacy block and its warning; extras sit right
      after `)`, the type is routed per instance (figure/img nesting per §7.4).
- [x] `plugin-cite` — dropped the legacy block and its warning (see Phase C).
- [x] `plugin-wiki` — audited: it parses its own `[.class]` infobox prefixes and
      never went through `parse_attrs` (see _Not legacy_).
- [x] `plugin-table` — `AttrSpec`/`parse_attr_block` replaced by the extras
      model; every slot is strictly adjacent: cell-front head before the content,
      delimiter-cell head after `:---(200px)`, caption head touching `||`, row head
      touching the last `\|`; the trailing cell block and `-[.row]` are gone.
- [x] `plugin-marker` / `plugin-directive` — both use the shared scanner.
- [x] `plugin-vicado` — keeps its own fence `{…}` syntax with in-crate
      `split_csv` / `split_key_value` / `unquote` / `parse_props_block`; the
      `pendon_extra` dependency is gone.

## Phase C — cite grammar (`plugin-cite`)

- [x] `[^^](ref "loc")` only: unquoted ref, optional quoted loc.
- [x] The `loc=` prop, the quoted-id form and `("a", "b")` are literal; the
      parser emits no warning and no deprecation event.
- [x] Crate unit tests and `crates/plugin-cite/README.md` updated.

## Phase D — directive fallback (`plugin-directive`)

- [x] `emit_directive` falls back to `NodeKind::Element("div")` for blocks and
      `NodeKind::Element("span")` inline when `custom.select(type)` returns `None`,
      carrying `type`, `name` and the extras, exactly like a marker (§10.1–§10.3).
- [x] The affected unit tests (`block directives render <div>`, inline ones
      `<span>`) are green.

## Phase E — docs (`docs/spec/SYNTAX.md`)

- [x] §2 names both spellings; §3 sigil inventory; §4 the two-entity model, the
      type rule, the literal fallback and the adjacency rule (with the list /
      blockquote exception); §4.3 malformed; §4.4 type names.
- [x] §5 grammar; §7.2–§7.4 construct examples; §8 table slots; §9.2/§9.3
      frozen as Phase 4; §10.1–§10.3 directive/marker fallback.
- [x] §13 warning table (unknown plugin, unknown `task.*` key, unclaimed type);
      §14 retirement table (legacy extras, `[^^]` forms, bare `===`,
      `-[.row]`); §16.3 fixtures; §17.x progress.
- [x] `docs/spec/PARSER.md` mentions neither setext nor legacy extras — no change
      needed.
- [x] Plugin READMEs re-swept (`anchor`, `cite`, `heading`, `img`, `table`): every
      leftover `[.class]`/`[…]{k:v}` example, every quoted cite form
      (`[^^]("ref-id", "loc")`) and every spaced-head claim is gone, and each
      example's documented output was re-generated with the real CLI
      (`sandbox/`-style temp project) instead of hand-written. Two examples were
      necessarily corrected while doing so: the figure caption no longer
      pretends a citation resolves without a reference source configured, and
      `plugin-img`'s `<div>` example uses `section: "hero"` (a `data-section` key
      renders as `data-data-section`).

## Phase F — fixtures and sandboxes

- [x] `golden/02-extras-literal` rewritten to the new rules (`.md` + `.jsx`).
- [x] `golden/21-extras-forms` added (`.md`, `.toml`, `.jsx`): bare head,
      type-only head, empty head, `@@type {…}` literal, adjacency across anchor,
      heading and table slots.
- [x] `golden/05-cite` rewritten (`[^^](book "hlm. 45")`; the `loc=` case is
      gone), `golden/09-table-layers` re-frozen for the new slot placement.
- [x] `sandbox/*/src/*.md` swept with `tools/migrate-extras.py` (one-shot,
      regexes validated case by case; a copy of `sandbox/` was kept in
      `/tmp/sandbox-backup`): `anchor`, `cite`, `image`, `table`, `unified`,
      `universal`. Every sandbox was rebuilt afterwards with no errors.
- [x] Every migrated sandbox was then **diffed against its `HEAD` output** to
      prove no attribute was silently lost. Two classes of defect surfaced and
      were fixed in the sources (outputs regenerated):
  - a space-separated head inside a table cell (`| … | 0 {.text-red} |`,
    `| … | 250.000 { rox: "rox" } |`) is literal text under D4, so both moved to
    the cell front, where the head must touch the cell's `|`:
    `| … |{.text-red} 0 |` and `| … |{rox: "rox"} 250.000 |`;
  - the malformed `[^^]("suryana-2016)` in `sandbox/table/src/integ.md` (an
    unbalanced quote the sweep could not fix) became `[^^](suryana-2016)`.
- [x] `list`/`blockquote` removed from every plugin list; the Q/L sections of
      `sandbox/ultimate` are frozen as plain text with an explanatory comment.

## Phase G — config/plugin hygiene (`apps/cli`)

- [x] An unknown plugin name in `task.plugin` warns once per task and is ignored
      (`apps/cli/src/process.rs`, `reported_plugins`).
- [x] An unknown `task.*` key warns and is ignored: `ConfigTask` gained
      `#[serde(flatten)] unknown: BTreeMap<String, toml::Value>`
      (`apps/cli/src/config.rs`) and `run_from_config` reports every key once
      (`apps/cli/src/run.rs`).

## Phase H — baseline

- [x] `sandbox/ultimate/out/` regenerated from `ultimate.md` with
      `pendon run -F` (clean run, no errors).
- [x] `sandbox/ultimate/out.frozen/` re-frozen from that run after reading the
      three diffs; the fixture tree is excluded from the commit hooks
      (`lefthook.yaml`), see _Open notes_ on `docs/spec/golden/**`.
- [x] `apps/cli/tests/ultimate_freeze_spec.rs` —
      `ultimate_matches_the_frozen_baseline` copies the sandbox into a temp dir,
      runs the real CLI for all three tasks and compares byte for byte, naming the
      first differing line. Verified to fail when a frozen file is touched
      (deliberate re-freeze only).

## Phase I — gates

- [x] `cargo test --workspace` — green.
- [x] `cargo build --workspace` — clean, no warnings.
- [x] `cargo fmt --check` — clean for every file this work touched.
- [x] `dprint fmt` on the touched docs (`docs/spec/SYNTAX.md`, `TODO.md`); the
      new goldens (`02`, `05`, `09`, `21`) were already dprint-clean.
- [x] `git diff` review of every golden and baseline change.

## Deferred to Phase 4

- `plugin-list` and `plugin-blockquote` do not exist: `>` and `-`/`*`/`+`/`1.`
  stay plain markdown, and the modules that would carry `list`, `unordered`,
  `ordered` and the blockquote layer do nothing yet (§9.2/§9.3).
- The Q and L sections of `sandbox/ultimate/src/ultimate.md` are frozen as plain
  text until those plugins land (see the comment in `sandbox/ultimate/pendon.toml`).
- Goldens `10`–`13` (decorator, list, blockquote) stay open for the same reason;
  `17`–`20` are pinned by crate unit tests instead (§17.4).
- A decorator applying to a **list item** or a **blockquote** may allow one
  space between marker and head; that exception is specified (§4.1) but only
  exercised once the two plugins exist.

## Not legacy — do not "migrate"

Two plugins carry a `[.class]`-shaped spelling of their **own**, parsed by their
**own** code. They never went through the retired core parser and their sandboxes
are intentionally unchanged (git reports no diff for `sandbox/wiki` and
`sandbox/vicado`):

- `plugin-wiki`: `:::infobox[rox]`, the `::[.img] … ::` div markers and the
  `Term =[.cill,.elit] Value` class prefix (`crates/plugin-wiki/src/infobox.rs`,
  `parse_class_prefix` / `parse_div_block`).
- `plugin-vicado`: the fence header `~~~ts vicado [.panel, #id] {k: "v"}` keeps
  its in-crate props parser (see Phase B).
- Brace groups in prose are literal text, not heads: `{adipisicing}`, `{foo}` and
  `{_p_ → _q_}` in `sandbox/universal/src/latex.md` are logic/math notation and
  render as text (`{…}` is only a head where a construct looks for one).

## Open notes

- D6 evidence: `Title` + `---` renders `Title` and `<hr />` (never a heading) and
  `Sub` + `===` stays literal text, checked through the real CLI with
  `plugin = "markdown"`; there is no setext code path to remove.
- `@@1anchor{.x}` stays **literal** (a type starts with a letter). This matches
  the frozen `02` case and the fence / `parse_type_marker` rule; it narrows D1's
  "alphanumeric" to "starts with a letter, then alphanumeric".
- Bare `{…}` is canonical, so a `{key: value}` in ordinary prose becomes a head.
  Inline code and raw HTML stay excluded, and the risk is accepted per D2.
- Formatter scope: `sandbox/**` is excluded from biome/dprint in `lefthook.yaml`
  (fixture data), but `docs/spec/golden/**` is not, so a bare
  `biome check` / `dprint check` still reports the _generated_ goldens. Those
  diagnostics are pre-existing and unrelated to this work (verified against the
  `HEAD` copies of untouched files); treating the golden tree like the sandbox
  tree is a separate, deliberate tooling change.
