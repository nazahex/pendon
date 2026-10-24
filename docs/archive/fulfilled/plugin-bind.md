# RFC: `plugin-bind` — data blocks as typed Solid props

- Status: **§2.1–§2.7 implemented and green** — phase 1 (blocks, `$var`, nested
  values, the `U+E001` transport) plus **paths**, **merge/override** and
  **external files**. The `OPEN-BIND-1..4` items of `SYNTAX.md` §15 were decided
  here and have landed; §2.8 lists what is **declined**. Phase 2 (`mdp`) is
  specified but not built. Live lines: `STATUS.md`.
- Owns: the `{{{lang[var] … }}}` data block (body payload and file payload) and
  the `$var` reference.
- Depends on: `crates/extra` (extras heads), `pendon-core` (`JSON_ATTR_PREFIX` /
  `JSX_ATTR_PREFIX`), `renderer-ast` (re-hydration), `renderer-solid` (`={…}`).

This RFC transforms JSON, JSONC, YAML, TOML, Pendon Markdown (MDP) and CSV data
written inside `{{{ … }}}` into a **valid single-line JSON** carried on a Solid
JSX property value. It is deliberately **Solid-first**: other renderers keep a
stringly-typed fallback (see §2.4).

## 1. Problem

An author wants to attach structured data to a component as a real JavaScript
value:

```jsx
<Chart data={[{"id":1,"tags":["admin"]},{"id":2,"tags":[]}]} />
```

Writing that JSON inline inside an extras head `{data: …}` is unreadable and
collides with the `{…}` head grammar: nested braces, newlines, commas and quotes
would all have to be escaped inside a single head. So the payload is declared
**once** in a fenced data block and **referenced** by name from any extras head.

## 2. Syntax (normative text now lives in `SYNTAX.md` §19)

### 2.1 Defining data

```text
bind_block   = "{{{" lang "[" var "]" body "}}}"
lang         = "json" | "jsonc" | "yaml" | "yml" | "toml" | "csv" | "mdp"
var          = ALPHA { ALPHA | DIGIT | "-" | "_" }
```

- `lang` and `var` are **required**.
- `var` must start with a letter (`ALPHA`); a leading digit is never a name, so
  `$100` is text. That rule is what makes §2.2's "no escape hatch is needed"
  sound, so it is not cosmetic. (**decided and implemented** — `OPEN-BIND-1`;
  the rule is enforced by `is_valid_var`, and no fixture ever used a
  digit-leading name, so nothing had to be re-frozen.)
- `var` must be unique across the document; a duplicate warns and **last wins**
  (mirroring the §6.4 last-wins rule for duplicate extras keys).
- A block **must** begin at the start of a line (optionally indented).
- **Block form** — the body starts on the line after the head and ends at the
  first line whose trimmed content is exactly `}}}`. This is the only form the
  reference in `sandbox/bind/bind-main.sb.md` uses.
- **Inline form** — a body on the head line itself (`{{{mdp[v] … }}}`) is only
  legal for `mdp` (§2.3). Any other language rejects the block.
- If `lang`/`var` are missing or invalid, the block is **dropped and not
  rendered**, with a `Warning`. Rationale: a leaked payload is parsed as prose
  and can corrupt the whole document (see §3 — a `#` line becomes a heading), so
  falling back to source text is _less_ safe than dropping.
- An empty body binds `null`, with **no** warning.
- A `{{{` block nested inside a body is literal text; the first terminator line
  ends the outer block.

### 2.2 Consuming data

There is exactly **one** way to consume a bound value: as the value of a `k: v`
extras item, written `$var`.

```md
@@{ dataT: $f21 }
```

- `$var` is a reference only when it is the **entire** leaf value (after
  whitespace, quoted or unquoted). `prefix$var` and `$var+` are plain strings —
  there is **no free-text interpolation**. `$var.x` is a **path** into the bound
  value (§2.5). (The first draft of this RFC stated that `$a.b` is a plain string
  _by decision_; §2.5 reverses that on purpose — the two statements must not be
  quoted against each other.)
- Refs resolve inside nested extras values too:

  ```md
  ===directiveCSV("Tabel"){ tabel: { karyawan: $tabel21-karyawan4, foo: { fooYi: $foo_yi } } }
  ```

