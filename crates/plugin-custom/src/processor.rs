mod attrs;
mod block;
mod blockquote;
mod codefence;
mod inline;
mod util;

use crate::specs::PluginSpec;
use pendon_core::Event;

pub fn process(events: &[Event], spec: &PluginSpec) -> Vec<Event> {
    if spec.kind == "inline" {
        return inline::process(events, spec);
    }

    match spec.matcher.parse_hint.as_deref() {
        Some("blockquote-sigil") => blockquote::process(events, spec),
        Some("codefence-viewer") | Some("codefence-lang") => codefence::process(events, spec),
        _ => block::process(events, spec),
    }
}
