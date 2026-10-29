export const headings = [{"id":"slug-section","level":2,"text":"A decorated heading"}];
export default function PendonView() { return (<>
<section depth="0">
<p>§9.1/D-10 rule 2 (S2-5.2): one decorator, one owner. The claim order is
section → table → list → blockquote → paragraph, and the owner is decided by
<strong>what the block is</strong>, never by which plugin happens to read the stream first:
</p>
</section>
<section class="lead" depth="2" id="slug-section">
<h2>A decorated heading</h2>
<p>The decorator above the heading decorates the <strong>section</strong>, not the heading
(§9.5): the heading carries its own extras on the <code>#</code> run, so a second head
above it would be redundant.
</p>
<ul class="card" id="list-1" type="unorderedA"><li>one</li>
<li>two</li>
</ul><blockquote class="note" type="aside"><p>a decorated quote
</p>
</blockquote><p class="lead" id="p1">A decorated paragraph.
</p>
<p>A table is claimed by <code>table</code>, not by <code>markdown</code> — even though <code>table</code> runs first
and the block below is still a <code>Paragraph</code> when it reads the stream:
</p>
<table class="grid" id="sales" type="tableA">
  <thead>
    <tr>
      <th>Produk</th>
      <th style="text-align: right;">Harga</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>Laptop</td>
      <td style="text-align: right;">10000</td>
    </tr>
  </tbody>
</table>
</section>

</>); }
