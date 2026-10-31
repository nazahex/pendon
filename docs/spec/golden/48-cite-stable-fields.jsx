import { Cite } from '@comp/Cite'
import { CiteY } from '@comp/CiteY'

export const frontmatter = {"references":{"book":{"id":"book","title":"A Book","type":"book"},"paper":{"id":"paper","title":"A Paper","type":"article"}}};
export const references = {"book":{"id":"book","title":"A Book","type":"book"},"paper":{"id":"paper","title":"A Paper","type":"article"}};
export default function PendonView() { return (<>
<p>§7.3: <code>citation_number</code> is stable for a repeated <strong>identity</strong>, and the identity is
whatever <code>[task.cite] stable_fields</code> lists — public field paths, before
<code>field.override</code> (D-4), in comparison order. <code>citeX</code> here produces all four names
listed in the config: <code>reference_id</code>, the paren&#39;s bare content as <code>locator_type</code>,
the paren&#39;s quote as <code>locator_value</code>, and the extras <code>&quot;…&quot;</code> as <code>author</code>:
</p>
<p>First <Cite reference={references["book"]} author="Smith" {...{ "bracket.quote": "p. 1" }} citation_number={1} locator_type="paper" locator_value="ed. 2001" type="citeX" /> esse do ut anim proident.
</p>
<p>The same four values keep the number — only the physical anchor <code>cite-&lt;n&gt;</code> is new:
</p>
<p>Second <Cite reference={references["book"]} author="Smith" {...{ "bracket.quote": "p. 1" }} citation_number={1} locator_type="paper" locator_value="ed. 2001" type="citeX" /> esse.
</p>
<p>One listed field changing is enough for a new number. Here the author differs:
</p>
<p><Cite reference={references["book"]} author="Jones" {...{ "bracket.quote": "p. 1" }} citation_number={2} locator_type="paper" locator_value="ed. 2001" type="citeX" /> esse.
</p>
<p>And here <code>locator_value</code> does:
</p>
<p><Cite reference={references["book"]} author="Smith" {...{ "bracket.quote": "p. 1" }} citation_number={3} locator_type="paper" locator_value="ed. 2002" type="citeX" /> esse.
</p>
<p><code>citeY</code> reads the same <code>stable_fields</code>, but it produces only two of them: the
reference comes from the paren&#39;s quote and the author from the extras tick, while
<code>locator_type</code> and <code>locator_value</code> stay absent. An absent field still counts as a
value, so two <code>citeY</code> citations match each other — and a <code>citeY</code> never matches a
<code>citeX</code>, because &quot;no <code>locator_type</code>&quot; is not <code>paper</code>:
</p>
<p><CiteY author="Smith" citation_number={4} reference_id="book" type="citeY" /> esse.
</p>
<p><CiteY author="Smith" citation_number={4} reference_id="book" type="citeY" /> esse.
</p>

</>); }
