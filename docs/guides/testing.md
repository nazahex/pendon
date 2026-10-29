# Testing Guide

Focused validation is preferred after each edit: run the smallest relevant package
test first, then widen to the workspace and sandbox checks.

## Before submitting a change

```bash
cargo fmt --all -- --check
cargo test --workspace
git diff --check
```

For JS/docs hygiene:

```bash
bun run check      # biome
bun run format     # biome + dprint
```

## Focused package tests

```bash
cargo test -p pendon-extra            # the shared @@-head grammar
cargo test -p pendon-plugin-marker    # one plugin
cargo test -p pendon-plugin-list
```

## Golden fixtures (executable truth)

The golden set in [`../spec/golden/`](../spec/golden/) is copied into a temp
project, rendered with the real CLI, and compared byte-for-byte.

```bash
cargo test -p pendon --test syntax_spec          # 35 fixtures
cargo test -p pendon --test extras_heads_spec    # extras on every element head
cargo test -p pendon --test slot_heads_spec      # every slot position on every element
```

`extras_heads_spec` is the **universality** audit (§6.1, D-12): one document per
element head and decorator, all carrying the same head, so a construct that stops
reading a specs position — or falls back to literal text — fails there instead of
in a golden nobody re-read.

`slot_heads_spec` is the same audit one layer down (§5.2, §6.5, D-7): it walks
`Slot::ALL × SlotKind::ALL` against the four slot-contracting elements' contracts,
then runs each element over a head with six distinct values and fails if one value
— or one `Diagnostic` — is missing.

A fixture may also freeze the **files** its render writes
(`docs/spec/golden/NN-name.out/`): an `[[task.<element>.extract]]` block writes
side files instead of markup, and the harness compares each of them byte for byte
too.

When output changes **intentionally**, re-freeze (do not hand-edit `.jsx`):

```bash
cargo run --bin pendon -- run -F     # inside the fixture or sandbox dir
# review the diff — only intentional changes should appear
```

Then explain the contract change in the change description. See
[`../spec/golden/README.md`](../spec/golden/README.md) for the format.

## Sandbox validation

Sandbox sources live in `sandbox/*/src/`; generated output in `out/`. A green test
is not enough if the JSX/HTML structure is wrong — inspect it directly.

```bash
cd sandbox/custom    && cargo run --bin pendon -- run -F
cd sandbox/syntect   && cargo run --bin pendon -- run -F
cd sandbox/universal && cargo run --bin pendon -- run -F
cd sandbox/ultimate  && cargo run --bin pendon -- run -F
```

Verify `out/` matches expectations. When output is part of an integration
contract, include only intentional changes in the commit — do not commit generated
artifacts from `target/`.

Not every sandbox tracks `HEAD`: some predate the current pipeline and are
**maintainer-owned**. Do not re-render those, do not freeze their `out/` into a
golden, and do not read a diff there as a regression in your change; the live
list of which ones are stale (and why) is the "Deferred" item in
[`STATUS.md`](../../STATUS.md). Similarly, `sandbox/number/` is a sketch with no
source and cannot run at all.

## Generated files

- Never revert unrelated user changes.
- Do not commit generated output merely because a command rewrote it unless the
  repo convention or task requires it.
- `sandbox/**` is excluded from biome/dprint (fixture data); `docs/spec/golden/**`
  is not, so its generated `.jsx` may surface lint noise — pre-existing and
  unrelated to behaviour.
