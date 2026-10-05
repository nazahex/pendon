export const frontmatter = {
  title: "Dialog Plugin Demo",
  cites: [
    {
      index: 1,
      id: "suryana-2026",
      loc: "p 45",
    },
    {
      index: 2,
      id: "paper-smith",
      loc: "p 210-225",
    },
    {
      index: 3,
      id: "web-react",
    },
  ],
  references: {
    // 1. Contoh Buku
    "suryana-2026": {
      id: "suryana-2026",
      type: "book",
      title: "Masa Depan Rekayasa Perangkat Lunak",
      authors: [{ firstName: "Eko", lastName: "Suryana" }],
      publisher: "TechPress Indonesia",
      publisherLocation: "Jakarta",
      issuedDate: { year: 2026 },
      isbn: "978-602-0000-00-0",
      language: "id",
    },

    // 2. Contoh Jurnal Akademik
    "paper-smith": {
      id: "paper-smith",
      type: "journal",
      title: "Generative MDX to PDF Compilation Architectures",
      authors: [
        { firstName: "John", lastName: "Smith" },
        { firstName: "Jane", lastName: "Doe" },
      ],
      containerTitle: "Journal of Web Engineering",
      volume: "18",
      issue: "4",
      pages: "210-225",
      issuedDate: { year: 2025, month: 8 },
      doi: "10.1016/j.jwe.2025.08.001",
      language: "en",
    },

    // 3. Contoh Website / Artikel Online
    "web-react": {
      id: "web-react",
      type: "website",
      title: "React Server Components Best Practices",
      publisherOrg: "React Documentation",
      url: "https://react.dev/learn/server-components",
      issuedDate: { year: 2026, month: 1, day: 15 },
      accessedDate: { year: 2026, month: 9, day: 5 },
      language: "en",
    },
  },
}
export default function PendonView() {
  return (
    <>
      <h1>Laporan Sintaks & Pengujian</h1>

      <p>
        Penerapan AI pada sistem modern sangat pesat{" "}
        <sup class="cite-ref">
          <a href="#citeref-1-suryana-2026" id="cra-1">
            [1]
          </a>
        </sup>
        . Dokumentasi lengkap dapat dilihat pada{" "}
        <a href="https://react.dev/learn">Portal Resmi React</a>.
      </p>

      <p>
        Optimasi <em>compiler</em> terbukti meningkatkan performa{" "}
        <sup class="cite-ref">
          <a href="#citeref-2-paper-smith" id="cra-2">
            [2]
          </a>
        </sup>
        . Penggunaan{" "}
        <sup class="cite-ref">
          <a href="#citeref-3-web-react" id="cra-3">
            [3]
          </a>
        </sup>{" "}
        juga disarankan untuk fleksibilitas arsitektur.
      </p>
    </>
  )
}
