Every item kind of §5 in one head. `class` accumulates, `id`/`slug`/`title` and
every prop is last-wins, and bare flags stay bare attributes (§6.3).

[All item kinds](/docs/bar "Head title")@@anchorA{`slug-foo`, "Extras title", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar}

The worked example of §5.1 in full:

[Worked](/docs)@@anchorB{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar}

Duplicates collapse to one attribute: the position of the first occurrence wins
and the value of the last one does (§6.4).

[Duplicates](/docs)@@anchorB{.first, .second, #dup, #other, foo: "one", foo: "two", isFlag, other}

Escapes inside values: a comma, a quote and a backslash all survive.

[Escaping](/docs)@@anchorA{csv: "a\,b", quote: "say \"hi\"", path: "dir\\file"}

A `class:` prop accumulates exactly like a `.x` item, head items first (§6.4).

[Class prop](/docs)@@anchorA{.head, class: "tail"}
