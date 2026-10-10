import Hint from "@comp/shared/Hint";
import Parego from "@comp/shared/Parego";

export const frontmatter = {"cites":[]};
export const headings = [{"id":"advanced-commonmark-list-demonstration","text":"Advanced CommonMark List Demonstration","level":1,"subheadings":[{"id":"multi-paragraph-nested-lists","text":"Multi-Paragraph & Nested Lists","level":2},{"id":"lists-containing-code-blocks-complex-elements","text":"Lists containing Code Blocks & Complex Elements","level":2},{"id":"mixed-ordering-complex-indentation","text":"Mixed Ordering & Complex Indentation","level":2}]}];
export default function PendonView() { return (<>
<section id="advanced-commonmark-list-demonstration">
<h1>1. Advanced CommonMark List Demonstration</h1>
<section id="multi-paragraph-nested-lists">
<h2>1.1. Multi-Paragraph &amp; Nested Lists</h2>
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
</section>
<section id="lists-containing-code-blocks-complex-elements">
<h2>1.2. Lists containing Code Blocks &amp; Complex Elements</h2>
<ul>
<li><strong>Arsitektur Kode Backend</strong><p>Gunakan arsitektur modular agar pemeliharaan kode (<i>maintenance</i>) menjadi lebih mudah di masa depan. Contoh struktur konfigurasi:
</p>
<pre lang="json"><code innerHTML={"<p>{</p><p>  <i><i>&quot;</i>service<i>&quot;</i></i>: <i><i>&quot;</i>authentication-api<i>&quot;</i></i>,</p><p>  <i><i>&quot;</i>version<i>&quot;</i></i>: <i><i>&quot;</i>v2.1.0<i>&quot;</i></i>,</p><p>  <i><i>&quot;</i>features<i>&quot;</i></i>: {</p><p>    <i><i>&quot;</i>oauth2<i>&quot;</i></i>: true,</p><p>    <i><i>&quot;</i>rate_limiting<i>&quot;</i></i>: {</p><p>      <i><i>&quot;</i>enabled<i>&quot;</i></i>: true,</p><p>      <i><i>&quot;</i>max_requests<i>&quot;</i></i>: 100</p><p>    }</p><p>  }</p><p>}</p>"} /></pre>
<p>Pastikan semua <i>environment variable</i> disimpan secara aman dan tidak dimasukkan ke dalam repositori publik.
</p>
</li>
<li><strong>Pengintegrasian Database</strong><p>Berikut adalah pemetaan skema dasar yang digunakan:
</p>
<table>
  <thead>
    <tr>
      <th style="text-align: left;">Nama Tabel</th>
      <th style="text-align: left;">Tipe Data Utama</th>
      <th style="text-align: left;">Deskripsi Singkat</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td style="text-align: left;"><code>users</code></td>
      <td style="text-align: left;">UUID</td>
      <td style="text-align: left;">Menyimpan kredensial dan profil utama</td>
    </tr>
    <tr>
      <td style="text-align: left;"><code>sessions</code></td>
      <td style="text-align: left;">String</td>
      <td style="text-align: left;">Menyimpan token autentikasi aktif</td>
    </tr>
    <tr>
      <td style="text-align: left;"><code>logs</code></td>
      <td style="text-align: left;">Timestamp</td>
      <td style="text-align: left;">Catatan aktivitas sistem</td>
    </tr>
  </tbody>
</table>
<ul>
<li><strong>Langkah Verifikasi Skema:</strong><ol start={1}>
<li>Jalankan perintah migrasi database:<pre lang="bash"><code innerHTML={"<p>npm run db:migrate<b> --</b> --env production</p>"} /></pre>
</li>
<li>Periksa status koneksi secara berkala.</li>
</ol>
</li>
</ul>
</li>
</ul>
</section>
<section id="mixed-ordering-complex-indentation">
<h2>1.3. Mixed Ordering &amp; Complex Indentation</h2>
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
<p>Laborum elit anim officia do ut ea eu nostrud sunt.
</p>
<Parego type="info" title="Lorem Ipsum"><ul>
<li>Laborum incididunt officia pariatur magna.</li>
<li>Id esse nisi commodo sit irure pariatur mollit nostrud ea tempor anim nostrud eiusmod.</li>
<li>Ex laboris cupidatat deserunt eu excepteur aliqua Lorem et deserunt voluptate ipsum excepteur officia.</li>
</ul>
</Parego><p>Mollit cupidatat sint occaecat mollit adipisicing cupidatat incididunt nostrud esse magna do.
</p>
<Parego type="info" title="Lorem Ipsum"><ul>
<li>Laborum incididunt officia pariatur magna.</li>
<li>Id esse nisi commodo sit irure pariatur mollit nostrud ea tempor anim nostrud eiusmod.</li>
<li>Ex laboris cupidatat deserunt eu excepteur aliqua Lorem et deserunt voluptate ipsum excepteur officia.</li>
</ul>
<p>Esse fugiat sit exercitation irure labore dolor.
</p>
</Parego><p>Mollit cupidatat sint occaecat mollit adipisicing cupidatat incididunt nostrud esse magna do.
</p>
<Parego type="info" title="Lorem Ipsum"><ul>
<li>Laborum incididunt officia pariatur magna.</li>
<li>Id esse nisi commodo sit irure pariatur mollit nostrud ea tempor anim nostrud eiusmod.</li>
<li>Ex laboris cupidatat deserunt eu excepteur aliqua Lorem et deserunt voluptate ipsum excepteur officia.</li>
</ul>
</Parego><p>Eu magna anim nisi eiusmod cillum est pariatur cupidatat incididunt amet aliquip eu.
</p>
<Hint type="?"><ul>
<li>Nisi veniam adipisicing pariatur ipsum consequat veniam nulla pariatur quis ea minim deserunt.
</li>
<li>Exercitation proident fugiat Lorem ad veniam amet ipsum.</li>
<li>Sunt veniam et non cupidatat anim cupidatat.</li>
</ul>
</Hint><p>Commodo sit reprehenderit occaecat voluptate ex voluptate officia irure.
</p>
</section>
</section>

</>); }