- An undefined `$var` warns and stays literal `$var` (never silently `null`).
- A `$` that does not form a reference is untouched.
- **No `$` escape — and none needed.** A reference must start with `ALPHA`
  (§2.1), so `$100`, `$1.50` and `$ 5` are ordinary text: a currency or price
  value is written as-is (`price: $100`, `price: "$100"`). Quoting is **not** an
  escape — the value is unquoted before the reference check, so `price: "$100"`
  raises exactly the same unbound warning as `price: $100` today (verified with
  the real CLI). Quoting is for whitespace and separators. The one shape that
  could re-open this question is a _path-looking_ literal (`$a.b`), which §2.5
  must answer.

### 2.3 MDP (`mdp`) — phase 2

For `mdp`, the body is Pendon Markdown, not a data serialization:

- Written on the head line ⇒ **inline** (no `<p>` wrapper); written on the next
  line ⇒ **block**.
- At resolve time the fragment is rendered through the document's inline
  pipeline and reaches Solid as a raw JSX fragment via `JSX_ATTR_PREFIX`
  (`prop={( … )}`), so the markup parses as JSX instead of being stringified.

MDP is specified here but **not implemented in phase 1**; a phase-1 build warns
and drops the block. See §7. A `(…)` group after `]` is **not** a file path for
`mdp` (§2.7 decision 10), so an `mdp` head-line body keeps this meaning.

### 2.4 Render

The bound value reaches the prop as a JavaScript expression:

```jsx
<Element data={[{"id":1}]} />
```

`renderer-solid` emits `={…}` for objects/arrays/numbers, `={false}` / `={null}`
for those scalars, and a bare flag for `true`. `renderer-ast` re-hydrates the
transport encoding into a real JSON value (see §4). `renderer-html` / `-json`
strip the sentinel and keep the compact JSON string — binding is a Solid
contract, and this fallback is documented, not accidental.

### 2.5 Path into a bound value — **decided and implemented** (`OPEN-BIND-2`)

Binding a whole payload (`k: $data`) cannot express "one leaf" — and the
motivating sandbox already hand-writes nesting to work around it. A **path**
extends a reference:

```text
ref   = "$" var { "." key | "[" index "]" | "[" dquote key dquote "]" }
key   = var                     ; a dotted key obeys the `var` character set
index = DIGIT+
```

```md
@@anchorA{ db: $config.db.host, first: $rows[0].name, odd: $m["key-with-dash"] }
```

- **Semantics.** Look up the root `var`, then walk left to right. A missing key,
  an index out of range, or a type mismatch warns and leaves the reference
  **literal** — the same rule as an unbound `$var` (§2.2), never `null`. The
  warning names the whole reference and the root it walked
  (`` `$rows[9].name` found no value in `$rows` ``); a bare miss keeps the phase-1
  wording.
- **Scope limits.** Dotted keys, integer indices and quoted keys **only**. No
  wildcards (`[*]`), no filters (`[?()]`), no recursive descent (`..`), no slices.
  Each of those drags in an expression language, an iteration semantics and an
  error model; the requirement is "reach one leaf", not "query". A shape outside
  the grammar (`$a.`, `$a[*]`, `$a..b`) is **not** a reference at all: it stays
  untouched text and raises nothing, exactly like `prefix$var`.
- **Decided — index on an object.** An integer index addresses an **array** only.
  On an object it is a miss, so `$m[0]` cannot silently read the numeric **string**
  key `"0"` (`m: { "0": … }` — reachable as `$m["0"]` if that is really meant).
- **Decided — the path-shaped literal.** `$a.b` becomes a path, so a document
  meaning the literal text `$a.b` changes meaning. There is no escape hatch by
  decision (§2.2), and the break is accepted: no escape is worth adding for a
  string that was never valid data. Because everything here is still alpha, it is
  not recorded in a changelog.
- **Dotted keys are `var`-shaped.** A step after `.` must start with a letter and
  may then use letters, digits, `-` and `_` — the same rule as `var` itself. A key
  like `123_numeric_key` or `a b` therefore needs the quoted form
  `["123_numeric_key"]`. One rule instead of two.
- **A quoted key holds anything but `]`.** `["a-b"]` is scanned to the first `]`,
  with no escape processing; a key containing `]` is out of scope.
- **Backwards compatibility.** `.` and `[` are not in `var`, so the extension is
  unambiguous once defined, and a plain `$var` keeps its meaning.

### 2.6 Merge / override — **decided and implemented** (`OPEN-BIND-3`)

A component usually wants _defaults plus this document's overrides_. Without a
merge, that is a list of paths re-stating the same object:

