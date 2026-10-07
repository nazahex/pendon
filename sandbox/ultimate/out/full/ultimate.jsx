import '@comp/shared/anchor-c.css'
import AdvancedImage from '@comp/content/AdvancedImage'
import Figure from '@comp/content/Figure'
import Note from '@comp/content/Note'
import Table from '@comp/content/Table'
import { AnchorAB } from '@comp/shared/Anchor'
import { AnchorC } from '@comp/shared/Anchor'
import { AnchorDefault } from '@comp/shared/Anchor'
import { AsideAB } from '@comp/content/AsideAB'
import { Bibliography } from '@comp/content/Cite'
import { Cite } from '@comp/content/Cite'
import { CiteX } from '@comp/content/Cite'
import { FigureX } from '@comp/content/Figure'
import { HeadingDefault } from '@comp/content/Heading'
import { HeadingXY } from '@comp/content/Heading'
import { ImageThumb } from '@comp/content/AdvancedImage'
import { MarkerA } from '@comp/content/Marker'
import { MarkerDefault } from '@comp/content/Marker'
import { TableBody } from '@comp/content/Table'
import { TableBodyX } from '@comp/content/Table'
import { TableCaption } from '@comp/content/Table'
import { TableCellAB } from '@comp/content/Table'
import { TableCellDefault } from '@comp/content/Table'
import { TableFoot } from '@comp/content/Table'
import { TableFootX } from '@comp/content/Table'
import { TableHead } from '@comp/content/Table'
import { TableRowAB } from '@comp/content/Table'
import { TableRowDefault } from '@comp/content/Table'
import { TableX } from '@comp/content/Table'

