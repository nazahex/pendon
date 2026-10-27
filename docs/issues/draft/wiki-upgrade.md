Berikut rangkuman sintaksis wikilink `[[]]` yang kompleks dari berbagai style (Obsidian, MediaWiki/Wikipedia, Roam, Logseq, Foam, Dendron, Zettlr, dll). Ini murni demo sintaks dan gambaran hasil render.

## 1. Obsidian (paling kaya fitur)

| Sintaks                             | Fungsi                                             | Gambaran output                |
| ----------------------------------- | -------------------------------------------------- | ------------------------------ |
| `[[Catatan]]`                       | Link dasar                                         | Catatan                        |
| `[[Catatan\|Alias]]`                | Teks tampilan beda                                 | Alias                          |
| `[[Catatan#Heading]]`               | Link ke heading                                    | Catatan > Heading              |
| `[[Catatan#Heading#Subheading]]`    | Heading bertingkat                                 | Catatan > Heading > Subheading |
| `[[Catatan#^abc123]]`               | Link ke block ID                                   | Catatan > ^ block              |
| `[[#Heading]]`                      | Heading di catatan yang sama                       | Heading                        |
| `[[#^abc123]]`                      | Block di catatan yang sama                         | blok lokal                     |
| `[[#^abc123\|Alias]]`               | Block + alias                                      | Alias                          |
| `[[Folder/Sub/Catatan]]`            | Path lengkap                                       | Catatan                        |
| `[[Folder/Catatan#Heading\|Alias]]` | Kombinasi path + heading + alias                   | Alias                          |
| `[[Catatan.md]]`                    | Dengan ekstensi                                    | Catatan                        |
| `[[## teks]]`                       | Pencarian heading di seluruh vault (saat mengetik) | popup saran heading            |
| `[[^^ teks]]`                       | Pencarian block di seluruh vault (saat mengetik)   | popup saran block              |

**Embed (transklusi)** dengan tambahan `!`:

```markdown
![[Catatan]] <- seluruh catatan ditanam
![[Catatan#Heading]] <- hanya satu bagian
![[Catatan#^abc123]] <- hanya satu blok
![[gambar.png]] <- gambar
![[gambar.png|300]] <- lebar 300px
![[gambar.png|300x200]] <- lebar x tinggi
![[dokumen.pdf]] <- PDF
![[dokumen.pdf#page=3]] <- halaman tertentu
![[dokumen.pdf#page=3&height=400]]
![[audio.mp3]] <- audio
![[video.mp4]] <- video
![[berkas.base]] <- database/Bases
```

**Mendefinisikan block ID** (di akhir paragraf/daftar):

```markdown
Ini paragraf yang bisa direferensikan. ^abc123

- item daftar A
- item daftar B

^daftar-id
```

**Escape pipe dalam tabel** (karena `|` bentrok dengan kolom tabel):

```markdown
| Kolom              |
| ------------------ |
| [[Catatan\|Alias]] |
```

**Properti frontmatter**: wikilink wajib dibungkus tanda kutip.

```yaml
---
terkait: "[[Catatan]]"
daftar:
  - "[[A]]"
  - "[[B|Alias B]]"
---
```

## 2. MediaWiki / Wikipedia

```wikitext
[[Artikel]]                      -> Artikel
[[Artikel|Teks tampilan]]        -> Teks tampilan
[[Artikel#Bagian]]               -> Artikel § Bagian
[[#Bagian]]                      -> link internal halaman
[[Artikel (disambig)|]]          -> pipe trick: "Artikel"
[[Artikel]]s                     -> "Artikels" (akhiran ikut jadi link)
[[:Kategori:Nama]]               -> link ke halaman kategori (bukan memasukkan)
[[Kategori:Nama]]                -> memasukkan halaman ke kategori (tak tampil)
[[Berkas:Foto.jpg|thumb|250px|kiri|Keterangan]]  -> gambar dengan bingkai
[[Berkas:Foto.jpg|alt=Teks alt|link=Artikel]]    -> gambar yang mengarah ke link lain
[[:en:Article]]                  -> interwiki (bahasa lain)
[[Pengguna:Nama/Halaman]]        -> ruang nama
[[Special:Search/kata]]          -> halaman khusus
```

Pipe trick (disimpan sebagai hasil ekspansi saat penyimpanan):

```wikitext
[[Pengguna:Budi|]]        -> [[Pengguna:Budi|Budi]]
[[Artikel (konteks)|]]    -> [[Artikel (konteks)|Artikel]]
```

