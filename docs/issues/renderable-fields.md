# Renderable Fields

## Working Terminology

- Rough illustration: an illustration created to roughly convey intent and expectations; it is non-binding and non-mandatory, may be incorrect or imprecise, and should be adjusted to match the current Pendon implementation or a better one.

## Problems and Goals

- Many users strongly need Pendon's ability to send fields as fully rendered JSX. `plugin-bind` already handles this very well. However, relying solely on `plugin-bind` can sometimes be less ergonomic and impractical in certain cases, especially when the intended code is actually short and entered repeatedly.
- This is why _renderable fields_ are introduced.

## General Mechanics

- Syntax:
  - `;;foo;;` for inline mode,
  - `;;;foo;;;` for block mode,
  - `\n` can be used for line breaks, since slots and specs do not support multi-line content.
- Values enclosed in `;;` or `;;;` are rendered as full `mdp`, just as if they were placed as `children`.
- Renderable fields can ONLY be used in:
  - slot contents (bracket & paren): bare, quote, tick
  - specs: quote, tick, `k: v`
- Renderable fields CANNOT be used in:
  - specs: `#id`, `.class`, style, `isFoo`
  - outside of slots and specs
- Renderable fields that appear within plain literal text (outside of heads) are treated as nothing special and are rendered as plain literal text.
- Renderable fields nested inside another renderable field value can still be rendered normally without issues. Pendon does not enforce a hard limit on the maximum nesting depth of renderable fields.
- `%%number` and internal numbering (`citation_number` from `plugin-cite`) MUST continue to function correctly. Counting is based on where the relevant element/head appears in the original `mdp` source (the expected standard behavior), regardless of its order in the actual output.
- Renderable fields MUST NOT be bind definitions `{{{lang[var]...}}}`; they will fall back to literal text. However, specs using `k: $bind` inside renderable fields can consume `$bind-var` properly as expected.

## "Rough" Illustrations

```mdp
{{markerX}}[;;lorem *ipsum* **dolor** sit;; ";;;lorem @[ipsum](/ipsum) dolor sit;;;" `;;{{markerAuthor}}{ name: ;;John Doe [^article-foo];; };;`]
```

expected output (rough illustration):

```jsx
<MarkerX note={(<>lorem <em>ipsum</em> <strong>dolor</strong> sit</>)} alert={(<><p>lorem <a href="/ipsum">ipsum</a> dolor sit</p></>)} bracket={{ tick: (<><MarkerAuthor name={(<>John Doe <Cite reference_id="article-foo" citation_number={1} /></>)} /></>) }}/>
```

```toml
[[task.marker.custom]]
type = "markerX"
name = "MarkerX"
template = "<MarkerX {...attrs} />"
field.bracket.bare = "note"
field.bracket.quote = "alert"
field.paren.bare = "author"
field.bracket.quote = "author"
```

Renderable fields nesting:

```mdp
{{{yaml[data-x]
location: Jakarta
}}}

##[`;;;Lorem [^page-baz]{ quote: ;;;> Flower is violet\n\n> Violet is blue;;;, details: { title: ;;*Blue* Flower ![](/flower.png ";;this is ::direY blue flower:: illustration [^page-wax];;");;, dataX: $data-x } } ipsum.\n\n===DirectiveX\n\nSit amet [^page-baz]\n\n===;;;`] Heading Soo

Ullamco nisi ea ;;ullamco;; consectetur ;;;deserunt;;; do qui voluptate.
```

Rough illustration of output:

```jsx
<Heading depth={2} slug="heading-soo" bracket={{ tick: (<><p>Lorem <Cite reference_id="page-baz" citation_number={2} quote={(<><blockquote><p>Flower is violet</p><p>Violet is blue</p></blockquote></>)} details={{ title: (<><em>Blue</em> Flower <Img alt="" src="/flower.png" paren={{ quote: (<>this is <DirectiveY>blue flower</DirectiveY> illustration <Cite reference_id="page-wax" citation_number={3} /></>) }} /></>), dataX: { location: "Jakarta" } }} /> ipsum</p><DirectiveX>Sit amet <Cite reference_id="page-baz" citation_number={2} /></DirectiveX></>) }}>Heading Soo</Heading>

<p>Ullamco nisi ea ;;ullamco;; consectetur ;;;deserunt;;; do qui voluptate.</p>
```

## Notes

