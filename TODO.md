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

## Phase M — list, blockquote and the decorator binder (in progress)

The Phase 4 work above, taken in order. Nothing in this phase is wired into the
CLI yet: `plugin-blockquote` / `plugin-list` still have to be added to
`apps/cli` (options + dispatch) and to the workspace members.

- [x] **Placement on `Element` nodes (`plugin-markdown`).** `resolve_placement`
      now reads the hidden `__plugin_kind` attribute for `Element` nodes exactly
      as it already did for `Custom` nodes (§11 rule 3). `plugin-directive`
      renders an unclaimed type as `<div>` / `<span>` (`D8`), so its body was
      passed through verbatim and the Markdown inside it leaked — a list inside
      a `==type … ==` block stayed raw text. `block` / `codefence` / `blockquote`
      now re-lex the body as blocks (closing open quotes/lists/tables first,
      like any other block node) and `inline` re-lexes it in place; only a node
      that declares `element` or nothing at all (the pre-rendered `figure` /
      `table` containers of `plugin-img` / `plugin-table`) stays verbatim.
- [x] Four new `plugin-markdown` unit tests pin the contract
      (`resolve_placement_reads_the_hidden_attribute`,
      `element_block_directive_body_is_relexed`,
      `element_inline_directive_body_is_relexed`,
      `element_without_plugin_kind_stays_verbatim`,
      `element_placement_keeps_pre_rendered_children`) and
      `crates/plugin-markdown/README.md` documents it under _Notes_.
- [x] `sandbox/list/inside/out/inside.jsx` regenerated: the two `==info[Lorem
      Ipsum]` directives now render `<ul>` / `<ol>` (nested three deep) and
      `<p>` instead of the raw Markdown text; `sandbox/unified/out/directive.jsx`
      (WIP fixture) regenerated too — the nested `<p>` inside `<span>` is gone.
- [x] No other fixture moves: `sandbox/ultimate/{default,full,subset}` are byte
      identical before and after the change.
- [x] **Decorator binder** (`crates/extra/src/decorator.rs`, §9.1).
      `parse_decorator_line` recognises a decorator line (its _entire_ content is
      a head, §4.3 keeps the rest literal) and `bind_decorators` binds the lines
      of a stream to the next block node: the run's **last** line wins (earlier
      ones are dropped as `Overridden`), a decorator indented deeper than the
      block stays literal text, a decorator with no following block is dropped as
      `NoFollowingBlock`, and the paragraph the core parser wrapped the line in is
      consumed with it. `target` is a caller callback, so the plugins own _what_ a
      decorator attaches to. 12 unit tests.
- [x] **`crates/plugin-blockquote`** (§9.2). Binds the inner head
      (`> @@type{…} content`, space after `>` and after the head allowed, §4.1)
      and a decorator line directly above the quote, the inner head winning. The
      paragraph is replaced by `Custom(name)` when `[[task.blockquote.custom]]`
      claims the `type` (§11 rule 3) and by `Element("blockquote")` otherwise
      (D8), marked `__plugin_kind = "block"` so `plugin-markdown` re-lexes the
      stripped body. 8 unit tests + 4 end-to-end tests
      (`tests/pipeline.rs`, core → blockquote → markdown → solid).
- [x] **`crates/plugin-list`** (§9.3/§9.4) — **L1, the container decorator, now
      end-to-end.** A decorator line above a list wraps the list in the container
      node (`Custom(name)` / `Element("ul"|"ol")`, `__plugin_kind =
      "unordered" | "ordered"`) and leaves the marker lines for `plugin-markdown`;
      the layer follows the marker, never the decorator's own type (§9.4). 6 unit
      tests + the 7 pipeline tests that pin the merge (below). L2/L3 (the `list` /
      `<li>` item layer), L4 (`start`) and L5 (nesting) are documented as not
      implemented in `crates/plugin-list/README.md`.
