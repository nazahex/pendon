import AdvancedImage from '@comp/content/AdvancedImage'
import Figure from '@comp/content/Figure'
import Table from '@comp/content/Table'
import { AnchorAB } from '@comp/shared/Anchor'
import { AnchorDefault } from '@comp/shared/Anchor'
import { Bibliography } from '@comp/content/Cite'
import { Cite } from '@comp/content/Cite'
import { HeadingDefault } from '@comp/content/Heading'
import { HeadingXY } from '@comp/content/Heading'
import { MarkerDefault } from '@comp/content/Marker'
import { TableBody } from '@comp/content/Table'
import { TableCaption } from '@comp/content/Table'
import { TableCellAB } from '@comp/content/Table'
import { TableCellDefault } from '@comp/content/Table'
import { TableFoot } from '@comp/content/Table'
import { TableHead } from '@comp/content/Table'
import { TableRowAB } from '@comp/content/Table'
import { TableRowDefault } from '@comp/content/Table'

export const frontmatter = {"cites":[{"id":"paper-smith","index":1},{"id":"suryana-2026","index":2,"loc":"hlm. 45"}],"references":{"paper-smith":{"authors":[{"firstName":"John","lastName":"Smith"}],"containerTitle":"Journal of Web Engineering","doi":"10.1016/j.jwe.2025.08.001","id":"paper-smith","issue":"4","issuedDate":{"month":8,"year":2025},"language":"en","pages":"210-225","title":"Generative MDX to PDF Compilation Architectures","type":"journal","volume":"18"},"suryana-2026":{"authors":[{"firstName":"Eko","lastName":"Suryana"}],"id":"suryana-2026","isbn":"978-602-0000-00-0","issuedDate":{"year":2026},"language":"id","publisher":"TechPress Indonesia","publisherLocation":"Jakarta","title":"Masa Depan Rekayasa Perangkat Lunak","type":"book"}},"title":"Unified Syntax Demo"};
export const headings = [{"id":"extras-head","text":"The extras head","level":2},{"id":"anchor","text":"Anchor","level":2},{"id":"heading","text":"Heading","level":2,"subheadings":[{"id":"heading-x","text":"Extras-routed heading","level":3},{"id":"heading-y","text":"Routed by the second marker","level":3},{"id":"heading-plain","text":"A plain heading without extras","level":3}]},{"id":"image","text":"Image and figure","level":2},{"id":"cite","text":"Cite","level":2},{"id":"wiki","text":"Wiki link","level":2},{"id":"marker","text":"Marker","level":2},{"id":"table","text":"Table","level":2},{"id":"latex","text":"LaTeX and code","level":2},{"id":"literal","text":"Literal fallback","level":2}];
export default function PendonView() { return (<>
<section id="extras-head">
<HeadingDefault level={2} number={1} raw_title={"The extras head"}>The extras head
</HeadingDefault><p>The unified attribute syntax is the <strong>extras head</strong>: <code>@@type&#123;…&#125;</code>, written
adjacent to a construct with no space in between. This document exercises every
construct that reads it — anchor, heading, image/figure, cite, wiki link and the
seven table layers.
</p>
</section>
<section id="anchor">
<HeadingDefault level={2} number={2} raw_title={"Anchor"}>Anchor
</HeadingDefault><p>Every item kind of §5 in one head: id, slug, title, <code>class</code>, a prop, a typed
prop, a CSS custom property and a bare flag (here <code>#rew</code> wins over the slug and
the head title over the extras title, §6.2).
</p>
<p><AnchorAB type={"anchorA"} baz={23} class={"extra class"} foo={"bar"} href={"/docs/bar"} id={"rew"} isBar={true} isFoo={true} style={"--wix: sum"} title={"Head title"}>Anchor with every item kind</AnchorAB>
</p>
<p>The head title wins over the extras title (§6.2) and the extras value is dropped
with a warning:
</p>
<p><AnchorAB type={"anchorA"} href={"/docs"} title={"Head title"}>Head title wins</AnchorAB>
</p>
<p><code>#id</code> beats the head slug, which beats the extras slug (§6.2); classes
accumulate head first (§6.4):
</p>
<p><AnchorAB type={"anchorB"} class={"from-extras"} href={"/docs"} id={"explicit"}>Slug precedence</AnchorAB>
</p>
<p>A marker without an entry falls back to the layer default (<code>AnchorDefault</code>); an
anchor without any head stays the plain <code>&lt;a&gt;</code> element:
</p>
<p><AnchorDefault class={"hero"} href={"/docs"} type={"anchorZ"}>Unclaimed marker</AnchorDefault>
</p>
<p><AnchorDefault href={"/docs"}>No marker at all</AnchorDefault>
</p>
<p>URL modifiers are parsed before the head, and <code>rel:</code>/<code>target:</code> extras merge with
them (§7.2):
</p>
<p><AnchorAB type={"anchorB"} href={"https://example.com/bar--!;"} hreflang={"en"} rel={"noopener sponsored prefetch"} target={"_blank"}>Modifiers plus extras</AnchorAB>
</p>
</section>
<section id="heading">
<HeadingDefault level={2} number={3} raw_title={"Heading"}>Heading
</HeadingDefault><p><code>[slug]</code>, <code>(&quot;title&quot;)</code> and <code>@@type&#123;…&#125;</code> are all optional and independent; the
head&#39;s <code>[slug]</code> wins and <code>(&quot;title&quot;)</code> is a separate attribute, never the text.
</p>
<section id="heading-x">
<HeadingXY level={"3"} type={"headingX"} number={"3.1."} class={"fancy"} raw_title={"Extras-routed heading"} title={"Heading X"}>Extras-routed heading
</HeadingXY></section>
<section id="heading-y">
<HeadingXY level={"3"} type={"headingY"} number={"auto"} raw_title={"Routed by the second marker"} title={"Heading Y"}>Routed by the second marker
</HeadingXY></section>
<section id="heading-plain">
<HeadingDefault level={3} number={"3.3."} raw_title={"A plain heading without extras"}>A plain heading without extras
</HeadingDefault></section>
</section>
<section id="image">
<HeadingDefault level={2} number={4} raw_title={"Image and figure"}>Image and figure
</HeadingDefault><p><code>~?!!</code> declares a figure: the extras attach to the outermost node (<code>&lt;figure&gt;</code>),
the <code>w</code>/<code>h</code> marker stays on the inner <code>&lt;img&gt;</code> and the trailing text is the
caption.
</p>
<Figure class={"wide"} container={"figure"} id={"fig-1"} isLazy={true} style={"--rotate: 5deg;"} type={"figureX"}><AdvancedImage alt={"Alt text"} async_decoding={1} height={300} lazy={1} src={"https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp"} width={800}></AdvancedImage>A figure caption with <strong>markup</strong>.</Figure><p>A plain image uses the <code>img</code> layer alone. The <code>!</code> marker (lazy) keeps the line
from being read as a Markdown link before the plugin sees it:
</p>
<AdvancedImage alt={"Alt text"} async_decoding={1} class={"thumb"} foo={"bar"} src={"https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp"} type={"imageX"} width={800}></AdvancedImage></section>
<section id="cite">
<HeadingDefault level={2} number={5} raw_title={"Cite"}>Cite
</HeadingDefault><p>The citation head is `<AnchorDefault href={"ref"}>^^</AnchorDefault>`; extras merge into the citation node while the
cite args win (§7.3). <code>#id</code>/slug feed the <code>cite-id</code> slot.
</p>
<p>Minim <Cite reference={frontmatter.references["paper-smith"]} cite-id={"smith"} class={"paper"} index={1} note={"short"} type={"citeX"} /> esse do ut anim
proident est qui magna non elit quis eiusmod dolore.
</p>
<p>A location argument stays the construct value:
</p>
<p><Cite reference={frontmatter.references["suryana-2026"]} class={"thin"} index={2} loc={"hlm. 45"} type={"citeX"} />
</p>
</section>
<section id="wiki">
<HeadingDefault level={2} number={6} raw_title={"Wiki link"}>Wiki link
</HeadingDefault><p>Unchanged apart from the adjacent extras head; <code>href</code> is produced by the plugin
and is never overridable (§7.5).
</p>
<p><a class="link" href="/wiki/Anim_Esta_(Officia)" title="Anim Esta (Officia)" type="wikiX">Anim</a> and <a href="/wiki/Wireless" title="Wireless">Wireless</a>.
</p>
</section>
<section id="marker">
<HeadingDefault level={2} number={7} raw_title={"Marker"}>Marker
</HeadingDefault><p>The `<MarkerDefault type={"type"}></MarkerDefault>` marker (§10.1) has no HTML equivalent: its type is mandatory and
doubles as the routing key of the <code>marker</code> layer. An inline marker keeps its
paragraph, a block marker owns its line and takes that line&#39;s trailing text as
its children.
</p>
<p>Inline <MarkerDefault type={"footnote"}></MarkerDefault> and block forms:
</p>
<Bibliography cites={frontmatter.cites} references={frontmatter.references} /><p>An unclaimed type falls back to the layer default:
</p>
<MarkerDefault type={"unknownType"}>Rendered through <code>MarkerDefault</code>.</MarkerDefault></section>
<section id="table">
<HeadingDefault level={2} number={8} raw_title={"Table"}>Table
</HeadingDefault><p>The §8 declaration line carries the <code>&lt;table&gt;</code> extras and the head
(<code>[slug]</code>, <code>(&quot;title&quot;)</code>); the caption line is <code>|| extras content ||</code>; rows and
cells carry their extras at the front of the row/cell, after the last <code>|</code>.
</p>
<Table class={"striped"} id={"sales-2026"} sortable={true} title={"Laporan Penjualan 2026"} type={"tableX"}><TableCaption class={"caption-note"} type={"captionX"}>Laporan Penjualan 2026</TableCaption><TableHead><TableRowDefault><TableCellAB type={"cellA"} align={"left"} class={"v-top"} width={"200px"}>Produk</TableCellAB><TableCellDefault align={"center"} class={"v-top"}>Stok</TableCellDefault><TableCellDefault align={"right"} class={"v-bottom"} width={"30%"}>Harga</TableCellDefault><TableCellDefault align={"center"}>Status</TableCellDefault></TableRowDefault></TableHead><TableBody><TableRowDefault><TableCellAB type={"cellA"} align={"left"} class={"v-top"} width={"200px"}>@@cellB&#123;.lead&#125; Laptop Pro</TableCellAB><TableCellDefault align={"center"} class={"v-top"}>15</TableCellDefault><TableCellDefault align={"right"} class={"v-bottom"} width={"30%"}>15.000.000</TableCellDefault><TableCellDefault align={"center"}><AnchorDefault href={"/docs"}>Tersedia</AnchorDefault></TableCellDefault></TableRowDefault><TableRowAB type={"rowB"} class={"row-info"}><TableCellAB type={"cellA"} align={"left"} class={"v-top"} width={"200px"}>Mouse Wireless</TableCellAB><TableCellDefault align={"center"} class={"v-top"}>@@cellB&#123;&#125; &gt;</TableCellDefault><TableCellDefault align={"right"} class={"v-bottom"} width={"30%"}>250.000</TableCellDefault><TableCellDefault align={"center"}>Tersedia</TableCellDefault></TableRowAB><TableRowDefault class={"row-danger"}><TableCellAB type={"cellA"} align={"left"} class={"v-top"} width={"200px"}>Keyboard Mekanikal</TableCellAB><TableCellDefault align={"center"} class={"v-top"}>0</TableCellDefault><TableCellDefault align={"right"} class={"v-bottom"} width={"30%"}>850.000</TableCellDefault><TableCellDefault align={"center"}>Habis</TableCellDefault></TableRowDefault></TableBody><TableFoot class={"total"} type={"tfootX"}><TableRowDefault><TableCellAB type={"cellA"} align={"left"} class={"v-top"} colspan={2} width={"200px"}>Total Inventaris</TableCellAB><TableCellDefault align={"right"} class={"v-bottom"} width={"30%"}>21.300.000</TableCellDefault><TableCellDefault align={"center"}>-</TableCellDefault></TableRowDefault></TableFoot></Table><p>A table without a declaration line keeps working, and its layers fall back to
the built-in elements:
</p>
<Table><TableHead><TableRowDefault><TableCellDefault>Kolom A</TableCellDefault><TableCellDefault>Kolom B</TableCellDefault></TableRowDefault></TableHead><TableBody><TableRowDefault><TableCellDefault>satu</TableCellDefault><TableCellDefault>dua</TableCellDefault></TableRowDefault></TableBody><TableFoot><TableRowDefault><TableCellDefault>total</TableCellDefault><TableCellDefault>dua</TableCellDefault></TableRowDefault></TableFoot></Table></section>
<section id="latex">
<HeadingDefault level={2} number={9} raw_title={"LaTeX and code"}>LaTeX and code
</HeadingDefault><p>Inline <code>$H \rightarrow O$</code> and a display block:
</p>
<p><span class="latex latex-block" style="display: block;" innerHTML={`<span class="katex-display"><span class="katex"><span class="katex-mathml"><math xmlns="http://www.w3.org/1998/Math/MathML" display="block"><semantics><mrow><mo stretchy="false">(</mo><mo stretchy="false">(</mo><mi>H</mi><mo>→</mo><mi>O</mi><mo stretchy="false">)</mo><mo>∧</mo><mi mathvariant="normal">¬</mi><mi>O</mi><mo stretchy="false">)</mo><mo>→</mo><mi mathvariant="normal">¬</mi><mi>H</mi></mrow><annotation encoding="application/x-tex">((H \\rightarrow O) \\land \\lnot O) \\rightarrow \\lnot H</annotation></semantics></math></span><span class="katex-html" aria-hidden="true"><span class="base"><span class="strut" style="height:1em;vertical-align:-0.25em;"></span><span class="mopen">((</span><span class="mord mathnormal" style="margin-right:0.08125em;">H</span><span class="mspace" style="margin-right:0.2778em;"></span><span class="mrel">→</span><span class="mspace" style="margin-right:0.2778em;"></span></span><span class="base"><span class="strut" style="height:1em;vertical-align:-0.25em;"></span><span class="mord mathnormal" style="margin-right:0.02778em;">O</span><span class="mclose">)</span><span class="mspace" style="margin-right:0.2222em;"></span><span class="mbin">∧</span><span class="mspace" style="margin-right:0.2222em;"></span></span><span class="base"><span class="strut" style="height:1em;vertical-align:-0.25em;"></span><span class="mord">¬</span><span class="mord mathnormal" style="margin-right:0.02778em;">O</span><span class="mclose">)</span><span class="mspace" style="margin-right:0.2778em;"></span><span class="mrel">→</span><span class="mspace" style="margin-right:0.2778em;"></span></span><span class="base"><span class="strut" style="height:0.6833em;"></span><span class="mord">¬</span><span class="mord mathnormal" style="margin-right:0.08125em;">H</span></span></span></span></span>`}></span>

</p>
<pre lang="rust"><code innerHTML={"<p>fn extras(head: <b>&amp;</b>str) -&gt; Option&lt;Head&gt; {</p><p>    parse_extras(head).ok()</p><p>}</p>"} /></pre>
</section>
<section id="literal">
<HeadingDefault level={2} number={10} raw_title={"Literal fallback"}>Literal fallback
</HeadingDefault><p>A malformed head never aborts the build: it stays literal text (§4.3).
</p>
<p><AnchorDefault href={"/docs"}>Unterminated</AnchorDefault>@@anchorX&#123;foo: &quot;bar&quot; and the rest of the line is text.
</p>
<p>A head separated by whitespace is not a head either: <AnchorDefault href={"/docs"}>Spaced</AnchorDefault> @@anchorA&#123;.x&#125;
</p>
</section>

</>); }
