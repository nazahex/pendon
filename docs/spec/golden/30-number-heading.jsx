import { Heading } from '@comp/Heading'

export default function PendonView() { return (<>
<p><code>[[task.heading.number]]</code> scopes one counter with <code>type</code> selectors (ADR-0010).
The plugin writes <code>attrs.number</code> — an array of one string per depth — and never
renders it; here the component&#39;s <code>&#123;...attrs&#125;</code> spread prints the array.
</p>
<Heading level={1} number={["1"]} raw_title="Will Be Numbered 1" slug="will-be-numbered-1" type="HeadingA">Will Be Numbered 1</Heading><Heading level={2} number={["1", "1"]} raw_title="Will Be Numbered 1.1" slug="will-be-numbered-1-1" type="HeadingB">Will Be Numbered 1.1</Heading><Heading level={2} number={["1", "2"]} raw_title="Will Be Numbered 1.2" slug="will-be-numbered-1-2" type="HeadingA">Will Be Numbered 1.2</Heading><Heading level={3} number={["1", "2", "a"]} raw_title="Will Be Numbered 1.2.a" slug="will-be-numbered-1-2-a" type="HeadingB">Will Be Numbered 1.2.a</Heading><Heading level={5} number={["1", "2", "a", "1", "1"]} raw_title="Will Be Numbered 1.2.a.1.1 (an undefined depth style defaults to decimal)" slug="will-be-numbered-1-2-a-1-1-an-undefined-depth-style-defaults-to-decimal" type="HeadingB">Will Be Numbered 1.2.a.1.1 (an undefined depth style defaults to decimal)</Heading><Heading level={2} raw_title="Will Not Be Numbered" slug="will-not-be-numbered" type="HeadingC">Will Not Be Numbered</Heading><Heading level={2} raw_title="Will Not Be Numbered" slug="will-not-be-numbered">Will Not Be Numbered</Heading><Heading level={1} raw_title="Will Not Be Numbered" slug="will-not-be-numbered">Will Not Be Numbered</Heading><Heading level={2} number={["1", "3"]} raw_title="Will Be Numbered 1.3" slug="will-be-numbered-1-3" type="HeadingA">Will Be Numbered 1.3</Heading>
</>); }
