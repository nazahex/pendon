export default function PendonView() { return (<>
<p>A renderable is a <strong>value</strong>, so it reaches every position a value reaches: a
spec&#39;s own object, an array item, and a field whose value is itself an object.
The value travels as JSON when it is nested, and the field is resolved in place —
the fragment nests the way a bound <code>mdp</code> value does:
</p>
<p><a href="/docs" specs={{caption: (<><em>nested</em> in an object</>)}} tokens={[(<>one</>), "two"]} type="anchorA">Card</a>
</p>
<p>Nesting a field inside a field needs escapes, because a bare <code>;;</code> closes the
outer span (<code>\;</code> is a semicolon). The inner element&#39;s head is read when the
payload runs, so its field is a field again — the escaped semicolons stay as
text, which is the price of one escape level and the reason a deep tree is
better bound than written:
</p>
<p><a href="/docs" note={(<>outer ;;<a href="/u" type="anchorB" v={(<>inner <em>i</em></>)}>In</a> ;; tail</>)} type="anchorA">Card</a>
</p>
<p>A <code>bind</code> value and a renderable are the same thing written two ways — one names
a payload the data block authored, the other writes it at the position:
</p>
<p><a body={(<>A <strong>written</strong> payload</>)} href="/docs" note={(<>A <strong>bound</strong> payload</>)} type="anchorA">Card</a>
</p>
<p>A position that needs <strong>text</strong> is not reserved (§6.7, D-RF-6): the image wants a
string for <code>alt</code>, so it reads the authored <code>;;…;;</code> spelling and behaves exactly
as it did before renderables existed:
</p>
<img alt=";;*alt* payload;;" src="https://example.com/chart.png" />

</>); }
