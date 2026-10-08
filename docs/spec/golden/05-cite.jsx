import { Cite } from '@comp/content/Cite'

export const frontmatter = {"cites":[{"id":"book","index":1,"loc":"hlm. 45"},{"id":"book","index":2}],"references":{"book":{"authors":[{"firstName":"John","lastName":"Smith"}],"id":"book","language":"en","title":"A Book","type":"book"}}};
export default function PendonView() { return (<>
<p>The citation head is `<a href="ref" title="loc">^^</a>`: an unquoted reference and an optional
quoted location. Extras merge into the citation node while the cite args win
(§7.3). <code>#id</code>/slug feed the <code>cite-id</code> slot.
</p>
<p>Minim <Cite reference={frontmatter.references["book"]} cite-id={"smith"} class={"paper"} index={1} loc={"hlm. 45"} note={"short"} type={"citeX"} /> esse do ut anim proident.
</p>
<p>A location argument stays the construct value even when the extras ask for one:
</p>
<p><Cite reference={frontmatter.references["book"]} class={"thin"} index={1} loc={"hlm. 45"} type={"citeX"} />
</p>
<p>The same reference twice keeps its identity and gets the next index:
</p>
<p><Cite reference={frontmatter.references["book"]} class={"repeat"} index={2} type={"citeX"} />
</p>
<p>Retired spellings are not citations; they stay literal text (§7.3):
</p>
<p><a href="book," title="hlm. 45">^^</a> and <a href="&quot;book&quot;">^^</a> and <a href="">^^</a>
</p>

</>); }
