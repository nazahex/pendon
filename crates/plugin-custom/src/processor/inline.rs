use crate::processor::{attrs, util};
use crate::specs::PluginSpec;
use pendon_core::{Event, InlinePipeline};
use regex::Regex;

pub fn process_with_context<C, P>(
    events: &[Event],
    spec: &PluginSpec,
    pipeline: &P,
    context: &mut C,
) -> Vec<Event>
where
    P: InlinePipeline<C>,
{
    let Some(detector) = util::build_start_detector(spec) else {
        return events.to_vec();
    };

    let mut out = Vec::with_capacity(events.len());
    // Adjacent text events are buffered before matching. The markdown plugin may
    // emit text one character at a time (which is what happens to the content of
    // a `<figcaption>`, `<td>` or any other element rendered by another plugin),
    // so a marker such as `::[ep1] reason::` would otherwise never match. This
    // mirrors how the built-in cite/anchor plugins buffer their text.
    //
    // The run is flushed at every non-text event (and at the end of the stream)
    // through `flush_run`, which restores the line structure the rest of the
    // pipeline depends on.
    let mut pending: Vec<Event> = Vec::new();
    let mut buffered = String::new();

    for ev in events {
        match ev {
            Event::Text(text) => {
                buffered.push_str(text);
                pending.push(ev.clone());
            }
            other => {
                flush_run(
                    &mut buffered,
                    &mut pending,
                    &detector,
                    spec,
                    pipeline,
                    context,
                    &mut out,
                );
                out.push(other.clone());
            }
        }
    }
    flush_run(
        &mut buffered,
        &mut pending,
        &detector,
        spec,
        pipeline,
        context,
        &mut out,
    );

    out
}

/// Emits one buffered run of adjacent text events, restoring its lines.
///
/// The core parser keeps every source line in its own `Text` event and separates
/// them with explicit `Text("\n")` events, and the markdown plugin parses block
/// structure line by line. Flattening a whole paragraph into one multi-line
/// `Text` event would therefore hide every list, blockquote or table boundary
/// from the plugins that run afterwards: a list rendered after such a run turns
/// into literal `- item` text. To keep that from happening the newlines are
/// always re-emitted as their own events, and a marker never matches across a
/// line break.
fn flush_run<C, P>(
    buffered: &mut String,
    pending: &mut Vec<Event>,
    detector: &Regex,
    spec: &PluginSpec,
    pipeline: &P,
    context: &mut C,
    out: &mut Vec<Event>,
) where
    P: InlinePipeline<C>,
{
    if buffered.is_empty() {
        // Only empty text events were buffered: keep them untouched.
        out.append(pending);
        return;
    }

    // Fast path: nothing in this run can match, so the original events are
    // forwarded verbatim (same chunking as the input).
    if !detector.is_match(buffered) {
        out.append(pending);
        buffered.clear();
        return;
    }

    let run = std::mem::take(buffered);
    pending.clear();
    let mut rest = run.as_str();
    while !rest.is_empty() {
        match rest.find('\n') {
            Some(idx) => {
                process_line(&rest[..idx], detector, spec, pipeline, context, out);
                out.push(Event::Text("\n".to_string()));
                rest = &rest[idx + 1..];
            }
            None => {
                process_line(rest, detector, spec, pipeline, context, out);
                break;
            }
        }
    }
}

