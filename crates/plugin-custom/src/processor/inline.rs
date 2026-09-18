use crate::processor::{attrs, util};
use crate::specs::PluginSpec;
use pendon_core::Event;

pub fn process(events: &[Event], spec: &PluginSpec) -> Vec<Event> {
    let Some(detector) = util::build_start_detector(spec) else {
        return events.to_vec();
    };

    let mut out = Vec::with_capacity(events.len());

    for ev in events {
        if let Event::Text(text) = ev {
            let mut last_idx = 0;
            let mut matched = false;

            for caps in detector.captures_iter(text) {
                matched = true;
                if let Some(m) = caps.get(0) {
                    // Emit preceding text
                    if m.start() > last_idx {
                        out.push(Event::Text(text[last_idx..m.start()].to_string()));
                    }

                    // Extract attributes
                    let (parsed_attrs, diags) = attrs::collect_attrs(spec, Some(&caps));
                    out.extend(diags);

                    // Emit inline component
                    util::emit_component(spec, &parsed_attrs, None, &mut out);

                    last_idx = m.end();
                }
            }

            if matched {
                // Emit remaining text after last match
                if last_idx < text.len() {
                    out.push(Event::Text(text[last_idx..].to_string()));
                }
            } else {
                out.push(ev.clone());
            }
        } else {
            out.push(ev.clone());
        }
    }

    out
}
