## Example

[Laporan Penjualan 2026][.striped,#sales-table,.foo]{sortable: "true", qux: true}
| Produk | Stok | Harga | Status |
| :---(200px)[.v-top] | :---:[.v-top] | ---:(30%)[.v-bottom] | :---: |
| Laptop Pro | 15 | 15.000.000 | Tersedia |
| Mouse Wireless | > | 250.000 | Tersedia |
| Keyboard Mekanikal | 0 [.text-red] | 850.000 | Habis | [.row-danger]
| ^ | 5 | 5.200.000 | Tersedia |
|===|
| Total Inventaris | > | 21.300.000 | - |

Expected Output:

```html
<table id="sales-table" class="striped foo" sortable="true" qux=true>
  <caption>Laporan Penjualan 2026</caption>
  <thead>
    <tr>
      <th class="v-top" style="text-align: left; width: 200px;">Produk</th>
      <th class="v-top" style="text-align: center;">Stok</th>
      <th class="v-bottom" style="text-align: right; width: 30%;">Harga</th>
      <th style="text-align: center;">Status</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td class="v-top" style="text-align: left;">Laptop Pro</td>
      <td class="v-top" style="text-align: center;">15</td>
      <td class="v-bottom" style="text-align: right;">15.000.000</td>
      <td style="text-align: center;">Tersedia</td>
    </tr>
    <tr>
      <td class="v-top" style="text-align: left;">Mouse Wireless</td>
      <td class="v-top" style="text-align: center;" colspan="2">250.000</td>
      <td style="text-align: center;">Tersedia</td>
    </tr>
    <tr class="row-danger">
      <td class="v-top" style="text-align: left;">Keyboard Mekanikal</td>
      <td class="v-top text-red" style="text-align: center;">0</td>
      <td class="v-bottom" style="text-align: right;">850.000</td>
      <td style="text-align: center;">Habis</td>
    </tr>
    <tr>
      <td class="v-top" style="text-align: left;" rowspan="2">Keyboard Mekanikal</td>
      <td class="v-top" style="text-align: center;">5</td>
      <td class="v-bottom" style="text-align: right;">5.200.000</td>
      <td style="text-align: center;">Tersedia</td>
    </tr>
  </tbody>
  <tfoot>
    <tr>
      <td class="v-top" style="text-align: left;" colspan="2">Total Inventaris</td>
      <td class="v-bottom" style="text-align: right;">21.300.000</td>
      <td style="text-align: center;">-</td>
    </tr>
  </tfoot>
</table>
```

## Proposal

### Custom Syntax

- **Caption & Table Attributes**: Diletakkan tepat di baris atas tabel dengan skema wajib berantai `[Caption][.class1,#id]{attr: "val"}`.
  - Jika hanya butuh _class_/_id_: `[][.foo,#bar]`
  - Jika hanya butuh _extra attrs_: `[][]{att1: true}`
  - Jika hanya butuh _caption_: `[Caption Text]`
- **Colspan (Horizontal Merge)**: Gunakan `>` pada sel untuk menyatukan sel ke sel di sebelah kirinya.
- **Rowspan (Vertical Merge)**: Gunakan `^` pada sel untuk menyatukan sel ke sel tepat di atasnya.
- **Row Attributes**: Diletakkan di ujung kanan baris tabel setelah `|`. Tidak wajib berantai, bisa langsung `[.row-class]` atau `{data-row: "true"}`.
- **Cell Attributes**: Diletakkan di dalam sel setelah teks. Tidak wajib berantai, bisa langsung `[.cell-class]` atau `{role: "button"}`.
- **Multi-Line Cells**: Escape `\n` di dalam sel diperluas menjadi baris nyata sebelum sel di-parse, sehingga satu sel dapat memuat konten blok seperti list (`| - A\n- B |`) atau beberapa paragraf.
- **Table Footer (`<tfoot>`)**: Dipisahkan menggunakan pembatas `|===|` sebelum baris footer.

### Alignment & Column Specs

Alignment, _width_, dan _classes/attrs_ kolom dikontrol via _delimiter row_ (`| --- |`):

- **Horizontal Alignment**: Sesuai GFM (`:---` left, `:---:` center, `---:` right).
- **Width Specifier**: Tanda kurung `(...)` tepat setelah delimiter untuk CSS `width`.
- **Column Classes & Extra Attrs**: Menggunakan `[.class]` dan `{attr: "val"}` untuk menginjeksi atribut ke seluruh sel pada kolom tersebut.
- Vertical alignment disarankan diatur via class (misal: `[.v-top]`).

Contoh:
`| :---(200px)[.v-top] | :---:[.v-top] | ---:(30%)[.v-bottom]{foo: "bar"} |`

### Extra Attr & Selectors

Atribut ekstra di-output secara presisi sesuai key-value (tidak otomatis diberi prefiks `data-`):

- Class & ID: `[.class-a,#id]`
- Extra Attributes: `{foo: "bar", role: "gridcell"}`

Contoh penerapan:

```md
[Data Karyawan][.table-bordered]{sortable: "true"}

| ID  | Nama                              | Peran     |
| :-- | :-------------------------------- | :-------- |
| 001 | Alice [.highlight]{role: "admin"} | Developer |
| 002 | Bob                               | Designer  |
```

Expected Output:

```html
<table class="table-bordered" sortable="true">
  <caption>Data Karyawan</caption>
  <thead>
    <tr>
      <th style="text-align: left;">ID</th>
      <th style="text-align: left;">Nama</th>
      <th style="text-align: left;">Peran</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td style="text-align: left;">001</td>
      <td style="text-align: left;" class="highlight" role="admin">Alice</td>
      <td style="text-align: left;">Developer</td>
    </tr>
    <tr class="bg-gray">
      <td style="text-align: left;">002</td>
      <td style="text-align: left;">Bob</td>
      <td style="text-align: left;">Designer</td>
    </tr>
  </tbody>
</table>
```

### Custom Node

Memfasilitasi arsitektur modular agar pengembang (khususnya Solid.js) dapat merender node tabel menggunakan komponen JSX kustom melalui AST Transformer (`plugin-table`).

- **AST Node Type**: Memetakan struktur tabel menjadi custom AST node (`TableNode`, `TableCaptionNode`, `TableHeadNode`, `TableBodyNode`, `TableFootNode`, `TableRowNode`, `TableCellNode`).
- **JSX Mapping Spec**:
  ```tsx
  export const customTableComponents = {
    Table: (props) => <MyTable class="{props.className}" id="{props.id}" {...props.extraAttrs}>{props.children}</MyTable>,
    Caption: (props) => <MyCaption>{props.children}</MyCaption>,
    Head: (props) => <MyThead>{props.children}</MyThead>,
    Body: (props) => <MyTbody>{props.children}</MyTbody>,
    Foot: (props) => <MyTfoot>{props.children}</MyTfoot>,
    Row: (props) => <MyTr class="{props.className}" {...props.extraAttrs}>{props.children}</MyTr>,
    Cell: (props) => (
      <MyTd align="{props.align}" class="{props.className}" colspan="{props.colspan}" rowspan="{props.rowspan}" width="{props.width}" {...props.extraAttrs}>
        {props.children}
      </MyTd>
    )
  };
  ```

### Custom Node Layer

Ketujuh layer bersifat **opsional dan independen**: layer yang tidak dikonfigurasi tetap
dirender sebagai elemen biasa (`<table>`, `<caption>`, `<thead>`, `<tbody>`, `<tfoot>`,
`<tr>`, `<td>`/`<th>`), sehingga konfigurasi parsial tetap menghasilkan output valid.

```toml
[task.table.custom_node]
# Imports yang berlaku untuk semua komponen di bawahnya.
imports = ["import { TableCaption } from '@/components/table';"]

[task.table.custom_node.table]
name = "CustomTable"
template = "<CustomTable id=\"{attrs.id}\" class=\"{attrs.class}\">{children}</CustomTable>"

[task.table.custom_node.thead]
name = "TableHead"
template = "<TableHead>{children}</TableHead>"

[task.table.custom_node.tbody]
name = "TableBody"
template = "<TableBody>{children}</TableBody>"

[task.table.custom_node.tfoot]
name = "TableFoot"
template = "<TableFoot>{children}</TableFoot>"

[task.table.custom_node.row]
name = "TableRow"
template = "<TableRow class=\"{attrs.class}\">{children}</TableRow>"

[task.table.custom_node.cell]
name = "TableCell"
template = "<TableCell align=\"{attrs.align}\" colspan=\"{attrs.colspan}\" rowspan=\"{attrs.rowspan}\">{children}</TableCell>"
```

Komponen section (`thead`/`tbody`/`tfoot`) tidak menerima atribut; template-nya cukup
memakai `{children}`.

### Imports Syntax

Seluruh plugin level task (`cite`, `img`, `anchor`, `heading`, `table`) memakai satu
sintaks `imports` yang sama. Setiap entri berupa _raw import line_ atau tabel
terstruktur, dan hasilnya dideduplikasi per `module` menjadi satu baris `import`.

```toml
imports = ["import { TableCaption } from '@/components/table';"]

[[task.table.custom_node.cell.imports]]
module = "@/components/table"
default = "TableCell" # opsional
names = ["TableHead"] # opsional
```

### Spread Attributes

Gunakan operator spread `{...attrs}` untuk meneruskan seluruh atribut — termasuk key
tambahan dari blok `{key: "value"}` — sebagai props komponen. Key yang sudah ditulis
eksplisit di template tidak diduplikasi:

```toml
[task.table.custom_node.row]
name = "TableRow"
template = "<TableRow {...attrs}>{children}</TableRow>"
```
