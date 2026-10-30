import { Heading } from '@comp/Heading'

export const headings = [{"id":"slug-heading","level":2,"text":"Title Heading - I.a (depth 2, so the missing parent counts as I)"},{"id":"slug","level":2,"text":"Will Be Numbered IV"},{"id":"will-be-numbered-iv-a-1-number-depth-3-even-though-heading-depth-is-2","level":2,"text":"Will Be Numbered IV.a.1 (number depth 3, even though heading depth is 2)"},{"id":"will-be-numbered-a","level":2,"subheadings":[{"id":"will-be-numbered-a-a","level":3,"text":"Will Be Numbered A.a"}],"text":"Will Be Numbered A"},{"id":"will-be-numbered-b","level":2,"text":"Will Be Numbered B"}];
export default function PendonView() { return (<>
<p><code>%%numberType</code> is an extras item (§5): the marker names a <code>[[task.number.custom]]</code>
block and carries its own depth (<code>%</code> count − 1). It always beats the element&#39;s
own numbering, so the same counter can run across headings, anchors and images.
</p>
<Heading depth={2} foo="bar" id="slug-heading" {...{ "number.depth": 2 }} {...{ "number.raw": ["I", "a"] }} {...{ "number.type": "numberB" }} slug="slug-heading" title="Title Heading" type="HeadingA">Title Heading - I.a (depth 2, so the missing parent counts as I)</Heading><p><a href="/foo" {...{ "number.depth": 1 }} {...{ "number.raw": ["II"] }} {...{ "number.type": "numberB" }} type="AnchorA">link</a> will be numbered II.
</p>
<p><a href="/foo" {...{ "number.depth": 2 }} {...{ "number.raw": ["II", "a"] }} {...{ "number.type": "numberB" }} type="AnchorA">link</a> will be numbered II.a.
</p>
<p><a href="/foo" {...{ "number.depth": 2 }} {...{ "number.raw": ["II", "b"] }} {...{ "number.type": "numberB" }} type="AnchorA">link</a> will be numbered II.b.
</p>
<p><a href="/foo" {...{ "number.depth": 3 }} {...{ "number.raw": ["II", "b", "1"] }} {...{ "number.type": "numberB" }} type="AnchorA">link</a> will be numbered II.b.1.
</p>
<p><a href="/foo" {...{ "number.depth": 5 }} {...{ "number.raw": ["II", "b", "1", "1", "1"] }} {...{ "number.type": "numberB" }} type="AnchorA">link</a> will be numbered II.b.1.1.1.
</p>
<p><a href="/foo" {...{ "number.depth": 2 }} {...{ "number.raw": ["II", "c"] }} {...{ "number.type": "numberB" }}>link</a> will be numbered II.c.
</p>
<p><a href="/foo" {...{ "number.depth": 1 }} {...{ "number.raw": ["III"] }} {...{ "number.type": "numberB" }}>link</a> will be numbered III.
</p>
<p><a href="/foo" {...{ "number.depth": 1 }} {...{ "number.raw": ["A"] }} {...{ "number.type": "numberA" }}>link</a> will be numbered A.
</p>
<p><a href="/foo" {...{ "number.depth": 3 }} {...{ "number.raw": ["A", "A", "A"] }} {...{ "number.type": "numberA" }}>link</a> will be numbered A.A.A.
</p>
<p><a href="/foo" {...{ "number.depth": 2 }} {...{ "number.raw": ["A", "B"] }} {...{ "number.type": "numberA" }}>link</a> will be numbered A.B.
</p>
<img alt="image foo" {...{ "number.depth": 1 }} {...{ "number.raw": ["B"] }} {...{ "number.type": "numberA" }} src="/foo.png" />
<p>Will be numbered B.
</p>
<img alt="image foo" {...{ "number.depth": 4 }} {...{ "number.raw": ["B", "A", "A", "A"] }} {...{ "number.type": "numberA" }} src="/foo.png" />
<p>Will be numbered B.A.A.A.
</p>
<img alt="image foo" {...{ "number.depth": 6 }} {...{ "number.raw": ["A", "a", "I", "i", "1", "1"] }} {...{ "number.type": "numberC" }} src="/foo.png" />
<p>Will be numbered A.a.I.i.1.1.
</p>
<Heading depth={2} id="slug" {...{ "number.depth": 1 }} {...{ "number.raw": ["IV"] }} {...{ "number.type": "numberB" }} slug="slug">Will Be Numbered IV</Heading><Heading depth={2} id="will-be-numbered-iv-a-1-number-depth-3-even-though-heading-depth-is-2" {...{ "number.depth": 3 }} {...{ "number.raw": ["IV", "a", "1"] }} {...{ "number.type": "numberB" }} slug="will-be-numbered-iv-a-1-number-depth-3-even-though-heading-depth-is-2">Will Be Numbered IV.a.1 (number depth 3, even though heading depth is 2)</Heading><Heading depth={2} id="will-be-numbered-a" {...{ "number.depth": 2 }} {...{ "number.raw": ["A"] }} {...{ "number.type": "HeadingA" }} slug="will-be-numbered-a" title="Title" type="HeadingA">Will Be Numbered A</Heading><Heading depth={3} id="will-be-numbered-a-a" {...{ "number.depth": 3 }} {...{ "number.raw": ["A", "a"] }} {...{ "number.type": "HeadingA" }} slug="will-be-numbered-a-a" type="HeadingA">Will Be Numbered A.a</Heading><Heading depth={2} id="will-be-numbered-b" {...{ "number.depth": 2 }} {...{ "number.raw": ["B"] }} {...{ "number.type": "HeadingA" }} slug="will-be-numbered-b" type="HeadingA">Will Be Numbered B</Heading>
</>); }
