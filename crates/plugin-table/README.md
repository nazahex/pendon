Berikut adalah draf README untuk `pendon-plugin-table`. Saya menyusunnya dalam bahasa Inggris agar selaras dengan dokumentasi `plugin-cite` Anda, namun menerapkan prinsip editorial premium: paragraf yang sangat ringkas, ide yang terfokus, dan tipografi yang memandu mata pembaca.

---

# pendon-plugin-table

Advanced, AsciiDoc-inspired table syntax plugin for Pendon.

`pendon-plugin-table` extends standard Markdown tables with a highly expressive syntax for complex data layouts. It intercepts raw paragraph blocks, builds a 2D spatial grid to calculate merges, and seamlessly integrates with the shared inline pipeline. This allows you to embed rich media, citations, and wiki links directly inside table cells while supporting custom Solid components.

## What This Plugin Does

- Parses **custom table syntax** including captions, column widths, and granular attributes.
- Builds a **2D matrix grid** in memory to accurately calculate `colspan` and `rowspan` merges.
- Intercepts table blocks **before** the standard Markdown parser runs, ensuring custom syntax is never mangled.
- Routes cell content and captions through the **shared inline pipeline** for full Markdown, citation, and image support.
- Supports **four distinct custom Solid components** (Table, Caption, Row, Cell) for complete frontend control.
- Provides a **graceful fallback** to standard GFM tables if the custom syntax is invalid or incomplete.

## Recommended Plugin Order

```text
micromatter,img,table,cite,wiki,anchor,markdown
```

**Why this order matters:**

1. `micromatter` must run first to extract frontmatter for citation contexts.
2. `img` runs before `table` so that standalone image paragraphs are processed before the table interceptor scans the document.
3. `table` runs before `markdown` to hijack raw paragraph text and prevent the standard parser from misinterpreting custom delimiters.
4. `cite`, `wiki`, and `anchor` are registered in the inline pipeline and process cell content internally.
5. `markdown` runs last to handle any remaining inline formatting that the table plugin passes through.

Example task config:

```toml
[[task]]
input = "./src/[slug].md"
output = "./out/[slug].jsx"
plugin = "micromatter,img,table,cite,wiki,anchor,markdown"
format = "solid"
```

## Custom Syntax Overview

The plugin replaces rigid Markdown tables with a flexible, attribute-rich structure.

### Caption & Table Attributes

Place the caption and global table attributes on the line immediately preceding the table header.

```md
[Laporan Penjualan 2026][.striped,#sales-table]{sortable: "true", qux: true}
| Produk | Stok | Harga |
```

- **Caption text** goes in the first bracket pair.
- **Classes and IDs** go in the second bracket pair.
- **Extra attributes** go in the curly braces.

### Alignment & Column Specs

Column alignment, width, and default classes are controlled via the delimiter row.

```md
| :---(200px)[.v-top] | :---:[.v-top] | ---:(30%)[.v-bottom] |
```

- **Horizontal alignment** follows GFM rules (`:---` left, `:---:` center, `---:` right).
- **Width specifiers** use parentheses `(200px)` immediately after the dashes.
- **Column classes** use brackets `[.v-top]` to inject attributes into every cell in that column.

### Colspan & Rowspan Merging

Use special single-character markers to merge cells across the 2D grid.

- **Colspan (`>`)**: Merges the current cell into the cell on its **left**.
- **Rowspan (`^`)**: Merges the current cell into the cell directly **above** it.

```md
| Mouse Wireless | > | 250.000 |
| ^ | 5 | 5.200.000 |
```

### Cell & Row Attributes

Inject granular attributes directly into specific cells or entire rows.

```md
| Keyboard | 0 [.text-red] | 850.000 | Habis | -[.row-danger]
```

- **Cell attributes** are placed inside the cell, separated by a space.
- **Row attributes** are placed at the very end of the row, optionally prefixed with a dash `-`.

### Table Footer

Separate the footer rows from the body using a strict delimiter line.

```md
|===|
| Total | > | 21.300.000 |
```

## Custom Solid Components

Replace the default HTML output with custom Solid components for each structural layer.

```toml
[task.table.custom_node.table]
name = "CustomTable"
template = "<CustomTable id=\"{attrs.id}\" class=\"{attrs.class}\">{children}</CustomTable>"

[[task.table.custom_node.table.imports]]
module = "@/components/CustomTable"
default = "CustomTable"

[task.table.custom_node.cell]
name = "TableCell"
template = "<TableCell align=\"{attrs.align}\" class=\"{attrs.class}\" colspan=\"{attrs.colspan}\" rowspan=\"{attrs.rowspan}\" width=\"{attrs.width}\">{children}</TableCell>"
```

### Available Template Attributes

| Attribute         | Description                                       |
| :---------------- | :------------------------------------------------ |
| `{attrs.id}`      | Custom ID from the caption or row block.          |
| `{attrs.class}`   | Space-separated merged class list.                |
| `{attrs.align}`   | Horizontal alignment (`left`, `center`, `right`). |
| `{attrs.width}`   | Explicit width from the column delimiter.         |
| `{attrs.colspan}` | Calculated horizontal span (only emitted if > 1). |
| `{attrs.rowspan}` | Calculated vertical span (only emitted if > 1).   |
| `{children}`      | Rendered inline JSX nodes for the cell content.   |

## Inline Pipeline Integration

This is where the plugin truly shines. Text inside captions and cells is not treated as dead strings.

It is routed through the **shared inline pipeline** before final rendering. This means your table cells natively support:

- **Wiki links**: `[[Target | Label]]`
- **Citations**: `[^^]("ref-id", "loc")`
- **Advanced images**: `~?!!h300[Alt](src)`
- **Standard Markdown**: `**bold**`, `*italic*`, `` `code` ``

The plugin uses **bracket-depth tracking** to ensure that pipe characters `|` inside wiki links or markdown links do not accidentally split your table columns.

## Behavioral Notes

- The plugin only intercepts **root-level paragraphs**. Tables nested inside blockquotes or lists will fall back to standard Markdown parsing.
- If the delimiter row is missing or malformed, the plugin **silently aborts** and lets the standard Markdown parser handle it as a GFM table.
- Empty attributes (e.g., `colspan="1"`) are automatically stripped from the AST to keep the JSX output clean.
- Style entries starting with `--` in attribute blocks are automatically merged into a single `style` string.

## Scope and Limitations

- **No nested tables**: Tables inside lists or blockquotes are not supported in v1.0 due to state-machine complexity.
- **Strict delimiter requirement**: A valid delimiter row (`| --- |`) is mandatory for the custom parser to engage.
- **Colspan logic**: The `>` marker always pulls the cell to its left. It cannot push to the right.
- **Row attributes**: Must be placed at the very end of the row line.

## License

MIT
