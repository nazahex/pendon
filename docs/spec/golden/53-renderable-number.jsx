export const frontmatter = {"references":{"alpha":{"id":"alpha","title":"Alpha","type":"book"},"beta":{"id":"beta","title":"Beta","type":"book"}}};
export const references = {"alpha":{"id":"alpha","title":"Alpha","type":"book"},"beta":{"id":"beta","title":"Beta","type":"book"}};
export default function PendonView() { return (<>
<p><code>number</code> and <code>cite</code> keep the <strong>page&#39;s</strong> scope: a renderable is not a numbering
island (D-RF-8). But a field is rendered <em>after</em> the chain has walked the
document, so its number cannot simply be the next one handed out — that would
number by <strong>output</strong> order, and §6.7 requires source order.
</p>
<p>So the field&#39;s citation identities are reserved <strong>where the field is written</strong>.
This citation is the first one in the source, even though the field renders last:
</p>
<p><a href="/docs" note={(<>first in source: <sup><a href="#1-alpha" id="cite-3">[1]</a></sup></>)} type="anchorA">Card</a>
</p>
<p>Body text cites <sup><a href="#2-beta" id="cite-1">[2]</a></sup> here, so <code>beta</code> is the second identity the document
introduced — not the first:
</p>
<p>The same identity keeps its number, whichever position cites it:
</p>
<p>Body text cites <sup><a href="#1-alpha" id="cite-2">[1]</a></sup> again.
</p>
<p><code>%%number</code> belongs to the same rule: the marker is the numbering pass&#39;s, so a
head inside a payload is numbered by the pass and not by the renderer. That half
is not pinned here yet — <code>number</code> numbers elements, not identities, so the
reservation this fixture proves for <code>cite</code> does not carry over for free.
</p>

</>); }
