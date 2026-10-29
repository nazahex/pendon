//! §11 layer emission (ADR-0008 `D-3a`).
//!
//! A table is seven layers — `table`, `caption`, `thead`, `tbody`, `tfoot`,
//! `row`, `cell` (§8) — and this module emits all of them as the plugin's own
//! built-in elements, tagging each node with [`LAYER_ATTR`] so the renderer can
//! resolve the document-wide `type` set against the layer that owns it. The
//! plugin never picks a component itself: it keeps the `type` marker it already
//! emitted and lets `pendon-renderer-solid` decide *per document*, which is what
//! makes the `_` (remainder) and `*` (everything) selectors of ADR-0008 work.
//!
//! Only the *shape* of the attributes is decided here, once per layer: a layer
//! with no component is emitted DOM-shaped (`style="text-align: left;"`), a
//! layer with a component is emitted props-shaped (`align="left"`). See
//! [`Shape`] for why that choice can never be per element.

use crate::attrs::{decorator_to_layer, push_cell_attrs, push_layer_attrs, LayerAttrs, Shape};
use crate::grid::{process_grid, ProcessedRow};
use crate::parser::TableBlock;
use crate::{CustomComponent, TableOptions};
use pendon_core::{
    element_close, element_open, parse, raw_inline, Event, InlinePipeline, NodeKind, Options,
};
use pendon_extra::BoundDecorator;
use pendon_plugin_markdown::process as process_markdown;
use pendon_renderer_solid::{layer_templates, SolidRenderHints};

/// §11 layer of the `<table>` element itself.
pub const TABLE_LAYER: &str = "table";
/// §11 layer of `<caption>`.
pub const CAPTION_LAYER: &str = "caption";
/// §11 layer of `<thead>`.
pub const THEAD_LAYER: &str = "thead";
/// §11 layer of `<tbody>`.
pub const TBODY_LAYER: &str = "tbody";
/// §11 layer of `<tfoot>`.
pub const TFOOT_LAYER: &str = "tfoot";
/// §11 layer of `<tr>`.
pub const ROW_LAYER: &str = "row";
/// §11 layer of `<th>` / `<td>`.
pub const CELL_LAYER: &str = "cell";

/// The layer a node belongs to (`§11` rule 3); stripped before rendering.
pub const LAYER_ATTR: &str = "__layer";

/// §11 rule 3: the children of these nodes are already rendered, so the second
/// Markdown pass must pass them through verbatim. Without it the pass re-lexed
/// every cell and dropped the whitespace-only chunks that carry the spaces
/// between words.
const PRE_RENDERED_KIND: &str = "__plugin_kind";
const PRE_RENDERED_VALUE: &str = "element";

/// Every §11 layer of a table, primary layer first.
pub fn layers() -> [&'static str; 7] {
    [
        TABLE_LAYER,
        CAPTION_LAYER,
        THEAD_LAYER,
        TBODY_LAYER,
        TFOOT_LAYER,
        ROW_LAYER,
        CELL_LAYER,
    ]
}

/// The primary layer of `plugin-table`: the `<table>` element. A `[[custom]]`
/// array (no `type`) addresses this layer.
pub fn primary_layer() -> &'static str {
    TABLE_LAYER
}

/// The attribute shape of all seven layers, resolved once per document.
#[derive(Clone, Copy, Debug)]
struct Shapes {
    table: Shape,
    caption: Shape,
    thead: Shape,
    tbody: Shape,
    tfoot: Shape,
    row: Shape,
    cell: Shape,
}

impl Shapes {
    /// No component anywhere: every layer keeps the DOM fallback shape.
    const PLAIN: Shapes = Shapes {
        table: Shape::Html,
        caption: Shape::Html,
        thead: Shape::Html,
        tbody: Shape::Html,
        tfoot: Shape::Html,
        row: Shape::Html,
        cell: Shape::Html,
    };

    fn of(options: &TableOptions) -> Shapes {
        let Some(custom) = options.custom_node.as_ref() else {
            return Shapes::PLAIN;
        };
        Shapes {
            table: Shape::of(!custom.table.is_empty()),
            caption: Shape::of(!custom.caption.is_empty()),
            thead: Shape::of(!custom.thead.is_empty()),
            tbody: Shape::of(!custom.tbody.is_empty()),
            tfoot: Shape::of(!custom.tfoot.is_empty()),
            row: Shape::of(!custom.row.is_empty()),
            cell: Shape::of(!custom.cell.is_empty()),
        }
    }

