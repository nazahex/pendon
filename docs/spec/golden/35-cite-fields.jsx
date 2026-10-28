import { CiteShort } from '@comp/CiteShort'

export const frontmatter = {"references":{"book":{"id":"book","title":"A Book"},"paper":{"id":"paper","title":"A Paper"}}};
export const references = {"book":{"id":"book","title":"A Book"},"paper":{"id":"paper","title":"A Paper"}};
export const citations = [{"citation_number":1,"reference_id":"book"},{"citation_number":2,"locator":"p. 2","reference_id":"paper"},{"citation_number":1,"reference_id":"book"}];
export default function PendonView() { return (<>
<p>A <code>field.*</code> block moves a position (§6.1) and can rename a public field (D-4).
This entry is the proposal&#39;s &quot;simpler version&quot;: <code>reference_id</code> sits in the tick
and <code>locator</code> in the bare content, while the node publishes them as <code>ref_id</code> and
<code>loc</code> and numbers them as <code>no</code>:
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
<p>The same citation seen twice keeps one number, because the identity is still the
<code>(reference_id, locator)</code> pair:
</p>
<p><CiteShort reference={references["book"]} {...{ "bracket.bare": "p. 1" }} no={1} type="citeShort" /> again.
</p>

</>); }
