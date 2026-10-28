export default function PendonView() { return (<>
<p>The head is <strong>tagged</strong> (§5.3, §7.2). Without the tag the text is not an anchor at
all — <code>plugin-markdown</code> links it and the extras head stays literal (§14): no
<code>type</code>, no <code>class</code>.
</p>
<pre lang="text"><code>@[Inner text](/docs &quot;Head title&quot;)@@anchorA&#123;.hero&#125;
</code></pre>
<p><a href="/docs">Tagged</a>
<a href="/docs">Untagged</a>@@anchorA&#123;.hero&#125;
</p>
<p>§6.5: the preset assigns <code>bracket.bare → inner_text</code>, <code>paren.bare → href</code> and
<code>paren.quote → title</code>. The other three positions of the same head are bare fields
under their slot-qualified names — kept, never dropped (D-7).
</p>
<p><a {...{ "bracket.quote": "Head quote" }} {...{ "bracket.tick": "Head tick" }} href="/docs" {...{ "paren.tick": "Paren tick" }} title="Head title">Inner</a>
</p>
<p>The tag is read wherever a head starts, because an anchor is inline by nature:
</p>
<p>Inline: see <a href="/docs">the docs</a> for details.
</p>
<p>§6.2: the construct head wins over the extras, and the extras value is dropped.
</p>
<p><a href="/docs" title="Head title" type="anchorA">Head title wins</a>
</p>
<p><code>#id</code> beats the extras slug, and an extras slug becomes the id when nothing else
did (§6.2).
</p>
<p><a href="/docs" id="explicit" type="anchorA">Explicit id</a>
</p>
<p><a href="/docs" id="slug-only" type="anchorA">Slug only</a>
</p>
<p><code>href</code> is owned by the link; the extras value is an <code>Error</code> that names
<code>field.override.href</code> (§13, D-12 rule 5).
</p>
<p><a href="/docs" type="anchorA">Href is owned</a>
</p>
<p>§14 (D-16): automatic external-link attributes and the URL-suffix modifiers are
gone. A URL is a URL — no <code>target</code>, no <code>rel</code> — and <code>^</code>, <code>!</code>, <code>$</code>, <code>;;</code>, <code>--</code> are
ordinary characters now:
</p>
<p><a href="https://example.com/bar">No external defaults</a>
<a href="/docs^!$;;--">Suffix is text</a>
</p>
<p><code>target</code> and <code>rel</code> are ordinary extras keys now (§6.3):
</p>
<p><a href="https://example.com/bar" hreflang="en" rel="prefetch" target="_self" type="anchorA">Extras attributes</a>
</p>
<p>A marker with no configured entry keeps the built-in <code>&lt;a&gt;</code> element and carries
the marker as its <code>type</code> attribute (§11 rule 3):
</p>
<p><a class="hero" href="/docs" type="anchorZ">Unclaimed marker</a>
</p>
<p><a href="/docs">No marker at all</a>
</p>

</>); }