    /// Whether any layer is answered by a component. One composed node is
    /// enough to move the whole table off the pretty-printed plain layout.
    fn any_composed(&self) -> bool {
        [
            self.table,
            self.caption,
            self.thead,
            self.tbody,
            self.tfoot,
            self.row,
            self.cell,
        ]
        .contains(&Shape::Props)
    }
}

/// A table's §11 layer templates, one per configured entry, tagged with the
/// layer the entry answers (§11 rule 3, `D-3a`).
pub fn solid_hints(options: &TableOptions) -> Option<SolidRenderHints> {
    let custom = options.custom_node.as_ref()?;
    let payload = |custom: &CustomComponent| (custom.template.clone(), custom.imports.clone());
    let sets = [
        (TABLE_LAYER, &custom.table),
        (CAPTION_LAYER, &custom.caption),
        (THEAD_LAYER, &custom.thead),
        (TBODY_LAYER, &custom.tbody),
        (TFOOT_LAYER, &custom.tfoot),
        (ROW_LAYER, &custom.row),
        (CELL_LAYER, &custom.cell),
    ];
    let mut hints = SolidRenderHints::default();
    for (layer, set) in sets {
        hints.templates.extend(layer_templates(layer, set, payload));
    }
    if hints.templates.is_empty() {
        return None;
    }
    Some(hints)
}

/// Emits one table block as built-in, `__layer`-tagged elements.
///
/// `decorator` is the §9.1 decorator line bound to this table, if any: it is the
/// whole of the `<table>` layer's extras (§8).
pub fn emit_table_with_context<C, P>(
    table_block: &TableBlock,
    decorator: Option<&BoundDecorator>,
    options: &TableOptions,
    inline_pipeline: &P,
    context: &mut C,
    out: &mut Vec<Event>,
) where
    P: InlinePipeline<C>,
{
    emit_table_inner(
        table_block,
        decorator,
        options,
        inline_pipeline,
        context,
        out,
    );
}

fn emit_table_inner<C, P>(
    table_block: &TableBlock,
    decorator: Option<&BoundDecorator>,
    options: &TableOptions,
    inline_pipeline: &P,
    context: &mut C,
    out: &mut Vec<Event>,
) where
    P: InlinePipeline<C>,
{
    let shapes = Shapes::of(options);
    let (body_rows, footer_rows) = process_grid(
        &table_block.columns,
        &table_block.body_rows,
        &table_block.footer_rows,
    );
    // A composed table is never pretty-printed: the layout whitespace would
    // end up as text children of somebody's component.
    let composed = shapes.any_composed();
    // §8: the `<table>` layer's extras are the decorator's head, in the same
    // mapping every other layer uses.
    let table_attrs = decorator.map(decorator_to_layer).unwrap_or_default();

    open_layer(out, "table", TABLE_LAYER, composed);
    push_layer_attrs(out, &table_attrs, shapes.table);
    gap(out, composed, "\n");

    if let Some(caption) = &table_block.caption {
        gap(out, composed, "  ");
        open_layer(out, "caption", CAPTION_LAYER, composed);
        push_layer_attrs(out, &caption.attrs, shapes.caption);
        push_content(&caption.text, inline_pipeline, context, out);
        close(out, "caption");
        gap(out, composed, "\n");
    }

    // §8: the header row has no extras slot of its own; its `<th>`s carry the
    // column marker.
    gap(out, composed, "  ");
    open_layer(out, "thead", THEAD_LAYER, composed);
    push_layer_attrs(out, &LayerAttrs::default(), shapes.thead);
    gap(out, composed, "\n    ");
    open_layer(out, "tr", ROW_LAYER, composed);
    push_layer_attrs(out, &LayerAttrs::default(), shapes.row);
    for column in &table_block.columns {
        gap(out, composed, "\n      ");
        open_layer(out, "th", CELL_LAYER, composed);
        push_cell_attrs(
            out,
            &column.attrs,
            column.align,
            column.width.as_deref(),
            1,
            1,
            shapes.cell,
        );
        push_content(&column.header_text, inline_pipeline, context, out);
        close(out, "th");
    }
    gap(out, composed, "\n    ");
    close(out, "tr");
    gap(out, composed, "\n  ");
    close(out, "thead");
    gap(out, composed, "\n");

    if !body_rows.is_empty() {
        // §8: the alignment row's end-of-line extras belong to the `<tbody>`.
        gap(out, composed, "  ");
        open_layer(out, "tbody", TBODY_LAYER, composed);
        push_layer_attrs(out, &table_block.tbody, shapes.tbody);
        emit_rows(&body_rows, &shapes, composed, inline_pipeline, context, out);
        gap(out, composed, "\n  ");
        close(out, "tbody");
        gap(out, composed, "\n");
    }

    if !footer_rows.is_empty() {
        // §8: extras of the `|===|` line belong to the `<tfoot>`.
        gap(out, composed, "  ");
        open_layer(out, "tfoot", TFOOT_LAYER, composed);
        push_layer_attrs(out, &table_block.tfoot, shapes.tfoot);
        emit_rows(
            &footer_rows,
            &shapes,
            composed,
            inline_pipeline,
            context,
            out,
        );
        gap(out, composed, "\n  ");
        close(out, "tfoot");
        gap(out, composed, "\n");
    }

    close(out, "table");
    if !composed {
        out.extend(raw_inline("\n"));
    }
}

