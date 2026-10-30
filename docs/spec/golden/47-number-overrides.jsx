import { Heading } from '@comp/Heading'

export const headings = [{"id":"one","level":1,"subheadings":[{"id":"one-one","level":2,"subheadings":[{"id":"one-one-one","level":3,"text":"One.one.one"}],"text":"One.one"}],"text":"One"},{"id":"two","level":1,"text":"Two"},{"id":"three","level":1,"text":"Three"}];
export default function PendonView() { return (<>
<p><code>field.override.number.*</code> on a <strong>numbering block</strong> renames the fields that block
publishes (ADR-0016 rule 2, S2-9.3). It has to live there — not in the
element&#39;s <code>custom</code> entry — because a <code>%%</code> group is declared once for a whole
task: the counter is the only place that knows every element it will number.
</p>
<Heading counter={["I"]} counter_type="HeadingA" depth={1} id="one" level={1} slug="one" type="HeadingA">One</Heading><Heading counter={["I", "a"]} counter_type="HeadingA" depth={2} id="one-one" level={2} slug="one-one" type="HeadingA">One.one</Heading><Heading counter={["I", "a", "1"]} counter_type="HeadingA" depth={3} id="one-one-one" level={3} slug="one-one-one" type="HeadingA">One.one.one</Heading><Heading depth={1} id="two" {...{ "number.depth": 1 }} {...{ "number.raw": ["1"] }} {...{ "number.type": "HeadingB" }} slug="two" type="HeadingB">Two</Heading><Heading counter={["II"]} counter_type="HeadingA" depth={1} id="three" level={1} slug="three" type="HeadingA">Three</Heading><p><a href="/first" num={["1"]} num_depth={1} num_type="numberC">First</a> is numbered by a marker group.
</p>
<p><a href="/second" num={["1", "1", "1"]} num_depth={3} num_type="numberC">Second</a> is numbered three levels deep.</p>

</>); }
