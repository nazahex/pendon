export const frontmatter = {"cites":[{"id":"suryana-2026","index":1,"loc":"hlm. 45"},{"id":"paper-smith","index":2,"loc":"hlm. 210-225"},{"id":"web-react","index":3}],"references":{"paper-smith":{"authors":[{"firstName":"John","lastName":"Smith"},{"firstName":"Jane","lastName":"Doe"}],"containerTitle":"Journal of Web Engineering","doi":"10.1016/j.jwe.2025.08.001","id":"paper-smith","issue":"4","issuedDate":{"month":8,"year":2025},"language":"en","pages":"210-225","title":"Generative MDX to PDF Compilation Architectures","type":"journal","volume":"18"},"suryana-2026":{"authors":[{"firstName":"Eko","lastName":"Suryana"}],"id":"suryana-2026","isbn":"978-602-0000-00-0","issuedDate":{"year":2026},"language":"id","publisher":"TechPress Indonesia","publisherLocation":"Jakarta","title":"Masa Depan Rekayasa Perangkat Lunak","type":"book"},"web-react":{"accessedDate":{"day":5,"month":9,"year":2026},"id":"web-react","issuedDate":{"day":15,"month":1,"year":2026},"language":"en","publisherOrg":"React Documentation","title":"React Server Components Best Practices","type":"website","url":"https://react.dev/learn/server-components"}},"title":"Lapiran Sintaks"};
export default function PendonView() { return (<>
<h1>Laporan Sintaks &amp; Pengujian</h1>
<p>Penerapan AI pada sistem modern sangat pesat <sup class="cite-ref"><a href="#citeref-1-suryana-2026" id="cra-1">[1]</a></sup>. Dokumentasi lengkap dapat dilihat pada <a href="https://react.dev/learn">Portal Resmi React</a>.
</p>
<p>Optimasi <i>compiler</i> terbukti meningkatkan performa <sup class="cite-ref"><a href="#citeref-2-paper-smith" id="cra-2">[2]</a></sup>. Penggunaan <sup class="cite-ref"><a href="#citeref-3-web-react" id="cra-3">[3]</a></sup> juga disarankan untuk fleksibilitas arsitektur.
</p>
<hr />
<h2>Expected HTML Output</h2>
<pre lang="html"><code>&lt;h1&gt;Laporan Sintaks &amp; Pengujian&lt;/h1&gt;

&lt;p&gt;
  Penerapan AI pada sistem modern sangat pesat &lt;sup class=&quot;cite-ref&quot;&gt;&lt;a href=&quot;#citeref-1-suryana-2026&quot; id=&quot;cra-1&quot;&gt;[1]&lt;/a&gt;&lt;/sup&gt;. 
  Dokumentasi lengkap dapat dilihat pada &lt;a href=&quot;https://react.dev/learn&quot;&gt;Portal Resmi React&lt;/a&gt;.
&lt;/p&gt;

&lt;p&gt;
  Optimasi &lt;em&gt;compiler&lt;/em&gt; terbukti meningkatkan performa &lt;sup class=&quot;cite-ref&quot;&gt;&lt;a href=&quot;#citeref-2-paper-smith&quot; id=&quot;cra-2&quot;&gt;[2]&lt;/a&gt;&lt;/sup&gt;. 
  Penggunaan &lt;sup class=&quot;cite-ref&quot;&gt;&lt;a href=&quot;#citeref-3-web-react&quot; id=&quot;cra-3&quot;&gt;[3]&lt;/a&gt;&lt;/sup&gt; juga disarankan untuk fleksibilitas arsitektur.
&lt;/p&gt;
</code></pre>
<h2>Expected Solid Output</h2>
<pre lang="jsx"><code>export const frontmatter = &#123;
  &quot;title&quot;: &quot;Dialog Plugin Demo&quot;,
  &quot;cites&quot;: [
    &#123;
      index: 1,
      id: &quot;suryana-2026&quot;,
      loc: &quot;p 45&quot;
    &#125;,
    &#123;
      index: 2,
      id: &quot;paper-smith&quot;,
      loc: &quot;p 210-225&quot;
    &#125;,
    &#123;
      index: 3,
      id: &quot;web-react&quot;
    &#125;,
  ],
  &quot;references&quot;: &#123;
    // 1. Contoh Buku
    &quot;suryana-2026&quot;: &#123;
      id: &quot;suryana-2026&quot;,
      type: &quot;book&quot;,
      title: &quot;Masa Depan Rekayasa Perangkat Lunak&quot;,
      authors: [&#123; firstName: &quot;Eko&quot;, lastName: &quot;Suryana&quot; &#125;],
      publisher: &quot;TechPress Indonesia&quot;,
      publisherLocation: &quot;Jakarta&quot;,
      issuedDate: &#123; year: 2026 &#125;,
      isbn: &quot;978-602-0000-00-0&quot;,
      language: &quot;id&quot;
    &#125;,

    // 2. Contoh Jurnal Akademik
    &quot;paper-smith&quot;: &#123;
      id: &quot;paper-smith&quot;,
      type: &quot;journal&quot;,
      title: &quot;Generative MDX to PDF Compilation Architectures&quot;,
      authors: [
        &#123; firstName: &quot;John&quot;, lastName: &quot;Smith&quot; &#125;,
        &#123; firstName: &quot;Jane&quot;, lastName: &quot;Doe&quot; &#125;
      ],
      containerTitle: &quot;Journal of Web Engineering&quot;,
      volume: &quot;18&quot;,
      issue: &quot;4&quot;,
      pages: &quot;210-225&quot;,
      issuedDate: &#123; year: 2025, month: 8 &#125;,
      doi: &quot;10.1016/j.jwe.2025.08.001&quot;,
      language: &quot;en&quot;
    &#125;,

    // 3. Contoh Website / Artikel Online
    &quot;web-react&quot;: &#123;
      id: &quot;web-react&quot;,
      type: &quot;website&quot;,
      title: &quot;React Server Components Best Practices&quot;,
      publisherOrg: &quot;React Documentation&quot;,
      url: &quot;https://react.dev/learn/server-components&quot;,
      issuedDate: &#123; year: 2026, month: 1, day: 15 &#125;,
      accessedDate: &#123; year: 2026, month: 9, day: 5 &#125;,
      language: &quot;en&quot;
    &#125;
  &#125;
&#125;
export default function PendonView() &#123;
  return (&lt;&gt;
    &lt;h1&gt;Laporan Sintaks &amp; Pengujian&lt;/h1&gt;

    &lt;p&gt;
      Penerapan AI pada sistem modern sangat pesat &lt;sup class=&quot;cite-ref&quot;&gt;&lt;a href=&quot;#citeref-1-suryana-2026&quot; id=&quot;cra-1&quot;&gt;[1]&lt;/a&gt;&lt;/sup&gt;. 
      Dokumentasi lengkap dapat dilihat pada &lt;a href=&quot;https://react.dev/learn&quot;&gt;Portal Resmi React&lt;/a&gt;.
    &lt;/p&gt;

    &lt;p&gt;
      Optimasi &lt;em&gt;compiler&lt;/em&gt; terbukti meningkatkan performa &lt;sup class=&quot;cite-ref&quot;&gt;&lt;a href=&quot;#citeref-2-paper-smith&quot; id=&quot;cra-2&quot;&gt;[2]&lt;/a&gt;&lt;/sup&gt;. 
      Penggunaan &lt;sup class=&quot;cite-ref&quot;&gt;&lt;a href=&quot;#citeref-3-web-react&quot; id=&quot;cra-3&quot;&gt;[3]&lt;/a&gt;&lt;/sup&gt; juga disarankan untuk fleksibilitas arsitektur.
    &lt;/p&gt;

  &lt;/&gt;);
&#125;
</code></pre>
<hr />
<h2>Configuration Proposal Demo</h2>
<pre lang="toml"><code>[[task]]
name = &quot;Cite Demo (Solid)&quot;
input = &quot;./src/[slug].md&quot;
output = &quot;./out/[slug].jsx&quot;
plugin = &quot;micromatter,cite,markdown&quot;
format = &quot;solid&quot;

cite_prefix = &quot;citeref-&quot;
cite_class = &quot;cite-ref&quot;
cite_id_prefix = &quot;cra-&quot;
cite_reference_source = &quot;frontmatter&quot; # Option: &quot;frontmatter&quot; | &quot;internal&quot; | &quot;external&quot;, &quot;frontmatter&quot; == &quot;internal&quot;
cite_reference_file = &quot;./src/citeref.yaml&quot; # file location if source is extenal
</code></pre>
<p>Jika source == external, maka (HANYA) referensi yang terpanggil di dalam konten diinjeksi ke dalam frontmatter. Jika ternyata file .md sudah ada <code>references</code> data, maka injeksi dilakukan sebagai penambahan, jika <code>id</code> referensi sama, maka skip referensi dengan id tersebut.
</p>

</>); }
