import CustomTable, {TableBody, TableCell, TableFoot, TableHead, TableRow} from "@comp/shared/Table";
import { TableCaption } from '@comp/shared/Table';

export default function PendonView() { return (<>
<h2>Custom Table Demo</h2>
<p>Adipisicing mollit dolore aute consequat culpa nisi consectetur.
</p>
<CustomTable><TableCaption>Laporan Penjualan 2026</TableCaption><TableHead><TableRow><TableCell align="left" class="v-top" width="200px">Produk</TableCell><TableCell align="center" class="v-top">Stok</TableCell><TableCell align="right" class="v-bottom" width="30%">Harga</TableCell><TableCell align="center">Status</TableCell></TableRow></TableHead><TableBody><TableRow><TableCell align="left" class="v-top" width="200px">Laptop Pro</TableCell><TableCell align="center" class="v-top">15</TableCell><TableCell align="right" class="v-bottom" width="30%">15.000.000</TableCell><TableCell align="center">Tersedia</TableCell></TableRow><TableRow><TableCell align="left" class="v-top" width="200px" colspan="2">Mouse Wireless</TableCell><TableCell align="right" class="v-bottom" width="30%">250.000</TableCell><TableCell align="center">Tersedia</TableCell></TableRow><TableRow class="row-danger"><TableCell align="left" class="v-top" width="200px" rowspan="2">Keyboard Mekanikal</TableCell><TableCell align="center" class="v-top text-red">0</TableCell><TableCell align="right" class="v-bottom" width="30%">850.000</TableCell><TableCell align="center">Habis</TableCell></TableRow><TableRow><TableCell align="center" class="v-top">5</TableCell><TableCell align="right" class="v-bottom" width="30%">5.200.000</TableCell><TableCell align="center">Tersedia</TableCell></TableRow></TableBody><TableFoot><TableRow><TableCell align="left" class="v-top" width="200px" colspan="2">Total Inventaris</TableCell><TableCell align="right" class="v-bottom" width="30%">21.300.000</TableCell><TableCell align="center">-</TableCell></TableRow></TableFoot></CustomTable><h2>Lorem</h2>
<p>Anim ea dolore sunt cupidatat nisi nisi dolor amet do.
</p>

</>); }
