# Pendon Advanced Diagnostics RFC

This RFC proposes improvements to Pendon's diagnostic system to make it more robust, user-friendly, and informative, while ensuring high utility for both end-users and tooling integrations.

## Current State

A diagnostic is represented by the event `Event::Diagnostic` in
`crates/core/src/event.rs`:

```rust
Diagnostic {
    severity: Severity,   // Warning | Error
    message:  String,
    span:     Option<Span>,   // Span { start: usize, end: usize } — byte offsets
}
```

Concretely, today:

- Diagnostics are pushed by the core parser (`crates/core/src/parser.rs` — the
  `max_doc_bytes` / `max_line_len` / `max_blank_run` guards and
  `crates/core/src/validate.rs`) and by plugins (`plugin-bind`,
  `plugin-micromatter`, `plugin-image`, `plugin-cite`, `plugin-marker`,
  `plugin-directive`, `extra::emit`, …).
- The `span` field **exists but is never populated**: every emitter in the
  workspace passes `span: None`, so no diagnostic currently carries a source
  location. Adding locations is therefore a producer-side task, not merely a
  formatter task.
- The AST renderer collects diagnostics into a document-level `diagnostics`
  list (`crates/renderer-ast/src/builder.rs`) but they are never surfaced in
  any rendered output.
- The `events` renderer serializes diagnostics as `{ severity, message }`
  (dropping `span` entirely).
- The CLI uses diagnostics **only** to choose the exit code
  (`apps/cli/src/main.rs`: any `Severity::Error` ⇒ exit `2`); it prints nothing
  on stderr for them.
- `--strict` **is** wired end-to-end (`apps/cli/src/cli.rs` →
  `main.rs` → `Options.strict` → `pendon_core::parse`), but it only escalates
  the core parser's three limit guards from `Warning` to `Error`.
  Plugin-emitted diagnostics ignore `--strict` and keep their fixed severity.

As a result, users see no feedback when Pendon encounters recoverable problems,
leading to silent failures and confusion.

## Goals

1. **User-Friendly**: Provide clear, actionable diagnostic messages with source code context (file, line, column, snippet).
2. **Tooling-Friendly**: Ensure diagnostics are available in machine-readable formats (JSON, etc.) for editor integration.
3. **Consistent**: Unify diagnostic emission across plugins and the core parser.
4. **Configurable**: Allow users to control diagnostic output verbosity and format.
5. **Backwards Compatible**: Do not break existing workflows that rely on the event stream or exit codes.

## Must-Have Features

These features are essential for a basic, usable diagnostic system.

1. **Human-Readable CLI Output**
   - The CLI shall print diagnostics to stderr in a standard format, e.g.:
     ```
     error: [plugin-bind] unresolved variable `$foo`
       --> input.md:10:5
        |
     10 |     Some text with {{$foo}} here
        |     ^^^^^^^
     ```
   - Include file name (or "stdin"), line number, column number, severity, diagnostic message, and a short snippet of the source line with the problematic range highlighted.
   - Group diagnostics by file when processing multiple files (via config-driven runs).

2. **Source Location Accuracy**
   - The `Span` field already exists on `Event::Diagnostic` but is **never
     populated** today. The first step is therefore producer-side: make the core
     parser and high-value plugins attach real byte offsets (`Span { start, end }`)
     when they emit a diagnostic. Without this, no location can be shown.
   - Once spans exist, resolve them to line/column (1-indexed, as editors expect)
     in the CLI, which has the source text (from `--input` or stdin). Byte
     offsets must be mapped against the _original_ source, so this belongs next to
     where the source is read, not in a renderer that only sees the event stream.
   - The span should point at the offending source construct (e.g. the
     `@@type{…}` head, the `{{$foo}}` interpolation, the over-long line).

