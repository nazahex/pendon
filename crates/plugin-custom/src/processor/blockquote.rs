// plugin-custom/processor/blockquote.rs
use crate::processor::{attrs, util};
use crate::specs::PluginSpec;
use pendon_core::{Event, NodeKind};
use std::collections::BTreeMap;

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

            // CRITICAL: Safely find the matching EndNode for this blockquote.
            // We must track nesting depth to avoid consuming nested blockquotes
            // or stopping prematurely at an inner closing tag.
            while j < events.len() && depth > 0 {
                match &events[j] {
                    Event::StartNode(NodeKind::Blockquote) => depth += 1,
                    Event::EndNode(NodeKind::Blockquote) => depth -= 1,
                    _ => {}
                }
                j += 1;
            }

            // If we reached EOF without finding a matching close tag,
            // emit the opening tag as-is and move on to prevent data loss.
            if depth != 0 {
                out.push(events[i].clone());
                i += 1;
                continue;
            }

            // Extract only the children of this specific blockquote level.
            // We exclude the outer Start/End Blockquote events themselves.
            let inner_events = &events[start + 1..j - 1];

            // DUAL STREAM ARCHITECTURE: Separate vanilla markdown content
            // from custom component output. This prevents custom components
            // from being accidentally re-wrapped in a redundant <blockquote>.
            let mut vanilla_out = Vec::new();
            let mut custom_out = Vec::new();

            // State tracker for the currently active custom component.
            // When a new marker is found, the previous component is flushed.
            let mut active_component: Option<(PluginSpec, BTreeMap<String, String>)> = None;
            let mut component_children = Vec::new();

            let mut k = 0usize;

            while k < inner_events.len() {
                if matches!(inner_events[k], Event::StartNode(NodeKind::Paragraph)) {
                    let p_start = k;
                    let mut p_end = k + 1;

                    while p_end < inner_events.len() {
                        if matches!(inner_events[p_end], Event::EndNode(NodeKind::Paragraph)) {
                            break;
                        }
                        p_end += 1;
                    }

                    // CRITICAL: Use inclusive end index (p_end + 1) so that
                    // EndNode(Paragraph) is included in the slice. Omitting it
                    // causes unclosed <p> tags and broken JSX tree structure.
                    let end_idx = if p_end < inner_events.len() {
                        p_end + 1
                    } else {
                        p_end
                    };

                    // Reconstruct full paragraph text for regex matching.
                    // The core parser fragments text char-by-char, so we must
                    // concatenate all Text events within this paragraph scope.
                    let mut full_text = String::new();
                    for ev in &inner_events[p_start..end_idx] {
                        if let Event::Text(t) = ev {
                            full_text.push_str(t);
                        }
                    }

                    // CRITICAL: Match regex against ONLY the first line.
                    // Soft breaks inject literal `\n` into paragraph text.
                    // Using `.*$` on multiline text would fail silently,
                    // causing valid markers to be treated as plain text.
                    let first_line = full_text.lines().next().unwrap_or(&full_text);
                    let mut matched = false;
                    let mut new_attrs = BTreeMap::new();
                    let mut new_diags = Vec::new();
                    let mut new_marker = String::new();

                    if let Some(caps) = detector.captures(first_line.trim_start()) {
                        matched = true;
                        let (a, d) = attrs::collect_attrs(spec, Some(&caps));
                        new_attrs = a;
                        new_diags = d;
                        new_marker = caps
                            .name("type")
                            .map(|m| m.as_str().to_string())
                            .unwrap_or_default();
                    }

                    if matched {
                        // Flush any previously active component before starting a new one.
                        // This ensures each marker creates its own discrete component boundary.
                        if let Some((active_spec, active_attrs)) = active_component.take() {
                            util::emit_component(
                                &active_spec,
                                &active_attrs,
                                Some(&component_children),
                                &mut custom_out,
                            );
                            component_children.clear();
                        }

                        custom_out.extend(new_diags);
                        active_component = Some((spec.clone(), new_attrs));

                        // Strip the marker prefix from AST events before absorbing.
                        // Uses a state machine because text is fragmented across
                        // multiple Event::Text nodes (char-by-char emission).
                        let cleaned_p_events =
                            strip_marker_from_events(&inner_events[p_start..end_idx], &new_marker);

                        // FIX: Check if the stripped paragraph text starts with a list marker.
                        // If so, transform it into a list item and merge with any subsequent
                        // list of the same kind to prevent broken JSX tree structure.
                        let mut first_text = String::new();
                        for ev in &cleaned_p_events {
                            if let Event::Text(t) = ev {
                                first_text.push_str(t);
                            }
                        }

                        let mut list_kind = None;
                        let mut marker_width = 0;
                        let mut start_attr = None;

                        let trimmed = first_text.trim_start();
                        if trimmed.starts_with("- ")
                            || trimmed.starts_with("* ")
                            || trimmed.starts_with("+ ")
                        {
                            list_kind = Some(NodeKind::BulletList);
                            marker_width = 2;
                        } else {
                            let mut chars = trimmed.chars();
                            let mut num_str = String::new();
                            while let Some(c) = chars.next() {
                                if c.is_ascii_digit() {
                                    num_str.push(c);
                                } else {
                                    break;
                                }
                            }
                            if !num_str.is_empty() {
                                let consumed = num_str.len();
                                if let Some(delim) = trimmed.chars().nth(consumed) {
                                    if (delim == '.' || delim == ')')
                                        && trimmed.chars().nth(consumed + 1) == Some(' ')
                                    {
                                        list_kind = Some(NodeKind::OrderedList);
                                        marker_width = consumed + 2;
                                        start_attr = num_str.parse::<usize>().ok();
                                    }
                                }
                            }
                        }

                        if let Some(kind) = list_kind {
                            let mut stripped_cleaned_events = Vec::new();
                            // State machine: 0=leading spaces, 1=marker, 2=trailing space, 3=normal
                            let mut state = 0;
                            let mut marker_chars_left = marker_width;

                            for ev in &cleaned_p_events {
                                match ev {
                                    Event::StartNode(NodeKind::Paragraph)
                                    | Event::EndNode(NodeKind::Paragraph) => continue,
                                    Event::Text(t) => {
                                        let mut new_text = String::new();
                                        for c in t.chars() {
                                            if state == 0 {
                                                if c == ' ' {
                                                    continue;
                                                } else {
                                                    state = 1;
                                                }
                                            }

                                            if state == 1 {
                                                marker_chars_left -= 1;
                                                if marker_chars_left == 0 {
                                                    state = 2;
                                                }
                                                continue;
                                            }

                                            if state == 2 {
                                                if c == ' ' {
                                                    state = 3;
                                                    continue;
                                                } else {
                                                    state = 3;
                                                }
                                            }

                                            if state == 3 {
                                                new_text.push(c);
                                            }
                                        }
                                        if !new_text.is_empty() {
                                            stripped_cleaned_events.push(Event::Text(new_text));
                                        }
                                    }
                                    _ => {
                                        stripped_cleaned_events.push(ev.clone());
                                    }
                                }
                            }

                            let mut transformed_events = Vec::new();
                            transformed_events.push(Event::StartNode(kind.clone()));
                            if let (NodeKind::OrderedList, Some(n)) = (&kind, start_attr) {
                                transformed_events.push(Event::Attribute {
                                    name: "start".to_string(),
                                    value: n.to_string(),
                                });
                            }
                            transformed_events.push(Event::StartNode(NodeKind::ListItem));
                            transformed_events.extend(stripped_cleaned_events);
                            transformed_events.push(Event::EndNode(NodeKind::ListItem));

                            let mut next_idx = end_idx;
                            let mut merged = false;
                            if next_idx < inner_events.len() {
                                if let Event::StartNode(next_kind) = &inner_events[next_idx] {
                                    if *next_kind == kind {
                                        let mut m = next_idx + 1;
                                        while m < inner_events.len() {
                                            if let Event::EndNode(end_kind) = &inner_events[m] {
                                                if *end_kind == kind {
                                                    m += 1;
                                                    break;
                                                }
                                            }
                                            transformed_events.push(inner_events[m].clone());
                                            m += 1;
                                        }
                                        transformed_events.push(Event::EndNode(kind.clone()));
                                        next_idx = m;
                                        merged = true;
                                    }
                                }
                            }

                            if !merged {
                                transformed_events.push(Event::EndNode(kind.clone()));
                            }

                            component_children.extend(transformed_events);
                            // FIX: Use continue to skip the unconditional k = end_idx below.
                            // This prevents double-processing of merged list items which caused
                            // duplicate lists inside Hint components.
                            k = next_idx;
                            continue;
                        } else {
                            component_children.extend(cleaned_p_events);
                        }
                    } else {
                        // Non-matching paragraph: absorb into active component if one exists,
                        // otherwise route to vanilla stream for standard blockquote rendering.
                        if active_component.is_some() {
                            component_children.extend_from_slice(&inner_events[p_start..end_idx]);
                        } else {
                            vanilla_out.extend_from_slice(&inner_events[p_start..end_idx]);
                        }
                    }
                    k = end_idx;
                } else {
                    // Non-paragraph block elements (lists, headings, code fences, images).
                    // Absorb directly into active component or vanilla stream as-is.
                    if active_component.is_some() {
                        component_children.push(inner_events[k].clone());
                    } else {
                        vanilla_out.push(inner_events[k].clone());
                    }
                    k += 1;
                }
            }

            // Flush the last active component when blockquote boundary is reached.
            if let Some((active_spec, active_attrs)) = active_component.take() {
                util::emit_component(
                    &active_spec,
                    &active_attrs,
                    Some(&component_children),
                    &mut custom_out,
                );
            }

            // CRITICAL: Only emit <blockquote> wrapper if vanilla content exists.
            // Custom-only blockquotes should render as bare components without
            // a redundant wrapper, keeping the JSX tree clean and semantic.
            if !vanilla_out.is_empty() {
                out.push(Event::StartNode(NodeKind::Blockquote));
                out.extend(vanilla_out);
                out.push(Event::EndNode(NodeKind::Blockquote));
            }

            // Emit custom components directly at the parent level,
            // completely outside any blockquote wrapper.
            out.extend(custom_out);

            i = j;
            continue;
        }

        out.push(events[i].clone());
        i += 1;
    }
    out
}