- [x] Both crates are workspace members; `cargo build --workspace` is clean.
- [x] **`plugin-markdown`: the list-container merge (§9.3 L1 + §9.4).** A node
      that declares `__plugin_kind = unordered | ordered` is a _container_
      placement: `plugin-list`'s wrapper is the list's container, so the list is
      built **inside** it (no `<ul>`/`<ol>` of its own) and the extras land on the
      container — or on the component that replaces it. The `start` offset of an
      ordered list is preserved, a nested list still opens its own element, a
      decorator that ends up in front of content that is not a list keeps its node
      (the extras are never dropped), and a decorated list never leaks into the
      next list. 4 unit tests + 7 end-to-end tests (`crates/plugin-list/tests/
      pipeline.rs`: core → list → markdown → solid).
- [x] Two `plugin-markdown` fixes the merge needed: a list item is closed through
      `emit_end`, so a block wrapper around a list (a directive, a decorated
      container) no longer emits a second `EndNode(ListItem)` for a stale stack
      entry; and the wrapper's own `EndNode` closes the list frame it became the
      container of, so a following list cannot continue inside the closed
      container. No fixture moved: every `sandbox/**/out*` file re-renders byte
      identically (`sandbox/list/inside` included).
- [ ] **Execution plan (ACTIVE).** The Phase-4 work of §9.2–§9.4, wired and
      gated end to end. Steps are ordered; each has its own gate. § = section
      of `docs/spec/SYNTAX.md`.

  - [x] **Step 1 — CLI wiring.** `apps/cli/Cargo.toml` gains
        `pendon-plugin-list` + `pendon-plugin-blockquote`; `config.rs` gains
        `ListTaskConfig` / `BlockquoteTaskConfig` and the `task.list` /
        `task.blockquote` fields; `plugins.rs` gains `build_list_options` /
        `build_blockquote_options` (layer sets via `layer_set`:
        `list` / `unordered` / `ordered`, `blockquote`) and pushes `solid_hints`;
        `process.rs` gains the `"list"` / `"blockquote"` dispatch arms before
        `"markdown"`. Gate: `cargo build`, the two plugin suites, and a manual
        `/tmp` render of a decorated list + quote. **Done.**
  - [x] **Step 2 — fixtures + goldens (L1).** New `sandbox/list/decorator/`
        fixture; goldens `11-blockquote` + `12-list-container` with tests (no
        `#[ignore]`); the two goldens use the canonical **touching** decorator
        form (§9.1) and `sandbox/ultimate` `Q01`–`Q24` / `L01`–`L46` are un-frozen
        (`out.frozen/` re-frozen for all three tasks; `syntax_spec` 15/15 and
        `ultimate_freeze_spec` green). **Done, plus a binder fix it surfaced:**
        the core wraps a blank-line-delimited chunk in ONE `Paragraph`, so the
        canonical spelling where a decorator line _touches_ its block
        (`@@{.u}` immediately above `- one`) shared a paragraph with the block
        and was silently treated as text. `crates/extra/src/decorator.rs` now
        splits a paragraph's leading decorator run off (`split_leading_decorator_paragraphs`)
        before binding, so the touching and blank-line spellings behave the same;
        4 new binder tests. `sandbox/list/inside/out/inside.jsx` re-frozen too.
  - [ ] **Step 3 — L2/L3 item layer.** `plugin-list` item pass emits an item
        wrapper; `plugin-markdown` gains `CustomPlacement::ListItem` +
        `PendingListItem` / `arm_list_item` / `take_list_item`; L4 `start:`
        rejection warning; L5 nesting test; golden `13-list-item`.
  - [x] **Step 4 — §9.1 general block decorators.** `plugin-markdown` calls
        `extra::decorator` for paragraph / fence / heading / table; merge
        placements `Paragraph` / `CodeFence` / `Heading` / `Table`; verify the
        warning + indentation rules; golden `10` green. **Done:** `markdown_target`
        accepts a `Paragraph` / `CodeFence` and declines a list- or
        quote-opening paragraph (a `Heading` was dropped in the §9.5 scope
        correction below), so `plugin-list` / `plugin-blockquote` keep
        theirs; the pending decorator is armed on the bound block and flushed at
        each Paragraph / Heading / CodeFence / Table start (the Table flush lands
        after `StartNode(Table)` and before `StartNode(TableHead)`); a decorator
        that bound to nothing emits a `[markdown]` warning. Golden
        `10-decorator-blocks` frozen (`syntax_spec` 16/16). A §9.1 subtlety the
        table path surfaced: a bare `@@`-less `{…}` head with **no following
        block** is ambiguous with literal text (a cell holding `{foo}`), so it now
        stays literal (`DecoratorLine::explicit`) while the `@@…` forms are still
        dropped with a warning; one binder unit test added.
  - [ ] **Step 5 — plugin-markdown gaps.** `Q14` fenced code in a blockquote;
        `parse_blockquote_prefix` indentation + blank-`>` split; re-freeze
        `sandbox/list/inside` with a reviewed diff.
  - [ ] **Step 6 — docs.** SYNTAX.md §9.2/§9.3 normative, §16.2/§17.4
        status, §13 `start:`, §14 `o.`, §15 OPEN-L1 resolved, §18 crit 4;
        `TODO.md` Phase M closed; crate READMEs.
  - [ ] **Step 7 — Phase-4 cleanup.** OPEN-R1 `:::` deprecated alias with
        warning; archive `docs/rfc/unified-syntax.md`; rewrite `sandbox/unified`;
        trim `docs/todo/syntaxes.md`.
  - [ ] **Step 8 — gates + commits.** build 0 warnings, fmt, dprint, full test
        green; logical commits (cli wiring / §9.3-9.4 / goldens+refreeze /
        markdown fixes / docs).
