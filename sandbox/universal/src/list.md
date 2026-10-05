# Advanced CommonMark List Demonstration

## Multi-Paragraph & Nested Lists

1. **Fase Perencanaan Utama**

   Pada fase ini, seluruh tim wajib menyelaraskan visi dan menentukan cakupan proyek secara mendalam. Jangan terburu-buru melakukan eksekusi sebelum dokumen _PRD_ (_Product Requirement Document_) disetujui.

   - **Sub-tugas A: Analisis Kebutuhan**
     - Mengumpulkan masukan dari pemangku kepentingan (_stakeholders_).
     - Menyusun skala prioritas fitur (_Must-have_, _Should-have_, _Nice-to-have_).
   - **Sub-tugas B: Alokasi Sumber Daya**
     - Menentukan anggaran operasional.
     - Mengatur pembagian beban kerja anggota tim.

   > **Catatan Penting:** Indentasi untuk konten lanjutan di dalam daftar berurutan harus sejajar dengan teks setelah nomor (biasanya 3–4 spasi).

2. **Fase Eksekusi & Pengembangan**

   Setelah tahap perencanaan selesai, proses pengerjaan dapat dimulai secara terstruktur sesuai iterasi _Sprint_.

## Lists containing Code Blocks & Complex Elements

- **Arsitektur Kode Backend**

  Gunakan arsitektur modular agar pemeliharaan kode (_maintenance_) menjadi lebih mudah di masa depan. Contoh struktur konfigurasi:

  ```json
  {
    "service": "authentication-api",
    "version": "v2.1.0",
    "features": {
      "oauth2": true,
      "rate_limiting": {
        "enabled": true,
        "max_requests": 100
      }
    }
  }
  ```

  Pastikan semua _environment variable_ disimpan secara aman dan tidak dimasukkan ke dalam repositori publik.

- **Pengintegrasian Database**

  Berikut adalah pemetaan skema dasar yang digunakan:

  | Nama Tabel | Tipe Data Utama | Deskripsi Singkat                     |
  | :--------- | :-------------- | :------------------------------------ |
  | `users`    | UUID            | Menyimpan kredensial dan profil utama |
  | `sessions` | String          | Menyimpan token autentikasi aktif     |
  | `logs`     | Timestamp       | Catatan aktivitas sistem              |

  - **Langkah Verifikasi Skema:**
    1. Jalankan perintah migrasi database:
       ```bash
       npm run db:migrate -- --env production
       ```
    2. Periksa status koneksi secara berkala.

## Mixed Ordering & Complex Indentation

1. Item Utama Pertama
   1. Sub-item berurutan (tingkat 2)
      - Sub-item tak berurutan (tingkat 3)
      - Sub-item tak berurutan lain
   2. Sub-item berurutan lanjutan
2. Item Utama Kedua

   Konten paragraf tambahan yang berada di dalam daftar item nomor dua.

Laborum elit anim officia do ut ea eu nostrud sunt.

:::info[Lorem Ipsum]

- Laborum incididunt officia pariatur magna.
- Id esse nisi commodo sit irure pariatur mollit nostrud ea tempor anim nostrud eiusmod.
- Ex laboris cupidatat deserunt eu excepteur aliqua Lorem et deserunt voluptate ipsum excepteur officia.

<!--  -->

:::

Mollit cupidatat sint occaecat mollit adipisicing cupidatat incididunt nostrud esse magna do.

:::info[Lorem Ipsum]

- Laborum incididunt officia pariatur magna.
- Id esse nisi commodo sit irure pariatur mollit nostrud ea tempor anim nostrud eiusmod.
- Ex laboris cupidatat deserunt eu excepteur aliqua Lorem et deserunt voluptate ipsum excepteur officia.

Esse fugiat sit exercitation irure labore dolor.

:::

Mollit cupidatat sint occaecat mollit adipisicing cupidatat incididunt nostrud esse magna do.

:::info[Lorem Ipsum]

- Laborum incididunt officia pariatur magna.
- Id esse nisi commodo sit irure pariatur mollit nostrud ea tempor anim nostrud eiusmod.
- Ex laboris cupidatat deserunt eu excepteur aliqua Lorem et deserunt voluptate ipsum excepteur officia.

:::

Eu magna anim nisi eiusmod cillum est pariatur cupidatat incididunt amet aliquip eu.

> ? - Nisi veniam adipisicing pariatur ipsum consequat veniam nulla pariatur quis ea minim deserunt.
>
> - Exercitation proident fugiat Lorem ad veniam amet ipsum.
> - Sunt veniam et non cupidatat anim cupidatat.

Commodo sit reprehenderit occaecat voluptate ex voluptate officia irure.
