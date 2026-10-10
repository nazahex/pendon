# Project Brief

**Pendon** is an event-driven, **plugin-first** Markdown-as-DSL engine with a
predictable intermediate representation (IR). Authors write a Markdown superset;
plugins transform an `Event` stream; renderers emit output (Solid/JSX, HTML,
JSON, AST).

## Core requirements

1. One shared implementation over per-plugin copies (shared syntax lives in
   `crates/extra`).
2. The event IR is the boundary between parse → transform → render.
3. Plugin-specific semantics stay inside the owning plugin.
4. Direct CLI and config-driven runs behave identically for equivalent options.
5. Behaviour is pinned by executable fixtures (golden set) — tests are the truth.

## Scope

- **In scope:** the unified typed-extras surface syntax (`@@type{…}` heads,
  decorators, markers, directives, tables, lists, blockquotes, sections),
  plugins under `crates/plugin-*`, renderers under `crates/renderer-*`, and the
  CLI in `apps/cli`.
- **Out of scope (documented non-goals):** `plugin-wiki` infobox internals,
  `plugin-quiz` (retired), renderer template semantics beyond attribute mapping.

## Source of truth for scope

This file defines scope. The normative grammar is `docs/spec/SYNTAX.md`; live
work is `STATUS.md`. See `docs/INDEX.md` for the full documentation map.
