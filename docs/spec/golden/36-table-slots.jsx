export default function PendonView() { return (<>
<p>§8: every table layer reads the same head — <code>@@type[…](&quot;…&quot;)&#123;…&#125;%%numberType</code> —
so a slot pair, the specs items and a numbering marker reach the <code>&lt;table&gt;</code>, the
<code>&lt;caption&gt;</code>, a column&#39;s <code>&lt;th&gt;</code>, the <code>&lt;tbody&gt;</code>, a <code>&lt;tr&gt;</code>, a <code>&lt;td&gt;</code> and the
<code>&lt;tfoot&gt;</code> alike. Nothing is layer-specific except <em>where</em> the head sits:
</p>
<table class="t" id="tbl-slug" number={["A", "A"]} title="Table title" type="tableX">
  <caption {...{ "bracket.bare": "cap-slug" }} class="c" number={["A", "B"]} {...{ "paren.bare": "Cap title" }} type="captionX">A caption</caption>
  <thead>
    <tr>
      <th {...{ "bracket.bare": "th-slug" }} class="v" number={["A", "C"]} {...{ "paren.bare": "Th title" }} style="text-align: left;" type="thX">A</th>
      <th style="text-align: right;">B</th>
      <th>C</th>
    </tr>
  </thead>
  <tbody class="tb" type="tbodyX">
    <tr class="rw" type="rowX">
      <td {...{ "bracket.bare": "cl-slug" }} class="cl" number={["A", "D"]} {...{ "paren.bare": "Cl title" }} style="text-align: left;" type="cellX">1</td>
      <td style="text-align: right;">2</td>
      <td>3</td>
    </tr>
  </tbody>
  <tfoot {...{ "bracket.bare": "ft-slug" }} class="ft" number={["A", "E"]} {...{ "paren.bare": "Ft title" }} type="tfootX">
    <tr>
      <td colspan="2" style="text-align: left;">Total</td>
      <td>9</td>
    </tr>
  </tfoot>
</table>
<p>The column extras above stay on the <code>&lt;th&gt;</code> (§8/OPEN-E1): the <code>&lt;td&gt;</code> under it
keeps only the alignment, which is why <code>cellX</code> appears once.
</p>

</>); }
