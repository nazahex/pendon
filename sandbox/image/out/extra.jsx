export const frontmatter = { cites: [] }
export default function PendonView() {
  return (
    <>
      <h2>Extra Syntax</h2>
      <p>Sintaksis tambahan untuk plugin-img</p>
      <h3>Width and Height</h3>
      <img
        alt="foo"
        height="800"
        src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp"
        width="300"
      />
      <p>Expected result:</p>
      <pre lang="html">
        <code>
          &lt;img width=&quot;300&quot; height=&quot;800&quot; alt=&quot;lorem ipsum&quot;
          src=&quot;https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp&quot;
          /&gt;
        </code>
      </pre>
      <hr />
      <figure>
        <img
          alt="foo"
          height="600"
          src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp"
        />
      </figure>
      <p>Expected result:</p>
      <pre lang="html">
        <code>
          &lt;figure&gt;&lt;img height=&quot;800&quot; alt=&quot;lorem ipsum&quot;
          src=&quot;https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp&quot;
          /&gt;&lt;/figure&gt;
        </code>
      </pre>
      <h3>Lazy Loading</h3>
      <img
        alt="lorem ipsum"
        loading="lazy"
        src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp"
      />
      <pre lang="html">
        <code>
          &lt;img loading=&quot;lazy&quot; alt=&quot;lorem ipsum&quot;
          src=&quot;https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp&quot;
          /&gt;
        </code>
      </pre>
      <p>html</p>
      <img
        loading="lazy"
        alt="lorem ipsum"
        src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp"
      />
      <pre>
        <code></code>
      </pre>
      <p>html</p>
      <figure>
        <img
          loading="lazy"
          alt="lorem ipsum"
          src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bnch_egora7.webp"
        />
      </figure>
      <pre>
        <code></code>
      </pre>
      <p>html</p>
      <img
        decoding="async"
        alt="lorem ipsum"
        src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp"
      />
      <hr />
      <img
        alt="lorem ipsum"
        decoding="async"
        src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp"
      />
      <pre lang="html">
        <code>
          &lt;img decoding=&quot;async&quot; alt=&quot;lorem ipsum&quot;
          src=&quot;https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp&quot;
          /&gt;
        </code>
      </pre>
      <hr />
      <figure>
        <img
          alt="lorem ipsum"
          decoding="async"
          src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp"
        />
      </figure>
      <pre lang="html">
        <code>
          &lt;figure&gt;&lt;img decoding=&quot;async&quot; alt=&quot;lorem ipsum&quot;
          src=&quot;https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bnch_egora7.webp&quot;
          /&gt;&lt;/figure&gt;
        </code>
      </pre>
      <h2>Complete Combination</h2>
      <figure
        class="extra class or"
        data-baz="23"
        data-foo="bar"
        id="custom-id"
        style="--wix:sum;--rotate:5deg;">
        <img
          alt="lorem ipsum"
          decoding="async"
          height="300"
          loading="lazy"
          src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp"
          width="800"
        />
        <figcaption>
          Exercitation qui <strong>exercitation</strong> dolor velit <em>aliqua</em> consectetur
          voluptate <a href="/consequat">consequat</a> labore elit non esse occaecat.
        </figcaption>
      </figure>
      <p>Expected result:</p>
      <pre lang="html">
        <code>
          &lt;figure id=&quot;custom-id&quot;class=&quot;extra class or&quot;
          data:foo=&quot;bar&quot; data:baz=&quot;23&quot;
          style=&quot;--wix:sum;--rotate:5deg;&quot;&gt;&lt;img decoding=&quot;async&quot;
          loading=&quot;lazy&quot; height=&quot;300&quot; width=&quot;800&quot; alt=&quot;lorem
          ipsum&quot;
          src=&quot;https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp&quot;
          /&gt;&lt;figcaption&gt;Exercitation qui &lt;strong&gt;exercitation&lt;/strong&gt; dolor
          velit &lt;em&gt;aliqua&lt;/em&gt; consectetur voluptate &lt;a
          href=&quot;/consequat&quot;&gt;consequat&lt;/a&gt; labore elit non esse
          occaecat.&lt;/figcaption&gt;&lt;/figure&gt;
        </code>
      </pre>
    </>
  )
}
