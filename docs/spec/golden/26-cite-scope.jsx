export const frontmatter = {"references":{"book":{"id":"book","title":"A Book","type":"book"},"paper":{"id":"paper","title":"A Paper","type":"article"}}};
export const references = {"book":{"id":"book","title":"A Book","type":"book"},"paper":{"id":"paper","title":"A Paper","type":"article"}};
export const citations = [{"citation_number":1,"locator":"hlm. 45","reference_id":"book"}];
export default function PendonView() { return (<>
<p>Frontmatter references are authored for <em>this</em> document, so the <code>references</code>
export carries all of them by default (<code>references_scope = &quot;all&quot;</code>, §2.6 of
RFC-0002). <code>paper</code> is never cited below, yet it still ships:
</p>
<p>Minim <sup class="cite-ref"><a href="#citeref-1-book" id="cite-1">[1]</a></sup> esse do ut anim proident.
</p>
<p><code>references_scope = &quot;cited&quot;</code> would drop <code>paper</code> from that export. The setting
never touches <code>citations</code>, <code>reference_id</code> or <code>citation_number</code>.
</p>

</>); }
