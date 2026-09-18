use crate::processor::{attrs, util};
use crate::specs::PluginSpec;
use pendon_core::{Event, NodeKind};
use regex::Match;

pub fn process(events: &[Event], spec: &PluginSpec) -> Vec<Event> {
    let Some(detector) = util::build_start_detector(spec) else {
        return events.to_vec();
    };

    let mut out = Vec::with_capacity(events.len());
    let mut i = 0usize;

    while i < events.len() {
        if matches!(events[i], Event::StartNode(NodeKind::Blockquote)) {
            let start = i;
            let mut depth = 1usize;
            let mut j = i + 1;

            while j < events.len() && depth > 0 {
                match &events[j] {
                    Event::StartNode(NodeKind::Blockquote) => depth += 1,
                    Event::EndNode(NodeKind::Blockquote) => depth -= 1,
                    _ => {}
                }
                j += 1;
            }

            if depth != 0 {
                out.push(events[i].clone());
                i += 1;
                continue;
            }

            let blockquote_inner = &events[start + 1..j - 1];
            if !contains_matching_paragraph(blockquote_inner, &detector) {
                out.extend_from_slice(&events[start..j]);
                i = j;
                continue;
            }

            let mut k = 0usize;
            while k < blockquote_inner.len() {
                if matches!(blockquote_inner[k], Event::StartNode(NodeKind::Paragraph)) {
                    let mut buf = String::new();
                    let mut p_events = Vec::new();
                    k += 1;

                    while k < blockquote_inner.len()
                        && !matches!(blockquote_inner[k], Event::EndNode(NodeKind::Paragraph))
                    {
                        if let Event::Text(t) = &blockquote_inner[k] {
                            buf.push_str(t);
                        }
                        p_events.push(blockquote_inner[k].clone());
                        k += 1;
                    }

                    let has_end = k < blockquote_inner.len()
                        && matches!(blockquote_inner[k], Event::EndNode(NodeKind::Paragraph));
                    if has_end {
                        k += 1;
                    }

                    if let Some(caps) = detector.captures(buf.trim()) {
                        let (attrs, diags) = attrs::collect_attrs(spec, Some(&caps));
                        out.extend(diags);
                        let cleaned = strip_leading_sigil(&p_events, caps.name("type"));
                        util::emit_component(spec, &attrs, Some(&cleaned), &mut out);
                    } else {
                        out.push(Event::StartNode(NodeKind::Blockquote));
                        out.push(Event::StartNode(NodeKind::Paragraph));
                        out.extend(p_events);
                        if has_end {
                            out.push(Event::EndNode(NodeKind::Paragraph));
                        }
                        out.push(Event::EndNode(NodeKind::Blockquote));
                    }
                } else {
                    // CRITICAL FIX: Preserve trapped markers (like :::) instead of eating them
                    out.push(blockquote_inner[k].clone());
                    k += 1;
                }
            }
            i = j;
            continue;
        }

        out.push(events[i].clone());
        i += 1;
    }
    out
}

fn contains_matching_paragraph(events: &[Event], detector: &regex::Regex) -> bool {
    let mut i = 0usize;
    while i < events.len() {
        if matches!(events[i], Event::StartNode(NodeKind::Paragraph)) {
            let mut text = String::new();
            i += 1;
            while i < events.len() && !matches!(events[i], Event::EndNode(NodeKind::Paragraph)) {
                if let Event::Text(value) = &events[i] {
                    text.push_str(value);
                }
                i += 1;
            }
            if detector.is_match(text.trim()) {
                return true;
            }
        }
        i += 1;
    }
    false
}

fn strip_leading_sigil(events: &[Event], sigil: Option<Match<'_>>) -> Vec<Event> {
    let mut out: Vec<Event> = Vec::with_capacity(events.len());
    let mut removed = false;
    let mut trim_next = false;
    let sig = sigil.map(|m| m.as_str().to_string());

    for ev in events {
        match ev {
            Event::Text(text) => {
                if !removed {
                    if let Some(sig) = &sig {
                        let trimmed = text.trim_start();
                        if trimmed.starts_with(sig) {
                            let mut rest = &trimmed[sig.len()..];
                            rest = rest.trim_start();
                            if rest.is_empty() {
                                trim_next = true;
                            } else {
                                out.push(Event::Text(rest.to_string()));
                            }
                            removed = true;
                            continue;
                        }
                    }
                }

                if trim_next {
                    out.push(Event::Text(text.trim_start().to_string()));
                    trim_next = false;
                } else {
                    out.push(Event::Text(text.clone()));
                }
            }
            other => {
                out.push(other.clone());
            }
        }
    }
    out
}
