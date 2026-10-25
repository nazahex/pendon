# Type Resolution — shared design (DRAFT)

**Status: draft for review.** This is the _implementation design_ for the `type`
semantics requested by the [Type Upgrade proposal](../issues/type-upgrade.md). It
is **not normative** and must not be cited as grammar. Once accepted, the
normative wording belongs in [`spec/SYNTAX.md`](../spec/SYNTAX.md) (§5 selectors,
§11 custom-component config, §15 open decisions) and the rationale in a new ADR
(`docs/decisions/0008-type-selectors.md`).

Three consumers need exactly this algorithm:

| Consumer                    | Uses it for                                       |
| --------------------------- | ------------------------------------------------- |
| `apps/cli` → renderer hints | which custom component renders an element         |
| `plugin-number`             | which numbering group/style an element belongs to |
| `plugin-extract`            | which extraction block(s) an element belongs to   |

Building it once (crate `pendon-types`, dir `crates/types`) is the whole point of
doing the Type Upgrade **first**.

---

## 1. Vocabulary

| Term                 | Meaning                                                                |
| -------------------- | ---------------------------------------------------------------------- |
| **type marker**      | the `@@type` / `{{type}}` / `::type` / `==type` on an element (§3, §4) |
| **untyped**          | an element with no marker                                              |
| **selector**         | one entry of a config `type = …`: a literal name, `?`, `_`, or `*`     |
| **occurring types**  | `{untyped} ∪ {every literal marker that appears in the document}`      |
| **definition group** | the set of rules resolved against each other (see §2.2)                |
| **target**           | the final entity a rule writes to (see §6)                             |
| **specificity**      | literal / `?` = **2**; `_` / `*` = **1**                               |
| **order**            | the rule's declaration index inside its group (0-based, top to bottom) |

Identity is the **exact string** (case-sensitive, §4 naming rules). `untyped` is a
distinct identity; it is never produced by a literal selector.

---

## 2. Rules, restated as data

### 2.1 Selector expansion (per rule)

Given the group's `defined` set and the document's `occurring` set:

| Selector  | Expands to                                                  | Specificity |
| --------- | ----------------------------------------------------------- | ----------- |
| `"TypeA"` | `{TypeA}` (even when it occurs 0 times)                     | 2           |
| `"?"`     | `{untyped}`                                                 | 2           |
| `"_"`     | `occurring − defined`                                       | 1           |
| `"*"`     | `occurring` (everything, incl. defined types and `untyped`) | 1           |

`defined` = every literal named by any rule in the group, plus `untyped` if any
rule wrote `?`. **`_` and `*` define nothing** — that is what makes `_` in a
second rule still see the same remaining set.

Specificity is **per selector, not per rule**: a type matched by both `"*"` and
`"TypeB"` in one rule has specificity 2 (max over the selectors matching it).
"The specificity of TypeB does not decrease just because it is combined with `*`"
— `type-upgrade.md` §Type `*`.

### 2.2 Definition groups (open decision D-2)

A group is the smallest set of rules that can override each other. Proposed
grouping — **one element section of one task**:

| Config section                                                  | Group = all rules of…                    |
| --------------------------------------------------------------- | ---------------------------------------- |
| `[[task.<element>.custom]]` / `[task.<element>.custom.<layer>]` | that element's layers                    |
| `[[task.<element>.number]]`                                     | that element's `.number` blocks          |
| `[[task.<element>.extract]]`                                    | that element's `.extract` blocks         |
| `[[task.number.custom]]`                                        | all custom number types (task-level, §9) |
| `[[task.bind.inject]]`                                          | all inject blocks                        |

Rationale: every example in both proposals stays inside one element section, and
cross-element coupling (`_` in `heading` affected by a `cite` type) would be
surprising. The alternative — **whole task** scope, matching the proposals' loose
"evaluated globally" wording — is a one-line change (`Group` built from the whole
config instead of one section). Decide before T1.5 (decision D-2 in
[`TASKS.md`](TASKS.md)).

Within a group, `_` is **global across targets**: a rule whose `_` excludes
`TypeA` because another rule named `TypeA` excludes it for _all_ of its own
targets, not only the shared one.

### 2.3 Override

For each `(target, type)` pair keep the best rule by `(specificity desc, order
desc)`. Higher specificity wins; equal specificity → the later rule wins.
Non-overlapping targets are independent: one rule can list several destinations
and still be overridden on only one of them (P.1.c overrides only the
`citationsC` export of P.1.b, not its file).

