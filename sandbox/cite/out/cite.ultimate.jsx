import { Bibliography } from "@comp/content/Bibliography";
import { Cite } from "@comp/content/Cite";

export const frontmatter = { title: "Cite Ultimate Sandbox" };
export const references = {
  "ahmad-2026": {
    access: "open",
    authors: [
      { family: "Ahmad", given: "Salim" },
      { family: "Nakamura", given: "Yuki" },
    ],
    containerTitle: "arXiv",
    issuedDate: { month: 2, year: 2026 },
    language: "en",
    title: "Scaling Laws for Low-Resource Language Models",
    type: "preprint",
    url: "https://example.org/abs/0000.00000",
  },
  "hartono-2023": {
    authors: [{ family: "Hartono", given: "Dewi" }],
    containerTitle: "Harian Nusantara",
    issuedDate: { day: 12, month: 9, year: 2023 },
    language: "id",
    title: "Kurikulum Baru Mulai Diuji di 500 Sekolah",
    type: "news",
    url: "https://example.org/berita/kurikulum-baru-diuji",
  },
  "kant-1781": {
    authors: [
      { family: "Kant", given: "Immanuel" },
      { family: "Guyer", given: "Paul", role: "translator" },
      { family: "Wood", given: "Allen W.", role: "translator" },
    ],
    issuedDate: { year: 1998 },
    language: "en",
    originalDate: { year: 1781 },
    publisher: "Cambridge University Press",
    title: "Critique of Pure Reason",
    type: "book",
  },
  "kusuma-2024": {
    authors: [
      { family: "Kusuma", given: "Dian" },
      { family: "Pertiwi", given: "Ayu" },
      { family: "Hasan", given: "Rafi" },
    ],
    containerTitle: "Jurnal Informatika Terapan",
    doi: "10.0000/jit.2024.8.1.45",
    issue: "1",
    issuedDate: { year: 2024 },
    language: "id",
    pages: "45-62",
    review: "peer-reviewed",
    title: "Analisis Beban Kognitif pada Praktikum Pemrograman Dasar",
    type: "journal",
    volume: "8",
  },
  "lestari-2025": {
    access: "open",
    authors: [
      { family: "Lestari", given: "Sari" },
      { family: "Wibowo", given: "Agus" },
    ],
    containerTitle: "Jurnal Pendidikan Sains Indonesia",
    doi: "10.0000/jpsi.2025.13.2.210",
    issue: "2",
    issuedDate: { year: 2025 },
    language: "id",
    note: "Tinjauan sistematis yang menjadi rujukan utama untuk bukti pembelajaran berbasis proyek di perguruan tinggi.",
    pages: "210-225",
    review: "peer-reviewed",
    title: "Efektivitas Pembelajaran Berbasis Proyek pada Pendidikan Tinggi",
    type: "journal",
    volume: "13",
  },
  "mdn-clipboard": {
    accessedDate: { day: 15, month: 6, year: 2026 },
    authors: [{ literal: "MDN Contributors" }],
    containerTitle: "MDN Web Docs",
    language: "en",
    title: "Clipboard API",
    type: "website",
    url: "https://developer.mozilla.org/en-US/docs/Web/API/Clipboard_API",
  },
  "nugroho-2024": {
    authors: [{ family: "Nugroho", given: "Ari", role: "speaker" }],
    containerTitle: "YouTube",
    issuedDate: { day: 8, month: 3, year: 2024 },
    language: "id",
    title: "Memahami Entropi dalam Termodinamika",
    type: "video",
    url: "https://www.youtube.com/watch?v=abc123XYZ00",
  },
  "prasetyo-2019": {
    authors: [{ family: "Prasetyo", given: "Joko" }],
    containerTitle: "Jurnal Gizi dan Kesehatan",
    doi: "10.0000/jgk.2019.5.4.300",
    issue: "4",
    issuedDate: { year: 2019 },
    language: "id",
    pages: "300-312",
    retraction: {
      date: { month: 5, year: 2021 },
      noticeUrl: "https://example.org/retraction/prasetyo-2019",
    },
    title: "Dampak Suplemen X terhadap Daya Ingat",
    type: "journal",
    volume: "5",
  },
  "pratama-2022": {
    authors: [
      { family: "Pratama", given: "Rizky" },
      { family: "Wijaya", given: "Hendra", role: "editor" },
      { family: "Anggraini", given: "Putri", role: "editor" },
      { family: "Santoso", given: "Budi Eko", role: "translator" },
    ],
    containerTitle: "Handbook Pendidikan Digital",
    issuedDate: { year: 2022 },
    language: "id",
    pages: "101-130",
    publisher: "Pustaka Cendekia",
    title: "Literasi Data bagi Pendidik",
    type: "chapter",
  },
  "pspt-2024": {
    authors: [{ literal: "Pusat Studi Pendidikan Terbuka" }],
    issuedDate: { year: 2024 },
    language: "id",
    number: "03/2024",
    publisher: "Pusat Studi Pendidikan Terbuka",
    title: "Pemetaan Akses Pendidikan Terbuka di Indonesia",
    type: "report",
    url: "https://example.org/laporan/03-2024",
  },
  "rahman-2021": {
    authors: [{ family: "Rahman", given: "Fajar" }],
    containerTitle: "Digilib UNS",
    genre: "Disertasi doktoral",
    institution: "Universitas Sebelas Maret",
    issuedDate: { year: 2021 },
    language: "id",
    title: "Model Prediksi Curah Hujan Berbasis Jaringan Saraf Tiruan",
    type: "thesis",
    url: "https://example.org/digilib/rahman-2021",
  },
  "sulastri-2020": {
    authors: [{ family: "Sulastri" }],
    containerTitle: "Jurnal Humaniora Nusantara",
    issue: "2",
    issuedDate: { year: 2020 },
    language: "id",
    pages: "88-104",
    title: "Tradisi Lisan dan Pembentukan Identitas Komunitas",
    type: "journal",
    volume: "3",
  },
  "suryana-2016": {
    authors: [{ family: "Suryana", given: "Budi" }],
    edition: "Edisi ke-2",
    isbn: "9780000000002",
    issuedDate: { year: 2016 },
    language: "id",
    publisher: "TechPress Indonesia",
    publisherLoc: "Jakarta",
    title: "Pengantar Rekayasa Perangkat Lunak",
    type: "book",
  },
  "tanpa-penulis-2020": {
    issuedDate: { year: 2020 },
    language: "id",
    publisher: "Penerbit Kampus Contoh",
    title: "Panduan Penulisan Karya Ilmiah",
    type: "report",
  },
  "van-dijk-2008": {
    authors: [{ family: "Dijk", given: "Teun A.", particle: "van" }],
    issuedDate: { year: 2008 },
    language: "en",
    publisher: "Cambridge University Press",
    title: "Discourse and Context: A Sociocognitive Approach",
    type: "book",
  },
};
export const citations = [
  { citation_number: 1, instance_index: 1, ref_id: "suryana-2016" },
  { citation_number: 2, instance_index: 2, loc: "page 27-35", ref_id: "suryana-2016" },
  { citation_number: 3, instance_index: 3, loc: "page 50-55", ref_id: "suryana-2016" },
  { citation_number: 4, instance_index: 4, ref_id: "prasetyo-2019" },
  { citation_number: 5, instance_index: 5, loc: "section 2.5-3.2", ref_id: "kant-1781" },
  { citation_number: 6, instance_index: 6, ref_id: "ahmad-2026" },
  { citation_number: 6, instance_index: 7, ref_id: "ahmad-2026" },
  { citation_number: 1, instance_index: 8, ref_id: "suryana-2016" },
  { citation_number: 2, instance_index: 9, loc: "page 27-35", ref_id: "suryana-2016" },
  { citation_number: 7, instance_index: 10, loc: "page 27-30", ref_id: "suryana-2016" },
  { citation_number: 4, instance_index: 11, ref_id: "prasetyo-2019" },
  { citation_number: 4, instance_index: 12, ref_id: "prasetyo-2019" },
  { citation_number: 8, instance_index: 13, loc: "section 2.1", ref_id: "prasetyo-2019" },
  { citation_number: 4, instance_index: 14, ref_id: "prasetyo-2019" },
];
export default function PendonView() {
  return (
    <>
      <p>
        CITE.U1 - Sintaksis dasar{" "}
        <Cite reference={references["suryana-2016"]} citation_number={1} instance_index={1} />{" "}
        ipsum.
      </p>
      <p>
        CITE.U2 - Locator{" "}
        <Cite
          reference={references["suryana-2016"]}
          citation_number={2}
          instance_index={2}
          loc="page 27-35"
        />{" "}
        ipsum.
      </p>
      <p>
        CITE.U3 - Multiple citations{" "}
        <Cite
          reference={references["suryana-2016"]}
          citation_number={3}
          instance_index={3}
          loc="page 50-55"
        />
        <Cite reference={references["prasetyo-2019"]} citation_number={4} instance_index={4} />
        <Cite
          reference={references["kant-1781"]}
          citation_number={5}
          instance_index={5}
          loc="section 2.5-3.2"
        />{" "}
        ipsum.
      </p>
      <p>
        CITE.U4a - Empty extras{" "}
        <Cite reference={references["ahmad-2026"]} citation_number={6} instance_index={6} />
      </p>
      <p>
        CITE.U4b - Empty type{" "}
        <Cite reference={references["ahmad-2026"]} citation_number={6} instance_index={7} />
        @@ <code>@@</code> should be literal
      </p>
      <p>
        CITE.U4c - Empty extras{" "}
        <Cite reference={references["suryana-2016"]} citation_number={1} instance_index={8} />
      </p>
      <p>
        CITE.U5a - Backtiq extras{" "}
        <Cite
          reference={references["suryana-2016"]}
          citation_number={2}
          cite-id="backtick-foo"
          instance_index={9}
          loc="page 27-35"
        />
      </p>
      <p>
        CITE.U5b - Quoted extras{" "}
        <Cite
          reference={references["suryana-2016"]}
          citation_number={7}
          instance_index={10}
          loc="page 27-30"
          title="Quoted Foo"
        />
      </p>
      <p>
        CITE.U5c - Classes extras{" "}
        <Cite
          reference={references["prasetyo-2019"]}
          citation_number={4}
          class="extra class-foo"
          instance_index={11}
        />
      </p>
      <p>
        CITE.U5d - Classes props{" "}
        <Cite
          reference={references["prasetyo-2019"]}
          citation_number={4}
          details={{ author: "Setyo", reviewed: "Satya" }}
          instance_index={12}
          level={2}
          note="Note ipsum"
        />
      </p>
      <p>
        CITE.U5e - Styles extras{" "}
        <Cite
          reference={references["prasetyo-2019"]}
          border="2px"
          citation_number={8}
          instance_index={13}
          loc="section 2.1"
          style="--color: red;"
        />
      </p>
      <p>
        CITE.U5f - Typed{" "}
        <Cite
          reference={references["prasetyo-2019"]}
          citation_number={4}
          instance_index={14}
          type="CiteA"
        />
      </p>
      <Bibliography references={references} citations={citations} type="bibliography" />
    </>
  );
}