## 3. Roam Research

```markdown
[[Halaman]] -> link halaman
#[[Halaman Panjang]] -> tag dengan spasi
#tag -> tag sederhana
Atribut:: [[Nilai]] -> atribut berpasangan
((uid-blok)) -> referensi blok
{{embed: ((uid-blok))}} -> embed blok
{{[[embed]]: [[Halaman]]}} -> embed halaman
{{[[TODO]]}} -> penanda tugas
[teks]([[Halaman]]) -> alias halaman
[teks](((uid-blok))) -> alias blok
```

## 4. Logseq

```markdown
[[Halaman]]
[[Halaman/Sub/Anak]] -> namespace hierarkis
#[[Tag Panjang]]
tags:: [[A]], [[B]] -> properti
alias:: [[Nama Lain]]
((uuid-blok)) -> referensi blok
{{embed [[Halaman]]}} -> embed halaman
{{embed ((uuid-blok))}} -> embed blok
[Teks]([[Halaman]]) -> label kustom
```

## 5. Foam (VS Code)

```markdown
[[catatan]]
[[catatan#heading]]
[[catatan|alias]]
![[catatan]] -> embed
[[kata-kunci]] -> bisa tak terselesaikan -> placeholder
```

## 6. Dendron

```markdown
[[proyek.web.desain]] -> hierarki dengan titik
[[Alias|proyek.web.desain]] -> alias di depan (urutan terbalik!)
[[proyek.web.desain#heading]]
[[proyek.web.desain#^blok]]
[[proyek.web.desain#heading:#:#penutup]] -> rentang (range) antar header
![[proyek.web.desain]] -> transklusi
[[proyek.web.desain.*]] -> wildcard, mencantumkan seluruh anak
```

## 7. Lainnya (ringkas)

```markdown
# Zettlr / Zettelkasten

[[202610251753]] -> ID timestamp
[[202610251753 Judul]]

# Org-roam / Emacs (bukan [[ ]] murni tapi mirip)

[[id:uuid][Teks]]
[[file:catatan.org::*Heading][Teks]]

# TiddlyWiki

[[Teks tampilan|Judul Tiddler]] -> alias di depan
[[https://contoh.com]]
{{Tiddler}} -> transklusi

# Notion / Confluence (pemicu saat mengetik)

[[ -> membuka menu pencarian halaman
@Halaman -> mention
```

## 8. Perbedaan penting antar style

| Aspek            | Obsidian            | MediaWiki            | Dendron             | TiddlyWiki          |
| ---------------- | ------------------- | -------------------- | ------------------- | ------------------- |
| Urutan alias     | `[[target\|alias]]` | `[[target\|alias]]`  | `[[alias\|target]]` | `[[alias\|target]]` |
| Pemisah hierarki | `/`                 | `/` & ruang nama `:` | `.`                 | tak ada             |
| Embed            | `![[x]]`            | `[[Berkas:x]]`       | `![[x]]`            | `{{x}}`             |
| Block ref        | `#^id`              | tak ada              | `#^id`              | tak ada             |
| Link ke bagian   | `#heading`          | `#heading`           | `#heading`          | `##` (tak standar)  |

## 9. Contoh gabungan "ekstrem" (Obsidian)

```markdown
Lihat [[Riset/2026/Proyek Alfa#Metodologi#Sampling|bagian sampling]]
lalu ![[Riset/2026/Proyek Alfa#^hasil-utama]] dan
tabel di ![[data.pdf#page=12&height=300]].

| Sumber                | Catatan                         |
| --------------------- | ------------------------------- |
| [[Buku/Dasar\|Dasar]] | [[#Ringkasan\|ringkasan lokal]] |
```

Gambaran hasilnya: teks "bagian sampling" menjadi link biru yang membuka bagian _Sampling_ di dalam _Metodologi_; blok `^hasil-utama` tampil sebagai kutipan tertanam; PDF tampil sebagai pratinjau di halaman 12 setinggi 300px; dan tabel menampilkan dua link beralias.

Catatan akhir: sintaks bisa sedikit berubah antar versi aplikasi, jadi untuk fitur terbaru (misalnya Bases di Obsidian atau opsi range Dendron) sebaiknya cek dokumentasi resmi masing-masing. Kalau mau, aku bisa fokuskan ke satu style saja dengan contoh lebih dalam.
