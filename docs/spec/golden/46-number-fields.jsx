import { Heading } from '@comp/Heading'

export const headings = [{"id":"chapter","level":1,"subheadings":[{"id":"section","level":2,"subheadings":[{"id":"subsection","level":3,"text":"Subsection"}],"text":"Section"},{"id":"a-plain-section","level":2,"text":"A Plain Section"}],"text":"Chapter"}];
export default function PendonView() { return (<>
<p>The numbering pass publishes <strong>three public fields</strong> (ADR-0016), never one
<code>number</code> attribute: <code>number.raw</code> is the array of formatted counters,
<code>number.depth</code> is the depth the counter used and <code>number.type</code> is the type that
selected the counter. <code>__number</code> — the marker&#39;s internal transport — is read by
the pass and never reaches this page.
</p>
<p>The first counter is scoped to <code>HeadingA</code>/<code>HeadingB</code>, so <code>number.type</code> is the
element&#39;s own type across three levels:
</p>
<Heading depth={1} id="chapter" {...{ "number.depth": 1 }} {...{ "number.raw": ["I"] }} {...{ "number.type": "HeadingA" }} slug="chapter" type="HeadingA">Chapter</Heading><Heading depth={2} id="section" {...{ "number.depth": 2 }} {...{ "number.raw": ["I", "a"] }} {...{ "number.type": "HeadingB" }} slug="section" type="HeadingB">Section</Heading><Heading depth={3} id="subsection" {...{ "number.depth": 3 }} {...{ "number.raw": ["I", "a", "1"] }} {...{ "number.type": "HeadingB" }} slug="subsection" type="HeadingB">Subsection</Heading><p>A heading no <code>type</code> selects is numbered by the second block, and it publishes no
<code>number.type</code> at all — a missing field is dropped, never emitted empty:
</p>
<Heading depth={2} id="a-plain-section" {...{ "number.depth": 2 }} {...{ "number.raw": ["1"] }} slug="a-plain-section">A Plain Section</Heading><p>A marker carries its own type, so <code>number.type</code> here is the marker&#39;s <code>numberB</code>
and not the anchor&#39;s <code>AnchorA</code>; <code>number.depth</code> is the marker&#39;s depth (<code>%%</code> = 1):
</p>
<p><a href="/a" {...{ "number.depth": 1 }} {...{ "number.raw": ["1"] }} {...{ "number.type": "numberB" }} type="AnchorA">Link</a> is numbered by the marker.
</p>

</>); }
