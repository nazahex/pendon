use pendon_core::{Event, NodeKind};
use pendon_extra::{
    bind_decorators, decorator_diagnostics, BoundDecorator, Claim, Receiver, ReceiverSet, Specs,
};

mod context;
mod end;
mod helpers;
mod math;
mod start;
mod text;

use context::ParseContext;

/// §9.3/§9.4: the list **container** a `plugin-list` wrapper stands for. Layer
/// `unordered` owns the `<ul>`, layer `ordered` the `<ol>`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ContainerLayer {
    Unordered,
    Ordered,
}

/// How a `NodeKind::Custom` / `NodeKind::Element` node should be treated by the
/// Markdown pass, as declared by its `__plugin_kind` attribute (§11 rule 3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CustomPlacement {
    /// Block content: re-lexed as blocks, closing open quotes/lists/tables.
    Block,
    /// Children were already rendered by another plugin (e.g. table cells):
    /// passed through verbatim, never re-lexed.
    Element,
    /// Inline content inside the current line.
    Inline,
    /// §9.3/§9.4: the wrapper `plugin-list` puts around a decorated list
    /// (`__plugin_kind = unordered | ordered`). Its body is re-lexed as blocks
    /// and its node **is** the list container: the list is built inside it
    /// instead of in a `<ul>`/`<ol>` of its own, so the wrapper's attributes
    /// land on the container instead of around it.
    ListContainer(ContainerLayer),
}

