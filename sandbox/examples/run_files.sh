#!/usr/bin/env bash
set -euo pipefail

OUT_DIR=".temp"
mkdir -p "$OUT_DIR"

echo "==> Building workspace (debug)"
cargo build --workspace

BIN="target/debug/pendon"

echo "==> Writing outputs to $OUT_DIR"

"$BIN" --format json --input sandbox/examples/basic.md > "$OUT_DIR/basic.json"
"$BIN" --format json --input sandbox/examples/multiline.md > "$OUT_DIR/multiline.json"
"$BIN" --format json --input sandbox/examples/multiline.md > "$OUT_DIR/file-input.json"
"$BIN" --format json --input sandbox/examples/heading-fence.md > "$OUT_DIR/heading-fence.json"
"$BIN" --format json --input sandbox/examples/blank-run.md > "$OUT_DIR/blank-run.json"

set +e
"$BIN" --strict --max-blank-run 1 --format json --input sandbox/examples/blank-run.md > "$OUT_DIR/blank-run-strict.json" 2> "$OUT_DIR/blank-run-strict.stderr"
code=$?
set -e
printf "%s\n" "$code" > "$OUT_DIR/blank-run-strict.exit"

echo "==> Writing AST outputs"
"$BIN" --format ast --input sandbox/examples/heading-fence.md > "$OUT_DIR/heading-fence.ast.json"
"$BIN" --plugin markdown --format ast --input sandbox/examples/heading-fence.md > "$OUT_DIR/heading-fence.plugin.ast.json"

echo "Done: outputs in $OUT_DIR"
