//! `plugin-number` — one numbering pass for every element (ADR-0010).
//!
//! Numbering used to be a per-plugin feature: `plugin-heading` counted headings
//! with its own counter and style table. This plugin owns **all** numbering:
//!
//! * `[[task.<element>.number]]` blocks select elements of one element plugin
//!   with ADR-0008's `type` selectors and give them a counter and a style;
//! * `[[task.number.custom]]` blocks are keyed by the `%%numberType` marker,
//!   whose grammar lives in `crates/extra` (§5) and arrives here as the internal
//!   `__number` attribute (`{"depth":1,"type":"numberB"}`, depth already
//!   normalized to `%`-count − 1);
//! * the result is written as `attrs.number` — an array of exactly `depth`
//!   formatted strings (`["1","2","a"]`). The plugin never renders the number
//!   itself: a display format is the client's business (ADR-0010).
//!
//! Two passes are mandatory. The group minimum ("the smallest depth is
//! determined across the whole file") means a `##` written at the end of the
//! document renumbers the `###` sections before it, so pass 1 collects every
//! member and its minimum depth, and only pass 2 assigns numbers.

use std::collections::BTreeMap;

use pendon_core::{Event, NodeKind, Severity, JSON_ATTR_PREFIX};
use pendon_types::{Group, Rule, Selector, TypeId, TypeSet};

/// The internal attribute `crates/extra` re-emits for `%%numberType` (§5, D-9).
pub const NUMBER_MARK_ATTR: &str = "__number";

/// The attribute this plugin writes (`["1","2","a"]`, ADR-0010 rule 5).
pub const NUMBER_ATTR: &str = "number";

/// Tier 1 number systems (D-8). Every other CSS `list-style-type` name falls
/// back to `decimal`, as does a value a system cannot represent.
pub const NUMBER_SYSTEMS: [&str; 8] = [
    "decimal",
    "lower-roman",
    "upper-roman",
    "lower-alpha",
    "upper-alpha",
    "lower-latin",
    "upper-latin",
    "lower-greek",
];

/// `style` of one numbering block (§7.2 of the workflow).
///
/// The TOML shape is either one string or an array, and the two mean different
/// things: a string applies to **every** depth, an array entry _n−1_ applies to
/// the _n_-th visible depth — counted from the group minimum, so a group that
/// starts at `###` still reads `style[0]` first. A depth the array does not
/// reach is `decimal`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NumberStyles {
    /// `style = "upper-alpha"`
    One(String),
    /// `style = ["upper-roman", "lower-alpha"]`
    PerDepth(Vec<String>),
}

impl Default for NumberStyles {
    fn default() -> Self {
        Self::One("decimal".to_string())
    }
}

impl NumberStyles {
    /// The system of visible `depth` (1 = the group's smallest depth): the
    /// string itself, or entry `depth − 1` (`decimal` when the array is
    /// shorter).
    pub fn at(&self, depth: u8) -> &str {
        match self {
            Self::One(system) => system,
            Self::PerDepth(systems) => {
                let index = usize::from(depth).saturating_sub(1);
                systems.get(index).map(String::as_str).unwrap_or("decimal")
            }
        }
    }
}

/// One config block: `[[task.<element>.number]]` or `[[task.number.custom]]`.
///
/// A block **is** a counter: two blocks over the same element number
/// independently, and `pendon-types` decides which one owns an element
/// (`Group::winner`, specificity 2 for a literal / `?`, 1 for `_` / `*`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NumberBlock {
    /// The element plugin token (`heading`, `anchor`, `image`, …). `None` for a
    /// task-level `[[task.number.custom]]` block, whose selectors name
    /// `numberType`s rather than element types.
    pub element: Option<String>,
    /// `type = …` selectors. Empty means "every element of this element"
    /// (equivalent to `*`), which is what a plain `[[task.element.number]]`
    /// without a `type` key means.
    pub selectors: Vec<Selector>,
    /// The style of each depth (§7.2).
    pub styles: NumberStyles,
}

impl NumberBlock {
    /// A block for `element` (`None` = task-level `%%` group).
    pub fn new(element: Option<&str>, selectors: Vec<Selector>, styles: NumberStyles) -> Self {
        Self {
            element: element.map(str::to_string),
            selectors,
            styles,
        }
    }
}

/// Every numbering block of one task, in config order (registration order is
/// the `pendon-types` tie-break).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NumberOptions {
    pub blocks: Vec<NumberBlock>,
}

