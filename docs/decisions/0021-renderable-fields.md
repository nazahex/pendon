# ADR-0021 — Renderable fields: `;;…;;` / `;;;…;;;` are `mdp` values

- Status: **Accepted** — grammar + transport + the resolve stage + the **cite**
  half of source-order numbering landed (`crates/extra`, `crates/core`,
  `apps/cli/src/chain.rs`, `crates/plugin-cite`); `%%` numbering is open
- Depends on: [`0007`](0007-mdp-subdocuments.md) (an `mdp` payload is a
  sub-document), [`0005`](0005-structured-extras-values.md) (a value travels as a
  sentinel, never as text), [`0012`](0012-universal-slots.md) /
  [`0020`](0020-unattached-heads-share-the-six-slots.md) (the six positions),
  [`0014`](0014-field-model.md) (a field's position is configuration)
- Spec: [`SYNTAX.md`](../spec/SYNTAX.md) §3, §5.2, §6.3, §6.7
- Evidence: `crates/extra/src/renderable.rs`, `crates/extra/src/slot.rs`,
  `crates/extra/src/typed.rs`, `crates/extra/src/value.rs`,
  `crates/extra/tests/renderable_fields_spec.rs`, `crates/core/src/event.rs`,
  `apps/cli/src/chain.rs` (`resolve_renderable`), `crates/plugin-cite`
  (`reserve_payload`), `crates/extract/src/scan.rs`, `crates/renderer-html`,
  `apps/cli/tests/renderable_fields_spec.rs`,
  `apps/cli/tests/extract_spec.rs`,
  goldens [`50`–`54`](../spec/golden/README.md)

## Context

A field's value is text. That is wrong for the one value Pendon already knows how
to render as a _document_: an `mdp` payload ([ADR-0007](0007-mdp-subdocuments.md))
resolves against the page's own state — wiki links, citations, figures, math,
extras heads, custom components — and becomes a fragment prop. Before this ADR
the only way to get one was `plugin-bind`'s `{{{mdp[var] … }}}` block plus a
`$var` attribute, i.e. a named variable and a second head. An author who wanted a
rendered body in _this_ element's `note` field had to leave the element.

The naive fix — a second grammar for `;;` in each plugin — is the layering
violation `CONTRIBUTORS.md` forbids: six slot positions, five specs positions,
nested objects/arrays, table cells and decorator lines all read slot/spec text,
and each of those lexers would need its own copy of "a payload may contain `,`
`"` `[` `]` `{` `}` `:` and whitespace".

## Decision

1. **One span scanner, in the grammar layer.** `renderable::span(text, start)`
   reads `;;` (inline) or `;;;` (block) — longest opener wins — and returns the
   payload's byte range plus the index just past the closer. `\` escapes the next
   character; a run of `;` at least as long as the opener closes the span and is
   consumed **whole** (a `;;;` run cannot leave a stray `;` in a `;;` span); an
   unclosed span is `None`, so the text stays literal (§4.3) rather than
   half-parsing. `renderable::whole(text)` is the whole-value rule: after
   trimming, the value is exactly one span and nothing trails it.
2. **Every lexer skips an opened span.** `slot::{find_close, read_bare,
   read_delimited}`, `typed::{find_group_close, split_items, split_key_value}`
   treat the interior as opaque. That single rule is what makes "rendered as full
   mdp" true everywhere: a payload's `]` cannot close a bracket, its `,` cannot
   split an item, its `:` cannot split a key.
3. **A renderable is a value, not a sigil.** `[pre ;;x;; post]` and
   `k: "pre ;;x;; post"` are text. `;` is ordinary punctuation, and a value that
   is only _partly_ a payload has no defined splice point.
4. **Every value position accepts one; no _position_ is structural.** A renderable
   is legal in all six slot contents and in every spec value — including the
   positions that _by default_ feed a construct's own field (`reference_id`,
   `slug`, `src`, `depth`). Those are not reserved slots: `field.*` lets an author
   move a construct's meaning to another position and put their own value in the
   old one, so the grammar cannot know which position is "structural". What a
   plugin does when the value it needs is not literal is the plugin's own rule,
   and it reads `AttrValue::literal()` — the authored `;;…;;` text — never a
   sentinel. `.class` / `#id` / `isFoo` / `style:` / `--var` are not field values
   at all (a _name_ or a _declaration_ grammar), so a span cannot be spelled
   there; the head is rejected as before.
