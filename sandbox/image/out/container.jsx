export const frontmatter = {"cites":[]};
export default function PendonView() { return (<>
<h2>Extra Container</h2>
<h3>Paragraph</h3>
<p class="extra class or" data-baz="23" data-foo="bar" id="custom-id" style="--wix: sum; --rotate: 5deg;"><img alt="foo" height="800" src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp" width="300" /></p>
<p>Expected result:
</p>
<pre lang="html"><code>&lt;p id=&quot;custom-id&quot; class=&quot;extra class or&quot; data:foo=&quot;bar&quot; data:baz=&quot;23&quot; style=&quot;--wix:sum;--rotate:5deg;&quot;&gt;&lt;img width=&quot;300&quot; height=&quot;800&quot; alt=&quot;lorem ipsum&quot; src=&quot;https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp&quot; /&gt;&lt;/p&gt;
</code></pre>
<hr />
<p><img alt="foo" src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp" /></p>
<p>Expected result:
</p>
<pre lang="html"><code>&lt;p&gt;&lt;img alt=&quot;foo&quot; src=&quot;https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp&quot; /&gt;&lt;/p&gt;
</code></pre>
<h3>Division</h3>
<div><img alt="foo" src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp" /></div>
<p>Expected result:
</p>
<pre lang="html"><code>&lt;p&gt;&lt;img alt=&quot;foo&quot; src=&quot;https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp&quot; /&gt;&lt;/p&gt;
</code></pre>
<hr />
<div class="extra class or" data-baz="23" data-foo="bar" id="custom-id" style="--wix: sum; --rotate: 5deg;"><img alt="foo" decoding="async" height="800" src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp" width="300" /></div>
<p>Expected result:
</p>
<pre lang="html"><code>&lt;div  id=&quot;custom-id&quot; class=&quot;extra class or&quot; data:foo=&quot;bar&quot; data:baz=&quot;23&quot; style=&quot;--wix:sum;--rotate:5deg;&quot;&gt;&lt;img decoding=&quot;async&quot; width=&quot;300&quot; height=&quot;800&quot; alt=&quot;foo&quot; src=&quot;https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp&quot; /&gt;&lt;/div&gt;
</code></pre>

</>); }
