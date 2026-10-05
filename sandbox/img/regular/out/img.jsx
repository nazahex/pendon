import AdvancedImage from "@/components/AdvancedImage"

export const frontmatter = {
  cites: [
    { id: "suryana-2026", index: 1 },
    { id: "suryana-2026", index: 2, loc: "hlm. 45" },
  ],
  references: {
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
  title: "Lapiran Sintaks",
}
export default function PendonView() {
  return (
    <>
      <p>Esse exercitation irure ullamco veniam fugiat eu consequat proident quis do.</p>
      <AdvancedImage
        id="custom-id"
        src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp"
        alt="lorem ipsum"
        foo="bar"
        baz="23"
        class="extra class or">
        Exercitationqui<strong>exercitation</strong>dolorvelit<em>aliqua</em>
        <a href="/id/wiki/Anim_Esta_(Officia)" title="Anim Esta (Officia)">
          Anim
        </a>
        consectetur
        <sup class="cite-ref">
          <a href="#citeref-1-suryana-2026" id="cra-1">
            [1]
          </a>
        </sup>
        voluptate
        <a
          href="/foo/bar"
          rel="noopener noreferrer sponsored nofollow"
          target="_blank"
          title="Buy Foo!">
          labore
        </a>
        laboreelitnonesseoccaecat.
        <sup class="cite-ref">
          <a href="#citeref-2-suryana-2026" id="cra-2">
            [2]
          </a>
        </sup>
      </AdvancedImage>
      <p>
        Officia amet cillum reprehenderit id pariatur commodo ullamco ad ipsum reprehenderit{" "}
        <a href="/id/wiki/Anim_Esta_(Officia)" title="Anim Esta (Officia)">
          Anim
        </a>{" "}
        proident reprehenderit.{" "}
        <sup class="cite-ref">
          <a href="#citeref-2-suryana-2026" id="cra-2">
            [2]
          </a>
        </sup>
      </p>
    </>
  )
}
