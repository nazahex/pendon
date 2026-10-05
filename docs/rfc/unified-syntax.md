# Unified Syntax and New Features Proposal

> **Status:** design history. The frozen grammar lives in
> [`docs/spec/SYNTAX.md`](../spec/SYNTAX.md) — where the two disagree, the spec
> wins. Open points from this RFC are tracked as `OPEN-*` items in the spec
> (§15).

## Syntaxes Revamp

### Image

Now:

~?!h300w800[alt text](https://foo.com/img.png)[.extra,.class,#id]{foo: "bar"} Figcaption lorem ipsum.

Recommendation:

~?!h300w800[alt text](https://foo.com/img.png)@@type{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar} Figcaption lorem ipsum.

Figure version:

~?!!h300w800[alt text](https://foo.com/img.png)@@type{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar} Figcaption lorem ipsum.

### Anchor

Now:

[Foo bar](https://foo.com/bar--!~$ "Title Foo")[.extra,.class,#id]{foo: "bar"}

Recommendation:

[Foo bar](https://foo.com/bar--!~$ "Title Foo")@@type{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar}

> Note: `title` inside `()` in anchor has higher priority. `title` inside `{}` would still used if `title` inside `()` does not exist.

### Cite

Now:

[^^]("paper-smith", "hlm. 55")[.extra,.class,#id]{foo: "bar"}

Recommendation:

[^^](paper-smith "hlm. 55")@@type{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar}

### Heading

Now:

###[slug-foo][.extra,.class,#id]{foo: "bar"} Foo Bar

Recommendation

###[slug-foo]("Title Foo")@@type{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar} Foo Bar

> Notes:
>
> - `slug-foo` inside `[]` and `"Title Foo"` inside `()` has higher priority.
> - `"Title Foo"` is different from innerText `Foo Bar`.
> - `[slug-foo]` and `("Title Foo")` are optinal.

### Wiki

Nisi [[Anim Esta]]@@type{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar} id id est officia.

Nisi [[Anim Esta (Officia) | Anim]]@@type{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar} id id est officia.

No changes to the original syntax. We only add extra attrs support

> Wiki infobox will be reworked later

### Table

Now:

[Table caption foo][.extra,.class,#id]{foo: "bar"}
| Qoo | Roo | Coo |
| :---(200px)[.extra,.class,#id] | :---:[.v-top] | ---:(30%)[.v-bottom] |
| Foo -[.extra,.class,#id] | Bar { foo: "bar "} | Baz |
| Qux | > | Bux |
|===|
| Goo | > | Fee -[.extra,.class,#id]{ foo: "bar "} |

Recommendation:

|-[slug-foo]@@type{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar}-|
||@@type{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar} Table caption foo ||
| Qoo |{`slug-foo`} Roo | Coo |{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar}
| :---(200px){`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar} | :---:{.v-top} | ---:(30%){.v-bottom} |@@type{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar}
|{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar} Foo |{ foo: "bar "} Bar | Baz |
| Qux | > | Bux |@@type{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar}
|===|@@type{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar}
| Goo | > |{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar} Fee |{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar}

> Notes:
> New syntax for optional table definition: `|- -|` (used only for declaring slug and extra attrs)
> New syntax for table caption: `|| foo bar ||`
>
> - `|-[slug-foo]@@type{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar}-|` is extra attrs for table. Must remain empty of any standar html text. Any standard text inside it will be discarded.
> - `||@@type{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar} Table caption foo ||` is extra attrs for table caption.
> - `| :---(200px){`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar} | :---:{.v-top} | ---:(30%){.v-bottom} |@@type{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar}`: extra attrs in the inside cell (after aligment and width code) is for thec orreponding collumns, meanwhile extra attrs at the very end of the line (outside table, after the last `|` ) is for `<tbody>`
> - `|===|@@type{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar}` is extra attrs for `<tfoot>`
> - extra attrs at the very end of line (outside table, after the last `|`) of tbody and tfoot is extra attrs for `<tr>`.
> - extra attrs after the opening `|` inside cell is extra attrs for the corresponding cell (th/td).

## New Syntaxes

### Blockquotes

Now: (It does not have complex syntax yet)

Recommendation:`

> @@type{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar} Lorem ipsum dolor sit amor.

### List

**Design A** (frozen in [`docs/spec/SYNTAX.md`](../spec/SYNTAX.md) §9.3):
extras are carried by **decorator lines**, never by a swallowed carrier list.

Container extras — a decorator line directly above the list (layer `unordered`
for `-`/`*`/`+`, layer `ordered` for `1.`):

@@type{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar}

- Foo bar
  - @@{`nested-item`} Bar foo
  - Hux Hoo
- Goo Baz

Item extras (layer `list`, element `<li>`) — decorator at the start of the item:

- @@liItem{`alpha`} Alpha text
  - @@{`beta`} Nested item

Ordered list: the start offset comes from the **first item's number** only —
`o.` is **retired**:

@@{`my-list`}
6. Goo quz

> Renders as `<ol start="6">`. `start:` passed through extras is rejected with a
> `Severity::Warning`.

### Marker

Inline:

Lorem {{type}}{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar} ispum.

Block:

{{type}}{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar}

### Inline Directives

Recommendation:

Lorem ::type[foo-bar]("Foo Bar"){`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar} ipsum dolor:: sit amor.

Nesting:

Lorem ::::type[foo-bar]("Foo Bar"){`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar} :::typeB ipsum ::typeC amet:: ::: dolor:::: sit amor.

> - Max of `::` is `:::::::` (7), min is `::` (2)
> - `[foo-bar]` and `("Foo Bar")` property names are configured via pendon.toml (`bracket_key` and `parentheses_key`). Default is `slug` and `title`respectively.

### Block Directives

===type[foo-bar]("Foo Bar"){`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar}

Lorem ipsum dolor sit amor.

==typeB

Et deserunt officia cupidatat.

==

===

Combined:

===type[foo-bar]("Foo Bar"){`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar}

Lorem ipsum dolor sit amor.

==typeB

Lorem ::::type[foo-bar]("Foo Bar"){`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar} :::typeB ipsum ::typeC amet:: ::: dolor:::: sit amor.

==

===

> Max of `==` is `=======` (7), min is `==` (2)
>
> - `[foo-bar]` and `("Foo Bar")` property names are configured via pendon.toml (`bracket_key` and `parentheses_key`). Default is `slug` and `title`respectively.

## Revamped Config

> Authoritative schema: [`docs/spec/SYNTAX.md`](../spec/SYNTAX.md) §11. Runnable
> example: `sandbox/unified/pendon.toml` (kept in sync). The block below is
> illustrative; `imports` is the canonical key and accepts an array or a single
> string.

```toml
[[task]]
name = "Unified Demo"
input = "./src/[slug].md"
output = "./out/[slug].jsx"
plugin = "micromatter,table,img,directive,cite,wiki,anchor,markdown,latex,syntect,heading,sectionize"
format = "solid"
markdown_strip_comments = true
markdown_allow_html = true

# === ANCHOR ===

[[task.anchor.custom]]
type = ["anchorA", "anchorB"]
name = "AnchorAB"
imports = ["import { AnchorAB } from '@comp/shared/Anchor'"]
template = "<AnchorAB type={\"{attrs.type}\"} {...attrs}>{children}</AnchorAB>"

[[task.anchor.custom]]
# blank type means it will be used as default for all other type and un-typed that are not defined
name = "AnchorDefault"
imports = ["import { AnchorDefault } from '@comp/shared/Anchor'"]
template = "<AnchorDefault {...attrs}>{children}</AnchorDefault>"

# === CITE ===

[task.cite]
reference_source = "internal"
prefix = "cite-"
class = "cite-ref"
id_prefix = "cra-"

[task.cite.custom]
name = "Cite"
imports = ["import { Cite } from '@comp/content/Cite'"]
template = "<Cite reference={frontmatter.references[\"{attrs.id}\"]} {...attrs} />"

# === DIRECTIVE ===

[[task.directive.custom]]
type = ["asideA", "asideB"]
name = "AsideAB"
bracket_key = "slug"
parentheses_key = "title"
imports = ["import { AsideAB } from '@comp/content/AsideAB'"]
template = "<AsideAB slug={\"{attrs.slug}\"} title={\"{attrs.title}\"} type={\"{attrs.type}\"} {...attrs}>{children}</AsideAB>"

[[task.directive.custom]]
type = ["note"]
name = "Note"
bracket_key = "id"
parentheses_key = "level"
imports = ["import Note from '@comp/content/Note'"]
template = "<Note {...attrs}>{children}</Note>"

# === MARKERS ===

[[task.marker.custom]]
type = "bibliography"
name = "Bibliography"
imports = ["import { Bibliography } from '@comp/content/Cite'"]
template = "<Bibliography cites={frontmatter.cites} references={frontmatter.references} />"

[[task.marker.custom]]
type = "markerA"
name = "MarkerA"
imports = ["import { MarkerA } from '@comp/content/Cite'"]
template = "<MarkerA {...attrs} />"

# === LIST ===

[[task.list.list.custom]]
type = ["check", "data"]
name = "ListCheck"
imports = ["import { ListCheck } from '@comp/content/List'"]
template = "<ListCheck {...attrs}>{children}</ListCheck>"

[[task.list.list.custom]]
name = "List"
imports = ["import { List } from '@comp/content/List'"]
template = "<List {...attrs}>{children}</List>"

[task.list.ordered.custom]
name = "OrderedList"
imports = ["import { OrderedList } from '@comp/content/List'"]
template = "<OrderedList {...attrs}>{children}</OrderedList>"

[[task.list.unordered.custom]]
type = ["compact", "checklist"]
name = "CompactList"
imports = ["import { CompactList } from '@comp/content/List'"]
template = "<CompactList {...attrs}>{children}</CompactList>"

[[task.list.unordered.custom]]
name = "UnorderedList"
imports = ["import { UnorderedList } from '@comp/content/List'"]
template = "<UnorderedList {...attrs}>{children}</UnorderedList>"

# === TABLE ===

[task.table.custom.table]
name = "Table"
imports = ["import Table from '@comp/content/Table'"]
template = "<Table {...attrs}>{children}</Table>"

[task.table.custom.caption]
name = "TableCaption"
imports = ["import { TableCaption } from '@comp/content/Table'"]
template = "<TableCaption {...attrs}>{children}</TableCaption>"

[[task.table.custom.thead]]
type = ["theadA", "theadD"]
name = "TableHeadAD"
imports = ["import { TableHeadAD } from '@comp/content/Table'"]
template = "<TableHeadAD {...attrs}>{children}</TableHeadAD>"

[[task.table.custom.thead]]
name = "TableHeadDefault"
imports = ["import { TableHeadDefault } from '@comp/content/Table'"]
template = "<TableHeadDefault {...attrs}>{children}</TableHeadDefault>"

[task.table.custom.tbody]
name = "TableBody"
imports = ["import TableBody from '@comp/content/Table'"]
template = "<TableBody {...attrs}>{children}</TableBody>"

[task.table.custom.tfoot]
name = "TableFoot"
imports = ["import TableFoot from '@comp/content/Table'"]
template = "<TableFoot {...attrs}>{children}</TableFoot>"

[[task.table.custom.row]]
type = ["rowA", "rowB"]
name = "TableRowAB"
imports = ["import { TableRowAB } from '@comp/content/Table'"]
template = "<TableRowAB {...attrs}>{children}</TableRowAB>"

[[task.table.custom.row]]
name = "TableRowDefault"
imports = ["import { TableRowDefault } from '@comp/content/Table'"]
template = "<TableRowDefault {...attrs}>{children}</TableRowDefault>"

[[task.table.custom.cell]]
type = ["cellA", "cellB"]
name = "TableCellAB"
imports = ["import { TableCellAB } from '@comp/content/Table'"]
template = "<TableCellAB {...attrs}>{children}</TableCellAB>"

[[task.table.custom.cell]]
name = "TableCellDefault"
imports = ["import { TableCellDefault } from '@comp/content/Table'"]
template = "<TableCellDefault {...attrs}>{children}</TableCellDefault>"

# === HEADING ===

[task.heading]
auto_number = true
number_style = "nested-number"

[[task.heading.custom]]
type = ["headingX", "headingY"]
name = "HeadingXY"
imports = ["import { HeadingXY } from '@comp/content/Heading'"]
template = "<HeadingXY level={\"{attrs.level}\"} id={\"{attrs.id}\"} class={\"{attrs.class}\"} number={\"{attrs.number}\"} {...attrs}>{children}</HeadingXY>"

[[task.heading.custom]]
name = "HeadingDefault"
imports = ["import { HeadingDefault } from '@comp/content/Heading'"]
template = "<HeadingDefault {...attrs}>{children}</HeadingDefault>"

# === IMAGE ===

[task.img.img.custom]
name = "AdvancedImage"
imports = ["import AdvancedImage from '@comp/content/AdvancedImage'"]
template = "<AdvancedImage {...attrs}>{children}</AdvancedImage>"

# in current implementation, figure custom node is not separated from img custom node, so we need to put it here
[task.img.figure.custom]
name = "Figure"
imports = ["import Figure from '@comp/content/Figure'"]
template = "<Figure {...attrs}>{children}</Figure>"

# === WIKI ===

[task.wiki]
link_prefix = "/id/wiki" # moved here, originally from task root config

[task.wiki.anchor.custom]
name = "WikiLink"
imports = ["import { WikiLink } from '@comp/content/Wiki'"]
template = "<WikiLink href={\"{attrs.href}\"} {...attrs}>{children}</WikiLink>"

[task.wiki.infobox.custom]
name = "Infobox"
imports = ["import { Infobox } from '@comp/content/Infobox'"]
template = "<Infobox {...attrs}>{children}</Infobox>"
```

## Proposal

- New plugins proposed:
  - `plugin-blockquote`
  - `plugin-directive`: Should be used using custom component. Otherwise they will be rendered as `<span>` for inline directive and `<div>` for block directive.
  - `plugin-marker`?

### Extra Attrs

- Extra attrs {`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar} has no strict order. You can mix up the order.
- Extra attrs is completely optional.
- `::type`, `==type`, `{{type}}`, and `@@type`: Extra identifier for the node. Useful for separating the component's render output when using a custom component. Alpha-only, cannot use space, `-`, `_`, and any other symbols (fooBar allowed). They are optional except marker `{{type}}`.
- Every extra syntax must not be separated by space, otherwise would be rendered as escaped html literal text (except extra syntaxes of list and blockquote).
