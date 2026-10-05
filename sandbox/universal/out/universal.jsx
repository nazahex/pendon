import Cite from "@comp/content/Cite"
import Epis from "@comp/content/Epis"
import Hint from "@comp/shared/Hint"
import Parego from "@comp/shared/Parego"

export const frontmatter = {
  cites: [
    { id: "suryana-2026", index: 1 },
    { id: "suryana-2026", index: 2, loc: "hlm. 45" },
    { id: "paper-smith", index: 3 },
  ],
  references: {
    "paper-smith": {
      authors: [
        { firstName: "John", lastName: "Smith" },
        { firstName: "Jane", lastName: "Doe" },
      ],
      containerTitle: "Journal of Web Engineering",
      doi: "10.1016/j.jwe.2025.08.001",
      id: "paper-smith",
      issue: "4",
      issuedDate: { month: 8, year: 2025 },
      language: "en",
      pages: "210-225",
      title: "Generative MDX to PDF Compilation Architectures",
      type: "journal",
      volume: "18",
    },
    "suryana-2026": {
      authors: [{ firstName: "Eko", lastName: "Suryana" }],
      id: "suryana-2026",
      isbn: "978-602-0000-00-0",
      issuedDate: { year: 2026 },
      language: "id",
      publisher: "TechPress Indonesia",
      publisherLocation: "Jakarta",
      title: "Masa Depan Rekayasa Perangkat Lunak",
      type: "book",
    },
  },
  title: "Universal Custom Syntax",
}
export const headings = [
  { id: "image", text: "Image", level: 2 },
  { id: "anchor", text: "Anchor", level: 2 },
  { id: "cite", text: "Cite", level: 2 },
  {
    id: "heading-slug",
    text: "Heading",
    level: 2,
    subheadings: [{ id: "foo-bar", text: "Foo Bar Barosa", level: 3 }],
  },
  { id: "table", text: "Table", level: 2 },
  { id: "latex", text: "Latex", level: 2 },
  { id: "custom-plugins", text: "Custom Plugins", level: 2 },
  { id: "lists", text: "Lists", level: 2 },
]
export default function PendonView() {
  return (
    <>
      <section id="image">
        <h2>1. Image</h2>
        <figure
          class="extra class or"
          data-baz="23"
          data-foo="bar"
          id="custom-id"
          style="--wix:sum;--rotate:5deg;">
          <img
            alt="lorem ipsum"
            decoding="async"
            height="300"
            loading="lazy"
            src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp"
            width="800"
          />
          <figcaption>
            Exercitation qui <strong>exercitation</strong>{" "}
            <span
              class="latex latex-inline"
              innerHTML={`<span class="katex"><span class="katex-mathml"><math xmlns="http://www.w3.org/1998/Math/MathML"><semantics><mrow><mi>d</mi><mi>o</mi><mi>l</mi><mi>o</mi><mi>r</mi></mrow><annotation encoding="application/x-tex">dolor</annotation></semantics></math></span><span class="katex-html" aria-hidden="true"><span class="base"><span class="strut" style="height:0.6944em;"></span><span class="mord mathnormal">d</span><span class="mord mathnormal">o</span><span class="mord mathnormal" style="margin-right:0.01968em;">l</span><span class="mord mathnormal" style="margin-right:0.02778em;">or</span></span></span></span>`}></span>{" "}
            &#123;velit&#125; <i>aliqua</i>{" "}
            <a href="/Anim_Esta_(Officia)" title="Anim Esta (Officia)">
              Anim
            </a>{" "}
            consectetur <Cite reference={frontmatter.references["suryana-2026"]} index={1} />{" "}
            voluptate{" "}
            <a
              href="/foo/bar"
              rel="noopener noreferrer sponsored nofollow"
              target="_blank"
              title="Buy Foo!">
              labore
            </a>{" "}
            labore elit non esse occaecat.{" "}
            <Cite reference={frontmatter.references["suryana-2026"]} index={2} loc={"hlm. 45"} />{" "}
            <Epis level={2}>Et deserunt sunt consectetur elit.</Epis>
          </figcaption>
        </figure>
      </section>
      <section id="anchor">
        <h2>2. Anchor</h2>
        <p>
          <a
            class="extra qur"
            href="https://foo.com/bar"
            hreflang="en"
            id="rew"
            qux="rox"
            rel="noopener noreferrer nofollow sponsored prefetch"
            target="_self"
            title="Title Foo">
            consectetur
          </a>{" "}
          Incididunt &#123;cupidatat&#125; quis et pariatur commodo laborum consectetur anim do
          minim anim in.
        </p>
      </section>
      <section id="cite">
        <h2>3. Cite</h2>
        <p>
          Minim{" "}
          <Cite
            reference={frontmatter.references["paper-smith"]}
            cite-id={"smith"}
            class={"paper"}
            index={3}
            rox={"hen"}
          />{" "}
          esse do ut anim proident est qui{" "}
          <span
            class="latex latex-inline"
            innerHTML={`<span class="katex"><span class="katex-mathml"><math xmlns="http://www.w3.org/1998/Math/MathML"><semantics><mrow><mi>F</mi></mrow><annotation encoding="application/x-tex">F</annotation></semantics></math></span><span class="katex-html" aria-hidden="true"><span class="base"><span class="strut" style="height:0.6833em;"></span><span class="mord mathnormal" style="margin-right:0.13889em;">F</span></span></span></span>`}></span>{" "}
          magna non elit quis eiusmod dolore.
        </p>
      </section>
      <section id="heading-slug">
        <h2>4. Heading</h2>
        <section id="foo-bar">
          <h3 class="hoo" qun="anu">
            4.1. Foo Bar Barosa
          </h3>
        </section>
      </section>
      <section id="table">
        <h2>5. Table</h2>
        <table class="striped foo" id="sales-table" qux="true" sortable="true">
          <caption>
            Laporan{" "}
            <a href="https://foo.com" rel="noopener" target="_blank" title="Foo">
              Penjualan
            </a>{" "}
            2026{" "}
            <a href="/Foo" title="Foo">
              Foo Bar
            </a>
          </caption>
          <thead>
            <tr>
              <th class="v-top" style="text-align: left; width: 200px;">
                Produk
              </th>
              <th class="v-top" style="text-align: center;">
                Stok
              </th>
              <th class="v-bottom" style="text-align: right; width: 30%;">
                Harga
              </th>
              <th style="text-align: center;">Status</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td class="v-top" style="text-align: left; width: 200px;">
                <a href="/Laptop_(Pro)" title="Laptop (Pro)">
                  Pro
                </a>
              </td>
              <td class="v-top" style="text-align: center;">
                Foo{" "}
                <img
                  alt="Aternative Text"
                  class="extra class or"
                  data-baz="23"
                  data-foo="bar"
                  id="custom-id"
                  src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp"
                  style="--wix:sum;--rotate:5deg;"
                />{" "}
                Bar
              </td>
              <td class="v-bottom" style="text-align: right; width: 30%;">
                15.000.000
              </td>
              <td style="text-align: center;">
                <a href="/foo" rel="nofollow sponsored" target="_self">
                  Tersedia
                </a>
              </td>
            </tr>
            <tr>
              <td class="v-top" colspan="2" style="text-align: left; width: 200px;">
                Mouse{" "}
                <a href="/Wireless" title="Wireless">
                  Wireless
                </a>
              </td>
              <td class="v-bottom" rox="rox" style="text-align: right; width: 30%;">
                250.000
              </td>
              <td style="text-align: center;">
                <img
                  alt="Aternative Text"
                  class="extra class or"
                  data-baz="23"
                  data-foo="bar"
                  id="custom-id"
                  src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp"
                  style="--wix:sum;--rotate:5deg;"
                />
              </td>
            </tr>
            <tr class="row-danger">
              <td class="v-top" rowspan="2" style="text-align: left; width: 200px;">
                &#123;Keyboard&#125; Mekanikal
                <Cite reference={frontmatter.references["suryana-2026"]} index={1} />
              </td>
              <td class="v-top text-red" style="text-align: center;">
                0
              </td>
              <td class="v-bottom" style="text-align: right; width: 30%;">
                <span
                  class="latex latex-inline"
                  innerHTML={`<span class="katex"><span class="katex-mathml"><math xmlns="http://www.w3.org/1998/Math/MathML"><semantics><mrow><mn>850.000</mn></mrow><annotation encoding="application/x-tex">850.000</annotation></semantics></math></span><span class="katex-html" aria-hidden="true"><span class="base"><span class="strut" style="height:0.6444em;"></span><span class="mord">850.000</span></span></span></span>`}></span>
              </td>
              <td style="text-align: center;">Habis</td>
            </tr>
            <tr>
              <td class="v-top" style="text-align: center;">
                5
              </td>
              <td class="v-bottom" style="text-align: right; width: 30%;">
                5.200.000
              </td>
              <td style="text-align: center;">
                <img
                  alt="lorem ipsum"
                  class="extra class or"
                  data-baz="23"
                  data-foo="bar"
                  decoding="async"
                  height="300"
                  id="custom-id"
                  loading="lazy"
                  src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp"
                  style="--wix:sum;--rotate:5deg;"
                  width="800"
                />
                Exercitation qui <strong>exercitation</strong> dolor velit <i>aliqua</i>{" "}
                <a href="/Anim_Esta_(Officia)" title="Anim Esta (Officia)">
                  Anim
                </a>{" "}
                consectetur <Cite reference={frontmatter.references["suryana-2026"]} index={1} />{" "}
                voluptate{" "}
                <a
                  href="/foo/bar"
                  rel="noopener noreferrer sponsored nofollow"
                  target="_blank"
                  title="Buy Foo!">
                  labore
                </a>{" "}
                labore elit non esse occaecat.{" "}
                <Cite
                  reference={frontmatter.references["suryana-2026"]}
                  index={2}
                  loc={"hlm. 45"}
                />
              </td>
            </tr>
          </tbody>
          <tfoot>
            <tr>
              <td class="v-top" colspan="2" style="text-align: left; width: 200px;">
                Total Inventaris
              </td>
              <td class="v-bottom" style="text-align: right; width: 30%;">
                21.300.000
              </td>
              <td style="text-align: center;">-</td>
            </tr>
          </tfoot>
        </table>
        <p>
          <Cite reference={frontmatter.references["suryana-2026"]} index={1} />
        </p>
      </section>
      <section id="latex">
        <h2>6. Latex</h2>
        <p>
          <span
            class="latex latex-block"
            style="display: block;"
            innerHTML={`<span class="katex-display"><span class="katex"><span class="katex-mathml"><math xmlns="http://www.w3.org/1998/Math/MathML" display="block"><semantics><mrow><mo stretchy="false">(</mo><mo stretchy="false">(</mo><mi>H</mi><mo>→</mo><mi>O</mi><mo stretchy="false">)</mo><mo>∧</mo><mi mathvariant="normal">¬</mi><mi>O</mi><mo stretchy="false">)</mo><mo>→</mo><mi mathvariant="normal">¬</mi><mi>H</mi></mrow><annotation encoding="application/x-tex">((H \\rightarrow O) \\land \\lnot O) \\rightarrow \\lnot H</annotation></semantics></math></span><span class="katex-html" aria-hidden="true"><span class="base"><span class="strut" style="height:1em;vertical-align:-0.25em;"></span><span class="mopen">((</span><span class="mord mathnormal" style="margin-right:0.08125em;">H</span><span class="mspace" style="margin-right:0.2778em;"></span><span class="mrel">→</span><span class="mspace" style="margin-right:0.2778em;"></span></span><span class="base"><span class="strut" style="height:1em;vertical-align:-0.25em;"></span><span class="mord mathnormal" style="margin-right:0.02778em;">O</span><span class="mclose">)</span><span class="mspace" style="margin-right:0.2222em;"></span><span class="mbin">∧</span><span class="mspace" style="margin-right:0.2222em;"></span></span><span class="base"><span class="strut" style="height:1em;vertical-align:-0.25em;"></span><span class="mord">¬</span><span class="mord mathnormal" style="margin-right:0.02778em;">O</span><span class="mclose">)</span><span class="mspace" style="margin-right:0.2778em;"></span><span class="mrel">→</span><span class="mspace" style="margin-right:0.2778em;"></span></span><span class="base"><span class="strut" style="height:0.6833em;"></span><span class="mord">¬</span><span class="mord mathnormal" style="margin-right:0.08125em;">H</span></span></span></span></span>`}></span>
        </p>
        <p>
          Rumus ini dibaca: bila{" "}
          <span
            class="latex latex-inline"
            innerHTML={`<span class="katex"><span class="katex-mathml"><math xmlns="http://www.w3.org/1998/Math/MathML"><semantics><mrow><mi>H</mi></mrow><annotation encoding="application/x-tex">H</annotation></semantics></math></span><span class="katex-html" aria-hidden="true"><span class="base"><span class="strut" style="height:0.6833em;"></span><span class="mord mathnormal" style="margin-right:0.08125em;">H</span></span></span></span>`}></span>{" "}
          mengimplikasikan{" "}
          <span
            class="latex latex-inline"
            innerHTML={`<span class="katex"><span class="katex-mathml"><math xmlns="http://www.w3.org/1998/Math/MathML"><semantics><mrow><mi>O</mi></mrow><annotation encoding="application/x-tex">O</annotation></semantics></math></span><span class="katex-html" aria-hidden="true"><span class="base"><span class="strut" style="height:0.6833em;"></span><span class="mord mathnormal" style="margin-right:0.02778em;">O</span></span></span></span>`}></span>{" "}
          dan{" "}
          <span
            class="latex latex-inline"
            innerHTML={`<span class="katex"><span class="katex-mathml"><math xmlns="http://www.w3.org/1998/Math/MathML"><semantics><mrow><mi>O</mi></mrow><annotation encoding="application/x-tex">O</annotation></semantics></math></span><span class="katex-html" aria-hidden="true"><span class="base"><span class="strut" style="height:0.6833em;"></span><span class="mord mathnormal" style="margin-right:0.02778em;">O</span></span></span></span>`}></span>{" "}
          tidak terjadi, maka{" "}
          <span
            class="latex latex-inline"
            innerHTML={`<span class="katex"><span class="katex-mathml"><math xmlns="http://www.w3.org/1998/Math/MathML"><semantics><mrow><mi>H</mi></mrow><annotation encoding="application/x-tex">H</annotation></semantics></math></span><span class="katex-html" aria-hidden="true"><span class="base"><span class="strut" style="height:0.6833em;"></span><span class="mord mathnormal" style="margin-right:0.08125em;">H</span></span></span></span>`}></span>{" "}
          tidak benar. Kebalikannya tidak sah: dari{" "}
          <span
            class="latex latex-inline"
            innerHTML={`<span class="katex"><span class="katex-mathml"><math xmlns="http://www.w3.org/1998/Math/MathML"><semantics><mrow><mi>H</mi><mo>→</mo><mi>O</mi></mrow><annotation encoding="application/x-tex">H \\rightarrow O</annotation></semantics></math></span><span class="katex-html" aria-hidden="true"><span class="base"><span class="strut" style="height:0.6833em;"></span><span class="mord mathnormal" style="margin-right:0.08125em;">H</span><span class="mspace" style="margin-right:0.2778em;"></span><span class="mrel">→</span><span class="mspace" style="margin-right:0.2778em;"></span></span><span class="base"><span class="strut" style="height:0.6833em;"></span><span class="mord mathnormal" style="margin-right:0.02778em;">O</span></span></span></span>`}></span>{" "}
          dan{" "}
          <span
            class="latex latex-inline"
            innerHTML={`<span class="katex"><span class="katex-mathml"><math xmlns="http://www.w3.org/1998/Math/MathML"><semantics><mrow><mi>O</mi></mrow><annotation encoding="application/x-tex">O</annotation></semantics></math></span><span class="katex-html" aria-hidden="true"><span class="base"><span class="strut" style="height:0.6833em;"></span><span class="mord mathnormal" style="margin-right:0.02778em;">O</span></span></span></span>`}></span>{" "}
          terjadi, kita tidak boleh menyimpulkan{" "}
          <span
            class="latex latex-inline"
            innerHTML={`<span class="katex"><span class="katex-mathml"><math xmlns="http://www.w3.org/1998/Math/MathML"><semantics><mrow><mi>H</mi></mrow><annotation encoding="application/x-tex">H</annotation></semantics></math></span><span class="katex-html" aria-hidden="true"><span class="base"><span class="strut" style="height:0.6833em;"></span><span class="mord mathnormal" style="margin-right:0.08125em;">H</span></span></span></span>`}></span>{" "}
          pasti benar, sebab &#123;hipotesis&#125; lain bisa menghasilkan{" "}
          <span
            class="latex latex-inline"
            innerHTML={`<span class="katex"><span class="katex-mathml"><math xmlns="http://www.w3.org/1998/Math/MathML"><semantics><mrow><mi>O</mi></mrow><annotation encoding="application/x-tex">O</annotation></semantics></math></span><span class="katex-html" aria-hidden="true"><span class="base"><span class="strut" style="height:0.6833em;"></span><span class="mord mathnormal" style="margin-right:0.02778em;">O</span></span></span></span>`}></span>{" "}
          yang sama. Karena itu, hipotesis yang lolos uji lazimnya disebut didukung, bukan terbukti.
        </p>
      </section>
      <section id="custom-plugins">
        <h2>7. Custom Plugins</h2>
        <Parego type="note" title="Foo Bar" tag={"Bax Xo"}>
          <p>
            Ullamco excepteur adipisicing quis ullamco ea mollit et nulla sint non et id commodo
            commodo{" "}
            <Cite
              reference={frontmatter.references["suryana-2026"]}
              cite-id={"surya"}
              class={"suyn"}
              index={2}
              loc={"hlm. 45"}
              rox={"hen"}
            />
            <Epis level={3}>Aliquip commodo commodo et ut reprehenderit qui magna laboris et.</Epis>
            .
          </p>
          <Hint type="!">
            <p>In amet deserunt consequat cupidatat laboris cupidatat.</p>
          </Hint>
        </Parego>
        <Hint type="?">
          <p>
            Ea reprehenderit sint exercitation velit quis officia aliqua est consectetur tempor
            cupidatat qui sint.
          </p>
          <ul>
            <li>Est deserunt excepteur minim in ad nisi.</li>
            <li>Est deserunt excepteur minim in ad nisi.</li>
          </ul>
        </Hint>
        <Parego type="error" title="Bux">
          <ul>
            <li>Ad excepteur nulla amet cupidatat aliqua.</li>
            <li>
              Aliqua nostrud mollit cillum reprehenderit deserunt nostrud voluptate eu duis officia
              incididunt excepteur.
            </li>
            <li>Velit adipisicing officia mollit aliquip anim cupidatat aute ad labore.</li>
          </ul>
        </Parego>
      </section>
      <section id="lists">
        <h2>8. Lists</h2>
        <ul>
          <li>Alpha</li>
          <li>Beta</li>
          <li>Gamma</li>
        </ul>
        <ol start={1}>
          <li>Satu</li>
          <li>Dua</li>
          <li>Tiga</li>
        </ol>
        <blockquote>
          <p>Blockquote dengan list:</p>
          <ul>
            <li>Alpha</li>
            <li>Beta</li>
          </ul>
        </blockquote>
        <Parego type="note" title="List in a Parego block" tag={"Ordered"}>
          <ol start={1}>
            <li>Primero</li>
            <li>Segundo</li>
            <li>Tercero</li>
          </ol>
        </Parego>
        <Hint type="?">
          <p>List in a Hint block.</p>
          <ul>
            <li>Est deserunt excepteur minim in ad nisi.</li>
            <li>Aliqua nostrud mollit cillum reprehenderit deserunt nostrud.</li>
          </ul>
        </Hint>
        <Parego type="error" title="List in an error block">
          <ul>
            <li>Ad excepteur nulla amet cupidatat aliqua.</li>
            <li>Velit adipisicing officia mollit aliquip anim cupidatat aute ad labore.</li>
          </ul>
        </Parego>
        <table class="striped" id="list-table" sortable="true">
          <caption>Lists in table cells</caption>
          <thead>
            <tr>
              <th style="text-align: left; width: 160px;">Jenis</th>
              <th style="text-align: left;">Isi Sel</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td style="text-align: left; width: 160px;">Bullet</td>
              <td style="text-align: left;">
                <ul>
                  <li>Alpha</li>
                  <li>Beta</li>
                  <li>Gamma</li>
                </ul>
              </td>
            </tr>
            <tr>
              <td style="text-align: left; width: 160px;">Ordered</td>
              <td style="text-align: left;">
                <ol start={1}>
                  <li>Satu</li>
                  <li>Dua</li>
                  <li>Tiga</li>
                </ol>
              </td>
            </tr>
            <tr>
              <td style="text-align: left; width: 160px;">Nested</td>
              <td style="text-align: left;">
                <ul>
                  <li>
                    <Epis level={4}>Aliquip commodo commodo.</Epis>
                    <ul>
                      <li>Nested satu</li>
                      <li>Nested dua</li>
                    </ul>
                  </li>
                </ul>
              </td>
            </tr>
            <tr>
              <td style="text-align: left; width: 160px;">Mixed</td>
              <td style="text-align: left;">
                Paragraf singkat.
                <ul>
                  <li>Setelah paragraf</li>
                  <li>Item kedua</li>
                </ul>
              </td>
            </tr>
          </tbody>
        </table>
      </section>
    </>
  )
}
