use pendon_core::Event;

mod context;
mod end;
mod helpers;
mod start;
mod text;

use context::ParseContext;

pub fn process(events: &[Event]) -> Vec<Event> {
    process_with_options(events, MarkdownOptions::default())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MarkdownOptions {
    pub allow_html: bool,
}

impl Default for MarkdownOptions {
    fn default() -> Self {
        Self { allow_html: false }
    }
}

pub fn process_with_options(events: &[Event], opts: MarkdownOptions) -> Vec<Event> {
    let mut ctx = ParseContext::new(events.len(), opts);
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

#[cfg(test)]
mod tests {
    use super::*;
    use pendon_core::{parse, Event, NodeKind, Options};

    fn run_markdown(src: &str, opts: MarkdownOptions) -> Vec<Event> {
        let events = parse(src, &Options::default());
        process_with_options(&events, opts)
    }

    fn html_text(events: &[Event], kind: NodeKind) -> Option<String> {
        let mut iter = events.iter();
        while let Some(ev) = iter.next() {
            match ev {
                Event::StartNode(k) if *k == kind => {
                    if let Some(Event::Text(text)) = iter.next() {
                        return Some(text.clone());
                    }
                }
                _ => {}
            }
        }
        None
    }

    #[test]
    fn html_block_is_emitted_when_allowed() {
        let opts = MarkdownOptions { allow_html: true };
        let events = run_markdown("<div>ok</div>\n", opts);
        assert!(events
            .iter()
            .any(|e| matches!(e, Event::StartNode(NodeKind::HtmlBlock))));
        assert_eq!(
            html_text(&events, NodeKind::HtmlBlock).unwrap(),
            "<div>ok</div>"
        );
    }

    #[test]
    fn html_inline_is_emitted_inside_text() {
        let opts = MarkdownOptions { allow_html: true };
        let events = run_markdown("before <span>inline</span> after\n", opts);
        assert!(events
            .iter()
            .any(|e| matches!(e, Event::StartNode(NodeKind::HtmlInline))));
    }

    #[test]
    fn html_is_ignored_when_disabled() {
        let opts = MarkdownOptions::default();
        let events = run_markdown("<div>ok</div>\n", opts);
        assert!(!events
            .iter()
            .any(|e| matches!(e, Event::StartNode(NodeKind::HtmlBlock))));
    }
}
