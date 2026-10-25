import { Heading } from '@comp/Heading'

export default function PendonView() { return (<>
<p><code>%%numberType</code> is an extras item (§5): the marker names a <code>[[task.number.custom]]</code>
block and carries its own depth (<code>%</code> count − 1). It always beats the element&#39;s
own numbering, so the same counter can run across headings, anchors and images.
</p>
<Heading foo="bar" id="slug-heading" level={2} number={["I", "a"]} raw_title="Title Heading - I.a (depth 2, so the missing parent counts as I)" title="Title Heading" type="HeadingA">Title Heading - I.a (depth 2, so the missing parent counts as I)</Heading><p><a href="/foo" number={["II"]} type="AnchorA">link</a> will be numbered II.
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
<Heading id="slug" level={2} number={["IV"]} raw_title="Will Be Numbered IV">Will Be Numbered IV</Heading><Heading level={2} number={["IV", "a", "1"]} raw_title="Will Be Numbered IV.a.1 (number depth 3, even though heading depth is 2)" slug="will-be-numbered-iv-a-1-number-depth-3-even-though-heading-depth-is-2">Will Be Numbered IV.a.1 (number depth 3, even though heading depth is 2)</Heading><Heading level={2} number={["A"]} raw_title="Will Be Numbered A" slug="will-be-numbered-a" title="Title" type="HeadingA">Will Be Numbered A</Heading><Heading level={3} number={["A", "a"]} raw_title="Will Be Numbered A.a" slug="will-be-numbered-a-a" type="HeadingA">Will Be Numbered A.a</Heading><Heading level={2} number={["B"]} raw_title="Will Be Numbered B" slug="will-be-numbered-b" type="HeadingA">Will Be Numbered B</Heading>
</>); }
