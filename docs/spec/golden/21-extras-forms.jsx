export default function PendonView() { return (<>
<p>The bare <code>&#123;…&#125;</code> spelling is the same head as <code>@@&#123;…&#125;</code> (§3); the <code>@@</code> prefix only
adds the optional type marker (§4.1):
</p>
<p><a class="hero" href="/docs" id="bare">Bare head</a>
</p>
<p><a class="typed" href="/docs" type="anchorA">Typed head</a>
</p>
<p>A type may stand alone as a type-only head; the symbol after it stays text, so a
head at the end of a sentence does not gain a space (§4.1):
</p>
<p><a href="/docs" type="anchorA">Type only</a>.
</p>
<p><a href="/docs" type="anchorX">Type and dash</a>-tail
</p>
<p>An empty head carries nothing at all:
</p>
<p><a href="/docs">Empty</a>
</p>
<p><a href="/docs" type="anchorY">Empty typed</a>
</p>
<p>The same heads attach to a heading and to table cells (§7.4, §8):
</p>
<h3 class="fancy" id="slug-a" type="headingX">A typed heading</h3>
<table>
  <thead>
    <tr>
      <th class="v-top" style="text-align: left;" type="cellA">A</th>
      <th style="text-align: right;">B</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td class="v-top lead" style="text-align: left;" type="cellB">1</td>
      <td style="text-align: right;">2</td>
    </tr>
  </tbody>
</table>

</>); }