3. **Diagnostic Escalation with `--strict`**
   - `--strict` is already wired for the core parser's limit guards
     (`max_doc_bytes`, `max_line_len`, `max_blank_run`), which it promotes from
     `Warning` to `Error`. The gap is that **plugin-emitted diagnostics ignore
     `--strict`** and always keep their fixed severity.
   - Extend escalation so that under `--strict` _all_ diagnostics (including
     plugin `Warning`s) count as errors for the purpose of the exit code.
   - Keep the event stream severity unchanged (so the recorded event still
     reflects the producer's intent); apply the escalation at the CLI's
     exit-code / presentation layer.

4. **Preserve Diagnostics in IR**
   - Ensure diagnostics remain in the event stream and are included in the `events` renderer output (already present).
   - Ensure the AST renderer continues to collect diagnostics at the document level (already implemented).

5. **Basic Diagnostic Categories**
   - While not requiring a full diagnostic code system initially, encourage plugins to use consistent phrasing and consider adding diagnostic codes in the future (see Should-Have).

## Should-Have Features

These features significantly improve the diagnostic experience and are important for a polished release.

1. **Diagnostic Codes**
   - Introduce diagnostic codes (e.g., `P0001`, `P0002`) that are stable across versions and can be referenced in documentation.
   - Include the code in CLI output (e.g. `` error[P0001]: unresolved variable `$foo` ``) and in structured formats.
   - Allow users to disable specific codes via `--warn-diagnostic-code P0001` or similar (see Nice-to-Have for granular control).

2. **Multiple Output Formats**
   - Add a `--format diagnostic` (or `--diagnostic-format`) option that outputs diagnostics in a structured JSON array (suitable for tooling), while suppressing normal rendering output.
   - Example JSON structure:
     ```json
     [
       {
         "file": "input.md",
         "range": { "start": { "line": 10, "column": 5 }, "end": { "line": 10, "column": 14 } },
         "severity": "error",
         "code": "P0001",
         "message": "unresolved variable `$foo`",
         "snippet": {
           "text": "Some text with {{$foo}} here",
           "highlight_start": 10,
           "highlight_end": 23
         }
       }
     ]
     ```

3. **Diagnostic Suppression**
   - Allow users to suppress diagnostics of a certain severity (e.g., `--no-warnings`) or by code (as mentioned above).
   - Respect existing mechanisms like disabling a plugin (which prevents its diagnostics from being emitted).

4. **Integration with Language Servers (LSP)**
   - Design the diagnostic output to be compatible with the Language Server Protocol (LSP) diagnostics section, enabling future VS Code or other editor plugins to display Pendon diagnostics inline.
   - This does not require implementing an LSP server now, but ensuring the JSON output can be easily mapped.

## Nice-to-Have Features

These features are valuable but can be deferred to later iterations.

1. **Diagnostic Help**
   - Provide a `--explain <code>` flag that prints detailed documentation for a given diagnostic code, including examples and possible fixes.

2. **Incremental Diagnostics**
   - For watch mode (`pendon --watch`), only re-run diagnostics on changed files and update the diagnostic display accordingly.

3. **Rich Diagnostic Information**
   - Allow plugins to attach additional structured data to diagnostics (e.g., suggested replacements, related diagnostic codes) that can be rendered in CLI or tooling.

4. **Suggestion Engine**
   - For certain diagnostics (e.g., unknown plugin name), suggest similar names based on Levenshtein distance.

5. **Diagnostic Categories**
   - Group diagnostics by category (e.g., "syntax", "reference", "binding") and allow filtering by category.

## Open Questions

1. **Output Stream for Diagnostics**
   - Should diagnostics go to stderr or stdout? Traditionally, compilers like gcc/clang send diagnostics to stderr, while stdout is reserved for the compiled output. Pendon's current behavior sends rendered output to stdout and nothing to stderr (except panic messages). We propose sending diagnostics to stderr to keep stdout clean for piped output.

2. **Default Verbosity**
   - Should the CLI display diagnostics by default (even without `--strict`)? Yes, because users need to know about problems. However, we must ensure that the output does not interfere when Pendon is used in a pipeline where stdout is captured. Since diagnostics go to stderr, this should be safe.

3. **Span Granularity**
   - Today no producer populates `span`, so the immediate question is scope: do we
     require every diagnostic to carry a span, or only the high-value ones (parser
     limits, plugin diagnostics that reference a specific construct)? A pragmatic
     answer is to start with the producers that can cheaply compute a span and
     fall back to "no location" otherwise.
   - Should we also store line/column directly in the event to avoid recomputation?
     We lean toward **no**: keep the byte-offset `Span` (already in the schema) and
     resolve to line/column lazily in the CLI, which has the source text. Renderers
     only see the event stream and have no source to map against.
   - Extending `Event::Diagnostic` with line/column would change the event
     structure; we avoid that for backwards compatibility.

4. **Diagnostic Codes Format**
   - Should diagnostic codes be numeric (P0001) or hierarchical (PENDON0001)? We'll use a simple prefix like `P` followed by four digits for simplicity.

## Adoption Strategy

- Phase 1: Implement Must-Have features. First make high-value producers attach
  spans, then print human-readable diagnostics to stderr, resolve spans to
  line/column in the CLI, and extend `--strict` escalation to plugin
  diagnostics. This alone removes the current silent-failure behaviour.
- Phase 2: Add Should-Have features (diagnostic codes, structured/JSON output,
  suppression).
- Phase 3: Add Nice-to-Have features based on community feedback.

## Backwards Compatibility

- The event stream structure (`Event::Diagnostic`) remains unchanged, so existing consumers of the event stream (e.g., the AST renderer) continue to work.
- The CLI's exit-code behaviour is preserved for the default case (exit `2` when
  any `Severity::Error` diagnostic is present). The change is confined to
  `--strict`, which additionally treats `Warning`s as errors for exit-code
  purposes; this is explicitly requested by the flag and documented in
  `docs/guides/cli.md`.
- `--strict`'s existing behaviour for the core parser's limit guards
  (`Warning` → `Error` at parse time) is unchanged; the new work only _extends_
  escalation to plugin diagnostics at the CLI layer, which today do not happen.
- Renderers that do not currently use diagnostics (HTML, solid, JSON) will continue to ignore them, so no change in output format.
- The `events` renderer will continue to output diagnostics as part of the event stream (no change).
- The AST renderer will continue to include diagnostics at the document level (no change).

By following this plan, we can improve Pendon's diagnostics while maintaining compatibility with existing integrations.
