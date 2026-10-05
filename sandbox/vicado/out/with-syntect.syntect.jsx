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
        <code
          innerHTML={
            "<p>fn main() {</p><p>    <b>for</b> i <b>in</b> 0<b>..</b>3 {</p><p>        println!(<i><i>&quot;</i>{}<i>&quot;</i></i>, i);</p><p>    }</p><p>}</p>"
          }
        />
      </pre>
      <pre lang="html">
        <code
          innerHTML={
            "<p><b><b>&lt;</b><b>section</b> <span><em>class</em><mark>=</mark></span><span><i><i>&quot;</i></i></span><span><i><abbr>demo</abbr><i>&quot;</i></i></span><b>&gt;</b></b></p><p>  <b><b>&lt;</b><b>h2</b><b>&gt;</b></b>Regular HTML block<b><b>&lt;/</b><b>h2</b><b>&gt;</b></b></p><p>  <b><b>&lt;</b><b>p</b><b>&gt;</b></b>This should stay as regular highlighted code.<b><b>&lt;/</b><b>p</b><b>&gt;</b></b></p><p><b><b>&lt;/</b><b>section</b><b>&gt;</b></b></p>"
          }
        />
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
        <code
          innerHTML={
            "<p>type User <b>=</b> { id: number; name: string }</p><p>const users: User[] <b>=</b> [{ id: 1, name: <i><i>&quot;</i>A<i>&quot;</i></i> }]</p><p>console.log(users)</p>"
          }
        />
      </pre>
    </>
  )
}
