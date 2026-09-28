use crate::processor::{attrs, util};
use crate::specs::PluginSpec;
use pendon_core::{Event, Pipeline};

pub fn process(events: &[Event], spec: &PluginSpec, pipeline: &Pipeline) -> Vec<Event> {
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

                    // Extract inner content. Convention: use capture group named "body" or "content".
                    // Fallback to "reason" for backward compatibility with existing configs.
                    let inner_text = caps
                        .name("body")
                        .or_else(|| caps.name("content"))
                        .or_else(|| caps.name("reason"))
                        .map(|m| m.as_str().to_string())
                        .unwrap_or_default();

                    // Process inner text through the inline pipeline so that
                    // subsequent plugins (like markdown, anchor, cite) can transform it.
                    let inner_events = if !inner_text.is_empty() {
                        pipeline.run(vec![Event::Text(inner_text)])
                    } else {
                        vec![]
                    };

                    // Emit component with processed children
                    util::emit_component(spec, &parsed_attrs, Some(&inner_events), &mut out);

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
