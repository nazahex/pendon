use pendon_core::{Event, NodeKind, Severity};
use pendon_extra::{scan_extras_chars, to_attributes, ExtrasAttr, ExtrasOptions};
use pendon_renderer_solid::{ComponentSet, ComponentTemplate, ImportEntry, SolidRenderHints};
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
    #[serde(default)]
    pub number_style: NumberStyle,
    /// §11 component set of the `heading` layer: typed entries plus at most one
    /// default, selected per instance by the `@@type{…}` marker (rule 3).
    pub custom: ComponentSet<HeadingCustomNode>,
    /// §9.5: when `plugin-section` owns the outline the heading must not emit an
    /// `id` — or its fallback `slug` — because the id transfers to the section.
    #[serde(default)]
    pub section_owns_id: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HeadingCustomNode {
    pub name: String,
    pub template: String,
    #[serde(default)]
    pub imports: Vec<ImportEntry>,
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

/// The heading head: `[slug]`, `("title")` and the §11 `{…}` / `@@type{…}`
/// extras (§7.4).
struct HeadingHead {
    /// Element id. `#id` > `[slug]` > extras slug (§6.2).
    id: Option<String>,
    /// class, head props and resolved extras: `head wins` (§6.2).
    attrs: BTreeMap<String, String>,
    /// Bare flags of the extras head (§6.3).
    flags: Vec<String>,
    /// The `@@type{…}` marker, when present: the §11 routing key (rule 3).
    type_marker: Option<String>,
    /// §13 warnings raised while resolving the head.
    warnings: Vec<String>,
    /// Characters of the input consumed by the head.
    consumed: usize,
}

/// Parses the optional head of a heading.
///
/// §4.1: the head touches the `#` run, so nothing is trimmed from the front.
/// The whitespace that separates the head from the heading text belongs to the
/// head and never becomes part of the title (§7.4).
///
/// Input examples: `[foo]("Title")@@heading{…} Foo` and `@@heading{…} Foo`.
/// The retired pre-§11 forms (`[.class]`, `{key: value}`) are literal text
/// (§14). Returns the id, the attributes, the flags and how much of the input
/// the head consumed.
fn parse_heading_head(raw_text: &str) -> HeadingHead {
    let chars: Vec<char> = raw_text.chars().collect();
    let mut cursor = 0;
    let mut head_id: Option<String> = None;
    let mut attrs = BTreeMap::new();
    let mut warnings: Vec<String> = Vec::new();

    // §7.4: an optional `[slug]` head, adjacent to the `#` run.
    if chars.get(cursor) == Some(&'[') {
        if let Some(close) = find_char(&chars, cursor + 1, ']') {
            let content: String = chars[cursor + 1..close].iter().collect();
            let trimmed = content.trim();
            // A `.`-bracket (or `#`-bracket) is not a slug head: literal text.
            if !trimmed.is_empty() && !trimmed.starts_with('.') && !trimmed.starts_with('#') {
                head_id = Some(trimmed.to_string());
                cursor = close + 1;
            }
        }
    }

    // §7.4: an optional `("title")` head, adjacent to the slug. It is a
    // separate attribute, never the heading text.
    if chars.get(cursor) == Some(&'(') {
        if let Some(close) = find_matching_paren(&chars, cursor) {
            let content: String = chars[cursor + 1..close].iter().collect();
            let title = content.trim().trim_matches('"').trim_matches('\'');
            if !title.is_empty() {
                attrs.insert("title".to_string(), title.to_string());
            }
            cursor = close + 1;
        }
    }

    // §7.4/§11: an adjacent `{…}` / `@@type{…}` head attaches to the heading.
    // The retired `{key: value}` block is literal text (§14).
    let mut flags = Vec::new();
    let mut type_marker = None;
    if let Some((head, next)) = scan_extras_chars(&chars, cursor) {
        cursor = next;
        type_marker = head.type_marker.clone();
        let parsed = to_attributes(&head, &ExtrasOptions::default());
        for warning in &parsed.warnings {
            warnings.push(pendon_extra::warning_message(warning));
        }
        let extras_id = parsed.value("id").map(|value| value.literal());
        let extras_slug = parsed.value("slug").map(|value| value.literal());
        for (key, value) in &parsed.items {
            if key == "id" || key == "slug" {
                continue;
            }
            match value {
                ExtrasAttr::Flag => {
                    if attrs.contains_key(key) {
                        warnings.push(format!(
                            "`{key}` was dropped because the construct head already sets it"
                        ));
                    } else {
                        flags.push(key.clone());
                    }
                }
                ExtrasAttr::Value(value) => {
                    let text = value.literal();
                    // §6.4: `class` accumulates, head classes first.
                    if key == "class" {
                        match attrs.get("class") {
                            Some(head) if !head.is_empty() => {
                                attrs.insert("class".to_string(), format!("{head} {text}"));
                            }
                            _ => {
                                attrs.insert("class".to_string(), text);
                            }
                        }
                        continue;
                    }
                    if attrs.contains_key(key) {
                        warnings.push(format!(
                            "`{key}` was dropped because the construct head already sets it"
                        ));
                        continue;
                    }
                    attrs.insert(key.clone(), text);
                }
            }
        }
        // §6.2: `#id` > `[slug]` > extras slug.
        match extras_id {
            Some(id) => {
                if head_id.is_some() {
                    warnings.push("extras `#id` replaced the `[slug]` head (§6.2)".to_string());
                }
                head_id = Some(id);
            }
            None => {
                if head_id.is_none() {
                    head_id = extras_slug;
                }
            }
        }
    }

    // §7.4: the whitespace that separates the head from the heading text
    // belongs to the head, so it never becomes part of the title.
    while matches!(chars.get(cursor), Some(' ' | '\t')) {
        cursor += 1;
    }

    // §11 rule 3: the marker is the routing key of the instance; it is carried
    // as a `type` attribute so a `{attrs.type}` template can read it back. An
    // explicit `type:` prop keeps its own value.
    if let Some(marker) = &type_marker {
        attrs
            .entry("type".to_string())
            .or_insert_with(|| marker.clone());
    }

    HeadingHead {
        id: head_id,
        attrs,
        flags,
        type_marker,
        warnings,
        consumed: cursor,
    }
}

/// Finds the `)` matching the `(` at `start`, counting nested pairs.
fn find_matching_paren(chars: &[char], start: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut i = start;
    while i < chars.len() {
        match chars[i] {
            '(' => depth += 1,
            ')' if depth == 1 => return Some(i),
            ')' => depth -= 1,
            _ => {}
        }
        i += 1;
    }
    None
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

            // Core preserves the Markdown marker as text; remove it before
            // parsing heading extras and computing the displayed title.
            let marker_len = raw_text
                .chars()
                .take_while(|character| *character == '#')
                .count();
            let marker_len = if marker_len == heading_level {
                marker_len + usize::from(raw_text.as_bytes().get(marker_len) == Some(&b' '))
            } else {
                0
            };
            let head = parse_heading_head(&raw_text[marker_len..]);
            let custom_id = head.id.clone();
            let extra_attrs = head.attrs.clone();
            let consumed_len = marker_len + head.consumed;

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
            // §11 rule 3: the marker picks the component, an unmatched marker
            // falls back to the layer default, no default to `<h*>`.
            let custom = options.custom.select(head.type_marker.as_deref());
            let use_custom = custom.is_some();
            let node_kind = custom
                .map(|custom| NodeKind::Custom(custom.name.clone()))
                .unwrap_or(NodeKind::Heading);

            // 6. Emit StartNode and the §13 warnings the head resolved.
            out.push(Event::StartNode(node_kind.clone()));
            for warning in &head.warnings {
                out.push(Event::Diagnostic {
                    severity: Severity::Warning,
                    message: format!("[heading] {warning}"),
                    span: None,
                });
            }

            // 7. Emit attributes based on rendering mode
            if use_custom {
                // Custom node mode: emit all attributes needed by the Solid template
                let custom = custom.expect("custom component");
                out.push(Event::Attribute {
                    name: "name".to_string(),
                    value: custom.name.clone(),
                });
                out.push(Event::Attribute {
                    name: "level".to_string(),
                    value: heading_level.to_string(),
                });

                // Emit id: explicit custom_id or auto-generated slug. §9.5: when
                // the section owns the ids, neither is emitted.
                if !options.section_owns_id {
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
                    if !options.section_owns_id {
                        out.push(Event::Attribute {
                            name: "id".to_string(),
                            value: id,
                        });
                    }
                }
                for (key, value) in &extra_attrs {
                    out.push(Event::Attribute {
                        name: key.clone(),
                        value: value.clone(),
                    });
                }
            }

            // §6.3: a bare extras flag stays a bare attribute, in both modes.
            for name in &head.flags {
                out.push(Event::AttributeFlag { name: name.clone() });
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

/// Builds SolidRenderHints for the custom heading components when configured.
/// Returns None if no component is defined, letting the renderer fall back to
/// its built-in Heading handler. Every entry of the set gets a template, typed
/// or not (§11 rule 3).
pub fn solid_hints(options: &HeadingOptions) -> Option<SolidRenderHints> {
    if options.custom.is_empty() {
        return None;
    }

    let mut hints = SolidRenderHints::default();
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

#[cfg(test)]
mod tests {
    use super::*;

    fn consumed_len(text: &str, head: &HeadingHead) -> usize {
        assert!(head.consumed <= text.chars().count());
        head.consumed
    }

    #[test]
    fn parses_the_slug_head_adjacent_to_the_marker() {
        let text = "[foo-bar] Foo";
        let head = parse_heading_head(text);
        assert_eq!(head.id.as_deref(), Some("foo-bar"));
        assert_eq!(consumed_len(text, &head), "[foo-bar] ".len());
        assert!(head.warnings.is_empty(), "{:?}", head.warnings);
    }

    /// §14: the retired `[.class]` / `{key: value}` forms are literal text.
    #[test]
    fn the_retired_legacy_forms_are_literal_text() {
        let text = "[foo-bar][.extra]{ qux: \"anu\" } Foo";
        let head = parse_heading_head(text);
        assert_eq!(head.id.as_deref(), Some("foo-bar"));
        assert_eq!(head.attrs.get("class"), None);
        assert_eq!(head.attrs.get("qux"), None);
        assert!(head.warnings.is_empty(), "{:?}", head.warnings);
        assert_eq!(consumed_len(text, &head), "[foo-bar]".len());

        // A `.`-bracket alone is not a head either.
        let head = parse_heading_head("[.extra] Foo");
        assert_eq!(head.id, None);
        assert_eq!(head.consumed, 0);
    }

    /// §4.1: `@@type {…}` is not a head, and neither is a spaced head.
    #[test]
    fn a_spaced_head_is_literal_text() {
        let text = "@@heading {.c} Title";
        let head = parse_heading_head(text);
        assert!(head.attrs.is_empty());
        assert_eq!(head.flags, Vec::<String>::new());
        // Only the separator space is consumed.
        assert_eq!(consumed_len(text, &head), 0);
    }

    #[test]
    fn parses_the_title_head_apart_from_the_text() {
        let text = "[foo](\"Boom\") Foo Bar";
        let head = parse_heading_head(text);
        assert_eq!(head.id.as_deref(), Some("foo"));
        assert_eq!(head.attrs.get("title").map(String::as_str), Some("Boom"));
        assert_eq!(consumed_len(text, &head), "[foo](\"Boom\") ".len());
        assert!(head.warnings.is_empty(), "{:?}", head.warnings);
    }

    #[test]
    fn extras_attach_to_the_heading_element() {
        let text = "@@heading{.c,isFoo} Title";
        let head = parse_heading_head(text);
        assert_eq!(head.id, None);
        assert_eq!(head.attrs.get("class").map(String::as_str), Some("c"));
        assert_eq!(head.flags, vec!["isFoo".to_string()]);
        assert_eq!(consumed_len(text, &head), "@@heading{.c,isFoo} ".len());
    }

    #[test]
    fn a_bare_head_attaches_like_the_prefixed_one() {
        let head = parse_heading_head("[slug]{.extra}");
        assert_eq!(head.id.as_deref(), Some("slug"));
        assert_eq!(head.attrs.get("class").map(String::as_str), Some("extra"));
    }

    #[test]
    fn id_precedence_is_hash_id_then_slug_then_extras_slug() {
        let head = parse_heading_head("[slug]@@heading{`extras-slug`}");
        assert_eq!(head.id.as_deref(), Some("slug"));

        let head = parse_heading_head("[slug]@@heading{#explicit}");
        assert_eq!(head.id.as_deref(), Some("explicit"));
        assert!(head.warnings.iter().any(|w| w.contains("#id")));

        let head = parse_heading_head("@@heading{`extras-slug`}");
        assert_eq!(head.id.as_deref(), Some("extras-slug"));
    }

    #[test]
    fn head_title_wins_over_extras_title() {
        let head = parse_heading_head("[slug](\"Head\")@@heading{\"Extras\"}");
        assert_eq!(head.attrs.get("title").map(String::as_str), Some("Head"));
        assert!(head.warnings.iter().any(|w| w.contains("`title`")));
    }

    #[test]
    fn malformed_extras_stay_literal_text() {
        let text = "@@heading{`unterminated} Foo";
        let head = parse_heading_head(text);
        assert_eq!(head.id, None);
        assert!(head.attrs.is_empty());
        assert_eq!(head.consumed, 0);
    }

    #[test]
    fn process_emits_extras_attributes_and_flags() {
        let events = vec![
            Event::StartNode(NodeKind::Heading),
            Event::Attribute {
                name: "level".to_string(),
                value: "2".to_string(),
            },
            Event::Text("##[foo]@@heading{.c,isFoo} Title".to_string()),
            Event::EndNode(NodeKind::Heading),
        ];
        let out = process(&events, &HeadingOptions::default());
        assert!(out.iter().any(
            |e| matches!(e, Event::Attribute { name, value } if name == "id" && value == "foo")
        ));
        assert!(out.iter().any(
            |e| matches!(e, Event::Attribute { name, value } if name == "class" && value == "c")
        ));
        assert!(out
            .iter()
            .any(|e| matches!(e, Event::AttributeFlag { name } if name == "isFoo")));
        assert!(out
            .iter()
            .any(|e| matches!(e, Event::Text(text) if text == "Title")));
        assert!(!out
            .iter()
            .any(|e| matches!(e, Event::Text(text) if text.contains("@@heading"))));
    }

    #[test]
    fn process_keeps_the_head_title_off_the_text() {
        let events = vec![
            Event::StartNode(NodeKind::Heading),
            Event::Attribute {
                name: "level".to_string(),
                value: "3".to_string(),
            },
            Event::Text("###(\"Boom\") Body".to_string()),
            Event::EndNode(NodeKind::Heading),
        ];
        let out = process(&events, &HeadingOptions::default());
        assert!(out.iter().any(
            |e| matches!(e, Event::Attribute { name, value } if name == "title" && value == "Boom")
        ));
        assert!(out
            .iter()
            .any(|e| matches!(e, Event::Text(text) if text == "Body")));
    }
}