- If `;;` or `;;;` is left unclosed, the behavior reverts to its original default (for example, if inside slot content, it becomes a plain string value; for specs, I have not yet verified the implementation—it seems the entire spec will fall back to literal text, because renderable-field values in specs are not wrapped in `"..."`, which would cause a syntax error).
  - **Verified** (see [Verified ground truth](#verified-ground-truth-code-not-guesswork)): the guess is right. An unquoted `k: ;;a b` hits `ExtrasError::UnquotedWhitespace` (`crates/extra/src/typed.rs:982`), and a malformed head falls back to literal text as a whole (§4.3) — so it is all-or-nothing, not a half-parsed spec.
- Golden tests for renderable fields will eventually need to include `%%number`. I have not yet created an example for that.

## Verified ground truth (code, not guesswork)

The Open Questions at the bottom asked whether Pendon already has the machinery.
It does, in three pieces:

- **A rendered value is a fragment, not a string.** `plugin-bind` renders an
  `mdp` payload through the host callback `MdpRenderer`
  (`crates/plugin-bind/src/lib.rs:53`; `MdpMode` at `:35`) and stores
  `renderer_ast::fragment(nodes)` — `{"$fragment": […]}`,
  `pendon_renderer_ast::FRAGMENT_KEY`. The CLI's callback is
  `chain::render_mdp` (`apps/cli/src/chain.rs`), which parses the payload, runs
  **the same chain** against the **same `ChainState`** (`Document::Payload`), and
  turns the events into children.
- **A structured attribute already has a transport.** `JSON_ATTR_PREFIX`
  (`\u{E001}`, `crates/core/src/event.rs:148`) + compact JSON;
  `renderer-ast` re-hydrates it into a real JSON value
  (`crates/renderer-ast/src/builder.rs:196`), Solid emits `name={(<>…</>)}`
  for a fragment (`crates/renderer-solid/src/node.rs`), HTML renders the markup
  into the attribute, AST/JSON keep the node list.
- **The resolve pass already walks every attribute.**
  `plugin-bind::resolve_with` (`crates/plugin-bind/src/lib.rs:1003`) maps over
  `Event::Attribute`; a bare `$mdp` reference becomes a fragment (`bound_value`,
  `lib.rs:1072`).

So renderable fields need **no new renderer** and **no new fragment format**.
What is missing:

1. **No syntax.** `;;` / `;;;` is plain text today. `crates/extra` has no
   notion of it, and the slot/spec lexers treat `,` `"` `` ` `` `[` `]` `{` `}`
   inside it as normal delimiters.
2. **No transport.** `AttrValue` (`crates/extra/src/value.rs:20`) has no
   "this is mdp" variant, so nothing can carry mode + source to the resolve
   pass.
3. **No resolve.** Only `plugin-bind` renders payloads, and only when `bind` is
   in `task.plugin`. There is no pass that renders a value that came from the
   grammar.
4. **`$bind` inside a payload is not resolved today.** `render_mdp` runs the
   chain but never calls `resolve_with`, and `resolve_with` treats a fragment as
   opaque (it only walks top-level attributes). The issue requires
   `k: $bind` inside a renderable field to work, so `render_mdp` must resolve
   bind for the payload it renders.
5. **`plugin-extract` reads `Event::Attribute` raw strings**
   (`crates/extract/src/scan.rs:114`), so it sees whatever the attribute holds
   at its position in the chain. It must run _after_ renderable resolution or
   it will read an unresolved `MDP_ATTR_PREFIX` string.

Also verified (answers the Notes / Open Questions):

- **Unclosed `;;` inside a spec.** `parse_scalar` rejects unquoted whitespace
  (`ExtrasError::UnquotedWhitespace`, `crates/extra/src/typed.rs:982`), and a
  malformed head falls back to literal text (§4.3, `ExtrasMatch::Malformed`).
  So the whole head is literal, as the issue suspected.
- **`\n` is not in the escape set.** `slot::ESCAPABLE`
  (`crates/extra/src/slot.rs:334`) has no `n`, so `\n` survives slot parsing as
  backslash + `n`; the renderable scanner must convert it.
- **`emit_attrs` uses `literal()`, not `transport_for`**
  (`crates/extra/src/emit.rs:41`, whose `value.literal()` is at `:47`), so any
  structured value emitted through a decorator (`plugin-list`,
  `plugin-markdown`, `plugin-wiki`) would lose its sentinel. This is a latent bug
  the renderable variant would expose.

## Design decisions (proposed)

**D-RF-1 — The grammar layer owns detection; the CLI owns rendering.**
`crates/extra` must not depend on a renderer or on `ChainState`. It stores
`MdpMode` + source in the typed value and transports it with a sentinel; the CLI
resolves it. This keeps the layer model (`CONTRIBUTORS.md`) intact.

**D-RF-2 — `;;` / `;;;` is an opaque span in every head lexer.** A renderable
value is "full mdp", so it may contain `,` `"` `'` `` ` `` `[` `]` `{` `}`
`(` `)` `:` and whitespace. Every scanner that reads slot/spec text
(`slot::scan_content`, `read_bare`, `read_delimited`, `find_close`,
`typed::split_items`, `split_key_value`, `find_group_close`) skips an opened
`;;` / `;;;` span as one opaque unit. Doing it in one place (a
`renderable::span` helper) is the only way "rendered as full mdp" holds.

**D-RF-3 — Longest opener wins; a `;` run of at least that length closes.** At a
content/spec start, `;;;` is block, otherwise `;;` is inline. Inside a span,
`\` escapes the next character, and a run of `;` whose length is **≥** the
opener's closes it and is consumed whole (so `;;;` cannot leave a stray `;` in a
`;;` span). `\n` becomes a newline. An unclosed span is **not** an error: the
value falls back to its pre-renderable literal.

**D-RF-4 — A renderable occupies a whole slot content or a whole spec value.**
`[;;…;;]` is renderable; `[pre ;;x;; post]` is literal text. `k: ;;…;;` is
renderable; `k: "…;;…;;…"` is a string. This removes the greedy "starts with
`;;` but is prose" ambiguity.

**D-RF-5 — One `MdpMode` type.** Today it lives in `plugin-bind`. Move it to
`pendon_core` (next to the attribute sentinels) and update every use — no
re-export (D-16). `pendon_core` has no dependencies, so the sentinel payload is
`mode byte + source` (`'i'` / `'b'`), not JSON; `pendon_extra` decodes it.

**D-RF-6 — No structural _positions_.** A renderable is legal in **all six slot
contents and every spec value**, including the positions that by default feed a
construct's own field: cite's `reference_id`, heading's `slug`, image's `src`,
`depth`. `field.*` lets the author move a construct's meaning elsewhere and put
their own value in the old position (§6.1, ADR-0014), so the grammar cannot know
which position is "structural" — refusing one would make a value's legality
depend on which element claimed the head. A plugin that needs literal text reads
`AttrValue::literal()`, i.e. the **raw `;;…;;` text** (nothing silently
disappears), and reports exactly what it reported before (cite's existing §13
error because the reference cannot resolve).

The genuinely unavailable positions are the ones that are **not field values**:
`#id`, `.class`, `isFoo`, `--*`, and `class:` / `style:` props. They keep the raw
`;;` text as a literal. `.class` / `#id` have a _name_ grammar, so a span cannot
even be spelled there — the head stays literal (§4.3), as it does today.

**D-RF-7 — Resolution is a chain stage, not a renderer stage.** The pass runs
after every head plugin (so the attributes exist), before `extract` (so
extraction reads the fragment) and before the citation finalize (so a field's
citations join the page list). It is `chain::resolve_renderable`;
`render_mdp` calls it too, which gives nesting for free.

**D-RF-8 — Numbering is page-scoped (decided).** A field's `citation_number` and
`%%` markers **continue the page's scope**; a renderable is not a numbering
island (it is the page's sub-document, ADR-0007). Exact source order is Phase 3's
option **B** (reserve at the source position, reuse at render), because:
A cannot recurse into the chain (`DocumentContext::citation` is an
`Arc<Mutex<CitationContext>>`, and the pipeline runs while that lock is held, so a
nested chain run would deadlock); C is the rejected sub-document scope; D is
undefined while the chain's order is the author's (golden 27 puts `anchor` after
`cite`).

## Implementation plan

Every phase is independently shippable and testable. A phase is complete only
when its goldens are frozen and `cargo test -p <crate>`, then
`cargo test --workspace`, is green.

### Phase 0 — Lock the decisions (docs only)

- [x] Add `docs/decisions/0021-renderable-fields.md` recording D-RF-1…8, the
      rejected alternatives (per-head rendering; a second `;;` parser in each
      plugin), and the numbering decision from Phase 3.
- [x] Update `docs/spec/SYNTAX.md`: a new §6.7 "Renderable fields" (grammar,
      allowed/forbidden positions, the opaque-span rule, `\n`), linked from
      §6.3 / §5.2, and add `;;` / `;;;` to §3's sigil inventory.
- [x] Update `CONTRIBUTORS.md` only if the opaque-span helper crosses crates (it
      should not — it lives in `crates/extra`; it does not).
- [ ] Add a one-line status entry to `STATUS.md`.

**Acceptance:** the ADR is Accepted, §6.7 exists, and `dprint check` passes.

### Phase 1 — Grammar: detect, store, transport

**1.1 The span scanner (`crates/extra/src/renderable.rs`, new)**

- [x] `pub enum RenderMode { Inline, Block }` (or reuse `pendon_core::MdpMode`,
      D-RF-5) and `pub struct Renderable { pub source: String, pub mode: RenderMode }`.
- [x] `pub fn span(text: &str, start: usize) -> Option<Span>`, `Span { body:
      Range<usize>, end: usize, mode }`:
      longest opener (`;;;` → `Block`, `;;` → `Inline`); `\` escapes the next
      byte; a `;` run of length ≥ the opener closes and is consumed whole; `\n`
      becomes `'\n'` when the body is materialised (not while scanning).
- [x] `pub fn whole(text: &str) -> Option<Renderable>`: true only when the
      trimmed text is exactly one span (D-RF-4).
- [x] Unit tests: inline / block, `;;;` inside `;;`, `\` escape, `\n`, unclosed,
      empty (`;;;;`), and "an opener only at a whole-value boundary".

**1.2 Slots (`crates/extra/src/slot.rs`)**

- [x] `SlotParts` stores `Option<SlotValue>` instead of `Option<String>`, where
      `SlotValue { text: String, render: Option<RenderMode> }`; `get` / `set` /
      `is_empty` / `SlotPair::slot` follow.
- [x] `scan_content` calls `renderable::span` first at each content cursor; on a
      hit it records `render: Some(mode)` and advances past the span.
      `read_bare` / `read_delimited` / `find_close` never see the interior.
- [x] `read_delimited` still owns the quote/tick delimiters; a `` `…` `` whose
      body is one span is `render: Some(..)`, not a string.
- [x] `parse_slot_parts` (the single-slot entry point) returns the same shape.
- [x] Tests (`crates/extra/tests/`): the six positions × inline/block; a span
      containing `,` `"` `[` `]` `{` `}` `(` `)` `:`; an unclosed span → the old
      `Str`; the bracket's `|…` run still works after a span.

**1.3 Specs (`crates/extra/src/typed.rs`)**

- [x] `split_items`, `split_key_value`, `find_group_close` skip an opened span
      (D-RF-2). This is what makes
      `{ title: ;;*Blue* Flower ![](/x.png "a, b");;, d: … }` parse.
- [x] `parse_scalar`: after unquoting a `"…"` / `` `…` `` body, run
      `renderable::whole`; a hit yields `AttrValue::Mdp { mode, source }`. For an
      unquoted value, `renderable::whole` runs **before** the
      `UnquotedWhitespace` check; a miss keeps today's behaviour.
- [x] `parse_value` keeps `{…}` / `[…]` dispatch; a renderable is never an
      object/array.
- [x] Enforce D-RF-6 in `to_attributes`: `class` / `style` props and anything
      reaching `push_class` / `push_style` / `set_declaration` convert `Mdp` back
      to a literal `Str` with its original delimiters.
- [x] Tests: `;;a, b;;` as a value, `k: ";;x;;"`, unclosed `;;`, a renderable in
      a nested object/array, `class: ;;x;;` staying literal.

**1.4 Value + transport (`crates/extra/src/value.rs`, `crates/core/src/event.rs`)**

- [x] `pendon_core`: `MDP_ATTR_PREFIX = "\u{E003}"`, `MdpMode` (moved from
      bind), `mdp_attr_payload(&str) -> Option<(MdpMode, &str)>`, `is_mdp_attr`.
- [x] `AttrValue::Mdp { mode, source }`: `literal()` → the raw source (the
      documented fallback); `transport()` / `transport_for()` → `MDP_ATTR_PREFIX`
      + mode byte + source; `to_json()` → `{"$mdp": {"mode": …, "source": …}}`;
      `from_json()` recognises it.
- [x] Fix `emit_attrs` to use `transport_for(name)` (latent bug above) and
      re-freeze any decorator golden that changes (expected: none).
- [x] Audit every exhaustive `AttrValue` match: `field.rs`, `preset.rs`,
      `plugin-image`, `plugin-cite`, `plugin-heading`, `plugin-directive`,
      `renderer-solid/components.rs`, `apps/cli/components.rs`.
- [x] Tests: `value.rs` round-trip (`literal` / `transport` / `to_json` /
      `from_json`); a `crates/extra` test that a slot renderable reaches `Attrs`
      as `Mdp`.

**Acceptance:** the issue's first illustration produces an `Event::Attribute`
whose value starts with `MDP_ATTR_PREFIX`, and no renderer sees `;;` yet.
`cargo test -p pendon-extra -p pendon-core` green.

### Phase 2 — Render: the resolve pass

**2.1 `chain::resolve_renderable` (`apps/cli/src/chain.rs`)**

- [x] `pub fn resolve_renderable(events: Vec<Event>, config: &ChainConfig, state:
      &mut ChainState) -> Vec<Event>`: for each `Event::Attribute` whose value is
      `mdp_attr_payload`, call `render_mdp(source, mode, config, state)`, replace
      the value with `JSON_ATTR_PREFIX + json(fragment.value)`, and append
      `fragment.diagnostics`.
- [x] A non-attribute event passes through untouched, so the pass is
      order-preserving and structural.
- [x] `render_mdp` already degrades an unbuildable tree to `Value::String(markdown)`;
      keep that.

**2.2 `render_mdp` renders nesting and `$bind`**

- [x] `render_mdp`: parse → `run_chain(…, Document::Payload)` →
      **`resolve_renderable`** (nesting) → `split_diagnostics` →
      `events_to_children`.
- [ ] Thread the bind `Registry` into `render_mdp` as an `Option<&Registry>`
      parameter (see the wiring bullet in 2.3 for why it cannot be a
      `ChainState` field) and call
      `pendon_plugin_bind::resolve_with` on the payload when `bind` is enabled.
      This is what makes `k: $bind` inside a field work (ground truth #4) and
      keeps `$mdp` nested binding working through `bound_value`'s callback.
      **Not done yet:** `$bind` inside _any_ payload — renderable or
      `{{{mdp…}}}` — is still unresolved, which is the pre-existing gap
      (`run_chain` skips `bind`); it is orthogonal to renderable fields.
- [x] A depth guard (`MAX_RENDER_DEPTH`, e.g. 32) shared by
      `render_mdp` / `resolve_renderable`; at the limit emit one
      `Event::Diagnostic` (`Error`) naming the attribute and leave the source
      literal. This is a **stack-safety** bound, not the grammar limit: the
      issue's "no hard nesting limit" is a _grammar_ promise, so 32 nested fields
      still render and only a cyclic `$mdp` reaches the guard (which must not hang
      the process).
- [ ] Unit tests in `chain.rs`: a citation inside a field joins the context;
      nested fields; a self-referential `$mdp` terminates with a diagnostic.
      **Partially done:** `apps/cli/tests/renderable_fields_spec.rs` covers the
      fragment, block structure, nesting and the literal fallback end to end; the
      `$mdp`-cycle case is still open (it needs the registry threading above).

**2.3 Where the pass runs**

- [x] `run_chain`: before the `extract` stage — and once at the end when
      `extract` is absent — run `resolve_renderable`. Every head plugin has run
      by then (`validate_extract_order` already forbids element plugins after
      `extract`), and `extract` reads the fragment.
- [x] `main.rs` / `process.rs`: keep `run` → bind `resolve_with` → citation
      finalize. `resolve_renderable` now happens inside `run_chain`, so the
      finalize sees the field's citations.
      **Invariant to document:** a renderable is resolved inside the chain, so
      `$bind` must be resolved inside `render_mdp` (2.2), not by the outer pass.
- [ ] **Registry wiring.** `bind.registry` already exists _before_ the chain runs
      (`apps/cli/src/process.rs:80`, `apps/cli/src/main.rs:68`), so thread it as
      `Option<&Registry>` through `chain::run` / `run_chain` / `render_mdp` —
      **not** as a `ChainState` field: `render_mdp` already borrows
      `&mut ChainState`, and putting the registry inside it would make the
      renderer unable to borrow both. (Still open — see 2.2.)
      Today `run_chain` skips `bind` (`chain.rs:293`) and never calls
      `resolve_with`, which is exactly why `$bind` inside a payload is
      unresolved (ground truth #4).
- [x] Update the three `resolve_with` call sites to one
      (`main.rs:154`, `process.rs:196`, plus the new in-chain one), or keep the
      outer call for `$mdp` attributes only — decide in the ADR so the same
      attribute is never resolved twice.
- [x] `apps/cli/tests/renderable_fields_spec.rs` (new, CLI-level): the issue's
      first illustration end-to-end (Solid); a `,` / `"` / `[`-heavy payload; an
      unclosed span staying literal; a field inside a decorator, a section, a
      table cell and a nested list.

**Acceptance:** the first illustration renders the issue's shape
(`note={(<>…</>)}`, nested `<Cite …/>`), and `cargo test -p pendon` is green.

### Phase 3 — Numbering and source order (decided: option B)

The requirement is explicit: `%%number` and `citation_number` count by **source
position**, not output position. The current machinery resolves a value _after_
every plugin has run, while `plugin-cite` / `plugin-number` number _during_ the
chain — so a citation in an early head that renders late would get the **next**
number, not the earlier one.

**Decision (D-RF-8): the scope is the page's; the ordering is option B.** The
rejected options are rejected for structural reasons, not taste:

- **(A) Per-head render** — the element plugins would need the chain, and the
  chain's citation context is an `Arc<Mutex<CitationContext>>` held for the whole
  pipeline run (`apps/cli/src/plugins.rs::DocumentContext`), so a render called
  from inside a pipeline step deadlocks on the lock it is already under. Rejects
  the layer model too.
- **(C) Sub-document semantics** — the maintainer's answer: `number` / `cite`
  continue the page's scope, so a field is _not_ a numbering island.
- **(D) Two-phase chain** — "head plugins" is undefined while the chain order is
  the author's (golden 27 puts `anchor` after `cite`).

- [x] **3.1 Measure the current behaviour** (probe, not a golden). Measured on
      `2026-11-01` with `plugin = "frontmatter,anchor,cite,wiki,markdown"`:

      ```text
      @[Card](/docs)@@anchorA{note: ;;see [^a];;}   ← line 3, the field
      Body text cites [^b] here.                     ← line 5
      Then [^a] again.                               ← line 7
      ```

      Output: `[^b]` → `1` (`id="cite-1"`), the body's `[^a]` → `2`, and the
      **field's** `[^a]` (which is the *first* occurrence in source) → `2` with
      `id="cite-3"`. So a renderable field is numbered at **render** time, i.e.
      after `cite`'s chain arm has already walked the body — the issue's §27
      order is violated exactly as predicted. Option B must reproduce
      `[^a]` → 1, `[^b]` → 2.
- [x] **3.2 Decide: (B) reserve in source order.** Late rendering stays; a pass
      walks the stream, and at each `mdp` value's position it registers the
      citation identities (`cite`) and `%%` members (`number`) the payload will
      need — then the render reuses the reservation instead of assigning next.
      (As 3.3b records, the two halves are not the same job: `cite` numbers an
      identity it can see in the payload's text, `number` numbers an element it
      can only see after the payload's own element plugins ran.)
- [x] **3.3a Implement B for `cite`** — done. `CitationContext::process_events`
      already sees the attribute at its source position, so `reserve_payload`
      pre-registers the identities the payload will need (cite's own head reader
      over the payload source; fenced code and inline code spans skipped, the two
      places markdown would hand cite a code node) and `emit_citation` reuses the
      reservation through the shared `reserve_identity`. Reserving records **no**
      `citations` row, so the field's physical occurrence is still written once,
      when it renders. The measurement in 3.1 now produces `[^a]` → 1 (`id="cite-3"`)
      and `[^b]` → 2 (`id="cite-1"`) — source order, as required.
- [ ] **3.3b Implement B for `%%`** — **not done, and it is not the same problem.**
      Measured `2026-11-01`: a `%%numberB` inside a field publishes
      `number.raw = ["1"]` and so does the body's marker — one page, two "1"s,
      because the payload's **own** `number` run owns a fresh counter. `number`
      numbers _elements_, not identities, so no reservation API helps: reserving
      needs the payload's members enumerated at the field's source position, and
      discovering them needs the element plugins to have run on the payload —
      which is what "render it later" defers. The fix is to split
      `pendon_number::process` into **discovery** (pass 1) and **assignment**
      (pass 2) over the host stream _plus_ each field's payload, i.e. an API
      change in `crates/number`, not a resolver change. Tracked here rather than
      hand-waved in the ADR (see ADR-0021 "Open").
- [x] **3.4a Pin the `cite` half with its own golden** — golden
      [`53-renderable-number`](../../docs/spec/golden/53-renderable-number.mdp) +
      `apps/cli/tests/renderable_fields_spec.rs`
      (`a_citation_inside_a_field_is_numbered_by_source_position`).
- [ ] **3.4b Pin the `%%` half** — blocked on 3.3b.
- [ ] **3.5 Cross-check `%%number` depth**: a marker inside a field inside a
      list/heading must see the host's `__layer` / `depth`. Blocked on 3.3b (the
      same discovery pass answers both).

**Acceptance:** the Phase-3 golden pins the decision; line 27 of this issue is
either satisfied or explicitly re-scoped in the ADR.

### Phase 4 — Extraction

- [x] Add an extract fixture that reads a renderable field **and** a `bind` `mdp`
      field from the same element — `apps/cli/tests/extract_spec.rs`
      (`a_renderable_field_carries_the_same_fragment_a_bound_mdp_field_does`).
      The page carries the identical `$fragment` for both; the row carries it for
      the renderable, while a `$bind` field's row still holds its reference —
      `bind` resolves in a post-pass that `process::render` runs **after** the
      chain (so after `extract`), which is a property of `bind`'s ordering, not
      of renderables. Pinned rather than papered over.
- [x] No text form: the structured node list is the value (`crates/extract/README.md`
      "A renderable field", `docs/spec/FIELDS-INVENTORY.md` §0 rules, §21 rule 11).
- [x] `docs/spec/SYNTAX.md` §21 rule 11: what a renderable attribute extracts to.
- [x] Tests: `apps/cli/tests/extract_spec.rs` (CLI). No `crates/extract` unit test
      is needed — the decode it performs (`attribute_value`) is already covered by
      the JSON-value tests, and the new behaviour is the _ordering_ the CLI owns.

### Phase 5 — Renderers (verify, do not rebuild)

- [x] **Solid, component attribute:** a resolved fragment is
      `name={(<>…</>)}` — golden `50` (a marker component's `note`).
- [x] **Solid, built-in element attribute:** the built-in path does not
      stringify it — `note={(<>hi <em>x</em></>)}` on a `<span>`
      (`apps/cli/tests/renderable_fields_spec.rs`,
      `a_builtin_element_attribute_carries_a_fragment`).
- [x] **Nested props:** `specs={{caption: (<>{…}</>)}}` and an array item —
      golden `52`. This is also where the **transport leak** was found and fixed:
      a nested field travels as JSON, so the resolver now walks a
      `JSON_ATTR_PREFIX` value for `{"$mdp": …}` wrappers (`chain::resolve_nested`).
- [x] **HTML:** escaped compact markup inside the attribute value
      (`a_renderable_in_html_is_escaped_markup_in_the_attribute`; the unit-level
      contract is `crates/renderer-html/src/lib.rs`). **JSON / AST:** the node
      list is kept (`a_renderable_is_a_node_list_in_json`).
- [x] **Deep nesting:** golden `52` freezes the exact fragment, including the
      `\;\;` escape and the escaped semicolons it leaves as text — recorded as an
      open question in ADR-0021 rather than hidden.

### Phase 6 — Goldens and the test gate

- [x] `50-renderable-inline` — marker + two element positions, inline mode,
      citations resolved against the page (the same reference cited twice keeps
      `[1]`).
- [x] `51-renderable-block` — `;;;…;;;` with a heading, a list and a blockquote,
      next to the same paragraph in `;;` form, which is what proves
      `MdpMode::Block` keeps the wrapper and inline drops it.
- [x] `52-renderable-nested` — spec object, array item, a nested head, the bound
      `mdp` field beside the written one, and the image whose `alt` reads the
      authored spelling (D-RF-6). The `\;\;` escape substitutes for the
      illustration's "field → spec → object → field" walk, whose unescaped form
      the grammar does not have (ADR-0021 "Open").
- [x] `53-renderable-number` — `citation_number` source order. The `%%` half is
      **not** in the golden: it is 3.3b, and freezing an island behaviour would
      freeze the defect.
- [x] `54-renderable-literal` — unclosed `;;`, `;;` in prose, `;;` in
      `.class` / `#id`, a quoted value that is not a whole span, and the `\`
      escape. `{{…}}` inside a field is **not** frozen: with `bind` out of the
      chain it is text, and with 2.3's registry threading it becomes a binding —
      a golden must not pin the answer before the question is settled.
- [x] Re-ran `cargo test -p pendon --test syntax_spec`: 50 tests green, every
      pre-existing golden byte-identical.
- [x] Harness guard: `assert_no_renderable_transport` runs on every render **and**
      once over every frozen `.jsx` (`no_frozen_page_carries_the_renderable_transport`),
      so neither a rendered page nor a hand-edited golden can carry `U+E003` or a
      `"$mdp"` wrapper.
- [x] Updated the fixture table in `docs/spec/golden/README.md` (rows 50–54).

### Phase 7 — Documentation

- [x] `docs/spec/SYNTAX.md` §3 (sigil inventory), §6.7 (the full rule),
      §21 rule 11 (what a renderable extracts to). §12.1 / §12.2 / §19 need no
      sentence: the resolve stage _is_ `render_mdp`, and "when to use which" is
      now one bullet in `crates/plugin-bind/README.md` rather than a second rule.
- [x] `crates/extra/README.md`: the span scanner, the opaque-interior rule and
      `AttrValue::Mdp`.
- [x] `crates/plugin-bind/README.md`: a renderable is the grammar-level spelling
      of the same fragment; the two are interchangeable; a payload may itself
      carry a renderable (`run_chain` resolves the sub-run's own fields).
- [x] `crates/extract/README.md` + `docs/spec/FIELDS-INVENTORY.md` §0: the row is
      a node list, never text.
- [x] `docs/guides/cli.md`: no flag and no setting appeared, so nothing to add.
- [x] This file: tasks ticked, Phase-3 decision recorded, both Open Questions
      closed above, and the two new open items (`%%` numbering, nesting
      ergonomics) written down in ADR-0021 rather than here.

### Phase 8 — Editor grammar (follow-up, not a blocker)

- [ ] `editors/vscode/`: scope `;;…;;` / `;;;…;;;` as
      `markup.other.renderable.pendon` and embed `source.markdown` inside it; add
      cases to `tools/scope-cases.mjs`; note the line-bounded limitation already
      documented in `editors/vscode/README.md`.
      **Deliberately not done in this pass:** `editors/vscode/**` already carries
      uncommitted work on this branch (table reflow, the `specs` scope rename), and
      mixing a grammar feature into it would make both diffs unreviewable. The
      scope name and the line-bounded note are decided here so the follow-up is
      mechanical.
- [ ] `editors/vscode/README.md`: link §6.7, do not restate it. Blocked on the
      same.

## Testing matrix

| Layer      | Where                                       | What it proves                                                |
| ---------- | ------------------------------------------- | ------------------------------------------------------------- |
| span       | `crates/extra/src/renderable.rs` unit tests | opener/closer, `;` runs, `\` escape, `\n`, unclosed           |
| slot lexer | `crates/extra/tests/…`                      | six positions, opaque interior, `\|` run unaffected           |
| spec lexer | `crates/extra/tests/…`                      | `,` / `"` / braces inside a value, nested object/array, quote |
| transport  | `crates/extra/src/value.rs` unit tests      | `literal` / `transport` / `to_json` / `from_json` round-trip  |
| resolve    | `apps/cli/src/chain.rs` unit tests          | fragment, diagnostics, nesting, depth guard                   |
| CLI e2e    | `apps/cli/tests/renderable_fields_spec.rs`  | every head kind, each renderer                                |
| golden     | `apps/cli/tests/syntax_spec.rs`             | byte-exact output, numbering, literal fallbacks               |
| extract    | `apps/cli/tests/extract_spec.rs`            | fragment shape, identical to `bind mdp`                       |

**Gate:** `cargo test --workspace`, `cargo clippy --workspace --all-targets`,
`cargo fmt --all -- --check`, `dprint check`.

## Open Questions

Answered by the ground-truth pass:

- **"Pendon already has a mechanism (VERIFY)."** Yes — fragment + `$fragment` +
  `render_mdp`. The work is grammar + transport + a resolve pass, not a new
  renderer. See D-RF-1/2.
- **"How to `extract` renderable fields?"** By construction they become a
  `JSON_ATTR_PREFIX` fragment, so `plugin-extract` hydrates the same
  `{"$fragment": […]}` a `bind mdp` consumer produces — but only if the resolve
  pass runs before `extract`. Phase 4 pins it.

Still open (must be closed in Phase 0 / Phase 3):

- **Numbering scope — ANSWERED (D-RF-8):** `number` / `cite` **continue the
  page's scope**; a renderable is not a numbering island. The _ordering_ is
  option B (Phase 3).
- **`;;` in a table cell / decorator line — ANSWERED:** legal. They use the same
  slot and spec lexers, so a table cell's `|` and a decorator's `@@type` boundary
  never see a span's interior (D-RF-2). Fixture `54` still pins the end-to-end
  behaviour.
- **A position that needs a literal (`src`, `slug`, `reference_id`) — ANSWERED
  (D-RF-6):** the grammar permits a renderable there. The position is
  configuration the author can move (§6.1), so nothing is structurally reserved;
  the plugin reads `literal()` (the authored `;;…;;`) and behaves as it always
  did. `.class` / `#id` / `isFoo` / `style:` / `--var` are not field values and
  keep the raw text.
- **Unclosed `;;` in an unquoted spec** (`k: ;;a b`): today `UnquotedWhitespace`
  makes the whole head literal. Keep that, or let an unclosed span fall back to
  a literal scalar so the head survives? Recommendation: **keep today's
  behaviour** (a head is all-or-nothing, §4.3) unless a golden shows it is
  surprising.
- **Block mode's wrapper:** `;;;…;;;` renders like `MdpMode::Block`, so a single
  paragraph keeps `<p>`. The issue's first illustration expects `<p>` for the
  quote slot (`(<><p>lorem …</p></>)`) and no `<p>` for the bare slot
  (`(<>lorem …</>)`). Confirm inline is the default for `;;` and block for
  `;;;`, and that a single-paragraph block keeps its `<p>` (it does today).