- [ ] `plugin-markdown`: fenced code inside a blockquote (`Q14`), listed as a
      known gap in its README. (tracked as Step 5)

### Gaps found in `sandbox/list/inside` (blockquote, not directives)

Two further defects the fixture exercises, both pre-existing in `plugin-markdown`
and **not** touched here (they need a decision — see below):

- a nested item inside a quote loses its nesting:
  `> - a` / `>   - b` / `> - c` renders three sibling `<li>`;
- a blank `>` line splits one list into two `<ul>`:
  `> - p` / `>` / `> - q` renders two lists instead of one loose list.

Root cause of the first: `helpers::parse_blockquote_prefix` consumes **every**
space after a `>`, so the indentation of `>   - b` is lost before the list
indent comparison in `text.rs` runs (`leading_spaces` is 0 for both items).
The second needs loose-list state to survive a blank quote line.

### Design decision (decorator binder) — settled

§9.1 says a decorator line "decorates the **next block node**", §9.3 gives the
lists three layers (`unordered` = `<ul>`, `ordered` = `<ol>`, `list` = `<li>`)
and §9.4 mirrors them in config. The spec does not fix **how** a pre-Markdown
plugin attaches attributes to a node `plugin-markdown` is about to build. The
binder lives in `crates/extra/src/decorator.rs` (extras glue, next to
`bind.rs`); the plugins pass the `target` callback.

**Chosen: a wrapper node carrying `__plugin_kind = <layer>`** (`blockquote`,
`unordered`, `ordered`, `list`), whose attributes are merged onto the block
`plugin-markdown` emits for the body. `plugin-blockquote` already worked that way
(its wrapper _is_ the quote node); the container half of that rule has now landed
for lists too: `unordered` / `ordered` are a _container_ placement in
`plugin-markdown`, so the wrapper becomes the container of the list built inside
it (`plugin-list` L1 is complete end-to-end). The **item** layer (`list`,
L2/L3) still has to decide how a per-`<li>` decorator reaches the pass — the two
candidates are (a) `plugin-markdown` reading an item-decorator `@@type{…}` at the
start of an item's content through `pendon-extra`, or (b) `plugin-list` stripping
it and handing the pass the resolved attributes of that item. Rejected: a
`__plugin_kind = block` variant carrying the target kind — it would duplicate
what the node's own element already says.

## Plan — decorator target adjustment (Step 4 scope correction)

