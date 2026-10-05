The construct head wins over the extras; the extras value is dropped (§6.2).

[Head title wins](/docs "Head title")@@anchorA{"Extras title"}

`#id` beats the extras slug, and an extras slug becomes the id when nothing else
did (§6.2).

[Explicit id](/docs)@@anchorA{`slug-extras`, #explicit}

[Slug only](/docs)@@anchorA{`slug-only`}

`href` is owned by the link; the extras value is ignored (§7.2).

[Href is owned](/docs)@@anchorA{href: "/evil"}

URL modifiers are parsed before the head and win over `target:`:

[Modifier wins](/docs^)@@anchorA{target: "_self"}

[No modifier](https://example.com/bar)@@anchorA{target: "_self"}

`rel:` merges with the modifier result instead of replacing it:

[Rel merge](https://example.com/bar!)@@anchorA{rel: "prefetch", hreflang: "en"}

A marker with no configured entry keeps the built-in `<a>` element and carries
the marker as its `type` attribute (§11 rule 3):

[Unclaimed marker](/docs)@@anchorZ{.hero}

[No marker at all](/docs)
