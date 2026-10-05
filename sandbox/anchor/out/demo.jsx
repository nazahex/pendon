export const frontmatter = { cites: [] }
export default function PendonView() {
  return (
    <>
      <h2> Example</h2>
      <p>
        Aliqua aute nulla{" "}
        <a href="https://google.com" rel="noopener" target="_blank">
          excepteur
        </a>{" "}
        nostrud eu voluptate nulla nisi <a href="/foo/bar">proident</a> ullamco cillum.
      </p>
      <p>
        Ea veniam{" "}
        <a href="https://anu.org" rel="noopener nofollow" target="_blank" title="Anu">
          veniam
        </a>{" "}
        id in adipisicing culpa irure ad nulla incididunt{" "}
        <a href="http://example.com" rel="noopener" target="_blank" title="Examplae Incididunt">
          mollit
        </a>{" "}
        excepteur sit.
      </p>
      <h2>Proposal</h2>
      <h3>Custom Syntax</h3>
      <ul>
        <li>
          Jika link diawali dengan <code>http://</code>, atau <code>https://</code>, atau domain
          website apapun, maka anchor otomatis mendapatkan{" "}
          <code>target=&quot;_blank&quot; rel=&quot;noopener&quot;</code>
        </li>
        <li>
          <code>^</code> di akhir link berfungsi untuk membubuhkan{" "}
          <code>target=&quot;_blank&quot; rel=&quot;noopener&quot;</code> secara paksa.
        </li>
        <li>
          <code>~</code> di akhir link berfungsi untuk membubuhkan{" "}
          <code>target=&quot;_self&quot;</code> secara paksa.
        </li>
        <li>
          <code>!</code> di akhir link berfungsi untuk membubuhkan{" "}
          <code>rel=&quot;nofollow&quot;</code>.
        </li>
        <li>
          <code>--</code> di akhir link berfungsi untuk membubuhkan{" "}
          <code>rel=&quot;noreferrer&quot;</code>.
        </li>
        <li>
          <code>$</code> di akhir link berfungsi untuk membubuhkan{" "}
          <code>rel=&quot;sponsored&quot;</code>.
        </li>
        <li>
          <code>;;</code> di akhir link berfungsi untuk membubuhkan <code>rel=&quot;ugc&quot;</code>
          .
        </li>
        <li>
          Semua simbol bisa digunakan semuanya atau sebagian dan tidak ada aturan urutan penggunaan.
        </li>
      </ul>
      <h3>Extra Attr</h3>
      <p>
        Aliquip{" "}
        <a
          href="https://foo.com/bar"
          hreflang="en"
          qux="rox"
          rel="noopener noreferrer nofollow prefetch"
          target="_blank"
          title="Title Foo">
          consectetur
        </a>{" "}
        magna velit cupidatat sint ad qui aliquip.
      </p>
      <p>
        Mollit{" "}
        <a
          href="/foo/bar"
          rel="noopener noreferrer sponsored nofollow"
          target="_blank"
          title="Buy Foo!">
          labore
        </a>{" "}
        anim ipsum in ullamco.
      </p>
      <p>Expected Output:</p>
      <pre lang="html">
        <code>
          &lt;p&gt;Aliquip &lt;a href=&quot;https://foo.com/bar&quot; target=&quot;_blank&quot;
          title=&quot;Title Foo&quot; rel=&quot;nooopener noreferrer nofollow prefetch&quot;
          hreflang=&quot;en&quot; qux=&quot;rox&quot;&gt;consectetur&lt;/a&gt; magna velit cupidatat
          sint ad qui aliquip.&lt;/p&gt; &lt;p&gt;Mollit &lt;a href=&quot;/foo/bar&quot;
          title=&quot;Buy Foo!&quot; target=&quot;_blank&quot; rel=&quot;noopener noreferrer
          sponsored nofollow&quot;&gt;labore&lt;/a&gt; anim ipsum in ullamco.&lt;/p&gt;
        </code>
      </pre>
      <h3>Custom Node</h3>
      <p>
        Buat supaya dev (Solid terutama) bisa mengurus output Jsx sendiri dengan komponen custom.
        Cara kerjanya mirip dengan custom component pada <code>plugin-cite</code>
      </p>
    </>
  )
}
