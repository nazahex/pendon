export const headings = [{"id":"a-heading-that-gets-no-decorator","level":2,"text":"A heading that gets no decorator"}];
export default function PendonView() { return (<>
<p>§9.1/D-10 rule 3 (S2-5.2): a decorator that no pass in the chain will ever claim
is <strong>removed with a <code>Warning</code></strong>. It never renders as a stray <code>@@…</code> line, and it
never decorates a block it does not own.
</p>
<p>The block can never receive a decorator at all (a thematic break):
</p>
<hr />
<p>The block belongs to a receiver this chain does not have — <code>plugin</code> is
<code>markdown</code> only, so <code>section</code>, <code>table</code>, <code>list</code> and <code>blockquote</code> are absent:
</p>
<h2>A heading that gets no decorator</h2>
<ul>
<li>a list that gets no decorator</li>
</ul>
<blockquote>
<p>a quote that gets no decorator
</p>
</blockquote>
<p>| A |
| --- |
| 1 |
</p>
<p>Two decorator lines in a row: only the last one applies, so the first is
overridden and dropped (<code>Overridden</code>):
</p>
<p class="b" type="second">A paragraph that keeps only <code>second</code>.
</p>

</>); }
