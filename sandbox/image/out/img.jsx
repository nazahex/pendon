export const frontmatter = { cites: [] }
export default function PendonView() {
  return (
    <>
      <h1>Image Demo</h1>
      <h2>Markdown Vanila</h2>
      <h3>Basic</h3>
      <p>
        <img
          alt="Aternative Text"
          src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp"
        />
      </p>
      <p>Expected result:</p>
      <pre lang="html">
        <code>
          &lt;img alt=&quot;Alternative Text&quot;
          src=&quot;https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp&quot;
          /&gt;
        </code>
      </pre>
      <hr />
      <p>
        <img
          alt=""
          src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp"
        />
      </p>
      <p>Expected result:</p>
      <pre lang="html">
        <code>
          &lt;img alt=&quot;&quot;
          src=&quot;https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp&quot;
          /&gt;
        </code>
      </pre>
      <h3>Figured</h3>
      <figure>
        <img
          alt="Aternative Text"
          src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp"
        />
        <figcaption>
          Exercitation qui <strong>exercitation</strong> dolor velit <em>aliqua</em> consectetur
          voluptate <a href="/consequat">consequat</a> labore elit non esse occaecat.
        </figcaption>
      </figure>
      <p>Expected result:</p>
      <pre lang="html">
        <code>
          &lt;figure&gt;&lt;img alt=&quot;Alternative Text&quot;
          src=&quot;https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp&quot;
          /&gt;&lt;figcaption&gt;Exercitation qui &lt;strong&gt;exercitation&lt;/strong&gt; dolor
          velit &lt;em&gt;aliqua&lt;/em&gt; consectetur voluptate &lt;a
          href=&quot;/consequat&quot;&gt;consequat&lt;/a&gt; labore elit non esse
          occaecat.&lt;/figcaption&gt;&lt;/figure&gt;
        </code>
      </pre>
      <blockquote>
        <p>Any markdown inline renderer must work properly.</p>
      </blockquote>
      <hr />
      <figure>
        <img
          alt=""
          src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp"
        />
      </figure>
      <p>Expected result:</p>
      <pre lang="html">
        <code>
          &lt;figure&gt;&lt;img alt=&quot;&quot;
          src=&quot;https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp&quot;
          /&gt;&lt;/figure&gt;
        </code>
      </pre>
      <h2>plugin-img</h2>
      <p>Plugin pendon untuk sintaksis img lebih lanjut.</p>
      <figure
        class="extra class or"
        data-baz="23"
        data-foo="bar"
        id="custom-id"
        style="--wix:sum;--rotate:5deg;">
        <img
          alt="Aternative Text"
          src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp"
        />
      </figure>
      <p>Expected result:</p>
      <pre lang="html">
        <code>
          &lt;figure id=&quot;custom-id&quot;class=&quot;extra class or&quot;
          data:foo=&quot;bar&quot; data:baz=&quot;23&quot;
          style=&quot;--wix:sum;--rotate:5deg;&quot;&gt;&lt;img alt=&quot;Alternative Text&quot;
          src=&quot;https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp&quot;
          /&gt;&lt;/figure&gt;
        </code>
      </pre>
      <hr />
      <img
        alt="Aternative Text"
        class="extra class or"
        data-baz="23"
        data-foo="bar"
        id="custom-id"
        src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp"
        style="--wix:sum;--rotate:5deg;"
      />
      <p>Expected result:</p>
      <pre lang="html">
        <code>
          &lt;img alt=&quot;Alternative Text&quot; id=&quot;custom-id&quot;class=&quot;extra class
          or&quot; data:foo=&quot;bar&quot; data:baz=&quot;23&quot;
          style=&quot;--wix:sum;--rotate:5deg;&quot;
          src=&quot;https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp&quot;
          /&gt;
        </code>
      </pre>
    </>
  )
}