/// Runs the matcher over one source line and appends the result.
fn process_line<C, P>(
    text: &str,
    detector: &Regex,
    spec: &PluginSpec,
    pipeline: &P,
    context: &mut C,
    out: &mut Vec<Event>,
) where
    P: InlinePipeline<C>,
{
    if text.is_empty() {
        return;
    }

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
                pipeline.run_with(context, vec![Event::Text(inner_text)])
            } else {
                vec![]
            };

            // Emit component with processed children
            util::emit_component(spec, &parsed_attrs, Some(&inner_events), out);

            last_idx = m.end();
        }
    }

    if !matched {
        out.push(Event::Text(text.to_string()));
    } else if last_idx < text.len() {
        // Emit remaining text after last match
        out.push(Event::Text(text[last_idx..].to_string()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pendon_core::{NodeKind, Pipeline};

    fn spec() -> PluginSpec {
        toml::from_str(
            r#"
name = "epis"
kind = "inline"

[matcher]
start_regex = "::\\[ep(?<level>[1-4])\\]\\s*(?<body>.*?)::"

[[attrs]]
name = "level"
type = "int"
required = true

[ast]
node = "Component"
node_name = "Epis"
"#,
        )
        .expect("valid spec")
    }

    fn document(children: Vec<Event>) -> Vec<Event> {
        let mut events = vec![Event::StartNode(NodeKind::Document)];
        events.extend(children);
        events.push(Event::EndNode(NodeKind::Document));
        events
    }

    /// The markdown plugin emits one event per character, so markers that were
    /// rendered inside a `<td>`/`<figcaption>` arrive split across many events.
    fn char_by_char(text: &str) -> Vec<Event> {
        text.chars().map(|c| Event::Text(c.to_string())).collect()
    }

    fn component_names(events: &[Event]) -> Vec<String> {
        events
            .iter()
            .filter_map(|event| match event {
                Event::StartNode(NodeKind::Custom(name)) => Some(name.clone()),
                _ => None,
            })
            .collect()
    }

    fn has_attribute(events: &[Event], name: &str, value: &str) -> bool {
        events.iter().any(|event| {
            matches!(event, Event::Attribute { name: n, value: v } if n == name && v == value)
        })
    }

    #[test]
    fn matches_marker_spanning_char_split_text() {
        let pipeline = Pipeline::default();
        let events = document(char_by_char("cell ::[ep2] reason:: end"));
        let out = crate::processor::process(&events, &spec(), &pipeline);

        assert_eq!(component_names(&out), vec!["Component".to_string()]);
        assert!(has_attribute(&out, "name", "Epis"));
        assert!(has_attribute(&out, "level", "2"));
        // No marker text is left behind.
        assert!(!out
            .iter()
            .any(|event| matches!(event, Event::Text(text) if text.contains("::[ep2]"))));
    }

    #[test]
    fn matches_marker_in_single_text_event() {
        let pipeline = Pipeline::default();
        let events = document(vec![Event::Text("cell ::[ep1] reason:: end".to_string())]);
        let out = crate::processor::process(&events, &spec(), &pipeline);

        assert_eq!(component_names(&out), vec!["Component".to_string()]);
        assert!(has_attribute(&out, "name", "Epis"));
        assert!(has_attribute(&out, "level", "1"));
    }

    #[test]
    fn keeps_text_untouched_when_no_marker_matches() {
        let pipeline = Pipeline::default();
        let events = document(vec![Event::Text("plain text".to_string())]);
        let out = crate::processor::process(&events, &spec(), &pipeline);

        assert!(component_names(&out).is_empty());
        assert_eq!(out, events);
    }

    fn text_events(events: &[Event]) -> Vec<String> {
        events
            .iter()
            .filter_map(|event| match event {
                Event::Text(text) => Some(text.clone()),
                _ => None,
            })
            .collect()
    }

    /// Regression guard: a paragraph that contains a marker used to be flattened
    /// into a single multi-line `Text` event, which made the markdown plugin see
    /// the following blockquote/list lines as literal paragraph text.
    #[test]
    fn keeps_one_text_event_per_source_line_when_a_marker_matches() {
        let pipeline = Pipeline::default();
        let events = document(vec![
            Event::Text("> ? Text ::[ep1] one::".to_string()),
            Event::Text("\n".to_string()),
            Event::Text("> - Est deserunt.".to_string()),
            Event::Text("\n".to_string()),
        ]);
        let out = crate::processor::process(&events, &spec(), &pipeline);

        assert_eq!(component_names(&out), vec!["Component".to_string()]);
        let texts = text_events(&out);
        assert!(
            texts
                .iter()
                .filter(|text| text.as_str() != "\n")
                .all(|text| !text.contains('\n')),
            "text events must not span lines: {texts:?}"
        );
        assert!(
            texts.iter().any(|text| text == "> - Est deserunt."),
            "the blockquote line must stay its own text event: {texts:?}"
        );
        assert_eq!(
            texts.iter().filter(|text| text.as_str() == "\n").count(),
            2,
            "every source line keeps its newline event: {texts:?}"
        );
    }

    /// A marker may sit on any line of a buffered run, not only the first one.
    #[test]
    fn matches_marker_on_a_later_line_of_a_run() {
        let pipeline = Pipeline::default();
        let events = document(vec![Event::Text(
            "intro line\ncell ::[ep2] reason:: end\n".to_string(),
        )]);
        let out = crate::processor::process(&events, &spec(), &pipeline);

        assert_eq!(component_names(&out), vec!["Component".to_string()]);
        assert!(has_attribute(&out, "level", "2"));
        let texts = text_events(&out);
        assert_eq!(texts.first().map(String::as_str), Some("intro line"));
        assert!(
            texts
                .iter()
                .filter(|text| text.as_str() != "\n")
                .all(|text| !text.contains('\n')),
            "text events must not span lines: {texts:?}"
        );
    }

    /// Text without any marker is forwarded exactly as it arrived.
    #[test]
    fn forwards_unmatched_multi_line_runs_verbatim() {
        let pipeline = Pipeline::default();
        let events = document(vec![
            Event::Text("first".to_string()),
            Event::Text("\n".to_string()),
            Event::Text("- second".to_string()),
            Event::Text("\n".to_string()),
        ]);
        let out = crate::processor::process(&events, &spec(), &pipeline);

        assert!(component_names(&out).is_empty());
        assert_eq!(out, events);
    }
}
