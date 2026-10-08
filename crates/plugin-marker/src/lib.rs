//! §10.1: the `{{type}}` marker.
//!
//! `type` is **mandatory** — `{{}}`, a missing `}}` and anything that does not
//! start with `{{` stay literal text (§10.1, §4.3). The extras head must be
//! adjacent (§4.1) and may be malformed (then it stays literal text, while the
//! marker itself still renders).
//!
//! Two forms:
//!
//! * **inline** — a marker inside a paragraph. It becomes an inline node and the
//!   surrounding text stays inline; without a configured component the fallback
//!   is `<span>`.
//! * **block** — the marker owns its line. It becomes a block node whose
//!   children are the trailing text of the same line and it does not absorb the
//!   next block; without a configured component the fallback is `<div>`.
//!
//! Routing (§11 rule 3): the marker type is the routing key and is emitted as the
//! `type` attribute of the node, the same convention `plugin-custom` uses for its
//! directive types. An extras head that carries its own type is reported and
//! ignored: a marker has exactly one type.

use pendon_core::{element_close, element_open, Event, NodeKind, Severity};
use pendon_extra::{
    read_positional_groups, scan_extras_chars, to_attributes, warning_message, ExtrasAttr,
    ExtrasHead, ExtrasWarning,
};
use pendon_renderer_solid::{ComponentSet, ComponentTemplate, ImportEntry, SolidRenderHints};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MarkerCustomNode {
    pub name: String,
    pub template: String,
    #[serde(default)]
    pub imports: Vec<ImportEntry>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MarkerOptions {
    /// §11 component set of the `marker` layer: typed entries plus at most one
    /// default, selected by the marker type (§11 rule 3).
    pub custom: ComponentSet<MarkerCustomNode>,
}

/// One parsed marker: its mandatory type and the attributes of the optional
/// adjacent extras head.
#[derive(Debug, Clone, Default, PartialEq)]
struct ParsedMarker {
    type_name: String,
    attrs: Vec<(String, ExtrasAttr)>,
    warnings: Vec<String>,
}

pub fn process(events: &[Event], options: &MarkerOptions) -> Vec<Event> {
    let mut out = Vec::with_capacity(events.len());
    let mut excluded = 0usize;
    let mut index = 0usize;

    while index < events.len() {
        match &events[index] {
            // The block form: a paragraph that starts with a marker becomes the
            // marker node itself (no `<p>` wrapper, no absorbed next block).
            Event::StartNode(NodeKind::Paragraph) if excluded == 0 => {
                if let Some(end) = matching_paragraph_end(&events, index) {
                    if let Some((marker, children)) = block_marker(&events[index + 1..end], options)
                    {
                        emit_marker(&marker, true, options, children.as_deref(), &mut out);
                        index = end + 1;
                        continue;
                    }
                }
                out.push(events[index].clone());
                index += 1;
            }
            Event::StartNode(kind) => {
                if is_verbatim(kind) {
                    excluded += 1;
                }
                out.push(events[index].clone());
                index += 1;
            }
            Event::EndNode(kind) => {
                if is_verbatim(kind) {
                    excluded = excluded.saturating_sub(1);
                }
                out.push(events[index].clone());
                index += 1;
            }
            Event::Text(text) if excluded == 0 => {
                emit_inline_text(text, options, &mut out);
                index += 1;
            }
            other => {
                out.push(other.clone());
                index += 1;
            }
        }
    }

    out
}

pub fn solid_hints(options: &MarkerOptions) -> Option<SolidRenderHints> {
    if options.custom.is_empty() {
        return None;
    }
    let mut hints = SolidRenderHints::default();
    // §11 rule 3: every entry answers the marker types it declares.
    for custom in options.custom.components() {
        hints.templates.push(ComponentTemplate {
            node_type: custom.name.clone(),
            node_name: Some(custom.name.clone()),
            template: custom.template.clone(),
        });
        if !custom.imports.is_empty() {
            hints.template_imports.insert(
                (custom.name.clone(), Some(custom.name.clone())),
                custom.imports.clone(),
            );
        }
    }
    Some(hints)
}

/// Nodes whose text is copied verbatim (code, raw HTML): `{{…}}` inside them is
/// literal text.
fn is_verbatim(kind: &NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::CodeFence | NodeKind::InlineCode | NodeKind::HtmlBlock | NodeKind::HtmlInline
    )
}

