---
references:
  book:
    id: book
    type: book
    title: A Book
    authors:
      - firstName: John
        lastName: Smith
    language: en
---

The citation head is `[^^](ref "loc")`: an unquoted reference and an optional
quoted location. Extras merge into the citation node while the cite args win
(§7.3). `#id`/slug feed the `cite-id` slot.

Minim [^^](book "hlm. 45")@@citeX{.paper, #smith, note: "short"} esse do ut anim proident.

A location argument stays the construct value even when the extras ask for one:

[^^](book "hlm. 45")@@citeX{loc: "dropped", .thin}

The same reference twice keeps its identity and gets the next index:

[^^](book)@@citeX{.repeat}

Retired spellings are not citations; they stay literal text (§7.3):

[^^](book, "hlm. 45") and [^^]("book") and [^^]()
