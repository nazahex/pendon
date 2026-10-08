§9.2: a quote carries its extras through an **inner head**, right after the `>`
marker (a space between marker, head and content is allowed, §4.1):

> @@quoteA{.q, #q1} quoted *text*

§9.2/§9.1: a **decorator line** directly above the quote decorates it too (the
line touches the `>` it decorates, §9.1):

@@{.outer}
> a plain quoted body

§9.2: when both exist the inner head wins; an unclaimed type falls back to the
built-in `<blockquote>` and still keeps every extra (§11 rule 3, D8):

@@quoteA{.lost}
> @@quoteZ{.z} inner wins
