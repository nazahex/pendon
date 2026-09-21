use crate::attrs::AttrSpec;
use crate::grid::{process_grid, ProcessedRow};
use crate::parser::{Align, TableBlock};
use pendon_core::{parse, Options, Pipeline};
use pendon_plugin_markdown::process as process_markdown;

pub fn render_table_html(table_block: &TableBlock, inline_pipeline: &Pipeline) -> String {
    let (body_rows, footer_rows) = process_grid(
        &table_block.columns,
        &table_block.body_rows,
        &table_block.footer_rows,
    );

    let mut out = String::new();

    // Open <table> with attributes from caption
    out.push_str("<table");
    if let Some(caption) = &table_block.caption {
        push_common_attrs(&mut out, &caption.attrs);
    }
    out.push_str(">\n");

    // Caption
    if let Some(caption) = &table_block.caption {
        out.push_str("  <caption>");
        out.push_str(&render_inline_text(&caption.text, inline_pipeline));
        out.push_str("</caption>\n");
    }

    // Thead
    out.push_str("  <thead>\n    <tr>\n");
    for col_spec in &table_block.columns {
        out.push_str("      <th");
        push_cell_attrs(
            &mut out,
            &col_spec.attrs,
            col_spec.align,
            col_spec.width.as_deref(),
        );
        out.push('>');
        out.push_str(&render_inline_text(&col_spec.header_text, inline_pipeline));
        out.push_str("</th>\n");
    }
    out.push_str("    </tr>\n  </thead>\n");

    // Tbody
    if !body_rows.is_empty() {
        out.push_str("  <tbody>\n");
        for row in &body_rows {
            render_row_html(&mut out, row, inline_pipeline, "    ");
        }
        out.push_str("  </tbody>\n");
    }

    // Tfoot
    if !footer_rows.is_empty() {
        out.push_str("  <tfoot>\n");
        for row in &footer_rows {
            render_row_html(&mut out, row, inline_pipeline, "    ");
        }
        out.push_str("  </tfoot>\n");
    }

    out.push_str("</table>\n");
    out
}

fn render_row_html(out: &mut String, row: &ProcessedRow, inline_pipeline: &Pipeline, indent: &str) {
    out.push_str(indent);
    out.push_str("<tr");
    push_common_attrs(out, &row.attrs);
    out.push_str(">\n");

    for cell in &row.cells {
        out.push_str(indent);
        out.push_str("  <td");
        push_cell_attrs(out, &cell.attrs, cell.align, cell.width.as_deref());
        if cell.colspan > 1 {
            out.push_str(" colspan=\"");
            out.push_str(&cell.colspan.to_string());
            out.push('"');
        }
        if cell.rowspan > 1 {
            out.push_str(" rowspan=\"");
            out.push_str(&cell.rowspan.to_string());
            out.push('"');
        }
        out.push('>');
        out.push_str(&render_inline_text(&cell.text, inline_pipeline));
        out.push_str("</td>\n");
    }
    out.push_str(indent);
    out.push_str("</tr>\n");
}

fn push_common_attrs(out: &mut String, attrs: &AttrSpec) {
    if let Some(id) = attrs.id.as_deref() {
        out.push_str(" id=\"");
        escape_html(id, out);
        out.push('"');
    }
    if !attrs.classes.is_empty() {
        out.push_str(" class=\"");
        escape_html(&attrs.classes.join(" "), out);
        out.push('"');
    }
    for (k, v) in &attrs.extra {
        if k.starts_with("--") {
            continue; // Style entries are handled in push_cell_attrs
        }
        out.push(' ');
        escape_html(k, out);
        out.push_str("=\"");
        escape_html(v, out);
        out.push('"');
    }
}

fn push_cell_attrs(out: &mut String, attrs: &AttrSpec, align: Align, width: Option<&str>) {
    // ID
    if let Some(id) = attrs.id.as_deref() {
        out.push_str(" id=\"");
        escape_html(id, out);
        out.push('"');
    }

    // Class
    if !attrs.classes.is_empty() {
        out.push_str(" class=\"");
        escape_html(&attrs.classes.join(" "), out);
        out.push('"');
    }

    // Extra attrs (not style)
    for (k, v) in &attrs.extra {
        if k.starts_with("--") {
            continue;
        }
        out.push(' ');
        escape_html(k, out);
        out.push_str("=\"");
        escape_html(v, out);
        out.push('"');
    }

    // Collect style entries
    let mut styles = Vec::new();

    // Align
    match align {
        Align::Left => styles.push("text-align: left".to_string()),
        Align::Center => styles.push("text-align: center".to_string()),
        Align::Right => styles.push("text-align: right".to_string()),
        Align::None => {}
    }

    // Width
    if let Some(w) = width {
        styles.push(format!("width: {}", w));
    }

    // Style entries from attrs.extra (starting with --)
    for (k, v) in &attrs.extra {
        if k.starts_with("--") {
            styles.push(format!("{}:{}", k, v));
        }
    }

    // Emit style if exists
    if !styles.is_empty() {
        out.push_str(" style=\"");
        escape_html(&styles.join("; "), out);
        out.push_str(";\"");
    }
}

fn render_inline_text(text: &str, inline_pipeline: &Pipeline) -> String {
    if text.trim().is_empty() {
        return String::new();
    }
    let parsed = parse(text, &Options::default());
    let processed = inline_pipeline.run(parsed);
    let processed = process_markdown(&processed);
    let rendered = pendon_renderer_html::render_html(&processed);
    let trimmed = rendered.trim();
    if let Some(inner) = trimmed
        .strip_prefix("<p>")
        .and_then(|s| s.strip_suffix("</p>"))
    {
        return inner.trim().to_string();
    }
    trimmed.to_string()
}

fn escape_html(s: &str, out: &mut String) {
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
}
