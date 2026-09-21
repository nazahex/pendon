use crate::attrs::AttrSpec;
use crate::grid::{process_grid, ProcessedRow};
use crate::parser::{Align, TableBlock};
use crate::{CustomComponent, TableOptions};
use pendon_core::{parse, Event, NodeKind, Options, Pipeline};
use pendon_plugin_markdown::process as process_markdown;
use pendon_renderer_solid::{ComponentTemplate, ImportEntry, SolidRenderHints};

pub fn emit_custom_table(
    table_block: &TableBlock,
    options: &TableOptions,
    inline_pipeline: &Pipeline,
    out: &mut Vec<Event>,
) {
    let custom = match options.custom_node.as_ref() {
        Some(c) => c,
        None => return,
    };

    let (body_rows, footer_rows) = process_grid(
        &table_block.columns,
        &table_block.body_rows,
        &table_block.footer_rows,
    );

    if let Some(table_comp) = &custom.table {
        let node_kind = NodeKind::Custom(table_comp.name.clone());
        out.push(Event::StartNode(node_kind.clone()));
        out.push(Event::Attribute {
            name: "name".to_string(),
            value: table_comp.name.clone(),
        });

        if let Some(caption) = &table_block.caption {
            emit_attr_spec(&caption.attrs, out);
        }

        emit_caption_node(&table_block.caption, &custom.caption, inline_pipeline, out);

        out.push(Event::Text("<thead>\n".to_string()));
        emit_thead_node(
            &table_block.columns,
            &custom.row,
            &custom.cell,
            inline_pipeline,
            out,
        );
        out.push(Event::Text("</thead>\n".to_string()));

        if !body_rows.is_empty() {
            out.push(Event::Text("<tbody>\n".to_string()));
            emit_tbody_node(&body_rows, &custom.row, &custom.cell, inline_pipeline, out);
            out.push(Event::Text("</tbody>\n".to_string()));
        }

        if !footer_rows.is_empty() {
            out.push(Event::Text("<tfoot>\n".to_string()));
            emit_tfoot_node(
                &footer_rows,
                &custom.row,
                &custom.cell,
                inline_pipeline,
                out,
            );
            out.push(Event::Text("</tfoot>\n".to_string()));
        }

        out.push(Event::EndNode(node_kind));
    }
}

fn emit_caption_node(
    caption: &Option<crate::parser::CaptionSpec>,
    custom: &Option<CustomComponent>,
    inline_pipeline: &Pipeline,
    out: &mut Vec<Event>,
) {
    let Some(caption_spec) = caption else {
        return;
    };
    let Some(comp) = custom else {
        return;
    };

    let node_kind = NodeKind::Custom(comp.name.clone());
    out.push(Event::StartNode(node_kind.clone()));
    out.push(Event::Attribute {
        name: "name".to_string(),
        value: comp.name.clone(),
    });
    emit_attr_spec(&caption_spec.attrs, out);

    for ev in render_inline_events(&caption_spec.text, inline_pipeline) {
        out.push(ev);
    }
    out.push(Event::EndNode(node_kind));
}

fn emit_thead_node(
    columns: &[crate::parser::ColumnSpec],
    row_custom: &Option<CustomComponent>,
    cell_custom: &Option<CustomComponent>,
    inline_pipeline: &Pipeline,
    out: &mut Vec<Event>,
) {
    let Some(row_comp) = row_custom else {
        return;
    };
    let Some(cell_comp) = cell_custom else {
        return;
    };

    let row_kind = NodeKind::Custom(row_comp.name.clone());
    out.push(Event::StartNode(row_kind.clone()));
    out.push(Event::Attribute {
        name: "name".to_string(),
        value: row_comp.name.clone(),
    });

    for col_spec in columns {
        let cell_kind = NodeKind::Custom(cell_comp.name.clone());
        out.push(Event::StartNode(cell_kind.clone()));
        out.push(Event::Attribute {
            name: "name".to_string(),
            value: cell_comp.name.clone(),
        });

        emit_cell_attrs(
            &col_spec.attrs,
            col_spec.align,
            col_spec.width.as_deref(),
            1,
            1,
            out,
        );

        for ev in render_inline_events(&col_spec.header_text, inline_pipeline) {
            out.push(ev);
        }
        out.push(Event::EndNode(cell_kind));
    }
    out.push(Event::EndNode(row_kind));
}

