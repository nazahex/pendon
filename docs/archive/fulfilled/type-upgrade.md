# Type Upgrade

Pendon currently has a `type` attr that is defined in several ways, such as `@@type`, `{{type}}`, `::type`, and `==type`.

At present, `type` is only used to divide and define custom components in `pendon.toml`.

Going forward, `type` will be used by other plugins and mechanisms, including:

- plugin-extract (RFC)
- plugin-number (RFC)

This upgrade adds a few extra mechanisms and some extra syntax.

## General Rules

- **Independent Execution Principle:** Each _task_ block fully runs its own job for the types it covers.
- **Target Resolution and Overriding Principle:**
  - If several _task_ blocks handle the same type but target different entities, channels, or instantiations, then **all tasks run in full**, side by side.
  - If _task_ blocks overlap on the same type **and target the same final entity** (such as an output file, an export variable, or an exclusive custom-component slot), the definition with higher specificity or later order **resolves (overrides)** the earlier definition.
- **Specificity Values:**
  - `?` and an explicitly written `type` are worth **2** (high).
  - `_` and `*` are worth **1** (low).
- If there are 2 definitions for a `type` on the same target:
  - The definition with the higher specificity value wins.
  - If the specificity values are equal, the last (bottom-most) definition wins.

## Type `?`

Type `?` means "ONLY untyped".

Example:

```toml
[[task.heading.custom]]
type = "?" # ONLY untyped
name = "HeadingUntyped"
imports = "import { HeadingUntyped } from '@comp/Heading'"
template = "<HeadingUntyped {...attrs}>{children}</HeadingUntyped>"
```

## Type `_`

`_` means: all types that have NOT yet been defined, including `untyped` ONLY IF `untyped` has not been defined.

`_` is evaluated globally, not per target. Only types that are explicitly named count as defined; `*` does not count as defining a type.

```toml
# If there are TypeA, TypeB, TypeC, and TypeD

[[task.foo]]
type = ["TypeA", "TypeB"] # ONLY TypeA and TypeB

[[task.foo]]
type = "_" # untyped, TypeC, TypeD
```

```toml
# If there are TypeA, TypeB, TypeC, and TypeD

[[task.foo]]
type = ["TypeA", "TypeB"] # ONLY TypeA and TypeB

[[task.foo]]
type = "?" # ONLY untyped

[[task.foo]]
type = ["TypeC", "_"] # TypeC, TypeD
```

## Type `*`

This means all types without exception, including `untyped` and types that have already been defined.
However, a more specific definition wins against `*` (on the same resolution target).

```toml
# If there are TypeA, TypeB, TypeC, and TypeD

# D.1
[[task.foo]]
type = ["TypeA", "TypeB"] # ONLY TypeA and TypeB

# D.2
[[task.foo]]
type = ["*"] # untyped, TypeA, TypeB, TypeC, TypeD

# D.1 and D.2 can run side by side independently as long as their resolution targets differ.
```

```toml
# If there are TypeA, TypeB, TypeC, and TypeD

# D.1
[[task.foo]]
type = ["TypeA", "TypeB"] # ONLY TypeA and TypeB

# D.2
[[task.foo]]
type = ["*", "TypeB"] # untyped, TypeA, TypeB, TypeC, TypeD

# D.3
[[task.foo]]
type = [
  "_",
] # untyped, TypeC, TypeD (TypeA and TypeB are explicitly named in D.1 and D.2; `*` does not count as defining a type)

# TypeB in D.2 has the same specificity (2) as TypeB in D.1. D.2 wins on TypeB because it comes later.
# The specificity of TypeB does not decrease just because it is combined with `*`.
```

## Type Not Written

If `type` is not written explicitly, it is treated as `_`.

