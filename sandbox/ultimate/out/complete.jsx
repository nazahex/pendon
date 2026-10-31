export const headings = [{"id":"inline-elements","level":2,"text":"Inline Elements"}];
export const references = {};
export const citations = [];
export default function PendonView() { return (<>
<section depth="2" id="inline-elements">
<h2>Inline Elements</h2>
<p>Anchor - <a {...{ "bracket.quote": "quote" }} {...{ "bracket.tick": "tick" }} class="extra class" href="/href" id="anchor-id" isElement key="value" {...{ "paren.tick": "tick" }} style="--color: #fff;" title="Title" type="anchorX">Inner Text</a>
</p>
<p>Cite -
</p>
</section>

</>); }
