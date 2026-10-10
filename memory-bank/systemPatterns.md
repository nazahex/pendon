# System Patterns

## Architecture

```
input ─▶ pendon_core::parse ─▶ Event stream ─▶ plugins (transform) ─▶ renderer ─▶ output
                                (IR: StartNode/Attribute/Text/EndNode/Diagnostic)
```

- **Two plugin stages:** pre-markdown (text → text, "protect") and post-markdown
  (events → events, "bind"). See `docs/spec/SYNTAX.md` §12.
- **Pipeline order is explicit** and matters (e.g. `section` before `markdown`
  before `heading`; `micromatter` before frontmatter-dependent context).

## Key technical decisions (see `docs/decisions/` for rationale)

- **Unified typed-extras head** `@@type{…}` shared by every construct
  (ADR-0001, D1–D10).
- **`crates/extra` owns shared syntax** (`AttrValue`, `ExtrasHead`,
  `parse_type_marker`, decorator binder). Plugins consume it; they do not
  re-implement head parsing.
- **Decorator binder** (`crates/extra/src/decorator.rs`): a standalone extras line
  binds to the _next_ block node via a caller-supplied `target` callback; the
  plugin owns _what_ it attaches to.
- **Layer model:** plugins expose named layers (e.g. table → `table`/`caption`/
  `thead`/`tbody`/`tfoot`/`row`/`cell`; list → `list`/`unordered`/`ordered`);
  `[task.<plugin>.custom.<layer>]` selects a Solid component per layer/type.
- **`__plugin_kind` marker:** a `Custom`/`Element` node tells `plugin-markdown`
  how to re-lex its body (`block`/`inline`/`element`/`unordered`/`ordered`/…).

## Critical implementation paths

- Shared head scan: `crates/extra/src/typed.rs` (+ `value.rs`, `decorator.rs`).
- Config/component loader: `apps/cli/src/components.rs`, `plugins.rs`.
- Renderers: `crates/renderer-solid` (JSX contract), `-html`, `-json`, `-ast`.

## Testing strategy

Golden fixtures in `docs/spec/golden/` are copied into a temp project, rendered
with the real CLI (`pendon run -F`), and compared byte-for-byte
(`apps/cli/tests/syntax_spec.rs`). See `docs/guides/testing.md`.
