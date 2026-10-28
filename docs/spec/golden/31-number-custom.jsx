import { Heading } from '@comp/Heading'

export const headings = [{"id":"slug-heading","level":2,"text":"Title Heading - I.a (depth 2, so the missing parent counts as I)"},{"id":"slug","level":2,"text":"Will Be Numbered IV"},{"id":"will-be-numbered-iv-a-1-number-depth-3-even-though-heading-depth-is-2","level":2,"text":"Will Be Numbered IV.a.1 (number depth 3, even though heading depth is 2)"},{"id":"will-be-numbered-a","level":2,"subheadings":[{"id":"will-be-numbered-a-a","level":3,"text":"Will Be Numbered A.a"}],"text":"Will Be Numbered A"},{"id":"will-be-numbered-b","level":2,"text":"Will Be Numbered B"}];
export default function PendonView() { return (<>
<p><code>%%numberType</code> is an extras item (§5): the marker names a <code>[[task.number.custom]]</code>
block and carries its own depth (<code>%</code> count − 1). It always beats the element&#39;s
own numbering, so the same counter can run across headings, anchors and images.
</p>
<Heading depth={2} foo="bar" id="slug-heading" number={["I", "a"]} slug="slug-heading" title="Title Heading" type="HeadingA">Title Heading - I.a (depth 2, so the missing parent counts as I)</Heading><p><a href="/foo" number={["II"]} type="AnchorA">link</a> will be numbered II.
</p>
<p><a href="/foo" number={["II", "a"]} type="AnchorA">link</a> will be numbered II.a.
</p>
<p><a href="/foo" number={["II", "b"]} type="AnchorA">link</a> will be numbered II.b.
</p>
<p><a href="/foo" number={["II", "b", "1"]} type="AnchorA">link</a> will be numbered II.b.1.
</p>
<p><a href="/foo" number={["II", "b", "1", "1", "1"]} type="AnchorA">link</a> will be numbered II.b.1.1.1.
</p>
<p><a href="/foo" number={["II", "c"]}>link</a> will be numbered II.c.
</p>
<p><a href="/foo" number={["III"]}>link</a> will be numbered III.
</p>
<p><a href="/foo" number={["A"]}>link</a> will be numbered A.
</p>
<p><a href="/foo" number={["A", "A", "A"]}>link</a> will be numbered A.A.A.
</p>
<p><a href="/foo" number={["A", "B"]}>link</a> will be numbered A.B.
</p>
<img alt="image foo" number={["B"]} src="/foo.png" />
<p>Will be numbered B.
</p>
<img alt="image foo" number={["B", "A", "A", "A"]} src="/foo.png" />
<p>Will be numbered B.A.A.A.
</p>
<img alt="image foo" number={["A", "a", "I", "i", "1", "1"]} src="/foo.png" />
<p>Will be numbered A.a.I.i.1.1.
</p>
<Heading depth={2} id="slug" number={["IV"]} slug="slug">Will Be Numbered IV</Heading><Heading depth={2} id="will-be-numbered-iv-a-1-number-depth-3-even-though-heading-depth-is-2" number={["IV", "a", "1"]} slug="will-be-numbered-iv-a-1-number-depth-3-even-though-heading-depth-is-2">Will Be Numbered IV.a.1 (number depth 3, even though heading depth is 2)</Heading><Heading depth={2} id="will-be-numbered-a" number={["A"]} slug="will-be-numbered-a" title="Title" type="HeadingA">Will Be Numbered A</Heading><Heading depth={3} id="will-be-numbered-a-a" number={["A", "a"]} slug="will-be-numbered-a-a" type="HeadingA">Will Be Numbered A.a</Heading><Heading depth={2} id="will-be-numbered-b" number={["B"]} slug="will-be-numbered-b" type="HeadingA">Will Be Numbered B</Heading>
</>); }
