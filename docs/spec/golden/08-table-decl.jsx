import { TableDefault } from '@comp/content/Table'

export default function PendonView() { return (<>
<p>The §8 declaration line carries the <code>&lt;table&gt;</code> extras and the head (<code>[slug]</code>,
<code>(&quot;title&quot;)</code>); the caption line is <code>|| extras content ||</code>:
</p>
<p>|-<a href="&quot;Laporan Penjualan 2026&quot;">sales-2026</a>@@tableX&#123;.striped, sortable: true&#125;-|
|| @@captionX&#123;.caption-note&#125; Laporan Penjualan 2026 ||
</p>
<TableDefault><thead><tr><th style="text-align: left;">Produk</th><th style="text-align: right;">Harga</th></tr></thead><tbody><tr><td style="text-align: left;">Laptop Pro</td><td style="text-align: right;">15.000.000</td></tr></tbody></TableDefault><p>A marker with no configured entry keeps the layer default; a table without a
declaration line keeps working and its layers fall back to plain elements:
</p>
<TableDefault><thead><tr><th>Kolom A</th><th>Kolom B</th></tr></thead><tbody><tr><td>satu</td><td>dua</td></tr><tr><td>===</td><td></td></tr><tr><td>total</td><td>dua</td></tr></tbody></TableDefault>
</>); }
