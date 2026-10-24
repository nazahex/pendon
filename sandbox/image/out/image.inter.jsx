import AdvancedImage from '@comp/content/AdvancedImage'
import Figure from '@comp/content/Figure'

export const references = {};
export const citations = [];
export default function PendonView() { return (<>
<hr />
<p>title: &quot;Lapiran Sintaks&quot;
references:
</p>
<h1>1. Contoh Buku</h1>
<p>suryana-2026:
id: suryana-2026
type: book
title: Masa Depan Rekayasa Perangkat Lunak
authors:
</p>
<ul>
<li>firstName: Eko<p>lastName: Suryana
publisher: TechPress Indonesia
publisherLocation: Jakarta
issuedDate:
year: 2026
isbn: 978-602-0000-00-0
language: id
<hr />
</p>
<p>Esse exercitation irure ullamco veniam fugiat eu consequat proident quis do.
</p>
<Figure container="figure"><AdvancedImage alt="lorem ipsum" async_decoding={1} height={300} lazy={1} src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp" width={800}></AdvancedImage>[.extra,.class,.or,#custom-id]&#123;foo:&quot;bar&quot;,baz:23,--wix:&quot;sum&quot;,--rotate:&quot;5deg&quot;&#125;Exercitationqui<strong>exercitation</strong>dolorvelit<i>aliqua</i>[[AnimEsta(Officia)|Anim]]consectetur<a href="" title="suryana-2026">^^</a>voluptate<a href="/foo/bar" rel="noopener noreferrer sponsored nofollow" target="_blank" title="Buy Foo!">labore</a>laboreelitnonesseoccaecat.<a href="suryana-2026" title="hlm. 45">^^</a></Figure><p>Officia amet cillum reprehenderit id pariatur commodo ullamco ad ipsum reprehenderit [[Anim Esta (Officia) | Anim]] proident reprehenderit. <a href="" title="suryana-2026&quot;, &quot;hlm. 45">^^</a>
</p>
</li>
</ul>

</>); }
