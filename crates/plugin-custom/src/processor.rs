use crate::specs::AttrType;
use crate::specs::PluginSpec;
use pendon_core::{Event, NodeKind, Severity};
use regex::Regex;
use std::collections::BTreeMap;

pub fn process(events: &[Event], spec: &PluginSpec) -> Vec<Event> {
    match spec.matcher.parse_hint.as_deref() {
        Some("blockquote-sigil") => return process_blockquote_sigil(events, spec),
        Some("codefence-viewer") | Some("codefence-lang") => {
            return process_codefence_lang(events, spec)
        }
        _ => {}
    }

    let Some(detector) = build_start_detector(spec) else {
        return events.to_vec();
    };
    let end_marker = spec
        .matcher
        .end
        .as_deref()
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| ":::".to_string());

    let mut out: Vec<Event> = Vec::with_capacity(events.len() + 4);
    let mut active: Option<ActiveBlock> = None;

    for ev in events.iter() {
        if active.is_none() {
            if let Event::Text(line) = ev {
                if detector.is_match(line.trim()) {
                    let captures = detector.captures(line.trim());
                    let (attrs, diags) = collect_attrs(spec, captures.as_ref());
                    for d in diags {
                        out.push(d);
                    }
                    let mut block = ActiveBlock::new(spec.clone(), attrs);
                    if matches!(out.last(), Some(Event::StartNode(NodeKind::Paragraph))) {
                        out.pop();
                        block.skip_para_close = true;
                    }
                    active = Some(block);
                    continue;
                }
            }
            out.push(ev.clone());
            continue;
        }

        if let Some(block) = active.as_mut() {
            if let Event::Text(line) = ev {
                if line.trim() == end_marker {
                    if let Some(idx) = block.last_para_start.take() {
                        block.inner.truncate(idx);
                        block.skip_para_close = true;
                    }
                    let mut flushed = active.take().unwrap().finish();
                    out.append(&mut flushed);
                    continue;
                }
            }

            match ev {
                Event::Text(line) => {
                    block.inner.push(Event::Text(line.clone()));
                }
                Event::StartNode(NodeKind::Paragraph) => {
                    block.last_para_start = Some(block.inner.len());
                    block.inner.push(ev.clone());
                }
                Event::EndNode(NodeKind::Paragraph) => {
                    if block.skip_para_close {
                        block.skip_para_close = false;
                    } else {
                        block.inner.push(ev.clone());
                    }
                }
                _ => block.inner.push(ev.clone()),
            }
        }
    }

    if let Some(block) = active {
        let mut flushed = block.finish();
        out.append(&mut flushed);
    }

    out
}

#[derive(Debug, Clone)]
struct ActiveBlock {
    spec: PluginSpec,
    attrs: BTreeMap<String, String>,
    inner: Vec<Event>,
    skip_para_close: bool,
    last_para_start: Option<usize>,
}

impl ActiveBlock {
    fn new(spec: PluginSpec, attrs: BTreeMap<String, String>) -> Self {
        ActiveBlock {
            spec,
            attrs,
            inner: Vec::new(),
            skip_para_close: false,
            last_para_start: None,
        }
    }

    fn finish(self) -> Vec<Event> {
        let mut out = Vec::with_capacity(self.inner.len() + 4);
        let nk = resolve_node_kind(&self.spec);
        out.push(Event::StartNode(nk.clone()));
        if let Some(ast) = &self.spec.ast {
            if let Some(name) = &ast.node_name {
                out.push(Event::Attribute {
                    name: "name".to_string(),
                    value: name.clone(),
                });
            }
            if let Some(map) = &ast.attrs_map {
                for (from, to) in map.iter() {
                    if let Some(val) = self.attrs.get(from) {
                        out.push(Event::Attribute {
                            name: to.clone(),
                            value: val.clone(),
                        });
                    }
                }
            }
        }
        out.extend(self.inner.into_iter());
        out.push(Event::EndNode(nk));
        out
    }
}

