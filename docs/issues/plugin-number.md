# Pendon plugin-number Proposal

## Problem

- Currently, only headings and citations get the auto-numbering feature.
- Precise, in-depth control over numbering is not currently possible.
- Numbering depth cannot be applied to elements whose markdown syntax has no built-in depth mechanism, such as images and tables.

## Goals

- Create a single system for managing numbering across the entire ecosystem and all plugins.
- Enable in-depth, flexible, and precise custom numbering.

## Mechanism

Every element gets one extra configuration, `[task.<element>.number]`.

### Settings

**Setting P.1**

```toml
# Setting P.1
[[task]]
input = "./src/[slug].mdp"
output = "./out/[slug].jsx"
plugin = "micromatter,marker,cite,heading,image,number,markdown"
format = "solid"

[[task.heading.number]]
type = ["HeadingA", "HeadingB"]
style = [
  "decimal",
  "decimal",
  "lower-alpha",
] # defines the number style for depth levels 1, 2, and 3; the default is decimal.

# untyped headings and other types are not defined, so they do not get numbering
```

Setting P.1 makes (ONLY) headings typed as `HeadingA` and `HeadingB` share a single counter and numbering style.

`type` here uses the mechanism from the [Type Upgrade Proposal](./type-upgrade.md). In particular:

- Untyped headings are selected with the `?` type symbol.
- If the same type appears in more than one definition, the resolution rules of the Type Upgrade Proposal apply: the definition with higher specificity wins, and if the specificity is equal, the bottom-most definition wins.

`style` may also be a single string (e.g., `style = "upper-alpha"`), which applies to all depths. This works in every number setting.

Illustration:

```mdp
#@@HeadingA Will Be Numbered 1

##@@HeadingB Will Be Numbered 1.1

##@@HeadingA Will Be Numbered 1.2

###@@HeadingB Will Be Numbered 1.2.a

#####@@HeadingB Will Be Numbered 1.2.a.1.1 (an undefined depth style defaults to decimal)

##@@HeadingC Will Not Be Numbered

## Will Not Be Numbered

# Will Not Be Numbered

##@@HeadingA Will Be Numbered 1.3
```

### If the Initial Depth Does Not Exist

If the initial depth of a definition block does not exist in a file, the 1st number depth starts from the element with the smallest depth.

```mdp
###@@HeadingB Will Be Numbered 1

###@@HeadingA Will Be Numbered 2

####@@HeadingB Will Be Numbered 2.1

# Will Not Be Numbered

#####@@HeadingB Will Be Numbered 2.1.a

##@@HeadingC Will Not Be Numbered

## Will Not Be Numbered

###@@HeadingA Will Be Numbered 3
```

If the smallest depth appears in the middle:

```mdp
###@@HeadingB Will Be Numbered 1.1

###@@HeadingA Will Be Numbered 1.2

##@@HeadingB Will Be Numbered 2

# Will Not Be Numbered

#####@@HeadingB Will Be Numbered 2.1.a.1

##@@HeadingC Will Not Be Numbered

## Will Not Be Numbered

###@@HeadingA Will Be Numbered 2.2
```

### Counting Rules

These rules apply to auto-numbering defined via `task.<element>.number`:

- Each definition block has its own counter. An element at depth _d_ increments the counter at depth _d_ and resets all deeper depths.
- A parent depth that does not exist yet is filled in with the first value of its style (a "ghost" parent). For example, if `###` appears before any `##`, it becomes `1.1`.
- The smallest depth is determined across the whole file, and only from elements that belong to the definition block. Elements outside the block (untyped headings, other types) are ignored. Consequently, adding an element with a smaller depth at the end of a file changes the numbering of the elements before it.
- Elements that do not belong to the definition block never affect its counter. They neither break nor reset it.
- `attrs.number` always contains exactly one entry per depth, up to the element's own (normalized) depth.
- Elements that naturally have no concept of depth (e.g., images and tables) only have depth 1 when auto-numbered. Deeper numbering requires `%%`.

## Number-Specific Extras `%%`

