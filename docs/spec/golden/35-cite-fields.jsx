import { CiteShort } from '@comp/CiteShort'

export const frontmatter = {"references":{"book":{"id":"book","title":"A Book"},"paper":{"id":"paper","title":"A Paper"}}};
export const references = {"book":{"id":"book","title":"A Book"},"paper":{"id":"paper","title":"A Paper"}};
export default function PendonView() { return (<>
<p>A <code>field.*</code> block moves a position (§6.1) and can rename a public field (D-4).
This entry puts <code>reference_id</code> in the tick and the author&#39;s own <code>locator</code> in the
bare content, while the node publishes them as <code>ref_id</code> and <code>loc</code> and numbers them
as <code>no</code>. <code>stable_fields = [&quot;reference_id&quot;, &quot;locator&quot;]</code> is what keeps the pair
meaningful: the identity follows the <strong>names</strong>, never the positions:
</p>
<p>Minim <CiteShort reference={references["book"]} {...{ "bracket.bare": "p. 1" }} no={1} type="citeShort" /> esse.
</p>
<p>Second <CiteShort reference={references["paper"]} loc="p. 2" no={2} type="citeShort" />.
</p>
<p>Assigning one slot position voids the whole preset (D-7), so a head that leaves
<code>reference_id</code> unassigned is an error rather than a citation — the bare content
stays the author&#39;s own field:
</p>
<p>[^p. 9]@@citeShort
</p>
<p>The same citation seen twice keeps one number, because <code>stable_fields</code> lists
<code>reference_id</code> and <code>locator</code> and both match:
</p>
<p><CiteShort reference={references["book"]} {...{ "bracket.bare": "p. 1" }} no={1} type="citeShort" /> again.
</p>

</>); }
