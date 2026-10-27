# Pendon plugin-extract Proposal

## Background and Goals

Currently, the data extraction mechanism in Pendon runs separately inside each plugin.

This approach leaves the code **scattered, redundant, and inconsistent**.

The main goal of this proposal is to unify the whole process into a single, centralized mechanism.

We want an extraction system that is **flexible, precise, and in-depth**.

## Supported Element Scope

This extraction plugin is designed to work on **all core elements** supported by the Pendon ecosystem.

These elements include _anchor_, _blockquote_, _cite_, _directive_, _heading_, _image_, _marker_, _list_, _section_, _table_, _wiki_ (_wikilink_ only), _codeblock_ (_vanilla_), _latex_, _bind_, and _micromatter_ (_frontmatter_).

Conversely, some modules are **intentionally excluded** from this scope.

Elements such as _quiz_, _codeblock-syntect_, _vicado_, _dialog_, and the old _plugin-custom_ syntax are considered deprecated or outdated.

The system must capture **all extra attributes (_extras_)** of each element in full.

```mdp
##[slug-bisa]("Title Bisa")@@typeBisa{ .class-bisa, #id-extra-bisa, foo: "Bisa", foo2: "bisa juga", bazBisa: { wax: 32, boo: true, fax: ["hoo", "hop"], roo : { qoo: 24 }}, --style-var-extra: "bisa" } Inner Text Bisa
```

## Configuration and Extraction Scenarios

Let's walk through how this configuration works in practice, using TOML format.

```toml
# SETTING P.1

# Suppose there are CiteA, CiteB, and CiteC

[[task]]
input = "./src/[category]/[slug].mdp"
output = "./out/[slug].jsx"
plugin = "micromatter,marker,cite,extract,markdown"
format = "solid"

# P.1.a
[[task.cite.extract]]
type = ["CiteA", "CiteB"]
output = ["./out/[slug].cite.json", "../../project-2/[category].yaml"]
format = ["json", "yaml"]
only = ["citation_number", "instance_id", "extraAttr1", "extraAttr5"]

# P.1.b
[[task.cite.extract]]
export = "citationsC"
omit = ["instance_id", "extraAttr1", "extraAttr2"]

# P.1.c
[[task.cite.extract]]
output = "./out/[category]-cite-c.yaml"
format = "yaml"
export = "citationsC"
omit = ["instance_id", "extraAttr1"]
```

- Block **P.1.a** targets extraction specifically for types `CiteA` and `CiteB`. The data is written to both a JSON file and a YAML file, **including only the fields** listed in the `only` array.
- Block **P.1.b** does not specify a type, so it is treated as `_` (see the [Type Upgrade Proposal](./type-upgrade.md)), acting as a safety net for types that have not been defined, in this case `untyped` and `CiteC`. Instead of creating a file, this configuration exports the data directly into a JavaScript variable (`export const citationsC`) while **omitting the fields** listed in the `omit` array.
- Block **P.1.c** overrides the export configuration of P.1.b because it comes later and targets the same variable. This block produces two outputs at once: it exports to a YAML file, and it also updates the `citationsC` variable with a shorter `omit` list. As a result, `extraAttr2` ends up in `citationsC`.

- If a field is not found on the original element, it is ignored entirely and no empty value is produced, even if the missing field is listed in `only`.
- `output` can be an array of multiple destinations. If `output` is an array, `format` must also be an array of the same `length`; otherwise an error occurs.
- `export` cannot be an array.
- `format` is required. The output is written in the specified format regardless of the file extension.
- If several inputs resolve to the same output file (for example, through the dynamic `[category]` path in `../../project-2/[category].yaml`), the file is overwritten.
- `export` only takes effect when the task `format` is `solid`. For any other task format, `export` is ignored and discarded.
- If no element matches, the task is skipped.

```toml
# SETTING P.2
[[task.micromatter.extract]]
output = "./out/[slug].json"
format = "json"
export = "frontmatter_2"
```

In scenario **P.2**, we see _micromatter_ (frontmatter) extraction exporting to the variable `frontmatter_2`.

Although `plugin-micromatter` already exports automatically to `const frontmatter`, this duplication is **still considered valid** and does not trigger a system warning. If `micromatter.extract` targets the same variable as the automatic export (`export = "frontmatter"`), `micromatter.extract` wins over the automatic export. This is useful when the client wants to drop certain fields.

## Filter Hierarchy: Omit, Only, and Recommended

The system provides three main parameters for filtering fields: `omit`, `only`, and `recommended`.

By default, if no filter is defined, the system extracts **all** available fields.

The `only` parameter works as an **allowlist**, restricting extraction to the listed fields.

Conversely, `omit` works as a **denylist**, dropping the listed fields and keeping the rest.

There is also a boolean parameter named `recommended`, which restricts extraction to the **preset fields** curated by Pendon.

The golden rule in this hierarchy is strict: **`omit` always wins** and overrides the instructions of both `only` and `recommended`.

```toml
# P.3

# Suppose cite has the following fields: citation_number, instance_id, ref_id, inner_text, and various custom extraAttrs from extras.
# Pendon's recommended extract preset includes ONLY: citation_number, instance_id, and ref_id.

[[task.cite.extract]]
output = "./out/[slug].cite.json"
format = "json"
recommended = true
only = ["citation_number", "instance_id", "extraAttr1", "extraAttr5"]
omit = ["instance_id"]
```

In example **P.3**, we enable `recommended = true`, which by default would take only _citation_number_, _instance_id_, and _ref_id_.

However, the presence of `only` narrows that selection further from within the preset, ignoring custom attributes that are not in the original recommendation.

At the same time, **`omit` forcibly cancels** the extraction of _instance_id_.

The final result is absolute precision, where **only `citation_number`** makes it into the output.

The `recommended` preset for each element is determined by the development team. The preset shown in this proposal is for illustration only.

## Removal of the Legacy Extract-Heading

Currently, the _heading_ module still relies on a separate extraction plugin, which is a _legacy implementation_: `plugin-extract-heading`.

With the arrival of this centralized _plugin-extract_, the old plugin **must be removed entirely**.

There will be no warning, _backward compatibility_, or _changelog_ for this removal. Because Pendon is still in its _alpha_ phase, we will do it **without nostalgia**.

Once `plugin-extract-heading` is removed, all elements are extracted as a flat array of objects, with depth information included for elements that have a concept of depth.

## Output Destination Flexibility

The output architecture of this system is designed not to limit the developer's choices.

Data can be written to **external files** such as JSON, YAML, or TOML.

Data can also be injected **internally** into JavaScript code through the `export const` mechanism of `plugin-export`.

Most powerfully, a single setting definition **can do both at once** within the same configuration block.

Which plugin or crate owns the `export const` mechanism is for the development team to decide after inspecting the source code directly.

## Integration with plugin-number

`plugin-extract` must be able to take data from all existing plugins, including `plugin-number`, which is currently still in RFC.

`plugin-extract` must run after the plugins whose data it extracts (for example, `plugin-number`). Where possible, this ordering is handled automatically. Where it is not, it must be documented so that clients order the plugins correctly.

## Implementation Notes and Next Steps

To complete this proposal, the team must finish two crucial tasks.

1. **Q.1**: Comprehensively document all values and attributes available in the Pendon ecosystem. This documentation is mandatory so that developers can extract data precisely and in depth without having to guess the original structure.
2. **Q.2**: More granular technical implementation details need to be added to this document. This can only be done after the team inspects the code of the actual implementation currently running in the repository.