export const frontmatter = {"cites":[{"id":"paper-smith","index":1},{"id":"paper-smith","index":2,"loc":"hlm. 55"},{"id":"suryana-2026","index":3,"loc":"hlm. 45"},{"id":"doe-2020","index":4},{"id":"paper-smith","index":5,"loc":"hlm. 1"},{"id":"suryana-2026","index":6},{"id":"paper-smith","index":7,"loc":"hlm. 7"}],"references":{"doe-2020":{"authors":[{"firstName":"Jane","lastName":"Doe"}],"id":"doe-2020","issuedDate":{"year":2020},"language":"en","publisher":"Test Press","publisherLocation":"Bandung","title":"Edge Cases in Practice","type":"book"},"paper-smith":{"authors":[{"firstName":"John","lastName":"Smith"}],"containerTitle":"Journal of Web Engineering","doi":"10.1016/j.jwe.2025.08.001","id":"paper-smith","issue":"4","issuedDate":{"month":8,"year":2025},"language":"en","pages":"210-225","title":"Generative MDX to PDF Compilation Architectures","type":"journal","volume":"18"},"suryana-2026":{"authors":[{"firstName":"Eko","lastName":"Suryana"}],"id":"suryana-2026","isbn":"978-602-0000-00-0","issuedDate":{"year":2026},"language":"id","publisher":"TechPress Indonesia","publisherLocation":"Jakarta","title":"Masa Depan Rekayasa Perangkat Lunak","type":"book"}},"title":"Unified Syntax Torture Demo"};
export const headings = [{"id":"q14-heading-inside","text":"Q14 heading inside","level":3},{"id":"asideas-l37body","text":"===asideAs-l37body","level":1}];
export default function PendonView() { return (<>
<section>
<HeadingDefault id={"s-top"} level={1} number={1} raw_title={"Unified Demo"}>Unified Demo
</HeadingDefault><HeadingDefault id={"s-anchor"} level={2} number={"1.1."} raw_title={"Anchor"}>Anchor
</HeadingDefault><p>A01 <AnchorDefault href={"/a"}>plain</AnchorDefault>
</p>
<p>A02 <AnchorDefault href={"/a"} title={"T"}>title</AnchorDefault>
</p>
<p>A03 <AnchorDefault href={"/a"}>empty title</AnchorDefault>
</p>
<p>A04 <AnchorDefault href={"/a"}>t</AnchorDefault>
</p>
<p>A04n <AnchorDefault href={"/a"}>t</AnchorDefault>
</p>
<p>A05 <AnchorAB type={"anchorA"} href={"/a"}>t</AnchorAB>
</p>
<p>A06 <AnchorAB type={"anchorA"} href={"/a"}>t</AnchorAB>
</p>
<p>A07 <AnchorDefault class={"x"} href={"/a"}>t</AnchorDefault>
</p>
<p>A07n <AnchorDefault class={"x"} href={"/a"}>t</AnchorDefault>
</p>
<p>A08 <AnchorAB type={"anchorA"} class={"x"} href={"/a"}>t</AnchorAB>
</p>
<p>A09 <AnchorAB type={"anchorB"} class={"x"} href={"/a"} id={"i"}>t</AnchorAB>
</p>
<p>A10 <AnchorC slug={""} title={""} id={"i"} class={"a b"} href={"/a"} k={"v"} n={2} type={"anchorC"}>t</AnchorC>
</p>
<p>A11 <AnchorAB type={"anchorA"} href={"/a"} id={"s-only"}>t</AnchorAB>
</p>
<p>A12 <AnchorAB type={"anchorA"} href={"/a"} title={"Extras title"}>t</AnchorAB>
</p>
<p>A13 <AnchorAB type={"anchorA"} href={"/a"} title={"Head title"}>t</AnchorAB>
</p>
<p>A14 <AnchorAB type={"anchorA"} href={"/a"} k={"v"}>t</AnchorAB>
</p>
<p>A15 <AnchorAB type={"anchorA"} href={"/a"} n={12}>t</AnchorAB><AnchorAB type={"anchorA"} href={"/a"} n={-1.5}>t</AnchorAB>
</p>
<p>A16 <AnchorAB type={"anchorA"} b={true} href={"/a"}>t</AnchorAB><AnchorAB type={"anchorA"} b={false} href={"/a"}>t</AnchorAB>
</p>
<p>A17 <AnchorAB type={"anchorA"} href={"/a"} style={"--v: 2rem"}>t</AnchorAB>
</p>
<p>A18 <AnchorAB type={"anchorA"} href={"/a"} isBar={true}>t</AnchorAB>
</p>
<p>A19 <AnchorAB type={"anchorA"} bar={12} class={"extra class"} foo={"bar"} href={"/a"} id={"id"} isBar={true} isFoo={true} style={"--style-var: 2rem"} title={"Head"}>t</AnchorAB>
</p>
<p>A20 <AnchorAB type={"anchorB"} bar={12} class={"extra class"} foo={"bar"} href={"/a"} id={"id"} isBar={true} isFoo={true} style={"--style-var: 2rem"} title={"Title Foo"}>t</AnchorAB>
</p>
<p>A21 <AnchorAB type={"anchorB"} class={"from-extras"} href={"/a"} id={"explicit"}>t</AnchorAB>
</p>
<p>A22 <AnchorDefault class={"hero"} href={"/a"} type={"anchorZ"}>t</AnchorDefault>
</p>
<p>A23 <AnchorAB type={"anchorA"} class={"u"} href={"https://example.com/a?q=1&r=2#h"} rel={"noopener"} target={"_blank"}>abs</AnchorAB>
</p>
<p>A24 <AnchorDefault class={"u"} href={"./a/b.md"} rel={"noopener"} target={"_blank"}>rel</AnchorDefault>
</p>
<p>A24n <AnchorDefault class={"u"} href={"./a/b.md"} rel={"noopener"} target={"_blank"}>rel</AnchorDefault>
</p>
<p>A25 <AnchorDefault class={"u"} href={"#s-anchor"}>hash</AnchorDefault>
</p>
<p>A26 <AnchorDefault class={"u"} href={"mailto:a@b.co"} rel={"noopener"} target={"_blank"}>mail</AnchorDefault>
</p>
<p>A27 <AnchorAB type={"anchorB"} href={"https://example.com/bar"} hreflang={"en"} rel={"noopener noreferrer nofollow sponsored prefetch"} target={"_self"} title={"T"}>mod</AnchorAB>
</p>
<p>A28 <AnchorDefault href={"https://example.com/bar--!;"} rel={"noopener sponsored"} target={"_blank"}>mod2</AnchorDefault>
</p>
<p>A29 <AnchorDefault class={"u"} href={"/a_(b)"}>paren</AnchorDefault>
</p>
<p>A30 <AnchorAB type={"anchorA"} class={"m"} href={"/a"}><strong>b</strong> <i>i</i> <code>c</code> <span class="latex latex-inline" innerHTML={`<span class="katex"><span class="katex-mathml"><math xmlns="http://www.w3.org/1998/Math/MathML"><semantics><mrow><mi>m</mi></mrow><annotation encoding="application/x-tex">m</annotation></semantics></math></span><span class="katex-html" aria-hidden="true"><span class="base"><span class="strut" style="height:0.4306em;"></span><span class="mord mathnormal">m</span></span></span></span>`}></span></AnchorAB>
</p>
<p>A31 [a \[b\] c](/a)&#123;.e&#125;
</p>
<p>A32 <AnchorDefault href={"/a"} k={"a, b} c"} q={"x\"y"} u={"日本 😀"}>t</AnchorDefault>
</p>
<p>A33 <AnchorDefault class={"a"} href={"/1"}>a</AnchorDefault><AnchorDefault class={"b"} href={"/2"}>b</AnchorDefault><AnchorDefault href={"/3"}>c</AnchorDefault>
</p>
<p>A34 (<AnchorDefault class={"p"} href={"/a"}>t</AnchorDefault>), <AnchorDefault class={"p"} href={"/a"}>t</AnchorDefault>. <em></em><AnchorDefault class={"p"} href={"/a"}>t</AnchorDefault><em></em> _<AnchorDefault class={"p"} href={"/a"}>t</AnchorDefault>_
</p>
<p>A35 <AnchorDefault class={"o"} href={"/a"}><MarkerA type={"footnote"} class={"k"}></MarkerA> in anchor</AnchorDefault> [<AsideAB slug={""} title={""} type={"asideA"}> d</AsideAB>](/a)&#123;.o&#125;
</p>
<p>A36 <AnchorDefault class={"a b"} href={"/a"} id={"i"} k={"v"} n={2}>t</AnchorDefault>
</p>
<p>A36n <AnchorDefault class={"a b"} href={"/a"} id={"i"} k={"v"} n={2}>t</AnchorDefault>
</p>
<p>A37 <AnchorDefault bar={12} class={"extra class"} foo={"bar"} href={"/a"} id={"id"} isBar={true} isFoo={true} style={"--style-var: 2rem"} title={"Head"}>t</AnchorDefault>
</p>
<p>A37n <AnchorDefault bar={12} class={"extra class"} foo={"bar"} href={"/a"} id={"id"} isBar={true} isFoo={true} style={"--style-var: 2rem"} title={"Head"}>t</AnchorDefault>
</p>
<p>A38 <AnchorDefault bar={12} class={"extra class"} foo={"bar"} href={"/a"} id={"id"} isBar={true} isFoo={true} style={"--style-var: 2rem"} title={"Title Foo"}>t</AnchorDefault>
</p>
<p>A39 <AnchorDefault class={"x"} href={"/a"} id={"i"}>t</AnchorDefault>
</p>
<p>A40 <AnchorDefault href={"/a"} type={"anchorZ"}>t</AnchorDefault>
</p>
<p>A41 <AnchorDefault href={"/a"} id={"s-only"}>t</AnchorDefault>
</p>
<p>A42 <AnchorDefault href={"/a"} title={"Extras title"}>t</AnchorDefault>
</p>
<p>A43 <AnchorDefault href={"/a"} k={"v"}>t</AnchorDefault>
</p>
<p>A44 <AnchorDefault href={"/a"} style={"--v: 2rem"}>t</AnchorDefault>
</p>
<p>A45 <AnchorDefault href={"/a"} isBar={true}>t</AnchorDefault>
</p>
<p>A46 <AnchorDefault href={"/a"} title={"Head title"}>t</AnchorDefault>
</p>
<p>A47 <AnchorDefault class={"from-extras"} href={"/a"} id={"explicit"}>t</AnchorDefault>
</p>
<p>A48 <AnchorDefault href={"https://example.com/bar--!;"} rel={"noopener sponsored prefetch"} target={"_blank"}>mod3</AnchorDefault>
</p>
<p>A49 <AnchorAB type={"anchorA"} href={"/a"}>t</AnchorAB>.
</p>
<p>A50 (<AnchorAB type={"anchorA"} href={"/a"}>t</AnchorAB>)
</p>
<p>A51 <AnchorAB type={"anchorA"} href={"/a"}>t</AnchorAB>, <AnchorAB type={"anchorB"} href={"/b"}>u</AnchorAB>; <AnchorAB type={"anchorA"} href={"/c"}>v</AnchorAB>!
</p>
<p>A52 <AnchorDefault class={"x"} href={"/a"} type={"anchor1"}>t</AnchorDefault>
</p>
<HeadingDefault id={"s-heading"} level={2} number={"1.2."} raw_title={"Heading"}>Heading
</HeadingDefault><HeadingDefault id={"h01"} level={3} number={"1.2.1."} raw_title={"H01 slug"}>H01 slug
</HeadingDefault><HeadingDefault level={3} number={"1.2.2."} raw_title={"H02 title"} slug={"h02-title"} title={"T02"}>H02 title
</HeadingDefault><HeadingXY level={"3"} type={"headingX"} number={"1.2.3."} class={"x"} raw_title={"H03 type+X"} slug={"h03-type-x"}>H03 type+X
</HeadingXY><HeadingDefault level={3} number={"1.2.4."} raw_title={"H04 plain"} slug={"h04-plain"}>H04 plain
</HeadingDefault><HeadingDefault id={"h05"} level={3} number={"1.2.5."} raw_title={"H05 slug+title"} title={"T05"}>H05 slug+title
</HeadingDefault><HeadingXY level={"3"} type={"headingX"} number={"1.2.6."} class={"x"} id={"h06"} raw_title={"H06 slug+X"}>H06 slug+X
</HeadingXY><HeadingXY level={"3"} type={"headingY"} number={"1.2.7."} class={"x"} id={"i"} raw_title={"H07 title+XY"} title={"T07"}>H07 title+XY
</HeadingXY><HeadingXY level={"3"} type={"headingX"} number={"1.2.8."} class={"a b"} id={"i"} k={"v"} n={2} raw_title={"H08 XXYZZ"} title={"T08"}>H08 XXYZZ
</HeadingXY><HeadingXY level={"3"} type={"headingX"} number={"1.2.9."} bar={12} class={"extra class"} foo={"bar"} id={"id"} isBar={true} isFoo={true} raw_title={"H09 all"} style={"--style-var: 2rem"} title={"T09"}>H09 all
</HeadingXY><HeadingDefault class={"x"} level={3} number={"1.2.10."} raw_title={"H10 untyped"} slug={"h10-untyped"}>H10 untyped
</HeadingDefault><HeadingDefault class={"x"} level={3} number={"1.2.11."} raw_title={"H10n untyped non-canonical"} slug={"h10n-untyped-non-canonical"}>H10n untyped non-canonical
</HeadingDefault><HeadingDefault level={3} number={"1.2.12."} raw_title={"H11 empty"} slug={"h11-empty"}>H11 empty
</HeadingDefault><HeadingDefault level={3} number={"1.2.13."} raw_title={"H11n empty non-canonical"} slug={"h11n-empty-non-canonical"}>H11n empty non-canonical
</HeadingDefault><HeadingDefault class={"x"} level={3} number={"1.2.14."} raw_title={"H12 unclaimed"} slug={"h12-unclaimed"} type={"headingZ"}>H12 unclaimed
</HeadingDefault><HeadingXY level={"3"} type={"headingX"} number={"1.2.15."} raw_title={"H13 type only"} slug={"h13-type-only"}>H13 type only
</HeadingXY><HeadingDefault class={"x"} id={"h14"} level={3} number={"1.2.16."} raw_title={"H14 precedence"} title={"T14"}>H14 precedence
</HeadingDefault><HeadingDefault class={"x"} id={"h14n"} level={3} number={"1.2.17."} raw_title={"H14n precedence non-canonical"} title={"T14n"}>H14n precedence non-canonical
</HeadingDefault><HeadingXY level={"1"} type={"headingX"} number={"2."} class={"l1"} id={"h15"} raw_title={"H15 level 1"}>H15 level 1
</HeadingXY><HeadingDefault class={"l2"} level={2} number={"2.1."} raw_title={"H16 level 2"} slug={"h16-level-2"}>H16 level 2
</HeadingDefault><HeadingDefault class={"l2n"} level={2} number={"2.2."} raw_title={"H16n level 2 non-canonical"} slug={"h16n-level-2-non-canonical"}>H16n level 2 non-canonical
</HeadingDefault><HeadingXY level={"4"} type={"headingY"} number={"2.2.0.1."} class={"l4"} id={"h17"} raw_title={"H17 level 4"}>H17 level 4
</HeadingXY><HeadingDefault class={"l6"} level={6} number={"2.2.0.1.0.1."} raw_title={"H18 level 6 (skips 5)"} slug={"h18-level-6-skips-5"} title={"T18"}>H18 level 6 (skips 5)
</HeadingDefault><HeadingDefault class={"l6n"} level={6} number={"2.2.0.1.0.2."} raw_title={"H18n level 6 non-canonical"} slug={"h18n-level-6-non-canonical"} title={"T18n"}>H18n level 6 non-canonical
</HeadingDefault><HeadingXY level={"3"} type={"headingX"} number={"2.2.1."} class={"k"} id={"h19"} raw_title={"H19 KS [a](/x \"T\")@@anchorA{.k} **b** _i_ `c` [[W]]@@wikiX{.k} [^^](paper-smith)@@citeX{.k} {{footnote}}{.k} ::asideA{.k} d:: $x$"} title={"T19"}>H19 KS <p><AnchorAB type={"anchorA"} class={"k"} href={"/x"} title={"T"}>a</AnchorAB> <strong>b</strong> <i>i</i> <code>c</code> <a class="k" href="/wiki/W" title="W" type="wikiX">W</a><CiteX reference={frontmatter.references["paper-smith"]} class={"k"} index={1} type={"citeX"} /><MarkerA type={"footnote"} class={"k"}></MarkerA><AsideAB slug={""} title={""} type={"asideA"} class={"k"}> d</AsideAB> <span class="latex latex-inline" innerHTML={`<span class="katex"><span class="katex-mathml"><math xmlns="http://www.w3.org/1998/Math/MathML"><semantics><mrow><mi>x</mi></mrow><annotation encoding="application/x-tex">x</annotation></semantics></math></span><span class="katex-html" aria-hidden="true"><span class="base"><span class="strut" style="height:0.4306em;"></span><span class="mord mathnormal">x</span></span></span></span>`}></span>
</p>
</HeadingXY><HeadingXY level={"3"} type={"headingX"} number={"2.2.2."} class={"x"} raw_title={""}>
</HeadingXY><HeadingDefault id={"h21"} level={3} number={"2.2.3."} raw_title={"H21 closing hashes ###"}>H21 closing hashes ###
</HeadingDefault><HeadingDefault class={"x"} level={3} number={"2.2.4."} raw_title={""}>
</HeadingDefault><HeadingDefault class={"k"} id={"h24"} level={3} number={"2.2.5."} raw_title={"H24 KSu [a](/x \"T\"){.k} **b** _i_ `c` [[W]]{.k} [^^](paper-smith){.k} {{footnote}}{.k} ::asideA{.k} d:: $x$"}>H24 KSu <p><AnchorDefault class={"k"} href={"/x"} title={"T"}>a</AnchorDefault> <strong>b</strong> <i>i</i> <code>c</code> <a class="k" href="/wiki/W" title="W">W</a><Cite reference={frontmatter.references["paper-smith"]} class={"k"} index={1} /><MarkerA type={"footnote"} class={"k"}></MarkerA><AsideAB slug={""} title={""} type={"asideA"} class={"k"}> d</AsideAB> <span class="latex latex-inline" innerHTML={`<span class="katex"><span class="katex-mathml"><math xmlns="http://www.w3.org/1998/Math/MathML"><semantics><mrow><mi>x</mi></mrow><annotation encoding="application/x-tex">x</annotation></semantics></math></span><span class="katex-html" aria-hidden="true"><span class="base"><span class="strut" style="height:0.4306em;"></span><span class="mord mathnormal">x</span></span></span></span>`}></span>
</p>
</HeadingDefault><HeadingDefault class={"a"} id={"i"} k={"v"} level={3} number={"2.2.6."} raw_title={"H25 slug+title+XXYZ untyped"} title={"T25"}>H25 slug+title+XXYZ untyped
</HeadingDefault><HeadingDefault class={"a"} level={3} number={"2.2.7."} raw_title={"H26 title+X untyped"} slug={"h26-title-x-untyped"} title={"T26"}>H26 title+X untyped
</HeadingDefault><HeadingDefault class={"a"} id={"h27"} level={3} number={"2.2.8."} raw_title={"H27 slug+X untyped, slug precedence"}>H27 slug+X untyped, slug precedence
</HeadingDefault><HeadingDefault id={"s-image"} level={2} number={"2.3."} raw_title={"Image and figure"}>Image and figure
</HeadingDefault><AdvancedImage alt={"I01"} async_decoding={1} src={"/i.png"} width={800}></AdvancedImage><p><AdvancedImage alt={"I02"} async_decoding={1} src={"/i.png"} width={800}></AdvancedImage> Caption only
</p>
<AdvancedImage alt={"I03"} async_decoding={1} height={300} src={"/i.png"}></AdvancedImage><AdvancedImage alt={"I03n"} async_decoding={1} height={300} src={"/i.png"}></AdvancedImage><ImageThumb type={"imageX"} alt={"I04"} async_decoding={1} class={"thumb"} height={300} src={"/i.png"} width={800}></ImageThumb><p><ImageThumb type={"imageX"} alt={"I05"} async_decoding={1} class={"x"} id={"i"} lazy={1} src={"/i.png"} width={800}></ImageThumb> Caption <strong>b</strong>
</p>
<p><ImageThumb type={"thumb"} alt={"I06"} async_decoding={1} bar={12} class={"extra class"} foo={"bar"} height={300} id={"id"} isBar={true} isFoo={true} lazy={1} src={"/i.png"} style={"--style-var: 2rem;"} title={"Title Foo"} width={800}></ImageThumb> Caption all
</p>
<Figure container={"figure"}><AdvancedImage alt={"I07"} async_decoding={1} lazy={1} src={"/i.png"} width={800}></AdvancedImage></Figure><Figure container={"figure"}><AdvancedImage alt={"I08"} async_decoding={1} height={300} lazy={1} src={"/i.png"} width={800}></AdvancedImage>Figure caption only</Figure><FigureX class={"wide"} container={"figure"} id={"fig-1"} isLazy={true} style={"--rotate: 5deg;"} type={"figureX"}><AdvancedImage alt={"I09"} async_decoding={1} height={300} lazy={1} src={"https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp"} width={800}></AdvancedImage>Caption <strong>b</strong></FigureX><Figure container={"figure"} id={"s-i10"}><AdvancedImage alt={"I10"} async_decoding={1} lazy={1} src={"/i.png"} width={800}></AdvancedImage></Figure><Figure container={"figure"} id={"s-i10n"}><AdvancedImage alt={"I10n"} async_decoding={1} lazy={1} src={"/i.png"} width={800}></AdvancedImage></Figure><FigureX container={"figure"} type={"figureX"}><AdvancedImage alt={"I11"} async_decoding={1} lazy={1} src={"/i.png"}></AdvancedImage></FigureX><Figure class={"empty-alt"} container={"figure"}><AdvancedImage alt={""} async_decoding={1} lazy={1} src={"/i.png"} width={800}></AdvancedImage></Figure><Figure class={"x"} container={"figure"} type={"figureZ"}><AdvancedImage alt={"I13"} async_decoding={1} height={300} lazy={1} src={"/i.png"}></AdvancedImage>unclaimed</Figure><ImageThumb type={"imageX"} alt={"I14"} async_decoding={1} class={"a b"} id={"i"} k={"v"} n={2} src={"/i.png \"Head T\""} title={"Extras T"} width={100}></ImageThumb><FigureX container={"figure"} isA={true} type={"figureX"}><AdvancedImage alt={"I15"} async_decoding={1} lazy={1} src={"/i.png"} width={100}></AdvancedImage>Caption KS <AnchorAB type={"anchorA"} class={"k"} href={"/x"}>a</AnchorAB><CiteX reference={frontmatter.references["paper-smith"]} class={"k"} index={1} type={"citeX"} /><a class="k" href="/wiki/W" title="W" type="wikiX">W</a><MarkerA type={"footnote"} class={"k"}></MarkerA><AsideAB slug={""} title={""} type={"asideA"} class={"k"}> d</AsideAB><span class="latex latex-inline" innerHTML={`<span class="katex"><span class="katex-mathml"><math xmlns="http://www.w3.org/1998/Math/MathML"><semantics><mrow><mi>x</mi></mrow><annotation encoding="application/x-tex">x</annotation></semantics></math></span><span class="katex-html" aria-hidden="true"><span class="base"><span class="strut" style="height:0.4306em;"></span><span class="mord mathnormal">x</span></span></span></span>`}></span><strong>b</strong></FigureX><p><img alt="I16 plain markdown image" src="/i.png" />
</p>
<AdvancedImage alt={"I17"} async_decoding={1} class={"thumb"} height={300} src={"/i.png"} width={800}></AdvancedImage><p><AdvancedImage alt={"I18"} async_decoding={1} bar={12} class={"extra class"} foo={"bar"} height={300} id={"id"} isBar={true} isFoo={true} lazy={1} src={"/i.png"} style={"--style-var: 2rem;"} title={"Title Foo"} width={800}></AdvancedImage> Caption all
</p>
<p><AdvancedImage alt={"I18n"} async_decoding={1} bar={12} class={"extra class"} foo={"bar"} height={300} id={"id"} isBar={true} isFoo={true} lazy={1} src={"/i.png"} style={"--style-var: 2rem;"} title={"Title Foo"} width={800}></AdvancedImage> Caption all
</p>
<Figure class={"wide"} container={"figure"} id={"fig-2"} isLazy={true} style={"--rotate: 5deg;"}><AdvancedImage alt={"I19"} async_decoding={1} height={300} lazy={1} src={"/i.png"} width={800}></AdvancedImage>Caption</Figure><Figure container={"figure"}><AdvancedImage alt={"I20"} async_decoding={1} lazy={1} src={"/i.png"}></AdvancedImage></Figure><FigureX container={"figure"} type={"figureX"}><AdvancedImage alt={"I21"} async_decoding={1} lazy={1} src={"/i.png"}></AdvancedImage></FigureX><AdvancedImage alt={"I22"} async_decoding={1} class={"a"} src={"/i.png \"Head T\""} title={"Extras T"} width={100}></AdvancedImage><Figure class={"f"} container={"figure"}><AdvancedImage alt={"I23"} async_decoding={1} lazy={1} src={"/i.png"} width={100}></AdvancedImage>Caption KSu <AnchorDefault class={"k"} href={"/x"}>a</AnchorDefault><Cite reference={frontmatter.references["paper-smith"]} class={"k"} index={1} /><a class="k" href="/wiki/W" title="W">W</a><MarkerA type={"footnote"} class={"k"}></MarkerA><AsideAB slug={""} title={""} type={"asideA"} class={"k"}> d</AsideAB><span class="latex latex-inline" innerHTML={`<span class="katex"><span class="katex-mathml"><math xmlns="http://www.w3.org/1998/Math/MathML"><semantics><mrow><mi>x</mi></mrow><annotation encoding="application/x-tex">x</annotation></semantics></math></span><span class="katex-html" aria-hidden="true"><span class="base"><span class="strut" style="height:0.4306em;"></span><span class="mord mathnormal">x</span></span></span></span>`}></span><strong>b</strong></Figure><HeadingDefault id={"s-cite"} level={2} number={"2.4."} raw_title={"Cite"}>Cite
</HeadingDefault><p><AnchorDefault href={"id"} title={"loc"}></AnchorDefault><AnchorDefault href={""} title={"id\", \"loc"}></AnchorDefault>
</p>
<p>C01 Minim <Cite reference={frontmatter.references["paper-smith"]} index={1} /> esse.
</p>
<p>C02 Minim <Cite reference={frontmatter.references["paper-smith"]} index={2} loc={"hlm. 55"} /> esse.
</p>
<p>C03 Minim <Cite reference={frontmatter.references["paper-smith"]} index={1} /> esse.
</p>
<p>C03n Minim <Cite reference={frontmatter.references["paper-smith"]} index={1} /> esse.
</p>
<p>C04 Minim <CiteX reference={frontmatter.references["paper-smith"]} index={1} type={"citeX"} /> esse.
</p>
<p>C05 Minim <CiteX reference={frontmatter.references["paper-smith"]} class={"paper"} index={1} type={"citeX"} /> esse.
</p>
<p>C06 Minim <CiteX reference={frontmatter.references["paper-smith"]} cite-id={"smith"} class={"paper"} index={1} type={"citeX"} /> esse.
</p>
<p>C07 Minim <CiteX reference={frontmatter.references["suryana-2026"]} cite-id={"i"} class={"a b"} index={3} loc={"hlm. 45"} n={2} note={"short"} type={"citeX"} /> esse.
</p>
<p>C08 Minim <CiteX reference={frontmatter.references["doe-2020"]} bar={12} cite-id={"id"} class={"extra class"} foo={"bar"} index={4} isBar={true} isFoo={true} style={"--style-var: 2rem;"} title={"Title Foo"} type={"citeX"} /> esse.
</p>
<p>C09 <CiteX reference={frontmatter.references["paper-smith"]} class={"thin"} index={5} loc={"hlm. 1"} type={"citeX"} />
</p>
<p>C10 <AnchorDefault class={"x"} href={"missing-ref"} type={"citeX"}>^^</AnchorDefault>
</p>
<p>C11 <Cite reference={frontmatter.references["paper-smith"]} index={1} /><Cite reference={frontmatter.references["suryana-2026"]} class={"y"} index={6} /><Cite reference={frontmatter.references["doe-2020"]} index={4} />
</p>
<p>C12 <Cite reference={frontmatter.references["paper-smith"]} index={1} /> dan lagi <CiteX reference={frontmatter.references["paper-smith"]} class={"again"} index={1} type={"citeX"} />
</p>
<p>C13 <AnchorDefault href={""}>^^</AnchorDefault>
</p>
<p>C14 <Cite reference={frontmatter.references["paper-smith"]} class={"x"} index={1} type={"citeZ"} /> unclaimed
</p>
<p>C15 <CiteX reference={frontmatter.references["paper-smith"]} index={1} type={"citeX"} />
</p>
<p>C16 Minim <Cite reference={frontmatter.references["paper-smith"]} class={"paper"} index={1} /> esse.
</p>
<p>C17 Minim <Cite reference={frontmatter.references["paper-smith"]} cite-id={"smith2"} class={"paper"} index={7} loc={"hlm. 7"} /> esse.
</p>
<p>C18 Minim <Cite reference={frontmatter.references["suryana-2026"]} cite-id={"i2"} class={"a b"} index={3} loc={"hlm. 45"} n={2} note={"short"} /> esse.
</p>
<p>C19 Minim <Cite reference={frontmatter.references["doe-2020"]} bar={12} cite-id={"id"} class={"extra class"} foo={"bar"} index={4} isBar={true} isFoo={true} style={"--style-var: 2rem;"} title={"Title Foo"} /> esse.
</p>
<p>C19n Minim <Cite reference={frontmatter.references["doe-2020"]} bar={12} cite-id={"id"} class={"extra class"} foo={"bar"} index={4} isBar={true} isFoo={true} style={"--style-var: 2rem;"} title={"Title Foo"} /> esse.
</p>
<p>C20 <AnchorDefault href={"paper-smith loc="} title={"p.1"}>^^</AnchorDefault>
</p>
<p>C21 <AnchorDefault href={""} title={"paper-smith"}>^^</AnchorDefault>
</p>
<p>C22 <AnchorDefault href={""} title={"paper-smith\", \"hlm. 55"}>^^</AnchorDefault>
</p>
<p>C23 Minim <CiteX reference={frontmatter.references["paper-smith"]} index={1} type={"citeX"} />. esse
</p>
<HeadingDefault id={"s-wiki"} level={2} number={"2.5."} raw_title={"Wiki"}>Wiki
</HeadingDefault><p>W01 <a href="/wiki/Wireless" title="Wireless">Wireless</a>
</p>
<p>W02 <a href="/wiki/Anim_Esta_(Officia)" title="Anim Esta (Officia)">Anim</a>
</p>
<p>W03 <a href="/wiki/Wireless" title="Wireless">Wireless</a>
</p>
<p>W03n <a href="/wiki/Wireless" title="Wireless">Wireless</a>
</p>
<p>W04 <a class="link" href="/wiki/Wireless" title="Wireless" type="wikiX">Wireless</a>
</p>
<p>W05 <a class="a" href="/wiki/Wireless" id="i" title="Wireless" type="wikiX">Wireless</a>
</p>
<p>W06 <a class="a b" href="/wiki/Wireless" id="i" k="v" n="2" title="Wireless" type="wikiX">Wireless</a>
</p>
<p>W07 <a bar="12" class="extra class" foo="bar" href="/wiki/Anim_Esta" id="id" isBar isFoo="true" slug="slug-foo" style="--style-var: 2rem" title="Anim Esta" type="wikiX">Anim Esta</a>
</p>
<p>W08 <a href="/wiki/Wireless" title="Wireless" type="wikiX">Wireless</a>
</p>
<p>W09 <a class="u" href="/wiki/Café_Ünïcode_日本" title="Café Ünïcode 日本">Café Ünïcode 日本</a>
</p>
<p>W09n <a class="u" href="/wiki/Café_Ünïcode_日本" title="Café Ünïcode 日本">Café Ünïcode 日本</a>
</p>
<p>W10 <a class="a" href="/wiki/A" title="A">a</a><a class="b" href="/wiki/B" title="B">b</a><a href="/wiki/C" title="C">c</a>
</p>
<p>W11 [[ ]]<a href="/wiki/A" title="A">b|c</a>[[]]
</p>
<p>W12 <a href="/wiki/Wireless" title="Wireless">Wireless</a> @@wikiX&#123;.spaced&#125;
</p>
<p>W13 <a class="x" href="/wiki/Wireless" title="Wireless" type="wikiZ">Wireless</a>
</p>
<p>W14 <a href="/wiki/Wireless" title="Wireless">Wireless</a>@@wikiX &#123;.x&#125; type and brace separated by space
</p>
<p>W15 <a href="/wiki/Wireless" title="Wireless">Wireless</a> &#123;.x&#125; bare brace separated by space
</p>
<p>W16 <a class="link" href="/wiki/Wireless" title="Wireless">Wireless</a>
</p>
<p>W17 <a class="a" href="/wiki/Wireless" id="i" title="Wireless">Wireless</a>
</p>
<p>W18 <a class="a b" href="/wiki/Wireless" id="i" k="v" n="2" title="Wireless">Wireless</a>
</p>
<p>W19 <a bar="12" class="extra class" foo="bar" href="/wiki/Anim_Esta" id="id" isBar isFoo="true" slug="slug-foo" style="--style-var: 2rem" title="Anim Esta">Anim Esta</a>
</p>
<p>W19n <a bar="12" class="extra class" foo="bar" href="/wiki/Anim_Esta" id="id" isBar isFoo="true" slug="slug-foo" style="--style-var: 2rem" title="Anim Esta">Anim Esta</a>
</p>
<p>W20 <a href="/wiki/Wireless" title="Wireless">Wireless</a>
</p>
<p>W21 <a href="/wiki/Wireless" title="Wireless" type="wikiX">Wireless</a>.
</p>
<HeadingDefault id={"s-marker"} level={2} number={"2.6."} raw_title={"Marker"}>Marker
</HeadingDefault><p>M01 inline <MarkerA type={"footnote"}></MarkerA> ipsum.
</p>
<p>M02 inline <MarkerA type={"footnote"} class={"x"}></MarkerA> ipsum.
</p>
<p>M03 inline <MarkerA type={"footnote"} class={"a"} id={"i"} slug={"s"}></MarkerA> ipsum.
</p>
<p>M04 inline <MarkerA type={"footnote"} class={"a b"} id={"i"} k={"v"} n={2}></MarkerA> ipsum.
</p>
<p>M05 inline <MarkerA type={"markerA"} bar={12} class={"extra class"} foo={"bar"} id={"id"} isBar={true} isFoo={true} slug={"slug-foo"} style={"--style-var: 2rem"} title={"Title Foo"}></MarkerA> ipsum.
</p>
<p>M06 <MarkerA type={"footnote"}></MarkerA>
</p>
<Bibliography cites={frontmatter.cites} references={frontmatter.references} /><Bibliography cites={frontmatter.cites} references={frontmatter.references} /><MarkerA type={"markerA"}>trailing text as children</MarkerA><p><MarkerA type={"markerA"} class={"x"} id={"i"}></MarkerA> trailing <strong>text</strong> <AnchorAB type={"anchorA"} class={"k"} href={"/x"}>a</AnchorAB>
</p>
<MarkerA type={"markerA"} bar={12} class={"extra class"} foo={"bar"} id={"id"} isBar={true} isFoo={true} slug={"slug-foo"} style={"--style-var: 2rem"} title={"Title Foo"}></MarkerA><MarkerDefault type={"unknownType"}>unclaimed falls to default</MarkerDefault><MarkerDefault class={"u"} type={"unknownType"}></MarkerDefault><p>M10 <MarkerA type={"footnote"} class={"a"}></MarkerA> dan <MarkerA type={"footnote"}></MarkerA> dan <MarkerA type={"markerA"} class={"b"}></MarkerA><MarkerA type={"markerA"}></MarkerA>
</p>
<p>M11 &#123;&#123;&#125;&#125; &#123;&#123;bad-type&#125;&#125; &#123;&#123;bad_type&#125;&#125; <MarkerDefault type={"type1"}></MarkerDefault> &#123;&#123; spaced &#125;&#125; <MarkerA type={"footnote"}></MarkerA> &#123;.spaced&#125;
</p>
<p>M12 <MarkerDefault class={"upper"} type={"Footnote"}></MarkerDefault><MarkerDefault type={"FOOTNOTE"}></MarkerDefault>
</p>
<HeadingDefault id={"s-table"} level={2} number={"2.7."} raw_title={"Table"}>Table
</HeadingDefault><TableX class={"striped wide"} cols={4} id={"tbl"} isDense={true} sortable={true} style={"--gap: 2rem"} title={"Title Foo"} type={"tableX"}><TableCaption class={"note"} id={"cap"} type={"captionX"}>T01 caption <strong>b</strong></TableCaption><TableHead><TableRowDefault><TableCellAB type={"cellA"} align={"left"} class={"v-top"} width={"200px"}>&#123;.h1&#125; H1</TableCellAB><TableCellDefault align={"center"} class={"c"}>@@cellA&#123;.h2&#125; H2</TableCellDefault><TableCellDefault align={"right"} width={"30%"}>H3</TableCellDefault><TableCellAB type={"cellB"} align={"left"} class={"a"} id={"i"} k={"v"} n={2}>H4</TableCellAB></TableRowDefault></TableHead><TableBodyX class={"tb"} type={"tbodyX"}><TableRowDefault><TableCellAB type={"cellB"} align={"left"} class={"v-top lead"} width={"200px"}>X</TableCellAB><TableCellDefault align={"center"} class={"c"}>15</TableCellDefault><TableCellDefault align={"right"} width={"30%"}><AnchorAB type={"anchorA"} class={"u"} href={"/docs"}>Ada</AnchorAB></TableCellDefault><TableCellAB type={"cellB"} align={"left"} class={"a"} id={"i"} k={"v"} n={2}><a class="w" href="/wiki/W" title="W" type="wikiX">W</a></TableCellAB></TableRowDefault><TableRowAB type={"rowB"} class={"row-info"}><TableCellAB type={"cellA"} align={"left"} class={"v-top"} colspan={2} width={"200px"}>plain</TableCellAB><TableCellDefault align={"right"} width={"30%"}><CiteX reference={frontmatter.references["paper-smith"]} class={"c"} index={1} type={"citeX"} /></TableCellDefault><TableCellAB type={"cellB"} align={"left"} class={"a"} id={"i"} k={"v"} n={2}><MarkerA type={"footnote"} class={"m"}></MarkerA></TableCellAB></TableRowAB><TableRowAB type={"rowA"}><TableCellAB type={"cellA"} align={"left"} class={"v-top"} width={"200px"}>empty-extras</TableCellAB><TableCellDefault align={"center"} class={"c"}><AsideAB slug={""} title={""} type={"asideA"} class={"k"}> d</AsideAB></TableCellDefault><TableCellDefault align={"right"} width={"30%"}><span class="latex latex-inline" innerHTML={`<span class="katex"><span class="katex-mathml"><math xmlns="http://www.w3.org/1998/Math/MathML"><semantics><mrow><msup><mi>x</mi><mn>2</mn></msup></mrow><annotation encoding="application/x-tex">x^2</annotation></semantics></math></span><span class="katex-html" aria-hidden="true"><span class="base"><span class="strut" style="height:0.8141em;"></span><span class="mord"><span class="mord mathnormal">x</span><span class="msupsub"><span class="vlist-t"><span class="vlist-r"><span class="vlist" style="height:0.8141em;"><span style="top:-3.063em;margin-right:0.05em;"><span class="pstrut" style="height:2.7em;"></span><span class="sizing reset-size6 size3 mtight"><span class="mord mtight">2</span></span></span></span></span></span></span></span></span></span></span>`}></span></TableCellDefault><TableCellAB type={"cellB"} align={"left"} class={"a"} id={"i"} k={"v"} n={2}><code>c</code></TableCellAB></TableRowAB><TableRowDefault class={"untyped-row"}><TableCellAB type={"cellA"} align={"left"} class={"v-top"} width={"200px"}>a</TableCellAB><TableCellDefault align={"center"} class={"c"}>b</TableCellDefault><TableCellDefault align={"right"} width={"30%"}>c</TableCellDefault><TableCellAB type={"cellB"} align={"left"} class={"a"} id={"i"} k={"v"} n={2}>d</TableCellAB></TableRowDefault><TableRowAB type={"rowB"} class={"a"} id={"r"} k={"v"} n={2}><TableCellAB type={"cellA"} align={"left"} class={"v-top"} width={"200px"}>\</TableCellAB><TableCellDefault align={"center"} class={"c"}>escaped</TableCellDefault><TableCellDefault align={"right"} width={"30%"}></TableCellDefault><TableCellAB type={"cellB"} align={"left"} class={"a"} id={"i"} k={"v"} n={2}></TableCellAB></TableRowAB><TableRowAB type={"rowB"} bar={12} class={"extra class"} foo={"bar"} id={"id"} isBar={true} isFoo={true} style={"--style-var: 2rem"} title={"Title Foo"}><TableCellAB type={"cellA"} align={"left"} class={"v-top"} width={"200px"}>all-row</TableCellAB><TableCellDefault align={"center"} class={"c"}>x</TableCellDefault><TableCellDefault align={"right"} width={"30%"}>y</TableCellDefault><TableCellAB type={"cellB"} align={"left"} class={"a"} id={"i"} k={"v"} n={2}>z</TableCellAB></TableRowAB><TableRowDefault><TableCellAB type={"cellA"} align={"left"} class={"v-top only-extras"} width={"200px"}></TableCellAB><TableCellDefault align={"center"} class={"c"}><strong>b</strong> <i>i</i></TableCellDefault><TableCellDefault align={"right"} width={"30%"}><AnchorAB type={"anchorA"} class={"k"} href={"/x"} title={"T"}>a</AnchorAB><CiteX reference={frontmatter.references["paper-smith"]} class={"k"} index={1} type={"citeX"} /></TableCellDefault><TableCellAB type={"cellB"} align={"left"} b={true} class={"a a"} flag={true} id={"i"} k={"v"} n={1} style={"--v: 1"} title={"T"}>KS</TableCellAB></TableRowDefault></TableBodyX><TableFootX class={"total"} type={"tfootX"}><TableRowAB type={"rowA"} class={"f"}><TableCellAB type={"cellA"} align={"left"} class={"v-top"} colspan={2} width={"200px"}>Total</TableCellAB><TableCellAB type={"cellB"} align={"left"} class={"a"} id={"i"} k={"v"} n={2}>1</TableCellAB></TableRowAB><TableRowDefault><TableCellAB type={"cellA"} align={"left"} class={"v-top t"} width={"200px"}>F2</TableCellAB><TableCellDefault align={"center"} class={"c"}>x</TableCellDefault><TableCellDefault align={"right"} width={"30%"}>y</TableCellDefault><TableCellAB type={"cellB"} align={"left"} class={"a"} id={"i"} k={"v"} n={2}>z</TableCellAB></TableRowDefault></TableFootX></TableX><p>T01n non-canonical @@&#123;...&#125; on every table slot:
</p>
<Table class={"x"} id={"t01n"}><TableCaption class={"c"}>T01n caption</TableCaption><TableHead><TableRowDefault><TableCellDefault align={"left"} class={"v"}>A</TableCellDefault><TableCellDefault>B</TableCellDefault></TableRowDefault></TableHead><TableBody class={"tb"}><TableRowDefault class={"r"}><TableCellDefault align={"left"} class={"v l"}>x</TableCellDefault><TableCellDefault>y</TableCellDefault></TableRowDefault></TableBody><TableFoot class={"f"}><TableRowDefault><TableCellDefault align={"left"} class={"v"}>t</TableCellDefault><TableCellDefault>u</TableCellDefault></TableRowDefault></TableFoot></Table><p>T02 plain, no declaration, no caption, no extras:
</p>
<Table><TableHead><TableRowDefault><TableCellDefault>A</TableCellDefault><TableCellDefault>B</TableCellDefault></TableRowDefault></TableHead><TableBody><TableRowDefault><TableCellDefault>1</TableCellDefault><TableCellDefault>2</TableCellDefault></TableRowDefault><TableRowDefault><TableCellDefault>3</TableCellDefault><TableCellDefault>4</TableCellDefault></TableRowDefault><TableRowDefault><TableCellDefault>===</TableCellDefault><TableCellDefault></TableCellDefault></TableRowDefault><TableRowDefault><TableCellDefault>total</TableCellDefault><TableCellDefault>6</TableCellDefault></TableRowDefault></TableBody></Table><p>|| T03 caption only ||
</p>
<Table><TableHead><TableRowDefault><TableCellDefault>A</TableCellDefault><TableCellDefault>B</TableCellDefault></TableRowDefault></TableHead><TableBody><TableRowDefault><TableCellDefault>1</TableCellDefault><TableCellDefault>2</TableCellDefault></TableRowDefault></TableBody></Table><p>|- -|
</p>
<Table><TableHead><TableRowDefault><TableCellDefault>T04 A</TableCellDefault><TableCellDefault>B</TableCellDefault></TableRowDefault></TableHead><TableBody><TableRowDefault><TableCellDefault>1</TableCellDefault><TableCellDefault>2</TableCellDefault></TableRowDefault></TableBody></Table><p>|-[t05]-|
</p>
<Table><TableHead><TableRowDefault><TableCellDefault>T05 A</TableCellDefault><TableCellDefault>B</TableCellDefault></TableRowDefault></TableHead><TableBody><TableRowDefault><TableCellDefault>1</TableCellDefault><TableCellDefault>2</TableCellDefault></TableRowDefault></TableBody></Table><p>|-(&quot;Table 06&quot;)-|
</p>
<Table><TableHead><TableRowDefault><TableCellDefault>T06 A</TableCellDefault><TableCellDefault>B</TableCellDefault></TableRowDefault></TableHead><TableBody><TableRowDefault><TableCellDefault>1</TableCellDefault><TableCellDefault>2</TableCellDefault></TableRowDefault></TableBody></Table><p>|-&#123;.x&#125;-|
</p>
<Table><TableHead><TableRowDefault><TableCellDefault>T07 A</TableCellDefault><TableCellDefault>B</TableCellDefault></TableRowDefault></TableHead><TableBody><TableRowDefault><TableCellDefault>1</TableCellDefault><TableCellDefault>2</TableCellDefault></TableRowDefault></TableBody></Table><p>|-@@&#123;.x&#125;-|
</p>
<Table><TableHead><TableRowDefault><TableCellDefault>T07n A</TableCellDefault><TableCellDefault>B</TableCellDefault></TableRowDefault></TableHead><TableBody><TableRowDefault><TableCellDefault>1</TableCellDefault><TableCellDefault>2</TableCellDefault></TableRowDefault></TableBody></Table><p>|-@@tableX-|
</p>
<Table><TableHead><TableRowDefault><TableCellDefault>T08 A</TableCellDefault><TableCellDefault>B</TableCellDefault></TableRowDefault></TableHead><TableBody><TableRowDefault><TableCellDefault>1</TableCellDefault><TableCellDefault>2</TableCellDefault></TableRowDefault></TableBody></Table><p>||&#123;.c&#125; T09 caption extras only ||
</p>
<Table><TableHead><TableRowDefault><TableCellDefault>A</TableCellDefault><TableCellDefault>B</TableCellDefault></TableRowDefault></TableHead><TableBody><TableRowDefault><TableCellDefault>1</TableCellDefault><TableCellDefault>2</TableCellDefault></TableRowDefault></TableBody></Table><p>||@@&#123;.c&#125; T09n caption extras only ||
</p>
<Table><TableHead><TableRowDefault><TableCellDefault>A</TableCellDefault><TableCellDefault>B</TableCellDefault></TableRowDefault></TableHead><TableBody><TableRowDefault><TableCellDefault>1</TableCellDefault><TableCellDefault>2</TableCellDefault></TableRowDefault></TableBody></Table><p>|-[t10]-|
|| T10 plain caption ||
</p>
<Table><TableHead><TableRowDefault><TableCellDefault>A</TableCellDefault><TableCellDefault>B</TableCellDefault></TableRowDefault></TableHead><TableBody><TableRowDefault><TableCellDefault>1</TableCellDefault><TableCellDefault>2</TableCellDefault></TableRowDefault></TableBody></Table><p>|-<AnchorDefault class={"x"} href={""} title={"T11"} type={"tableZ"}>t11</AnchorDefault>-|
||@@captionZ&#123;.y&#125; T11 unclaimed ||
</p>
<Table><TableHead><TableRowDefault><TableCellDefault>A</TableCellDefault><TableCellDefault>B</TableCellDefault></TableRowDefault></TableHead><TableBody><TableRowDefault><TableCellDefault>1</TableCellDefault><TableCellDefault>2</TableCellDefault></TableRowDefault></TableBody></Table><p>T12 header only, no body:
</p>
<Table><TableHead><TableRowDefault><TableCellDefault align={"left"}>A</TableCellDefault><TableCellDefault align={"right"}>B</TableCellDefault></TableRowDefault></TableHead></Table><p>T13 footer without body rows:
</p>
<Table><TableHead><TableRowDefault><TableCellDefault>A</TableCellDefault></TableRowDefault></TableHead><TableBody><TableRowDefault><TableCellDefault>===</TableCellDefault></TableRowDefault><TableRowDefault><TableCellDefault>tot</TableCellDefault></TableRowDefault></TableBody></Table><p>T14 ragged:
</p>
<Table><TableHead><TableRowDefault><TableCellDefault>A</TableCellDefault><TableCellDefault>B</TableCellDefault><TableCellDefault>C</TableCellDefault></TableRowDefault></TableHead><TableBody><TableRowDefault><TableCellDefault>1</TableCellDefault><TableCellDefault></TableCellDefault><TableCellDefault></TableCellDefault></TableRowDefault><TableRowDefault><TableCellDefault>1</TableCellDefault><TableCellDefault>2</TableCellDefault><TableCellDefault>3</TableCellDefault></TableRowDefault><TableRowDefault><TableCellDefault colspan={2}></TableCellDefault></TableRowDefault></TableBody></Table><p>T15 alignment/width variants:
</p>
<Table><TableHead><TableRowDefault><TableCellDefault>a</TableCellDefault><TableCellDefault align={"left"}>b</TableCellDefault><TableCellDefault align={"center"}>c</TableCellDefault><TableCellDefault align={"right"}>d</TableCellDefault><TableCellDefault width={"10%"}>e</TableCellDefault><TableCellDefault align={"center"} width={"120px"}>f</TableCellDefault></TableRowDefault></TableHead><TableBody><TableRowDefault><TableCellDefault>1</TableCellDefault><TableCellDefault align={"left"}>2</TableCellDefault><TableCellDefault align={"center"}>3</TableCellDefault><TableCellDefault align={"right"}>4</TableCellDefault><TableCellDefault width={"10%"}>5</TableCellDefault><TableCellDefault align={"center"} width={"120px"}>6</TableCellDefault></TableRowDefault></TableBody></Table><p>T16 one column:
</p>
<Table><TableHead><TableRowDefault><TableCellDefault>Only</TableCellDefault></TableRowDefault></TableHead><TableBody><TableRowDefault><TableCellDefault>x</TableCellDefault></TableRowDefault></TableBody></Table><p>T17 table directly after paragraph line
</p>
<Table><TableHead><TableRowDefault><TableCellDefault>A</TableCellDefault><TableCellDefault>B</TableCellDefault></TableRowDefault></TableHead><TableBody><TableRowDefault><TableCellDefault>1</TableCellDefault><TableCellDefault>2</TableCellDefault></TableRowDefault></TableBody></Table><p>T18 untyped canonical on every table slot:
</p>
<Table class={"striped wide"} cols={4} id={"tbl18"} isDense={true} sortable={true} style={"--gap: 2rem"} title={"Title Foo"}><TableCaption class={"note"} id={"cap18"}>T18 caption <strong>b</strong></TableCaption><TableHead><TableRowDefault><TableCellDefault align={"left"} class={"v-top"} width={"200px"}>&#123;.h1&#125; H1</TableCellDefault><TableCellDefault align={"center"} class={"c"} id={"i"}>&#123;.h2&#125; H2</TableCellDefault><TableCellDefault align={"right"} class={"a b"} id={"i2"} k={"v"} n={2} width={"30%"}>H3</TableCellDefault><TableCellDefault align={"left"} b={true} class={"a"} flag={true} id={"i3"} k={"v"} n={1} style={"--v: 1"} title={"T"}>H4</TableCellDefault></TableRowDefault></TableHead><TableBody class={"tb"}><TableRowDefault><TableCellDefault align={"left"} class={"v-top lead"} width={"200px"}>X</TableCellDefault><TableCellDefault align={"center"} class={"c"} id={"i"}>15</TableCellDefault><TableCellDefault align={"right"} class={"a b"} id={"i2"} k={"v"} n={2} width={"30%"}><AnchorDefault class={"u"} href={"/docs"}>Ada</AnchorDefault></TableCellDefault><TableCellDefault align={"left"} b={true} class={"a"} flag={true} id={"i3"} k={"v"} n={1} style={"--v: 1"} title={"T"}><a class="w" href="/wiki/W" title="W">W</a></TableCellDefault></TableRowDefault><TableRowDefault class={"row-info"}><TableCellDefault align={"left"} class={"v-top"} colspan={2} width={"200px"}>plain</TableCellDefault><TableCellDefault align={"right"} class={"a b"} id={"i2"} k={"v"} n={2} width={"30%"}><Cite reference={frontmatter.references["paper-smith"]} class={"c"} index={1} /></TableCellDefault><TableCellDefault align={"left"} b={true} class={"a"} flag={true} id={"i3"} k={"v"} n={1} style={"--v: 1"} title={"T"}><MarkerA type={"footnote"} class={"m"}></MarkerA></TableCellDefault></TableRowDefault><TableRowDefault><TableCellDefault align={"left"} class={"v-top"} width={"200px"}>empty</TableCellDefault><TableCellDefault align={"center"} class={"c"} id={"i"}><AsideAB slug={""} title={""} type={"asideA"} class={"k"}> d</AsideAB></TableCellDefault><TableCellDefault align={"right"} class={"a b"} id={"i2"} k={"v"} n={2} width={"30%"}><span class="latex latex-inline" innerHTML={`<span class="katex"><span class="katex-mathml"><math xmlns="http://www.w3.org/1998/Math/MathML"><semantics><mrow><msup><mi>x</mi><mn>2</mn></msup></mrow><annotation encoding="application/x-tex">x^2</annotation></semantics></math></span><span class="katex-html" aria-hidden="true"><span class="base"><span class="strut" style="height:0.8141em;"></span><span class="mord"><span class="mord mathnormal">x</span><span class="msupsub"><span class="vlist-t"><span class="vlist-r"><span class="vlist" style="height:0.8141em;"><span style="top:-3.063em;margin-right:0.05em;"><span class="pstrut" style="height:2.7em;"></span><span class="sizing reset-size6 size3 mtight"><span class="mord mtight">2</span></span></span></span></span></span></span></span></span></span></span>`}></span></TableCellDefault><TableCellDefault align={"left"} b={true} class={"a"} flag={true} id={"i3"} k={"v"} n={1} style={"--v: 1"} title={"T"}><code>c</code></TableCellDefault></TableRowDefault><TableRowDefault class={"a"} id={"r"} k={"v"} n={2}><TableCellDefault align={"left"} class={"v-top"} width={"200px"}>a</TableCellDefault><TableCellDefault align={"center"} class={"c"} id={"i"}>b</TableCellDefault><TableCellDefault align={"right"} class={"a b"} id={"i2"} k={"v"} n={2} width={"30%"}>c</TableCellDefault><TableCellDefault align={"left"} b={true} class={"a"} flag={true} id={"i3"} k={"v"} n={1} style={"--v: 1"} title={"T"}>d</TableCellDefault></TableRowDefault><TableRowDefault bar={12} class={"extra class"} foo={"bar"} id={"id"} isBar={true} isFoo={true} style={"--style-var: 2rem"} title={"Title Foo"}><TableCellDefault align={"left"} class={"v-top"} width={"200px"}>all-row</TableCellDefault><TableCellDefault align={"center"} class={"c"} id={"i"}>x</TableCellDefault><TableCellDefault align={"right"} class={"a b"} id={"i2"} k={"v"} n={2} width={"30%"}>y</TableCellDefault><TableCellDefault align={"left"} b={true} class={"a"} flag={true} id={"i3"} k={"v"} n={1} style={"--v: 1"} title={"T"}>z</TableCellDefault></TableRowDefault></TableBody><TableFoot class={"total"}><TableRowDefault class={"f"}><TableCellDefault align={"left"} class={"v-top"} colspan={2} width={"200px"}>Total</TableCellDefault><TableCellDefault align={"left"} b={true} class={"a"} flag={true} id={"i3"} k={"v"} n={1} style={"--v: 1"} title={"T"}>1</TableCellDefault></TableRowDefault><TableRowDefault><TableCellDefault align={"left"} class={"v-top t"} width={"200px"}>F2</TableCellDefault><TableCellDefault align={"center"} class={"c"} id={"i"}>x</TableCellDefault><TableCellDefault align={"right"} class={"a b"} id={"i2"} k={"v"} n={2} width={"30%"}>y</TableCellDefault><TableCellDefault align={"left"} b={true} class={"a"} flag={true} id={"i3"} k={"v"} n={1} style={"--v: 1"} title={"T"}>z</TableCellDefault></TableRowDefault></TableFoot></Table><p>T19 type and brace separated by space: head stays literal:
</p>
<p>|-[t19]@@tableX &#123;.x&#125;-|
||@@captionX &#123;.y&#125; T19 caption ||
</p>
<Table><TableHead><TableRowDefault><TableCellDefault>A</TableCellDefault><TableCellDefault>B</TableCellDefault></TableRowDefault></TableHead><TableBody><TableRowDefault><TableCellDefault>@@cellA &#123;.z&#125; 1</TableCellDefault><TableCellDefault>2</TableCellDefault></TableRowDefault></TableBody></Table><p>T20 head not touching the pipe: stays literal (no exception for tables):
</p>
<p>|| &#123;.c&#125; T20 caption spaced ||
</p>
<Table><TableHead><TableRowDefault><TableCellDefault>A</TableCellDefault><TableCellDefault>B</TableCellDefault></TableRowDefault></TableHead><TableBody><TableRowDefault><TableCellDefault>&#123;.z&#125; spaced cell</TableCellDefault><TableCellDefault>@@cellA&#123;.y&#125; spaced typed cell</TableCellDefault></TableRowDefault></TableBody></Table><Table><TableHead><TableRowDefault><TableCellDefault align={"left"}>A</TableCellDefault><TableCellDefault>B</TableCellDefault></TableRowDefault></TableHead><TableBody><TableRowDefault><TableCellDefault align={"left"}>1</TableCellDefault><TableCellDefault>2</TableCellDefault></TableRowDefault></TableBody></Table><p>T22 rowspan (^ merges into the cell above):
</p>
<Table><TableHead><TableRowDefault><TableCellDefault>A</TableCellDefault><TableCellDefault>B</TableCellDefault><TableCellDefault>C</TableCellDefault></TableRowDefault></TableHead><TableBody><TableRowDefault><TableCellDefault rowspan={3}>r1</TableCellDefault><TableCellDefault>x</TableCellDefault><TableCellDefault>y</TableCellDefault></TableRowDefault><TableRowDefault><TableCellDefault>p</TableCellDefault><TableCellDefault>q</TableCellDefault></TableRowDefault><TableRowDefault><TableCellDefault>s</TableCellDefault><TableCellDefault>t</TableCellDefault></TableRowDefault></TableBody></Table><p>T23 colspan (&gt; merges into the cell on its right), also in the header:
</p>
<Table><TableHead><TableRowDefault><TableCellDefault>A</TableCellDefault><TableCellDefault><blockquote>
</blockquote>
</TableCellDefault><TableCellDefault>C</TableCellDefault></TableRowDefault></TableHead><TableBody><TableRowDefault><TableCellDefault colspan={2}>1</TableCellDefault><TableCellDefault>3</TableCellDefault></TableRowDefault><TableRowDefault><TableCellDefault>x</TableCellDefault><TableCellDefault>y</TableCellDefault><TableCellDefault>z</TableCellDefault></TableRowDefault></TableBody></Table><p>T24 chains and full-width spans:
</p>
<Table><TableHead><TableRowDefault><TableCellDefault>A</TableCellDefault><TableCellDefault>B</TableCellDefault><TableCellDefault>C</TableCellDefault><TableCellDefault>D</TableCellDefault></TableRowDefault></TableHead><TableBody><TableRowDefault><TableCellDefault colspan={2}></TableCellDefault><TableCellDefault>all four</TableCellDefault></TableRowDefault><TableRowDefault><TableCellDefault colspan={2}>1</TableCellDefault><TableCellDefault>three</TableCellDefault></TableRowDefault><TableRowDefault><TableCellDefault></TableCellDefault><TableCellDefault>2</TableCellDefault><TableCellDefault>3</TableCellDefault><TableCellDefault>4</TableCellDefault></TableRowDefault></TableBody></Table><p>T25 colspan and rowspan forming a 2x2 block:
</p>
<Table><TableHead><TableRowDefault><TableCellDefault>A</TableCellDefault><TableCellDefault>B</TableCellDefault><TableCellDefault>C</TableCellDefault></TableRowDefault></TableHead><TableBody><TableRowDefault><TableCellDefault rowspan={2}></TableCellDefault><TableCellDefault rowspan={2}>block 2x2</TableCellDefault><TableCellDefault>z</TableCellDefault></TableRowDefault><TableRowDefault><TableCellDefault>y</TableCellDefault></TableRowDefault><TableRowDefault><TableCellDefault>p</TableCellDefault><TableCellDefault>q</TableCellDefault><TableCellDefault>r</TableCellDefault></TableRowDefault></TableBody></Table><Table><TableHead><TableRowDefault><TableCellDefault>A</TableCellDefault><TableCellDefault>B</TableCellDefault><TableCellDefault>^</TableCellDefault></TableRowDefault></TableHead><TableBody><TableRowDefault><TableCellDefault></TableCellDefault><TableCellDefault colspan={2}>1</TableCellDefault></TableRowDefault><TableRowDefault><TableCellDefault>a</TableCellDefault><TableCellDefault>b</TableCellDefault><TableCellDefault>c</TableCellDefault></TableRowDefault></TableBody></Table><Table><TableHead><TableRowDefault><TableCellDefault>A</TableCellDefault><TableCellDefault>B</TableCellDefault><TableCellDefault>C</TableCellDefault></TableRowDefault></TableHead><TableBody><TableRowDefault><TableCellDefault colspan={2}>1</TableCellDefault><TableCellDefault>3</TableCellDefault></TableRowDefault><TableRowDefault><TableCellDefault rowspan={2}></TableCellDefault><TableCellDefault>4</TableCellDefault></TableRowDefault><TableRowDefault></TableRowDefault></TableBody></Table><p>T28 lookalikes that must stay literal text:
</p>
<Table><TableHead><TableRowDefault><TableCellDefault>a &gt; b</TableCellDefault><TableCellDefault><blockquote>
=</blockquote>
</TableCellDefault><TableCellDefault><blockquote>
<blockquote>
</blockquote>
</blockquote>
</TableCellDefault><TableCellDefault>x^2</TableCellDefault><TableCellDefault>^^</TableCellDefault><TableCellDefault>\&gt;</TableCellDefault><TableCellDefault>\^</TableCellDefault><TableCellDefault><blockquote>
x</blockquote>
</TableCellDefault><TableCellDefault>^ y</TableCellDefault></TableRowDefault></TableHead><TableBody><TableRowDefault><TableCellDefault>1</TableCellDefault><TableCellDefault>2</TableCellDefault><TableCellDefault>3</TableCellDefault><TableCellDefault>4</TableCellDefault><TableCellDefault>5</TableCellDefault><TableCellDefault>6</TableCellDefault><TableCellDefault>7</TableCellDefault><TableCellDefault>8</TableCellDefault><TableCellDefault>9</TableCellDefault></TableRowDefault></TableBody></Table><Table><TableHead><TableRowDefault><TableCellDefault>A</TableCellDefault><TableCellDefault>B</TableCellDefault><TableCellDefault>C</TableCellDefault></TableRowDefault></TableHead><TableBody><TableRowDefault><TableCellDefault>1</TableCellDefault><TableCellDefault>2</TableCellDefault><TableCellDefault>3</TableCellDefault></TableRowDefault><TableRowDefault><TableCellDefault rowspan={3}>===</TableCellDefault><TableCellDefault></TableCellDefault><TableCellDefault></TableCellDefault></TableRowDefault><TableRowDefault><TableCellDefault>t</TableCellDefault></TableRowDefault><TableRowDefault><TableCellDefault>x</TableCellDefault><TableCellDefault>y</TableCellDefault></TableRowDefault></TableBody></Table><Table><TableHead><TableRowDefault><TableCellDefault>A</TableCellDefault><TableCellDefault>B</TableCellDefault><TableCellDefault>C</TableCellDefault></TableRowDefault></TableHead><TableBody><TableRowDefault><TableCellDefault></TableCellDefault><TableCellDefault>x</TableCellDefault><TableCellDefault>y</TableCellDefault></TableRowDefault><TableRowDefault><TableCellDefault class={"s"} rowspan={2}>@@cellB&#123;&#125; &gt;</TableCellDefault><TableCellDefault>x</TableCellDefault><TableCellDefault>y</TableCellDefault></TableRowDefault><TableRowDefault><TableCellDefault>x</TableCellDefault><TableCellDefault>y</TableCellDefault></TableRowDefault><TableRowDefault><TableCellDefault class={"s"} colspan={2}>a</TableCellDefault><TableCellDefault>y</TableCellDefault></TableRowDefault><TableRowDefault><TableCellDefault></TableCellDefault><TableCellDefault>@@cellB&#123;.span&#125; merged</TableCellDefault><TableCellDefault>z</TableCellDefault></TableRowDefault></TableBody></Table><p>T31 a cell starting with a marker touching the pipe (head vs marker, edge):
</p>
<Table><TableHead><TableRowDefault><TableCellDefault>A</TableCellDefault><TableCellDefault>B</TableCellDefault><TableCellDefault>C</TableCellDefault></TableRowDefault></TableHead><TableBody><TableRowDefault><TableCellDefault><MarkerA type={"footnote"} class={"m"}></MarkerA> x</TableCellDefault><TableCellDefault><MarkerA type={"markerA"}></MarkerA> y</TableCellDefault><TableCellDefault>z</TableCellDefault></TableRowDefault><TableRowDefault><TableCellDefault><MarkerA type={"footnote"}></MarkerA></TableCellDefault><TableCellDefault>w</TableCellDefault><TableCellDefault>v</TableCellDefault></TableRowDefault></TableBody></Table><HeadingDefault id={"s-quote"} level={2} number={"2.8."} raw_title={"Blockquote"}>Blockquote
</HeadingDefault><blockquote>
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
<section id="q14-heading-inside">
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
<pre lang="ts"><code innerHTML={"<p><b>&gt;</b> const q <b>=</b> <i><i>&quot;</i>@@bqA{.literal}<i>&quot;</i></i>;</p><p><b>&gt;</b> <i><i>`</i><i>`</i></i><i><i>`</i></p><p>&gt;</p><p>&gt; $$x = y$$</p><p>&gt;</p><p>&gt; </p><p>&#8203;</p><p>&#8203;</p><p>&gt; @@bqA{.q15} KS a **b** _i_ <i>`</i></i><i>c<i>`</i> W    d $x$</p><p>&#8203;</p><p>&#8203;</p><p>&gt; {} Q16 untyped empty</p><p>&#8203;</p><p>&#8203;</p><p>&gt; {.x, #i} Q17 untyped XY</p><p>&#8203;</p><p>&gt; {.a, .b, #i, k: &quot;v&quot;, n: 2} Q18 untyped XXYZZ</p><p>&#8203;</p><p>&gt; {<i>`</i></i>slug<b>-</b><i>foo<i>`</i>, &quot;Title Foo&quot;, .extra, .class, #id, foo: &quot;bar&quot;, bar: 12, isFoo: true, --style-var: &quot;2rem&quot;, isBar} Q19 untyped all</p><p>&#8203;</p><p>&gt; {.x} Q20 untyped, no space after &gt;</p><p>&#8203;</p><p>&gt; Q21 not first {.x} literal</p><p>&#8203;</p><p>&gt; {.x}</p><p>&#8203;</p><p>&gt; @@bqA {.x} Q23 type and brace separated by space (literal)</p><p>&#8203;</p><p>&gt; {.q24} KSu a **b** _i_ <i>`</i></i><i>c<i>`</i> W    d $x$</p><p>&#8203;</p><p>&#8203;</p><p>List</p><p>&#8203;</p><p>&lt;!-- PENDING PHASE 4: plugin-list does not exist yet. Expected output of this whole section = plain markdown/literal text, frozen as such. Re-review when implemented. --&gt;</p><p>&#8203;</p><p>L01</p><p>&#8203;</p><p>- L01 plain</p><p>  - L01a</p><p>- L01b</p><p>&#8203;</p><p>L02</p><p>&#8203;</p><p>{.u}</p><p>&#8203;</p><p>- L02 decorator untyped X</p><p>- L02b</p><p>&#8203;</p><p>L02n</p><p>&#8203;</p><p>@@{.u}</p><p>&#8203;</p><p>- L02n decorator non-canonical</p><p>- L02nb</p><p>&#8203;</p><p>L03</p><p>&#8203;</p><p>@@unorderedA{.u, #l03}</p><p>&#8203;</p><p>- L03 star, XY</p><p>- L03b</p><p>&#8203;</p><p>L04</p><p>&#8203;</p><p>@@compact{.a, .b, #l04, k: &quot;v&quot;, n: 2}</p><p>&#8203;</p><p>- L04 plus, XXYZZ</p><p>- L04b</p><p>&#8203;</p><p>L05</p><p>&#8203;</p><p>@@unorderedA{<i>`</i></i>slug<b>-</b><i>foo<i>`</i>, &quot;Title Foo&quot;, .extra, .class, #id, foo: &quot;bar&quot;, bar: 12, isFoo: true, --style-var: &quot;2rem&quot;, isBar}</p><p>&#8203;</p><p>- L05 all</p><p>- L05b</p><p>&#8203;</p><p>L06</p><p>&#8203;</p><p>@@unorderedZ{.u}</p><p>&#8203;</p><p>- L06 unclaimed</p><p>&#8203;</p><p>L07</p><p>&#8203;</p><p>{}</p><p>&#8203;</p><p>- L07 empty</p><p>&#8203;</p><p>L07n</p><p>&#8203;</p><p>@@{}</p><p>&#8203;</p><p>- L07n empty non-canonical</p><p>&#8203;</p><p>L08</p><p>&#8203;</p><p>@@unorderedA</p><p>&#8203;</p><p>- L08 type only</p><p>&#8203;</p><p>L10</p><p>&#8203;</p><p>- @@liItem{<i>`</i></i><i>alpha<i>`</i>} L10 typed item</p><p>  - {<i>`</i></i><i>beta<i>`</i>} L10a nested untyped</p><p>  - @@liItem{.a, #i} L10b XY</p><p>- @@check{.c, k: &quot;v&quot;, n: 2} L11 XXYZZ</p><p>- @@data{<i>`</i></i>slug<b>-</b><i>foo<i>`</i>, &quot;Title Foo&quot;, .extra, .class, #id, foo: &quot;bar&quot;, bar: 12, isFoo: true, --style-var: &quot;2rem&quot;, isBar} L12 all</p><p>- @@liZ{.z} L13 unclaimed</p><p>- {} L14 empty</p><p>- @@{} L14n empty non-canonical</p><p>- @@liItem L15 type only</p><p>- L16 plain among extras</p><p>- {.only}</p><p>- @@liItem{.x}L17 glued</p><p>- {.x}L17u glued untyped</p><p>- @@liItem {.x} L17s type and brace separated by space (literal)</p><p>&#8203;</p><p>L20</p><p>&#8203;</p><p>{<i>`</i></i>my<b>-</b><i>list<i>`</i>}</p><p>6. L20 start at 6</p><p>7. L20b</p><p>&#8203;</p><p>L20n</p><p>&#8203;</p><p>@@{<i>`</i></i>my<b>-</b>list<b>-</b><i>n<i>`</i>}</p><p>6. L20n start at 6 non-canonical</p><p>7. L20nb</p><p>&#8203;</p><p>L21</p><p>&#8203;</p><p>1. L21 plain ol</p><p>2. L21b</p><p>&#8203;</p><p>L22</p><p>&#8203;</p><p>@@orderedA{.o, #l22}</p><p>&#8203;</p><p>1. {.i} L22 container+item</p><p>2. @@liItem{.a, .b, #i2, k: &quot;v&quot;, n: 2} L22b</p><p>&#8203;</p><p>L22n</p><p>&#8203;</p><p>@@{.o}</p><p>&#8203;</p><p>1. @@{.i} L22n non-canonical container+item</p><p>&#8203;</p><p>L24</p><p>&#8203;</p><p>0. L24 start 0</p><p>1. L24b</p><p>&#8203;</p><p>L25</p><p>&#8203;</p><p>{start: 3}</p><p>&#8203;</p><p>1. L25 start via extras (expect Warning)</p><p>&#8203;</p><p>L25n</p><p>&#8203;</p><p>@@{start: 3}</p><p>&#8203;</p><p>1. L25n start via non-canonical extras (expect Warning)</p><p>&#8203;</p><p>L26</p><p>&#8203;</p><p>o. L26 retired o. syntax (expect plain paragraph)</p><p>&#8203;</p><p>L30</p><p>&#8203;</p><p>@@unorderedA{.l1}</p><p>&#8203;</p><p>- L30 level 1</p><p>  @@orderedA{.l2}</p><p>  1. L30a decorator right under item text (edge)</p><p>     - L30a1</p><p>  2. L30b</p><p>- L33 level 1</p><p>&#8203;</p><p>  @@orderedA{.l2b}</p><p>  1. L33a decorator after blank, inside item</p><p>  2. L33b</p><p>&#8203;</p><p>L34</p><p>&#8203;</p><p>- L34 multi-paragraph</p><p>&#8203;</p><p>  second paragraph KS a **b** W    d</p><p>&#8203;</p><p>&#8203;</p><p>  <i>`</i></i><i><i>`</i><i>`</i></i>ts</p><p>  const l <b>=</b> <i><i>&quot;</i>@@{.literal}<i>&quot;</i></i>;</p>"} /></pre>
<ul>
<li>L35 table in item<Table><TableHead><TableRowDefault><TableCellDefault>A</TableCellDefault><TableCellDefault>B</TableCellDefault></TableRowDefault></TableHead><TableBody><TableRowDefault><TableCellDefault>1</TableCellDefault><TableCellDefault>2</TableCellDefault></TableRowDefault></TableBody></Table></li>
<li>L36 quote in item<blockquote>
<p>@@bqA&#123;.in-li&#125; quoted
</p>
</blockquote>
</li>
<li>L37 directive in item</li>
</ul>
</section>
<section>
<h1>===asideA</h1>
</section>
</blockquote>
</section>

</>); }
