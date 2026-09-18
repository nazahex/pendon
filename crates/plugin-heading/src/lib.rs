use pendon_core::{Event, NodeKind};
use pendon_renderer_solid::{ComponentTemplate, ImportEntry, SolidRenderHints};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// --- Configuration & Types ---

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NumberStyle {
    NestedNumber,
    Flat,
}

impl Default for NumberStyle {
    fn default() -> Self {
        Self::NestedNumber
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HeadingOptions {
    pub auto_number: bool,
    pub number_style: NumberStyle,
    pub custom_node: Option<HeadingCustomNode>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HeadingCustomNode {
    pub name: String,
    pub template: String,
    #[serde(default)]
    pub imports: Vec<HeadingImport>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadingImport {
    pub module: String,
    pub default: Option<String>,
    #[serde(default)]
    pub names: Vec<String>,
}

// --- Counter Logic ---

struct HeadingCounters {
    counts: Vec<usize>,
}

impl HeadingCounters {
    fn new() -> Self {
        Self { counts: Vec::new() }
    }

    fn increment(&mut self, level: usize) {
        if level == 0 {
            return;
        }
        if self.counts.len() < level {
            self.counts.resize(level, 0);
        } else {
            self.counts.truncate(level);
        }
        if let Some(last) = self.counts.last_mut() {
            *last += 1;
        }
    }

    /// Returns a formatted number string (e.g. "1.2. ") or empty if no significant digits exist.
    fn format(&self, style: &NumberStyle) -> String {
        if self.counts.is_empty() {
            return String::new();
        }

        // Suppress leading zeros from the counter stack
        let significant: Vec<&usize> = self.counts.iter().skip_while(|&&n| n == 0).collect();

        if significant.is_empty() {
            return String::new();
        }

        match style {
            NumberStyle::Flat => format!("{}. ", significant.last().unwrap_or(&&0)),
            NumberStyle::NestedNumber => {
                let nums: Vec<String> = significant.iter().map(|n| n.to_string()).collect();
                format!("{}. ", nums.join("."))
            }
        }
    }
}

// --- Extra Attributes Parsing ---

/// Parses the bracketed identifier/class section and the curly-brace attributes section.
/// Input example: `[foo][.class,.extra]{ qux: "rox", nor: 12 } Foo Bar`
/// Returns: (custom_id, extra_attrs_map, total_bytes_consumed_from_start)
fn parse_heading_extras(raw_text: &str) -> (Option<String>, BTreeMap<String, String>, usize) {
    let chars: Vec<char> = raw_text.chars().collect();
    let mut cursor = 0;
    let mut custom_id: Option<String> = None;
    let mut classes: Vec<String> = Vec::new();
    let mut attrs = BTreeMap::new();

    // Skip leading whitespace
    while cursor < chars.len() && chars[cursor].is_whitespace() {
        cursor += 1;
    }

    // Parse first bracket group: [id]
    if cursor < chars.len() && chars[cursor] == '[' {
        if let Some(close) = find_char(&chars, cursor + 1, ']') {
            let content: String = chars[cursor + 1..close].iter().collect();
            let trimmed = content.trim();
            // Only treat as ID if it doesn't start with '.' (which is a class-only bracket)
            if !trimmed.is_empty() && !trimmed.starts_with('.') {
                custom_id = Some(trimmed.to_string());
            } else if trimmed.starts_with('.') {
                // First bracket is actually a class group
                parse_class_tokens(trimmed, &mut classes);
            }
            cursor = close + 1;
        }
    }

    // Parse subsequent bracket groups: [.class,.extra]
    while cursor < chars.len() {
        // Skip whitespace between bracket groups
        while cursor < chars.len() && chars[cursor].is_whitespace() {
            cursor += 1;
        }
        if cursor >= chars.len() || chars[cursor] != '[' {
            break;
        }
        if let Some(close) = find_char(&chars, cursor + 1, ']') {
            let content: String = chars[cursor + 1..close].iter().collect();
            parse_class_tokens(content.trim(), &mut classes);
            cursor = close + 1;
        } else {
            break;
        }
    }

    // Parse curly brace attributes: { qux: "rox", nor: 12 }
    // Skip whitespace before brace
    while cursor < chars.len() && chars[cursor].is_whitespace() {
        cursor += 1;
    }
    if cursor < chars.len() && chars[cursor] == '{' {
        if let Some(close) = find_matching_brace(&chars, cursor) {
            let content: String = chars[cursor + 1..close].iter().collect();
            attrs = parse_kv_attrs(&content);
            cursor = close + 1;
        }
    }

    // Merge classes into attrs under "class" key
    if !classes.is_empty() {
        attrs.insert("class".to_string(), classes.join(" "));
    }

    (custom_id, attrs, cursor)
}

/// Splits comma-separated class tokens, stripping leading dots.
fn parse_class_tokens(input: &str, out: &mut Vec<String>) {
    for part in input.split(',') {
        let token = part.trim().trim_start_matches('.');
        if !token.is_empty() {
            out.push(token.to_string());
        }
    }
}

/// Parses key: value pairs from inside curly braces.
/// Supports quoted strings and unquoted values (numbers, booleans).
fn parse_kv_attrs(input: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for part in input.split(',') {
        let Some((key, value)) = part.split_once(':') else {
            continue;
        };
        let key = key.trim().to_string();
        let value = value
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .to_string();
        if !key.is_empty() {
            map.insert(key, value);
        }
    }
    map
}

fn find_char(chars: &[char], mut index: usize, wanted: char) -> Option<usize> {
    while index < chars.len() {
        if chars[index] == wanted {
            return Some(index);
        }
        index += 1;
    }
    None
}

fn find_matching_brace(chars: &[char], start: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut i = start;
    while i < chars.len() {
        match chars[i] {
            '{' => depth += 1,
            '}' if depth == 1 => return Some(i),
            '}' => depth -= 1,
            _ => {}
        }
        i += 1;
    }
    None
}

// --- Main Processor ---

pub fn process(events: &[Event], options: &HeadingOptions) -> Vec<Event> {
    let mut out = Vec::with_capacity(events.len());
    let mut counters = HeadingCounters::new();
    let slug_regex = Regex::new(r"[^a-z0-9]+").unwrap();

    let mut i = 0;
    while i < events.len() {
        if matches!(&events[i], Event::StartNode(NodeKind::Heading)) {
            let start_idx = i;
            let mut end_idx = start_idx + 1;
            let mut heading_level: usize = 1;
            let mut raw_text = String::new();
            let mut inner_events = Vec::new();
            let mut text_indices = Vec::new();

            // 1. Collect heading content and extract level attribute
            while end_idx < events.len()
                && !matches!(events[end_idx], Event::EndNode(NodeKind::Heading))
            {
                match &events[end_idx] {
                    Event::Attribute { name, value } if name == "level" => {
                        heading_level = value.parse().unwrap_or(1);
                    }
                    Event::Text(t) => {
                        raw_text.push_str(t);
                        text_indices.push(inner_events.len());
                        inner_events.push(events[end_idx].clone());
                    }
                    ev => inner_events.push(ev.clone()),
                }
                end_idx += 1;
            }

            // 2. Parse all extras: [id][.classes]{attrs} from the beginning of raw text
            let (custom_id, extra_attrs, consumed_len) = parse_heading_extras(&raw_text);

            // 3. Strip the entire extras prefix from Text events
            if consumed_len > 0 {
                let mut remaining = consumed_len;
                for idx in text_indices {
                    if remaining == 0 {
                        break;
                    }
                    if let Event::Text(t) = &inner_events[idx] {
                        if t.len() <= remaining {
                            remaining -= t.len();
                            inner_events[idx] = Event::Text(String::new());
                        } else {
                            inner_events[idx] = Event::Text(t[remaining..].to_string());
                            remaining = 0;
                        }
                    }
                }
                inner_events.retain(|e| !matches!(e, Event::Text(t) if t.is_empty()));
            }

            // 4. Compute numbering for current heading level
            counters.increment(heading_level);
            let num_str = if options.auto_number {
                counters.format(&options.number_style)
            } else {
                String::new()
            };

            // 5. Determine node type: Custom component or standard Heading
            let use_custom = options.custom_node.is_some();
            let node_kind = if use_custom {
                NodeKind::Custom(options.custom_node.as_ref().unwrap().name.clone())
            } else {
                NodeKind::Heading
            };

            // 6. Emit StartNode
            out.push(Event::StartNode(node_kind.clone()));

            // 7. Emit attributes based on rendering mode
            if use_custom {
                // Custom node mode: emit all attributes needed by the Solid template
                let custom = options.custom_node.as_ref().unwrap();
                out.push(Event::Attribute {
                    name: "name".to_string(),
                    value: custom.name.clone(),
                });
                out.push(Event::Attribute {
                    name: "level".to_string(),
                    value: heading_level.to_string(),
                });

                // Emit id: explicit custom_id or auto-generated slug
                if let Some(id) = &custom_id {
                    out.push(Event::Attribute {
                        name: "id".to_string(),
                        value: id.clone(),
                    });
                } else {
                    let clean_text = raw_text[consumed_len..].trim();
                    let slug = slug_regex
                        .replace_all(&clean_text.to_lowercase(), "-")
                        .trim_matches('-')
                        .to_string();
                    if !slug.is_empty() {
                        out.push(Event::Attribute {
                            name: "slug".to_string(),
                            value: slug,
                        });
                    }
                }

                if !num_str.is_empty() {
                    out.push(Event::Attribute {
                        name: "number".to_string(),
                        value: num_str.trim_end().to_string(),
                    });
                }

                // raw_title: heading text without any extras prefix
                let raw_title = raw_text[consumed_len..].trim().to_string();
                out.push(Event::Attribute {
                    name: "raw_title".to_string(),
                    value: raw_title,
                });

                // Emit all extra attributes (class, qux, nor, etc.)
                for (key, value) in &extra_attrs {
                    out.push(Event::Attribute {
                        name: key.clone(),
                        value: value.clone(),
                    });
                }
            } else {
                // Standard mode: emit level, id, and all extra attributes
                out.push(Event::Attribute {
                    name: "level".to_string(),
                    value: heading_level.to_string(),
                });
                if let Some(id) = custom_id {
                    out.push(Event::Attribute {
                        name: "id".to_string(),
                        value: id,
                    });
                }
                for (key, value) in &extra_attrs {
                    out.push(Event::Attribute {
                        name: key.clone(),
                        value: value.clone(),
                    });
                }
            }

            // 8. Prepend number to first text child (standard mode only).
            // In custom node mode, the number is available via {attrs.number}
            // and should NOT be injected into children.
            if !use_custom && !num_str.is_empty() {
                let mut prefixed = false;
                for ev in inner_events.iter_mut() {
                    if let Event::Text(t) = ev {
                        if !prefixed && !t.is_empty() {
                            *ev = Event::Text(format!("{}{}", num_str, t));
                            prefixed = true;
                            break;
                        }
                    }
                }
                if !prefixed {
                    inner_events.insert(0, Event::Text(num_str));
                }
            }

            // 9. Emit inner children and closing EndNode
            out.extend(inner_events);
            out.push(Event::EndNode(node_kind));

            i = end_idx + 1;
            continue;
        }

        out.push(events[i].clone());
        i += 1;
    }

    out
}

// --- Solid Hints Integration ---

/// Builds SolidRenderHints for the custom heading component when configured.
/// Returns None if no custom_node is defined, letting the renderer fall back
/// to its built-in Heading handler.
pub fn solid_hints(options: &HeadingOptions) -> Option<SolidRenderHints> {
    let custom = options.custom_node.as_ref()?;
    let key = (custom.name.clone(), Some(custom.name.clone()));

    let mut hints = SolidRenderHints::default();
    hints.templates.push(ComponentTemplate {
        node_type: custom.name.clone(),
        node_name: Some(custom.name.clone()),
        template: custom.template.clone(),
    });

    let imports: Vec<ImportEntry> = custom
        .imports
        .iter()
        .map(|imp| ImportEntry::Structured {
            module: imp.module.clone(),
            default: imp.default.clone(),
            names: imp.names.clone(),
        })
        .collect();

    if !imports.is_empty() {
        hints.template_imports.insert(key, imports);
    }

    Some(hints)
}
