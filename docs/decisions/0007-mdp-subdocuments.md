# ADR-0007 — `mdp` payloads are sub-documents (fragment transport)

- Status: **Accepted / implemented**
- Depends on: [`0004`](0004-pre-parse-bind-stage.md) (the pre-parse bind stage),
  [`0005`](0005-structured-extras-values.md) (structured values + `U+E001`)
- Spec: [`SYNTAX.md`](../spec/SYNTAX.md) §19
- Evidence: golden [`27-bind-mdp`](../spec/golden/README.md), re-frozen
  [`23-bind`](../spec/golden/README.md); `pendon-plugin-bind` unit tests;
  `pendon-renderer-{ast,solid,html}` tests

## Context

`mdp` (Pendon Markdown as a value) was rendered by `plugin-bind`'s own private
"inline pipeline": `plugin-markdown` plus a hand-picked inline set (img, wiki,
cite), and the result was transported with `JSX_ATTR_PREFIX` (`U+E002`) as raw
JSX text. Four problems followed:

1. **The plugin list was a lie.** A payload rendered "through the inline
   pipeline", but latex, table, heading, marker, directive, section and custom
   components never reached it — adding one meant editing both `plugin-bind` and
   the CLI.
2. **The payload had no access to the page's state.** It was rendered by bind's
   own pipeline with its own options, so a citation inside a payload could not
   join the page's citation list, and a figure's hints/imports were lost.
3. **One renderer's syntax lived in the IR.** `U+E002` carried JSX, so every
   other renderer could only escape or strip it; HTML and JSON had no structured
   description of the payload at all.
4. **The emitted output was already broken** and frozen as such: golden `23-bind`
   showed `<code>` dropped and `<a href="…"link</a>` mangled in inline mode.

## Decision

1. **`mdp` is a sub-document.** Its body is parsed with `pendon_core::parse` and
   run through the **host's own chain** — the CLI's `chain::run_chain` with the
   payload flag — so the payload sees exactly the plugins the page does. Only the
   stage that writes _document-level_ outputs (`extract`, ADR-0011) is skipped;
   every other enabled plugin runs and shares the host `DocumentContext`.
2. **`plugin-bind` does not know the plugins.** `resolve_with(events, &registry,
   &mut render)` takes the renderer from the caller; `resolve(events, &registry)`
   remains as the Markdown-only wrapper for embedders that have no chain.
3. **A payload travels as a fragment.** The node list is wrapped as
   `{"$fragment": [ … ]}` under `renderer_ast::FRAGMENT_KEY` and carried on
   `Event::Attribute` as `JSON_ATTR_PREFIX` + compact JSON (ADR-0005's transport).
   `renderer-ast` re-hydrates it and **each renderer owns its own format**: Solid
   emits `name={(<>…</>)}`, HTML renders the markup into the attribute value, and
   the AST/JSON outputs keep the structured node list. `JSX_ATTR_PREFIX` keeps its
   one job — an authored `( … )` extras expression.
4. **Inline vs block form** is the wrapper, not the rendering: the inline form
   drops the payload's `Document` and its single `Paragraph`, so text written on
   the head line contributes inline nodes; the block form keeps both.
5. **`resolve` runs after every plugin's extras emit and before the citation
   export** (ADR-0004's order, which now matters because a payload may cite), so
   a payload's citations are numbered with the page's.
6. **`micromatter` is the one stage before the citation context.** The context is
   built from the `Frontmatter` node that `micromatter` produces, so the CLI runs
   a `prelude` (text stages) _before_ constructing `DocumentContext` and the rest
   of the chain after it.

## Rejected alternatives

- **Keep the hand-picked inline pipeline and add plugins to it.** Every new
  plugin would need an edit in two crates, and the payload would still not share
  the page's context or hints.
- **Transport rendered markup (a `String`).** The renderer that produced it wins;
  Solid would have to re-parse HTML, and the AST could not describe the payload.
- **Keep raw JSX in the IR (`U+E002`).** Solid's syntax in the model: HTML/JSON
  can only escape it, which is exactly the output that had to be re-frozen.
- **Extract frontmatter before `micromatter`.** At that point the block is still
  plain text — it only _becomes_ a `Frontmatter` node inside `micromatter` — so
  the citation context would always be empty. This is the regression that
  `cli_spec::run_config_cite_frontmatter_defaults_to_all_and_honours_cited`
  pins, and the reason for the `prelude` stage.
- **Render the payload through a second, throwaway `ChainState`.** The page's
  citation numbering and hint list are the point; a fresh state would restart
  both.

## Consequences

- A payload behaves like the page: wiki links, citations, figures, tables, math,
  extras heads and custom components work inside it, and the hints their plugins
  need join the page's (`renderer-solid` walks fragments with
  `collect_fragment_nodes` to emit their imports).
- HTML attribute values are escaped markup (`data="&lt;p&gt;…"`) — an attribute
  is text by definition; a fragment nested inside a structured value renders in
  place so the surrounding JSON stays readable.
- AST/JSON keep `{"$fragment": …}`; a consumer that renders JSON itself must
  handle the key rather than print it.
- Golden `23-bind` was re-frozen (its old expectation froze the broken output)
  and `27-bind-mdp` pins the cross-plugin behaviour.
- The chain gained a two-step entry (`prelude` then `run`), because the citation
  context sits between them.
- **Not** covered: a `$var` inside a payload's _own_ extras head
  (`{{{mdp[x] {{m}}{data: $foo} }}}`) stays literal and silent — resolution is one
  pass over the page's attributes, and the payload's heads are emitted inside that
  pass (verified with the real CLI). Making it work needs a second, depth-guarded
  pass over the fragment tree; it is a separate decision (§19 non-goal).
