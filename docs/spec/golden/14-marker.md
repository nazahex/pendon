The `type` of a marker is mandatory (§10.1) and is the routing key of the §11
component set. This fixture configures typed entries but no layer default, so an
unclaimed type falls back to the built-in `<span>` / `<div>`.

Inline markers keep their paragraph: see {{note}}, {{aside}} and the text after
them, plus an unclaimed one ({{unknown}}).

The block form owns its line and carries the trailing text of that line as its
children, so a caption needs a template with `{children}` (§11 rule 6):

{{bibliography}} Every reference cited above.

An unclaimed block type renders the `<div>` fallback with the same children:

{{ghost}} Trailing text of an unclaimed block marker.

The extras head must be adjacent, so only the first marker reads it:
{{aside}}@@asideAB{#a1, .box, isOpen} keeps its attributes, while
{{aside}} @@asideAB{.box} does not.

A marker without a legal type stays literal text: {{}}, {{ note }}, {{a-b}} and
an unterminated {{note are all text.

§6.1/§10.1: the groups sit between `}}` and an adjacent extras head and beat a
same-key body item (§6.2), while a malformed group is not consumed — the marker
still renders and the group stays literal text (§4.3):

{{bibliography}}[refs]("Cited sources"){`slug-body`, "title-body", .box} Every reference cited above.

{{bibliography}}[unclosed("Cited") still a caption.