fn resolve_node_kind(spec: &PluginSpec) -> NodeKind {
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

fn build_start_detector(spec: &PluginSpec) -> Option<Regex> {
    if let Some(re) = &spec.matcher.start_regex {
        Regex::new(re).ok()
    } else if let Some(lit) = &spec.matcher.start {
        let escaped = regex::escape(lit.trim());
        Regex::new(&format!("^{}$", escaped)).ok()
    } else {
        None
    }
}

fn collect_attrs(
    spec: &PluginSpec,
    caps: Option<&regex::Captures>,
) -> (BTreeMap<String, String>, Vec<Event>) {
    let mut out: BTreeMap<String, String> = BTreeMap::new();
    let mut diags: Vec<Event> = Vec::new();

    let kv_blob = caps
        .and_then(|c| c.name("kv"))
        .map(|m| m.as_str().to_string())
        .unwrap_or_default();
    let kv_map = if kv_blob.is_empty() {
        BTreeMap::new()
    } else {
        parse_keyvals(&kv_blob)
    };

    for attr in &spec.attrs {
        let attr_type: AttrType = attr.r#type.as_str().into();
        let mut value: Option<String> = None;
        if let Some(caps) = caps {
            if let Some(m) = caps.name(&attr.name) {
                value = Some(m.as_str().to_string());
            }
        }
        if value.is_none() {
            if let Some(val) = kv_map.get(&attr.name) {
                value = Some(val.clone());
            }
        }
        if value.is_none() {
            if let Some(def) = &attr.default {
                value = Some(def.clone());
            }
        }
        match value {
            Some(v) => match attr_type.parse(&v) {
                Some(parsed) => {
                    out.insert(attr.name.clone(), parsed);
                }
                None => {
                    diags.push(Event::Diagnostic {
                        severity: Severity::Error,
                        message: format!(
                            "[plugin-custom:{}] attribute '{}' failed to parse as {}",
                            spec.name, attr.name, attr.r#type
                        ),
                        span: None,
                    });
                }
            },
            None => {
                if attr.required {
                    diags.push(Event::Diagnostic {
                        severity: Severity::Error,
                        message: format!(
                            "[plugin-custom:{}] missing required attribute '{}'",
                            spec.name, attr.name
                        ),
                        span: None,
                    });
                }
            }
        }
    }

    (out, diags)
}

fn process_blockquote_sigil(events: &[Event], spec: &PluginSpec) -> Vec<Event> {
    let Some(detector) = build_start_detector(spec) else {
        return events.to_vec();
    };
    let mut out = Vec::with_capacity(events.len());
    let mut i = 0usize;
    while i < events.len() {
        if matches!(events[i], Event::StartNode(NodeKind::Blockquote)) {
            let start = i;
            let mut depth = 1usize;
            let mut j = i + 1;
            while j < events.len() && depth > 0 {
                match &events[j] {
                    Event::StartNode(NodeKind::Blockquote) => depth += 1,
                    Event::EndNode(NodeKind::Blockquote) => depth -= 1,
                    _ => {}
                }
                j += 1;
            }
            if depth != 0 {
                out.push(events[i].clone());
                i += 1;
                continue;
            }

            let mut k = start + 1;
            while k + 1 < j {
                if matches!(events[k], Event::StartNode(NodeKind::Paragraph)) {
                    let para_start = k;
                    let mut buf = String::new();
                    let mut body_events: Vec<Event> = Vec::new();
                    k += 1;
                    while k < j && !matches!(events[k], Event::EndNode(NodeKind::Paragraph)) {
                        if let Event::Text(t) = &events[k] {
                            buf.push_str(t);
                        }
                        body_events.push(events[k].clone());
                        k += 1;
                    }
                    if k < j {
                        k += 1; // skip paragraph end
                    }

                    if let Some(caps) = detector.captures(buf.trim()) {
                        let (attrs, diags) = collect_attrs(spec, Some(&caps));
                        out.extend(diags);
                        let cleaned = strip_leading_sigil(&body_events, caps.name("type"));
                        emit_component(spec, &attrs, Some(&cleaned), &mut out);
                    } else {
                        out.extend_from_slice(&events[para_start..k]);
                    }
                } else {
                    out.push(events[k].clone());
                    k += 1;
                }
            }
            i = j;
            continue;
        }

        out.push(events[i].clone());
        i += 1;
    }
    out
}

fn process_codefence_lang(events: &[Event], spec: &PluginSpec) -> Vec<Event> {
    let Some(detector) = build_start_detector(spec) else {
        return events.to_vec();
    };
    let mut out = Vec::with_capacity(events.len());
    let mut i = 0usize;
    while i < events.len() {
        if matches!(events[i], Event::StartNode(NodeKind::CodeFence)) {
            let mut j = i + 1;
            let mut lang: Option<String> = None;
            let mut body = String::new();
            while j < events.len() {
                match &events[j] {
                    Event::Attribute { name, value } if name == "lang" => {
                        lang = Some(value.clone());
                    }
                    Event::Text(t) => body.push_str(t),
                    Event::EndNode(NodeKind::CodeFence) => {
                        j += 1;
                        break;
                    }
                    _ => {}
                }
                j += 1;
            }

            if let Some(l) = lang.as_deref() {
                if detector.is_match(l.trim()) {
                    let (attrs, diags) = collect_attrs(spec, None);
                    out.extend(diags);
                    emit_component_with_body(spec, &attrs, &body, &mut out);
                    i = j;
                    continue;
                }
            }

            out.extend_from_slice(&events[i..j]);
            i = j;
            continue;
        }

        out.push(events[i].clone());
        i += 1;
    }
    out
}

fn emit_component(
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

fn emit_component_with_body(
    spec: &PluginSpec,
    attrs: &BTreeMap<String, String>,
    body: &str,
    out: &mut Vec<Event>,
) {
    let children = [Event::Text(body.to_string())];
    emit_component(spec, attrs, Some(&children), out);
}

fn parse_keyvals(raw: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    let trimmed = raw
        .trim()
        .trim_start_matches('{')
        .trim_end_matches('}')
        .trim();
    if trimmed.is_empty() {
        return map;
    }
    let mut buf = String::new();
    let mut in_quote = false;
    let mut escape = false;
    for ch in trimmed.chars() {
        if escape {
            buf.push(ch);
            escape = false;
            continue;
        }
        if ch == '\\' {
            escape = true;
            continue;
        }
        if ch == '"' {
            in_quote = !in_quote;
            buf.push(ch);
            continue;
        }
        if ch == ',' && !in_quote {
            ingest_kv(&mut map, &buf);
            buf.clear();
            continue;
        }
        buf.push(ch);
    }
    if !buf.is_empty() {
        ingest_kv(&mut map, &buf);
    }
    map
}

fn ingest_kv(map: &mut BTreeMap<String, String>, raw: &str) {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return;
    }
    let (key, val) = match trimmed.split_once(':') {
        Some(pair) => pair,
        None => return,
    };
    map.insert(key.trim().to_string(), val.trim().to_string());
}

fn strip_leading_sigil(events: &[Event], sigil: Option<regex::Match<'_>>) -> Vec<Event> {
    let mut out: Vec<Event> = Vec::with_capacity(events.len());
    let mut stripped = false;
    let sig = sigil.map(|m| m.as_str().to_string());

    for ev in events {
        if !stripped {
            if let (Some(sig), Event::Text(text)) = (&sig, ev) {
                let mut new_text = text.clone();
                if let Some(rest) = new_text.strip_prefix(sig) {
                    new_text = rest.trim_start().to_string();
                    stripped = true;
                }
                out.push(Event::Text(new_text));
                continue;
            }
        }
        out.push(ev.clone());
    }

    out
}
