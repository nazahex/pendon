export const frontmatter = {"references":{"book":{"id":"book","title":"A Book","type":"book"}}};
export const headings = [{"id":"slot-modifiers","level":1,"text":"Slot modifiers"},{"id":"mod-head","level":1,"text":""}];
export const references = {"book":{"id":"book","title":"A Book","type":"book"}};
export const citations = [{"citation_number":1,"locator":"hlm. 45","reference_id":"book"}];
export default function PendonView() { return (<>
<h1 id="slot-modifiers">Slot modifiers</h1>
<p>The bracket&#39;s <code>|…</code> run is <strong>pure configuration</strong> (§5.4, D-9): every symbol names
a table in <code>pendon.toml</code> and contributes that table&#39;s fields to the element.
Pendon ships no modifier of its own, so a symbol with no config contributes
nothing and is not a diagnostic — and neither the entry nor the run needs a
custom component to exist (D-8).
</p>
<p><a class="external" href="https://example.com/two" rel={["nofollow", "noopener"]} {...{ "soo.baz": ["wax", "wax"] }}>two symbols</a>
</p>
<p><code>?</code> and <code>!</code> both write <code>rel</code> and <code>soo.baz</code>, so two contributions to one path
become an array in <strong>written</strong> order — never sorted, and never deduplicated:
<code>rel</code> is <code>[&quot;nofollow&quot;, &quot;noopener&quot;]</code> and <code>soo.baz</code> keeps both <code>&quot;wax&quot;</code> entries.
</p>
<p><a class="hero external" href="https://example.com/class" rel="nofollow" {...{ "soo.baz": "wax" }} type="anchor">class accumulates</a>
</p>
<p><code>class</code> is the one field that adds up instead of clashing (§6.4), and a token
the element already carries is not repeated.
</p>
<p><a class="external" href="https://example.com/head" rel="ugc" {...{ "soo.baz": ["wax", "wax"] }} type="anchor">head wins</a>
</p>
<p>A modifier is configuration, the head is authored text: <code>rel</code> stays <code>ugc</code> and
the losing contribution is dropped with a warning.
</p>
<p><a href="https://example.com/none">unknown symbols</a>
</p>
<p><code>xyz</code> configures nothing, so this link carries exactly what its head does.
</p>
<img alt="An image" class="hero" data-loading="lazy" src="https://x.test/a.webp" />
<p>A block image reads the same run: <code>?</code> adds <code>class</code>, and <code>loading</code> is a prop the
<code>&lt;img&gt;</code> has no attribute for, so it lands under the <code>data-</code> prefix (§6.3).
</p>
<p>Minim <sup class="cite-ref hero cited"><a href="#citeref-1-book" id="cite-1" data-type="citeX" compact>[1]</a></sup> esse do ut anim proident.
</p>
<p>A citation reads the same run, and a <code>true</code> contribution is a <strong>bare flag</strong>
(<code>compact</code>), never <code>compact=&quot;compact&quot;</code> (§6.3). The run is configuration, so it
never writes the reference or the locator: the citation still numbers as usual.
</p>
<h1 class="marked" id="mod-head" title="Modifier heading"></h1>
<p>This heading and the one above differ only by the run: the entry defines no
component, only a modifier contribution.
</p>

</>); }
