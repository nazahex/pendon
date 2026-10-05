# RFC: Ultra-Minimal, Safe, and Compatible HTML Syntax Highlighter

## Status

**Proposed**

## Author

- Kaizan Sultan

## Ringkasan Singkat

RFC ini mengusulkan desain **syntax highlighter HTML yang sangat minimal secara ukuran**, tanpa JavaScript, tanpa inline style, dan tetap **aman serta kompatibel lintas browser**.
Pendekatan utama: **menggunakan tag HTML pendek sebagai token**, bukan class atau inline style.

---

## 1. Latar Belakang & Masalah

Sebagian besar syntax highlighter saat ini (Prism, Highlight.js, Shiki, Pygments):

- Menghasilkan HTML **besar dan verbose**
- Menggunakan:

  - inline style
  - class panjang
  - `<span>` berlapis

- Tidak optimal untuk:

  - static site
  - dokumentasi
  - bandwidth terbatas
  - embed HTML

Masalah utama:

> **Ukuran HTML meningkat drastis hanya untuk pewarnaan visual.**

---

## 2. Tujuan

Highlighter ini bertujuan untuk:

- Menghasilkan **HTML sekecil mungkin**
- Tetap:

  - valid HTML
  - aman untuk copy–paste
  - ramah SEO
  - kompatibel di browser modern & lama

- Tanpa:

  - JavaScript
  - inline style
  - fitur CSS eksperimental

Non-tujuan:

- Bukan highlighter semantik lengkap ala IDE
- Tidak mengejar akurasi lexer/compiler 100%

---

## 3. Prinsip Desain

1. **Token visual ≠ token bahasa**
2. **Jumlah token dikompresi ke minimum**
3. **HTML lebih penting daripada presisi highlight**
4. **CSS global, reusable, cacheable**
5. **Build-time friendly, runtime zero-cost**

---

## 4. Solusi yang Diusulkan

### 4.1 Tokenisasi Minimal

Gunakan **6–7 token universal**:

| Token      | Makna             |
| ---------- | ----------------- |
| keyword    | struktur bahasa   |
| identifier | variabel / fungsi |
| string     | literal           |
| number     | literal numerik   |
| comment    | komentar          |
| operator   | simbol            |

---

### 4.2 Representasi HTML (Kunci Utama)

Alih-alih:

```html
<span class="token keyword">for</span>
```

Gunakan **tag HTML pendek**:

```html
<b>for</b>
```

Contoh lengkap:

```html
<pre><code>
<b>for</b> (<em>i</em>=<u>0</u>; <em>i</em>&lt;<u>10</u>; <em>i</em>++)
</code></pre>
```

---

### 4.3 Mapping Token → Tag

| Token      | Tag                 |
| ---------- | ------------------- |
| keyword    | `<b>`               |
| identifier | `<em>`              |
| string     | `<i>`               |
| number     | `<u>`               |
| comment    | `<s>`               |
| operator   | `<mark>` (opsional) |

Semua tag di-_reset_ secara visual via CSS.

---

### 4.4 CSS Global

```css
pre {
  white-space: pre;
  overflow: auto;
}
b {
  color: #c792ea;
  font-weight: 600;
}
em {
  color: #82aaff;
  font-style: normal;
}
i {
  color: #ecc48d;
}
u {
  color: #f78c6c;
  text-decoration: none;
}
s {
  color: #5c6370;
  text-decoration: none;
}
mark {
  background: none;
  color: #89ddff;
}
```

CSS hanya dimuat **sekali**, HTML tetap minimal.

---

## 5. Keamanan & Kompatibilitas

### Aman karena:

- HTML valid
- Tanpa JS
- Tanpa inline style
- Copy–paste tidak rusak
- Screen reader tetap membaca teks

### Kompatibel karena:

- Menggunakan tag HTML standar
- CSS dasar
- Tidak bergantung pada fitur modern

---

## 6. Dampak Ukuran (Estimasi)

Per token:

| Pendekatan             | Karakter |
| ---------------------- | -------- |
| Inline style           | ~35      |
| Span + class panjang   | ~25      |
| Span + class pendek    | ~14      |
| `<mark>`               | ~11      |
| **Tag pendek (`<b>`)** | **~7**   |

➡️ Penghematan **60–80% ukuran HTML** dibanding highlighter umum.

---

## 7. Trade-off yang Diterima

| Trade-off              | Alasan                |
| ---------------------- | --------------------- |
| Semantic tag “dibajak” | Visual-only, aman     |
| Token detail dikurangi | Ukuran HTML prioritas |
| Tidak cocok untuk IDE  | Fokus dokumentasi     |

Trade-off ini **disengaja dan terukur**.

---

## 8. Alternatif yang Dipertimbangkan

- Inline style → ditolak (ukuran & repetisi)
- Span + class → ditolak (tetap verbose)
- JS runtime highlight → ditolak (kompatibilitas)
- Shadow DOM / Canvas → ditolak (copy & SEO)

---

## 9. Kesimpulan

RFC ini mengusulkan **syntax highlighter ekstrem secara ukuran**, namun:

- Aman
- Kompatibel
- Sederhana
- Mudah diimplementasikan

Pendekatan ini ideal untuk:

- static site
- dokumentasi teknikal
- blog engineering
- embedded HTML
