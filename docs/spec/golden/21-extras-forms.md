The bare `{…}` spelling is the same head as `@@{…}` (§3); the `@@` prefix only
adds the optional type marker (§4.1):

[Bare head](/docs){.hero, #bare}

[Typed head](/docs)@@anchorA{.typed}

A type may stand alone as a type-only head; the symbol after it stays text, so a
head at the end of a sentence does not gain a space (§4.1):

[Type only](/docs)@@anchorA.

[Type and dash](/docs)@@anchorX-tail

An empty head carries nothing at all:

[Empty](/docs){}

[Empty typed](/docs)@@anchorY{}

The same heads attach to a heading and to table cells (§7.4, §8):

###@@headingX{`slug-a`, .fancy} A typed heading

| A | B |
| :---@@cellA{.v-top} | ---: |
|@@cellB{.lead} 1 | 2 |
