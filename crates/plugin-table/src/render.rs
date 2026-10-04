use crate::attrs::AttrSpec;
use crate::grid::{process_grid, ProcessedRow};
use crate::parser::{Align, TableBlock};
use pendon_core::{
    element_close, element_open, parse, raw_inline, Event, InlinePipeline, NodeKind, Options,
};
use pendon_plugin_markdown::process as process_markdown;

/// Emits the whole table as structured [`NodeKind::Element`] events.
///
/// Emitting real events (instead of one raw HTML string) keeps custom
/// components alive inside cells: citations, inline custom plugins and custom
/// img/anchor/table components all nest correctly in every renderer.
pub fn emit_table_elements<C, P>(
    table_block: &TableBlock,
    inline_pipeline: &P,
    context: &mut C,
    out: &mut Vec<Event>,
) where
    P: InlinePipeline<C>,
{
    let (body_rows, footer_rows) = process_grid(
        &table_block.columns,
        &table_block.body_rows,
        &table_block.footer_rows,
    );

    open(out, "table");
    if let Some(caption) = &table_block.caption {
        push_common_attrs(out, &caption.attrs);
    }
    text(out, "\n");

    if let Some(caption) = &table_block.caption {
        text(out, "  ");
        open(out, "caption");
        let rendered = render_inline_events(&caption.text, inline_pipeline, context);
        out.extend(rendered);
        close(out, "caption");
        text(out, "\n");
    }

    // Thead
    text(out, "  ");
    open(out, "thead");
    text(out, "\n    ");
    open(out, "tr");
    for col_spec in &table_block.columns {
        text(out, "\n      ");
        open(out, "th");
        push_cell_attrs(
            out,
            &col_spec.attrs,
            col_spec.align,
            col_spec.width.as_deref(),
        );
        let rendered = render_inline_events(&col_spec.header_text, inline_pipeline, context);
        out.extend(rendered);
        close(out, "th");
    }
    text(out, "\n    ");
    close(out, "tr");
    text(out, "\n  ");
    close(out, "thead");
    text(out, "\n");

    // Tbody
    if !body_rows.is_empty() {
        text(out, "  ");
        open(out, "tbody");
        for row in &body_rows {
            emit_row_elements(row, inline_pipeline, context, out);
        }
        text(out, "\n  ");
        close(out, "tbody");
        text(out, "\n");
    }

    // Tfoot
    if !footer_rows.is_empty() {
        text(out, "  ");
        open(out, "tfoot");
        for row in &footer_rows {
            emit_row_elements(row, inline_pipeline, context, out);
        }
        text(out, "\n  ");
        close(out, "tfoot");
        text(out, "\n");
    }

    close(out, "table");
    out.extend(raw_inline("\n"));
}

fn emit_row_elements<C, P>(
    row: &ProcessedRow,
    inline_pipeline: &P,
    context: &mut C,
    out: &mut Vec<Event>,
) where
    P: InlinePipeline<C>,
{
    text(out, "\n    ");
    open(out, "tr");
    push_common_attrs(out, &row.attrs);

    for cell in &row.cells {
        text(out, "\n      ");
        open(out, "td");
        push_cell_attrs(out, &cell.attrs, cell.align, cell.width.as_deref());
        if cell.colspan > 1 {
            attribute(out, "colspan", &cell.colspan.to_string());
        }
        if cell.rowspan > 1 {
            attribute(out, "rowspan", &cell.rowspan.to_string());
        }
        let rendered = render_inline_events(&cell.text, inline_pipeline, context);
        out.extend(rendered);
        close(out, "td");
    }

    text(out, "\n    ");
    close(out, "tr");
}

/// Emits `id`, `class`, extra properties and the computed `style` attribute for
/// a table header/body cell.
fn push_cell_attrs(out: &mut Vec<Event>, attrs: &AttrSpec, align: Align, width: Option<&str>) {
    if let Some(id) = attrs.id.as_deref() {
        attribute(out, "id", id);
    }
    if !attrs.classes.is_empty() {
        attribute(out, "class", &attrs.classes.join(" "));
    }
    for (k, v) in &attrs.properties {
        if !k.starts_with("--") {
            attribute(out, k, v);
        }
    }

    let mut styles = Vec::new();
    match align {
        Align::Left => styles.push("text-align: left".to_string()),
        Align::Center => styles.push("text-align: center".to_string()),
        Align::Right => styles.push("text-align: right".to_string()),
        Align::None => {}
    }
    if let Some(w) = width {
        styles.push(format!("width: {}", w));
    }
    for (k, v) in &attrs.properties {
        if k.starts_with("--") {
            styles.push(format!("{}:{}", k, v));
        }
    }
    if !styles.is_empty() {
        attribute(out, "style", &format!("{};", styles.join("; ")));
    }
}

/// Emits `id`, `class`, extra properties and `style` for `<table>` / `<tr>`.
fn push_common_attrs(out: &mut Vec<Event>, attrs: &AttrSpec) {
    if let Some(id) = attrs.id.as_deref() {
        attribute(out, "id", id);
    }
    if !attrs.classes.is_empty() {
        attribute(out, "class", &attrs.classes.join(" "));
    }
    for (k, v) in &attrs.properties {
        if k.starts_with("--") {
            continue;
        }
        attribute(out, k, v);
    }
    let styles: Vec<String> = attrs
        .properties
        .iter()
        .filter(|(k, _)| k.starts_with("--"))
        .map(|(k, v)| format!("{}:{};", k, v))
        .collect();
    if !styles.is_empty() {
        attribute(out, "style", &styles.concat());
    }
}

fn text(out: &mut Vec<Event>, value: &str) {
    out.push(Event::Text(value.to_string()));
}

fn attribute(out: &mut Vec<Event>, name: &str, value: &str) {
    out.push(Event::Attribute {
        name: name.to_string(),
        value: value.to_string(),
    });
}

fn open(out: &mut Vec<Event>, tag: &str) {
    out.extend(element_open(tag));
}

fn close(out: &mut Vec<Event>, tag: &str) {
    out.push(element_close(tag));
}

/// Renders caption/cell content into inline AST events. Inline plugins run
/// before markdown so they can consume raw patterns such as `[[wiki]]`.
pub(crate) fn render_inline_events<C, P>(
    text: &str,
    inline_pipeline: &P,
    context: &mut C,
) -> Vec<Event>
where
    P: InlinePipeline<C>,
{
    // Cells are authored on a single source line, so a literal `\n` escape is
    // expanded into a real line break before parsing. This is what makes block
    // content expressible inside one cell:
    // `| - Alpha\n- Beta |` renders `<ul><li>Alpha</li><li>Beta</li></ul>`.
    let unescaped = text.replace("\\n", "\n");
    let source = unescaped.trim();
    if source.is_empty() {
        return Vec::new();
    }
    let parsed = parse(source, &Options::default());
    let processed = inline_pipeline.run_with(context, parsed);
    let processed = process_markdown(&processed);
    let processed = inline_pipeline.run_after_markdown(context, processed);

    let mut out = Vec::new();
    for ev in processed {
        match &ev {
            Event::StartNode(NodeKind::Document) | Event::EndNode(NodeKind::Document) => {}
            Event::StartNode(NodeKind::Paragraph) | Event::EndNode(NodeKind::Paragraph) => {}
            _ => out.push(ev),
        }
    }
    out
}