/// Scans `{{type}}` and its adjacent extras head at `start`.
///
/// `None` when the text is not a marker, which keeps `{{`, `{{}}`, a missing
/// `}}` and a type with illegal characters as literal text (§10.1, §4.3).
fn scan_marker(
    chars: &[char],
    start: usize,
    options: &MarkerOptions,
) -> Option<(ParsedMarker, usize)> {
    if chars.get(start) != Some(&'{') || chars.get(start + 1) != Some(&'{') {
        return None;
    }

    let mut cursor = start + 2;
    let type_start = cursor;
    // §4.4: ALPHA ( ALPHA | DIGIT )*, ASCII only.
    while let Some(character) = chars.get(cursor) {
        let allowed =
            character.is_ascii_alphabetic() || (character.is_ascii_digit() && cursor > type_start);
        if !allowed {
            break;
        }
        cursor += 1;
    }

    // §10.1: the type is mandatory.
    let type_name: String = chars[type_start..cursor].iter().collect();
    if type_name.is_empty()
        || chars.get(cursor) != Some(&'}')
        || chars.get(cursor + 1) != Some(&'}')
    {
        return None;
    }
    cursor += 2;

    let mut marker = ParsedMarker {
        type_name,
        ..ParsedMarker::default()
    };

    // §6.1/§10.1: the positional groups sit directly after `}}`, before any
    // extras head (`{{type}}[…](…){…}`). They are optional; a malformed group
    // is not consumed and stays literal text with the marker (§4.3).
    let mut group_bracket: Option<String> = None;
    let mut group_parentheses: Option<String> = None;
    if cursor < chars.len() {
        let text: String = chars[cursor..].iter().collect();
        if let Ok((bracket, parentheses, consumed)) = read_positional_groups(&text) {
            if bracket.is_some() || parentheses.is_some() {
                group_bracket = bracket;
                group_parentheses = parentheses;
                cursor += text[..consumed].chars().count();
            }
        }
    }

    // §6.1/§11 rule 5: the entry answering this marker `type` names the
    // extras positional keys; the groups map through the same keys.
    let keys = options.custom.keys_for(Some(marker.type_name.as_str()));

    // §4.1: the extras head must be adjacent; a malformed one is left alone
    // (the marker still renders, §4.3).
    if let Some((mut head, next)) = scan_extras_chars(chars, cursor) {
        cursor = next;
        // §6.2 head-wins: the marker's own groups beat the extras head's.
        if group_bracket.is_some() && head.bracket.is_some() {
            marker
                .warnings
                .push(warning_message(&ExtrasWarning::Overridden {
                    key: keys.bracket_key.clone(),
                }));
        }
        if group_parentheses.is_some() && head.parentheses.is_some() {
            marker
                .warnings
                .push(warning_message(&ExtrasWarning::Overridden {
                    key: keys.parentheses_key.clone(),
                }));
        }
        if group_bracket.is_some() {
            head.bracket = group_bracket.clone();
        }
        if group_parentheses.is_some() {
            head.parentheses = group_parentheses.clone();
        }
        let parsed = to_attributes(&head, &keys);
        for warning in &parsed.warnings {
            marker.warnings.push(warning_message(warning));
        }
        if head.type_marker.is_some() {
            marker.warnings.push(
                "a marker is routed by its own type; the extras head type was ignored (§10.1)"
                    .to_string(),
            );
        }
        marker.attrs = parsed.items;
    } else if group_bracket.is_some() || group_parentheses.is_some() {
        // Groups without an extras head still resolve to attributes (§6.1).
        let head = ExtrasHead {
            bracket: group_bracket,
            parentheses: group_parentheses,
            ..ExtrasHead::default()
        };
        let parsed = to_attributes(&head, &keys);
        for warning in &parsed.warnings {
            marker.warnings.push(warning_message(warning));
        }
        marker.attrs = parsed.items;
    }

    Some((marker, cursor))
}