impl NumberOptions {
    /// `true` when nothing is configured, i.e. the pass is a no-op.
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }
}

// ---------------------------------------------------------------- number systems

/// Formats `value` in `system`, falling back to `decimal` for an unknown system
/// and for a value the system cannot represent (roman past 3999, greek past 24).
pub fn format_number(system: &str, value: u64) -> String {
    match system {
        "lower-roman" => roman(value)
            .map(|text| text.to_lowercase())
            .unwrap_or_else(|| value.to_string()),
        "upper-roman" => roman(value).unwrap_or_else(|| value.to_string()),
        "lower-alpha" | "lower-latin" => alphabetic(value, false),
        "upper-alpha" | "upper-latin" => alphabetic(value, true),
        "lower-greek" => greek(value),
        _ => value.to_string(),
    }
}

/// `1..=3999` as Roman numerals; `0`, and anything above 3999, is `None` so the
/// caller falls back to `decimal` (there is no zero numeral and no letter for
/// 5000 in the classic set).
fn roman(value: u64) -> Option<String> {
    if value == 0 || value > 3999 {
        return None;
    }
    const UNITS: [(u64, &str); 13] = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut remaining = value;
    let mut out = String::new();
    for (amount, letters) in UNITS {
        while remaining >= amount {
            out.push_str(letters);
            remaining -= amount;
        }
    }
    Some(out)
}

/// The CSS alphabetic system: `a…z`, `aa`, `bb`, … (bijective base 26, so 27 is
/// `aa`, exactly like `list-style-type: lower-alpha`).
fn alphabetic(value: u64, upper: bool) -> String {
    if value == 0 {
        return "0".to_string();
    }
    let base = u64::from(b'z' - b'a') + 1;
    let mut remaining = value;
    let mut digits = Vec::new();
    while remaining > 0 {
        remaining -= 1;
        let offset = (remaining % base) as u8;
        digits.push(if upper { b'A' + offset } else { b'a' + offset });
        remaining /= base;
    }
    digits.reverse();
    String::from_utf8(digits).unwrap_or_else(|_| value.to_string())
}

/// The 24 letters of `lower-greek`; a larger value falls back to `decimal`.
fn greek(value: u64) -> String {
    const LETTERS: [char; 24] = [
        'α', 'β', 'γ', 'δ', 'ε', 'ζ', 'η', 'θ', 'ι', 'κ', 'λ', 'μ', 'ν', 'ξ', 'ο', 'π', 'ρ', 'σ',
        'τ', 'υ', 'φ', 'χ', 'ψ', 'ω',
    ];
    match usize::try_from(value)
        .ok()
        .and_then(|value| value.checked_sub(1))
    {
        Some(index) if index < LETTERS.len() => LETTERS[index].to_string(),
        _ => value.to_string(),
    }
}

/// The style of visible `depth` for `styles` (a string applies to every depth,
/// an array is indexed by `depth − 1`, `decimal` beyond its end).
fn style_at(styles: &NumberStyles, depth: u8) -> &str {
    styles.at(depth)
}

// ---------------------------------------------------------------- element mapping

/// The element plugin a node belongs to, from the `__layer` tag every element
/// plugin emits (§11 rule 3), with a `NodeKind` fallback for the standard
/// (non-component) heading path.
fn element_token(kind: &NodeKind, attrs: &BTreeMap<String, String>) -> Option<&'static str> {
    if let Some(layer) = attrs.get("__layer").map(String::as_str) {
        return Some(match layer {
            "heading" => "heading",
            "anchor" => "anchor",
            "img" => "image",
            "table" => "table",
            "marker" => "marker",
            "blockquote" => "blockquote",
            "directive" => "directive",
            "list" | "unordered" | "ordered" => "list",
            "cite" => "cite",
            "section" => "section",
            _ => return None,
        });
    }
    matches!(kind, NodeKind::Heading).then_some("heading")
}

/// A selector with its literal lowercased; wildcards are unchanged.
fn lower_selector(selector: &Selector) -> Selector {
    match selector {
        Selector::Exact(name) => Selector::Exact(name.to_lowercase()),
        other => other.clone(),
    }
}

