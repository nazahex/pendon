Sandbox for trying Pendon CLI quickly.

Quick start

- Ensure Rust toolchain is installed.
- From repo root, run one of the scripts below.

Demo to terminal

```bash
bash sandbox/run.sh
```

Generate output files

```bash
bash sandbox/run_files.sh
```

Outputs will be written to `.temp/`:

- `.temp/basic.json`
- `.temp/multiline.json`
- `.temp/file-input.json`
- `.temp/heading-fence.json`
- `.temp/blank-run.json`
- `.temp/blank-run-strict.json`
- `.temp/blank-run-strict.exit` (exit code)
- `.temp/blank-run-strict.stderr` (stderr, if any)

Capabilities

- Basic: concatenate text to JSON IR.
- Multiline: preserves newlines as text ("\n").
- Guards + strict: emit diagnostics as errors and exit non-zero, while still printing JSON.
- Heading/CodeFence: parser hints are emitted internally; renderer still concatenates text.
