import Table from '@comp/content/Table'
import { AnchorAll } from '@comp/shared/Anchor'
import { TableCellA } from '@comp/content/Table'

export const frontmatter = {"references":{"doe-2020":{"authors":[{"firstName":"Jane","lastName":"Doe"}],"id":"doe-2020","issuedDate":{"year":2020},"language":"en","publisher":"Test Press","publisherLocation":"Bandung","title":"Edge Cases in Practice","type":"book"},"paper-smith":{"authors":[{"firstName":"John","lastName":"Smith"}],"containerTitle":"Journal of Web Engineering","doi":"10.1016/j.jwe.2025.08.001","id":"paper-smith","issue":"4","issuedDate":{"month":8,"year":2025},"language":"en","pages":"210-225","title":"Generative MDX to PDF Compilation Architectures","type":"journal","volume":"18"},"suryana-2026":{"authors":[{"firstName":"Eko","lastName":"Suryana"}],"id":"suryana-2026","isbn":"978-602-0000-00-0","issuedDate":{"year":2026},"language":"id","publisher":"TechPress Indonesia","publisherLocation":"Jakarta","title":"Masa Depan Rekayasa Perangkat Lunak","type":"book"}},"title":"Unified Syntax Torture Demo"};
export default function PendonView() { return (<>
<p>&lt;!--
Legend: setiap kasus berlabel (A=anchor H=heading I=image C=cite W=wiki M=marker
T=table Q=blockquote L=list D=inline directive B=block directive N=nesting E=edge).
Kanonik: &#123;...&#125; tanpa @@. @@type hanya untuk routing. @@&#123;...&#125; = non-kanonik, label berakhiran n.
@@type dan &#123;...&#125; harus menempel; dipisah spasi = literal. Type = alfanumerik saja.
</p>
<table>
<thead>
<tr>
<th>Sel tabel: head menempel ke</th>
<th>; sel alignment: head setelah kode alignment. &gt; = colspan, ^ = rowspan.</th>
</tr>
</thead>
<tbody>
<tr>
<td>Varian extras: none</td>
<td>empty</td>
<td>type-only</td>
<td>X</td>
<td>XY</td>
<td>XXYZZ</td>
<td>all</td>
<td>shuffled</td>
<td>typed</td>
<td>untyped</td>
<td>unclaimed.</td>
</tr>
</tbody>
</table>
<p>KS (typed) / KSu (untyped) = anchor + bold + italic + code + wiki + cite + marker + directive + math.
Label pensiun: H22, E44 (setext tidak didukung).
--&gt;
</p>
<h1 id="s-top">1. Unified Demo</h1>
<h2 id="s-anchor">1.1. Anchor</h2>
<p>A01 <a href="/a">plain</a>
</p>
<p>A02 <a href="/a" title="T">title</a>
</p>
<p>A03 <a href="/a">empty title</a>
</p>
<p>A04 <a href="/a">t</a>
</p>
<p>A04n <a href="/a">t</a>
</p>
<p>A05 <AnchorAll href={"/a"} type={"anchorA"}>t</AnchorAll>
</p>
<p>A06 <AnchorAll href={"/a"} type={"anchorA"}>t</AnchorAll>
</p>
<p>A07 <a class="x" href="/a">t</a>
</p>
<p>A07n <a class="x" href="/a">t</a>
</p>
<p>A08 <AnchorAll class={"x"} href={"/a"} type={"anchorA"}>t</AnchorAll>
</p>
<p>A09 <AnchorAll class={"x"} href={"/a"} id={"i"} type={"anchorB"}>t</AnchorAll>
</p>
<p>A10 <AnchorAll class={"a b"} href={"/a"} id={"i"} k={"v"} n={2} type={"anchorC"}>t</AnchorAll>
</p>
<p>A11 <AnchorAll href={"/a"} id={"s-only"} type={"anchorA"}>t</AnchorAll>
</p>
<p>A12 <AnchorAll href={"/a"} title={"Extras title"} type={"anchorA"}>t</AnchorAll>
</p>
<p>A13 <AnchorAll href={"/a"} title={"Head title"} type={"anchorA"}>t</AnchorAll>
</p>
<p>A14 <AnchorAll href={"/a"} k={"v"} type={"anchorA"}>t</AnchorAll>
</p>
<p>A15 <AnchorAll href={"/a"} n={12} type={"anchorA"}>t</AnchorAll><AnchorAll href={"/a"} n={-1.5} type={"anchorA"}>t</AnchorAll>
</p>
<p>A16 <AnchorAll b={true} href={"/a"} type={"anchorA"}>t</AnchorAll><AnchorAll b={false} href={"/a"} type={"anchorA"}>t</AnchorAll>
</p>
<p>A17 <AnchorAll href={"/a"} style={"--v: 2rem"} type={"anchorA"}>t</AnchorAll>
</p>
<p>A18 <AnchorAll href={"/a"} isBar={true} type={"anchorA"}>t</AnchorAll>
</p>
<p>A19 <AnchorAll bar={12} class={"extra class"} foo={"bar"} href={"/a"} id={"id"} isBar={true} isFoo={true} style={"--style-var: 2rem"} title={"Head"} type={"anchorA"}>t</AnchorAll>
</p>
<p>A20 <AnchorAll bar={12} class={"extra class"} foo={"bar"} href={"/a"} id={"id"} isBar={true} isFoo={true} style={"--style-var: 2rem"} title={"Title Foo"} type={"anchorB"}>t</AnchorAll>
</p>
<p>A21 <AnchorAll class={"from-extras"} href={"/a"} id={"explicit"} type={"anchorB"}>t</AnchorAll>
</p>
<p>A22 <AnchorAll class={"hero"} href={"/a"} type={"anchorZ"}>t</AnchorAll>
</p>
<p>A23 <AnchorAll class={"u"} href={"https://example.com/a?q=1&r=2#h"} rel={"noopener"} target={"_blank"} type={"anchorA"}>abs</AnchorAll>
</p>
<p>A24 <a class="u" href="./a/b.md" rel="noopener" target="_blank">rel</a>
</p>
<p>A24n <a class="u" href="./a/b.md" rel="noopener" target="_blank">rel</a>
</p>
<p>A25 <a class="u" href="#s-anchor">hash</a>
</p>
<p>A26 <a class="u" href="mailto:a@b.co" rel="noopener" target="_blank">mail</a>
</p>
<p>A27 <AnchorAll href={"https://example.com/bar"} hreflang={"en"} rel={"noopener noreferrer nofollow sponsored prefetch"} target={"_self"} title={"T"} type={"anchorB"}>mod</AnchorAll>
</p>
<p>A28 <a href="https://example.com/bar--!;" rel="noopener sponsored" target="_blank">mod2</a>
</p>
<p>A29 <a class="u" href="/a_(b)">paren</a>
</p>
<p>A30 <AnchorAll class={"m"} href={"/a"} type={"anchorA"}><strong>b</strong> <i>i</i> <code>c</code> $m$</AnchorAll>
</p>
<p>A31 [a \[b\] c](/a)&#123;.e&#125;
</p>
<p>A32 <a href="/a" k="a, b&#125; c" q="x&quot;y" u="日本 😀">t</a>
</p>
<p>A33 <a class="a" href="/1">a</a><a class="b" href="/2">b</a><a href="/3">c</a>
</p>
<p>A34 (<a class="p" href="/a">t</a>), <a class="p" href="/a">t</a>. <em></em><a class="p" href="/a">t</a><em></em> _<a class="p" href="/a">t</a>_
</p>
<p>A35 <a class="o" href="/a">&#123;&#123;footnote&#125;&#125;&#123;.k&#125; in anchor</a><a class="o" href="/a">::asideA d::</a>
</p>
<p>A36 <a class="a b" href="/a" id="i" k="v" n="2">t</a>
</p>
<p>A36n <a class="a b" href="/a" id="i" k="v" n="2">t</a>
</p>
<p>A37 <a bar="12" class="extra class" foo="bar" href="/a" id="id" isBar isFoo="true" style="--style-var: 2rem" title="Head">t</a>
</p>
<p>A37n <a bar="12" class="extra class" foo="bar" href="/a" id="id" isBar isFoo="true" style="--style-var: 2rem" title="Head">t</a>
</p>
<p>A38 <a bar="12" class="extra class" foo="bar" href="/a" id="id" isBar isFoo="true" style="--style-var: 2rem" title="Title Foo">t</a>
</p>
<p>A39 <a class="x" href="/a" id="i">t</a>
</p>
<p>A40 <AnchorAll href={"/a"} type={"anchorZ"}>t</AnchorAll>
</p>
<p>A41 <a href="/a" id="s-only">t</a>
</p>
<p>A42 <a href="/a" title="Extras title">t</a>
</p>
<p>A43 <a href="/a" k="v">t</a>
</p>
<p>A44 <a href="/a" style="--v: 2rem">t</a>
</p>
<p>A45 <a href="/a" isBar>t</a>
</p>
<p>A46 <a href="/a" title="Head title">t</a>
</p>
<p>A47 <a class="from-extras" href="/a" id="explicit">t</a>
</p>
<p>A48 <a href="https://example.com/bar--!;" rel="noopener sponsored prefetch" target="_blank">mod3</a>
</p>
<p>A49 <AnchorAll href={"/a"} type={"anchorA"}>t</AnchorAll>.
</p>
<p>A50 (<AnchorAll href={"/a"} type={"anchorA"}>t</AnchorAll>)
</p>
<p>A51 <AnchorAll href={"/a"} type={"anchorA"}>t</AnchorAll>, <AnchorAll href={"/b"} type={"anchorB"}>u</AnchorAll>; <AnchorAll href={"/c"} type={"anchorA"}>v</AnchorAll>!
</p>
<p>A52 <a class="x" href="/a" type="anchor1">t</a>
</p>
<h2 id="s-heading">1.2. Heading</h2>
<h3 id="h01">1.2.1. H01 slug</h3>
<h3 title="T02">1.2.2. H02 title</h3>
<h3 class="x" type="headingX">1.2.3. H03 type+X</h3>
<h3>1.2.4. H04 plain</h3>
<h3 id="h05" title="T05">1.2.5. H05 slug+title</h3>
<h3 class="x" id="h06" type="headingX">1.2.6. H06 slug+X</h3>
<h3 class="x" id="i" title="T07" type="headingY">1.2.7. H07 title+XY</h3>
<h3 class="a b" id="i" k="v" n="2" title="T08" type="headingX">1.2.8. H08 XXYZZ</h3>
<h3 bar="12" class="extra class" foo="bar" id="id" isBar isFoo="true" style="--style-var: 2rem" title="T09" type="headingX">1.2.9. H09 all</h3>
<h3 class="x">1.2.10. H10 untyped</h3>
<h3 class="x">1.2.11. H10n untyped non-canonical</h3>
<h3>1.2.12. H11 empty</h3>
<h3>1.2.13. H11n empty non-canonical</h3>
<h3 class="x" type="headingZ">1.2.14. H12 unclaimed</h3>
<h3 type="headingX">1.2.15. H13 type only</h3>
<h3 class="x" id="h14" title="T14">1.2.16. H14 precedence</h3>
<h3 class="x" id="h14n" title="T14n">1.2.17. H14n precedence non-canonical</h3>
<h1 class="l1" id="h15" type="headingX">2. H15 level 1</h1>
<h2 class="l2">2.1. H16 level 2</h2>
<h2 class="l2n">2.2. H16n level 2 non-canonical</h2>
<h4 class="l4" id="h17" type="headingY">2.2.0.1. H17 level 4</h4>
<h6 class="l6" title="T18">2.2.0.1.0.1. H18 level 6 (skips 5)</h6>
<h6 class="l6n" title="T18n">2.2.0.1.0.2. H18n level 6 non-canonical</h6>
<h3 class="k" id="h19" title="T19" type="headingX">2.2.1. H19 KS <AnchorAll class={"k"} href={"/x"} title={"T"} type={"anchorA"}>a</AnchorAll> <strong>b</strong> <i>i</i> <code>c</code> [[W]]@@wikiX&#123;.k&#125; <a class="k" href="paper-smith" type="citeX">^^</a> &#123;&#123;footnote&#125;&#125;&#123;.k&#125; ::asideA&#123;.k&#125; d:: $x$</h3>
<p>&lt;!-- H20: heading with empty text, typed --&gt;
</p>
<h3 class="x" type="headingX">2.2.2. 
</h3>
<h3 id="h21">2.2.3. H21 closing hashes ###</h3>
<p>&lt;!-- H23: heading with empty text, untyped --&gt;
</p>
<h3 class="x">2.2.4. 
</h3>
<h3 class="k" id="h24">2.2.5. H24 KSu <a class="k" href="/x" title="T">a</a> <strong>b</strong> <i>i</i> <code>c</code> [[W]]&#123;.k&#125; <a class="k" href="paper-smith">^^</a> &#123;&#123;footnote&#125;&#125;&#123;.k&#125; ::asideA&#123;.k&#125; d:: $x$</h3>
<h3 class="a" id="i" k="v" title="T25">2.2.6. H25 slug+title+XXYZ untyped</h3>
<h3 class="a" title="T26">2.2.7. H26 title+X untyped</h3>
<h3 class="a" id="h27">2.2.8. H27 slug+X untyped, slug precedence</h3>
<h2 id="s-image">2.3. Image and figure</h2>
<p>~!w800<a href="/i.png">I01</a>
</p>
<p>~!w800<a href="/i.png">I02</a> Caption only
</p>
<p>~!h300<a href="/i.png">I03</a>
</p>
<p>~!h300<a href="/i.png">I03n</a>
</p>
<p>~!h300w800<a class="thumb" href="/i.png" type="imageX">I04</a>
</p>
<p>~?!w800<a class="x" href="/i.png" id="i" type="imageX">I05</a> Caption <strong>b</strong>
</p>
<p>~?!h300w800<a bar="12" class="extra class" foo="bar" href="/i.png" id="id" isBar isFoo="true" style="--style-var: 2rem" title="Title Foo" type="thumb">I06</a> Caption all
</p>
<p>~?!!w800<a href="/i.png">I07</a>
</p>
<p>~?!!h300w800<a href="/i.png">I08</a> Figure caption only
</p>
<p>~?!!h300w800<a class="wide" href="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp" id="fig-1" isLazy rel="noopener" style="--rotate: 5deg" target="_blank" type="figureX">I09</a> Caption <strong>b</strong>
</p>
<p>~?!!w800<a href="/i.png" id="s-i10">I10</a>
</p>
<p>~?!!w800<a href="/i.png" id="s-i10n">I10n</a>
</p>
<p>~?!!<a href="/i.png">I11</a>@@figureX&#123;&#125;
</p>
<p>&lt;!-- I12: figure with empty alt, untyped --&gt;
</p>
<p>~?!!w800<a class="empty-alt" href="/i.png"></a>
</p>
<p>~?!!h300<a class="x" href="/i.png" type="figureZ">I13</a> unclaimed
</p>
<p>~!w100<a class="a b" href="/i.png" id="i" k="v" n="2" title="Head T" type="imageX">I14</a>
</p>
<p>~?!!w100<a href="/i.png" isA type="figureX">I15</a> Caption KS <AnchorAll class={"k"} href={"/x"} type={"anchorA"}>a</AnchorAll><a class="k" href="paper-smith" type="citeX">^^</a> [[W]]@@wikiX&#123;.k&#125; &#123;&#123;footnote&#125;&#125;&#123;.k&#125; ::asideA&#123;.k&#125; d:: $x$ <strong>b</strong>
</p>
<p><img alt="I16 plain markdown image" src="/i.png" />
</p>
<p>~!h300w800<a class="thumb" href="/i.png">I17</a>
</p>
<p>~?!h300w800<a bar="12" class="extra class" foo="bar" href="/i.png" id="id" isBar isFoo="true" style="--style-var: 2rem" title="Title Foo">I18</a> Caption all
</p>
<p>~?!h300w800<a bar="12" class="extra class" foo="bar" href="/i.png" id="id" isBar isFoo="true" style="--style-var: 2rem" title="Title Foo">I18n</a> Caption all
</p>
<p>~?!!h300w800<a class="wide" href="/i.png" id="fig-2" isLazy style="--rotate: 5deg">I19</a> Caption
</p>
<p>~?!!<a href="/i.png">I20</a>&#123;&#125;
</p>
<p>~?!!<a href="/i.png">I21</a>@@figureX
</p>
<p>~!w100<a class="a" href="/i.png" title="Head T">I22</a>
</p>
<p>~?!!w100<a class="f" href="/i.png">I23</a> Caption KSu <a class="k" href="/x">a</a><a class="k" href="paper-smith">^^</a> [[W]]&#123;.k&#125; &#123;&#123;footnote&#125;&#125;&#123;.k&#125; ::asideA&#123;.k&#125; d:: $x$ <strong>b</strong>
</p>
<h2 id="s-cite">2.4. Cite</h2>
<p>&lt;!-- Cite form: <a href="id" title="loc">^^</a>. The old <a href="" title="id&quot;, &quot;loc">^^</a> is obsolete. --&gt;
</p>
<p>C01 Minim <a href="paper-smith">^^</a> esse.
</p>
<p>C02 Minim <a href="paper-smith" title="hlm. 55">^^</a> esse.
</p>
<p>C03 Minim <a href="paper-smith">^^</a> esse.
</p>
<p>C03n Minim <a href="paper-smith">^^</a> esse.
</p>
<p>C04 Minim <a href="paper-smith" type="citeX">^^</a> esse.
</p>
<p>C05 Minim <a class="paper" href="paper-smith" type="citeX">^^</a> esse.
</p>
<p>C06 Minim <a class="paper" href="paper-smith" id="smith" type="citeX">^^</a> esse.
</p>
<p>C07 Minim <a class="a b" href="suryana-2026" id="i" n="2" note="short" title="hlm. 45" type="citeX">^^</a> esse.
</p>
<p>C08 Minim <a bar="12" class="extra class" foo="bar" href="doe-2020" id="id" isBar isFoo="true" style="--style-var: 2rem" title="Title Foo" type="citeX">^^</a> esse.
</p>
<p>C09 <a class="thin" href="paper-smith" loc="dropped" title="hlm. 1" type="citeX">^^</a>
</p>
<p>C10 <a class="x" href="missing-ref" type="citeX">^^</a>
</p>
<p>C11 <a href="paper-smith">^^</a><a class="y" href="suryana-2026">^^</a><a href="doe-2020">^^</a>
</p>
<p>C12 <a href="paper-smith">^^</a> dan lagi <a class="again" href="paper-smith" type="citeX">^^</a>
</p>
<p>C13 <a href="">^^</a>
</p>
<p>C14 <a class="x" href="paper-smith" type="citeZ">^^</a> unclaimed
</p>
<p>C15 <a href="paper-smith" type="citeX">^^</a>
</p>
<p>C16 Minim <a class="paper" href="paper-smith">^^</a> esse.
</p>
<p>C17 Minim <a class="paper" href="paper-smith" id="smith2" title="hlm. 7">^^</a> esse.
</p>
<p>C18 Minim <a class="a b" href="suryana-2026" id="i2" n="2" note="short" title="hlm. 45">^^</a> esse.
</p>
<p>C19 Minim <a bar="12" class="extra class" foo="bar" href="doe-2020" id="id" isBar isFoo="true" style="--style-var: 2rem" title="Title Foo">^^</a> esse.
</p>
<p>C19n Minim <a bar="12" class="extra class" foo="bar" href="doe-2020" id="id" isBar isFoo="true" style="--style-var: 2rem" title="Title Foo">^^</a> esse.
</p>
<p>&lt;!-- C20-C22: retired cite forms. Expected: plain literal text, no warning. --&gt;
</p>
<p>C20 <a href="paper-smith loc=" title="p.1">^^</a>
</p>
<p>C21 <a href="" title="paper-smith">^^</a>
</p>
<p>C22 <a href="" title="paper-smith&quot;, &quot;hlm. 55">^^</a>
</p>
<p>C23 Minim <a href="paper-smith" type="citeX">^^</a>. esse
</p>
<h2 id="s-wiki">2.5. Wiki</h2>
<p>W01 [[Wireless]]
</p>
<p>W02 [[Anim Esta (Officia) | Anim]]
</p>
<p>W03 [[Wireless]]&#123;&#125;
</p>
<p>W03n [[Wireless]]@@&#123;&#125;
</p>
<p>W04 [[Wireless]]@@wikiX&#123;.link&#125;
</p>
<p>W05 [[Wireless]]@@wikiX&#123;.a, #i&#125;
</p>
<p>W06 [[Wireless]]@@wikiX&#123;.a, .b, #i, k: &quot;v&quot;, n: 2&#125;
</p>
<p>W07 [[Anim Esta]]@@wikiX&#123;<code>slug-foo</code>, &quot;Title Foo&quot;, .extra, .class, #id, foo: &quot;bar&quot;, bar: 12, isFoo: true, --style-var: &quot;2rem&quot;, isBar&#125;
</p>
<p>W08 [[Wireless]]@@wikiX&#123;href: &quot;/evil&quot;, title: &quot;dropped&quot;&#125;
</p>
<p>W09 [[Café Ünïcode 日本]]&#123;.u&#125;
</p>
<p>W09n [[Café Ünïcode 日本]]@@&#123;.u&#125;
</p>
<p>W10 [[a]]&#123;.a&#125;[[b]]&#123;.b&#125;[[c]]
</p>
<p>W11 [[ ]] [[a|b|c]] [[]]
</p>
<p>W12 [[Wireless]] @@wikiX&#123;.spaced&#125;
</p>
<p>W13 [[Wireless]]@@wikiZ&#123;.x&#125;
</p>
<p>W14 [[Wireless]]@@wikiX &#123;.x&#125; type and brace separated by space
</p>
<p>W15 [[Wireless]] &#123;.x&#125; bare brace separated by space
</p>
<p>W16 [[Wireless]]&#123;.link&#125;
</p>
<p>W17 [[Wireless]]&#123;.a, #i&#125;
</p>
<p>W18 [[Wireless]]&#123;.a, .b, #i, k: &quot;v&quot;, n: 2&#125;
</p>
<p>W19 [[Anim Esta]]&#123;<code>slug-foo</code>, &quot;Title Foo&quot;, .extra, .class, #id, foo: &quot;bar&quot;, bar: 12, isFoo: true, --style-var: &quot;2rem&quot;, isBar&#125;
</p>
<p>W19n [[Anim Esta]]@@&#123;<code>slug-foo</code>, &quot;Title Foo&quot;, .extra, .class, #id, foo: &quot;bar&quot;, bar: 12, isFoo: true, --style-var: &quot;2rem&quot;, isBar&#125;
</p>
<p>W20 [[Wireless]]&#123;href: &quot;/evil&quot;, title: &quot;dropped&quot;&#125;
</p>
<p>W21 [[Wireless]]@@wikiX.
</p>
<h2 id="s-marker">2.6. Marker</h2>
<p>M01 inline &#123;&#123;footnote&#125;&#125; ipsum.
</p>
<p>M02 inline &#123;&#123;footnote&#125;&#125;&#123;.x&#125; ipsum.
</p>
<p>M03 inline &#123;&#123;footnote&#125;&#125;&#123;<code>s</code>, .a, #i&#125; ipsum.
</p>
<p>M04 inline &#123;&#123;footnote&#125;&#125;&#123;.a, .b, #i, k: &quot;v&quot;, n: 2&#125; ipsum.
</p>
<p>M05 inline &#123;&#123;markerA&#125;&#125;&#123;<code>slug-foo</code>, &quot;Title Foo&quot;, .extra, .class, #id, foo: &quot;bar&quot;, bar: 12, isFoo: true, --style-var: &quot;2rem&quot;, isBar&#125; ipsum.
</p>
<p>M06 &#123;&#123;footnote&#125;&#125;&#123;&#125;
</p>
<p>&#123;&#123;bibliography&#125;&#125;
</p>
<p>&#123;&#123;bibliography&#125;&#125;&#123;.x&#125;
</p>
<p>&#123;&#123;markerA&#125;&#125; trailing text as children
</p>
<p>&#123;&#123;markerA&#125;&#125;&#123;.x, #i&#125; trailing <strong>text</strong> <AnchorAll class={"k"} href={"/x"} type={"anchorA"}>a</AnchorAll>
</p>
<p>&#123;&#123;markerA&#125;&#125;&#123;<code>slug-foo</code>, &quot;Title Foo&quot;, .extra, .class, #id, foo: &quot;bar&quot;, bar: 12, isFoo: true, --style-var: &quot;2rem&quot;, isBar&#125;
</p>
<p>&#123;&#123;unknownType&#125;&#125; unclaimed falls to default
</p>
<p>&#123;&#123;unknownType&#125;&#125;&#123;.u&#125;
</p>
<p>M10 &#123;&#123;footnote&#125;&#125;&#123;.a&#125; dan &#123;&#123;footnote&#125;&#125; dan &#123;&#123;markerA&#125;&#125;&#123;.b&#125;&#123;&#123;markerA&#125;&#125;
</p>
<p>M11 &#123;&#123;&#125;&#125; &#123;&#123;bad-type&#125;&#125; &#123;&#123;bad_type&#125;&#125; &#123;&#123;type1&#125;&#125; &#123;&#123; spaced &#125;&#125; &#123;&#123;footnote&#125;&#125; &#123;.spaced&#125;
</p>
<p>M12 &#123;&#123;Footnote&#125;&#125;&#123;.upper&#125; &#123;&#123;FOOTNOTE&#125;&#125;
</p>
<p>&lt;!-- §6.1/§10.1 groups: a bracket group directly after <code>&#125;&#125;</code> fills the marker&#39;s
bracket<i>key (slug) and a parentheses group fills parentheses</i>key (title). A bracket
group immediately followed by a parentheses group is also an anchor head, and this task
lists <code>anchor</code> before <code>marker</code>, so the anchor layer claims such a pair first (§6.1 layer
order) and the marker renders bare. --&gt;
</p>
<p>M13 &#123;&#123;markerA&#125;&#125;[m13] bracket group, trailing text as children
</p>
<p>M14 &#123;&#123;markerA&#125;&#125;(&quot;M14&quot;) parentheses group, trailing text as children
</p>
<p>M15 &#123;&#123;markerA&#125;&#125;[m15]@@asideA&#123;.h&#125; group then extras head
</p>
<p>M16 &#123;&#123;markerA&#125;&#125;[m16( malformed group stays literal, marker still renders
</p>
<p>M17 &#123;&#123;markerA&#125;&#125;<a href="" title="M17">m17</a> anchor layer wins the pair, marker stays bare
</p>
<p>M18 &#123;&#123;unknownType&#125;&#125;[m18] unclaimed type keeps the group
</p>
<p>M19 inline &#123;&#123;footnote&#125;&#125;[m19] and &#123;&#123;markerA&#125;&#125;(&quot;M19&quot;) ipsum.
</p>
<h2 id="s-table">2.7. Table</h2>
<p>&lt;!--
</p>
<table>
<thead>
<tr>
<th>Placement: heads touch the cell&#39;s opening</th>
<th>; alignment cells take the head AFTER the</th>
</tr>
</thead>
<tbody>
<tr>
<td>alignment code ; trailing heads touch the last</td>
<td>. A cell holding only &gt; is merged into the</td>
</tr>
</tbody>
</table>
<p>cell on its right (colspan) ; only ^ is merged into the cell above (rowspan).
--&gt;
</p>
<Table class={"striped wide"} cols={4} id={"tbl"} isDense={true} sortable={true} style={"--gap: 2rem"} title={"Title Foo"} type={"tableX"}><caption class="note" id="cap" type="captionX">T01 caption <strong>b</strong></caption><thead><tr><TableCellA align={"left"} class={"v-top"} type={"cellA"} width={"200px"}>&#123;.h1&#125; H1</TableCellA><th class="c" style="text-align: center;">@@cellA&#123;.h2&#125; H2</th><th style="text-align: right; width: 30%;">H3</th><th class="a" id="i" k="v" n="2" style="text-align: left;" type="cellB">H4</th></tr></thead><tbody class="tb" type="tbodyX"><tr><td class="v-top lead" style="text-align: left; width: 200px;" type="cellB">X</td><td class="c" style="text-align: center;">15</td><td style="text-align: right; width: 30%;"><AnchorAll class={"u"} href={"/docs"} type={"anchorA"}>Ada</AnchorAll></td><td class="a" id="i" k="v" n="2" style="text-align: left;" type="cellB">[[W]]@@wikiX&#123;.w&#125;</td></tr><tr class="row-info" type="rowB"><TableCellA align={"left"} class={"v-top"} colspan={2} type={"cellA"} width={"200px"}>plain</TableCellA><td style="text-align: right; width: 30%;"><a class="c" href="paper-smith" type="citeX">^^</a></td><td class="a" id="i" k="v" n="2" style="text-align: left;" type="cellB">&#123;&#123;footnote&#125;&#125;&#123;.m&#125;</td></tr><tr type="rowA"><TableCellA align={"left"} class={"v-top"} type={"cellA"} width={"200px"}>empty-extras</TableCellA><td class="c" style="text-align: center;">::asideA&#123;.k&#125; d::</td><td style="text-align: right; width: 30%;">$x^2$</td><td class="a" id="i" k="v" n="2" style="text-align: left;" type="cellB"><code>c</code></td></tr><tr class="untyped-row"><TableCellA align={"left"} class={"v-top"} type={"cellA"} width={"200px"}>a</TableCellA><td class="c" style="text-align: center;">b</td><td style="text-align: right; width: 30%;">c</td><td class="a" id="i" k="v" n="2" style="text-align: left;" type="cellB">d</td></tr><tr class="a" id="r" k="v" n="2" type="rowB"><TableCellA align={"left"} class={"v-top"} type={"cellA"} width={"200px"}>\</TableCellA><td class="c" style="text-align: center;">escaped</td><td style="text-align: right; width: 30%;"></td><td class="a" id="i" k="v" n="2" style="text-align: left;" type="cellB"></td></tr><tr bar="12" class="extra class" foo="bar" id="id" isBar isFoo="true" style="--style-var: 2rem" title="Title Foo" type="rowB"><TableCellA align={"left"} class={"v-top"} type={"cellA"} width={"200px"}>all-row</TableCellA><td class="c" style="text-align: center;">x</td><td style="text-align: right; width: 30%;">y</td><td class="a" id="i" k="v" n="2" style="text-align: left;" type="cellB">z</td></tr><tr><TableCellA align={"left"} class={"v-top only-extras"} type={"cellA"} width={"200px"}></TableCellA><td class="c" style="text-align: center;"><strong>b</strong> <i>i</i></td><td style="text-align: right; width: 30%;"><AnchorAll class={"k"} href={"/x"} title={"T"} type={"anchorA"}>a</AnchorAll><a class="k" href="paper-smith" type="citeX">^^</a></td><td b="true" class="a a" flag id="i" k="v" n="1" style="text-align: left;" title="T" type="cellB">KS</td></tr></tbody><tfoot class="total" type="tfootX"><tr class="f" type="rowA"><TableCellA align={"left"} class={"v-top"} colspan={2} type={"cellA"} width={"200px"}>Total</TableCellA><td class="a" id="i" k="v" n="2" style="text-align: left;" type="cellB">1</td></tr><tr><TableCellA align={"left"} class={"v-top t"} type={"cellA"} width={"200px"}>F2</TableCellA><td class="c" style="text-align: center;">x</td><td style="text-align: right; width: 30%;">y</td><td class="a" id="i" k="v" n="2" style="text-align: left;" type="cellB">z</td></tr></tfoot></Table><p>T01n non-canonical @@&#123;...&#125; on every table slot:
</p>
<Table class={"x"} id={"t01n"}><caption class="c">T01n caption</caption><thead><tr><th class="v" style="text-align: left;">A</th><th>B</th></tr></thead><tbody class="tb"><tr class="r"><td class="v l" style="text-align: left;">x</td><td>y</td></tr></tbody><tfoot class="f"><tr><td class="v" style="text-align: left;">t</td><td>u</td></tr></tfoot></Table><p>T02 plain, no declaration, no caption, no extras:
</p>
<Table><thead><tr><th>A</th><th>B</th></tr></thead><tbody><tr><td>1</td><td>2</td></tr><tr><td>3</td><td>4</td></tr><tr><td>===</td><td></td></tr><tr><td>total</td><td>6</td></tr></tbody></Table><p>|| T03 caption only ||
</p>
<Table><thead><tr><th>A</th><th>B</th></tr></thead><tbody><tr><td>1</td><td>2</td></tr></tbody></Table><p>|- -|
</p>
<Table><thead><tr><th>T04 A</th><th>B</th></tr></thead><tbody><tr><td>1</td><td>2</td></tr></tbody></Table><p>|-[t05]-|
</p>
<Table><thead><tr><th>T05 A</th><th>B</th></tr></thead><tbody><tr><td>1</td><td>2</td></tr></tbody></Table><p>|-(&quot;Table 06&quot;)-|
</p>
<Table><thead><tr><th>T06 A</th><th>B</th></tr></thead><tbody><tr><td>1</td><td>2</td></tr></tbody></Table><p>|-&#123;.x&#125;-|
</p>
<Table><thead><tr><th>T07 A</th><th>B</th></tr></thead><tbody><tr><td>1</td><td>2</td></tr></tbody></Table><p>|-@@&#123;.x&#125;-|
</p>
<Table><thead><tr><th>T07n A</th><th>B</th></tr></thead><tbody><tr><td>1</td><td>2</td></tr></tbody></Table><p>|-@@tableX-|
</p>
<Table><thead><tr><th>T08 A</th><th>B</th></tr></thead><tbody><tr><td>1</td><td>2</td></tr></tbody></Table><p>||&#123;.c&#125; T09 caption extras only ||
</p>
<Table><thead><tr><th>A</th><th>B</th></tr></thead><tbody><tr><td>1</td><td>2</td></tr></tbody></Table><p>||@@&#123;.c&#125; T09n caption extras only ||
</p>
<Table><thead><tr><th>A</th><th>B</th></tr></thead><tbody><tr><td>1</td><td>2</td></tr></tbody></Table><p>|-[t10]-|
|| T10 plain caption ||
</p>
<Table><thead><tr><th>A</th><th>B</th></tr></thead><tbody><tr><td>1</td><td>2</td></tr></tbody></Table><p>|-<a class="x" href="" title="T11" type="tableZ">t11</a>-|
||@@captionZ&#123;.y&#125; T11 unclaimed ||
</p>
<Table><thead><tr><th>A</th><th>B</th></tr></thead><tbody><tr><td>1</td><td>2</td></tr></tbody></Table><p>T12 header only, no body:
</p>
<Table><thead><tr><th style="text-align: left;">A</th><th style="text-align: right;">B</th></tr></thead></Table><p>T13 footer without body rows:
</p>
<Table><thead><tr><th>A</th></tr></thead><tbody><tr><td>===</td></tr><tr><td>tot</td></tr></tbody></Table><p>T14 ragged:
</p>
<Table><thead><tr><th>A</th><th>B</th><th>C</th></tr></thead><tbody><tr><td>1</td><td></td><td></td></tr><tr><td>1</td><td>2</td><td>3</td></tr><tr><td colspan="2"></td></tr></tbody></Table><p>T15 alignment/width variants:
</p>
<Table><thead><tr><th>a</th><th style="text-align: left;">b</th><th style="text-align: center;">c</th><th style="text-align: right;">d</th><th style="width: 10%;">e</th><th style="text-align: center; width: 120px;">f</th></tr></thead><tbody><tr><td>1</td><td style="text-align: left;">2</td><td style="text-align: center;">3</td><td style="text-align: right;">4</td><td style="width: 10%;">5</td><td style="text-align: center; width: 120px;">6</td></tr></tbody></Table><p>T16 one column:
</p>
<Table><thead><tr><th>Only</th></tr></thead><tbody><tr><td>x</td></tr></tbody></Table><p>T17 table directly after paragraph line
</p>
<Table><thead><tr><th>A</th><th>B</th></tr></thead><tbody><tr><td>1</td><td>2</td></tr></tbody></Table><p>T18 untyped canonical on every table slot:
</p>
<Table class={"striped wide"} cols={4} id={"tbl18"} isDense={true} sortable={true} style={"--gap: 2rem"} title={"Title Foo"}><caption class="note" id="cap18">T18 caption <strong>b</strong></caption><thead><tr><th class="v-top" style="text-align: left; width: 200px;">&#123;.h1&#125; H1</th><th class="c" id="i" style="text-align: center;">&#123;.h2&#125; H2</th><th class="a b" id="i2" k="v" n="2" style="text-align: right; width: 30%;">H3</th><th b="true" class="a" flag id="i3" k="v" n="1" style="text-align: left;" title="T">H4</th></tr></thead><tbody class="tb"><tr><td class="v-top lead" style="text-align: left; width: 200px;">X</td><td class="c" id="i" style="text-align: center;">15</td><td class="a b" id="i2" k="v" n="2" style="text-align: right; width: 30%;"><a class="u" href="/docs">Ada</a></td><td b="true" class="a" flag id="i3" k="v" n="1" style="text-align: left;" title="T">[[W]]&#123;.w&#125;</td></tr><tr class="row-info"><td class="v-top" colspan="2" style="text-align: left; width: 200px;">plain</td><td class="a b" id="i2" k="v" n="2" style="text-align: right; width: 30%;"><a class="c" href="paper-smith">^^</a></td><td b="true" class="a" flag id="i3" k="v" n="1" style="text-align: left;" title="T">&#123;&#123;footnote&#125;&#125;&#123;.m&#125;</td></tr><tr><td class="v-top" style="text-align: left; width: 200px;">empty</td><td class="c" id="i" style="text-align: center;">::asideA&#123;.k&#125; d::</td><td class="a b" id="i2" k="v" n="2" style="text-align: right; width: 30%;">$x^2$</td><td b="true" class="a" flag id="i3" k="v" n="1" style="text-align: left;" title="T"><code>c</code></td></tr><tr class="a" id="r" k="v" n="2"><td class="v-top" style="text-align: left; width: 200px;">a</td><td class="c" id="i" style="text-align: center;">b</td><td class="a b" id="i2" k="v" n="2" style="text-align: right; width: 30%;">c</td><td b="true" class="a" flag id="i3" k="v" n="1" style="text-align: left;" title="T">d</td></tr><tr bar="12" class="extra class" foo="bar" id="id" isBar isFoo="true" style="--style-var: 2rem" title="Title Foo"><td class="v-top" style="text-align: left; width: 200px;">all-row</td><td class="c" id="i" style="text-align: center;">x</td><td class="a b" id="i2" k="v" n="2" style="text-align: right; width: 30%;">y</td><td b="true" class="a" flag id="i3" k="v" n="1" style="text-align: left;" title="T">z</td></tr></tbody><tfoot class="total"><tr class="f"><td class="v-top" colspan="2" style="text-align: left; width: 200px;">Total</td><td b="true" class="a" flag id="i3" k="v" n="1" style="text-align: left;" title="T">1</td></tr><tr><td class="v-top t" style="text-align: left; width: 200px;">F2</td><td class="c" id="i" style="text-align: center;">x</td><td class="a b" id="i2" k="v" n="2" style="text-align: right; width: 30%;">y</td><td b="true" class="a" flag id="i3" k="v" n="1" style="text-align: left;" title="T">z</td></tr></tfoot></Table><p>T19 type and brace separated by space: head stays literal:
</p>
<p>|-[t19]@@tableX &#123;.x&#125;-|
||@@captionX &#123;.y&#125; T19 caption ||
</p>
<Table><thead><tr><th>A</th><th>B</th></tr></thead><tbody><tr><td>@@cellA &#123;.z&#125; 1</td><td>2</td></tr></tbody></Table><p>T20 head not touching the pipe: stays literal (no exception for tables):
</p>
<p>|| &#123;.c&#125; T20 caption spaced ||
</p>
<Table><thead><tr><th>A</th><th>B</th></tr></thead><tbody><tr><td>&#123;.z&#125; spaced cell</td><td>@@cellA&#123;.y&#125; spaced typed cell</td></tr></tbody></Table><p>&lt;!-- T21: spaced head in a delimiter row. Not a valid delimiter row, so the table may not form (edge). --&gt;
</p>
<Table><thead><tr><th style="text-align: left;">A</th><th>B</th></tr></thead><tbody><tr><td style="text-align: left;">1</td><td>2</td></tr></tbody></Table><p>T22 rowspan (^ merges into the cell above):
</p>
<Table><thead><tr><th>A</th><th>B</th><th>C</th></tr></thead><tbody><tr><td rowspan="3">r1</td><td>x</td><td>y</td></tr><tr><td>p</td><td>q</td></tr><tr><td>s</td><td>t</td></tr></tbody></Table><p>T23 colspan (&gt; merges into the cell on its right), also in the header:
</p>
<Table><thead><tr><th>A</th><th><blockquote>
</blockquote>
</th><th>C</th></tr></thead><tbody><tr><td colspan="2">1</td><td>3</td></tr><tr><td>x</td><td>y</td><td>z</td></tr></tbody></Table><p>T24 chains and full-width spans:
</p>
<Table><thead><tr><th>A</th><th>B</th><th>C</th><th>D</th></tr></thead><tbody><tr><td colspan="2"></td><td>all four</td></tr><tr><td colspan="2">1</td><td>three</td></tr><tr><td></td><td>2</td><td>3</td><td>4</td></tr></tbody></Table><p>T25 colspan and rowspan forming a 2x2 block:
</p>
<Table><thead><tr><th>A</th><th>B</th><th>C</th></tr></thead><tbody><tr><td rowspan="2"></td><td rowspan="2">block 2x2</td><td>z</td></tr><tr><td>y</td></tr><tr><td>p</td><td>q</td><td>r</td></tr></tbody></Table><p>&lt;!-- T26: spans without a target: ^ in the header, ^ right under the header row, &gt; in the last column (edge). --&gt;
</p>
<Table><thead><tr><th>A</th><th>B</th><th>^</th></tr></thead><tbody><tr><td></td><td colspan="2">1</td></tr><tr><td>a</td><td>b</td><td>c</td></tr></tbody></Table><p>&lt;!-- T27: conflicting/ambiguous spans: ^ under a merged cell, &gt; next to ^ (edge, no expectation). --&gt;
</p>
<Table><thead><tr><th>A</th><th>B</th><th>C</th></tr></thead><tbody><tr><td colspan="2">1</td><td>3</td></tr><tr><td rowspan="2"></td><td>4</td></tr><tr></tr></tbody></Table><p>T28 lookalikes that must stay literal text:
</p>
<Table><thead><tr><th>a &gt; b</th><th><blockquote>
=</blockquote>
</th><th><blockquote>
<blockquote>
</blockquote>
</blockquote>
</th><th>x^2</th><th>^^</th><th>\&gt;</th><th>\^</th><th><blockquote>
x</blockquote>
</th><th>^ y</th></tr></thead><tbody><tr><td>1</td><td>2</td><td>3</td><td>4</td><td>5</td><td>6</td><td>7</td><td>8</td><td>9</td></tr></tbody></Table><p>&lt;!-- T29: spans in tfoot, including ^ right under the tbody/tfoot boundary (edge). --&gt;
</p>
<Table><thead><tr><th>A</th><th>B</th><th>C</th></tr></thead><tbody><tr><td>1</td><td>2</td><td>3</td></tr><tr><td rowspan="3">===</td><td></td><td></td></tr><tr><td>t</td></tr><tr><td>x</td><td>y</td></tr></tbody></Table><p>&lt;!-- T30: heads on cells that hold &gt; or ^, and a head on the target cell (edge, no expectation). --&gt;
</p>
<Table><thead><tr><th>A</th><th>B</th><th>C</th></tr></thead><tbody><tr><td></td><td>x</td><td>y</td></tr><tr><td class="s" rowspan="2">@@cellB&#123;&#125; &gt;</td><td>x</td><td>y</td></tr><tr><td>x</td><td>y</td></tr><tr><td class="s" colspan="2">a</td><td>y</td></tr><tr><td></td><td>@@cellB&#123;.span&#125; merged</td><td>z</td></tr></tbody></Table><p>T31 a cell starting with a marker touching the pipe (head vs marker, edge):
</p>
<Table><thead><tr><th>A</th><th>B</th><th>C</th></tr></thead><tbody><tr><td>&#123;&#123;footnote&#125;&#125;&#123;.m&#125; x</td><td>&#123;&#123;markerA&#125;&#125; y</td><td>z</td></tr><tr><td>&#123;&#123;footnote&#125;&#125;</td><td>w</td><td>v</td></tr></tbody></Table><h2 id="s-quote">2.8. Blockquote</h2>
<p>&lt;!-- §9.2: inner head (right after <code>&gt;</code>) + a decorator line directly above the quote. The inner head wins; an unclaimed type falls back to the layer default (§11 rule 3). §6.1 groups: a bracket group becomes the inner head&#39;s slug and a parentheses group its title; a bracket group immediately followed by a parentheses group is an anchor head and the anchor layer runs first, so the head keeps only its type. --&gt;
</p>
<blockquote>
<p>Q01 plain
continues
</p>
</blockquote>
<blockquote>
<p>&#123;.x&#125; Q02 untyped X
</p>
</blockquote>
<blockquote>
<p>@@&#123;.x&#125; Q02n untyped non-canonical
</p>
</blockquote>
<blockquote>
<p>@@bqA&#123;&#125; Q03 typed empty
</p>
</blockquote>
<blockquote>
<p>@@bqA&#123;.x&#125; Q04 X
</p>
</blockquote>
<blockquote>
<p>@@bqA&#123;.x, #i&#125; Q05 XY
</p>
</blockquote>
<blockquote>
<p>@@bqA&#123;.a, .b, #i, k: &quot;v&quot;, n: 2&#125; Q06 XXYZZ
</p>
</blockquote>
<blockquote>
<p>@@bqB&#123;<code>slug-foo</code>, &quot;Title Foo&quot;, .extra, .class, #id, foo: &quot;bar&quot;, bar: 12, isFoo: true, --style-var: &quot;2rem&quot;, isBar&#125; Q07 all
</p>
</blockquote>
<blockquote>
<p>@@bqZ&#123;.x&#125; Q08 unclaimed
</p>
</blockquote>
<blockquote>
<p>@@bqA Q09 type only
</p>
</blockquote>
<blockquote>
<p>@@bqA&#123;.x&#125; Q10 no space after &gt;
</p>
</blockquote>
<blockquote>
<p>Q11 not first @@bqA&#123;.x&#125; literal
</p>
</blockquote>
<blockquote>
<p>@@bqA&#123;.x&#125;
</p>
</blockquote>
<blockquote>
<p>@@bqA&#123;.outer&#125; Q13 outer
</p>
<blockquote>
<p>@@bqB&#123;.inner&#125; Q13a inner
</p>
<blockquote>
<p>Q13b triple plain
back to outer lazy
lazy continuation without marker
</p>
</blockquote>
</blockquote>
</blockquote>
<blockquote>
<p>@@bqA&#123;.q&#125; Q14 mixed
</p>
<h3>Q14 heading inside</h3>
<ul>
<li>@@liItem&#123;.i&#125; Q14 item</li>
<li>Q14 item 2</li>
</ul>
<table>
<thead>
<tr>
<th>A</th>
<th>B</th>
</tr>
</thead>
<tbody>
<tr>
<td>1</td>
<td>2</td>
</tr>
</tbody>
</table>
<pre lang="ts"><code>&gt; const q = &quot;@@bqA&#123;.literal&#125;&quot;;
&gt; ```
&gt;
&gt; $$x = y$$
&gt;
&gt; &#123;&#123;markerA&#125;&#125;&#123;.q&#125;

&gt; @@bqA&#123;.q15&#125; KS  **b** _i_ `c` [[W]]@@wikiX&#123;.k&#125;  &#123;&#123;footnote&#125;&#125;&#123;.k&#125; ::asideA&#123;.k&#125; d:: $x$

&gt; &#123;&#125; Q16 untyped empty

&gt; &#123;.x, #i&#125; Q17 untyped XY

&gt; &#123;.a, .b, #i, k: &quot;v&quot;, n: 2&#125; Q18 untyped XXYZZ

&gt; &#123;`slug-foo`, &quot;Title Foo&quot;, .extra, .class, #id, foo: &quot;bar&quot;, bar: 12, isFoo: true, --style-var: &quot;2rem&quot;, isBar&#125; Q19 untyped all

&gt;&#123;.x&#125; Q20 untyped, no space after &gt;

&gt; Q21 not first &#123;.x&#125; literal

&gt; &#123;.x&#125;

&gt; @@bqA &#123;.x&#125; Q23 type and brace separated by space (literal)

&gt; @@bqA[q25] Q25 bracket group on the inner head

&gt; @@bqA[q26( Q26 malformed inner-head group stays literal, type falls back

&gt; &#123;.q24&#125; KSu 
&lt;!-- §9.3 L1: a container decorator line above a list decorates the container (§9.4 layer = the marker). The `list` item layer (L2/L3) is not wired yet, so the L10/L22 item heads stay literal for now. --&gt;


- L01 plain
  - L01a
- L01b


&#123;.u&#125;
- L02 decorator untyped X
- L02b


@@&#123;.u&#125;
- L02n decorator non-canonical
- L02nb


@@unorderedA&#123;.u, #l03&#125;
- L03 star, XY
- L03b


@@compact&#123;.a, .b, #l04, k: &quot;v&quot;, n: 2&#125;
- L04 plus, XXYZZ
- L04b


@@unorderedA&#123;`slug-foo`, &quot;Title Foo&quot;, .extra, .class, #id, foo: &quot;bar&quot;, bar: 12, isFoo: true, --style-var: &quot;2rem&quot;, isBar&#125;
- L05 all
- L05b


@@unorderedZ&#123;.u&#125;
- L06 unclaimed


&#123;&#125;
- L07 empty


@@&#123;&#125;
- L07n empty non-canonical


@@unorderedA
- L08 type only


- @@liItem&#123;`alpha`&#125; L10 typed item
  - &#123;`beta`&#125; L10a nested untyped
  - @@liItem&#123;.a, #i&#125; L10b XY
- @@check&#123;.c, k: &quot;v&quot;, n: 2&#125; L11 XXYZZ
- @@data&#123;`slug-foo`, &quot;Title Foo&quot;, .extra, .class, #id, foo: &quot;bar&quot;, bar: 12, isFoo: true, --style-var: &quot;2rem&quot;, isBar&#125; L12 all
- @@liZ&#123;.z&#125; L13 unclaimed
- &#123;&#125; L14 empty
- @@&#123;&#125; L14n empty non-canonical
- @@liItem L15 type only
- L16 plain among extras
- &#123;.only&#125;
- @@liItem&#123;.x&#125;L17 glued
- &#123;.x&#125;L17u glued untyped
- @@liItem &#123;.x&#125; L17s type and brace separated by space (literal)


&#123;`my-list`&#125;
6. L20 start at 6
7. L20b


@@&#123;`my-list-n`&#125;
6. L20n start at 6 non-canonical
7. L20nb


1. L21 plain ol
2. L21b


@@orderedA&#123;.o, #l22&#125;
1. &#123;.i&#125; L22 container+item
2. @@liItem&#123;.a, .b, #i2, k: &quot;v&quot;, n: 2&#125; L22b


@@&#123;.o&#125;
1. @@&#123;.i&#125; L22n non-canonical container+item


0. L24 start 0
1. L24b


&#123;start: 3&#125;
1. L25 start via extras (expect Warning)


@@&#123;start: 3&#125;
1. L25n start via non-canonical extras (expect Warning)


o. L26 retired o. syntax (expect plain paragraph)


@@unorderedA&#123;.l1&#125;
- L30 level 1
  @@orderedA&#123;.l2&#125;
  1. L30a decorator right under item text (edge)
     - L30a1
  2. L30b
- L33 level 1
  @@orderedA&#123;.l2b&#125;
  1. L33a decorator after blank, inside item
  2. L33b


- L34 multi-paragraph

  second paragraph KS   ```ts
  const l = &quot;@@&#123;.literal&#125;&quot;;
</code></pre>
</blockquote>
<ul>
<li>L35 table in item<Table><thead><tr><th>A</th><th>B</th></tr></thead><tbody><tr><td>1</td><td>2</td></tr></tbody></Table></li>
<li>L36 quote in item<blockquote>
<p>@@bqA&#123;.in-li&#125; quoted
</p>
</blockquote>
</li>
<li>L37 directive in item<h1>===asideA</h1>
</li>
</ul>

</>); }