fn emit_tbody_node(
    rows: &[ProcessedRow],
    row_custom: &Option<CustomComponent>,
    cell_custom: &Option<CustomComponent>,
    inline_pipeline: &Pipeline,
    out: &mut Vec<Event>,
) {
    let Some(row_comp) = row_custom else {
        return;
    };
    let Some(cell_comp) = cell_custom else {
        return;
    };

    for row in rows {
        let row_kind = NodeKind::Custom(row_comp.name.clone());
        out.push(Event::StartNode(row_kind.clone()));
        out.push(Event::Attribute {
            name: "name".to_string(),
            value: row_comp.name.clone(),
        });
        emit_attr_spec(&row.attrs, out);

        for cell in &row.cells {
            let cell_kind = NodeKind::Custom(cell_comp.name.clone());
            out.push(Event::StartNode(cell_kind.clone()));
            out.push(Event::Attribute {
                name: "name".to_string(),
                value: cell_comp.name.clone(),
            });

            emit_cell_attrs(
                &cell.attrs,
                cell.align,
                cell.width.as_deref(),
                cell.colspan,
                cell.rowspan,
                out,
            );

            for ev in render_inline_events(&cell.text, inline_pipeline) {
                out.push(ev);
            }
            out.push(Event::EndNode(cell_kind));
        }
        out.push(Event::EndNode(row_kind));
    }
}

fn emit_tfoot_node(
    rows: &[ProcessedRow],
    row_custom: &Option<CustomComponent>,
    cell_custom: &Option<CustomComponent>,
    inline_pipeline: &Pipeline,
    out: &mut Vec<Event>,
) {
    emit_tbody_node(rows, row_custom, cell_custom, inline_pipeline, out);
}

fn emit_attr_spec(attrs: &AttrSpec, out: &mut Vec<Event>) {
    if let Some(id) = &attrs.id {
        out.push(Event::Attribute {
            name: "id".to_string(),
            value: id.clone(),
        });
    }
    if !attrs.classes.is_empty() {
        out.push(Event::Attribute {
            name: "class".to_string(),
            value: attrs.classes.join(" "),
        });
    }
    for (k, v) in &attrs.extra {
        out.push(Event::Attribute {
            name: k.clone(),
            value: v.clone(),
        });
    }
}

fn emit_cell_attrs(
    attrs: &AttrSpec,
    align: Align,
    width: Option<&str>,
    colspan: usize,
    rowspan: usize,
    out: &mut Vec<Event>,
) {
    if let Some(id) = &attrs.id {
        out.push(Event::Attribute {
            name: "id".to_string(),
            value: id.clone(),
        });
    }

    if !attrs.classes.is_empty() {
        out.push(Event::Attribute {
            name: "class".to_string(),
            value: attrs.classes.join(" "),
        });
    }

    // Extra attrs (bukan style)
    for (k, v) in &attrs.extra {
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
    for (k, v) in &attrs.extra {
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
}

fn align_to_string(align: Align) -> String {
    match align {
        Align::Left => "left".to_string(),
        Align::Center => "center".to_string(),
        Align::Right => "right".to_string(),
        Align::None => String::new(),
    }
}

fn render_inline_events(text: &str, inline_pipeline: &Pipeline) -> Vec<Event> {
    if text.trim().is_empty() {
        return Vec::new();
    }
    let parsed = parse(text, &Options::default());
    let processed = inline_pipeline.run(parsed);
    let processed = process_markdown(&processed);

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

pub fn solid_hints(options: &TableOptions) -> Option<SolidRenderHints> {
    let custom = options.custom_node.as_ref()?;
    let mut hints = SolidRenderHints::default();

    let mut register = |comp: &Option<CustomComponent>| {
        if let Some(c) = comp {
            let key = (c.name.clone(), Some(c.name.clone()));
            hints.templates.push(ComponentTemplate {
                node_type: c.name.clone(),
                node_name: Some(c.name.clone()),
                template: c.template.clone(),
            });
            let imports: Vec<ImportEntry> = c
                .imports
                .iter()
                .map(|imp| ImportEntry::Structured {
                    module: imp.module.clone(),
                    default: imp.default.clone(),
                    names: imp.names.clone(),
                })
                .collect();
            if !imports.is_empty() {
                hints.template_imports.insert(key, imports);
            }
        }
    };

    register(&custom.table);
    register(&custom.caption);
    register(&custom.row);
    register(&custom.cell);

    Some(hints)
}
