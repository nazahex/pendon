export default function PendonView() { return (<>
<p>§9.1: a <strong>decorator line</strong> above a paragraph decorates it; the line touches the
block it decorates (a blank line between them is allowed too):
</p>
<p class="lead" id="p1">A plain decorated paragraph.
</p>
<p>§9.1: the same for a code fence — a decorator-looking line <em>inside</em> the fence
stays literal text (§4.3), only the line above it decorates:
</p>
<pre class="src" lang="rust" slug="listing-1" type="codeBlock"><code>fn main() &#123;&#125;
</code></pre>
<p>§9.1/§6.1: a decorator line carries the positional groups too — the typed form and
the untyped <code>@@[…]</code> spelling, whose slots default to <code>slug</code> / <code>title</code>:
</p>
<p class="lead" slug="intro" title="Aside title" type="aside">A paragraph decorated by a typed group head.
</p>
<p slug="intro2" title="Another title">A paragraph decorated by the <code>@@</code>-only spelling.
</p>

</>); }
