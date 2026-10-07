use crate::attrs::{push_flags, LayerAttrs};
use crate::grid::{process_grid, ProcessedRow};
use crate::parser::{Align, TableBlock};
use crate::render::{push_cell_attrs, push_common_attrs, render_inline_events};
use crate::{CustomComponent, TableOptions};
use pendon_core::{element_close, element_open, Event, InlinePipeline, NodeKind, Pipeline};
use pendon_renderer_solid::ComponentSet;
use pendon_renderer_solid::{ComponentTemplate, ImportEntry, SolidRenderHints};

/// Marker attribute telling `plugin-markdown` that the children of this custom
/// node were already rendered by the table plugin and must pass through the
/// second Markdown pass verbatim. Without it the pass re-lexed every cell and
/// dropped the whitespace-only chunks that carry the spaces between words.
const PRE_RENDERED_KIND: &str = "__plugin_kind";
const PRE_RENDERED_VALUE: &str = "element";

pub fn emit_custom_table(
    table_block: &TableBlock,
    options: &TableOptions,
    inline_pipeline: &Pipeline,
    out: &mut Vec<Event>,
) {
    emit_custom_table_inner(table_block, options, inline_pipeline, &mut (), out);
}

pub fn emit_custom_table_with_context<C, P>(
    table_block: &TableBlock,
    options: &TableOptions,
    inline_pipeline: &P,
    context: &mut C,
    out: &mut Vec<Event>,
) where
    P: InlinePipeline<C>,
{
    emit_custom_table_inner(table_block, options, inline_pipeline, context, out);
}

fn emit_custom_table_inner<C, P>(
    table_block: &TableBlock,
    options: &TableOptions,
    inline_pipeline: &P,
    context: &mut C,
    out: &mut Vec<Event>,
) where
    P: InlinePipeline<C>,
{
    let custom = match options.custom_node.as_ref() {
        Some(c) => c,
        None => return,
    };

    let (body_rows, footer_rows) = process_grid(
        &table_block.columns,
        &table_block.body_rows,
        &table_block.footer_rows,
    );

    // §8: the declaration line owns the `<table>` extras; the pre-§8 caption
    // form keeps configuring the table itself.
    let table_layer = table_layer_attrs(table_block);
    let table_comp = custom
        .table
        .select(table_block.attrs.type_marker.as_deref());

    // Table wrapper: the custom component when configured, plain <table> when not.
    match table_comp {
        Some(table_comp) => {
            open_custom(table_comp, out);
            emit_layer(&table_layer, out);
        }
        None => {
            out.extend(element_open("table"));
            push_common_attrs(out, &table_layer);
        }
    }

    // §11 rule 3: the caption marker routes the caption layer.
    let caption_comp = custom.caption.select(
        table_block
            .caption
            .as_ref()
            .and_then(|caption| caption.attrs.type_marker.as_deref()),
    );
    emit_caption_node(
        &table_block.caption,
        caption_comp,
        inline_pipeline,
        context,
        out,
    );

    // Section wrappers: <thead>/<tbody>/<tfoot>, or the configured components.
    // §8: the header row has no extras slot of its own; its `<th>`s take the
    // column marker.
    let thead_comp = custom.thead.select(None);
    open_section(thead_comp, "thead", &LayerAttrs::default(), out);
    emit_header_row(
        &table_block.columns,
        &custom.row,
        &custom.cell,
        inline_pipeline,
        context,
        out,
    );
    close_section(thead_comp, "thead", out);

    if !body_rows.is_empty() {
        // §8: the alignment row's end-of-line extras belong to the `<tbody>`.
        let tbody_comp = custom
            .tbody
            .select(table_block.tbody.type_marker.as_deref());
        open_section(tbody_comp, "tbody", &table_block.tbody, out);
        emit_body_rows(
            &body_rows,
            &custom.row,
            &custom.cell,
            inline_pipeline,
            context,
            out,
        );
        close_section(tbody_comp, "tbody", out);
    }

    if !footer_rows.is_empty() {
        // §8: extras of the `|===|` line belong to the `<tfoot>`.
        let tfoot_comp = custom
            .tfoot
            .select(table_block.tfoot.type_marker.as_deref());
        open_section(tfoot_comp, "tfoot", &table_block.tfoot, out);
        emit_body_rows(
            &footer_rows,
            &custom.row,
            &custom.cell,
            inline_pipeline,
            context,
            out,
        );
        close_section(tfoot_comp, "tfoot", out);
    }

    match table_comp {
        Some(table_comp) => out.push(Event::EndNode(NodeKind::Custom(table_comp.name.clone()))),
        None => out.push(element_close("table")),
    }
}

/// The `<table>` extras: the §8 declaration line.
fn table_layer_attrs(table_block: &TableBlock) -> LayerAttrs {
    table_block.attrs.clone()
}