/// The depth an element has without a `%%` marker: a heading's own `depth`
/// (§7.4), 1 for everything else (§7.2: "elements without a natural depth are
/// depth 1").
fn natural_depth(kind: &NodeKind, attrs: &BTreeMap<String, String>) -> u8 {
    if !matches!(kind, NodeKind::Heading) {
        return 1;
    }
    attrs
        .get("depth")
        .and_then(|depth| depth.parse::<u8>().ok())
        .map(|depth| depth.clamp(1, 6))
        .unwrap_or(1)
}

/// `(depth, number_type)` of a `__number` attribute, when it parses.
fn number_mark(value: &str) -> Option<(u8, String)> {
    let parsed: serde_json::Value = serde_json::from_str(value).ok()?;
    let depth = u8::try_from(parsed.get("depth")?.as_u64()?).ok()?;
    let number_type = parsed.get("type")?.as_str()?.to_string();
    (depth > 0).then_some((depth, number_type))
}

/// `(depth, number_type)` of a node's `__number` attribute.
fn number_mark_attr(attrs: &BTreeMap<String, String>) -> Option<(u8, String)> {
    number_mark(attrs.get(NUMBER_MARK_ATTR)?)
}

/// One node's attribute block, collected in a single forward scan.
struct Block {
    /// Index of the `StartNode` event.
    start: usize,
    /// Index of the first event after the attribute block: where a `number`
    /// attribute is inserted (after the producer's own attributes, §6.4).
    insert_at: usize,
    attrs: BTreeMap<String, String>,
    /// Index of an authored `number` attribute (`{number: "x"}`), replaced by
    /// the computed one with a D-10 warning.
    authored_number: Option<usize>,
}

/// Collects every node with an attribute block, keyed by its `StartNode` index
/// (so the map iterates in document order).
fn attribute_blocks(events: &[Event]) -> BTreeMap<usize, Block> {
    let mut blocks = BTreeMap::new();
    let mut index = 0usize;
    while index < events.len() {
        if !matches!(events[index], Event::StartNode(_)) {
            index += 1;
            continue;
        }
        let start = index;
        index += 1;
        let mut attrs = BTreeMap::new();
        let mut authored_number = None;
        while let Some(event) = events.get(index) {
            match event {
                Event::Attribute { name, value } => {
                    if name == NUMBER_ATTR {
                        authored_number = Some(index);
                    }
                    attrs.insert(name.clone(), value.clone());
                    index += 1;
                }
                Event::AttributeFlag { name } => {
                    attrs.insert(name.clone(), String::new());
                    index += 1;
                }
                Event::Diagnostic { .. } => index += 1,
                _ => break,
            }
        }
        blocks.insert(
            start,
            Block {
                start,
                insert_at: index,
                attrs,
                authored_number,
            },
        );
    }
    blocks
}

/// The `NodeKind` of the node whose `StartNode` sits at `index` (read back from
/// the borrowed event list, so [`Block`] stays small).
fn node_kind(events: &[Event], index: usize) -> Option<NodeKind> {
    match events.get(index) {
        Some(Event::StartNode(kind)) => Some(kind.clone()),
        _ => None,
    }
}

/// One element that receives a number.
struct Member {
    start: usize,
    insert_at: usize,
    /// Which counter numbers it: the index of the winning block.
    block: usize,
    depth: u8,
    authored_number: Option<usize>,
}

/// Counter state of one block: one slot per visible depth, `levels[0]` being
/// the block's minimum depth.
#[derive(Default)]
struct Counter {
    levels: Vec<u64>,
}

