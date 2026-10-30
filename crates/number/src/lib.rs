//! `number`: one numbering pass for every element (ADR-0010).
//!
//! Numbering used to be a per-plugin feature: `plugin-heading` counted headings
//! with its own counter and style table. This crate owns **all** numbering:
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

/// The authored extras key the pass reserves (D-10) and the **namespace** every
/// public field below lives in.
///
/// Nothing is written under this bare name any more (S2-9.2): an authored
/// `number:` is replaced by a warning and the namespace is split into
/// [`NUMBER_RAW_ATTR`], [`NUMBER_DEPTH_ATTR`] and [`NUMBER_TYPE_ATTR`].
pub const NUMBER_ATTR: &str = "number";

/// `number.raw` — the formatted counters, one per visible level (`["1","2","a"]`).
///
/// The array ADR-0010 rule 5 defines, and the reason nothing is joined here: a
/// display format (`1.2.a.`) belongs to the component. Carried through ADR-0005's
/// structured transport, so a renderer hands the component a real array and
/// `extract` emits a real array (`string[]` in `FIELDS-INVENTORY.md` §0).
pub const NUMBER_RAW_ATTR: &str = "number.raw";

/// `number.depth` — the depth the counter assigned this element (OBJ-6).
///
/// A marker's `%`-count − 1, a heading's own `depth`, `1` for anything else: the
/// value the pass **incremented**, copied out, never re-counted from
/// [`NUMBER_RAW_ATTR`] (whose length is the depth minus the group's minimum, so
/// the two are not the same number in a group that starts at `###`). Transported
/// as a JSON scalar, so it reaches a component and an extracted row as a `number`
/// rather than the string the event stream holds.
pub const NUMBER_DEPTH_ATTR: &str = "number.depth";

/// `number.type` — the type the counter was selected by.
///
/// A `%%numberB` marker publishes `numberB` (the authored spelling `__number`
/// carries, §5); an element numbered by `[[task.<element>.number]]` publishes its
/// own `type` (the §11 selector that matched it, which may differ from the marker
/// on the same node); an untyped element publishes nothing, because a missing
/// field is dropped rather than emitted empty.
pub const NUMBER_TYPE_ATTR: &str = "number.type";

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
    /// `field.override.number.*` — the **public names this counter publishes**
    /// (S2-9.3, ADR-0016 rule 2).
    ///
    /// Keyed by the canonical field (§0 of `FIELDS-INVENTORY.md`) and valued by
    /// the configured name, both dotted. The pass applies the rename as it
    /// writes, which is why the override only has to be declared once, on the
    /// counter, instead of in every element's `custom` entry that a `%%` marker
    /// group happens to number. Nothing here is read back: numbering keeps using
    /// the values it computed, so a rename can never move a number (D-4).
    pub overrides: BTreeMap<String, String>,
}

impl NumberBlock {
    /// A block for `element` (`None` = task-level `%%` group).
    pub fn new(element: Option<&str>, selectors: Vec<Selector>, styles: NumberStyles) -> Self {
        Self {
            element: element.map(str::to_string),
            selectors,
            styles,
            overrides: BTreeMap::new(),
        }
    }

