# Product Context

## Why this project exists

Authoring rich documents (tables, figures, citations, wiki links, components)
in plain Markdown quickly hits its limits. Pendon extends Markdown with a
**small, unified** surface syntax so an author can attach typed attributes and
bind custom components without leaving a Markdown-like file — while keeping the
transform pipeline predictable and testable.

## Problems it solves

- Attribute/component syntax that is consistent across constructs (one `{…}`
  head grammar shared by anchors, images, tables, directives, …).
- Safe fallback: malformed markup becomes literal text, never a crash.
- Deterministic output so fixtures can be frozen and diffed byte-for-byte.

## How it should feel to use

- Predictable: the same rules apply everywhere (adjacency, escaping, fallback).
- Composable: enable plugins per task; unknown plugins/keys warn, not fail.
- Transparent: an `events` and `ast` renderer expose the IR for debugging.

## UX goals

- CLI reads stdin or a file; `--format` switches output; `--pretty` indents.
- `docs/spec/golden/` doubles as executable documentation and living examples.