`%%` is a special extra for numbering notation.

- Its canonical position is at the very end of any other extras.
- It must always be attached to the head or other extras (not separated by a space), as per the current extras rules. If separated, it becomes a literal.

Basic syntax: `%%numberType` up to `%%%%%%%numberType` (7 `%` characters).

- `%%` always wins against auto-numbering defined via `task.<element>.number`. An element with `%%` leaves its auto-number group and does not affect that group's counter.
- `%%` has a depth mechanism (1-6). The depth is the number of `%` characters minus 1: `%%` is depth 1, and `%%%%%%%` is depth 6. Its functions are:
  - Giving numbering depth to all elements that naturally have no concept of depth.
  - Overriding numbering depth on elements that do have a concept of depth.
  - Allowing a single numbering counter to be shared across different elements.
- `%%` is defined via `[[task.number.custom]]`. The `type` of each definition is the `numberType` that `%%` refers to.
- The `type` rules and overriding mechanism are the same as those proposed in the [Type Upgrade Proposal](./type-upgrade.md).
- Each `numberType` has its own counter.
- `%%` uses absolute depth, so ghost parents are always counted.
- A bare `@@` before `%%` (e.g., `@@%%numberB`) is valid syntax, but has no effect.

The `numberType` naming rules are the same as the `type` naming rules for `@@type`.

Example:

```toml
# SETTING P.2
[[task.heading.number]]
type = ["HeadingA"]
style = ["upper-alpha", "lower-alpha"]

[[task.number.custom]]
type = ["NumberA"]
style = "upper-alpha" # all number depths will be upper-alpha

[[task.number.custom]]
type = ["NumberB"]
style = [
  "upper-roman",
  "lower-alpha",
  "decimal",
] # depth 1 will be upper-roman, depth 2 will be lower-alpha, depth 3 will be decimal. e.g. I. a. 1., I.b , IV.c.12

[[task.number.custom]]
type = ["NumberC"]
style = ["upper-alpha", "lower-alpha", "upper-roman", "lower-roman"]
```

```mdp
Based on P.2

##[slug-heading]("Title Heading")@@HeadingA{ foo: "bar" }%%%numberB Inner Text Heading - Will Be Numbered I.a (depth 2, so the missing parent depth 1 is counted as I)

[link](/foo)@@AnchorA%%numberB will be numbered II.
[link](/foo)@@AnchorA%%%numberB will be numbered II.a.
[link](/foo)@@AnchorA%%%numberB will be numbered II.b.
[link](/foo)@@AnchorA%%%%numberB will be numbered II.b.1.
[link](/foo)@@AnchorA%%%%%%numberB will be numbered II.b.1.1.1.
[link](/foo)@@%%%numberB will be numbered II.c.
[link](/foo)@@%%numberB will be numbered III.
[link](/foo)%%numberA will be numbered A.
[link](/foo)%%%%numberA will be numbered A.A.A.
[link](/foo)%%%numberA will be numbered A.B.

![image foo](/foo.png)%%numberA will be numbered B.
![image foo](/foo.png)%%%%%numberA will be numbered B.A.A.A.

![image foo](/foo.png)%%%%%%%numberC will be numbered A.a.I.i.1.1.

##[slug]%%numberB Will Be Numbered IV
##%%%%numberB Will Be Numbered IV.a.1 (number depth 3, even though heading depth is 2)
##("Title")@@HeadingA Will Be Numbered A
###@@HeadingA Will Be Numbered A.a
##@@HeadingA{} Will Be Numbered B
```

Unlike numbering depth on elements, `%%` has no mechanism for ignoring a depth when the smallest depth does not exist in a file.

## Render

`plugin-number` does not render these numbers directly into the inner text (as the legacy auto-number in `plugin-heading` does). Instead, it passes them as an attrs array of strings.

In fact, the numbering does not have to be used as render output. It can also be used purely internally, or only as `class` and `id`.

Example:

```toml
[[task.element.number]]
style = ["decimal", "decimal", "lower-alpha"]
```

The numbers are stored in `attrs.number` in a form like this (illustration only):

