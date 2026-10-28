import { Cite } from '@comp/Cite'

export const frontmatter = {"references":{"book":{"authors":[{"firstName":"John","lastName":"Smith"}],"id":"book","language":"en","title":"A Book","type":"book"}}};
export const references = {"book":{"authors":[{"firstName":"John","lastName":"Smith"}],"id":"book","language":"en","title":"A Book","type":"book"}};
export const citations = [{"citation_number":1,"locator":"hlm. 45","reference_id":"book"},{"citation_number":1,"locator":"hlm. 45","reference_id":"book"},{"citation_number":2,"reference_id":"book"},{"citation_number":2,"reference_id":"book"},{"citation_number":3,"locator":"hlm. 7","reference_id":"book"}];
export default function PendonView() { return (<>
<p>The citation head is `[^reference_id &quot;locator&quot;]`: the caret is the slot tag, the
bare content is the reference and the quoted one the locator (§7.3). They land in
the preset&#39;s two fields, so the node carries <code>reference_id</code> and <code>locator</code>, and an
adjacent extras head decorates the node without ever writing them (§6.2):
</p>
<p>Minim <Cite reference={references["book"]} citation_number={1} cite-id="smith" class="paper" locator="hlm. 45" note="short" type="citeX" /> esse do ut anim proident.
</p>
<p>The locator stays the construct value even when the extras ask for one:
</p>
<p><Cite reference={references["book"]} citation_number={1} class="thin" locator="hlm. 45" type="citeX" />
</p>
<p>A locator is part of the citation identity: the same reference with a different
(or no) locator gets a new number.
</p>
<p><Cite reference={references["book"]} citation_number={2} class="repeat" type="citeX" />
</p>
<p>A repeated <code>(reference_id, locator)</code> keeps its <code>citation_number</code>:
</p>
<p><Cite reference={references["book"]} citation_number={2} class="again" type="citeX" />
</p>
<p>Every position the preset does not name stays a bare field, so a paren&#39;s contents
are ordinary fields of the node (§6.1, D-7):
</p>
<p><Cite reference={references["book"]} citation_number={3} locator="hlm. 7" {...{ "paren.bare": "paperback" }} {...{ "paren.quote": "Reprint" }} type="citeX" />
</p>
<p>A head without a reference is an error, not a citation (§7.3): <code>[^]</code> and
<code>[^&quot;book&quot;]</code> stay text, and so does a citation of a reference the document does
not know. The retired `<a href="book" title="hlm. 45">^^</a>` spelling is not a citation either —
its second caret is now the reference:
</p>
<p>[^] and [^&quot;book&quot;] and [^nosuch]
</p>

</>); }
