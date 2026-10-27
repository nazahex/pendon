use pendon_core::{parse, Event, NodeKind, Options, Severity};

fn render_concat(events: &[Event]) -> String {
    let mut s = String::new();
    for ev in events {
        if let Event::Text(t) = ev {
            s.push_str(t);
        }
    }
    s
}

#[test]
fn empty_input_emits_only_document() {
    let ev = parse("", &Options::default());
    assert_eq!(
        ev,
        vec![
            Event::StartNode(NodeKind::Document),
            Event::EndNode(NodeKind::Document)
        ]
    );
}

#[test]
fn single_line_text() {
    let ev = parse("Hello", &Options::default());
    assert_eq!(render_concat(&ev), "Hello");
    assert!(matches!(
        ev.first(),
        Some(Event::StartNode(NodeKind::Document))
    ));
    assert!(matches!(
        ev.last(),
        Some(Event::EndNode(NodeKind::Document))
    ));
}

#[test]
fn single_newline_split() {
    let ev = parse("A\nB", &Options::default());
    assert_eq!(render_concat(&ev), "A\nB");
    // One paragraph
    let p_starts = ev
        .iter()
        .filter(|e| matches!(e, Event::StartNode(NodeKind::Paragraph)))
        .count();
    let p_ends = ev
        .iter()
        .filter(|e| matches!(e, Event::EndNode(NodeKind::Paragraph)))
        .count();
    assert_eq!((p_starts, p_ends), (1, 1));
    assert!(ev.iter().any(|e| matches!(e, Event::Text(t) if t == "A")));
    assert!(ev.iter().any(|e| matches!(e, Event::Text(t) if t == "\n")));
    assert!(ev.iter().any(|e| matches!(e, Event::Text(t) if t == "B")));
}

#[test]
fn double_newline_split() {
    let ev = parse("A\n\nB", &Options::default());
    assert_eq!(render_concat(&ev), "A\n\nB");
    let count_newlines = ev
        .iter()
        .filter(|e| matches!(e, Event::Text(t) if t == "\n"))
        .count();
    assert_eq!(count_newlines, 2);
    // Two paragraphs
    let p_starts = ev
        .iter()
        .filter(|e| matches!(e, Event::StartNode(NodeKind::Paragraph)))
        .count();
    let p_ends = ev
        .iter()
        .filter(|e| matches!(e, Event::EndNode(NodeKind::Paragraph)))
        .count();
    assert_eq!((p_starts, p_ends), (2, 2));
}

#[test]
fn crlf_normalized_to_lf() {
    let ev = parse("A\r\nB", &Options::default());
    assert_eq!(render_concat(&ev), "A\nB");
}

#[test]
fn blank_run_limit_emits_diagnostic_but_preserves_text() {
    let opts = Options {
        strict: false,
        max_doc_bytes: None,
        max_line_len: None,
        max_blank_run: Some(1),
    };
    let ev = parse("A\n\n\nB", &opts);
    // Text is preserved fully
    assert_eq!(render_concat(&ev), "A\n\n\nB");
    // One diagnostic when run first exceeds limit
    let diag_count = ev
        .iter()
        .filter(|e| matches!(e, Event::Diagnostic { .. }))
        .count();
    assert_eq!(diag_count, 1);
}

#[test]
fn leading_and_trailing_blank_lines() {
    let ev = parse("\n\nA\n\n", &Options::default());
    assert_eq!(render_concat(&ev), "\n\nA\n\n");
    let p_starts = ev
        .iter()
        .filter(|e| matches!(e, Event::StartNode(NodeKind::Paragraph)))
        .count();
    let p_ends = ev
        .iter()
        .filter(|e| matches!(e, Event::EndNode(NodeKind::Paragraph)))
        .count();
    assert_eq!((p_starts, p_ends), (1, 1));
}

#[test]
fn strict_escalates_blank_run_to_error() {
    let ev = parse(
        "A\n\n\nB",
        &Options {
            strict: true,
            max_doc_bytes: None,
            max_line_len: None,
            max_blank_run: Some(1),
        },
    );
    assert!(ev.iter().any(
        |e| matches!(e, Event::Diagnostic { severity, .. } if matches!(severity, Severity::Error))
    ));
}

/// `OBJ-1` (ADR-0012, D-11): the core parser implements no footnote extension,
/// so `[^ref]` reaches the slot parser instead of becoming a footnote. There is
/// no footnote node kind at all, so the remaining risk is `[^ref]` becoming a
/// link — which this pins as *not* happening.
#[test]
fn footnote_syntax_is_not_claimed_by_the_core_parser() {
    for source in ["See [^ref] here\n", "[^ref]: a note\n"] {
        let ev = parse(source, &Options::default());
        assert!(
            !ev.iter()
                .any(|e| matches!(e, Event::StartNode(NodeKind::Link))),
            "core turned footnote syntax into a link in {source:?}"
        );
        assert!(
            render_concat(&ev).contains("^ref"),
            "footnote text was swallowed in {source:?}"
        );
    }
}
