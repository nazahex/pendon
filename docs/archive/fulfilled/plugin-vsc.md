# Pendon VS Code Plugin RFC

This RFC describes the proposed VS Code extension for Pendon, a Markdown-as-DSL engine. Pendon canonical extension is `.mdp`; `.md` remains supported.

## Must-Have Features

0. **Custom Icon for .mdp Files**

1. **Syntax Highlighting**
   - Properly highlight Pendon's extended Markdown syntax, including:
     - `@@type{...}` typed-extras heads
     - Decorators (lines starting with `@@` that apply to the next block)
     - Markers (e.g., `%%` for comments, if implemented)
     - Directives (e.g., `:::info{...}`)
     - Tables, lists, blockquotes, sections with Pendon-specific attributes
   - Fallback to standard Markdown highlighting for core constructs when Pendon syntax is not recognized.

2. **Basic Diagnostics**
   - Display errors and warnings from the Pendon parser (via the `pendon` CLI) in the VS Code Problems pane.
   - Examples: unknown plugin names, malformed attribute blocks, unmatched fences.

3. **Command Palette Integration**
   - Provide commands to run Pendon on the current file or workspace:
     - `Pendon: Render Current File`
     - `Pendon: Render with Format` (HTML, Solid/JSX, JSON, AST, etc.)
     - `Pendon: Watch and Render` (for live preview during development)

## Should-Have Features

1. **Live Preview Panel**
   - A webview panel that shows the rendered output (defaulting to HTML) of the current Pendon file.
   - Updates automatically on file save (or manually via command).
   - Option to choose the renderer (HTML, Solid, etc.) and format (pretty, minified).

2. **Configuration Support**
   - Read VS Code settings for Pendon CLI options (e.g., `--format`, `--pretty`, plugin enables/disables).
   - Allow specifying a custom path to the `pendon` binary (for development vs. installed versions).

3. **Completion Suggestions**
   - Suggest Pendon plugin attribute names and values based on the current file's configuration and enabled plugins.
   - Example: Suggesting `type` values for `@@type{...}` heads when inside a known plugin's scope.

4. **Code Lens / Lens Actions**
   - Show "Render" links above code fences or blocks that can be clicked to render that specific segment.

## Nice-to-Have Features

1. **Advanced Diagnostics**
   - Inline squiggles and hover messages for Pendon-specific warnings and errors.
   - Integration with the Pendon event stream to show detailed diagnostic information (location, severity).

2. **Refactoring Tools**
   - Rename a component type across all `@@type{...}` heads (with preview).
   - Extract a selected block into a component or directive.

3. **Integration with Pendon Testing**
   - Commands to run Pendon golden tests (`cargo test -p pendon --test syntax_spec`) and show results in VS Code.
   - Ability to update golden fixtures from the test failure view.

4. **Snippets**
   - Provide snippets for common Pendon patterns (e.g., `@@type{...}` heads, decorator blocks, common directives).

5. **Status Bar Indicator**
   - Show the current Pendon version, renderer format, or watch status in the status bar.

6. **Workspace Symbols**
   - Allow searching for Pendon-specific symbols (e.g., component names, directive types) across the workspace.

## Open Questions

1. **Execution Environment**: Should the VS Code plugin bundle a Rust binary, rely on a globally installed `pendon` command, or use a WASM-compiled version of Pendon for in-editor rendering?
   - _Recommendation_: Start by shelling out to the `pendon` CLI (requiring it to be in PATH or configurable) for simplicity.

2. **Preview Security**: How to safely render custom components in the preview webview? Should we isolate them or allow only a safe subset?
   - _Recommendation_: For HTML preview, sanitize output. For Solid/JSX preview, we cannot run arbitrary components in the webview without a full Solid runtime; thus, we may stick to HTML preview for safety and simplicity.

3. **Configuration Scope**: Should Pendon settings be per-workspace, per-folder, or global? How to handle project-specific `pendon.toml` files?
   - _Recommendation_: Respect the existing Pendon configuration cascade (CLI flags > `pendon.toml` > defaults) and allow VS Code settings to override or supplement.

## Adoption Strategy

- Phase 1: Implement Must-Have features (syntax highlighting, diagnostics, basic commands).
- Phase 2: Add Should-Have features (live preview, configuration, completions).
- Phase 3: Add Nice-to-Have features based on community feedback.

## Backwards Compatibility

This extension does not alter the Pendon core or plugins; it is purely a client-side tool. Therefore, it poses no backwards compatibility risks to the Pendon engine itself.