5. **Transport is a sentinel, never text.** `AttrValue::Mdp { mode, source }`
   transports as `pendon_core::MDP_ATTR_PREFIX` (`U+E003`) + the `MdpMode` byte +
   source. `literal()` keeps the authored spelling — the documented fallback —
   and `to_json()` / `from_json()` carry `{"$mdp": {"mode": …, "source": …}}`.
   `MdpMode` moves from `plugin-bind` into `pendon_core` (one mode type for `;;`,
   `;;;` and `{{{mdp…}}}`, D-16: no re-export), so `crates/extra` needs no
   dependency on a plugin.
6. **`emit_attrs` transports.** It emitted `literal()`, so a structured value
   crossing a decorator lost its sentinel; it now emits `transport_for(name)`
   (an internal `__` key keeps its literal — `number`'s marker is parsed before
   any renderer runs).
7. **Numbering stays page-scoped.** A field's `citation_number` and `%%` markers
   continue the page's scope: the payload is the page's sub-document
   (ADR-0007), not a numbering island.
8. **Order is source order, so a field reserves its numbers where it is
   written.** A field renders after the chain has walked the document, so
   assigning "the next number" at render time numbers by **output** order. For
   citations the reservation is exact and cheap: `cite`'s own pass already sees
   the attribute at its source position, so it pre-registers the identities the
   payload will need (`CitationContext::reserve_payload`, cite's own head reader
   over the payload source, fenced code and inline code skipped) and the render
   reuses them. Reserving writes **no** `citations` row, so the field's physical
   occurrence is still recorded exactly once — when it renders.

## Rejected alternatives

- **A second `;;` parser in each plugin.** Six copies of the same scanner, and
  every one of them would have to be kept in step with §5.2's escape rules. This
  is the reason `%` normalization lives in one place (ADR-0010).
- **A new renderer or a fragment format.** None is needed: the payload is the
  same `{"$fragment": […]}` a `bind mdp` value produces, `renderer-ast` already
  re-hydrates `JSON_ATTR_PREFIX`, and every renderer already carries a node list.
- **Treat `;;` as a sigil anywhere in prose.** A marker that fires mid-sentence
  needs a rule for what it splices into. Whole values only.
- **Refuse a renderable where a plugin needs a literal (`src`, `slug`,
  `reference_id`).** The position is configuration (§6.1, ADR-0014): the author
  decides what `field.bracket.bare` names, so no position is reserved. Refusing
  would make a value's legality depend on which element happened to claim the
  head. A construct that cannot use it keeps the authored text and reports what
  it always reported.
- **`;;` inside quotes is always a string.** Then `k: ";;x y;;"` and
  `k: ;;x y;;` would differ only in quoting, which §5.2 says must never decide a
  value's meaning. The quotes are an escape hatch for characters the head cannot
  carry, not a downgrade.

## Consequences

- `crates/extra` gains `renderable` (`Span`, `Renderable`, `span`, `whole`,
  `materialize`); `SlotParts` stores `SlotValue { text, render }` (its `get`
  still returns `&str`, so every existing reader is unchanged); `AttrValue` gains
  `Mdp`; `pendon_core` gains `MDP_ATTR_PREFIX`, `MdpMode`, `mdp_attr`,
  `is_mdp_attr`, `mdp_attr_payload`.
- `SlotMatch` grows past clippy's `large_enum_variant` threshold and allows the
  lint (same rationale as ADR-0020).
- A block payload in a one-line spec value writes its breaks as `\n`;
  materialisation turns them into real newlines.
