/**
 * Tipe sumber referensi yang didukung oleh generator.
 */
export type SourceType =
  | "book" // Buku cetak / E-book
  | "journal" // Jurnal ilmiah / Paper akademik
  | "conference" // Prosiding konferensi / Paper simposium
  | "website" // Dokumentasi, artikel blog, halaman web
  | "news" // Berita media massa online/cetak
  | "report" // Whitepaper, laporan organisasi/pemerintah
  | "thesis" // Skripsi, Tesis, Disertasi
  | "video" // YouTube, rekaman presentasi, dokumenter
  | "podcast" // Episode audio / podcast
  | "other" // Sumber kustom lainnya

/**
 * Entitas individu atau organisasi yang berkontribusi pada karya.
 */
export interface Contributor {
  lastName: string // Nama belakang atau nama utama (misal: "Suryana" / "Google")
  firstName?: string // Nama depan (opsional untuk entitas tunggal/organisasi)
  role?: "author" | "editor" | "translator" | "speaker" | "interviewer" // Default: 'author'
}

/**
 * Tanggal fleksibel untuk menangani data publikasi yang tidak lengkap
 * (misal: hanya tahu tahun terbit, atau bulan dan tahun).
 */
export interface FlexibleDate {
  year: number
  month?: number // 1 - 12
  day?: number // 1 - 31
  raw?: string // Text fallback (misal: "Musim Semi 2026")
}

/**
 * Interface Utama: ReferenceData
 * Menggunakan pendekatan flat-structured dengan optional fields agar tidak bloated.
 */
export interface ReferenceData {
  id: string // Identifikasi unik (misal: "suryana-2026" atau UUID)
  type: SourceType // Tipe sumber publikasi
  title: string // Judul utama karya/artikel/buku

  // --- KONTRIBUSI & PUBLIKASI ---
  authors?: Contributor[] // Kontributor utama
  publisherOrg?: string // Nama organisasi/instansi penerbit (misal: "UNESCO", "React Docs")
  publisher?: string // Nama perusahaan penerbit (misal: "TechPress Indonesia")
  publisherLoc?: string // Kota/Lokasi penerbit (misal: "Jakarta")
  issuedDate?: FlexibleDate // Tanggal publikasi/rilis

  // --- WADAH & IDENTIFIKASI SPESIFIK (JOURNAL/WEBSITE/CONFERENCE) ---
  containerTitle?: string // Nama Jurnal, Nama Website, Nama Majalah, atau Nama Konferensi
  volume?: string // Volume jurnal/majalah
  issue?: string // Edisi/Issue nomor
  pages?: string // Rentang halaman pada sumber asli (misal: "210-225")
  edition?: string // Edisi buku (misal: "Edisi ke-2")

  // --- DIGITAL IDENTIFIERS & ACCESS ---
  url?: string // Tautan langsung ke sumber
  doi?: string // Digital Object Identifier (Standar Paper/Jurnal)
  isbn?: string // International Standard Book Number
  accessedDate?: FlexibleDate // Tanggal diakses (sangat krusial untuk Website/News)

  // --- LOKALISASI & FLEKSIBILITAS FUTUREPROOF ---
  language?: string // Kode ISO bahasa dokumen (misal: "id", "en", "ja")
  extra?: Record<string, string | number | boolean> // Escape hatch untuk metadata kustom di masa depan
}

/**
 * Peta data referensi (dictionary) untuk disimpan di Frontmatter atau File YAML eksternal.
 */
export type ReferenceMap = Record<string, ReferenceData>