    /// The public name this block publishes `source` under — its own name when
    /// the block renames nothing.
    pub fn public_name<'a>(&'a self, source: &'a str) -> &'a str {
        self.overrides
            .get(source)
            .map(String::as_str)
            .unwrap_or(source)
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
    insert_at: usize,
    /// Which counter numbers it: the index of the winning block.
    block: usize,
    depth: u8,
    /// The type the counter was selected by (marker type or the element's own
    /// `type`), published as [`NUMBER_TYPE_ATTR`]; `None` publishes nothing.
    number_type: Option<String>,
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
/// Publishes `number.raw`, `number.depth` and `number.type` on every numbered
/// element of `events` (ADR-0016; `FIELDS-INVENTORY.md` §0).
///
/// The values are the ones the pass already computed — one counter, so a plugin
/// can never observe two different depths for one element (OBJ-6). `__number`
/// stays internal: nesting and matching keep reading it, and no renderer or
/// projection ever sees it (§11 rule 3).
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
            let local_type = number_type.to_lowercase();
            let local = TypeSet::from_types([local_type.as_str()]);
            if let Some(rule) = custom.winner(&local, TypeId::Literal(&local_type)) {
                members.push(Member {
                    insert_at: block.insert_at,
                    block: rule.payload,
                    depth,
                    // §5: the marker keeps the authored spelling (`%%numberB`
                    // publishes `numberB`), which is also what `__number` carries.
                    number_type: Some(number_type),
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
            insert_at: block.insert_at,
            block: rule.payload,
            depth: natural_depth(&kind, &block.attrs),
            // §11: the type that selected this counter is the element's own.
            number_type: block.attrs.get("type").cloned(),
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
    let mut assigned: Vec<(usize, Vec<Event>)> = Vec::new();
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
        let block = &options.blocks[member.block];
        let mut published = vec![
            // ADR-0016 rule 2: the array, structured (ADR-0005 transport).
            public_attr(
                block.public_name(NUMBER_RAW_ATTR),
                format!("{JSON_ATTR_PREFIX}{json}"),
            ),
            // A JSON **scalar**, so `number.depth` is a number in the AST, in a
            // component's props and in an extracted row — not the string every
            // ordinary attribute is (`FIELDS-INVENTORY.md` §0).
            public_attr(
                block.public_name(NUMBER_DEPTH_ATTR),
                format!("{JSON_ATTR_PREFIX}{}", member.depth),
            ),
        ];
        if let Some(number_type) = &member.number_type {
            published.push(public_attr(
                block.public_name(NUMBER_TYPE_ATTR),
                number_type.clone(),
            ));
        }
        // Two fields publishing under one name would make the map's answer
        // depend on attribute order; the loader rejects such a table
        // (`field.override` cannot land on a name the same block publishes).
        debug_assert!(
            {
                let mut names: Vec<&str> = published
                    .iter()
                    .filter_map(|event| match event {
                        Event::Attribute { name, .. } => Some(name.as_str()),
                        _ => None,
                    })
                    .collect();
                names.sort_unstable();
                names.windows(2).all(|pair| pair[0] != pair[1])
            },
            "a numbering block renames two of its fields onto one name: {published:?}"
        );
        assigned.push((member.insert_at, published));
    }

    // Write the attributes into each node's attribute block, last.
    let mut out = events.to_vec();
    let mut inserts = assigned;
    // D-10: `number` is reserved while the pass runs. The authored attribute
    // becomes a **diagnostic** instead of competing with the computed values —
    // the same event at the same index, so no reader's position shifts.
    for member in &members {
        if let Some(index) = member.authored_number {
            out[index] = Event::Diagnostic {
                severity: Severity::Warning,
                message: format!(
                    "[number] `{NUMBER_ATTR}` is reserved by the numbering pass; the \
                     authored value was replaced by `{NUMBER_RAW_ATTR}`"
                ),
                span: None,
            };
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

/// One public numbering field as an `Event::Attribute`.
fn public_attr(name: &str, value: String) -> Event {
    Event::Attribute {
        name: name.to_string(),
        value,
    }
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

    /// Every `number.raw` attribute, in document order, as its list of entries.
    fn numbers(events: &[Event]) -> Vec<Vec<String>> {
        arrays(events, NUMBER_RAW_ATTR)
    }

    /// Every entry list published under `name`, in document order.
    fn arrays(events: &[Event], name: &str) -> Vec<Vec<String>> {
        published(events, name)
            .into_iter()
            .map(|value| serde_json::from_str(&value).expect("an array of strings"))
            .collect()
    }

    /// The numbers joined the way a client would print them.
    fn joined(events: &[Event]) -> Vec<String> {
        numbers(events)
            .into_iter()
            .map(|parts| parts.join("."))
            .collect()
    }

    /// Every value of one public numbering field, in document order, as its
    /// literal text (`number.depth` is a JSON scalar, so it decodes to `"1"`).
    fn published(events: &[Event], name: &str) -> Vec<String> {
        events
            .iter()
            .filter_map(|event| match event {
                Event::Attribute { name: key, value } if key == name => Some(
                    pendon_core::json_attr_payload(value)
                        .map(str::to_string)
                        .unwrap_or_else(|| value.clone()),
                ),
                _ => None,
            })
            .collect()
    }

    /// S2-9.2: every numbered element publishes `number.raw`, `number.depth` and
    /// — when it has a type — `number.type`; the internal `__number` never
    /// appears, and the bare `number` name is never written.
    #[test]
    fn every_numbered_element_publishes_the_public_fields() {
        let options = NumberOptions {
            blocks: vec![
                NumberBlock::new(
                    Some("heading"),
                    selectors(&["HeadingA"]),
                    styles(&["upper-roman", "lower-alpha"]),
                ),
                NumberBlock::new(None, selectors(&["numberB"]), one("decimal")),
            ],
        };
        let events = document(vec![
            heading(1, Some("HeadingA"), None, "I"),
            heading(2, Some("HeadingA"), None, "I.a"),
            // A marker on a *typed* element: `number.type` is the marker's, not
            // the element's (§5), and the depth is the marker's own.
            anchor(Some((2, "numberB")), "1.1"),
        ]);
        let out = process(&events, &options);
        assert_eq!(
            joined(&out),
            vec!["I", "I.a", "1.1"],
            "the numbers themselves are unchanged"
        );
        assert_eq!(published(&out, NUMBER_DEPTH_ATTR), vec!["1", "2", "2"]);
        assert_eq!(
            published(&out, NUMBER_TYPE_ATTR),
            vec!["HeadingA", "HeadingA", "numberB"]
        );
        // The namespace's bare name is gone: nothing is written as `number`.
        assert!(published(&out, NUMBER_ATTR).is_empty());
        // `__number` is still the marker's transport — read, never published.
        assert_eq!(
            out.iter()
                .filter(|event| matches!(
                    event,
                    Event::Attribute { name, .. } if name == NUMBER_MARK_ATTR
                ))
                .count(),
            1
        );
    }

    /// `number.depth` is the depth the counter used, not the length of
    /// `number.raw`: a group whose smallest depth is `3` numbers its first
    /// member at depth 3 while the array holds one entry (§10.5, OBJ-6).
    #[test]
    fn number_depth_is_the_absolute_depth_not_the_array_length() {
        let options = NumberOptions {
            blocks: vec![NumberBlock::new(
                Some("heading"),
                selectors(&["HeadingA"]),
                one("decimal"),
            )],
        };
        let events = document(vec![
            heading(3, Some("HeadingA"), None, "1"),
            heading(4, Some("HeadingA"), None, "1.1"),
        ]);
        let out = process(&events, &options);
        assert_eq!(numbers(&out), vec![vec!["1"], vec!["1", "1"]]);
        assert_eq!(published(&out, NUMBER_DEPTH_ATTR), vec!["3", "4"]);
    }

    /// S2-9.3: a block renames the fields **it** publishes, and the numbers
    /// themselves do not move — the pass never reads a published name back
    /// (D-4), so `field.override.number.*` cannot reorder a document.
    #[test]
    fn a_block_renames_its_own_fields_without_moving_a_number() {
        let plain = document(vec![
            heading(1, Some("HeadingA"), None, "I"),
            heading(2, Some("HeadingA"), None, "I.a"),
            heading(1, Some("HeadingA"), None, "II"),
            anchor(Some((2, "numberB")), "1.1"),
        ]);
        let options = |overrides: &[(&str, &str)]| NumberOptions {
            blocks: vec![
                NumberBlock {
                    overrides: overrides
                        .iter()
                        .map(|(source, target)| (source.to_string(), target.to_string()))
                        .collect(),
                    ..NumberBlock::new(
                        Some("heading"),
                        selectors(&["HeadingA"]),
                        styles(&["upper-roman", "lower-alpha"]),
                    )
                },
                NumberBlock::new(None, selectors(&["numberB"]), one("decimal")),
            ],
        };

        let renamed = process(
            &plain,
            &options(&[
                (NUMBER_RAW_ATTR, "counter"),
                (NUMBER_DEPTH_ATTR, "level"),
                (NUMBER_TYPE_ATTR, "counter_type"),
            ]),
        );
        // The rename applies to the heading block only: the marker's block keeps
        // the canonical names, so one document can publish both spellings.
        assert_eq!(
            arrays(&renamed, "counter"),
            vec![
                vec!["I".to_string()],
                vec!["I".to_string(), "a".to_string()],
                vec!["II".to_string()],
            ]
        );
        assert_eq!(published(&renamed, "level"), vec!["1", "2", "1"]);
        assert_eq!(
            published(&renamed, "counter_type"),
            vec!["HeadingA", "HeadingA", "HeadingA"]
        );
        assert_eq!(arrays(&renamed, NUMBER_RAW_ATTR), vec![vec!["1", "1"]]);
        assert_eq!(published(&renamed, NUMBER_TYPE_ATTR), vec!["numberB"]);

        // The counter itself never moves: the unrenamed run publishes the same
        // three arrays under the canonical names, and `public_name` only renames
        // the write (D-4).
        let mut canonical = arrays(&process(&plain, &options(&[])), NUMBER_RAW_ATTR);
        let marker = canonical.pop().expect("the marker's array");
        assert_eq!(arrays(&renamed, "counter"), canonical);
        assert_eq!(arrays(&renamed, NUMBER_RAW_ATTR), vec![marker]);
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
    /// every acceptance value of `docs/archive/fulfilled/plugin-number.md`.
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
