export default function PendonView() { return (<>
<h1>Advanced CommonMark List Demonstration</h1>
<h2>Multi-Paragraph &amp; Nested Lists</h2>
<ol start={1}>
<li><strong>Fase Perencanaan Utama</strong><p>Pada fase ini, seluruh tim wajib menyelaraskan visi dan menentukan cakupan proyek secara mendalam. Jangan terburu-buru melakukan eksekusi sebelum dokumen <i>PRD</i> (<i>Product Requirement Document</i>) disetujui.
</p>
<ul>
<li><strong>Sub-tugas A: Analisis Kebutuhan</strong><ul>
<li>Mengumpulkan masukan dari pemangku kepentingan (<i>stakeholders</i>).</li>
<li>Menyusun skala prioritas fitur (<i>Must-have</i>, <i>Should-have</i>, <i>Nice-to-have</i>).</li>
</ul>
</li>
<li><strong>Sub-tugas B: Alokasi Sumber Daya</strong><ul>
<li>Menentukan anggaran operasional.</li>
<li>Mengatur pembagian beban kerja anggota tim.</li>
</ul>
</li>
</ul>
<blockquote>
<p><strong>Catatan Penting:</strong> Indentasi untuk konten lanjutan di dalam daftar berurutan harus sejajar dengan teks setelah nomor (biasanya 3–4 spasi).
</p>
</blockquote>
</li>
<li><strong>Fase Eksekusi &amp; Pengembangan</strong><p>Setelah tahap perencanaan selesai, proses pengerjaan dapat dimulai secara terstruktur sesuai iterasi <i>Sprint</i>.
</p>
</li>
</ol>
<h2>Lists containing Code Blocks &amp; Complex Elements</h2>
<ul>
<li><strong>Arsitektur Kode Backend</strong><p>Gunakan arsitektur modular agar pemeliharaan kode (<i>maintenance</i>) menjadi lebih mudah di masa depan. Contoh struktur konfigurasi:
</p>
<pre lang="json"><code>&#123;
  &quot;service&quot;: &quot;authentication-api&quot;,
  &quot;version&quot;: &quot;v2.1.0&quot;,
  &quot;features&quot;: &#123;
    &quot;oauth2&quot;: true,
    &quot;rate_limiting&quot;: &#123;
      &quot;enabled&quot;: true,
      &quot;max_requests&quot;: 100
    &#125;
  &#125;
&#125;
</code></pre>
<p>Pastikan semua <i>environment variable</i> disimpan secara aman dan tidak dimasukkan ke dalam repositori publik.
</p>
</li>
<li><strong>Pengintegrasian Database</strong><p>Berikut adalah pemetaan skema dasar yang digunakan:
</p>
<table>
<thead>
<tr>
<th>Nama Tabel</th>
<th>Tipe Data Utama</th>
<th>Deskripsi Singkat</th>
</tr>
</thead>
<tbody>
<tr>
<td><code>users</code></td>
<td>UUID</td>
<td>Menyimpan kredensial dan profil utama</td>
</tr>
<tr>
<td><code>sessions</code></td>
<td>String</td>
<td>Menyimpan token autentikasi aktif</td>
</tr>
<tr>
<td><code>logs</code></td>
<td>Timestamp</td>
<td>Catatan aktivitas sistem</td>
</tr>
</tbody>
</table>
<ul>
<li><strong>Langkah Verifikasi Skema:</strong><ol start={1}>
<li>Jalankan perintah migrasi database:<pre lang="bash"><code>npm run db:migrate -- --env production
</code></pre>
</li>
<li>Periksa status koneksi secara berkala.</li>
</ol>
</li>
</ul>
</li>
</ul>
<h2>Mixed Ordering &amp; Complex Indentation</h2>
<ol start={1}>
<li>Item Utama Pertama<ol start={1}>
<li>Sub-item berurutan (tingkat 2)<ul>
<li>Sub-item tak berurutan (tingkat 3)</li>
<li>Sub-item tak berurutan lain</li>
</ul>
</li>
<li>Sub-item berurutan lanjutan</li>
</ol>
</li>
<li>Item Utama Kedua<p>Konten paragraf tambahan yang berada di dalam daftar item nomor dua.
</p>
</li>
</ol>

</>); }
