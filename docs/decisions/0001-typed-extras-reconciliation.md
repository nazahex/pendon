# ADR-0001 — Typed Extras Reconciliation (D1–D10)

- Status: **Accepted / complete** (implemented, documented, frozen, gated)
- Normative text: [`docs/spec/SYNTAX.md`](../spec/SYNTAX.md)
- Evidence: golden fixtures `01`, `02`, `05`, `09`, `21`; `sandbox/ultimate` frozen baseline

This ADR reconciles the parser, the fixtures and the docs with ten syntax
decisions. Each is pinned by a fixture or a unit test; the table records the rule
and where it is enforced.

| #       | Decision                                                                                                                                                                                                                                                                                                                                                                               | Pinned by                   |
| ------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------- |
| **D1**  | A type is `ALPHA ( ALPHA \| DIGIT )*` (ASCII, any case, no symbols). After the type run: `{` ⇒ typed head; whitespace/EOL/EOF ⇒ **type-only head**; any other symbol ⇒ type-only head with the symbol left as literal text. `@@type {…}` (space then `{`) ⇒ **literal**. `@@type-x{.x}` ⇒ type `type` + literal `-x{.x}`. `@@1anchor{.x}` stays literal (a type starts with a letter). | §4.4; golden `02`, `21`     |
| **D2**  | `{}` and `@@{}` are both a valid **empty head**, same output, no warning. `@@{…}` stays a valid untyped head.                                                                                                                                                                                                                                                                          | §4.3, §5; golden `01`, `21` |
| **D3**  | The legacy `[.class,#id]{k:v}` form is **removed completely** (code, docs, sandboxes, READMEs): no warning, no deprecation path. `legacy_extras_warning`, `parse_attrs`, `ParsedAttrs` and the core `ExtraAttrs` are gone.                                                                                                                                                             | §14 retirement              |
| **D4**  | Strict adjacency everywhere; `@@type {…}` stays literal. Only **list** and **blockquote** allow a space between marker / head / content (reserved for Phase 4). Tables get no exception: a head touches its cell's `\|`, and in a delimiter cell it goes _after_ the alignment code (and width) and touches it.                                                                        | §4.1, §8; golden `08`, `21` |
| **D5**  | Cite is only `[^^](ref "loc")`: unquoted ref, optional quoted loc. `[^^]("book")`, `[^^]("book", "loc")`, `loc=` and `[^^]()` are literal text.                                                                                                                                                                                                                                        | §7.3; golden `05`           |
| **D6**  | Setext headings never existed; `---` is micromatter / `<hr />` only.                                                                                                                                                                                                                                                                                                                   | §4, §14; CLI check          |
| **D7**  | Drop `list`/`blockquote` from the plugin lists and the TOML until Phase 4 (freeze the Q/L sections as plain text). Unknown plugin names and unknown `task.*` keys warn instead of being silent.                                                                                                                                                                                        | §13, §14                    |
| **D8**  | An unclaimed/default directive renders `<span>` (inline) / `<div>` (block), mirroring markers.                                                                                                                                                                                                                                                                                         | §10; golden `15`, `16`      |
| **D9**  | Commit `out.frozen/`, add the freeze gate, and re-freeze the baseline (see **Status**).                                                                                                                                                                                                                                                                                                | `ultimate_freeze_spec`      |
| **D10** | Keep the golden `docs/spec/golden/**` tree out of biome/dprint churn (a separate, deliberate tooling change; the goldens are generated).                                                                                                                                                                                                                                               | tooling note                |

## Related notes

- `plugin-wiki` and `plugin-vicado` carry their **own** `[.class]`-shaped spelling,
  parsed by their own code — they never went through the retired core parser, so
  they are **not** "legacy" and must not be "migrated".
- Brace groups in prose (`{adipisicing}`, `{foo}`, `{_p_ → _q_}`) are literal text;
  `{…}` is only a head where a construct looks for one.