A decorator line above a block is bound only by the plugin that owns that
block's **own** extras. Decorators were therefore removed from **heading** and
**table** (both parse `@@type{…}` / `{…}` on the construct itself; a second
binding above them would be redundant) and kept for **paragraph**, **code
fence**, **list container** (§9.4) and **blockquote** (§9.2).

**Done.**

- [x] Removed `Event::StartNode(NodeKind::Heading)` from `markdown_target`
      (`crates/plugin-markdown/src/lib.rs`); paragraph and code fence stay accepted,
      a table stays declined (`_ => None`) and a heading is now declined too.
- [x] Rewrote the doc comments (§9.1 comment above `process_with_options` and the
      `markdown_target` doc) to the final target set.
- [x] Confirmed both binder invariants: a declined-target decorator stays in the
      stream (`crates/extra/src/decorator.rs`) and a decorator that bound to nothing
      warns + drops. A heading decorator now reaches `plugin-section`'s binder.
- [x] Golden `10-decorator-blocks`: dropped the heading case, re-frozen;
      `11-blockquote` / `12-list-container` unaffected.
- [x] Gates: `syntax_spec` (17/17) + full workspace green with
      `CARGO_INCREMENTAL=0`.
- [x] Docs: SYNTAX.md §9.1 / §16.2, `plugin-markdown` README, and the Step 4 note
      above.

## Plan — `plugin-sectionize` rework → `plugin-section`

Record of `docs/rfc/plugin-section.md`, approved in principle. **Done.**

1. [x] Renamed `crates/plugin-sectionize` → `crates/plugin-section`, plugin name
       `sectionize` → `section`: CLI dispatch (`apps/cli/src/process.rs`,
       `plugins.rs`), `config.rs`, READMEs, SYNTAX.md, and every task list
       (`sandbox/heading`, `sandbox/ultimate`, `sandbox/unified`, `sandbox/universal`).
2. [x] Rewrote the behavior: each heading opens a `Section` at its level (a
       deeper heading nests; an equal/shallower one closes first), with the id from
       the head chain.
3. [x] Section extras: the decorator line **above** the heading binds to the
       section through `pendon-extra::bind_decorators` (the heading keeps its own
       extras on the `#` run).
4. [x] Level markers: `<--->` deepens by one nested section (capped at level 6),
       `>---<` closes the innermost section (no-op on the preface). **Pipeline
       decision:** `section` runs _before_ `markdown` and `heading`, because `>---<`
       would otherwise be a blockquote and the head stripped before the id chain
       could read it. Markdown parses the interior of each `Section`.
5. [x] Config: `[task.section.custom.section]` (`name` / `template` / `imports`),
       wired like heading/table in `apps/cli/src/config.rs` + `plugins.rs`.
6. [x] ID priority: `#sectionID` > `` `slug-section` `` > `` `slug-head` `` >
       `` `slug-head-extras` `` > title slug. A heading's own extras `#id` is
       **ignored with a warning** (it is not in the RFC chain). The heading yields
       its `id` / fallback `slug` when `section` is enabled
       (`HeadingOptions::section_owns_id`), so the id transfers to the section.
7. [x] Evidence: new golden `22-section`; re-froze `sandbox/heading` +
       `sandbox/ultimate` (`out.frozen/`); SYNTAX.md §9.5 / §16.2 / §16.3 / §17.4;
       READMEs; this checklist.

**Notable re-freeze changes** (`sandbox/ultimate`): the heading id moves from the
`<h*>` / `HeadingDefault` node onto the wrapping `<section>`; a heading's extras
`#id` is now ignored (so `H07`–`H09`, `H25` ids became their title slugs); a
heading inside a blockquote/directive is no longer given its own `Section` (it
belongs to the enclosing section); content before the first heading opens a
preface `Section`. The `extract-heading` export reads the section id, so it keeps
the explicit slugs (`s-top`, `h01`, …).

