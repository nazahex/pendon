import { Vicado } from "vicado"

export default function PendonView() {
  return (
    <>
      <h1>Vicado With Regular Codeblock</h1>
      <p>Dokumen ini menunjukkan Vicado berjalan bersamaan dengan code fence reguler.</p>
      <h2>Vicado Fence</h2>
      <Vicado
        language="typescript"
        code={"function sum(a: number, b: number) {\n  return a + b\n}"}
        class="panel"
        id="vicado-one"
        baz={89}
        enabled={true}
        foo="bar"
        mount="visible"
      />
      <h2>Regular Fence (Should Be Highlighted By Syntect)</h2>
      <pre lang="rust">
        <code>
          fn main() &#123; for i in 0..3 &#123; println!(&quot;&#123;&#125;&quot;, i); &#125; &#125;
        </code>
      </pre>
      <pre lang="html">
        <code>
          &lt;section class=&quot;demo&quot;&gt; &lt;h2&gt;Regular HTML block&lt;/h2&gt;
          &lt;p&gt;This should stay as regular highlighted code.&lt;/p&gt; &lt;/section&gt;
        </code>
      </pre>
      <h2>Another Vicado Fence</h2>
      <Vicado
        language="js"
        code={
          'const names = ["ana", "budi", "caca"]\nconsole.log(names.map((n) => n.toUpperCase()))'
        }
        compact={true}
        mount="lazy"
      />
      <h2>Regular Fence Again</h2>
      <pre lang="ts">
        <code>
          type User = &#123; id: number; name: string &#125; const users: User[] = [&#123; id: 1,
          name: &quot;A&quot; &#125;] console.log(users)
        </code>
      </pre>
    </>
  )
}