---

## 3. Data model (`crates/types`)

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Selector {
    Literal(String), // "TypeA"
    Untyped,         // "?"
    Undefined,       // "_"
    Any,             // "*"
}
impl Selector {
    pub fn specificity(&self) -> u8 {
        match self { Self::Literal(_) | Self::Untyped => 2, _ => 1 }
    }
}

/// The document's type identities. Ordered for deterministic output.
#[derive(Clone, Debug, Default)]
pub struct TypeSet { names: BTreeSet<String>, untyped: bool }

/// One rule. `P` is the caller's payload (component, number block, extract
/// block), `T` its target identity (§6) — `pendon-types` interprets neither.
#[derive(Clone, Debug)]
pub struct Rule<P, T> { pub selectors: Vec<Selector>, pub targets: Vec<T>, pub payload: P }

/// Stable, comparable key for a destination (§6).
pub trait TargetId { fn key(&self) -> String; }

pub struct Group<P, T> { pub rules: Vec<Rule<P, T>> }
```

## 4. API (two entry points)

```rust
/// Exclusive consumers: components, numbering, one export name.
/// `None` = no rule covers this type (the caller applies its default).
pub fn winner<'a, P, T: TargetId>(g: &'a Group<P, T>, t: &TypeId) -> Option<&'a P>;

/// Additive consumers: extraction destinations.
/// Every rule covering `t`, deduplicated per target by §2.3, deterministic order.
pub fn matching<'a, P, T: TargetId>(g: &'a Group<P, T>, t: &TypeId) -> Vec<&'a P>;
```

Both resolve the whole group once per document (memoised) and then answer per
type — O(rules × targets × types), never per element.

## 5. Algorithm (pseudo-code)

```text
defined(g) = ⋃ literals(r) ∪ ({untyped} if any r writes ?)

expand(r, occurring, defined):
    out = []
    for s in r.selectors:
        Literal(x) → push(x)
        Untyped    → push(untyped)
        Undefined  → push every t ∈ occurring where t ∉ defined
        Any        → push every t ∈ occurring
    return dedupe(out)                     # first occurrence wins

resolve(g, occurring):
    defined = defined(g)
    best = {}                              # (target, type) → (spec, order, rule)
    for (i, r) in g.rules.enumerate():
        for t in expand(r, occurring, defined):
            s = max specificity among the selectors of r that match t
            for target in r.targets:
                match best[(target, t)]:
                    None                                               → insert (s, i, r)
                    Some((bs, bi, _)) if s > bs or (s == bs and i > bi) → replace
                    _                                                  → keep
    return best
