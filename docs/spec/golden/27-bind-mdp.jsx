import Figure from '@comp/Figure'

export const frontmatter = {"references":{"book":{"id":"book","title":"A Book","type":"book"}}};
export const references = {"book":{"id":"book","title":"A Book","type":"book"}};
export const citations = [{"citation_number":1,"locator":"hlm. 45","reference_id":"book"},{"citation_number":2,"reference_id":"book"}];
export default function PendonView() { return (<>
<p>An <code>mdp</code> payload is a <strong>sub-document</strong>: it is parsed and run through the <em>same</em>
chain as the page, so every enabled plugin reaches inside it — not just
Markdown. The payload becomes a fragment prop, and each renderer decides how to
carry a node list (Solid: <code>(&lt;&gt;&#123;…&#125;&lt;/&gt;)</code>).
</p>
<p><a body={(<><h2 id="payload-heading">Payload heading</h2>
<p>A <a href="/Nested_Wiki" title="Nested Wiki">Nested Wiki</a> link, a citation <sup class="cite-ref"><a href="#citeref-1-book" id="cite-1">[1]</a></sup> and inline math <span class="latex latex-inline" innerHTML={`<span class="katex"><span class="katex-mathml"><math xmlns="http://www.w3.org/1998/Math/MathML"><semantics><mrow><msup><mi>x</mi><mn>2</mn></msup></mrow><annotation encoding="application/x-tex">x^2</annotation></semantics></math></span><span class="katex-html" aria-hidden="true"><span class="base"><span class="strut" style="height:0.8141em;"></span><span class="mord"><span class="mord mathnormal">x</span><span class="msupsub"><span class="vlist-t"><span class="vlist-r"><span class="vlist" style="height:0.8141em;"><span style="top:-3.063em;margin-right:0.05em;"><span class="pstrut" style="height:2.7em;"></span><span class="sizing reset-size6 size3 mtight"><span class="mord mtight">2</span></span></span></span></span></span></span></span></span></span></span>`}></span>
all resolve against the page&#39;s own state.
</p>
<Figure class="wide" container="figure" type="figureX"><img alt="Chart" decoding="async" loading="lazy" src="https://example.com/chart.png" width="400" />A caption from the payload.</Figure>
<table>
  <thead>
    <tr>
      <th>Name</th>
      <th style="text-align: right;">Value</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>a</td>
      <td style="text-align: right;">1</td>
    </tr>
  </tbody>
</table>
</>)} href="/docs" type="anchorA">Card</a>
</p>
<p>The inline form drops the payload&#39;s own <code>&lt;p&gt;</code> wrapper, so a payload written on
the head line contributes inline nodes only:
</p>
<p><a href="/docs" note={(<>See <a href="/Wireless" title="Wireless">Wireless</a> and <sup class="cite-ref"><a href="#citeref-2-book" id="cite-2">[2]</a></sup>.</>)} type="anchorA">Note</a>
</p>

</>); }
