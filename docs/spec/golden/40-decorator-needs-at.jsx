export default function PendonView() { return (<>
<p>§9.1/D-10: <code>@@</code> is mandatory. A bare head line is <strong>literal text</strong> — it has no
construct touching it, so it is not a decorator and never configures the block
below it (§14):
</p>
<p>&#123;.lead, #nope&#125;
</p>
<blockquote>a quote that keeps no extras</blockquote><p>The same head with the sigil does decorate the next block, so the two spellings
are not interchangeable:
</p>
<blockquote class="lead" id="nope">a quote that gets both</blockquote>
</>); }
