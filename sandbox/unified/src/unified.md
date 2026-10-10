---
title: "Unified Syntax Demo"
references:
  suryana-2026:
    id: suryana-2026
    type: book
    title: Masa Depan Rekayasa Perangkat Lunak
    authors:
      - firstName: Eko
        lastName: Suryana
    publisher: TechPress Indonesia
    publisherLocation: Jakarta
    issuedDate:
      year: 2026
    isbn: 978-602-0000-00-0
    language: id
  paper-smith:
    id: paper-smith
    type: journal
    title: Generative MDX to PDF Compilation Architectures
    authors:
      - firstName: John
        lastName: Smith
    containerTitle: Journal of Web Engineering
    volume: "18"
    issue: "4"
    pages: 210-225
    issuedDate:
      year: 2025
      month: 8
    doi: 10.1016/j.jwe.2025.08.001
    language: en
---

##[extras-head] The extras head

The unified attribute syntax is the **extras head**: `@@type{…}`, written
adjacent to a construct with no space in between. This document exercises every
construct that reads it — anchor, heading, image/figure, cite, wiki link and the
seven table layers.

##[anchor] Anchor

Every item kind of §5 in one head: id, slug, title, `class`, a prop, a typed
prop, a CSS custom property and a bare flag (here `#rew` wins over the slug and
the head title over the extras title, §6.2).

[Anchor with every item kind](/docs/bar "Head title")@@anchorA{.extra, .class, #rew, `slug-a`, "Extras title", foo: "bar", baz: 23, isFoo: true, --wix: "sum", isBar}

The head title wins over the extras title (§6.2) and the extras value is dropped
with a warning:

[Head title wins](/docs "Head title")@@anchorA{"Extras title"}

`#id` beats the head slug, which beats the extras slug (§6.2); classes
accumulate head first (§6.4):

[Slug precedence](/docs)@@anchorB{`slug-extras`, #explicit, .from-extras}

A marker without an entry falls back to the layer default (`AnchorDefault`); an
anchor without any head stays the plain `<a>` element:

[Unclaimed marker](/docs)@@anchorZ{.hero}

[No marker at all](/docs)

URL modifiers are parsed before the head, and `rel:`/`target:` extras merge with
them (§7.2):

[Modifiers plus extras](https://example.com/bar--!;$)@@anchorB{rel: "prefetch", hreflang: "en"}

##[heading] Heading

`[slug]`, `("title")` and `@@type{…}` are all optional and independent; the
head's `[slug]` wins and `("title")` is a separate attribute, never the text.

###[heading-x]("Heading X")@@headingX{`slug-extras`, .fancy} Extras-routed heading

###[heading-y]("Heading Y")@@headingY{number: "auto"} Routed by the second marker

###[heading-plain] A plain heading without extras

##[image] Image and figure

`~?!!` declares a figure: the extras attach to the outermost node (`<figure>`),
the `w`/`h` marker stays on the inner `<img>` and the trailing text is the
caption.

~?!!h300w800[Alt text](https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp)@@figureX{.wide, #fig-1, --rotate: "5deg", isLazy} A figure caption with **markup**.

A plain image uses the `img` layer alone. The `!` marker (lazy) keeps the line
from being read as a Markdown link before the plugin sees it:

~!w800[Alt text](https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp)@@imageX{.thumb, foo: "bar"}

##[cite] Cite

The citation head is `[^^](ref)`; extras merge into the citation node while the
cite args win (§7.3). `#id`/slug feed the `cite-id` slot.

Minim [^^](paper-smith)@@citeX{.paper, #smith, note: "short"} esse do ut anim
proident est qui magna non elit quis eiusmod dolore.

A location argument stays the construct value:

[^^](suryana-2026 "hlm. 45")@@citeX{loc: "dropped", .thin}

##[wiki] Wiki link

Unchanged apart from the adjacent extras head; `href` is produced by the plugin
and is never overridable (§7.5).

[[Anim Esta (Officia) | Anim]]@@wikiX{.link, title: "dropped"} and [[Wireless]].

##[marker] Marker

The `{{type}}` marker (§10.1) has no HTML equivalent: its type is mandatory and
doubles as the routing key of the `marker` layer. An inline marker keeps its
paragraph, a block marker owns its line and takes that line's trailing text as
its children.

Inline {{footnote}} and block forms:

{{bibliography}}

An unclaimed type falls back to the layer default:

{{unknownType}} Rendered through `MarkerDefault`.

##[table] Table

The §8 declaration line carries the `<table>` extras and the head
(`[slug]`, `("title")`); the caption line is `|| extras content ||`; rows and
cells carry their extras at the front of the row/cell, after the last `|`.

|-[sales-2026]("Laporan Penjualan 2026")@@tableX{.striped, sortable: true}-|
||@@captionX{.caption-note} Laporan Penjualan 2026||
| Produk | Stok | Harga | Status |
| :---(200px)@@cellA{.v-top} | :---:{.v-top} | ---:(30%){.v-bottom} | :---: |
| @@cellB{.lead} Laptop Pro | 15 | 15.000.000 | [Tersedia](/docs) |
| Mouse Wireless | @@cellB{} > | 250.000 | Tersedia |@@rowB{.row-info}
| Keyboard Mekanikal | 0 | 850.000 | Habis |{.row-danger}
|===|@@tfootX{.total}
| Total Inventaris | > | 21.300.000 | - |

A table without a declaration line keeps working, and its layers fall back to
the built-in elements:

| Kolom A | Kolom B |
| --- | --- |
| satu | dua |
|===|
| total | dua |

##[latex] LaTeX and code

Inline `$H \rightarrow O$` and a display block:

$$((H \rightarrow O) \land \lnot O) \rightarrow \lnot H$$

```rust
fn extras(head: &str) -> Option<Head> {
    parse_extras(head).ok()
}
```

##[literal] Literal fallback

A malformed head never aborts the build: it stays literal text (§4.3).

[Unterminated](/docs)@@anchorX{foo: "bar" and the rest of the line is text.

A head separated by whitespace is not a head either: [Spaced](/docs) @@anchorA{.x}
