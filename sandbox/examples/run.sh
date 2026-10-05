#!/usr/bin/env bash
set -euo pipefail

echo "==> Building workspace (debug)"
cargo build --workspace

BIN="target/debug/pendon"

echo
echo "==> Basic (stdin)"
cat sandbox/examples/basic.md | "$BIN" --format json

echo
echo "==> Multiline (stdin)"
cat sandbox/examples/multiline.md | "$BIN" --format json

echo
echo "==> File input"
"$BIN" --format json --input sandbox/examples/multiline.md

echo
echo "==> Heading + CodeFence (hints only; renderer concatenates)"
"$BIN" --format json --input sandbox/examples/heading-fence.md

echo
echo "==> Blank-run diagnostic (non-strict)"
"$BIN" --format json --input sandbox/examples/blank-run.md

echo
echo "==> Blank-run strict (exit non-zero, still prints JSON)"
set +e
"$BIN" --strict --max-blank-run 1 --format json --input sandbox/examples/blank-run.md
code=$?
set -e
echo "(exit code: $code)"

echo
echo "Done."
