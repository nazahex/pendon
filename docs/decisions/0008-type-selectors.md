# ADR-0008 — `type` is a selector list resolved against a document

- Status: **Accepted / landed** — `crates/types` resolves each layer in
  `renderer-solid`, and all ten element plugins (anchor, blockquote, cite,
  directive, heading, image, list, marker, section, table) now emit
  `__layer`-tagged built-ins instead of a `NodeKind::Custom` swap (Phase 1 of
  [`workflows/numbering-extract-type-export.md`](../workflows/numbering-extract-type-export.md))
- Depends on: [`0001`](0001-typed-extras-reconciliation.md) (typed extras),
  [`0003`](0003-positional-groups.md) (positional keys)
- Spec: [`SYNTAX.md`](../spec/SYNTAX.md) §11 (component sets), §5 (`type`),
  §15 (open decisions)
- Design: [`workflows/type-resolution.md`](../workflows/type-resolution.md)
- Evidence: `crates/types` unit tests, golden
  [`28-type-selectors`](../spec/golden/README.md)

## Context

§11 rule 2 let a component entry name the `type` markers it answers, and
matching was an exact string compare done _inside each element plugin_ while it
built its nodes (`options.custom.select(marker)` → `NodeKind::Custom(name)`).
That has three consequences the proposals asked to remove:

1. **No wildcards are possible.** `_` ("every type no other entry named") and
   `*` ("every type") need the _document's_ occurring types. A plugin sees one
   element at a time and never the document, so the choice cannot be made there.
2. **The rule is duplicated ten times.** anchor, blockquote, cite, directive,
   heading, image, list, marker, section and table each re-implement "exact
   match, else default, else built-in".
3. **Two more consumers want the same algorithm.** `plugin-number` (Phase 3)
   and `plugin-extract` (Phase 4) scope their blocks by the same selectors, so
   three copies would eventually disagree.

## Decision

**`type` is a selector list, and resolution happens once per document, in
`renderer-solid`.** The algorithm lives in a new crate `pendon-types`
(`D-1`, `D-2`, `D-3a`, `D-4` of the workflow doc).

1. A selector is `Exact(name)`, `?` (untyped), `_` (remainder) or `*`
   (everything). `type = "TypeA"` and `type = ["TypeA"]` are the same list.
2. **Specificity** is per selector: literal / `?` = 2, `_` / `*` = 1. The winner
   of a type is the highest specificity, ties broken by **later registration**.
3. **`defined` is computed up front for the whole group** (one layer of one
   task): every literal any rule names, plus `untyped` if any rule wrote `?`.
   `_` expands to `occurring − defined`, `*` to `occurring`. `_`/`*` define
   nothing, so two `_` rules still see the same remainder, and a rule declared
   later cannot change what an earlier `_` meant.
4. **Plugins stop choosing.** They keep emitting their built-in node and the
   `type` attribute they already emit; `template::select_template` builds the
   layer's `pendon_types::Group` from the document's type set and picks the
   winning entry. Imports are emitted only for winning entries, so a losing
   component's `import` line never appears.
5. **A config `type` that is not a selector is a `ConfigError`**, naming the
   layer and the key. Typos never silently become wildcards.
6. `names_for` / `names_for_opt` (positional names, §6.1) stay
   **literal-only**: `_`/`*` contribute no `slug`/`title` override, because a
   wildcard has no single entry to read the names from.

## Rejected alternatives

- **D-3b — pre-scan the document in the CLI.** Cheaper (no plugin edits), but
  it re-parses `@@`-head markup outside `crates/extra`, which the layer model
  forbids (`CONTRIBUTORS.md`). Removed from the plan once D-3a was accepted.
- **Resolve inside each plugin.** Needs a document-wide type set threaded into
  every plugin option struct, and keeps ten copies of the matching rule.
- **`type` as a glob / regex.** Authors write marker names, not patterns; a
  typo would match silently instead of falling back to the layer default.
- **Compute `defined` lazily, per resolution.** Makes the meaning of `_` depend
  on the order elements are visited — the bug class this ADR exists to prevent.
- **Treat `??`/`**` as escapes.** They are not special; a selector is exactly
  one token, anything longer is a literal name.

## Consequences

- `ComponentEntry.types: Vec<String>` becomes `selectors: Vec<Selector>` in the
  CLI and `TypedComponent.selectors` in `renderer-solid`. `ComponentSet::select`
  is deleted: plugins no longer ask "which component is mine?", and the CLI keeps
  only the config-side entries (`typed()` / `default_entry()`) that it hands to
  `layer_templates`.
- Ten element plugins lose their `select` + `Custom(name)` swap. `NodeKind::Custom`
  stays only where a `ComponentSet` is not involved at all — `plugin-custom`'s
  `toml:` specs, `plugin-quiz` and `plugin-vicado` — and the renderer answers
  those through node identity (`LayerKey::Identity`), which is untouched by this
  ADR. **Scope note:** `quiz`, `vicado` and the wiki infobox are scheduled for
  deletion in the next workflow. They are ignored here — the identity path only
  has to keep working until they are gone, so do not spend a migration on them
  and do not treat their output as a constraint.
- Duplicate literal `type` in one layer stays a hard error; duplicate `?`/`_`/`*`
  are allowed and ordered (`D-4`).
- The renderer now needs the document's occurring types: `render_solid_with_hints`
  walks the AST once, collects `type` attributes, and threads a `TypeSet`
  through node rendering and import gating.
- `plugin-number` and `plugin-extract` reuse the crate instead of re-deriving
  "which types does this rule own" (`matching`).
- **Landed without moving a byte of output.** The ten cutovers were verified by
  re-freezing their goldens (`03`, `06`, `08`, `09`, `27`): the table's
  `09-table-layers` — seven layers, typed rows/cells, wildcards and the
  default/fallback split — is byte-identical to its pre-cutover `.jsx`, so the
  move from "plugin chooses" to "renderer resolves" is behaviour-preserving.
