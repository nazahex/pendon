export default function PendonView() { return (<>
<p><code>;;;…;;;</code> is the block form: the payload is a sub-document, so its block
structure survives and the field becomes a fragment of block nodes. A slot or
spec value is <strong>one line</strong> (§6.7), so the block payload writes its breaks as
<code>\n</code> — they are materialised before the payload is parsed:
</p>
<p><a body={(<><h2>Payload heading</h2>
<p>A paragraph with a <a href="/Wiki" title="Wiki">Wiki</a> link.
</p>
<ul>
<li>one</li>
<li>two</li>
</ul>
<blockquote>
<p>quoted line
</p>
</blockquote>
</>)} href="/docs" title="Docs" type="anchorA">Card</a>
</p>
<p>The inline form drops the payload&#39;s <strong>document</strong> wrapper, so a single paragraph
contributes its inline nodes; the block form keeps it, so the same paragraph
arrives inside a <code>&lt;p&gt;</code>. That is the only difference between the two spellings —
both run the page&#39;s chain, both carry nodes:
</p>
<p><a body={(<><p>one paragraph, wrapper kept</p>
</>)} href="/docs" note={(<>still <em>inline</em>: no wrapper</>)} type="anchorA">Card</a>
</p>
<p>A payload is not a numbering island either: a citation inside it belongs to the
page, and one <em>before</em> the body text keeps the earlier number (§6.7, D-RF-8) —
fixture 53 measures that from the other side.
</p>

</>); }