**extract-heading × custom heading templates**: `plugin-heading` swaps
`NodeKind::Heading` for `NodeKind::Custom(name)` when a `[task.heading.custom]`
template is configured, so `extract-heading` (which only scanned `Heading`) found
no headings and emitted no `Headings` export at all. It now recognises those
components by their `level` attribute — heading's signature — and the look-ahead
skips the §13 `Diagnostic` warnings pushed before the attribute run. §11 rule 5
later made `level` ambiguous: a directive entry may rename a positional slot to
it (`parentheses_key = "level"`), so a `Custom` node that also carries the
internal `__plugin_kind` marker (emitted by `plugin-directive`, `plugin-custom`
and `plugin-section`, never by `plugin-heading`) is excluded — otherwise
`::note(…)` was harvested into the `headings` export. It scans to the heading's
end by counting only
same-kind nodes (not every `StartNode`, which let an unbalanced non-heading node
inside a heading over-run into the following blocks). The id chain also reads the
component's auto-`slug`. Verified: `sandbox/heading/out/heading.jsx` regains
`export const headings`; `sandbox/ultimate/out.frozen/full` re-frozen (2 → 6
top-level headings); `default`/`subset` byte-identical.

## §11 positional keys + empty-attr omission (done)

- `PositionalKeys` (`crates/extra/src/typed.rs`, all-`Option` plus
  `resolve()` / `is_empty()`, serde) is the config-side form of
  `ExtrasOptions`; `ExtrasOptions` gained `bracket_key` / `parentheses_key`
  (defaults `slug` / `title`).
- `KeyResolver` (`&dyn Fn(Option<&str>) -> ExtrasOptions`) replaced the
  `&ExtrasOptions` parameter of `parse_decorator_line` / `bind_decorators`, so
  each plugin resolves keys per `type` marker: exact entry → its keys, else the
  layer default, else the built-in `slug` / `title` (§11 rules 3 and 5).
- Wired through: blockquote, section, list (3-layer `keys_for_opt` chain),
  markdown (its blocks carry no §11 set, so it stays on the built-in keys),
  anchor, heading, marker, img, cite and directive — directive additionally
  maps `[bracket]` / `("paren")` through `bracket_key` / `parentheses_key`.
- Empty string (`k: ""`) is omitted from plain-element attributes and
  `{...attrs}` spreads in `renderer-solid` / `renderer-html`, except `alt`,
  whose empty value is meaningful (§6.3). Explicit template bindings
  (`k={attrs.k}`) are untouched and still render `k={""}`.
- `extract-heading` no longer mistakes a directive for a heading (see the note
  above), so `sandbox/ultimate` no longer harvests `::note(…)` into `headings`.
- Re-frozen: golden `05` (`href=""` dropped), golden `15` (all four keys
  exercised), `sandbox/ultimate/out.frozen/{default,full,subset}`.
  `sandbox/directive` and `sandbox/heading` outputs are unchanged.
- Test coverage: `crates/extra/tests/extras_spec.rs`
  (`positional_key_overrides_resolve_unset_keys_to_the_defaults`) and
  `crates/plugin-directive/tests/pipeline.rs`
  (`each_type_resolves_its_own_positional_keys`).

### Deferred from this pass

- **`plugin-table`**: `extras_to_layer(head)` in
  `crates/plugin-table/src/attrs.rs` still calls
  `to_attributes(head, &ExtrasOptions::default())`. It owns 7 layers
  (`TABLE_LAYERS`) and the wiring is invisible to every fixture, so it needs a
  follow-up threading `TableOptions` into the layer head scan.
- **`plugin-wiki`**: `WikiOptions` carries no `ComponentSet`, so there is
  nothing to resolve keys against; its extras stay on the built-in
  `slug` / `title`. Adding a `[task.wiki.custom]` table would lift this.
- **`extract-heading`**: `process(events)` takes no options, so its `#id` >
  extras-`slug` fallback still reads the built-in key. Thread the heading
  options in if a `[task.heading.custom]` entry renames `backtick_key`.

## Decorator & marker positional groups — `@@type[…](…){…}` (ACTIVE)

Option 2 (confirmed): the `[…]` / `(…)` positional groups sit **between the
type and the body**, so a decorator line still starts with `@@` and a plain
`[x](y)` paragraph line can never be mistaken for one (omitting `@@` ⇒ link, by
design). Markers take their groups directly after `}}`:

