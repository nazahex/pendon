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
<Figure baz={23} class="extra class or" container="figure" foo="bar" id="custom-id" style="--wix: sum; --rotate: 5deg;"><AdvancedImage alt="lorem ipsum" async_decoding={1} height={300} lazy={1} src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp" width={800}></AdvancedImage>Exercitation qui <strong>exercitation</strong> dolor velit <i>aliqua</i> <a href="/Anim_Esta_(Officia)" title="Anim Esta (Officia)">Anim</a> consectetur <a href="" title="suryana-2026">^^</a> voluptate <a href="/foo/bar" rel="noopener noreferrer sponsored nofollow" target="_blank" title="Buy Foo!">labore</a> labore elit non esse occaecat. <a href="suryana-2026" title="hlm. 45">^^</a></Figure><p>Officia amet cillum reprehenderit id pariatur commodo ullamco ad ipsum reprehenderit <a href="/Anim_Esta_(Officia)" title="Anim Esta (Officia)">Anim</a> proident reprehenderit. <a href="" title="suryana-2026&quot;, &quot;hlm. 45">^^</a>
</p>
</li>
</ul>

</>); }