```toml
# If there are TypeA, TypeB, TypeC, and TypeD

# D.1
[[task.extract]]
type = ["TypeA"] # ONLY TypeA extracts to this file
output = "./out/type-a.json"

# D.1b
[[task.extract]]
type = ["TypeA"]
output = "./out/type-a.json" # Same type and same output as D.1, so D.1b overrides D.1. The effect is the same as running the task once.

# D.1c
[[task.extract]]
type = ["TypeA"]
output = "./out/type-a-2.json" # Same type but a different output, so it does not override D.1b and both run. Useful for doing the same task with different details.

# D.2
[[task.extract]]
# Because type is not written, it is treated as _ (untyped, TypeB, TypeC, TypeD)
output = "./out/remaining.json"

# D.2b
[[task.extract]]
# Second execution (untyped, TypeB, TypeC, TypeD)
output = ".../../project-2/out/remaining.json"

# D.3
[[task.extract]]
# If a configuration without type is written again further down with the same output,
# D.3 overrides D.2 for those remaining types.
output = "./out/remaining.json"
format = "yaml"

# D.4
[[task.extract]]
type = ["TypeA", "_"] # TypeA, plus untyped, TypeB, TypeC, and TypeD (via `_`), extract to this file
output = "./out/type-a-and-u.json"

# D.5
[[task.extract]]
type = "*"
output = "./out/all.json" # D.5 extracts all types, including untyped and types that are already defined.
```

## Complex Example: Custom Component Resolution

Because custom components are exclusive per `type` (one element can only be rendered by one component), a more specific definition, or one located further down, **resolves (overrides)** the earlier component.

```toml
# Suppose there are the types TypeA, TypeB, TypeC, and elements without a type (untyped / ?)

# R.1: Global default fallback for all types, using a generic component
[[task.heading.custom]]
type = ["*"]
name = "HeadingDefault"
imports = "import { HeadingDefault } from '@comp/Base'"
template = '<HeadingDefault level="{attrs.level}" {...attrs}>{children}</HeadingDefault>'

# R.2: Special handling for components of type 'TypeA'
[[task.heading.custom]]
type = ["TypeA"]
name = "HeadingTypeA"
imports = "import { HeadingTypeA } from '@comp/SpecialA'"
template = '<HeadingTypeA data="{attrs.meta}" modern variant {...attrs}>{children}</HeadingTypeA>'

# R.3: Handling for the remaining types that have not been defined (TypeB and TypeC; untyped is excluded because R.4 defines it)
[[task.heading.custom]]
type = ["_"]
name = "HeadingFallback"
imports = "import { HeadingFallback } from '@comp/Fallback'"
template = '<HeadingFallback dark theme {...attrs}>{children}</HeadingFallback>'

# R.4: Specific override for untyped (because ? has a specificity of 2, higher than the _ in R.3)
[[task.heading.custom]]
type = "?"
name = "HeadingUntyped"
imports = "import { HeadingUntyped } from '@comp/Untyped'"
template = '<HeadingUntyped strictMode="{true}" {...attrs}>{children}</HeadingUntyped>'
```

### Explanation of the Component Resolution Above

1. **`TypeA`** is rendered using **`HeadingTypeA`** (because it is covered by R.2, which has an explicit specificity of 2, beating R.1, which uses `*`).
2. **`TypeB` and `TypeC`** are rendered using **`HeadingFallback`** (caught by R.3 through the `_` symbol as the remaining types that have not been handled; R.3 and R.1 have the same specificity of 1, and R.3 comes later).
3. **`untyped` (`?`)** is rendered using **`HeadingUntyped`**. R.4 explicitly defines `type = "?"` with a specificity value of `2`, which beats the specificity of 1 from R.3 and R.1, so R.4 fully **resolves (_overrides_)** `untyped`. Because `?` is explicitly defined, the `_` in R.3 no longer includes untyped.
4. In this example, **`HeadingDefault`** from R.1 is fully overridden: every type is covered by R.2, R.3, or R.4, so R.1 is never used. R.1 would only apply to a type that no more specific or later definition covers.