/// Emits one layer's attributes onto a **custom** component node: keys verbatim
/// (`data-` prefixes are an HTML-fallback concern) plus the §6.3 bare flags.
fn emit_layer(layer: &LayerAttrs, out: &mut Vec<Event>) {
    if let Some(id) = &layer.attrs.id {
        out.push(Event::Attribute {
            name: "id".to_string(),
            value: id.clone(),
        });
    }
    if !layer.attrs.classes.is_empty() {
        out.push(Event::Attribute {
            name: "class".to_string(),
            value: layer.attrs.classes.join(" "),
        });
    }
    for (key, value) in &layer.attrs.properties {
        out.push(Event::Attribute {
            name: key.clone(),
            value: value.clone(),
        });
    }
    push_flags(out, &layer.flags);
}

fn emit_caption_node<C>(
    caption: &Option<crate::parser::CaptionSpec>,
    custom: Option<&CustomComponent>,
    inline_pipeline: &impl InlinePipeline<C>,
    context: &mut C,
    out: &mut Vec<Event>,
) where
    C: Sized,
{
    let Some(caption_spec) = caption else {
        return;
    };

    // Caption: the configured component, or a plain <caption> when not set.
    match custom {
        Some(comp) => {
            open_custom(comp, out);
            emit_layer(&caption_spec.attrs, out);
        }
        None => {
            out.extend(element_open("caption"));
            push_common_attrs(out, &caption_spec.attrs);
        }
    }

    for ev in render_inline_events(&caption_spec.text, inline_pipeline, context) {
        out.push(ev);
    }

    match custom {
        Some(comp) => out.push(Event::EndNode(NodeKind::Custom(comp.name.clone()))),
        None => out.push(element_close("caption")),
    }
}

/// Starts a custom component node. The `__plugin_kind` marker keeps the second
/// Markdown pass from re-lexing (and whitespace-stripping) the children this
/// plugin already rendered.
fn open_custom(comp: &CustomComponent, out: &mut Vec<Event>) {
    out.push(Event::StartNode(NodeKind::Custom(comp.name.clone())));
    out.push(Event::Attribute {
        name: PRE_RENDERED_KIND.to_string(),
        value: PRE_RENDERED_VALUE.to_string(),
    });
    out.push(Event::Attribute {
        name: "name".to_string(),
        value: comp.name.clone(),
    });
}

fn open_section(
    custom: Option<&CustomComponent>,
    tag: &str,
    layer: &LayerAttrs,
    out: &mut Vec<Event>,
) {
    match custom {
        Some(comp) => {
            open_custom(comp, out);
            emit_layer(layer, out);
        }
        None => {
            out.extend(element_open(tag));
            push_common_attrs(out, layer);
        }
    }
}

fn close_section(custom: Option<&CustomComponent>, tag: &str, out: &mut Vec<Event>) {
    match custom {
        Some(comp) => out.push(Event::EndNode(NodeKind::Custom(comp.name.clone()))),
        None => out.push(element_close(tag)),
    }
}

fn open_row(
    row_custom: Option<&CustomComponent>,
    row_attrs: Option<&LayerAttrs>,
    out: &mut Vec<Event>,
) {
    match row_custom {
        Some(comp) => {
            open_custom(comp, out);
            if let Some(attrs) = row_attrs {
                emit_layer(attrs, out);
            }
        }
        None => {
            out.extend(element_open("tr"));
            if let Some(attrs) = row_attrs {
                push_common_attrs(out, attrs);
            }
        }
    }
}

fn close_row(row_custom: Option<&CustomComponent>, out: &mut Vec<Event>) {
    match row_custom {
        Some(comp) => out.push(Event::EndNode(NodeKind::Custom(comp.name.clone()))),
        None => out.push(element_close("tr")),
    }
}

/// Emits one cell with the configured cell component, or a plain `<th>` /
/// `<td>` when that component is not configured. Every layer of the table is
/// optional, so a partially configured `custom_node` still renders.
#[allow(clippy::too_many_arguments)]
fn emit_cell<C>(
    tag: &str,
    cell_custom: Option<&CustomComponent>,
    attrs: &LayerAttrs,
    align: Align,
    width: Option<&str>,
    colspan: usize,
    rowspan: usize,
    content: &str,
    inline_pipeline: &impl InlinePipeline<C>,
    context: &mut C,
    out: &mut Vec<Event>,
) where
    C: Sized,
{
    match cell_custom {
        Some(comp) => {
            open_custom(comp, out);
            emit_cell_attrs(attrs, align, width, colspan, rowspan, out);
        }
        None => {
            out.extend(element_open(tag));
            push_cell_attrs(out, attrs, align, width);
            if colspan > 1 {
                out.push(Event::Attribute {
                    name: "colspan".to_string(),
                    value: colspan.to_string(),
                });
            }
            if rowspan > 1 {
                out.push(Event::Attribute {
                    name: "rowspan".to_string(),
                    value: rowspan.to_string(),
                });
            }
        }
    }

    for ev in render_inline_events(content, inline_pipeline, context) {
        out.push(ev);
    }

    match cell_custom {
        Some(comp) => out.push(Event::EndNode(NodeKind::Custom(comp.name.clone()))),
        None => out.push(element_close(tag)),
    }
}