```text
@@type[…](…){extras}      @@[…](…){extras}     untyped decorator
{{type}}[…](…){extras}    {{type}}[…](…)@@head{extras}
```

Confirmed decisions:

- [x] Option 2 for decorators; `@@[…]` for an untyped decorator; a line that
      omits `@@` stays a link/paragraph — no scanning for `[` at line start.
- [x] The groups parse in the **shared `read_head`** (`crates/extra/src/typed.rs`),
      so every `@@head` accepts them (uniform grammar; a repo grep found zero
      existing `@@type[` / `{{type}}[` patterns — no meaning changes).
- [x] Marker order: groups right after `}}`, **before** any extras head
      (`{{}}[…] (…)@@{}` order). They map through `keys_for(marker type)` →
      `bracket_key` / `parentheses_key` (§6.1, same as directives).
- [x] Key collision: the **positional group wins** over a same-key `{…}` item;
      the dropped value is reported with an §13 `Overridden` warning (§6.2
      head-wins style).
- [x] Malformed group (`[` or `(` without its closer) ⇒ `UnterminatedHead` ⇒
      the head/line stays literal text (§4.3), matching directive behaviour.
- [x] Emission order (§6.4): `class`, `id`, then the positional keys —
      `bracket_key`, `parentheses_key`, `backtick_key`, `quote_key` — then the
      remaining props/flags in source order.
- [x] Layer order (found while verifying against the parser, now §6.1): the
      groups belong to the **layer that runs first** in the task plugin list. A
      `[x]("y")` pair is also a legal anchor head, so under the canonical order
      (`anchor` before `marker`) the anchor claims it and the marker keeps no
      attribute from it (`{{markerA}}[g]("T")` renders a bare marker plus an
      `Anchor` sibling); swapping the two entries in a copy of the sandbox
      config hands the marker both groups (`slug` + `title`). Single-group and
      type-only forms parse in either order. No config or golden changed: the
      sandbox keeps the canonical order and the rule is now spec text.

Steps (ordered, each with its own gate; § = `docs/spec/SYNTAX.md`):

- [x] **Step 1 — shared parser** (`crates/extra/src/typed.rs`). Done:
      `ExtrasHead.bracket` / `.parentheses`, the factored
      `read_positional_groups(&str)` called from `read_head` after the type run
      (before `{`, or as the head itself when no `{` follows) and the
      `to_attributes` mapping through `bracket_key` / `parentheses_key` with
      group-wins precedence and the §6.4 order. Gate green:
      `cargo test -p pendon-extra`.
- [x] **Step 2 — marker** (`crates/plugin-marker/src/lib.rs`). Done:
      `scan_marker` reads the adjacent `[…]` / `(…)` after `}}` before
      `scan_extras_chars`, an `Overridden` warning fires when the group and the
      extras head fill one key, a group-only marker still gets its attributes
      and a malformed group leaves the marker rendered with the group as literal
      text. Gate green: `cargo test -p pendon-plugin-marker`.
- [x] **Step 3 — decorator tests** (`crates/extra/src/decorator.rs` +
      `crates/extra/tests/extras_spec.rs`). Done:
      `positional_groups_make_a_decorator_line`,
      `a_group_decorator_binds_its_attributes`, `heads_carry_positional_groups`,
      `groups_must_be_adjacent` and `groups_map_through_the_positional_keys`
      cover `@@type[…](…){…}`, `@@[…]`, type+group without a body, a plain
      `[x](y)` line staying a link, a malformed group staying literal and the
      collision warning. Gate green: `cargo test -p pendon-extra`.
- [x] **Step 4 — docs.** Done: SYNTAX.md §6.1 (groups on every head + the
      layer-order rule), §6.2/§6.4 (order + group-wins), §9.1/§9.2 (decorator
      lines, including `@@[…]`, and the blockquote inner head), §10.1 (marker
      forms) and the §17.5 progress note; the `marker`, `blockquote`, `list` and
      `section` READMEs name the group forms.
