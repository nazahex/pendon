import { TableCaption } from '@comp/Table'
import { TableDefault } from '@comp/Table'

export default function PendonView() { return (<>
<p>§8: the <code>&lt;table&gt;</code> extras are the §9.1 decorator line above the table — the
layers inside it (caption, column, row, cell, foot) carry their own heads, so no
line <em>inside</em> a table configures the <code>&lt;table&gt;</code> itself:
</p>
<TableDefault class="striped" id="sales-2026" sortable={true} title="Laporan Penjualan 2026" type="tableX"><TableCaption class="caption-note" type="captionX">Laporan Penjualan 2026</TableCaption><thead><tr><th style="text-align: left;">Produk</th><th style="text-align: right;">Harga</th></tr></thead><tbody><tr><td style="text-align: left;">Laptop Pro</td><td style="text-align: right;">15.000.000</td></tr></tbody></TableDefault><p>A marker with no configured entry keeps the layer default; a table with no
decorator keeps working and its layers fall back to plain elements:
</p>
<TableDefault><thead><tr><th>Kolom A</th><th>Kolom B</th></tr></thead><tbody><tr><td>satu</td><td>dua</td></tr><tr><td>===</td><td></td></tr><tr><td>total</td><td>dua</td></tr></tbody></TableDefault>
</>); }