fn emit_header_row<C>(
    columns: &[crate::parser::ColumnSpec],
    row_set: &ComponentSet<CustomComponent>,
    cell_set: &ComponentSet<CustomComponent>,
    inline_pipeline: &impl InlinePipeline<C>,
    context: &mut C,
    out: &mut Vec<Event>,
) where
    C: Sized,
{
    let row_custom = row_set.select(None);
    open_row(row_custom, None, out);
    for col_spec in columns {
        // §11 rule 3: the column marker routes its `<th>` cells.
        let cell_custom = cell_set.select(col_spec.attrs.type_marker.as_deref());
        emit_cell(
            "th",
            cell_custom,
            &col_spec.attrs,
            col_spec.align,
            col_spec.width.as_deref(),
            1,
            1,
            &col_spec.header_text,
            inline_pipeline,
            context,
            out,
        );
    }
    close_row(row_custom, out);
}

fn emit_body_rows<C>(
    rows: &[ProcessedRow],
    row_set: &ComponentSet<CustomComponent>,
    cell_set: &ComponentSet<CustomComponent>,
    inline_pipeline: &impl InlinePipeline<C>,
    context: &mut C,
    out: &mut Vec<Event>,
) where
    C: Sized,
{
    for row in rows {
        // §11 rule 3: the row's end-of-line marker routes the `<tr>`.
        let row_custom = row_set.select(row.attrs.type_marker.as_deref());
        open_row(row_custom, Some(&row.attrs), out);
        for cell in &row.cells {
            let cell_custom = cell_set.select(cell.attrs.type_marker.as_deref());
            emit_cell(
                "td",
                cell_custom,
                &cell.attrs,
                cell.align,
                cell.width.as_deref(),
                cell.colspan,
                cell.rowspan,
                &cell.text,
                inline_pipeline,
                context,
                out,
            );
        }
        close_row(row_custom, out);
    }
}

fn emit_cell_attrs(
    attrs: &LayerAttrs,
    align: Align,
    width: Option<&str>,
    colspan: usize,
    rowspan: usize,
    out: &mut Vec<Event>,
) {
    if let Some(id) = &attrs.attrs.id {
        out.push(Event::Attribute {
            name: "id".to_string(),
            value: id.clone(),
        });
    }

    if !attrs.attrs.classes.is_empty() {
        out.push(Event::Attribute {
            name: "class".to_string(),
            value: attrs.attrs.classes.join(" "),
        });
    }

    // Extra attrs (bukan style)
    for (k, v) in &attrs.attrs.properties {
        if k.starts_with("--") {
            continue;
        }
        if !v.is_empty() {
            out.push(Event::Attribute {
                name: k.clone(),
                value: v.clone(),
            });
        }
    }

    if align != Align::None {
        out.push(Event::Attribute {
            name: "align".to_string(),
            value: align_to_string(align),
        });
    }

    if let Some(w) = width {
        if !w.is_empty() {
            out.push(Event::Attribute {
                name: "width".to_string(),
                value: w.to_string(),
            });
        }
    }

    // PENTING: Hanya pancarkan jika > 1
    if colspan > 1 {
        out.push(Event::Attribute {
            name: "colspan".to_string(),
            value: colspan.to_string(),
        });
    }
    if rowspan > 1 {
        out.push(Event::Attribute {
            name: "rowspan".to_string(),
            value: rowspan.to_string(),
        });
    }

    // Style entries
    let mut styles = Vec::new();
    for (k, v) in &attrs.attrs.properties {
        if k.starts_with("--") {
            styles.push(format!("{}:{}", k, v));
        }
    }
    if !styles.is_empty() {
        out.push(Event::Attribute {
            name: "style".to_string(),
            value: styles.join("; "),
        });
    }

    // §6.3: a bare flag stays a bare attribute.
    push_flags(out, &attrs.flags);
}

fn align_to_string(align: Align) -> String {
    match align {
        Align::Left => "left".to_string(),
        Align::Center => "center".to_string(),
        Align::Right => "right".to_string(),
        Align::None => String::new(),
    }
}

pub fn solid_hints(options: &TableOptions) -> Option<SolidRenderHints> {
    let custom = options.custom_node.as_ref()?;
    let mut hints = SolidRenderHints::default();

    // Imports listed once under `[task.table.custom]` apply to every component;
    // a component's own list is appended (the renderer deduplicates).
    let shared: &[ImportEntry] = &custom.imports;
    let mut register = |c: &CustomComponent| {
        let key = (c.name.clone(), Some(c.name.clone()));
        hints.templates.push(ComponentTemplate {
            node_type: c.name.clone(),
            node_name: Some(c.name.clone()),
            template: c.template.clone(),
        });
        let mut imports = shared.to_vec();
        imports.extend(c.imports.iter().cloned());
        if !imports.is_empty() {
            hints.template_imports.insert(key, imports);
        }
    };

    // §11 rule 3: every entry of every layer answers instances of its own.
    for set in [
        &custom.table,
        &custom.caption,
        &custom.thead,
        &custom.tbody,
        &custom.tfoot,
        &custom.row,
        &custom.cell,
    ] {
        for component in set.components() {
            register(component);
        }
    }

    Some(hints)
}