impl Counter {
    /// Increments depth `depth` (absolute), resets every deeper level and
    /// ghost-fills a missing parent with the style's first value (1, which every
    /// tier-1 system spells as its first symbol: `1`, `I`, `i`, `A`, `a`, `α`).
    fn increment(&mut self, base: u8, depth: u8) -> &[u64] {
        let index = usize::from(depth.saturating_sub(base));
        if self.levels.len() <= index {
            self.levels.resize(index + 1, 0);
        }
        self.levels.truncate(index + 1);
        self.levels[index] += 1;
        for slot in self.levels.iter_mut().take(index + 1) {
            if *slot == 0 {
                *slot = 1;
            }
        }
        &self.levels
    }
}
/// Assigns `attrs.number` to every numbered element of `events`.
///
/// The pass runs after the element plugins that emit attributes and before
/// `markdown` re-lexes the remaining text; a misordered chain is a CLI
/// diagnostic, never a silent difference.
pub fn process(events: &[Event], options: &NumberOptions) -> Vec<Event> {
    if options.is_empty() {
        return events.to_vec();
    }

    let blocks = attribute_blocks(events);

    // The occurring types of the document in the `pendon-types` sense: every
    // typed numberable node, plus "untyped" when one exists.
    let mut types = TypeSet::new();
    for block in blocks.values() {
        let Some(kind) = node_kind(events, block.start) else {
            continue;
        };
        if element_token(&kind, &block.attrs).is_none() && number_mark_attr(&block.attrs).is_none()
        {
            continue;
        }
        match block.attrs.get("type") {
            Some(name) => types.insert(name.clone()),
            None => types.insert_untyped(),
        }
    }

    // One definition group per element plugin (§2.2 of `type-resolution.md`),
    // plus the task-level group the `%%` markers resolve against. Registration
    // order is config order, which is also the equal-specificity tie-break.
    let mut rules_by_element: BTreeMap<&str, Vec<Rule<usize, usize>>> = BTreeMap::new();
    let mut custom_rules: Vec<Rule<usize, usize>> = Vec::new();
    for (index, block) in options.blocks.iter().enumerate() {
        let selectors = if block.selectors.is_empty() {
            vec![Selector::Star]
        } else {
            block.selectors.clone()
        };
        match &block.element {
            Some(element) => rules_by_element
                .entry(element.as_str())
                .or_default()
                .push(Rule::new(selectors, index, index)),
            // §5: a marker is written in lower camel case (`%%numberB`) and
            // names the `NumberB` type, so the task-level group compares its
            // literals case-insensitively. Element groups stay exact.
            None => custom_rules.push(Rule::new(
                selectors.iter().map(lower_selector).collect(),
                index,
                index,
            )),
        }
    }
    let groups: BTreeMap<&str, Group<usize, usize>> = rules_by_element
        .into_iter()
        .map(|(element, rules)| (element, Group::new(rules)))
        .collect();
    let custom = Group::new(custom_rules);

    // Pass 1: who is numbered, at which depth, by which block.
    let mut members: Vec<Member> = Vec::new();
    for block in blocks.values() {
        let Some(kind) = node_kind(events, block.start) else {
            continue;
        };
        if let Some((depth, number_type)) = number_mark_attr(&block.attrs) {
            let number_type = number_type.to_lowercase();
            let local = TypeSet::from_types([number_type.as_str()]);
            if let Some(rule) = custom.winner(&local, TypeId::Literal(&number_type)) {
                members.push(Member {
                    start: block.start,
                    insert_at: block.insert_at,
                    block: rule.payload,
                    depth,
                    authored_number: block.authored_number,
                });
            }
            // A marker with no block still leaves the element's own group: an
            // authored `%%` always wins over auto-numbering (ADR-0010 rule 2).
            continue;
        }
        let Some(element) = element_token(&kind, &block.attrs) else {
            continue;
        };
        let Some(group) = groups.get(element) else {
            continue;
        };
        let id = match block.attrs.get("type") {
            Some(name) => TypeId::Literal(name),
            None => TypeId::Untyped,
        };
        let Some(rule) = group.winner(&types, id) else {
            continue;
        };
        members.push(Member {
            start: block.start,
            insert_at: block.insert_at,
            block: rule.payload,
            depth: natural_depth(&kind, &block.attrs),
            authored_number: block.authored_number,
        });
    }

    // Pass 1b: the group minimum. Only element blocks have one; `%%` is defined
    // on absolute depth, so it always counts from 1 (ghost parents included).
    let mut minima: Vec<Option<u8>> = vec![None; options.blocks.len()];
    for member in &members {
        if options.blocks[member.block].element.is_none() {
            continue;
        }
        let slot = &mut minima[member.block];
        *slot = Some(slot.map_or(member.depth, |current| current.min(member.depth)));
    }

    // Pass 2: assign the numbers, in document order.
    let mut counters: Vec<Counter> = (0..options.blocks.len())
        .map(|_| Counter::default())
        .collect();
    let mut assigned: Vec<(usize, usize, Option<usize>, String)> = Vec::new();
    for member in &members {
        let base = minima[member.block].unwrap_or(1);
        let styles = &options.blocks[member.block].styles;
        let levels = counters[member.block].increment(base, member.depth);
        let parts: Vec<String> = levels
            .iter()
            .enumerate()
            .map(|(index, value)| format_number(style_at(styles, index as u8 + 1), *value))
            .collect();
        let json = serde_json::to_string(&parts).unwrap_or_else(|_| "[]".to_string());
        assigned.push((member.insert_at, member.start, member.authored_number, json));
    }

    // Write the attribute into each node's attribute block, last.
    let mut out = events.to_vec();
    let mut inserts: Vec<(usize, Vec<Event>)> = Vec::new();
    for (insert_at, start, authored_number, json) in assigned {
        let value = format!("{JSON_ATTR_PREFIX}{json}");
        match authored_number {
            Some(index) => {
                // D-10: `number` is reserved while this plugin runs; the
                // authored value is replaced and reported.
                out[index] = Event::Attribute {
                    name: NUMBER_ATTR.to_string(),
                    value,
                };
                inserts.push((
                    start + 1,
                    vec![Event::Diagnostic {
                        severity: Severity::Warning,
                        message: format!(
                            "[number] `{NUMBER_ATTR}` is reserved by plugin-number; the \
                             authored value was replaced"
                        ),
                        span: None,
                    }],
                ));
            }
            None => inserts.push((
                insert_at,
                vec![Event::Attribute {
                    name: NUMBER_ATTR.to_string(),
                    value,
                }],
            )),
        }
    }
    // Apply from the back so an earlier insertion never shifts a later index.
    inserts.sort_by_key(|(index, _)| std::cmp::Reverse(*index));
    for (index, events) in inserts {
        for (offset, event) in events.into_iter().enumerate() {
            out.insert(index + offset, event);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn selectors(tokens: &[&str]) -> Vec<Selector> {
        Selector::parse_all(tokens.iter().copied()).expect("valid selectors")
    }

    fn styles(tokens: &[&str]) -> NumberStyles {
        NumberStyles::PerDepth(tokens.iter().map(|token| token.to_string()).collect())
    }

    fn one(system: &str) -> NumberStyles {
        NumberStyles::One(system.to_string())
    }

    fn heading(
        level: u8,
        type_name: Option<&str>,
        mark: Option<(u8, &str)>,
        text: &str,
    ) -> Vec<Event> {
        let mut events = vec![
            Event::StartNode(NodeKind::Heading),
            Event::Attribute {
                name: "__layer".to_string(),
                value: "heading".to_string(),
            },
            Event::Attribute {
                name: "depth".to_string(),
                value: level.to_string(),
            },
        ];
        if let Some(name) = type_name {
            events.push(Event::Attribute {
                name: "type".to_string(),
                value: name.to_string(),
            });
        }
        if let Some((depth, number_type)) = mark {
            events.push(Event::Attribute {
                name: NUMBER_MARK_ATTR.to_string(),
                value: format!("{{\"depth\":{depth},\"type\":\"{number_type}\"}}"),
            });
        }
        events.push(Event::Text(text.to_string()));
        events.push(Event::EndNode(NodeKind::Heading));
        events
    }

    fn anchor(mark: Option<(u8, &str)>, text: &str) -> Vec<Event> {
        let mut events = vec![
            Event::StartNode(NodeKind::Link),
            Event::Attribute {
                name: "__layer".to_string(),
                value: "anchor".to_string(),
            },
            Event::Attribute {
                name: "href".to_string(),
                value: "/foo".to_string(),
            },
        ];
        if let Some((depth, number_type)) = mark {
            events.push(Event::Attribute {
                name: NUMBER_MARK_ATTR.to_string(),
                value: format!("{{\"depth\":{depth},\"type\":\"{number_type}\"}}"),
            });
        }
        events.push(Event::Text(text.to_string()));
        events.push(Event::EndNode(NodeKind::Link));
        events
    }

    fn document(parts: Vec<Vec<Event>>) -> Vec<Event> {
        let mut events = vec![Event::StartNode(NodeKind::Document)];
        for part in parts {
            events.extend(part);
        }
        events.push(Event::EndNode(NodeKind::Document));
        events
    }

    /// Every `number` attribute, in document order, as its list of entries.
    fn numbers(events: &[Event]) -> Vec<Vec<String>> {
        events
            .iter()
            .filter_map(|event| match event {
                Event::Attribute { name, value } if name == NUMBER_ATTR => {
                    let json = pendon_core::json_attr_payload(value).expect("a JSON attribute");
                    Some(serde_json::from_str(json).expect("an array of strings"))
                }
                _ => None,
            })
            .collect()
    }

    /// The numbers joined the way a client would print them.
    fn joined(events: &[Event]) -> Vec<String> {
        numbers(events)
            .into_iter()
            .map(|parts| parts.join("."))
            .collect()
    }

    /// T3.3: every tier-1 system at the documented sample values, and the
    /// out-of-range fallback (roman past 3999, greek past 24).
    #[test]
    fn every_tier_one_system_formats_its_sample_values() {
        let cases: [(&str, [&str; 6]); 8] = [
            ("decimal", ["1", "4", "9", "27", "3999", "4000"]),
            (
                "lower-roman",
                ["i", "iv", "ix", "xxvii", "mmmcmxcix", "4000"],
            ),
            (
                "upper-roman",
                ["I", "IV", "IX", "XXVII", "MMMCMXCIX", "4000"],
            ),
            ("lower-alpha", ["a", "d", "i", "aa", "ewu", "ewv"]),
            ("upper-alpha", ["A", "D", "I", "AA", "EWU", "EWV"]),
            ("lower-latin", ["a", "d", "i", "aa", "ewu", "ewv"]),
            ("upper-latin", ["A", "D", "I", "AA", "EWU", "EWV"]),
            ("lower-greek", ["α", "δ", "ι", "27", "3999", "4000"]),
        ];
        let samples = [1, 4, 9, 27, 3999, 4000];
        for (system, expected) in cases {
            for (value, expected) in samples.iter().zip(expected) {
                assert_eq!(
                    format_number(system, *value),
                    expected,
                    "{system} at {value}"
                );
            }
        }
        // The tier list and the formatter agree on the names.
        for system in NUMBER_SYSTEMS {
            assert!(!format_number(system, 1).is_empty(), "{system}");
        }
        // An unknown name is `decimal`, never a panic or an empty string.
        assert_eq!(format_number("none", 3), "3");
        assert_eq!(format_number("decimal-leading-zero", 3), "3");
    }

    /// §7.4 P.1: one block over two types, styles per depth, an undefined depth
    /// falling back to decimal, and the other headings untouched.
    #[test]
    fn the_p1_illustration_numbers_the_selected_types_only() {
        let options = NumberOptions {
            blocks: vec![NumberBlock::new(
                Some("heading"),
                selectors(&["HeadingA", "HeadingB"]),
                styles(&["decimal", "decimal", "lower-alpha"]),
            )],
        };
        let events = document(vec![
            heading(1, Some("HeadingA"), None, "1"),
            heading(2, Some("HeadingB"), None, "1.1"),
            heading(2, Some("HeadingA"), None, "1.2"),
            heading(3, Some("HeadingB"), None, "1.2.a"),
            heading(5, Some("HeadingB"), None, "1.2.a.1.1"),
            heading(2, Some("HeadingC"), None, "none"),
            heading(2, None, None, "none"),
            heading(1, None, None, "none"),
            heading(2, Some("HeadingA"), None, "1.3"),
        ]);

        let out = process(&events, &options);
        assert_eq!(
            joined(&out),
            vec!["1", "1.1", "1.2", "1.2.a", "1.2.a.1.1", "1.3"]
        );
    }

    /// §7.4: the group minimum is the smallest depth that occurs anywhere in the
    /// file, not the first one the counter meets.
    #[test]
    fn an_absent_initial_depth_starts_at_the_smallest_member() {
        let options = NumberOptions {
            blocks: vec![NumberBlock::new(
                Some("heading"),
                selectors(&["HeadingA", "HeadingB"]),
                styles(&["decimal", "decimal", "lower-alpha"]),
            )],
        };
        let events = document(vec![
            heading(3, Some("HeadingB"), None, "1"),
            heading(3, Some("HeadingA"), None, "2"),
            heading(4, Some("HeadingB"), None, "2.1"),
            heading(1, None, None, "none"),
            heading(5, Some("HeadingB"), None, "2.1.a"),
            heading(2, Some("HeadingC"), None, "none"),
            heading(2, None, None, "none"),
            heading(3, Some("HeadingA"), None, "3"),
        ]);

        let out = process(&events, &options);
        assert_eq!(joined(&out), vec!["1", "2", "2.1", "2.1.a", "3"]);
    }

    /// §7.4: the smallest depth appearing in the **middle** renumbers the
    /// earlier elements (the two-pass requirement).
    #[test]
    fn a_smallest_depth_in_the_middle_renumbers_earlier_elements() {
        let options = NumberOptions {
            blocks: vec![NumberBlock::new(
                Some("heading"),
                selectors(&["HeadingA", "HeadingB"]),
                styles(&["decimal", "decimal", "lower-alpha"]),
            )],
        };
        let events = document(vec![
            heading(3, Some("HeadingB"), None, "1.1"),
            heading(3, Some("HeadingA"), None, "1.2"),
            heading(2, Some("HeadingB"), None, "2"),
            heading(1, None, None, "none"),
            heading(5, Some("HeadingB"), None, "2.1.a.1"),
            heading(2, Some("HeadingC"), None, "none"),
            heading(2, None, None, "none"),
            heading(3, Some("HeadingA"), None, "2.2"),
        ]);

        let out = process(&events, &options);
        assert_eq!(joined(&out), vec!["1.1", "1.2", "2", "2.1.a.1", "2.2"]);
    }

    /// §7.2: `%%` members leave their element group, number from the task-level
    /// block at their **absolute** depth and never bump the element counter.
    #[test]
    fn the_marker_group_uses_absolute_depth_and_leaves_the_element_group() {
        let options = NumberOptions {
            blocks: vec![
                NumberBlock::new(Some("heading"), selectors(&["HeadingA"]), one("decimal")),
                NumberBlock::new(
                    None,
                    selectors(&["numberB"]),
                    styles(&["upper-roman", "lower-alpha"]),
                ),
            ],
        };
        let events = document(vec![
            heading(1, Some("HeadingA"), None, "1"),
            heading(2, None, Some((2, "numberB")), "I.a"),
            anchor(Some((1, "numberB")), "II"),
            // Still the second `HeadingA`: the marked heading did not count.
            heading(1, Some("HeadingA"), None, "2"),
            // A marker nobody configured neither numbers nor counts.
            anchor(Some((1, "unknown")), "none"),
        ]);

        let out = process(&events, &options);
        assert_eq!(joined(&out), vec!["1", "I.a", "II", "2"]);
    }

    /// §7.2/T3.5: a string style applies to every depth, an array style is
    /// indexed by absolute depth, and a deeper depth falls back to `decimal`.
    #[test]
    fn styles_resolve_per_depth() {
        let options = NumberOptions {
            blocks: vec![NumberBlock::new(
                Some("heading"),
                selectors(&["NumberB"]),
                styles(&["upper-roman", "lower-alpha", "upper-roman", "lower-roman"]),
            )],
        };
        let events = document(vec![
            heading(1, Some("NumberB"), None, "I"),
            heading(2, Some("NumberB"), None, "I.a"),
            heading(3, Some("NumberB"), None, "I.a.I"),
            heading(4, Some("NumberB"), None, "I.a.I.i"),
            heading(5, Some("NumberB"), None, "I.a.I.i.1"),
            heading(6, Some("NumberB"), None, "I.a.I.i.1.1"),
        ]);
        assert_eq!(
            {
                let out = process(&events, &options);
                joined(&out)
            },
            vec!["I", "I.a", "I.a.I", "I.a.I.i", "I.a.I.i.1", "I.a.I.i.1.1"]
        );

        // A single style string applies to every depth.
        let options = NumberOptions {
            blocks: vec![NumberBlock::new(
                Some("heading"),
                selectors(&["HeadingA"]),
                one("upper-alpha"),
            )],
        };
        let events = document(vec![
            heading(1, Some("HeadingA"), None, "A"),
            heading(2, Some("HeadingA"), None, "A.A"),
            heading(3, Some("HeadingA"), None, "A.A.A"),
        ]);
        let out = process(&events, &options);
        assert_eq!(joined(&out), vec!["A", "A.A", "A.A.A"]);
    }

    /// A block with no `type` numbers every element of that element (§7.2), and
    /// an element without a natural depth numbers at depth 1.
    #[test]
    fn a_block_without_selectors_numbers_every_element() {
        let options = NumberOptions {
            blocks: vec![NumberBlock::new(Some("anchor"), Vec::new(), one("decimal"))],
        };
        let mut typed = anchor(None, "one");
        typed.insert(
            2,
            Event::Attribute {
                name: "type".to_string(),
                value: "AnchorA".to_string(),
            },
        );
        let events = document(vec![anchor(None, "one"), typed]);
        let out = process(&events, &options);
        assert_eq!(joined(&out), vec!["1", "2"]);
    }

    /// D-10: an authored `number:` is replaced and reported, never merged.
    #[test]
    fn an_authored_number_is_replaced_with_a_warning() {
        let options = NumberOptions {
            blocks: vec![NumberBlock::new(
                Some("heading"),
                selectors(&["HeadingA"]),
                one("decimal"),
            )],
        };
        let mut events = heading(1, Some("HeadingA"), None, "1");
        events.insert(
            3,
            Event::Attribute {
                name: "number".to_string(),
                value: "mine".to_string(),
            },
        );
        let out = process(&document(vec![events]), &options);
        assert_eq!(joined(&out), vec!["1"]);
        let warnings = out
            .iter()
            .filter(|event| matches!(event, Event::Diagnostic { .. }))
            .count();
        assert_eq!(warnings, 1, "{out:?}");
    }

    /// No blocks: the event stream is untouched (an unused plugin is free).
    #[test]
    fn without_blocks_nothing_changes() {
        let events = document(vec![heading(1, Some("HeadingA"), None, "x")]);
        assert_eq!(process(&events, &NumberOptions::default()), events);
    }

    fn image(mark: Option<(u8, &str)>, text: &str) -> Vec<Event> {
        let mut events = vec![
            Event::StartNode(NodeKind::Image),
            Event::Attribute {
                name: "__layer".to_string(),
                value: "img".to_string(),
            },
            Event::Attribute {
                name: "src".to_string(),
                value: "/foo.png".to_string(),
            },
        ];
        if let Some((depth, number_type)) = mark {
            events.push(Event::Attribute {
                name: NUMBER_MARK_ATTR.to_string(),
                value: format!("{{\"depth\":{depth},\"type\":\"{number_type}\"}}"),
            });
        }
        events.push(Event::Text(text.to_string()));
        events.push(Event::EndNode(NodeKind::Image));
        events
    }

    /// §7.4 P.2 verbatim: four blocks (one element block, three `%%` groups) and
    /// every acceptance value of `docs/issues/plugin-number.md`.
    #[test]
    fn the_p2_illustration_matches_the_proposal() {
        let options = NumberOptions {
            blocks: vec![
                NumberBlock::new(
                    Some("heading"),
                    selectors(&["HeadingA"]),
                    styles(&["upper-alpha", "lower-alpha"]),
                ),
                NumberBlock::new(None, selectors(&["NumberA"]), one("upper-alpha")),
                NumberBlock::new(
                    None,
                    selectors(&["NumberB"]),
                    styles(&["upper-roman", "lower-alpha", "decimal"]),
                ),
                NumberBlock::new(
                    None,
                    selectors(&["NumberC"]),
                    styles(&["upper-alpha", "lower-alpha", "upper-roman", "lower-roman"]),
                ),
            ],
        };
        let events = document(vec![
            heading(2, Some("HeadingA"), Some((2, "NumberB")), "I.a"),
            anchor(Some((1, "NumberB")), "II"),
            anchor(Some((2, "NumberB")), "II.a"),
            anchor(Some((2, "NumberB")), "II.b"),
            anchor(Some((3, "NumberB")), "II.b.1"),
            anchor(Some((5, "NumberB")), "II.b.1.1.1"),
            anchor(Some((2, "NumberB")), "II.c"),
            anchor(Some((1, "NumberB")), "III"),
            anchor(Some((1, "NumberA")), "A"),
            anchor(Some((3, "NumberA")), "A.A.A"),
            anchor(Some((2, "NumberA")), "A.B"),
            image(Some((1, "NumberA")), "B"),
            image(Some((4, "NumberA")), "B.A.A.A"),
            image(Some((6, "NumberC")), "A.a.I.i.1.1"),
            heading(2, None, Some((1, "NumberB")), "IV"),
            heading(2, None, Some((3, "NumberB")), "IV.a.1"),
            heading(2, Some("HeadingA"), None, "A"),
            heading(3, Some("HeadingA"), None, "A.a"),
            heading(2, Some("HeadingA"), None, "B"),
        ]);

        let out = process(&events, &options);
        assert_eq!(
            joined(&out),
            vec![
                "I.a",
                "II",
                "II.a",
                "II.b",
                "II.b.1",
                "II.b.1.1.1",
                "II.c",
                "III",
                "A",
                "A.A.A",
                "A.B",
                "B",
                "B.A.A.A",
                "A.a.I.i.1.1",
                "IV",
                "IV.a.1",
                "A",
                "A.a",
                "B",
            ]
        );
    }
}
