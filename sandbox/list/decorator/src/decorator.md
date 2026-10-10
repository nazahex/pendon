# List & Blockquote Decorators

One decorator line, two owners: a line directly above a **blockquote** decorates
the quote (§9.2), a line directly above a **list** decorates the whole list
container (§9.3 L1). This fixture drives both layers end to end.

## Blockquote

A decorator line directly above the quote decorates it:

@@bqA{.q, #q1}

> a quoted *body*

When both a decorator line and an inner head are present the **inner head wins**;
an unclaimed type falls back to the built-in `<blockquote>` and still keeps every
extra (§9.2, §11 rule 3, D8):

@@bqA{.lost}

> @@quoteZ{.z} inner head wins

A quote with neither a head nor a decorator line is left alone:

> a plain quote

## List container

A container decorator above a list decorates the *container*, never the items
(§9.3 L1). The **marker decides the layer** (§9.4), the decorator's type only
selects the component:

@@unorderedA{.u, #l1}

- one
- two

An untyped decorator therefore routes through the marker's layer default:

@@{.o}

1. first
2. second

An unclaimed type falls back to the layer default too:

@@unorderedZ{.z}

* star one
* star two
