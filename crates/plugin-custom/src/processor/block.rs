use crate::processor::{attrs, util};
use crate::specs::PluginSpec;
use pendon_core::{Event, NodeKind};
use std::collections::BTreeMap;

pub fn process(events: &[Event], spec: &PluginSpec) -> Vec<Event> {
    let Some(detector) = util::build_start_detector(spec) else {
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
                    let (attrs, diags) = attrs::collect_attrs(spec, captures.as_ref());
                    out.extend(diags);
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
pub struct ActiveBlock {
    spec: PluginSpec,
    attrs: BTreeMap<String, String>,
    inner: Vec<Event>,
    skip_para_close: bool,
    last_para_start: Option<usize>,
}

impl ActiveBlock {
    pub fn new(spec: PluginSpec, attrs: BTreeMap<String, String>) -> Self {
        ActiveBlock {
            spec,
            attrs,
            inner: Vec::new(),
            skip_para_close: false,
            last_para_start: None,
        }
    }

    pub fn finish(self) -> Vec<Event> {
        let mut out = Vec::with_capacity(self.inner.len() + 4);
        util::emit_component(&self.spec, &self.attrs, Some(&self.inner), &mut out);
        out
    }
}