fn emit_rows<C, P>(
    rows: &[ProcessedRow],
    shapes: &Shapes,
    composed: bool,
    inline_pipeline: &P,
    context: &mut C,
    out: &mut Vec<Event>,
) where
    P: InlinePipeline<C>,
{
    for row in rows {
        gap(out, composed, "\n    ");
        // §11 rule 3: the row's end-of-line marker routes the `<tr>`.
        open_layer(out, "tr", ROW_LAYER, composed);
        push_layer_attrs(out, &row.attrs, shapes.row);
        for cell in &row.cells {
            gap(out, composed, "\n      ");
            open_layer(out, "td", CELL_LAYER, composed);
            push_cell_attrs(
                out,
                &cell.attrs,
                cell.align,
                cell.width.as_deref(),
                cell.colspan,
                cell.rowspan,
                shapes.cell,
            );
            push_content(&cell.text, inline_pipeline, context, out);
            close(out, "td");
        }
        gap(out, composed, "\n    ");
        close(out, "tr");
    }
}

/// Opens one element and tags it with the layer that owns it (§11 rule 3).
fn open_layer(out: &mut Vec<Event>, tag: &str, layer: &str, composed: bool) {
    out.extend(element_open(tag));
    if composed {
        push_pre_rendered(out);
    }
    out.push(Event::Attribute {
        name: LAYER_ATTR.to_string(),
        value: layer.to_string(),
    });
}

fn push_pre_rendered(out: &mut Vec<Event>) {
    out.push(Event::Attribute {
        name: PRE_RENDERED_KIND.to_string(),
        value: PRE_RENDERED_VALUE.to_string(),
    });
}

/// Layout whitespace: only the plain (all-HTML) table is pretty-printed.
fn gap(out: &mut Vec<Event>, composed: bool, value: &str) {
    if !composed {
        out.push(Event::Text(value.to_string()));
    }
}

fn close(out: &mut Vec<Event>, tag: &str) {
    out.push(element_close(tag));
}

fn push_content<C, P>(text: &str, inline_pipeline: &P, context: &mut C, out: &mut Vec<Event>)
where
    P: InlinePipeline<C>,
{
    for event in render_inline_events(text, inline_pipeline, context) {
        out.push(event);
    }
}

/// Renders caption/cell content into inline AST events. Inline plugins run
/// before markdown so they can consume raw patterns such as `[[wiki]]`.
fn render_inline_events<C, P>(text: &str, inline_pipeline: &P, context: &mut C) -> Vec<Event>
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
    for event in processed {
        match &event {
            Event::StartNode(NodeKind::Document) | Event::EndNode(NodeKind::Document) => {}
            Event::StartNode(NodeKind::Paragraph) | Event::EndNode(NodeKind::Paragraph) => {}
            _ => out.push(event),
        }
    }
    out
}
