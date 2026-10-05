export const frontmatter = {
  cites: [{ id: "paper-smith", index: 1 }],
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
  },
}
export default function PendonView() {
  return (
    <>
      <p>
        Aliquip{" "}
        <a
          href="https://foo.com/bar"
          hreflang="en"
          qux="rox"
          rel="noopener noreferrer nofollow prefetch"
          target="_blank"
          title="Title Foo">
          consectetur
        </a>{" "}
        magna velit cupidatat sint ad qui aliquip.
      </p>
      <p>
        <img
          alt="Aternative Text"
          src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp"
        />
      </p>
      <p>
        Mollit{" "}
        <a
          href="/foo/bar"
          rel="noopener noreferrer sponsored nofollow"
          target="_blank"
          title="Buy Foo!">
          labore
        </a>{" "}
        anim ipsum
        <sup class="cite-ref">
          <a href="#citeref-1-paper-smith" id="cra-1">
            [1]
          </a>
        </sup>{" "}
        in ullamco.
      </p>
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
          Exercitation qui <strong>exercitation</strong> dolor velit <em>aliqua</em> consectetur
          voluptate <a href="/consequat">consequat</a> labore elit non esse occaecat.
        </figcaption>
      </figure>
      <p>Velit consequat culpa magna fugiat non occaecat voluptate.</p>
    </>
  )
}
