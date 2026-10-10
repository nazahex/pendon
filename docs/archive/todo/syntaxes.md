<!-- WARNING: This is an obsolete TODO. -->

Berikut rangkuman tindakan konkret (untuk tim) agar syntect men‑tokenize TypeScript + JavaScript (Babel) dengan benar. Kamu sudah punya "JavaScript (Babel).sublime-syntax" — bagus. Ikuti langkah di bawah ini.

Tujuan singkat

- Pastikan grammar JavaScript (Babel) dan grammar TypeScript (+ TypeScriptReact jika pakai .tsx) dimuat ke SyntaxSet yang sama sehingga semua include/resolution dapat terselesaikan. Tes dengan file .ts/.tsx/.js/.jsx dan verifikasikan token scopes.

Langkah-langkah (urutan kerja)

1. Siapkan folder grammar

- Letakkan file grammar berikut dalam satu folder yang akan dimuat:
  - TypeScript.tmLanguage (atau TypeScript.YAML-tmLanguage → hasil build .tmLanguage)
  - TypeScriptReact.tmLanguage (jika pakai TSX)
  - JavaScript (Babel).sublime-syntax (file yang sudah kamu download)
  - Jika ada grammar JS tambahan (language-javascript, babel-sublime dari sumber lain) ikutkan juga
- Rekomendasi: satu folder seperti grammars/ atau syntaxes/.

2. Cara muat (pilihan, rekomendasi: muat seluruh folder)

- Rekomendasi paling mudah dan andal: muat seluruh folder grammar sekaligus.
  - Mengapa: syntect akan mendaftarkan semua grammar sehingga include/internal scope biasanya dapat di-resolve.

Contoh Rust (muat seluruh folder)

```rust
use syntect::parsing::SyntaxSetBuilder;
use std::path::Path;

fn build_syntax_set() -> syntect::parsing::SyntaxSet {
    let mut builder = SyntaxSetBuilder::new();
    // Ganti path ke folder yang berisi semua grammar (.tmLanguage/.tmLanguage.json/.sublime-syntax)
    builder.add_from_folder(Path::new("path/to/grammars/")).expect("add_from_folder failed");
    builder.build()
}
```

Jika kamu menambahkan file satu-per-satu (tidak direkomendasikan kecuali perlu kontrol urutan), tambahkan JavaScript (Babel) sebelum TypeScript:

```rust
use syntect::parsing::SyntaxSetBuilder;
use std::fs;

let mut builder = SyntaxSetBuilder::new();
let js_text = fs::read_to_string("grammars/JavaScript (Babel).sublime-syntax")?;
builder.add_from_str(&js_text, true, None)?;
let ts_text = fs::read_to_string("grammars/TypeScript.tmLanguage")?;
builder.add_from_str(&ts_text, true, None)?;
let ss = builder.build();
```

3. Verifikasi scope grammar yang tersedia

- Pastikan ss memiliki syntax dengan scopeName yang diperlukan (contoh yang umumnya dipakai: "source.js", "source.js.jsx", "source.ts", "source.tsx").
- Contoh pemeriksaan:

```rust
if ss.find_syntax_by_scope("source.js").is_none() {
    eprintln!("source.js tidak ditemukan — pastikan grammar JS dimuat");
}
```

4. Cek include yang belum ter-resolve (diagnostik cepat)

- Cara praktis: buka TypeScript/TypeScriptReact grammar (header) dan cari include yang bukan internal (bukan '#...') → mis. include: "source.js" atau include: "source.js.jsx".
- Alternatif: skrip kecil untuk men-scan file grammar mencari include yang mengandung titik (scope) dan memastikan ss.find_syntax_by_scope(scope) ada. Kalau tidak ada, tambahkan grammar yang sesuai.

5. Tes tokenization

- Ambil contoh file: test.ts, test.tsx, test.js, test.jsx.
- Tokenize dan print token scopes untuk baris yang bermasalah (mis. function, variable, komentar) untuk membandingkan apakah JS basic sekarang tokenized.
- Contoh (Rust):

```rust
let syntax = ss.find_syntax_by_extension("ts").unwrap(); // atau find_syntax_by_scope
let mut highlighter = syntect::easy::HighlightLines::new(syntax, &syntect::highlighting::ThemeSet::load_defaults().themes["base16-ocean.dark"]);
for line in text.lines() {
    let regions = highlighter.highlight_line(line, &ss).unwrap();
    // print regions/scopes...
}
```

6. Jika masih ada token yang hilang — debugging checklist

- Pastikan file grammar yang dipakai valid (tmLanguage / sublime-syntax yang didukung).
- Pastikan tidak ada nama scope mismatch: buka header grammar JS/Babel dan catat scopeName (baris name/scopeName di header).
- Pastikan TypeScript grammar include scope yang sama persis (case-sensitive).
- Coba muat grammar JS/TS dengan add_from_folder untuk menghindari urutan masalah.
- Print daftar semua syntax yang terdaftar:

```rust
for syn in ss.syntaxes() {
    println!("{} -> {}", syn.name, syn.scope.to_string());
}
```

- Jika kamu menggunakan prebuilt SyntaxSet (binary dump) — pastikan dump itu dibuat dengan grammar JS dan TS yang lengkap.

7. Performance & deployment

- Membangun SyntaxSet dari folder di runtime mahal; untuk produksi:
  - Bangun sekali di development, serialisasikan ke file (ss.save_to_file atau dump) dan load yang sudah serialized di runtime.
  - Atau buat step build (CI) yang menghasilkan precompiled syntaxset.

8. Tindakan segera untuk tim (checklist singkat)

- [ ] Taruh TypeScript.tmLanguage (+ TypeScriptReact jika ada) dan JavaScript (Babel).sublime-syntax di folder grammars/
- [ ] Jalankan builder contoh (add_from_folder) untuk membuat SyntaxSet
- [ ] Jalankan test tokenisasi pada file .js/.ts/.tsx; bandingkan scopes
- [ ] Jika ada missing includes, catat scope yang hilang dan tambahkan grammar yang menyediakannya
- [ ] Setelah OK, buat prebuilt dump untuk production

Referensi & catatan

- Repo TypeScript-TmLanguage menggabungkan bahan dari babel-sublime & language-javascript (lihat ThirdPartyNotices). Itu artinya grammar JS eksternal adalah bagian yang dibutuhkan.
- Jika mau, aku dapat:
  - membuat skrip Rust lengkap untuk muat folder + memeriksa "missing scopes" dan mencetak daftar grammar yang terdaftar; atau
  - periksa header file JavaScript (Babel).sublime-syntax yang kamu punya untuk memberi tahu scopeName yang harus dicari, dan menyesuaikan urutan add_from_str bila perlu.
