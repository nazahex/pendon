use pendon_core::{Event, NodeKind};
use pendon_renderer_html;
use pendon_renderer_solid::{ComponentTemplate, ImportEntry, SolidRenderHints};
use regex::Regex;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
struct Choice {
    content: String,
    correct: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
struct Feedback {
    correct: Option<String>,
    wrong: Option<String>,
}

pub fn process(events: &[Event]) -> Vec<Event> {
    let mut out: Vec<Event> = Vec::with_capacity(events.len());
    let mut i = 0usize;
    while i < events.len() {
        if let Some(start_end) = marker_paragraph_at(events, i, ":::quiz") {
            let content_start = start_end + 1;
            if let Some((close_start, close_end)) = find_marker(events, content_start, ":::") {
                let inner = &events[content_start..close_start];
                if let Some(mut built) = build_quiz_component(inner) {
                    out.append(&mut built);
                } else {
                    out.extend_from_slice(inner);
                }
                i = close_end + 1;
                continue;
            }
        }
        out.push(events[i].clone());
        i += 1;
    }

    out
}

fn marker_paragraph_at(events: &[Event], idx: usize, marker: &str) -> Option<usize> {
    if !matches!(events.get(idx), Some(Event::StartNode(NodeKind::Paragraph))) {
        return None;
    }
    let mut buf = String::new();
    let mut j = idx + 1;
    while j < events.len() {
        match &events[j] {
            Event::Text(t) => buf.push_str(t),
            Event::EndNode(NodeKind::Paragraph) => {
                return if buf.trim() == marker { Some(j) } else { None };
            }
            _ => return None,
        }
        j += 1;
    }
    None
}

fn find_marker(events: &[Event], start: usize, marker: &str) -> Option<(usize, usize)> {
    let mut i = start;
    while i < events.len() {
        if let Some(end_idx) = marker_paragraph_at(events, i, marker) {
            return Some((i, end_idx));
        }
        i += 1;
    }
    None
}

fn build_quiz_component(inner: &[Event]) -> Option<Vec<Event>> {
    let list_range = find_first_range(inner, |k| {
        matches!(k, NodeKind::BulletList | NodeKind::OrderedList)
    });
    let (feedback_range, feedback) = find_feedback(inner);

    let question_events = filter_out_ranges(inner, &[list_range, feedback_range]);
    let question_html = render_fragment(&question_events).trim().to_string();

    let choices = list_range
        .and_then(|(s, e)| extract_choices(&inner[s..=e]))
        .unwrap_or_default();

    if question_html.is_empty() && choices.is_empty() {
        return None;
    }

    let choices_json = serde_json::to_string(&choices).unwrap_or_else(|_| "[]".to_string());
    let feedback_json = serde_json::to_string(&feedback).unwrap_or_else(|_| "{}".to_string());
    let mut out: Vec<Event> = Vec::with_capacity(question_events.len() + 6);
    out.push(Event::StartNode(NodeKind::Custom("Quiz".to_string())));
    out.push(Event::Attribute {
        name: "name".to_string(),
        value: "Quiz".to_string(),
    });
    out.push(Event::Attribute {
        name: "choices".to_string(),
        value: choices_json,
    });
    out.push(Event::Attribute {
        name: "feedback".to_string(),
        value: feedback_json,
    });
    out.extend(question_events);
    out.push(Event::EndNode(NodeKind::Custom("Quiz".to_string())));

    Some(out)
}

fn filter_out_ranges(events: &[Event], ranges: &[Option<(usize, usize)>]) -> Vec<Event> {
    let mut out = Vec::new();
    for (idx, ev) in events.iter().enumerate() {
        if ranges
            .iter()
            .any(|r| matches!(r, Some((s, e)) if idx >= *s && idx <= *e))
        {
            continue;
        }
        out.push(ev.clone());
    }
    out
}

fn find_first_range<F>(events: &[Event], matcher: F) -> Option<(usize, usize)>
where
    F: Fn(&NodeKind) -> bool,
{
    let mut i = 0usize;
    while i < events.len() {
        if let Event::StartNode(kind) = &events[i] {
            if matcher(kind) {
                let end = matching_end(events, i)?;
                return Some((i, end));
            }
        }
        i += 1;
    }
    None
}

fn matching_end(events: &[Event], start_idx: usize) -> Option<usize> {
    let mut depth = 0isize;
    for (idx, ev) in events.iter().enumerate().skip(start_idx) {
        match ev {
            Event::StartNode(_) => depth += 1,
            Event::EndNode(_) => {
                depth -= 1;
                if depth == 0 {
                    return Some(idx);
                }
            }
            _ => {}
        }
    }
    None
}

fn extract_choices(list_events: &[Event]) -> Option<Vec<Choice>> {
    if list_events.is_empty() {
        return None;
    }
    let mut choices: Vec<Choice> = Vec::new();
    let mut i = 0usize;
    while i < list_events.len() {
        if matches!(list_events[i], Event::StartNode(NodeKind::ListItem)) {
            let end = matching_end(list_events, i)?;
            let item_events = list_events[i + 1..end].to_vec();
            let (correct, cleaned) = strip_checkbox_marker(item_events);
            let html = render_fragment(&cleaned).trim().to_string();
            choices.push(Choice {
                content: html,
                correct,
            });
            i = end + 1;
            continue;
        }
        i += 1;
    }
    Some(choices)
}

fn strip_checkbox_marker(mut events: Vec<Event>) -> (bool, Vec<Event>) {
    events = merge_adjacent_text(events);
    let mut correct = false;
    let re = Regex::new(r"^\[(x|X| )\]\s*").ok();
    if let Some(re) = re {
        for ev in events.iter_mut() {
            if let Event::Text(text) = ev {
                let trimmed = text.trim_start();
                if let Some(m) = re.find(trimmed) {
                    correct = trimmed[m.start()..m.end()]
                        .to_ascii_lowercase()
                        .starts_with("[x]");
                    let stripped = trimmed[m.end()..].to_string();
                    *text = stripped;
                    break;
                }
            }
        }
    }
    (correct, events)
}

fn find_feedback(events: &[Event]) -> (Option<(usize, usize)>, Feedback) {
    let mut i = 0usize;
    while i < events.len() {
        if matches!(events[i], Event::StartNode(NodeKind::Blockquote)) {
            if let Some(end) = matching_end(events, i) {
                let slice = &events[i..=end];
                let fb = parse_feedback(slice);
                if fb.correct.is_some() || fb.wrong.is_some() {
                    return (Some((i, end)), fb);
                }
                i = end + 1;
                continue;
            } else {
                break;
            }
        }
        i += 1;
    }
    (None, Feedback::default())
}

fn parse_feedback(events: &[Event]) -> Feedback {
    let mut fb = Feedback::default();
    let text = collect_text(events);
    let trimmed = text.trim();
    if !matches!(
        trimmed.chars().next().map(|c| c.to_ascii_lowercase()),
        Some('x' | 'v')
    ) {
        return fb;
    }
    if let Ok(re) = Regex::new(r"(?i)[xv]\s") {
        let matches: Vec<regex::Match<'_>> = re.find_iter(trimmed).collect();
        for (idx, m) in matches.iter().enumerate() {
            let marker = trimmed[m.start()..m.start() + 1]
                .chars()
                .next()
                .unwrap_or(' ')
                .to_ascii_lowercase();
            let start = m.end();
            let end = matches
                .get(idx + 1)
                .map(|n| n.start())
                .unwrap_or(trimmed.len());
            let body = trimmed[start..end].trim();
            if body.is_empty() {
                continue;
            }
            match marker {
                'x' => fb.wrong = Some(body.to_string()),
                'v' => fb.correct = Some(body.to_string()),
                _ => {}
            }
        }
    }
    fb
}

fn merge_adjacent_text(events: Vec<Event>) -> Vec<Event> {
    let mut out: Vec<Event> = Vec::with_capacity(events.len());
    for ev in events {
        match ev {
            Event::Text(t) => {
                if let Some(Event::Text(prev)) = out.last_mut() {
                    prev.push_str(&t);
                } else {
                    out.push(Event::Text(t));
                }
            }
            other => out.push(other),
        }
    }
    out
}

fn collect_text(events: &[Event]) -> String {
    let mut out = String::new();
    for ev in events {
        match ev {
            Event::Text(t) => out.push_str(t),
            Event::EndNode(NodeKind::Paragraph) => out.push('\n'),
            _ => {}
        }
    }
    out
}

fn render_fragment(events: &[Event]) -> String {
    if events.is_empty() {
        return String::new();
    }
    let mut doc: Vec<Event> = Vec::with_capacity(events.len() + 2);
    doc.push(Event::StartNode(NodeKind::Document));
    doc.extend_from_slice(events);
    doc.push(Event::EndNode(NodeKind::Document));
    pendon_renderer_html::render_html(&doc)
}

pub fn solid_hints() -> SolidRenderHints {
    let mut hints = SolidRenderHints::default();
    let key = ("Quiz".to_string(), Some("Quiz".to_string()));
    hints.templates.push(ComponentTemplate {
        node_type: "Quiz".to_string(),
        node_name: Some("Quiz".to_string()),
        template: "<Quiz choices={{attrs.choices}} feedback={{attrs.feedback}}>{children}</Quiz>"
            .to_string(),
    });
    hints
        .template_imports
        .entry(key)
        .or_default()
        .push(ImportEntry::Structured {
            module: "./Quiz".to_string(),
            default: Some("Quiz".to_string()),
            names: Vec::new(),
        });
    hints
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_checkbox_marker() {
        let events = vec![Event::Text("[x] answer".to_string())];
        let (correct, cleaned) = strip_checkbox_marker(events);
        assert!(correct);
        assert!(matches!(cleaned.first(), Some(Event::Text(t)) if t == "answer"));
    }

    #[test]
    fn parses_feedback_lines() {
        let events = vec![
            Event::StartNode(NodeKind::Blockquote),
            Event::StartNode(NodeKind::Paragraph),
            Event::Text("x wrong v right".to_string()),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Blockquote),
        ];
        let fb = parse_feedback(&events);
        assert_eq!(fb.wrong.as_deref(), Some("wrong"));
        assert_eq!(fb.correct.as_deref(), Some("right"));
    }
}
