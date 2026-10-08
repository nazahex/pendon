export default function PendonView() { return (<>
<p>A malformed head is literal text; it never aborts the build (§4.3).
</p>
<p><a href="/docs">Unterminated</a>@@anchorA&#123;foo: &quot;bar&quot; and the rest of the line is text.
</p>
<p><a href="/docs">Type without letters</a>@@1anchor&#123;.x&#125;
</p>
<p><a href="/docs">Empty value</a>@@anchorA&#123;foo: &#125;
</p>
<p><a href="/docs">Unterminated quote</a>@@anchorA&#123;foo: &quot;bar&#125;
</p>
<p><a href="/docs">Trailing junk</a>@@anchorA&#123;foo: &quot;bar&quot; x&#125;
</p>
<p><a href="/docs">Unterminated backtick</a>@@anchorA&#123;`slug&#125;
</p>
<p><a href="/docs">Unterminated bare head</a>&#123;.x
</p>
<p>A head separated from its construct, or from its own <code>&#123;</code>, by whitespace is not a
head (§4.1):
</p>
<p><a href="/docs">Spaced</a> &#123;.x&#125;
</p>
<p><a href="/docs">Spaced body</a>@@anchorA &#123;.x&#125;
</p>
<p>A valid head after a literal one still attaches to its construct:
</p>
<p><a class="hero" href="/docs" type="anchorA">Valid</a> followed by <a href="/docs">plain</a> text.
</p>

</>); }