/// The block form: a paragraph whose text starts (after whitespace) with a
/// marker. Returns the marker and the trailing text of the same line, which
/// becomes its `children`.
fn block_marker(
    inner: &[Event],
    options: &MarkerOptions,
) -> Option<(ParsedMarker, Option<String>)> {
    if !inner
        .iter()
        .all(|event| matches!(event, Event::Text(_) | Event::Diagnostic { .. }))
    {
        return None;
    }
    let text: String = inner
        .iter()
        .filter_map(|event| match event {
            Event::Text(text) => Some(text.as_str()),
            _ => None,
        })
        .collect();

    let chars: Vec<char> = text.chars().collect();
    let start = chars
        .iter()
        .take_while(|character| character.is_whitespace())
        .count();
    let (marker, next) = scan_marker(&chars, start, options)?;
    let rest: String = chars[next..].iter().collect();
    let rest = rest.trim().to_string();
    Some((marker, (!rest.is_empty()).then_some(rest)))
}

/// The inline form: markers inside a paragraph keep their surrounding text.
fn emit_inline_text(text: &str, options: &MarkerOptions, out: &mut Vec<Event>) {
    let chars: Vec<char> = text.chars().collect();
    let mut cursor = 0usize;
    let mut plain = String::new();

    while cursor < chars.len() {
        if chars[cursor] == '{' && chars.get(cursor + 1) == Some(&'{') {
            if let Some((marker, next)) = scan_marker(&chars, cursor, options) {
                flush(&mut plain, out);
                emit_marker(&marker, false, options, None, out);
                cursor = next;
                continue;
            }
        }
        plain.push(chars[cursor]);
        cursor += 1;
    }

    flush(&mut plain, out);
}

fn emit_marker(
    marker: &ParsedMarker,
    block: bool,
    options: &MarkerOptions,
    children: Option<&str>,
    out: &mut Vec<Event>,
) {
    for message in &marker.warnings {
        out.push(Event::Diagnostic {
            severity: Severity::Warning,
            message: format!("[marker] {message}"),
            span: None,
        });
    }

    match options.custom.select(Some(&marker.type_name)) {
        Some(custom) => {
            let node = NodeKind::Custom(custom.name.clone());
            out.push(Event::StartNode(node.clone()));
            out.push(Event::Attribute {
                name: "name".to_string(),
                value: custom.name.clone(),
            });
            push_attrs(marker, out);
            if let Some(children) = children {
                out.push(Event::Text(children.to_string()));
            }
            out.push(Event::EndNode(node));
        }
        None => {
            let tag = if block { "div" } else { "span" };
            out.extend(element_open(tag));
            push_attrs(marker, out);
            if let Some(children) = children {
                out.push(Event::Text(children.to_string()));
            }
            out.push(element_close(tag));
        }
    }
}

/// The node's attributes: its `type` (the routing key, §11 rule 3) followed by
/// the extras head items, bare flags included (§6.3).
fn push_attrs(marker: &ParsedMarker, out: &mut Vec<Event>) {
    out.push(Event::Attribute {
        name: "type".to_string(),
        value: marker.type_name.clone(),
    });
    for (key, value) in &marker.attrs {
        match value {
            ExtrasAttr::Flag => out.push(Event::AttributeFlag { name: key.clone() }),
            ExtrasAttr::Value(value) => out.push(Event::Attribute {
                name: key.clone(),
                value: value.literal(),
            }),
        }
    }
}

fn flush(text: &mut String, out: &mut Vec<Event>) {
    if !text.is_empty() {
        out.push(Event::Text(std::mem::take(text)));
    }
}

