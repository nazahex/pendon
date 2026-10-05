A malformed head is literal text; it never aborts the build (§4.3).

[Unterminated](/docs)@@anchorA{foo: "bar" and the rest of the line is text.

[No braces](/docs)@@anchorA

[Type without letters](/docs)@@1anchor{.x}

[Empty head](/docs)@@{}

[Empty value](/docs)@@anchorA{foo: }

[Unterminated quote](/docs)@@anchorA{foo: "bar}

[Trailing junk](/docs)@@anchorA{foo: "bar" x}

[Unterminated backtick](/docs)@@anchorA{`slug}

A head separated from its construct by whitespace is not a head (§4.1):

[Spaced](/docs) @@anchorA{.x}

A valid head after a literal one still attaches to its construct:

[Valid](/docs)@@anchorA{.hero} followed by [plain](/docs) text.
