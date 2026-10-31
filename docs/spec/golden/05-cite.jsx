import { Cite } from '@comp/Cite'

export const frontmatter = {"references":{"book":{"authors":[{"firstName":"John","lastName":"Smith"}],"id":"book","language":"en","title":"A Book","type":"book"}}};
export const references = {"book":{"authors":[{"firstName":"John","lastName":"Smith"}],"id":"book","language":"en","title":"A Book","type":"book"}};
export const citations = [{"citation_number":1,"reference_id":"book"},{"citation_number":1,"reference_id":"book"},{"citation_number":1,"reference_id":"book"},{"citation_number":1,"reference_id":"book"},{"citation_number":1,"reference_id":"book"}];
export default function PendonView() { return (<>
<p>The citation head is `[^reference_id]`: the caret is the slot tag and the bare
content is the reference (§7.3). Every other position is an ordinary field of the
author&#39;s — the bracket&#39;s quote included — so a head never loses content for want
of a preset row, and an adjacent extras head decorates the node without ever
writing the reference (§6.2):
</p>
<p>Minim <Cite reference={references["book"]} {...{ "bracket.quote": "hlm. 45" }} citation_number={1} class="paper" id="smith" note="short" type="citeX" /> esse do ut anim proident.
</p>
<p><code>locator</code> is not a cite field any more, so an extras <code>locator:</code> prop is an ordinary
prop like any other — the head&#39;s quote stays <code>bracket.quote</code>:
</p>
<p><Cite reference={references["book"]} {...{ "bracket.quote": "hlm. 45" }} citation_number={1} class="thin" locator="kept" type="citeX" />
</p>
<p><code>citation_number</code> is stable by the reference alone (<code>stable_fields</code> defaults to
<code>[&quot;reference_id&quot;]</code>), so a second mention keeps the number whatever else differs:
</p>
<p><Cite reference={references["book"]} citation_number={1} class="repeat" type="citeX" />
</p>
<p>A repeated citation keeps its number; only its anchor is new (<code>cite-&lt;n&gt;</code>):
</p>
<p><Cite reference={references["book"]} citation_number={1} class="again" type="citeX" />
</p>
<p>Every position the preset does not name stays a bare field, so a paren&#39;s contents
are ordinary fields of the node (§6.1, D-7):
</p>
<p><Cite reference={references["book"]} {...{ "bracket.quote": "hlm. 7" }} citation_number={1} {...{ "paren.bare": "paperback" }} {...{ "paren.quote": "Reprint" }} type="citeX" />
</p>
<p>A head without a reference is an error, not a citation (§7.3): <code>[^]</code> and <code>[^&quot;book&quot;]</code>
stay text, and so does a citation of a reference the document does not know. The
retired `<a href="book" title="hlm. 45">^^</a>` spelling is not a citation either — its second caret
is now the reference:
</p>
<p>[^] and [^&quot;book&quot;] and [^nosuch]
</p>

</>); }
