# Using External Grammars with Syntect

Pendon plugin-codeblock-syntect can load additional grammars (Sublime `.sublime-syntax` or TextMate `.tmLanguage`) for better language coverage like TypeScript/TSX.

How to enable:

- Put grammar files in a folder, e.g. `docs/grammar/syntaxes` or `sandbox/syntect/syntaxes`.
- Set the environment variable `PENDON_SYNTECT_SYNTAX_DIR` to that folder path when running the CLI.
- Alternatively, Pendon will auto-scan common paths (`./syntaxes`, `./sandbox/syntect/syntaxes`, `./docs/grammar/syntaxes`).

Recommended grammars:

- TypeScript: https://github.com/microsoft/TypeScript-TmLanguage (MIT)
- TSX/JSX: community grammars compatible with Sublime/TextMate.

Notes:

- Large grammars are not bundled to keep repository size small.
- If a requested language is missing, Pendon falls back to JavaScript or Plain Text where appropriate.
