use crate::processor::{attrs, util};
use crate::specs::PluginSpec;
use pendon_core::{Event, NodeKind};

pub fn process(events: &[Event], spec: &PluginSpec) -> Vec<Event> {
    let Some(detector) = util::build_start_detector(spec) else {
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
                    let (attrs, diags) = attrs::collect_attrs(spec, None);
                    out.extend(diags);
                    util::emit_component_with_body(spec, &attrs, &body, &mut out);
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