fn matching_paragraph_end(events: &[Event], start: usize) -> Option<usize> {
    events
        .iter()
        .enumerate()
        .skip(start + 1)
        .find_map(|(index, event)| {
            matches!(event, Event::EndNode(NodeKind::Paragraph)).then_some(index)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pendon_renderer_solid::TypedComponent;

    fn doc(inner: Vec<Event>) -> Vec<Event> {
        let mut events = vec![Event::StartNode(NodeKind::Document)];
        events.extend(inner);
        events.push(Event::EndNode(NodeKind::Document));
        events
    }

    fn paragraph(text: &str) -> Vec<Event> {
        doc(vec![
            Event::StartNode(NodeKind::Paragraph),
            Event::Text(text.to_string()),
            Event::EndNode(NodeKind::Paragraph),
        ])
    }

    fn attr(events: &[Event], name: &str) -> Option<String> {
        events.iter().find_map(|event| match event {
            Event::Attribute { name: n, value } if n == name => Some(value.clone()),
            _ => None,
        })
    }

    fn flag(events: &[Event], name: &str) -> bool {
        events
            .iter()
            .any(|event| matches!(event, Event::AttributeFlag { name: n } if n == name))
    }

    fn text_of(events: &[Event]) -> String {
        events
            .iter()
            .filter_map(|event| match event {
                Event::Text(text) => Some(text.as_str()),
                _ => None,
            })
            .collect()
    }

    fn elements(events: &[Event]) -> Vec<String> {
        events
            .iter()
            .filter_map(|event| match event {
                Event::StartNode(NodeKind::Element(name)) => Some(name.clone()),
                _ => None,
            })
            .collect()
    }

    fn customs(events: &[Event]) -> Vec<String> {
        events
            .iter()
            .filter_map(|event| match event {
                Event::StartNode(NodeKind::Custom(name)) => Some(name.clone()),
                _ => None,
            })
            .collect()
    }

    fn warnings(events: &[Event]) -> Vec<String> {
        events
            .iter()
            .filter_map(|event| match event {
                Event::Diagnostic { message, .. } => Some(message.clone()),
                _ => None,
            })
            .collect()
    }

    fn component(name: &str) -> MarkerCustomNode {
        MarkerCustomNode {
            name: name.to_string(),
            template: format!("<{name} />"),
            imports: Vec::new(),
        }
    }

    fn typed(types: &[&str], name: &str) -> TypedComponent<MarkerCustomNode> {
        TypedComponent::typed(types.to_vec(), component(name))
    }

    fn run(events: &[Event], options: &MarkerOptions) -> Vec<Event> {
        process(events, options)
    }

    /// §10.1: an inline marker keeps its paragraph and renders `<span>` when no
    /// component is configured.
    #[test]
    fn inline_marker_falls_back_to_a_span() {
        let out = run(
            &paragraph("Text {{note}} more text."),
            &MarkerOptions::default(),
        );
        assert_eq!(elements(&out), vec!["span".to_string()]);
        assert_eq!(attr(&out, "type").as_deref(), Some("note"));
        assert_eq!(text_of(&out), "Text  more text.");
        assert!(out
            .iter()
            .any(|event| matches!(event, Event::StartNode(NodeKind::Paragraph))));
    }

    /// §10.1: the block form owns its line, becomes a block node and carries the
    /// trailing text of the same line as its children.
    #[test]
    fn block_marker_falls_back_to_a_div_with_children() {
        let out = run(
            &paragraph("{{note}} Trailing caption."),
            &MarkerOptions::default(),
        );
        assert_eq!(elements(&out), vec!["div".to_string()]);
        assert_eq!(attr(&out, "type").as_deref(), Some("note"));
        assert_eq!(text_of(&out), "Trailing caption.");
        assert!(!out
            .iter()
            .any(|event| matches!(event, Event::StartNode(NodeKind::Paragraph))));
    }

    /// §10.1: the type is mandatory; `{{}}`, `{{`, a spaced type and an illegal
    /// type name stay literal text.
    #[test]
    fn a_marker_without_a_legal_type_stays_literal_text() {
        for source in ["{{}}", "{{", "{{ note }}", "{{a-b}}", "{note}"] {
            let out = run(&paragraph(source), &MarkerOptions::default());
            assert!(
                elements(&out).is_empty(),
                "{source} must stay literal: {out:?}"
            );
            assert_eq!(text_of(&out), source);
        }
    }

    /// §4.1: the extras head must be adjacent to the marker.
    #[test]
    fn extras_head_must_be_adjacent() {
        let adjacent = run(
            &paragraph("{{note}}@@noteX{.hero}"),
            &MarkerOptions::default(),
        );
        assert_eq!(attr(&adjacent, "class").as_deref(), Some("hero"));

        let spaced = run(
            &paragraph("{{note}} @@noteX{.hero}"),
            &MarkerOptions::default(),
        );
        assert_eq!(attr(&spaced, "class"), None);
        assert!(text_of(&spaced).contains("@@noteX"), "{}", text_of(&spaced));
    }

    /// §4.3: a malformed extras head stays literal text while the marker still
    /// renders.
    #[test]
    fn malformed_extras_stay_literal_text() {
        let out = run(
            &paragraph("{{note}}@@noteX{`unterminated"),
            &MarkerOptions::default(),
        );
        assert_eq!(attr(&out, "type").as_deref(), Some("note"));
        assert!(text_of(&out).contains("@@noteX"), "{}", text_of(&out));
    }

    /// §6.3/§6.4: props, classes and bare flags of the extras head land on the
    /// node.
    #[test]
    fn extras_items_become_attributes() {
        let out = run(
            &paragraph("{{note}}@@noteX{.hero, level: 2, --tone: \"red\", isOpen}"),
            &MarkerOptions::default(),
        );
        assert_eq!(attr(&out, "class").as_deref(), Some("hero"));
        assert_eq!(attr(&out, "level").as_deref(), Some("2"));
        assert_eq!(attr(&out, "style").as_deref(), Some("--tone: red"));
        assert!(flag(&out, "isOpen"));
    }

    /// §6.1/§10.1: the positional groups sit directly after `}}`, before any
    /// extras head, and map through the marker type's keys.
    #[test]
    fn groups_follow_the_marker_type() {
        // Groups without an extras head still become attributes.
        let out = run(
            &paragraph("{{note}}[intro](\"A title\") trailing"),
            &MarkerOptions::default(),
        );
        assert_eq!(attr(&out, "type").as_deref(), Some("note"));
        assert_eq!(attr(&out, "slug").as_deref(), Some("intro"));
        assert_eq!(attr(&out, "title").as_deref(), Some("A title"));
        assert!(text_of(&out).contains("trailing"));

        // The groups precede the extras head: {{type}}[…] (…)@@head{…}.
        let out = run(
            &paragraph("{{note}}[intro](\"T\")@@noteX{.hero}"),
            &MarkerOptions::default(),
        );
        assert_eq!(attr(&out, "slug").as_deref(), Some("intro"));
        assert_eq!(attr(&out, "title").as_deref(), Some("T"));
        assert_eq!(attr(&out, "class").as_deref(), Some("hero"));

        // Adjacency: a space before `[` keeps the rest literal.
        let out = run(
            &paragraph("{{note}} [intro] trailing"),
            &MarkerOptions::default(),
        );
        assert_eq!(attr(&out, "slug"), None);
        assert!(text_of(&out).contains("[intro]"), "{}", text_of(&out));
    }

    /// §6.2: the marker's own group beats the extras head's same-key group,
    /// dropped with an `Overridden` warning; a malformed group stays literal.
    #[test]
    fn marker_groups_win_and_malformed_groups_stay_literal() {
        let out = run(
            &paragraph("{{note}}[mine]@@noteX[theirs]{}"),
            &MarkerOptions::default(),
        );
        assert_eq!(attr(&out, "slug").as_deref(), Some("mine"));
        assert!(
            warnings(&out)
                .iter()
                .any(|message| message.contains("slug")),
            "{:?}",
            warnings(&out)
        );

        // §4.3: an unterminated group is not consumed; the marker still
        // renders and the text stays literal.
        let out = run(
            &paragraph("{{note}}[unclosed trailing"),
            &MarkerOptions::default(),
        );
        assert_eq!(attr(&out, "type").as_deref(), Some("note"));
        assert_eq!(attr(&out, "slug"), None);
        assert!(text_of(&out).contains("[unclosed"), "{}", text_of(&out));
    }

    /// §11 rule 3: the marker type routes to its own entry, an unclaimed type to
    /// the layer default, and a layer without a default to the fallback element.
    #[test]
    fn the_marker_type_routes_the_component_set() {
        let options = MarkerOptions {
            custom: ComponentSet::from_entries([
                typed(&["note", "aside"], "NoteAside"),
                TypedComponent::default_component(component("MarkerDefault")),
            ]),
        };

        let out = run(&paragraph("{{note}} inline"), &options);
        assert_eq!(customs(&out), vec!["NoteAside".to_string()]);

        let out = run(&paragraph("{{unknown}}"), &options);
        assert_eq!(customs(&out), vec!["MarkerDefault".to_string()]);

        // Without a layer default the built-in element renders; the paragraph
        // holds only the marker, so this is the block form (`<div>`).
        let typed_only = MarkerOptions {
            custom: ComponentSet::from_entries([typed(&["note"], "Note")]),
        };
        let out = run(&paragraph("{{unknown}}"), &typed_only);
        assert_eq!(elements(&out), vec!["div".to_string()]);
        assert!(customs(&out).is_empty());
    }

    /// Every entry of the set needs a template, typed or not.
    #[test]
    fn hints_cover_every_entry() {
        let options = MarkerOptions {
            custom: ComponentSet::from_entries([
                typed(&["note"], "Note"),
                TypedComponent::default_component(component("MarkerDefault")),
            ]),
        };
        let hints = solid_hints(&options).expect("hints");
        assert_eq!(hints.templates.len(), 2);
        assert!(solid_hints(&MarkerOptions::default()).is_none());
    }

    /// Code and raw HTML keep their text; a marker there is literal.
    #[test]
    fn verbatim_nodes_keep_their_text() {
        let events = doc(vec![
            Event::StartNode(NodeKind::CodeFence),
            Event::Text("{{note}}".to_string()),
            Event::EndNode(NodeKind::CodeFence),
            Event::StartNode(NodeKind::Paragraph),
            Event::StartNode(NodeKind::InlineCode),
            Event::Text("{{note}}".to_string()),
            Event::EndNode(NodeKind::InlineCode),
            Event::EndNode(NodeKind::Paragraph),
        ]);
        let out = run(&events, &MarkerOptions::default());
        assert!(elements(&out).is_empty(), "{out:?}");
        assert_eq!(text_of(&out), "{{note}}{{note}}");
    }

    /// The block form must not absorb the following block.
    #[test]
    fn block_marker_does_not_absorb_the_next_block() {
        let events = doc(vec![
            Event::StartNode(NodeKind::Paragraph),
            Event::Text("{{note}} caption".to_string()),
            Event::EndNode(NodeKind::Paragraph),
            Event::StartNode(NodeKind::Paragraph),
            Event::Text("Next block".to_string()),
            Event::EndNode(NodeKind::Paragraph),
        ]);
        let out = run(&events, &MarkerOptions::default());
        assert_eq!(elements(&out), vec!["div".to_string()]);
        assert!(text_of(&out).contains("Next block"));
        assert!(!text_of(&out).contains("{{note}}"));
    }

    /// A marker has exactly one type: an extras head type is reported and
    /// ignored rather than silently overriding the routing key.
    #[test]
    fn an_extras_type_is_reported_and_ignored() {
        let out = run(&paragraph("{{note}}@@noteX{}"), &MarkerOptions::default());
        assert_eq!(attr(&out, "type").as_deref(), Some("note"));
        assert!(
            warnings(&out)
                .iter()
                .any(|message| message.contains("routed by its own type")),
            "{:?}",
            warnings(&out)
        );
    }
}
