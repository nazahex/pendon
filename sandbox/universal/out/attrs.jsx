import Cite from "@comp/content/Cite";
import Hint from "@comp/shared/Hint";
import Parego from "@comp/shared/Parego";
import Epis from '@comp/content/Epis'

export const frontmatter = {"cites":[{"id":"suryana-2026","index":1,"loc":"hlm. 45"}],"references":{"suryana-2026":{"authors":[{"firstName":"Eko","lastName":"Suryana"}],"id":"suryana-2026","isbn":"978-602-0000-00-0","issuedDate":{"year":2026},"language":"id","publisher":"TechPress Indonesia","publisherLocation":"Jakarta","title":"Masa Depan Rekayasa Perangkat Lunak","type":"book"}},"title":"Universal Custom Syntax"};
export const headings = [{"id":"custom-plugins","text":"Custom Plugins","level":2},{"id":"joo","text":"Foo","level":2}];
export default function PendonView() { return (<>
<section id="custom-plugins">
<h2>1. Custom Plugins</h2>
<Parego type="note" title="Foo Bar" tag={"Bax Xo"}><p>Ullamco excepteur adipisicing quis ullamco ea mollit et nulla sint non et id commodo commodo <Cite reference={frontmatter.references["suryana-2026"]} cite-id={"surya"} class={"suyn"} index={1} loc={"hlm. 45"} rox={"hen"} /><Epis doo={"bax"} level={3}>Aliquip commodo commodo et ut reprehenderit qui magna laboris et.</Epis>.
</p>
<Hint type="!"><p>In amet deserunt consequat cupidatat laboris cupidatat.
</p>
</Hint><figure class="extra class or" data-baz="23" data-foo="bar" id="custom-id" style="--wix: sum; --rotate: 5deg;"><img alt="lorem ipsum" decoding="async" loading="lazy" src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp" /><figcaption>Proident ex incididunt non sunt ad deserunt proident ex et in fugiat.</figcaption></figure>
</Parego></section>
<section id="joo">
<h2 gaz="12">2. Foo</h2>
<p>Lorem est reprehenderit do ut ut laborum nisi nostrud eiusmod eu deserunt aliquip.
</p>
</section>

</>); }
