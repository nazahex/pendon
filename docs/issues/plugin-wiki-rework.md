# `plugin-wiki` rework: wikilinks only, with `[task.wiki.custom]`

**Status: scheduled, not started.** The infobox half of the plugin is _deleted_,
not migrated; the wikilink half is kept and brought up to the Stage-2 contract
(§5 extras heads, §11 component sets).

## Why

`crates/plugin-wiki` still carries an infobox implementation written before the
unified syntax existed — `infobox.rs` is 385 of its 826 lines and owns spellings
of its own (`:::infobox[rox]`, `::[.img] … ::`, `Term =[.cill,.elit] Value`).
`crates/plugin-wiki/README.md` already commits it to deletion in the next
workflow, together with `quiz` and `vicado`, and forbids the alternatives
(migrating infobox nodes to `__layer`, giving them types/selectors, debugging
their output).

The ground has already moved: `:::` is not grammar any more. §14 records
`:::name` (container → `::type…::` / `==type…==`, or a runtime `plugin-custom`
spec's own literal) and `plugin-quiz`'s `:::quiz` (the plugin is not in the
pipeline, so the line is ordinary text). What is left in this crate is a parser
for syntax the spec no longer has.

Wikilinks, by contrast, are normative: §7.5 defines
`[[target]]` / `[[target | label]]` plus the shared §5 extras head, and `href`
comes from `link_prefix` and wins over extras. Two gaps remain:

- **No §11 component set.** `WikiOptions` is `link_prefix` + `latex`; `wikilink`
  calls `Specs::default()`, so `field.*` and `bracket.modifier` never reach a wiki
  `<a>` and a `[[…]]@@wikiA{…}` head can only pick the built-in `slug`/`title`
  (recorded as a threading gap in `STATUS.md`).
- **`latex` option exists only for infobox fragments.** Once the infobox goes, the
  option has no caller.

## What the rework does

1. **Delete the infobox path**: `infobox.rs`, the div-marker and `Term =[…]`
   prefix parsing, the `latex` option and its re-run. Keep `util.rs` only for what
   the wikilink pass still uses.
2. **Keep wikilinks, upgrade them**: read the trailing extras head through the
   shared reader in `crates/extra`, project the spec fields through the element's
   `ComponentSet`, and add `[task.wiki.custom]` (`WikiOptions::custom:
   ComponentSet<WikiCustomNode>`) so §11 `type = …` selectors and
   `field.override.*` behave exactly as on the other nine layers — including
   `solid_hints` for a component template.
3. **Delete the leftovers that named infobox/quiz/vicado** in `renderer-solid`
   comments, `crates/core/README.md` and `crates/extra/README.md`. (The dead
   `"quiz"` no-op arm in `apps/cli/src/chain.rs` is already gone — `S2-10.4`; a
   `plugin = "quiz"` name now resolves through the custom registry or warns once,
   §13.)
4. **Rewrite the docs**: the plugin README drops its deletion note and documents
   only wikilinks, options and the §11 set; §7.5 gains the §11 pointer; §14 keeps
   its `:::name` / `:::quiz` rows unchanged (they describe markup, not this crate).
5. **Replay the sandboxes** that still ship `plugins/quiz.toml` and `:::quiz`
   sources (`sandbox/custom`, `sandbox/dialog`, `sandbox/universal`): with the
   plugin removed, those blocks are ordinary text — that is the same
   maintainer-owned replay `STATUS.md` "Deferred" describes.

## Non-goals

- Do **not** re-introduce `:::` as a container spelling (§14, D-16: a removed
  spelling is literal text).
- Do **not** port infobox behaviour onto `__layer` / `layer_templates` or give it
  types/selectors.
- No new wikilink syntax: §7.5 already fixes the grammar; this is a config and
  plumbing change.

## Exit

- `grep -rni infobox crates apps` returns only historical text in `docs/`.
- `[task.wiki.custom]` exists with typed entries, one default, `field.*` and
  `bracket.modifier` reaching the `<a>`; a golden freezes it.
- `cargo test --workspace` green; `crates/plugin-wiki/README.md` links §7.5 and
  §11 instead of restating them.