```

Two properties to pin in tests:

1. **Order-independent expansion.** `defined` comes from the _whole_ group up
   front, so a later `?` removes `untyped` from an earlier `_`. Never compute
   `defined` incrementally.
2. **Idempotent destinations.** `matching` dedupes by target key after §2.3, so
   `["./out/a.json", "./out/a.json"]` yields one destination.

---

## 6. Target identity

| Consumer                            | Target key                                   | Exclusive?                       |
| ----------------------------------- | -------------------------------------------- | -------------------------------- |
| custom components                   | `plugin + ":" + layer` (e.g. `image:figure`) | yes (one component per element)  |
| `[[task.<element>.number]]`         | `<element>:auto`                             | yes (one counter group per type) |
| `[[task.number.custom]]`            | `numberType:<name>`                          | yes (one counter per numberType) |
| `[[task.<element>.extract]]` file   | normalized destination path (§7)             | yes (one block owns a file)      |
| `[[task.<element>.extract]]` export | `export:<var>`                               | yes (one `export const`)         |
| `[[task.bind.inject]]`              | `export:<var>` (same namespace as extract)   | yes (shared with extract)        |

A rule contributes **one target per destination it names**. "Exclusive" only
means "at most one rule wins per type"; different targets of the same rule all
still run.

## 7. Destination-path normalization (extract only)

Two blocks collide when they write the same file. Key = the path after `[slug]` /
`[category]` substitution, with `.`, `..` and duplicate `/` collapsed (lexical —
no `canonicalize`, the file may not exist yet), made absolute against the task's
working directory. `./out/a.json` and `out/a.json` are one target;
`./out/a.json` and `./out/A.json` are not (case-sensitive).

## 8. Worked examples

### 8.1 Components — `type-upgrade.md` R.1–R.4

Document occurs `{untyped, TypeA, TypeB, TypeC}`;
`defined = {TypeA, TypeB, untyped}` (R.2 literal, R.4 `?`; R.1 `*` and R.3 `_`
define nothing).

| Type    | Candidates (spec, order)          | Winner |
| ------- | --------------------------------- | ------ |
| TypeA   | R.1 `*` (1,0), R.2 (2,1)          | R.2    |
| TypeB   | R.1 `*` (1,0), R.3 `_` (1,2)      | R.3    |
| TypeC   | R.1 `*` (1,0), R.3 `_` (1,2)      | R.3    |
| untyped | R.1 `*` (1,0), R.3 `_`, R.4 (2,3) | R.4    |

R.1 claims no type → `HeadingDefault` is never used. R.3's `_` expanded to
`{TypeB, TypeC}` only, because `untyped ∈ defined`.

### 8.2 Extraction — `type-upgrade.md` D.1–D.5

Different destinations never override: D.1 → `type-a.json`, D.2/D.3 →
`remaining.json`, D.4 → `type-a-and-u.json`, D.5 → `all.json` all run. On the
same destination: D.1b beats D.1 (equal spec, later); D.2b has a different path →
both run; D.3 beats D.2 on `./out/remaining.json`. D.5 `*` claims everything but
loses to any spec-2 rule **on that destination only**.

### 8.3 Numbering — `plugin-number.md` P.1

`[[task.heading.number]] type = ["HeadingA","HeadingB"]` → target `heading:auto`,
specificity 2 for both types, one payload (counter + styles) shared by both. A
later block `type = ["HeadingB"]` takes `HeadingB` away from the first block
entirely (2 vs 2, later wins), leaving the first numbering `HeadingA` alone.

`%%` overrides `.number` outright: an element carrying `%%numberB` resolves in the
`[[task.number.custom]]` group (target `numberType:numberB`) and is therefore
**not a member** of any `.number` group — it must not touch that counter.

## 9. `%%` and the task-level custom group

`[[task.number.custom]]` is task-level because a single `numberType` counter is
shared across element kinds (`plugin-number.md` P.2 numbers headings, anchors and
images with `numberB`). Its occurrence set is therefore _every `numberType` that
appears anywhere in the document_, not per element. `_` there means "every
occurring numberType no rule named", each still with its own counter.

## 10. Edge cases and their decisions

| Case                                     | Behaviour                                                          |
| ---------------------------------------- | ------------------------------------------------------------------ |
| `type = []`                              | diagnostic "no selector"; the rule is dropped                      |
| `type = "TypeA"` vs `["TypeA"]`          | identical (string-or-array already accepted)                       |
| literal that never occurs                | valid, produces nothing                                            |
| `_` when every occurring type is defined | empty expansion → extract: task skipped; components: layer default |
| `*` and `_` in one rule                  | `*` subsumes `_`, both spec 1, no error                            |
| duplicate identical selector in one rule | deduped silently                                                   |
| literal named `untyped`                  | a normal literal, distinct from the untyped identity               |
| `?`/`_`/`*` as an _element_ marker       | grammar's job (§4 naming rules), not this module's                 |

## 11. Test matrix (`crates/types/src/lib.rs` unit tests)

1. each selector's expansion against a fixed `TypeSet`;
2. specificity: literal beats `*`; `?` beats `_`;
3. tie → later wins (order);
4. `defined` is group-wide (a later `?` removes untyped from an earlier `_`);
5. `_` excludes literals named in _any_ rule of the group, including one that only
   lists outputs;
6. `*` never defines (two `*` rules + `_` → `_` still sees everything);
7. `matching` dedupes identical destinations;
8. `winner` returns `None` when nothing covers a type (untyped document + only
   literal rules);
9. resolution is order-stable (BTreeSet iteration) — assert on `Vec<String>`;
10. property: for every type, `winner` is a member of `matching`.

## 12. Integration points (what each consumer changes)

- `apps/cli/src/components.rs` — `types: Vec<String>` accumulation becomes
  `Vec<Selector>`; the exact-literal duplicate check stays; hints carry selectors
  instead of an expanded marker list (decision D-3).
- `crates/renderer-solid/src/{components,node,template,imports}.rs` —
  `select(type)` gains a document-scoped resolution; imports are collected for the
  _winning_ components only.
- `crates/plugin-number/src/lib.rs` — two groups: per-element `.number` and the
  task-level `[[task.number.custom]]`; counters keyed by rule identity.
- `crates/plugin-extract/src/lib.rs` — one group per element section; destinations
  are `(path, format, filter)` tuples produced by the winning rules.