/// State machine that strips a marker prefix from fragmented AST text events.
///
/// The core markdown parser emits text character-by-character as separate
/// Event::Text nodes. Simple string trimming cannot handle this fragmentation.
/// This function tracks skip state across multiple events to correctly remove:
///   1. Leading whitespace before the marker
///   2. The marker characters themselves (may span multiple Text events)
///   3. Exactly one trailing space after the marker (if present)
fn strip_marker_from_events(events: &[Event], marker_str: &str) -> Vec<Event> {
    let mut cleaned_p_events = Vec::new();
    let mut stripped_marker = false;
    let mut skip_leading_spaces = true;
    let mut chars_to_skip = marker_str.chars().count();
    let mut skip_trailing_space = true;

    for ev in events {
        if !stripped_marker {
            if let Event::Text(text) = ev {
                let mut remaining = text.as_str();

                // Phase 1: Skip leading whitespace before the marker
                while skip_leading_spaces && remaining.starts_with(' ') {
                    remaining = &remaining[1..];
                }
                if !remaining.is_empty() && !remaining.starts_with(' ') {
                    skip_leading_spaces = false;
                }

                // Phase 2: Consume marker characters one by one.
                // Handles cases where "?" and " " are in separate Text events.
                while chars_to_skip > 0 && !remaining.is_empty() {
                    let mut chars = remaining.chars();
                    chars.next();
                    remaining = chars.as_str();
                    chars_to_skip -= 1;
                }

                // Phase 3: Skip exactly one trailing space after marker
                if chars_to_skip == 0 && skip_trailing_space && remaining.starts_with(' ') {
                    let mut chars = remaining.chars();
                    chars.next();
                    remaining = chars.as_str();
                    skip_trailing_space = false;
                }

                // Only emit non-empty text remnants to avoid ghost empty nodes
                if !remaining.is_empty() {
                    cleaned_p_events.push(Event::Text(remaining.to_string()));
                }

                // Transition to passthrough mode once all phases complete
                if chars_to_skip == 0 && !skip_trailing_space {
                    stripped_marker = true;
                }
                continue;
            }
        }
        // Passthrough: all events after marker stripping are emitted unchanged
        cleaned_p_events.push(ev.clone());
    }
    cleaned_p_events
}
