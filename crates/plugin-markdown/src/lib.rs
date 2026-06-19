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

    fn has_line_break(events: &[Event]) -> bool {
        events
            .iter()
            .any(|ev| matches!(ev, Event::StartNode(NodeKind::HtmlInline)))
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
    fn double_space_line_break_inserts_br() {
        let opts = MarkdownOptions { allow_html: false };
        let events = run_markdown("line  \nnext\n", opts);
        assert!(has_line_break(&events));
    }

    #[test]
    fn double_backslash_line_break_inserts_br() {
        let opts = MarkdownOptions { allow_html: false };
        let events = run_markdown("line\\\\\nnext\n", opts);
        assert!(has_line_break(&events));
    }

    #[test]
    fn html_is_ignored_when_disabled() {
        let opts = MarkdownOptions::default();
        let events = run_markdown("<div>ok</div>\n", opts);
        assert!(!events
            .iter()
            .any(|e| matches!(e, Event::StartNode(NodeKind::HtmlBlock))));
    }

    #[test]
    fn parses_vanilla_markdown_image() {
        let opts = MarkdownOptions::default();
        let events = run_markdown("![Alt](https://example.com/a.webp)\n", opts);

        let mut found = false;
        let mut alt = None::<String>;
        let mut src = None::<String>;
        for ev in events {
            match ev {
                Event::StartNode(NodeKind::Image) => found = true,
                Event::Attribute { name, value } if name == "alt" => alt = Some(value),
                Event::Attribute { name, value } if name == "src" => src = Some(value),
                _ => {}
            }
        }

        assert!(found);
        assert_eq!(alt.as_deref(), Some("Alt"));
        assert_eq!(src.as_deref(), Some("https://example.com/a.webp"));
    }

    #[test]
    fn preserves_double_bang_for_advanced_image_plugin() {
        let opts = MarkdownOptions::default();
        let events = run_markdown("!![Alt](https://example.com/a.webp)\n", opts);

        assert!(!events
            .iter()
            .any(|e| matches!(e, Event::StartNode(NodeKind::Image))));
    }

    #[test]
    fn parses_link_title_without_polluting_href() {
        let opts = MarkdownOptions::default();
        let events = run_markdown("[foo](https://example.com \"Baz Wax\")\n", opts);

        let mut href = None::<String>;
        let mut title = None::<String>;
        for ev in events {
            match ev {
                Event::Attribute { name, value } if name == "href" => href = Some(value),
                Event::Attribute { name, value } if name == "title" => title = Some(value),
                _ => {}
            }
        }

        assert_eq!(href.as_deref(), Some("https://example.com"));
        assert_eq!(title.as_deref(), Some("Baz Wax"));
    }

    #[test]
    fn parses_image_title_without_polluting_src() {
        let opts = MarkdownOptions::default();
        let events = run_markdown("![Alt](https://example.com/a.webp \"Baz Wax\")\n", opts);

        let mut src = None::<String>;
        let mut title = None::<String>;
        for ev in events {
            match ev {
                Event::Attribute { name, value } if name == "src" => src = Some(value),
                Event::Attribute { name, value } if name == "title" => title = Some(value),
                _ => {}
            }
        }

        assert_eq!(src.as_deref(), Some("https://example.com/a.webp"));
        assert_eq!(title.as_deref(), Some("Baz Wax"));
    }

    #[test]
    fn parses_triple_asterisk_as_nested_strong_emphasis() {
        let opts = MarkdownOptions::default();
        let events = run_markdown("Foo ***bar*** baz.\n", opts);

        assert!(events.windows(7).any(|window| {
            matches!(window[0], Event::StartNode(NodeKind::Strong))
                && matches!(window[1], Event::StartNode(NodeKind::Emphasis))
                && matches!(window[2], Event::Text(ref text) if text == "b")
                && matches!(window[3], Event::Text(ref text) if text == "a")
                && matches!(window[4], Event::Text(ref text) if text == "r")
                && matches!(window[5], Event::EndNode(NodeKind::Emphasis))
                && matches!(window[6], Event::EndNode(NodeKind::Strong))
        }));
    }

    #[test]
    fn keeps_inline_link_inside_list_item_for_preprocessed_events() {
        let opts = MarkdownOptions::default();
        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Paragraph),
            Event::Text("- ".to_string()),
            Event::StartNode(NodeKind::Link),
            Event::Attribute {
                name: "href".to_string(),
                value: "/id/wiki/Foo".to_string(),
            },
            Event::Attribute {
                name: "title".to_string(),
                value: "Foo".to_string(),
            },
            Event::Text("Foo".to_string()),
            Event::EndNode(NodeKind::Link),
            Event::Text(": bar".to_string()),
            Event::Text("\n".to_string()),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ];

        let out = process_with_options(&events, opts);

        assert!(out.windows(3).any(|w| {
            matches!(w[0], Event::StartNode(NodeKind::ListItem))
                && matches!(w[1], Event::StartNode(NodeKind::Link))
                && matches!(w[2], Event::Attribute { ref name, .. } if name == "href")
        }));
    }

    #[test]
    fn closes_list_after_blank_line_before_plain_text() {
        let opts = MarkdownOptions::default();
        let events = run_markdown("- Foo\n\nBar\n", opts);

        let bullet_starts = events
            .iter()
            .filter(|ev| matches!(ev, Event::StartNode(NodeKind::BulletList)))
            .count();
        let bullet_ends = events
            .iter()
            .filter(|ev| matches!(ev, Event::EndNode(NodeKind::BulletList)))
            .count();

        let mut saw_list_end = false;
        let mut saw_paragraph_after_list = false;
        for ev in &events {
            match ev {
                Event::EndNode(NodeKind::BulletList) => saw_list_end = true,
                Event::StartNode(NodeKind::Paragraph) if saw_list_end => {
                    saw_paragraph_after_list = true;
                    break;
                }
                _ => {}
            }
        }

        assert_eq!(bullet_starts, 1);
        assert_eq!(bullet_ends, 1);
        assert!(saw_paragraph_after_list);
    }
}
