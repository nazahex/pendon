A malformed head is literal text; it never aborts the build (§4.3).

[Unterminated](/docs)@@anchorA{foo: "bar" and the rest of the line is text.

[Type without letters](/docs)@@1anchor{.x}

[Empty value](/docs)@@anchorA{foo: }

[Unterminated quote](/docs)@@anchorA{foo: "bar}

[Trailing junk](/docs)@@anchorA{foo: "bar" x}

[Unterminated backtick](/docs)@@anchorA{`slug}

[Unterminated bare head](/docs){.x

A head separated from its construct, or from its own `{`, by whitespace is not a
head (§4.1):

[Spaced](/docs) {.x}

[Spaced body](/docs)@@anchorA {.x}

A valid head after a literal one still attaches to its construct:

[Valid](/docs)@@anchorA{.hero} followed by [plain](/docs) text.
