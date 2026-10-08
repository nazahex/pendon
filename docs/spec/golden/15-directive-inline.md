Inline directives keep their paragraph (§10.2) and their content is inline
content. The head `[bracket]` / `("paren")` slots map to `id` / `level`, and the
backticked / quoted extras items map to `author` / `summary` (§6.1, §10.2, §10.4):

::note[note-1]("A level")@@note{`Jane Doe`, "A summary", .box, isOpen} some *inline* content:: and the
text after it stays inline.

An unclaimed type routes to the layer default (§11 rule 3):

::aside this one is not typed:: but it still becomes a node.

The closing run must be at least as long as the opening run, so a directive nests
and an inner `::` closes before the outer one (§10.2):

::outer a ::note b:: c:: trailing text stays outside.

A bare `::` and an unterminated opener stay literal text (§4.3):

:: alone, and ::note no close here.
