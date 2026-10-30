export default function PendonView() { return (<>
<p>This page&#39;s one anchor carries an extras object a path can address and — because
<code>[[task.anchor.number]]</code> selects it — the public numbering fields. A row is built
from the projected fields the renderer sees, so <code>number.raw</code> is reachable by
<code>include</code> / <code>only</code> / <code>preset = &quot;all&quot;</code> and by nothing else: it is not in the
<code>pendon</code> preset.
</p>
<p><a href="/buah/jambu_biji" {...{ "number.depth": 1 }} {...{ "number.raw": ["1"] }} price={{details: {amount: {total: 12, unit: "thousand"}, currency: "IDR"}}}>Jambu biji</a>
</p>

</>); }
