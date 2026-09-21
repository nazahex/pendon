use crate::attrs::{merge_attrs, AttrSpec};
use crate::parser::{Align, ColumnSpec, RowSpec};

#[derive(Debug, Clone)]
pub struct ProcessedCell {
    pub text: String,
    pub attrs: AttrSpec,
    pub colspan: usize,
    pub rowspan: usize,
    pub align: Align,
    pub width: Option<String>,
    pub is_hidden: bool,
}

#[derive(Debug, Clone)]
pub struct ProcessedRow {
    pub cells: Vec<ProcessedCell>,
    pub attrs: AttrSpec,
}

pub fn process_grid(
    columns: &[ColumnSpec],
    body_rows: &[RowSpec],
    footer_rows: &[RowSpec],
) -> (Vec<ProcessedRow>, Vec<ProcessedRow>) {
    let num_cols = columns.len();
    let processed_body = process_row_group(body_rows, columns, num_cols);
    let processed_footer = process_row_group(footer_rows, columns, num_cols);
    (processed_body, processed_footer)
}

fn process_row_group(
    rows: &[RowSpec],
    columns: &[ColumnSpec],
    num_cols: usize,
) -> Vec<ProcessedRow> {
    if rows.is_empty() {
        return Vec::new();
    }

    let num_rows = rows.len();
    let mut grid: Vec<Vec<ProcessedCell>> = Vec::new();

    for _ in 0..num_rows {
        let mut row = Vec::new();
        for _ in 0..num_cols {
            row.push(ProcessedCell {
                text: String::new(),
                attrs: AttrSpec::default(),
                colspan: 1,
                rowspan: 1,
                align: Align::None,
                width: None,
                is_hidden: false,
            });
        }
        grid.push(row);
    }

    for (row_idx, row_spec) in rows.iter().enumerate() {
        let mut col_idx = 0;

        for cell_spec in &row_spec.cells {
            while col_idx < num_cols && grid[row_idx][col_idx].is_hidden {
                col_idx += 1;
            }
            if col_idx >= num_cols {
                break;
            }

            let col_spec = columns.get(col_idx);

            if cell_spec.is_colspan_marker {
                // > combine this cell with the cell to the LEFT
                if col_idx > 0 {
                    grid[row_idx][col_idx - 1].colspan += 1;
                    grid[row_idx][col_idx].is_hidden = true;
                }
            } else if cell_spec.is_rowspan_marker {
                // ^ combine this cell with the cell to the ABOVE
                if row_idx > 0 {
                    let mut source_row = row_idx - 1;
                    while source_row > 0 && grid[source_row][col_idx].is_hidden {
                        source_row -= 1;
                    }
                    grid[source_row][col_idx].rowspan += 1;
                    grid[row_idx][col_idx].is_hidden = true;
                }
            } else {
                let mut cell_attrs = col_spec.map(|c| c.attrs.clone()).unwrap_or_default();
                merge_attrs(&mut cell_attrs, &cell_spec.attrs);

                // Important: Preserve existing colspan and rowspan values if they were already set by previous cells
                let existing_colspan = grid[row_idx][col_idx].colspan;
                let existing_rowspan = grid[row_idx][col_idx].rowspan;

                grid[row_idx][col_idx] = ProcessedCell {
                    text: cell_spec.text.clone(),
                    attrs: cell_attrs,
                    colspan: existing_colspan,
                    rowspan: existing_rowspan,
                    align: col_spec.map(|c| c.align).unwrap_or(Align::None),
                    width: col_spec.and_then(|c| c.width.clone()),
                    is_hidden: false,
                };
            }
            col_idx += 1;
        }
    }

    let mut result = Vec::new();
    for (row_idx, row_spec) in rows.iter().enumerate() {
        let mut cells = Vec::new();
        for col_idx in 0..num_cols {
            let cell = &grid[row_idx][col_idx];
            if !cell.is_hidden {
                cells.push(cell.clone());
            }
        }
        result.push(ProcessedRow {
            cells,
            attrs: row_spec.attrs.clone(),
        });
    }

    result
}