```md
@@anchorA{ theme: { accent: $brand.accent, mode: $brand.mode, scale: 1.2 } }
```

The spread item inside a nested `{…}` value:

```text
pairs  = ( pair | spread ) { ws "," ws ( pair | spread ) } [ ws "," ]
spread = "..." ws ref            ; `ref` is §2.5, so `...$brand.accent` works too
```

```md
@@anchorA{ theme: { ...$brand, scale: 1.2 } }
```

- **Semantics.** `...$ref` contributes the bound object's keys as **defaults**.
  Shallow: `...$a` overrides a whole sub-object rather than merging into it. A
  deep merge needs per-type policy (arrays: append or replace? `null`: delete or
  set?), which is a different feature.
- **Precedence — the one place this RFC revised its own recommendation.** The
  earlier draft said "item order decides precedence, left to right, later wins",
  copying §6.4. The transport cannot honour that (see below), so the rule was
  decided and implemented as: **a key written explicitly in the object always wins
  over a spread**, and among several spreads the **later** one wins. `{...$base,
  a: 2}` and `{a: 2, ...$base}` therefore both give `a = 2`. This is the behaviour
  the feature exists for ("defaults plus this document's overrides"); it is
  recorded here as a decision, not as an accident.
- **Transport — why precedence had to be defined that way.** `to_attributes`
  hands a nested value to `resolve` as **compact JSON**, and this workspace's
  `serde_json` has no `preserve_order` feature, so a JSON object is emitted with
  its keys **sorted**: position inside `{…}` is gone before `plugin-bind` sees it.
  The spread therefore travels under a **reserved key** — `SPREAD_KEY` (`"..."`)
  in `crates/extra/src/value.rs` — whose value is a JSON **array** of reference
  strings in source order. `...` can never be an authored prop key (`is_key`
  requires a leading `ALPHA` or `_`), so the key cannot collide with real data, and
  the reference grammar itself stays in `plugin-bind` (extra only transports).
- **Decided — spreading a non-object.** A bound value that is not an object
  (`...$rows` where `rows` is an array), an unbound name, or malformed reference
  text: `Warning`, **the spread item is ignored**, the object's other items still
  apply. Rationale: this is a semantic type error rather than a parse error, so
  §4.3's all-or-nothing rule (which protects against half-parsed syntax) does not
  apply, and dropping the whole head would punish the author for one bad default.
- **Spread data is data, never walked.** `$…`-looking strings _inside_ the bound
  object are copied verbatim; only the author's own values are resolved. Otherwise
  a payload containing the text `$x` would be silently rewritten.
- **Placement.** A spread is an item of a nested `{…}` **object value** and nothing
  else: `a: ...$x` is still the `Raw` scalar it always was, and `[...$x]` is
  untouched. The grammar is additive, so no existing document changes meaning.
- **Without `bind`.** The value degrades to a valid JSON object carrying the
  `"..."` key (e.g. `{"...":["$brand"],"scale":1.2}`); the feature is a `bind`
  feature, documented as such, not a new extras-head semantic.

### 2.7 External data files — **decided and implemented** (`OPEN-BIND-4`)

**Syntax.** A `( … )` group after `]` takes the payload from a file; the block is
then **bodyless**:

```text
bind_block = "{{{" lang "[" var "]" [ "(" path ")" ] "}}}"      # file form
           | "{{{" lang "[" var "]" body "}}}"                  # body form (§2.1)
```

```md
The table below is generated from the fixture file:

{{{csv[rows](../src/fixtures/rows.csv)}}}

[Rows](/docs)@@anchorA{ meta: { rows: $rows, n: 2 } }
```

Everything else — `lang`, `var`, `$var`, uniqueness, last-wins, the `U+E001`
transport — is unchanged; only the payload's origin differs.

#### Why this shape