- `emit_attrs` now transports structured values; a head's nested object reaches
  the AST as an object instead of JSON _text_ (no golden depended on the old
  spelling — the workspace stayed green).
- `pendon_plugin_bind::MdpMode` is gone; callers import `pendon_core::MdpMode`.
- The resolve stage is `chain::resolve_renderable`, run at the tail of
  `run_chain` and again just before the `extract` arm (extract reads
  attributes, so it must never see a transport sentinel). It reuses `render_mdp`,
  so nesting is automatic, and a `MAX_RENDER_DEPTH` guard degrades a runaway to
  text with one `Error` instead of overflowing the stack.
- A field in a **nested** position (`k: [;;a;;, "b"]`, `specs: {deep: ;;c;;}`)
  travels as JSON, so the stage walks a `JSON_ATTR_PREFIX` value for
  `{"$mdp": {"mode", "source"}}` wrappers and replaces each with the rendered
  fragment (`resolve_nested`). Without that walk the wrapper reached the page as
  a JS object literal — the transport leaking through a value instead of an
  attribute — which is what golden [`52`](../spec/golden/README.md) pins.
- Each renderer keeps its own shape and none of them stringifies a field: Solid
  emits `name={(<>…</>)}` (component prop and built-in attribute alike), HTML
  escapes the compact markup into the attribute value, and JSON/AST keep the node
  list (`apps/cli/tests/renderable_fields_spec.rs`,
  `crates/renderer-html/src/lib.rs`). Extraction needs no change beyond the
  resolve-before-scan ordering: the row carries the same `{"$fragment": …}` a
  bound `mdp` field produces (`apps/cli/tests/extract_spec.rs`,
  [`crates/extract/README.md`](../../crates/extract/README.md)).
- The golden harness asserts that no rendered **and** no frozen page carries
  `U+E003` or a `"$mdp"` wrapper (`assert_no_renderable_transport`), so a new
  emitter cannot find a path around the resolver.
- A _quoted_ renderable (`k: ";;…;;"`, ``[`…`]``) is read from its authored
  body, not from the unescaped string: otherwise `\n` would be the quote's
  escape (`n`) instead of a payload line break. A quoted value that is not a
  whole span keeps §6.3's unescaping exactly.

## Open

- **`%%` numbering (item 8, marker half):** the citation half is landed and
  pinned (golden [`53`](../spec/golden/README.md)), the marker half is not.
  Measured `2026-11-01` with `[[task.number.custom]] type = ["NumberB"]`: a
  `%%numberB` inside a field is numbered by the payload's **own** sub-run, so it
  and a body marker both publish `raw = ["1"]` — one page, two "1"s. `number`
  numbers **elements**, not identities, so the cite reservation does not carry
  over: reserving means enumerating the payload's members at the field's source
  position, and discovering them needs the element plugins to have run on the
  payload, which is exactly what is deferred to render. The shape of the fix is
  therefore not "add a reservation API" but "split `number` into discovery
  (pass 1) and assignment (pass 2) over the host stream _plus_ every field's
  payload" — a change to `pendon_number`'s public API, tracked in
  [`../issues/renderable-fields.md`](../issues/renderable-fields.md) §3.5.
- **Nesting leaves its semicolons.** A nested field is written with escapes
  (`note: ;;outer \;\;@[In](/u)@@anchorB{v: \;\;inner\;\;} \;\; tail;;`) and the
  escaped semicolons stay as **text** in the payload, so a hand-written deep
  tree shows `;;` around each nested element (golden
  [`52`](../spec/golden/README.md)). For anything deeper than one level a `$bind`
  variable is the better spelling; whether the grammar should offer an
  unescaped nesting form is genuinely undecided.
- **`$bind` inside a payload.** `k: $var` in a renderable is still unresolved,
  because `run_chain` skips `bind` for every payload (pre-existing, not specific
  to renderables): the fix is to thread the `Registry` into `render_mdp` as
  `Option<&Registry>` and call `resolve_with` on the payload.
