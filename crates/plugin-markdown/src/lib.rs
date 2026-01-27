use pendon_core::Event;

mod context;
mod end;
mod helpers;
mod start;
mod text;

use context::ParseContext;

pub fn process(events: &[Event]) -> Vec<Event> {
    let mut ctx = ParseContext::new(events.len());

    for ev in events {
        match ev {
            Event::StartNode(kind) => start::handle(&mut ctx, kind),
            Event::EndNode(kind) => end::handle(&mut ctx, kind),
            Event::Text(s) => text::handle(&mut ctx, s),
            Event::Diagnostic { .. } | Event::Attribute { .. } => ctx.push_event(ev),
        }
    }

    ctx.finalize()
}