- **The payload is per-document.** The earlier position ("per-file overrides come
  from that directory's `pendon.toml`") cannot express _this_ page's table: it
  would need a new path-keyed config convention, and it splits the declaration
  from its use site. `( … )` keeps both in the document, like every other extras
  declaration.
- **`( … )` reuses an existing precedent.** The positional group is already the
  Pendon way to put a positional argument before the extras head (§6.1,
  `@@type[…](…){…}`; ADR-0003). A `src:` _item_ inside the block body was
  considered and rejected: the body is the payload, so a `src:` line would be
  parsed as data (`src: foo` is valid YAML/TOML) — ambiguous by construction.
- **Additive and unambiguous.** Immediately after `]` the only legal byte today
  is a newline (block form) or a terminator (`mdp` inline form), so `(` cannot
  reinterpret an existing document.

#### Decisions (all recorded **and implemented**)

1. **Bodyless inline terminator.** `{{{csv[rows](../src/x.csv)}}}` closes the head
   with `}}}` on the same line: a same-line `}}}` is legal iff a path is present
   (an empty body). The alternative — a terminator line after an empty body — is
   pure noise and was rejected.
2. **Path _and_ body.** Ambiguous intent: `Warning` + **the block is dropped**, and
   the scan consumes up to the `}}}` line so the payload never leaks as markup.
   (Same treatment for trailing text after the `)`.)
3. **Resolution base = the source file's directory, never the process CWD.**
   Implemented as `extract_with_base(source, base)`; the base-less `extract(source)`
   is the stdin/in-memory entry point. Because the base is the _document's_ own
   directory, a relative path resolves identically at **any depth** of
   `src/…` — `../../data/rows.csv` from `src/deep/page.md` is verified by a golden,
   a unit test and `run`-mode cache invalidation. Both CLI call sites pass the
   directory: `apps/cli/src/process.rs` (a task file) and `apps/cli/src/main.rs`
   (a `--input` file; `None` in stdin mode).
4. **stdin has no directory** → hard `Severity::Error` (a silent CWD fallback would
   contradict (3)). No `--data-base` flag was added; the error tells the author to
   pass the source as a file. Consequence of the moment: an `Error` sets the CLI
   exit code (2) and appears in the `events` / `json` IR, which is where the CLI
   surfaces diagnostics today — a `solid` render shows no message.
5. **`../` and absolute paths are allowed.** This is a build tool reading the
   author's own tree, like `[task.import]`. It is **not a security boundary**, and
   the docs say so rather than pretending otherwise.
6. **`lang` vs. file extension.** `lang` is authoritative (the payload may live in
   a `.txt` file). If the extension is a known data extension _and_ disagrees with
   `lang`, warn — a typo guard, not a rule. `.json`/`.jsonc` both accept `json` and
   `jsonc`; elsewhere the match is exact; an unknown extension is silent.
7. **Diagnostics.** Missing, unreadable, directory-instead-of-file or non-UTF-8
   file → `Warning` + block dropped, consistent with an unparseable body payload.
   A `..`-escaping path is **not** warned (see (5)).
8. **Cache invalidation — done, not forgotten.** `Extract.external_files` lists
   every file read, and `apps/cli/src/process.rs` appends them to the task's `deps`
   before the cache entry is created, so editing `src/fixtures/rows.csv` re-renders
   the page. Verified in `run` mode: the file appears in `.pendon/cache.toml` with
   its hash and a plain `pendon run` (no `-F`) picks the change up. A file is
   recorded as soon as it is **read**, even when its payload then fails to parse,
   because fixing it must re-render.
9. **Frontier, rejected.** Defaulting the payload to the file and dropping `lang`
   (`{{{[rows](../src/rows.json)}}}`) is **not** in: explicit parsing is the
   phase-1 contract and a missing `lang` is one of the few well-tested error paths
   (§2.1).
10. **`mdp` is exempt from the `(…)` form.** For `mdp` the head-line body keeps its
    meaning, so `{{{mdp[x](hello) }}}` is still an inline `mdp` body (which phase 1
    warns and drops) rather than a file read: the two readings are otherwise
    indistinguishable, and `mdp` file payloads belong to the phase-2 `mdp` work.
    A `(` is also accepted after optional whitespace, not only adjacent to `]`.

### 2.8 Non-goals — **declined**

Recorded so they are not re-proposed (each was considered and rejected):

- **`\$` escape.** Unnecessary: a reference must start with `ALPHA`, so `$100`
  and `$1.50` are text, and a currency/price value is written literally
  (§2.2). The only shape that could re-open the question is a path-looking
  literal (`$a.b`), which §2.5 answers.
- **Frontmatter / micromatter as a payload source.** A payload declaration must
  be visible at its point of use; a second, document-head location would make
  "where does `$this` come from?" a two-file question, and Pendon's extras model
  is explicitly local.
- **Free-text interpolation** (`Total: $sum`). Only entire extras values are
  resolved (§2.2); interpolating into prose needs escaping, typing and an error
  model that the extras path avoids.
- **Reactive / runtime binding.** `bind` is a **build-time** transform: the output
  is a static JSX value. No signal, no re-resolution, no partial hydration
  contract.
- **Type-checking a bound value against a component's props.** A mismatch is the
  consuming component's concern, not the binder's.
- **Fetching data over the network.**

## 3. Pipeline placement — extraction happens **before** `parse`

A payload contains lines that the core lexer treats as markup: a TOML `# comment`
is a Heading, a CSV `-` is a list, `|` is a table, blank lines separate blocks.
Confirmed with the real CLI:

```text
$ printf '# Head\n\n{\n "a": 1\n}\n\n# comment\n' | pendon --format ast
{"type":"Document","children":[{"type":"Heading", …}]}
```

Therefore a post-parse text scan is impossible. `plugin-bind` adds a new stage:

```text
source text
  │  plugin-bind::extract      (pre-parse, text → text + Registry)
  ▼
clean text (blocks replaced by one blank line)  +  Registry { var → Value }
  │  pendon_core::parse
  ▼
Event stream (extras already carry literal "$var" strings)
  │  the other plugins (markdown, blockquote, directive, table, …)
  ▼
plugin-bind::resolve           (post-markdown, events → events)
  ▼
Event stream with JSON-prefixed attributes
  │  renderer
  ▼
output
```

Ordering contract: with `bind` enabled the CLI runs `extract` **before**
`pendon_core::parse` and `resolve` **after** every other plugin. `bind` is a
no-op inside the plugin loop. This is the first Pendon stage that runs on raw
text; it is recorded as ADR-0004.

## 4. Value model and transport

Every payload is parsed into a `serde_json::Value` — a normalized JSON-only
superset so resolution never has to know the source language.

| `lang`  | parse                                                                                                               |
| ------- | ------------------------------------------------------------------------------------------------------------------- |
| `json`  | `serde_json::from_str`                                                                                              |
| `jsonc` | strip `//` and `/* … */` comments outside strings, then `serde_json`                                                |
| `yaml`  | `serde_yaml` → JSON; merge keys (`<<`) expanded, `inf`/`nan` → `null`                                               |
| `toml`  | `toml::from_str`, converted (datetimes → RFC 3339 strings, `inf`/`nan` → `null`)                                    |
| `csv`   | RFC 4180 rows; header row names the keys; a cell that is valid JSON becomes a value, else a string; `NULL` → `null` |
| `mdp`   | phase 2 (§2.3)                                                                                                      |

Transport reuses the two Private-Use-Area sentinels already defined in
`crates/core/src/event.rs`:

- `JSON_ATTR_PREFIX` (`U+E001`) + compact JSON — an encoded JSON literal.
  `renderer-ast` strips it and re-hydrates a real `Value`; `renderer-solid`
  emits `={…}`.
- `JSX_ATTR_PREFIX` (`U+E002`) + fragment — a raw JSX expression (MDP); Solid
  emits `prop={( … )}`.

Only `plugin-bind` may emit either sentinel. `U+E000` remains the unrelated
pre-markdown protect sentinel (§12.1).

The payload's **origin** does not change the value model: a body payload (§2.1)
and a file payload (§2.7) both become a `serde_json::Value` before resolution, so
`resolve` never learns where a value came from.

## 5. Algorithm

### 5.1 `extract`

1. Scan line by line for a line whose trimmed start is `{{{`.
2. Parse `lang` (up to `[`), then `var` (up to `]`); anything malformed drops
   the block with a warning.
3. Block form: collect lines until a `}}}`-only line; an EOF without a
   terminator drops the block with a warning. Inline form: only `mdp`. With the
   file form (§2.7) there is no body: the file is read at this step, resolved
   against the **base directory** the caller passes (`extract_with_base`; `None`
   means stdin and is a hard `Error`), and each path read is recorded in
   `Extract.external_files` so the CLI can add it to the task's dependency set.
4. Parse the body (or the file contents) per §4. On a parse error, warn and drop
   the block.
5. Insert `var → Value` (last wins) and remove the block lines, leaving exactly
   one blank line so the surrounding blocks stay separate.

### 5.2 `resolve`

Walk the event stream once; for every `Event::Attribute`:

1. Exactly a reference (§2.2, §2.5) ⇒ replace the value with the transport
   encoding of the value it resolves to, or warn and keep the literal on a miss
   (root unbound, or a path step that did not land).
2. Otherwise, if the value parses as a JSON object/array (a nested extras
   `{…}`/`[…]` value), walk it and:
   - replace every **string leaf** that is exactly a reference;
   - expand a `SPREAD_KEY` (`"..."`) item (§2.6): its reference strings are
     resolved in source order into a defaults map, then the author's own
     (already resolved) keys are merged over it, so an explicit key wins.
     Spread values are copied, never walked. Re-encode with `JSON_ATTR_PREFIX`
     when anything changed.
3. Anything else passes through untouched.

## 6. Diagnostics

| Condition                                                        | Severity                                    |
| ---------------------------------------------------------------- | ------------------------------------------- |
| `{{{` block with missing/invalid `lang` or `var`                 | Warning (block dropped)                     |
| non-`mdp` body on the head line                                  | Warning (block dropped)                     |
| unclosed `{{{` at EOF                                            | Warning (block dropped)                     |
| payload fails to parse for its `lang`                            | Warning (block dropped)                     |
| `mdp` block in a phase-1 build                                   | Warning (block dropped)                     |
| duplicate `var`                                                  | Warning (last wins)                         |
| undefined `$var`                                                 | Warning (stays literal)                     |
| `$a.b` / `$rows[9]` — `a` bound, leaf missing                    | Warning (stays literal)                     |
| path missing, unreadable, a directory, non-UTF-8                 | Warning (block dropped)                     |
| path **and** body in the same block                              | Warning (block dropped)                     |
| `lang` disagrees with a known file extension                     | Warning (extension kept, `lang` used)       |
| external block with no source directory (stdin)                  | Error                                       |
| `...$v` where `$v` is unbound, not an object, or not a reference | Warning (spread ignored, other items apply) |

A `$`-shaped value that is **not** a reference (`$100`, `prefix$var`, `$a.`,
`$a[*]`) raises nothing: it was never a reference. That is what keeps "no `$`
escape hatch" sound (§2.2, §2.5).

## 7. Scope, phases and status

- **Phase 1 — done, green.** `json`, `jsonc`, `yaml`/`yml`, `toml`, `csv`; block
  form; `$var` in extras values, nested values included; the `U+E001` transport;
  the `ALPHA` rule for `var` (`OPEN-BIND-1`); golden `23-bind`.
- **Paths, merge/override, external files — done, green.**
  `OPEN-BIND-2` (§2.5), `OPEN-BIND-3` (§2.6), `OPEN-BIND-4` (§2.7): decided in
  this document, implemented in `crates/plugin-bind` (+ the `SPREAD_KEY`
  transport in `crates/extra`, + the `deps` recording in `apps/cli`), and frozen
  end-to-end by golden `24-bind-paths`.
- **Phase 2 — `mdp`.** Needs pinned: the inline-pipeline entry point, `U+E002`
  handling in `renderer-ast`, the escaping policy, the error behaviour (§2.3) and
  whether `mdp` gets a `(…)` file payload at all (§2.7 decision 10).
- **Deferred — `[task.bind]`.** No config keys are needed; the table stays
  reserved.
- **Declined.** `\$` escape, frontmatter/micromatter source, free-text
  interpolation, reactive/runtime binding, prop type-checking — see §2.8.

## 8. Brief example

Fuller examples live in `sandbox/bind/bind-main.sb.md`. Executable ones are the
goldens: [`23-bind`](../spec/golden/23-bind.mdp) (blocks, `$var`, nested values)
and [`24-bind-paths`](../spec/golden/24-bind-paths.mdp) (paths, spread/merge, a
file payload read through `../data/`).

```md
# Bind Sandbox

@@{ dataT: $f21 }
Amet magna do {minim} est $eu$ pariatur.

{{{json[data-X]
{
"numbers": [ 0, -42, 3.14, 1e5 ],
"primitives": { "b": true, "n": null, "arr": [] },
"users": [ { "id": 1, "tags": [ "admin" ] } ]
}
}}}

@@{ dataX: $data-X }
Eiusmod qui sunt labore nisi.

{{{csv[tabel21-karyawan4]
id,nama,meta,aktif
1,"Budi, S.T.",{"role":"admin"},true
2,"Siti ""B""","[""qa""]",false
}}}

===directiveCSV("Tabel Karyawan"){ tabel: { karyawan: $tabel21-karyawan4 } }
```

```jsx
<Element data={[{"id":1,"tags":["admin"]}]} />
```