- [x] **Step 5 — goldens.** Done: fixture 10 carries the typed
      `@@aside[intro]("Aside title"){.lead}` line and the untyped `@@[intro]("…")`
      spelling, fixture 14 carries `{{bibliography}}[refs]("Cited sources"){…}`
      and the malformed-group case; both `.md` / `.jsx` pairs were re-rendered
      with `pendon run -F` in a fixture copy, the `.toml` files were untouched,
      and the §16.2 fixture table was updated. Gate green:
      `cargo test -p pendon --test syntax_spec` (17 fixtures).
- [x] **Step 6 — ultimate sandbox (groups).** Markers: M13–M19 appended to the
      M section (`[x]` → `slug`, `("x")` → `title`, group + extras head, a
      malformed group, the anchor layer winning an adjacent pair, an unclaimed
      type, and one inline pair). Inner head: Q25/Q26 in the Q section, placed
      **before** Q24 so the pre-existing subset swallow cannot splice into them.
      The section comments document the §6.1 layer-order rule, and the stale
      "blockquote/list pending Phase 4" notes in the sandbox header and
      `pendon.toml` were corrected (the truncation note below was added too).
      All three tasks were re-rendered with `pendon run -F`, the diff reviewed
      (purely additive apart from one deliberately extended comment line) and
      `out.frozen/<task>/` re-frozen. Gate green:
      `cargo test -p pendon --test ultimate_freeze_spec`.
- [ ] **Step 6b — ultimate sandbox (list container).** L53/L54/L55 (a container
      decorator with a bracket group → slug, an unclaimed container type, and a
      pair lost to the anchor layer) are **blocked by the L37 truncation**: every
      case from L40 on is missing from the rendered body (verified by rendering
      them — they never appear). Add them once the tail renders again; the same
      fix is what makes the D/B/N/E sections reachable.
- [ ] **Step 7 — full gates.** `cargo test --workspace` (green, log
      `/tmp/final_test2.log`), `cargo build --workspace`, `cargo fmt --check`
      (green) and a `git diff` review of every golden / baseline move.

## Sandbox ultimate: the rendered body stops at L37 (OPEN, found 2026-10-08)

`sandbox/ultimate/out*/**/ultimate.jsx` ends after the L37 list item in all three
tasks: L40–L46, D, B and N plus the whole E section never render, while the
`headings` export still lists them. Minimal repro — a heading inside a list item
whose text carries an extras head:

```text
##[s-a] V1

- item

  # [x]("T"){.d}body

D01 after
```

Subset task (`plugin = "micromatter,heading,table,anchor,markdown"`) renders
`<li>item<h1>xbody\nD01 after\n</h1></li>`: the head scan runs past the end of
the heading's line and the swallowed text is re-emitted inside the `<h1>`. The
same scan explains Q24 in the subset task (`> {.q24} KSu …` loses everything
after `KSu`). In `sandbox/ultimate` the trigger is
`# ===asideA[s-l37]("L37"){.d}body` (L37, §9.3), which eats the rest of the
document — so the frozen baseline currently encodes a truncated body. The fix
belongs to the shared head reader (`crates/extra`) and the plugin that scans it
(`plugin-anchor` / `plugin-heading`); afterwards re-render, review the newly
visible sections, re-freeze, and add the regression case to the E section (which
is inside the truncated region today).

## Deferred to Phase 4

- `plugin-list` and `plugin-blockquote` now exist: both are wired in
  `apps/cli/src/plugins.rs` and configured in `sandbox/ultimate/pendon.toml`
  (Tasks 1 and 2; Task 3 leaves them out, so the Q/L syntax stays literal text
  there). What is still missing is the **list item** layer (L2/L3) — the
  container layers (`list` / `unordered` / `ordered` and the blockquote layer)
  landed with the plugins.
- The Q and L sections of `sandbox/ultimate/src/ultimate.md` render as
  quotes/lists in Tasks 1 and 2 now; their L40+ cases are unreachable because
  the rendered body stops at L37 (see the L37 entry above).
- Golden `13` (`list-item`, L2/L3) stays open for the same reason; `10`–`12`
  (decorator, blockquote, list container) are now frozen and green (§16.2) and
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
