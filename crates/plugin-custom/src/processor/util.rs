use crate::specs::PluginSpec;
use pendon_core::{Event, NodeKind};
use regex::Regex;
use std::collections::BTreeMap;

pub fn resolve_node_kind(spec: &PluginSpec) -> NodeKind {
    if let Some(ast) = &spec.ast {
        if let Some(node) = ast.node.as_deref() {
            return match node {
                "Document" => NodeKind::Document,
                "Frontmatter" => NodeKind::Frontmatter,
                "Paragraph" => NodeKind::Paragraph,
                "Blockquote" => NodeKind::Blockquote,
                "CodeFence" => NodeKind::CodeFence,
                "Heading" => NodeKind::Heading,
                "ThematicBreak" => NodeKind::ThematicBreak,
                "BulletList" => NodeKind::BulletList,
                "OrderedList" => NodeKind::OrderedList,
                "ListItem" => NodeKind::ListItem,
                "Table" => NodeKind::Table,
                "TableHead" => NodeKind::TableHead,
                "TableBody" => NodeKind::TableBody,
                "TableRow" => NodeKind::TableRow,
                "TableCell" => NodeKind::TableCell,
                "Section" => NodeKind::Section,
                "Emphasis" => NodeKind::Emphasis,
                "Strong" => NodeKind::Strong,
                "InlineCode" => NodeKind::InlineCode,
                "Link" => NodeKind::Link,
                "Bold" => NodeKind::Bold,
                "Italic" => NodeKind::Italic,
                other => NodeKind::Custom(other.to_string()),
            };
        }
    }
    NodeKind::Custom(spec.name.clone())
}

pub fn build_start_detector(spec: &PluginSpec) -> Option<Regex> {
    if let Some(re) = &spec.matcher.start_regex {
        Regex::new(re).ok()
    } else if let Some(lit) = &spec.matcher.start {
        let escaped = regex::escape(lit.trim());
        Regex::new(&format!("^{}$", escaped)).ok()
    } else {
        None
    }
}

pub fn emit_component(
    spec: &PluginSpec,
    attrs: &BTreeMap<String, String>,
    children: Option<&[Event]>,
    out: &mut Vec<Event>,
) {
    let nk = resolve_node_kind(spec);
    out.push(Event::StartNode(nk.clone()));
    if let Some(ast) = &spec.ast {
        if let Some(name) = &ast.node_name {
            out.push(Event::Attribute {
                name: "name".to_string(),
                value: name.clone(),
            });
        }
        if let Some(map) = &ast.attrs_map {
            for (from, to) in map.iter() {
                if let Some(val) = attrs.get(from) {
                    out.push(Event::Attribute {
                        name: to.clone(),
                        value: val.clone(),
                    });
                }
            }
        }
    }
    if let Some(children) = children {
        out.extend(children.iter().cloned());
    }
    out.push(Event::EndNode(nk));
}

pub fn emit_component_with_body(
    spec: &PluginSpec,
    attrs: &BTreeMap<String, String>,
    body: &str,
    out: &mut Vec<Event>,
) {
    let children = [Event::Text(body.to_string())];
    emit_component(spec, attrs, Some(&children), out);
}