pub fn process(events: &[Event]) -> Vec<Event> {
    process_with_options(events, MarkdownOptions::default())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MarkdownOptions {
    pub allow_html: bool,
    pub strip_comments: bool,
}

impl Default for MarkdownOptions {
    fn default() -> Self {
        Self {
            allow_html: false,
            strip_comments: false,
        }
    }
}

/// Renders a string as inline markdown, producing events without any block-level
/// wrapping (no `<p>` tags). Used by `plugin-bind` for inline `mdp` values (§2.3).
pub fn render_inline(s: &str, opts: MarkdownOptions) -> Vec<Event> {
    let mut events = Vec::new();
    helpers::emit_inline(s, &mut events, opts);
    events
}

pub fn process_with_options(events: &[Event], opts: MarkdownOptions) -> Vec<Event> {
    process_with_receivers(events, opts, ReceiverSet::all())
}

/// §9.1/D-10 rules 2–3: the Markdown pass, told which receivers the document's
/// **chain** has.
///
/// The Markdown pass claims the catch-all receiver — a paragraph or a code
/// fence — and, being the last binder to run, also drops the decorators no
/// plugin in `receivers` will ever claim. Without it every such decorator would
/// render as a stray `@@…` line (a heading's decorator in a chain without
/// `plugin-section`, a list's in a chain without `plugin-list`, …). Its blocks
/// carry no §11 component set, so the §6.1 specs stay the built-in
/// `slug` / `title`.
pub fn process_with_receivers(
    events: &[Event],
    opts: MarkdownOptions,
    receivers: ReceiverSet,
) -> Vec<Event> {
    // §9.1: decorator lines that decorate the blocks this pass owns — paragraphs
    // and code fences — are bound here. Every other block belongs to the plugin
    // whose receiver `classify_block` names, and `bind_decorators` leaves its
    // decorator in the stream for that pass whatever order the chain runs in.
    let specs = |_: Option<&str>| Specs::default();
    let claim = Claim::sweeping(Receiver::Paragraph, receivers);
    let bindings = bind_decorators(events, &specs, claim);
    let decorators: std::collections::HashMap<usize, &BoundDecorator> = bindings
        .bound
        .iter()
        .map(|bound| (bound.target_index, bound))
        .collect();

    let mut ctx = ParseContext::new(bindings.events.len(), opts);
    // §9.1: a decorator that bound to nothing — or whose owner is not in the
    // chain — is dropped with a warning.
    for diagnostic in decorator_diagnostics(&bindings, "markdown") {
        ctx.push_event(&diagnostic);
    }

    let mut i = 0;
    while i < bindings.events.len() {
        let ev = &bindings.events[i];
        match ev {
            Event::StartNode(kind) => {
                if let Some(decorator) = decorators.get(&i) {
                    ctx.arm_block_decorator(decorator.type_marker.clone(), decorator.attrs.clone());
                }
                let placement = resolve_placement(kind, &bindings.events[(i + 1)..]);
                start::handle(&mut ctx, kind, placement);
            }
            Event::EndNode(kind) => end::handle(&mut ctx, kind),
            Event::Text(s) => text::handle(&mut ctx, s),
            Event::Diagnostic { .. } | Event::Attribute { .. } | Event::AttributeFlag { .. } => {
                ctx.push_event(ev)
            }
        }
        i += 1;
    }
    ctx.finalize()
}

/// §11 rule 3: the placement of a construct node is declared by the emitting
/// plugin through the hidden `__plugin_kind` attribute it writes first
/// (`plugin-custom`, `plugin-directive`, `plugin-table`).
///
/// Both `Custom` and `Element` nodes honour it. `Element` nodes are also how
/// `plugin-image` / `plugin-table` emit structured HTML whose children are already
/// rendered, so an `Element` **without** the attribute stays verbatim
/// ([`CustomPlacement::Element`]); a `Custom` node without it keeps the historic
/// inline default. Any other attribute may precede it — the very first event
/// that is not an attribute ends the look-ahead.
fn resolve_placement(kind: &NodeKind, following: &[Event]) -> CustomPlacement {
    let default = match kind {
        NodeKind::Element(_) => CustomPlacement::Element,
        _ => CustomPlacement::Inline,
    };
    if !matches!(kind, NodeKind::Custom(_) | NodeKind::Element(_)) {
        return default;
    }
    for event in following {
        match event {
            Event::Attribute { name, value } if name == "__plugin_kind" => {
                return match value.as_str() {
                    "block" | "codefence" | "blockquote" => CustomPlacement::Block,
                    "element" => CustomPlacement::Element,
                    "inline" => CustomPlacement::Inline,
                    // §9.4: the layers of `plugin-list` own the list container.
                    "unordered" => CustomPlacement::ListContainer(ContainerLayer::Unordered),
                    "ordered" => CustomPlacement::ListContainer(ContainerLayer::Ordered),
                    _ => default,
                };
            }
            Event::Attribute { .. } => continue,
            _ => break,
        }
    }
    default
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
    fn structured_heading_events_are_balanced() {
        let events = run_markdown("# Plain\n", MarkdownOptions::default());
        assert!(pendon_core::validate_events(&events).is_empty());
    }

    /// Collects the text of the first heading, ignoring its attributes.
    fn heading_text(events: &[Event]) -> Option<String> {
        let mut iter = events.iter();
        while let Some(event) = iter.next() {
            if matches!(event, Event::StartNode(NodeKind::Heading)) {
                let mut text = String::new();
                for event in iter.by_ref() {
                    match event {
                        Event::EndNode(NodeKind::Heading) => return Some(text),
                        Event::Text(chunk) => text.push_str(chunk),
                        _ => {}
                    }
                }
            }
        }
        None
    }

    /// The core parser emits the `#` run as its own chunk, so the marker has to
    /// be consumed for every depth: only `#` used to be recognised, which leaked
    /// the `##` of every deeper heading into the rendered text.
    #[test]
    fn heading_marker_is_stripped_for_every_depth() {
        let cases = [
            ("# One\n", "One"),
            ("## Two\n", "Two"),
            ("###### Six\n", "Six"),
        ];
        for (source, expected) in cases {
            let events = run_markdown(source, MarkdownOptions::default());

            let depths: Vec<&String> = events
                .iter()
                .filter_map(|event| match event {
                    Event::Attribute { name, value } if name == "depth" => Some(value),
                    _ => None,
                })
                .collect();
            assert_eq!(
                depths.len(),
                1,
                "exactly one depth attribute for {source:?}: {depths:?}"
            );
            let hashes = source.chars().take_while(|&c| c == '#').count().to_string();
            assert_eq!(depths[0], &hashes, "depth attribute for {source:?}");

            let text = heading_text(&events).expect("heading text");
            assert_eq!(text.trim(), expected, "marker leaked for {source:?}");
        }
    }

    fn has_line_break(events: &[Event]) -> bool {
        events
            .iter()
            .any(|ev| matches!(ev, Event::StartNode(NodeKind::HtmlInline)))
    }

    // FIX: Added `..Default::default()` to prevent missing field errors
    #[test]
    fn html_block_is_emitted_when_allowed() {
        let opts = MarkdownOptions {
            allow_html: true,
            ..Default::default()
        };
        let events = run_markdown("<div>ok</div>\n", opts);
        assert!(events
            .iter()
            .any(|e| matches!(e, Event::StartNode(NodeKind::HtmlBlock))));
        assert_eq!(
            html_text(&events, NodeKind::HtmlBlock).unwrap(),
            "<div>ok</div>"
        );
    }

    // FIX: Added `..Default::default()`
    #[test]
    fn html_inline_is_emitted_inside_text() {
        let opts = MarkdownOptions {
            allow_html: true,
            ..Default::default()
        };
        let events = run_markdown("before <span>inline</span> after\n", opts);
        assert!(events
            .iter()
            .any(|e| matches!(e, Event::StartNode(NodeKind::HtmlInline))));
    }

    // FIX: Added `..Default::default()`
    #[test]
    fn double_space_line_break_inserts_br() {
        let opts = MarkdownOptions {
            allow_html: false,
            ..Default::default()
        };
        let events = run_markdown("line  \nnext\n", opts);
        assert!(has_line_break(&events));
    }

    // FIX: Added `..Default::default()`
    #[test]
    fn double_backslash_line_break_inserts_br() {
        let opts = MarkdownOptions {
            allow_html: false,
            ..Default::default()
        };
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

    /// Children of a node marked `__plugin_kind="element"` were already rendered
    /// by their own plugin (every table cell and section of a custom table, for
    /// instance), so they must pass through verbatim. Re-lexing them made the
    /// pass drop the whitespace-only chunks, which glued the words of every
    /// multi-word cell together.
    #[test]
    fn element_placement_passes_children_through_verbatim() {
        let events = vec![
            Event::StartNode(NodeKind::Custom("TableCell".to_string())),
            Event::Attribute {
                name: "__plugin_kind".to_string(),
                value: "element".to_string(),
            },
            Event::Text("Laptop".to_string()),
            Event::Text(" ".to_string()),
            Event::StartNode(NodeKind::Strong),
            Event::Text("Pro".to_string()),
            Event::EndNode(NodeKind::Strong),
            Event::Text(" ".to_string()),
            Event::Text("15".to_string()),
            Event::EndNode(NodeKind::Custom("TableCell".to_string())),
        ];

        let out = process_with_options(&events, MarkdownOptions::default());

        let text: String = out
            .iter()
            .filter_map(|event| match event {
                Event::Text(chunk) => Some(chunk.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(text, "Laptop Pro 15");
        assert!(out
            .iter()
            .any(|e| matches!(e, Event::StartNode(NodeKind::Custom(name)) if name == "TableCell")));
        assert!(out
            .iter()
            .any(|e| matches!(e, Event::StartNode(NodeKind::Strong))));
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
    fn a_leading_bang_is_text_before_an_image() {
        // §7.1 retired the `!!` figure marker (§14), so markdown no longer holds a
        // double bang back: the `!` that touches `[` is the image tag and the first
        // one is ordinary text.
        let opts = MarkdownOptions::default();
        let events = run_markdown("!![Alt](https://example.com/a.webp)\n", opts);

        assert!(events
            .iter()
            .any(|e| matches!(e, Event::StartNode(NodeKind::Image))));
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::Text(text) if text.contains('!'))),
            "the first `!` stays text: {events:?}"
        );
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
    fn keeps_leading_inline_link_inside_paragraph() {
        let opts = MarkdownOptions::default();
        let events = run_markdown("[foo](/docs) bar\n", opts);

        assert!(events.windows(2).any(|window| {
            matches!(window[0], Event::StartNode(NodeKind::Paragraph))
                && matches!(window[1], Event::StartNode(NodeKind::Link))
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

    fn has_node(events: &[Event], kind: NodeKind) -> bool {
        events
            .iter()
            .any(|ev| matches!(ev, Event::StartNode(k) if *k == kind))
    }

    fn text_contains(events: &[Event], needle: &str) -> bool {
        events
            .iter()
            .any(|ev| matches!(ev, Event::Text(t) if t.contains(needle)))
    }

    #[test]
    fn preserves_underscores_inside_inline_math() {
        let opts = MarkdownOptions::default();
        let events = run_markdown("Variabel $O_{m}$ di sini.\n", opts);
        assert!(!has_node(&events, NodeKind::Italic));
        assert!(text_contains(&events, "$O_{m}$"));
    }

    #[test]
    fn preserves_asterisks_inside_display_math() {
        let opts = MarkdownOptions::default();
        let events = run_markdown("$$f(x) = a * x^{2} + b * x + c$$\n", opts);
        assert!(!has_node(&events, NodeKind::Emphasis));
        assert!(text_contains(&events, "a * x"));
    }

    #[test]
    fn preserves_backslash_in_multiline_display_math() {
        let opts = MarkdownOptions::default();
        let src = "$$\\begin{aligned}\nA &= B \\\\\nC &= D\n\\end{aligned}$$\n";
        let events = run_markdown(src, opts);
        assert!(!html_text(&events, NodeKind::HtmlInline).is_some_and(|h| h == "<br />"));
        assert!(text_contains(&events, "\\\\"));
    }

    #[test]
    fn italic_outside_math_still_works() {
        let opts = MarkdownOptions::default();
        let events = run_markdown("_garis bawah untuk miring_\n", opts);
        assert!(has_node(&events, NodeKind::Italic));
    }

    fn all_text(events: &[Event]) -> String {
        events
            .iter()
            .filter_map(|ev| match ev {
                Event::Text(t) => Some(t.as_str()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn dollar_amount_is_not_treated_as_math() {
        let opts = MarkdownOptions::default();
        let events = run_markdown("Nilai $100 dan $500.\n", opts);
        let text = all_text(&events);
        assert!(text.contains("$100"));
        assert!(text.contains("$500"));
    }

    // =========================================================
    // NEW TESTS FOR `strip_comments` FEATURE
    // =========================================================

    #[test]
    fn preserves_html_comments_when_strip_is_false() {
        let opts = MarkdownOptions {
            allow_html: true,
            strip_comments: false,
        };
        let events = run_markdown("<!-- this is a comment -->\n", opts);
        assert!(has_node(&events, NodeKind::HtmlBlock));
        assert!(text_contains(&events, "<!-- this is a comment -->"));
    }

    #[test]
    fn strips_single_line_html_comment() {
        let opts = MarkdownOptions {
            allow_html: true,
            strip_comments: true,
        };
        let events = run_markdown("<!-- this is a comment -->\nParagraph below.\n", opts);
        assert!(!has_node(&events, NodeKind::HtmlBlock));
        let text = all_text(&events);
        assert!(!text.contains("<!--"));
        assert!(text.contains("Paragraph below."));
    }

    #[test]
    fn strips_multiline_html_comment() {
        let opts = MarkdownOptions {
            allow_html: true,
            strip_comments: true,
        };
        let src = "<!--\nThis is a\nmultiline comment\n-->\nReal content.\n";
        let events = run_markdown(src, opts);
        assert!(!has_node(&events, NodeKind::HtmlBlock));
        let text = all_text(&events);
        assert!(!text.contains("multiline comment"));
        assert!(text.contains("Real content."));
    }

    #[test]
    fn strips_inline_html_comment() {
        let opts = MarkdownOptions {
            allow_html: true,
            strip_comments: true,
        };
        let events = run_markdown("Some text <!-- inline --> more text.\n", opts);
        assert!(!has_node(&events, NodeKind::HtmlInline));
        let text = all_text(&events);
        assert!(!text.contains("<!--"));
        assert!(text.contains("Some text"));
        assert!(text.contains("more text."));
    }

    /// Concatenated text of the first `kind` subtree.
    fn node_text(events: &[Event], kind: NodeKind) -> String {
        let mut out = String::new();
        let mut inside = false;
        let mut depth = 0usize;
        for event in events {
            match event {
                Event::StartNode(k) if *k == kind && !inside => {
                    inside = true;
                    depth = 0;
                }
                Event::EndNode(k) if *k == kind && inside && depth == 0 => inside = false,
                Event::StartNode(_) if inside => depth += 1,
                Event::EndNode(_) if inside && depth > 0 => depth -= 1,
                Event::Text(t) if inside => out.push_str(t),
                _ => {}
            }
        }
        out
    }

    /// A code span may start at column 0. The lexer used to split the leading
    /// backtick run into its own token, which left the literal backticks in the
    /// output (and affected every table cell that starts with a code span,
    /// because a cell is parsed as its own document).
    #[test]
    fn inline_code_at_line_start_is_parsed() {
        let events = run_markdown("`code` starts the line.\n", MarkdownOptions::default());
        assert!(has_node(&events, NodeKind::InlineCode));
        assert_eq!(node_text(&events, NodeKind::InlineCode), "code");
        let text = all_text(&events);
        assert!(!text.contains('`'), "backticks leaked: {text}");
        assert!(text.contains("starts the line."));
    }

    /// An indented fence (the fence of a list item) removes up to its own
    /// indentation from every content line, as CommonMark requires.
    #[test]
    fn indented_fence_content_is_dedented() {
        let src = "- item\n\n  ```sh\n  echo hi\n    echo indented\n  ```\n";
        let events = run_markdown(src, MarkdownOptions::default());
        assert!(has_node(&events, NodeKind::CodeFence));
        assert_eq!(
            node_text(&events, NodeKind::CodeFence),
            "echo hi\n  echo indented\n"
        );
    }

    /// Root-level fences keep their content untouched (the fence indentation is
    /// zero, so nothing is removed).
    #[test]
    fn root_fence_content_keeps_indentation() {
        let src = "```sh\n  echo hi\n```\n";
        let events = run_markdown(src, MarkdownOptions::default());
        assert_eq!(node_text(&events, NodeKind::CodeFence), "  echo hi\n");
    }

    fn plugin_kind(value: &str) -> Event {
        Event::Attribute {
            name: "__plugin_kind".to_string(),
            value: value.to_string(),
        }
    }

    /// Mirrors the events `plugin-directive` emits for one directive: the node
    /// (an `Element` fallback for an unclaimed type, §10.2/§10.3 + D8) followed
    /// by `__plugin_kind` as its first attribute and the raw body text.
    fn directive(kind: &str, node: &str, body: &[&str]) -> Vec<Event> {
        let node = NodeKind::Element(node.to_string());
        let mut events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(node.clone()),
            plugin_kind(kind),
            Event::Attribute {
                name: "type".to_string(),
                value: "aside".to_string(),
            },
        ];
        events.extend(body.iter().map(|chunk| Event::Text((*chunk).to_string())));
        events.push(Event::EndNode(node));
        events.push(Event::EndNode(NodeKind::Document));
        events
    }

    /// §11 rule 3: the placement of an `Element` node is read from the hidden
    /// `__plugin_kind` attribute, exactly like a `Custom` node. Only a node that
    /// declares nothing (or `element`) is a pre-rendered subtree.
    #[test]
    fn resolve_placement_reads_the_hidden_attribute() {
        let custom = NodeKind::Custom("Note".to_string());
        let element = NodeKind::Element("div".to_string());
        let plain = [Event::Text("body".to_string())];

        assert_eq!(
            resolve_placement(&custom, &[plugin_kind("block")]),
            CustomPlacement::Block
        );
        assert_eq!(
            resolve_placement(&element, &[plugin_kind("block")]),
            CustomPlacement::Block
        );
        assert_eq!(
            resolve_placement(&element, &[plugin_kind("blockquote")]),
            CustomPlacement::Block
        );
        assert_eq!(
            resolve_placement(&element, &[plugin_kind("inline")]),
            CustomPlacement::Inline
        );
        assert_eq!(
            resolve_placement(&element, &[plugin_kind("element")]),
            CustomPlacement::Element
        );
        // §11 rule 3: another attribute may precede it (`plugin-custom` writes
        // `__plugin_kind` first, but nothing guarantees it).
        assert_eq!(
            resolve_placement(
                &element,
                &[
                    Event::Attribute {
                        name: "class".to_string(),
                        value: "k".to_string(),
                    },
                    plugin_kind("block"),
                ]
            ),
            CustomPlacement::Block
        );
        // Nothing declared: an `Element` is a pre-rendered subtree (the img /
        // table containers), a `Custom` node keeps the historic inline default.
        assert_eq!(
            resolve_placement(&element, &plain),
            CustomPlacement::Element
        );
        assert_eq!(resolve_placement(&custom, &plain), CustomPlacement::Inline);
        // An unknown value never overrides the node's own default.
        assert_eq!(
            resolve_placement(&element, &[plugin_kind("nonsense")]),
            CustomPlacement::Element
        );
    }

    /// §9.3/§9.4: the `unordered` and `ordered` layers of `plugin-list` are
    /// container placements, not plain block nodes.
    #[test]
    fn resolve_placement_reads_the_list_layers() {
        let element = NodeKind::Element("ul".to_string());
        assert_eq!(
            resolve_placement(&element, &[plugin_kind("unordered")]),
            CustomPlacement::ListContainer(ContainerLayer::Unordered)
        );
        assert_eq!(
            resolve_placement(&element, &[plugin_kind("ordered")]),
            CustomPlacement::ListContainer(ContainerLayer::Ordered)
        );
    }

    /// §9.3/§9.4: the wrapper **is** the list container — the items are built
    /// inside it, and no `<ul>` of its own is opened around them.
    #[test]
    fn a_list_container_wrapper_is_the_container() {
        let events = directive("unordered", "ul", &["- one", "\n", "- two", "\n"]);
        let out = process_with_options(&events, MarkdownOptions::default());
        assert!(!has_node(&out, NodeKind::BulletList), "{out:?}");
        assert_eq!(
            out.iter()
                .filter(|event| matches!(event, Event::StartNode(NodeKind::ListItem)))
                .count(),
            2,
            "{out:?}"
        );
        // The wrapper comes first and the items follow it directly.
        let wrapper = out
            .iter()
            .position(
                |event| matches!(event, Event::StartNode(NodeKind::Element(name)) if name == "ul"),
            )
            .expect("wrapper");
        let item = out
            .iter()
            .position(|event| matches!(event, Event::StartNode(NodeKind::ListItem)))
            .expect("item");
        assert!(wrapper < item, "{out:?}");
        assert!(pendon_core::validate_events(&out).is_empty(), "{out:?}");
    }

    /// The wrapper's declared layer decides: an `ordered` wrapper adopts the
    /// ordered list and keeps the `start` offset `plugin-markdown` found (§9.4).
    #[test]
    fn an_ordered_wrapper_keeps_the_start_offset() {
        let events = directive("ordered", "ol", &["6. Goo", "\n"]);
        let out = process_with_options(&events, MarkdownOptions::default());
        assert!(!has_node(&out, NodeKind::OrderedList), "{out:?}");
        assert!(
            out.iter().any(|event| matches!(
                event,
                Event::Attribute { name, value } if name == "start" && value == "6"
            )),
            "{out:?}"
        );
        assert!(pendon_core::validate_events(&out).is_empty(), "{out:?}");
    }

    /// A wrapper whose body turns out not to be a list stays an ordinary block
    /// node, so its attributes and component are never dropped.
    #[test]
    fn a_list_container_wrapper_without_a_list_stays_a_block() {
        let events = directive("unordered", "ul", &["plain text", "\n"]);
        let out = process_with_options(&events, MarkdownOptions::default());
        assert!(has_node(&out, NodeKind::Paragraph), "{out:?}");
        assert!(
            has_node(&out, NodeKind::Element("ul".to_string())),
            "{out:?}"
        );
        assert!(all_text(&out).contains("plain text"), "{out:?}");
        assert!(pendon_core::validate_events(&out).is_empty(), "{out:?}");
    }

    /// A block directive whose `type` no component claims renders `<div>`; its
    /// body must still be re-lexed as block content (§10.3), so lists and inline
    /// markup inside it are not left as raw text.
    #[test]
    fn element_block_directive_body_is_relexed() {
        let events = directive(
            "block",
            "div",
            &["\n", "*a* and **b**", "\n", "- one", "\n", "- two", "\n"],
        );
        let out = process_with_options(&events, MarkdownOptions::default());
        assert!(has_node(&out, NodeKind::BulletList), "{out:?}");
        assert!(has_node(&out, NodeKind::Emphasis), "{out:?}");
        assert!(has_node(&out, NodeKind::Strong), "{out:?}");
        assert!(!text_contains(&out, "**b**"), "raw markup leaked: {out:?}");
    }

    /// An inline directive whose `type` no component claims renders `<span>`;
    /// its body stays inline content (§10.2) and is re-lexed in place.
    #[test]
    fn element_inline_directive_body_is_relexed() {
        let events = directive("inline", "span", &[" *i* content", "\n"]);
        let out = process_with_options(&events, MarkdownOptions::default());
        assert!(has_node(&out, NodeKind::Emphasis), "{out:?}");
        // Inline content never opens a block inside the span.
        assert!(!has_node(&out, NodeKind::BulletList), "{out:?}");
        assert!(all_text(&out).contains("content"));
    }

    /// An `Element` that declares nothing is a structured HTML container whose
    /// children were already rendered: its text is never re-lexed.
    #[test]
    fn element_without_plugin_kind_stays_verbatim() {
        let td = NodeKind::Element("td".to_string());
        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(td.clone()),
            Event::Text("*a* and **b**".to_string()),
            Event::EndNode(td),
            Event::EndNode(NodeKind::Document),
        ];
        let out = process_with_options(&events, MarkdownOptions::default());
        assert!(!has_node(&out, NodeKind::Emphasis), "{out:?}");
        assert!(!has_node(&out, NodeKind::Strong), "{out:?}");
        assert!(all_text(&out).contains("*a* and **b**"));
    }

    /// The `element` placement marks the same pre-rendered subtrees when the
    /// attribute is present (`plugin-table`).
    #[test]
    fn element_placement_keeps_pre_rendered_children() {
        let td = NodeKind::Element("td".to_string());
        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(td.clone()),
            plugin_kind("element"),
            Event::Text("*a*".to_string()),
            Event::EndNode(td),
            Event::EndNode(NodeKind::Document),
        ];
        let out = process_with_options(&events, MarkdownOptions::default());
        assert!(!has_node(&out, NodeKind::Emphasis), "{out:?}");
        assert!(all_text(&out).contains("*a*"));
    }

    /// A pre-rendered `Element` node sits where its marker was, so the text that
    /// follows it continues the same line. Leaving `at_line_start` armed made
    /// the next chunk look like the start of a source line, so its leading space
    /// was stripped as indentation: `<img … />Caption only`.
    #[test]
    fn element_placement_keeps_the_space_after_the_node() {
        let img = NodeKind::Element("img".to_string());
        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Paragraph),
            Event::StartNode(img.clone()),
            Event::EndNode(img),
            Event::Text(" Caption only".to_string()),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ];
        let out = process_with_options(&events, MarkdownOptions::default());
        assert_eq!(all_text(&out), " Caption only", "{out:?}");
    }
}
