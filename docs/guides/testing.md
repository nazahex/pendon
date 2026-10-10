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
cargo test -p pendon --test syntax_spec          # 17 fixtures
cargo test -p pendon --test ultimate_freeze_spec # sandbox/ultimate baseline
```

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

## Generated files

- Never revert unrelated user changes.
- Do not commit generated output merely because a command rewrote it unless the
  repo convention or task requires it.
- `sandbox/**` is excluded from biome/dprint (fixture data); `docs/spec/golden/**`
  is not, so its generated `.jsx` may surface lint noise — pre-existing and
  unrelated to behaviour.