```js
[
  { element_props: "foo", number: ["1"], extraAttr: "foo" /* ... */ },
  { element_props: "foo", number: ["1", "1", "a"], extraAttr: "foo" /* ... */ },
  { element_props: "foo", number: ["1", "2"], extraAttr: "foo" /* ... */ },
  { element_props: "foo", number: ["2"], extraAttr: "foo" /* ... */ },
  { element_props: "foo", number: ["2", "1"], extraAttr: "foo" /* ... */ },
  // and so on
]
```

This way, the actual rendered result becomes highly flexible because it is left entirely to the client. It is not forced to use a particular format (`n.n.n`).

## Number Styles

The goal is for `plugin-number` to support all of these styles:

```ts
/**
 * List of CSS `list-style-type` numeral systems
 * (predefined counter styles, CSS Counter Styles Level 3).
 *
 * The list below has not been thoroughly verified for existence or accuracy.
 */
type NUMBER_SYSTEMS =
  // --- Numeric: Latin / common ---
  | "decimal"
  | "lower-roman"
  | "upper-roman"

  // --- Numeric: other scripts ---
  | "arabic-indic"
  | "persian"
  | "bengali"
  | "cambodian"
  | "khmer"
  | "devanagari"
  | "gujarati"
  | "gurmukhi"
  | "kannada"
  | "lao"
  | "malayalam"
  | "mongolian"
  | "myanmar"
  | "oriya"
  | "tamil"
  | "telugu"
  | "thai"
  | "tibetan"
  | "cjk-decimal"
  | "hebrew"
  | "georgian"
  | "armenian"
  | "lower-armenian"
  | "upper-armenian"

  // --- Alphabetic ---
  | "lower-alpha"
  | "upper-alpha"
  | "lower-latin"
  | "upper-latin"
  | "lower-greek"
  | "hiragana"
  | "hiragana-iroha"
  | "katakana"
  | "katakana-iroha"

  // --- CJK ideographic, formal, and informal ---
  | "cjk-ideographic"
  | "cjk-earthly-branch"
  | "cjk-heavenly-stem"
  | "japanese-informal"
  | "japanese-formal"
  | "korean-hangul-formal"
  | "korean-hanja-informal"
  | "korean-hanja-formal"
  | "simp-chinese-informal"
  | "simp-chinese-formal"
  | "trad-chinese-informal"
  | "trad-chinese-formal"

  // --- Others ---
  | "ethiopic-numeric";
```

`none` is intentionally excluded: it is not a valid style for `plugin-number`. `decimal-leading-zero` is also intentionally excluded.

`lower-latin` is an alias of `lower-alpha`; the two behave identically.

If a number falls outside the range a style can represent, that style falls back to `decimal`.

However, if supporting all of these is too costly, it is enough to implement only the most commonly used styles: decimal, lower-roman, upper-roman, lower-alpha, upper-alpha, lower-latin, upper-latin, and lower-greek.

## plugin-heading Legacy Auto-Number

Currently, `plugin-heading` has its own numbering mechanism. We need to remove it entirely. Auto-numbering will use the new `plugin-number` in a centralized way.

There will be no warning, _backward compatibility_, or _changelog_ for this removal. Because Pendon is still in its _alpha_ phase, we will do it **without nostalgia**.

## Scope

This numbering plugin is designed to work on all elements that support extras.

Conversely, some modules are **intentionally excluded** from this scope.

Elements such as _quiz_, _codeblock-syntect_, _vicado_, _dialog_, and the old _plugin-custom_ syntax are considered deprecated or outdated.

Meanwhile, `plugin-cite` will not receive any changes for the time being. Its integration with `plugin-number` will be addressed at a later stage, as the numbering mechanism in `plugin-cite` is more complex than that of other elements.

## VSCode Highlighting Support

- VSCode must be able to distinguish the color of `%` for each depth (rainbow).
- The first `%%` has the same color. Each subsequent `%` has a different color.
- Future plan: fancier VSCode features, such as tracking, jumping, and a dedicated lens for numbering.
