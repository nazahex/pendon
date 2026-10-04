mod attrs;
mod block;
mod blockquote;
mod codefence;
mod inline;
mod util;

use crate::specs::PluginSpec;
use pendon_core::{Event, InlinePipeline, Pipeline};

pub fn process(events: &[Event], spec: &PluginSpec, pipeline: &Pipeline) -> Vec<Event> {
    process_with_context(events, spec, pipeline, &mut ())
}

pub fn process_with_context<C, P>(
    events: &[Event],
    spec: &PluginSpec,
    pipeline: &P,
    context: &mut C,
) -> Vec<Event>
where
    P: InlinePipeline<C>,
{
    if spec.kind == "inline" {
        return inline::process_with_context(events, spec, pipeline, context);
    }

    match spec.matcher.parse_hint.as_deref() {
        Some("blockquote-sigil") => blockquote::process(events, spec),
        Some("codefence-viewer") | Some("codefence-lang") => codefence::process(events, spec),
        _ => block::process(events, spec),
    }
}
