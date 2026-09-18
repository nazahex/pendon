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

    let mut i = 0usize;
    while i < events.len() {
        let ev = &events[i];

        if active.is_none() {
            if let Event::Text(line) = ev {
                if detector.is_match(line.trim()) {
                    let captures = detector.captures(line.trim());
                    let (attrs, diags) = attrs::collect_attrs(spec, captures.as_ref());
                    out.extend(diags);
                    let block = ActiveBlock::new(spec.clone(), attrs);

                    if matches!(out.last(), Some(Event::StartNode(NodeKind::Paragraph))) {
                        out.pop();
                    }
                    active = Some(block);
                    i += 1;
                    continue;
                }
            }
            out.push(ev.clone());
            i += 1;
            continue;
        }

        if let Some(block) = active.as_mut() {
            if let Event::Text(line) = ev {
                if line.trim() == end_marker {
                    while matches!(
                        block.inner.last(),
                        Some(Event::StartNode(NodeKind::Paragraph))
                    ) {
                        block.inner.pop();
                    }

                    let mut flushed = active.take().unwrap().finish();
                    out.append(&mut flushed);

                    // Skip the trailing EndNode(Paragraph) to prevent unbalanced JSX tags
                    let mut next_idx = i + 1;
                    while next_idx < events.len()
                        && matches!(events[next_idx], Event::EndNode(NodeKind::Paragraph))
                    {
                        next_idx += 1;
                    }
                    i = next_idx;
                    continue;
                }
            }

            block.inner.push(ev.clone());
        }
        i += 1;
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
}

impl ActiveBlock {
    pub fn new(spec: PluginSpec, attrs: BTreeMap<String, String>) -> Self {
        ActiveBlock {
            spec,
            attrs,
            inner: Vec::new(),
        }
    }

    pub fn finish(self) -> Vec<Event> {
        let mut out = Vec::with_capacity(self.inner.len() + 4);
        util::emit_component(&self.spec, &self.attrs, Some(&self.inner), &mut out);
        out
    }
}
