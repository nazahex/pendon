import Foo from "./Foo"
import Hint from "./Hint"

export default function PendonView() {
  return (
    <>
      <h1>Custom Syntax Demo</h1>
      <Foo argument="bar" attributes={{ qux: "fred", waldo: 8, isBar: true }}>
        <h2>I am inside a component</h2>
        <p>
          Let get <i>some</i> help then!
        </p>
        <Hint type="!">
          <p>I don&#39;t know what want to say!</p>
        </Hint>
        <pre lang="html">
          <code>
            &lt;!DOCTYPE html&gt; &lt;!-- comment --&gt; &lt;html lang=&quot;en&quot;&gt;
            &lt;head&gt; &lt;meta charset=&quot;utf-8&quot; /&gt; &lt;title&gt;Sample &amp;amp;
            Test&lt;/title&gt; &lt;style&gt; body &#123; color: #333; &#125; .container &gt;
            .item[id=&quot;x&quot;] &#123; margin: 10px; &#125; &lt;/style&gt; &lt;/head&gt;
            &lt;body&gt; &lt;div id=&quot;app&quot; class=&quot;container&quot;&gt;Hello &lt;span
            data-id=&quot;1&quot;&gt;world&lt;/span&gt;&lt;/div&gt; &lt;script&gt; const x = 1 + 2;
            // JS inline &lt;/script&gt; &lt;/body&gt; &lt;/html&gt;
          </code>
        </pre>
      </Foo>
    </>
  )
}
