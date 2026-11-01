# ADR-0019 — A component's dotted field is a nested prop; an element's is a spread

- Status: **Accepted** — supersedes [D-18](../archive/workflows/unified-syntax-stage-2/DECISIONS.md)
  for the **component** path only; the built-in element keeps D-18's one-key spread
- Depends on: [`0014`](0014-field-model.md) (public fields are a projection)
- Spec: [`SYNTAX.md`](../spec/SYNTAX.md) §6.5 (public fields), §11 (`custom`
  component config and its `template` tokens)
- Evidence: `crates/renderer-solid` unit tests
  (`a_component_receives_a_dotted_field_as_a_nested_prop`,
  `a_deeper_dotted_path_nests_one_level_further`,
  `an_explicitly_bound_leaf_leaves_the_rest_of_the_prop`,
  `a_kebab_case_segment_is_quoted_in_the_prop`,
  `an_empty_leaf_is_dropped_from_the_prop`); goldens `05-cite`, `06-heading`,
  `08-table-head`, `14-marker`, `28-type-selectors`, `30-number-heading`,
  `31-number-custom`, `32-extract-heading`, `33-extract-scopes`,
  `34-specs-assignment`, `35-cite-fields`, `39-decorator-order`,
  `46-number-fields`, `47-number-overrides`, `48-cite-stable-fields`
  (components re-frozen; the built-in `<a>` / `<p>` rows in the same fixtures
  stayed byte-identical)

## Context

A public field is a **path** ([ADR-0014](0014-field-model.md) rule 1):
`specs.tick`, `bracket.bare`, `locator.label`. No JSX attribute name may contain
one — `a.b=` parses as a member expression — so `renderer-solid` had to invent a
spelling, and D-18 picked **one** for both of the renderer's paths: the one-key
spread `{...{ "specs.tick": "x" }}`, emitted by a single helper
(`node::render_dotted_attr`) that `render_attrs` (built-in elements) and
`template::generate_spread_attrs` (a component's `{...attrs}`) both called.

Two things have changed since.

1. **The component path grew real users.** `plugin-cite`'s `stable_fields`
   vocabulary (`locator_type` / `locator_value`) and the extras slots now put a
   `locator.*` / `bracket.*` pair on every component, and a component author reads
   `props.locator.label`. The spread spelling forced every such template to either
   take the flat keys and split them itself, or spread two one-key objects per
   field.
2. **The two paths were never the same problem.** A built-in element is written
   into the DOM, where the value of an object-valued prop is `[object Object]` and
   `specs.tick="x"` is a legal name for `renderer-html`. A component is a JS
   function: it takes values, so an object is not a lossy encoding — it is the
   natural one.

## Decision

The spelling follows the **node's renderer**, not the field.

1. **A custom component nests.** `{...attrs}` regroups a dotted path under its
   first segment: `locator.label` + `locator.value` become one prop,
   `locator={{ label: "page", value: "110-112" }}`. A deeper path nests one level
   per segment (`a.b.c` → `a={{ b: { c: … } }}`).
2. **The built-in element spreads.** `render_attrs` keeps the one-key spread
   `{...{ "specs.tick": "x" }}` (D-18), because a DOM element takes attributes.
3. **Only the emission shape changes.** `{attrs.locator.label}` still resolves
   against the **flat** field map, and a leaf the template binds itself is simply
   absent from the spread, so the prop shrinks (`locator={{ value: … }}`) rather
   than falling back.
4. **No value is invented, dropped or reordered.** Leaves keep the projection's
   order; a prop is written where its first leaf sat; an empty value is dropped
   exactly as it is from a flat spread (§6.3, `alt` excepted); a segment that is
   not a bare JS identifier is quoted (`cross-ref={{ "b-c": … }}`), which
   `check_segment`'s kebab-case allowance makes reachable.
5. **A scalar field wins.** If a prefix is also a field of its own in the same map
   (`locator` _and_ `locator.label`), the spread is used for the dotted one: one
   prop is never written twice. `FieldMap::insert` already rejects such a pair
   (`ParentIsNotObject` / `ParentHasChildren`), so this is a guard, not a case.

## Rejected

- **Nesting on both paths (D-18's option, re-examined).** `[object Object]` on
  every built-in element that carries a dotted field, and `renderer-html` would
  print `specs.tick="x"` while the compiled JSX set a prop — the two DOM targets
  would stop agreeing on what a field means.
- **The one-key spread on both paths (D-18, status quo).** Correct but unusable
  for a component: the author reads a flat key with a dot in it, and a field pair
  costs two spreads. D-18's first reason ("three renderers must agree") is the
  _element_ argument; a component has no HTML counterpart to agree with.
- **A config switch per component (`nest_fields = true`).** A second spelling of
  one field, chosen by whoever wrote the entry, so a template would have to work
  under both shapes.
- **A deeper merge for `specs` / `number` only.** These are namespaces today, but
  `field.specs.<k>` can assign any path (`field.specs.id = "foo.id"`), so a
  whitelist would make nesting depend on which names happen to be in the map.

## Consequences

- `renderer-solid` has two emitters by design: `render_dotted_attr` for built-in
  elements and `template::nested_prop` (plus `nested_prefixes` / `nest` /
  `render_prop_tree`) for a component's spread. The shared piece is
  `node::js_value`, so a leaf is typed identically inside a prop and outside it.
- The golden harness guard (`assert_no_dotted_attribute_name`) is unchanged and
  still necessary: it forbids the invalid `specs.tick="x"` attribute name, not
  nesting. It now passes on `specs={{ tick: "x" }}` by construction — the dotted
  name is gone from the output entirely.
- Fifteen component fixtures were re-frozen; their built-in rows (a `<p>`, an
  `<a>` in `47-number-overrides`) are byte-identical, which is the evidence that
  the element path did not move.
- `renderer-html`, `renderer-json` and `renderer-ast` are untouched: nesting is a
  JSX-only shape of the _same_ projection, and `extract` reads the flat map.
