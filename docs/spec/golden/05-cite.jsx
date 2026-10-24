import { Cite } from '@comp/Cite'

export const frontmatter = {"references":{"book":{"authors":[{"firstName":"John","lastName":"Smith"}],"id":"book","language":"en","title":"A Book","type":"book"}}};
export const references = {"book":{"authors":[{"firstName":"John","lastName":"Smith"}],"id":"book","language":"en","title":"A Book","type":"book"}};
export const citations = [{"citation_number":1,"instance_index":1,"loc":"hlm. 45","ref_id":"book"},{"citation_number":1,"instance_index":2,"loc":"hlm. 45","ref_id":"book"},{"citation_number":2,"instance_index":3,"ref_id":"book"},{"citation_number":2,"instance_index":4,"ref_id":"book"}];
export default function PendonView() { return (<>
<p>The citation head is `<a href="ref" title="loc">^^</a>`: an unquoted reference and an optional
quoted location. Extras merge into the citation node while the cite args win
(§7.3). <code>#id</code>/slug feed the <code>cite-id</code> slot.
</p>
<p>Minim <Cite reference={references["book"]} citation_number={1} cite-id="smith" class="paper" instance_index={1} loc="hlm. 45" note="short" type="citeX" /> esse do ut anim proident.
</p>
<p>A location argument stays the construct value even when the extras ask for one:
</p>
<p><Cite reference={references["book"]} citation_number={1} class="thin" instance_index={2} loc="hlm. 45" type="citeX" />
</p>
<p>A locator is part of the citation identity: the same reference with a different
(or no) locator gets a new number.
</p>
<p><Cite reference={references["book"]} citation_number={2} class="repeat" instance_index={3} type="citeX" />
</p>
<p>A repeated <code>(ref_id, loc)</code> keeps its <code>citation_number</code> but takes a fresh
<code>instance_index</code>, so every physical occurrence has its own backlink anchor:
</p>
<p><Cite reference={references["book"]} citation_number={2} class="again" instance_index={4} type="citeX" />
</p>
<p>Retired spellings are not citations; they stay literal text (§7.3):
</p>
<p><a href="book," title="hlm. 45">^^</a> and <a href="&quot;book&quot;">^^</a> and <a href="">^^</a>
</p>

</>); }
