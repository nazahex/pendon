use pendon_core::{Event, NodeKind};
use pendon_extra::{
    append_class, issue_event, modifier_contributions, parse_extras, parse_slots, to_attributes,
    ElementHead, ExtrasAttr, ExtrasHead, ExtrasIssue, ExtrasMatch, FieldName, ModifierConfig,
    Preset, Slot, SlotContract, SlotKind, SlotMatch, SlotPair, SlotPos, SlotTag, SpecsKey,
};
use pendon_renderer_solid::{layer_templates, ComponentSet, ImportEntry, SolidRenderHints};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// --- Slot head (§7.4, §6.1) ---

/// Every position heading claims: `#[slug "bracket quote" `bracket tick`](paren …)`
/// — a claimed head is read whole, so an unnamed content becomes a bare field
/// (`paren.bare`, …) rather than being dropped (D-7).
const SLOTS: &[SlotPos] = &[
    (Slot::Bracket, SlotKind::Bare),
    (Slot::Bracket, SlotKind::Quote),
    (Slot::Bracket, SlotKind::Tick),
    (Slot::Paren, SlotKind::Bare),
    (Slot::Paren, SlotKind::Quote),
    (Slot::Paren, SlotKind::Tick),
];

/// §7.4's preset assignment: `bracket.bare` → `slug`, `paren.quote` → `title`.
///
/// `depth` and `inner_text` are part of the element's *fields* but not of this
/// table: neither comes from a slot (`depth` is the §7.4 `#` run, `inner_text`
/// the Markdown body), so there is no position to assign.
const PRESET: &[Preset] = &[
    Preset::new(Slot::Bracket, SlotKind::Bare, FieldName::new("slug")),
    Preset::new(Slot::Paren, SlotKind::Quote, FieldName::new("title")),
];

/// §6.1/§6.2: the `specs.*` positions heading reads into a field of its own — the
/// extras tick is the id chain's third link (`#id` > head slug > extras slug) and
/// the extras quote is the fallback of the head's `("title")`. Both count only
/// while the matched entry keeps the position on its default path; a
/// `field.specs.<k>` assignment moves it out of reach (`Specs::element_path`).
const ELEMENT_READS: &[(SpecsKey, &str)] = &[(SpecsKey::Tick, "slug"), (SpecsKey::Quote, "title")];

/// The tag that claims heading's head (D-11), the positions it claims, and their
/// fallback-to-preset assignment (§6.1, §6.6, D-7).
pub const HEAD: ElementHead = ElementHead::new(
    SlotContract {
        accepted: SLOTS,
        tag: SlotTag::Heading,
    },
    PRESET,
);

// --- Configuration & Types ---

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HeadingOptions {
    /// §11 component set of the `heading` layer: typed entries plus at most one
    /// default, resolved for the whole document by the renderer against the
    /// `@@type{…}` markers it occurs (rule 3, ADR-0008 `D-3a`).
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

// --- Extra Attributes Parsing ---

/// The heading head: the `#[slug …](… "title" …)` slots and the §11 `{…}` /
/// `@@type{…}` extras attached to them (§7.4).
#[derive(Debug, Default)]
struct HeadingHead {
    /// `bracket.bare` — or wherever the preset's row moved (§6.1, §6.6). The
    /// heading's public id field; it falls back to a slug of `inner_text`.
    slug: Option<String>,
    /// `paren.quote`: the heading's `title` attribute.
    title: Option<String>,
    /// Every other claimed position, under its (possibly moved) field name: a
    /// position the preset does not name keeps its slot-qualified path
    /// (`bracket.tick`, `paren.bare`, …, D-7).
    values: BTreeMap<String, String>,
    /// `class` / `id` / `style` and the extras head's own fields: head wins
    /// (§6.2, §6.3).
    attrs: BTreeMap<String, String>,
    /// Bare flags of the extras head (§6.3).
    flags: Vec<String>,
    /// §13 diagnostics raised while resolving the head.
    issues: Vec<ExtrasIssue>,
    /// The head claimed the `#` tag, so the depth is the run **minus one**
    /// (§7.4, D-11).
    tagged: bool,
    /// Bytes of the input consumed by the head, counted from the tag.
    consumed: usize,
}

/// What a heading's text starts with (§4.3, §7.4).
#[derive(Debug)]
enum Read {
    /// Nothing to claim: the text stays text.
    Literal,
    /// Head-shaped but illegal: an `Error`, and the text stays literal (§4.3).
    Malformed(String),
    /// A head, and how much of the input it owns.
    Head(Box<HeadingHead>),
}

/// Reads a heading head at the start of `rest` — the `#` tag, the slots (§5.2–
/// §5.3) and the §11 extras head attached to the closing delimiter.
///
/// §7.4 / D-11: the `#` **immediately before `[` is the tag**, and the rest of
/// the run is the depth — so `###[slug]` is a depth-2 heading and `######[]()` a
/// depth-5 one. A run with no bracket claims nothing (`## Title`), which keeps
/// the tag and the depth in one run instead of adding a second sigil.
fn read_head(rest: &str, options: &HeadingOptions) -> Read {
    debug_assert!(rest.starts_with('#'), "the tag is the last `#` of the run");

    let (pair, mut consumed, tagged) = match parse_slots(rest) {
        SlotMatch::Slots { pair, rest: after } if pair.tag == SlotTag::Heading => {
            (Some(pair), rest.len() - after.len(), true)
        }
        // §4.3 / D-2: tagged and slot-shaped but illegal.
        SlotMatch::Malformed { error, .. } => return Read::Malformed(error.to_string()),
        // No bracket after the run: `#` is depth only. An extras head still
        // attaches to the run itself (`###@@type{…} Title`, §4.1).
        _ => (None, 1, false),
    };

    // §4.1: the extras head must touch the closing delimiter (or the run).
    let extras = match parse_extras(&rest[consumed..]) {
        ExtrasMatch::Head { head, rest: after } => {
            consumed = rest.len() - after.len();
            Some(head)
        }
        ExtrasMatch::Absent { .. } | ExtrasMatch::Malformed { .. } => None,
    };

    if pair.is_none() && extras.is_none() {
        return Read::Literal;
    }

    // §6.6 / D-7: the preset is voided by the **matched** entry, never by
    // another entry's `field.*`. No extras head means no `type` marker, which is
    // exactly the entry the renderer will pick (the layer default).
    let marker = extras.as_ref().and_then(|head| head.type_marker.as_deref());
    let matched = options.custom.field_for(marker);

    let mut head = HeadingHead {
        tagged,
        ..HeadingHead::default()
    };
    let mut head_keys: Vec<String> = Vec::new();
    if let Some(pair) = &pair {
        // §6.6: `bracket.bare → slug`, `paren.quote → title`; every other
        // position is a bare field (D-7).
        let fields = HEAD.assign(pair, matched).unwrap_or_else(|error| {
            // The entry's assignments contradict each other (one path is a
            // prefix of another), so they cannot all be published. A config
            // clash is the author's bug: say so and keep the preset.
            head.issues.push(ExtrasIssue::error(format!(
                "the matched entry's `field.*` assignments clash ({error}); the preset is kept"
            )));
            HEAD.assign(pair, None)
                .expect("an untouched preset cannot clash")
        });
        // The names this construct's own head produced; an extras item spelling
        // one loses (§6.2).
        for (path, value) in fields.iter() {
            let name = path.dotted();
            let text = value.literal();
            head_keys.push(name.clone());
            match name.as_str() {
                // §7.4's preset names, and only those: a moved or renamed
                // position (`field.bracket.bare = "label"`) publishes a field of
                // the author's own, which the heading neither reads nor renames
                // (D-4, §6.1).
                "slug" => {
                    let slug = text.trim();
                    if !slug.is_empty() {
                        head.slug = Some(slug.to_string());
                    }
                }
                "title" => {
                    let title = text.trim();
                    if !title.is_empty() {
                        head.title = Some(title.to_string());
                    }
                }
                _ => {
                    head.values.insert(name, text);
                }
            }
        }
    }

    if let Some(extras) = &extras {
        merge_heading_extras(&mut head, extras, options, &head_keys);
    }
    // §5.4 / D-9: the bracket's modifier run contributes last, so every field the
    // head or the extras already produced wins (§6.2).
    if let Some(pair) = &pair {
        merge_modifiers(&mut head, pair, options.custom.modifier_for(marker));
    }
    head.consumed = consumed;
    Read::Head(Box::new(head))
}

/// Merges `pair`'s modifier run into the heading's fields (§5.4, D-9).
///
/// `class` accumulates (§6.4); every other contributed field is dropped with a
/// warning when the element already carries it, and an unusable merge (D-9's
/// `Severity::Error` tier) drops the whole run.
fn merge_modifiers(head: &mut HeadingHead, pair: &SlotPair, config: Option<&ModifierConfig>) {
    let fields = match modifier_contributions(config, pair) {
        Ok(fields) => fields,
        Err(error) => {
            head.issues
                .push(ExtrasIssue::error(error.message().to_string()));
            return;
        }
    };
    for field in fields {
        let carried = head.attrs.contains_key(&field.name)
            || (field.name == "slug" && head.slug.is_some())
            || (field.name == "title" && head.title.is_some());
        if field.is_flag() {
            if carried {
                head.issues.push(ExtrasIssue::modifier_dropped(&field.name));
            } else {
                head.flags.push(field.name);
            }
            continue;
        }
        let text = field.text();
        if field.name == "class" {
            let classes = append_class(head.attrs.get("class").map(String::as_str), &text);
            head.attrs.insert("class".to_string(), classes);
        } else if carried {
            head.issues.push(ExtrasIssue::modifier_dropped(&field.name));
        } else if field.name == "slug" {
            head.slug = Some(text);
        } else if field.name == "title" {
            head.title = Some(text);
        } else {
            head.attrs.insert(field.name, text);
        }
    }
}

/// Merges a §7.4 extras head into the heading's attributes.
///
/// The construct side wins (§6.2): an item that spells a name the head already
/// produced (`slug`, `title`, a slot field) is dropped and reported, `class`
/// accumulates (§6.4) and bare flags stay bare attributes (§6.3). The two
/// positional fallbacks (`ELEMENT_READS`) fill in only what the head left unset,
/// so the extras tick can still become the `slug` (§6.2's id chain).
fn merge_heading_extras(
    head: &mut HeadingHead,
    extras: &ExtrasHead,
    options: &HeadingOptions,
    head_keys: &[String],
) {
    // §6.1/§11 rule 5: the entry answering this marker names the extras specs.
    let specs = options.custom.specs_for(extras.type_marker.as_deref());
    let parsed = to_attributes(extras, &specs);
    head.issues.extend(parsed.issues.iter().cloned());

    // §6.2: an extras item that spells a name the heading's own head produced
    // loses — the head is the construct, the extras only decorate it.
    let mut items = Vec::with_capacity(parsed.items.len());
    for (key, value) in parsed.items.iter() {
        if head_keys.iter().any(|head| head == key) {
            head.issues.push(ExtrasIssue::warning(format!(
                "`{key}` was dropped because the construct head already sets it"
            )));
            continue;
        }
        items.push((key.clone(), value.clone()));
    }

    // §6.3: `class` / `id` / `style` are the universal DOM fields; §6.2 adds the
    // two positions *this* element reads into its own fields — the extras tick is
    // the id chain's third link and the extras quote its `title` fallback. Both
    // only count while the position keeps its default path (§6.1), which is
    // exactly what `dom_fields`'s element table checks.
    for (key, value) in specs.dom_fields(&items, ELEMENT_READS) {
        match value {
            ExtrasAttr::Flag => {
                if head.attrs.contains_key(key) {
                    head.issues.push(ExtrasIssue::warning(format!(
                        "`{key}` was dropped because the construct head already sets it"
                    )));
                } else {
                    head.flags.push(key.to_string());
                }
            }
            ExtrasAttr::Value(value) => {
                let text = value.transport_for(key);
                match key {
                    // §7.4/§6.2: the head's `[slug]` wins over the extras slug,
                    // which is the fallback; either way the field is published
                    // once, as `slug`.
                    "slug" => match &head.slug {
                        Some(_) => head.issues.push(ExtrasIssue::warning(
                            "`slug` was dropped because the construct head already sets it",
                        )),
                        None if !text.trim().is_empty() => {
                            head.slug = Some(text.trim().to_string())
                        }
                        None => {}
                    },
                    // §7.4: `("title")` is the head's; an extras `"title"` only
                    // fills in for it.
                    "title" => match &head.title {
                        Some(_) => head.issues.push(ExtrasIssue::warning(
                            "`title` was dropped because the construct head already sets it",
                        )),
                        None if !text.trim().is_empty() => {
                            head.title = Some(text.trim().to_string())
                        }
                        None => {}
                    },
                    // §6.4: `class` accumulates, head classes first.
                    "class" => match head.attrs.get("class") {
                        Some(head_class) if !head_class.is_empty() => {
                            head.attrs
                                .insert("class".to_string(), format!("{head_class} {text}"));
                        }
                        _ => {
                            head.attrs.insert("class".to_string(), text);
                        }
                    },
                    _ => {
                        if head.attrs.contains_key(key) {
                            head.issues.push(ExtrasIssue::warning(format!(
                                "`{key}` was dropped because the construct head already sets it"
                            )));
                        } else {
                            head.attrs.insert(key.to_string(), text);
                        }
                    }
                }
            }
        }
    }

    // §11 rule 3: the marker is the routing key of the instance; it is carried
    // as a `type` attribute so a `{attrs.type}` template can read it back. An
    // explicit `type:` prop keeps its own value.
    if let Some(marker) = &extras.type_marker {
        head.attrs
            .entry("type".to_string())
            .or_insert_with(|| marker.clone());
    }
}

/// §7.4: the `slug` fallback — `inner_text`, lowercased, with every run of
/// non-alphanumerics collapsed into a `-`.
fn slug_of(inner_text: &str, slug_regex: &Regex) -> Option<String> {
    let slug = slug_regex
        .replace_all(&inner_text.trim().to_lowercase(), "-")
        .trim_matches('-')
        .to_string();
    (!slug.is_empty()).then_some(slug)
}

// --- Main Processor ---

pub fn process(events: &[Event], options: &HeadingOptions) -> Vec<Event> {
    let mut out = Vec::with_capacity(events.len());
    let slug_regex = Regex::new(r"[^a-z0-9]+").unwrap();

    let mut i = 0;
    while i < events.len() {
        if matches!(&events[i], Event::StartNode(NodeKind::Heading)) {
            let start_idx = i;
            let mut end_idx = start_idx + 1;
            let mut run_depth: usize = 1;
            let mut raw_text = String::new();
            let mut inner_events = Vec::new();
            let mut text_indices = Vec::new();

            // 1. Collect the heading's text and the `#` run the core recorded.
            while end_idx < events.len()
                && !matches!(events[end_idx], Event::EndNode(NodeKind::Heading))
            {
                match &events[end_idx] {
                    Event::Attribute { name, value } if name == "depth" => {
                        run_depth = value.parse().unwrap_or(1);
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

            // 2. §7.4 / D-11: the core keeps the `#` run as text, and the **last
            // `#` of the run is the tag** that claims the head — so a heading
            // with a head is one depth shallower than its run (`###[slug]` is
            // depth 2, `#######[]()` depth 6). A run without a bracket keeps its
            // whole length as depth (`## Title`).
            let hashes = raw_text
                .chars()
                .take_while(|character| *character == '#')
                .count();
            let mut head = HeadingHead::default();
            let mut consumed = hashes;
            if hashes > 0 {
                match read_head(&raw_text[hashes - 1..], options) {
                    Read::Head(read) => {
                        consumed = hashes - 1 + read.consumed;
                        head = *read;
                    }
                    // §4.3: a malformed head is literal text with an `Error`.
                    Read::Malformed(message) => head.issues.push(ExtrasIssue::error(message)),
                    Read::Literal => {}
                }
            }
            let depth = if head.tagged {
                // §7.4: a claimed head spends one `#` on the tag, so the depth
                // never falls below 1 (`#[slug]` is still an `<h1>`).
                hashes.saturating_sub(1).max(1)
            } else {
                run_depth.max(hashes)
            };

            // §7.4: the whitespace that separates the head from the heading text
            // belongs to the head, so it never becomes part of `inner_text`.
            while matches!(raw_text.as_bytes().get(consumed), Some(b' ' | b'\t')) {
                consumed += 1;
            }

            // 3. Strip the head from the Text events: it is a head, not content.
            if consumed > 0 {
                let mut remaining = consumed;
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

            // 4. §7.4: the element's fields. `slug` is the bracket, else a slug
            // of `inner_text`; `id` is an extras `#id` (§6.3) and falls back to
            // the slug (§6.2); `depth` is the run without its tag. `inner_text`
            // itself is the node's content, never an attribute (§7.4).
            let inner_text = raw_text[consumed..].to_string();
            let mut attrs = head.attrs.clone();
            for (name, value) in &head.values {
                attrs.insert(name.clone(), value.clone());
            }
            if let Some(title) = &head.title {
                attrs.insert("title".to_string(), title.clone());
            }
            attrs.insert("depth".to_string(), depth.to_string());
            // §9.5: when the section owns the ids, the heading emits neither.
            if !options.section_owns_id {
                if let Some(slug) = head
                    .slug
                    .clone()
                    .or_else(|| slug_of(&inner_text, &slug_regex))
                {
                    attrs
                        .entry("id".to_string())
                        .or_insert_with(|| slug.clone());
                    attrs.insert("slug".to_string(), slug);
                }
            }

            // 5. Emit StartNode, the §11 layer tag, the §13 diagnostics and the
            // fields. There is one emission path: what a Solid template reads is
            // what the AST/JSON renderers carry. Numbering is `number`'s
            // job (ADR-0010); this plugin no longer counts anything.
            out.push(Event::StartNode(NodeKind::Heading));
            // §11 rule 3 / D-3a: every node names the layer that owns it, in both
            // modes, so whole-document consumers (`number`,
            // `plugin-extract`) can tell the element kind from the event stream.
            out.push(Event::Attribute {
                name: "__layer".to_string(),
                value: HEADING_LAYER.to_string(),
            });
            for issue in &head.issues {
                out.push(issue_event("heading", issue));
            }
            for (name, value) in &attrs {
                out.push(Event::Attribute {
                    name: name.clone(),
                    value: value.clone(),
                });
            }
            // §6.3: a bare extras flag stays a bare attribute.
            for name in &head.flags {
                out.push(Event::AttributeFlag { name: name.clone() });
            }

            // 6. Emit inner children and closing EndNode
            out.extend(inner_events);
            out.push(Event::EndNode(NodeKind::Heading));

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

    Some(SolidRenderHints {
        templates: layer_templates(HEADING_LAYER, &options.custom, |custom| {
            (custom.template.clone(), custom.imports.clone())
        }),
        ..SolidRenderHints::default()
    })
}

/// §11: the layer `[[task.heading.custom]]` declares.
const HEADING_LAYER: &str = "heading";

/// The layer key `[[task.heading.custom]]` addresses (always the primary).
pub fn primary_layer() -> &'static str {
    HEADING_LAYER
}

#[cfg(test)]
mod tests {
    use super::*;
    use pendon_extra::{AttrValue, FieldConfig, Path};
    use pendon_renderer_solid::TypedComponent;

    /// §7.4's preset, position by position (S2-2.5). `slug` is the slot value and
    /// `title` the paren's quote; `depth`/`inner_text` are not slot-derived, so
    /// they must not appear here.
    #[test]
    fn the_head_declares_the_spec_preset() {
        assert_eq!(HEAD.contract.tag, SlotTag::Heading);
        assert!(
            HEAD.is_consistent(),
            "a preset must sit on a claimed position"
        );
        let got: Vec<String> = HEAD
            .presets()
            .iter()
            .map(|preset| {
                format!(
                    "{}.{} -> {}",
                    preset.slot.name(),
                    preset.kind.name(),
                    preset.name
                )
            })
            .collect();
        assert_eq!(got, ["bracket.bare -> slug", "paren.quote -> title"]);
        assert!(!got
            .iter()
            .any(|row| row.contains("inner_text") || row.contains("depth")));
        // The bracket quote is claimed but unpreset: it stays `bracket.quote`.
        assert!(HEAD.contract.accepts(Slot::Bracket, SlotKind::Quote));
        assert_eq!(HEAD.preset_for(Slot::Bracket, SlotKind::Quote), None);
    }

    /// §6.2: the extras slug is the id chain's third link (`#id` > head slug >
    /// extras slug), so it becomes the heading's `slug` when no `[slug]` head
    /// exists — and loses, with a warning, when one does.
    #[test]
    fn the_extras_slug_fills_in_for_a_missing_head_slug() {
        let (_, head) = head_of("#@@headingX{`slug-a`, .fancy} A typed heading");
        assert_eq!(head.slug.as_deref(), Some("slug-a"));
        assert_eq!(head.attrs.get("class").map(String::as_str), Some("fancy"));

        let (_, head) = head_of("#[head-slug]@@headingX{`extras-slug`}");
        assert_eq!(head.slug.as_deref(), Some("head-slug"));
        assert!(head.issues.iter().any(|i| i.message.contains("`slug`")));
    }

    /// §6.1: once the matched entry moves the tick position, the item is a field
    /// of the author's own and no longer feeds the slug.
    #[test]
    fn a_moved_tick_position_is_not_the_slug() {
        let mut field = FieldConfig::new();
        field
            .assign_specs(
                "tick",
                Some(Path::from_dotted("meta.slug").expect("a path")),
            )
            .expect("a known specs key");
        let options = HeadingOptions {
            custom: ComponentSet::from_entries([TypedComponent::typed(
                ["headingX"],
                HeadingCustomNode {
                    name: "HeadingX".into(),
                    template: "<HeadingX>{children}</HeadingX>".into(),
                    imports: Vec::new(),
                },
            )
            .with_field(field)]),
            ..HeadingOptions::default()
        };
        let (_, head) = head_of_with("#@@headingX{`moved`} Title", &options);
        assert_eq!(head.slug, None);
        assert_eq!(
            head.attrs.get("meta.slug").map(String::as_str),
            Some("moved")
        );
        assert_eq!(head.attrs.get("specs.tick"), None);
    }

    /// Reads a head that must parse; returns what it consumed and the result.
    fn head_of(text: &str) -> (String, HeadingHead) {
        head_of_with(text, &HeadingOptions::default())
    }

    /// [`head_of`] with an explicit option set.
    fn head_of_with(text: &str, options: &HeadingOptions) -> (String, HeadingHead) {
        match read_head(text, options) {
            Read::Head(head) => (text[..head.consumed].to_string(), *head),
            Read::Malformed(message) => panic!("malformed head: {message}"),
            Read::Literal => panic!("not a head: {text:?}"),
        }
    }

    // --- §5.4 slot modifiers (S2-6.2, D-9) --------------------------------

    /// §5.4 / D-9: the bracket's modifier run contributes its configured fields
    /// to the heading, `class` included, and a `true` contribution is a bare
    /// flag (§6.3). Pendon ships no symbol of its own, so nothing else happens.
    #[test]
    fn a_modifier_run_contributes_configured_fields() {
        let mut modifier = ModifierConfig::new();
        modifier.define(
            '?',
            vec![
                (
                    Path::from_dotted("class").unwrap(),
                    AttrValue::Str("fancy".into()),
                ),
                (
                    Path::from_dotted("outlined").unwrap(),
                    AttrValue::Bool(true),
                ),
            ],
        );
        let options = HeadingOptions {
            custom: ComponentSet::from_entries([TypedComponent::default_component(
                HeadingCustomNode {
                    name: "DocHeading".into(),
                    template: "<DocHeading>{children}</DocHeading>".into(),
                    imports: Vec::new(),
                },
            )
            .with_modifier(modifier)]),
            section_owns_id: false,
        };
        // The run is consumed with the bracket: it is configuration, not slug.
        let (consumed, head) = head_of_with("#[foo |?] Title", &options);
        assert_eq!(consumed, "#[foo |?]");
        assert_eq!(head.slug.as_deref(), Some("foo"));
        assert_eq!(head.attrs.get("class").map(String::as_str), Some("fancy"));
        assert!(
            head.flags.contains(&"outlined".to_string()),
            "{:?}",
            head.flags
        );
        assert!(head.issues.is_empty(), "{:?}", head.issues);
    }

    /// §7.4: the bracket is `slug` and the paren quote `title`; both are consumed,
    /// so neither becomes part of `inner_text`.
    #[test]
    fn parses_the_slug_and_title_head() {
        let (consumed, head) = head_of("#[foo](\"Boom\")");
        assert_eq!(consumed, "#[foo](\"Boom\")");
        assert_eq!(head.slug.as_deref(), Some("foo"));
        assert_eq!(head.title.as_deref(), Some("Boom"));
        assert!(head.issues.is_empty(), "{:?}", head.issues);
    }

    /// D-7: the preset names `bracket.bare` and `paren.quote` only, so a claimed
    /// position it leaves unassigned stays a field of the author's, under its
    /// qualified name.
    #[test]
    fn an_unpreset_position_keeps_its_qualified_name() {
        let (_, head) = head_of("#[foo](/bar)");
        assert_eq!(head.slug.as_deref(), Some("foo"));
        assert_eq!(head.title, None);
        assert_eq!(
            head.values.get("paren.bare").map(String::as_str),
            Some("/bar")
        );
    }

    /// §14: the retired `[.class]` / `{key: value}` forms are literal text, and a
    /// `.`-bracket is not a head either.
    #[test]
    fn the_retired_legacy_forms_are_literal_text() {
        assert!(matches!(
            read_head("#[foo-bar][.extra]", &HeadingOptions::default()),
            Read::Head(_)
        ));
        // `[.extra]` follows the head as text, so only the slug is consumed.
        let (consumed, head) = head_of("#[foo-bar][.extra]{ qux: \"anu\" } Foo");
        assert_eq!(consumed, "#[foo-bar]");
        assert_eq!(head.slug.as_deref(), Some("foo-bar"));
        assert_eq!(head.attrs.get("class"), None);
        assert_eq!(head.attrs.get("qux"), None);
        assert!(head.issues.is_empty(), "{:?}", head.issues);

        // A `.`-bracket alone is a slug, not a class: §14 retired the class
        // bracket, it did not narrow the slug one.
        let (_, head) = head_of("#[.extra] Foo");
        assert_eq!(head.slug.as_deref(), Some(".extra"));
        assert_eq!(head.attrs.get("class"), None);
    }

    /// §4.1: a spaced extras head is not a head.
    #[test]
    fn a_spaced_head_is_literal_text() {
        assert!(matches!(
            read_head("#@@heading {.c}", &HeadingOptions::default()),
            Read::Literal
        ));
        // …while the adjacent one is.
        let (consumed, head) = head_of("#@@heading{.c}");
        assert_eq!(consumed, "#@@heading{.c}");
        assert_eq!(head.attrs.get("class").map(String::as_str), Some("c"));
    }

    #[test]
    fn extras_attach_to_the_heading_element() {
        let (consumed, head) = head_of("#@@heading{.c,isFoo}");
        assert_eq!(consumed, "#@@heading{.c,isFoo}");
        assert_eq!(head.slug, None);
        assert_eq!(head.attrs.get("class").map(String::as_str), Some("c"));
        assert_eq!(head.flags, vec!["isFoo".to_string()]);
    }

    #[test]
    fn a_bare_head_attaches_like_the_tagged_one() {
        let (_, head) = head_of("#[slug]{.extra}");
        assert_eq!(head.slug.as_deref(), Some("slug"));
        assert_eq!(head.attrs.get("class").map(String::as_str), Some("extra"));
    }

    /// §6.2: an extras `#id` beats the head slug, and both are published — the
    /// processor's `id` fallback then keeps the explicit one.
    #[test]
    fn an_extras_id_beats_the_head_slug() {
        let (_, head) = head_of("#[slug]@@heading{#explicit}");
        assert_eq!(head.slug.as_deref(), Some("slug"));
        assert_eq!(head.attrs.get("id").map(String::as_str), Some("explicit"));
        assert!(head.issues.is_empty(), "{:?}", head.issues);
    }

    /// §6.2: the head wins over a same-named extras item, with a warning.
    #[test]
    fn head_title_wins_over_extras_title() {
        let (_, head) = head_of("#[slug](\"Head\")@@heading{title: \"Extras\"}");
        assert_eq!(head.title.as_deref(), Some("Head"));
        assert!(head.issues.iter().any(|i| i.message.contains("`title`")));
    }

    /// §6.4: `class` accumulates inside one head.
    #[test]
    fn class_accumulates() {
        let (_, head) = head_of("#[slug]{.a, .b}");
        assert_eq!(head.attrs.get("class").map(String::as_str), Some("a b"));
    }

    #[test]
    fn malformed_extras_stay_literal_text() {
        let (consumed, head) = head_of("#[foo]@@heading{`unterminated}");
        assert_eq!(consumed, "#[foo]");
        assert_eq!(head.slug.as_deref(), Some("foo"));
        assert!(head.attrs.is_empty());
    }

    /// §4.3: tagged and slot-shaped but illegal is an `Error`, not a silent
    /// fallback.
    #[test]
    fn a_malformed_head_is_an_error() {
        match read_head("#[\"unclosed] Foo", &HeadingOptions::default()) {
            Read::Malformed(_) => {}
            other => panic!("an unterminated quote must be malformed, got {other:?}"),
        }
    }

    /// §7.4 / D-11: the last `#` of the run is the tag, so a claimed head is one
    /// depth shallower than its run; a run with no bracket keeps its whole length.
    #[test]
    fn the_tag_spends_one_hash_of_the_run() {
        for (source, depth) in [
            ("# Title", "1"),
            ("## Title", "2"),
            ("###### Title", "6"),
            ("#[slug] Title", "1"),
            ("##[slug] Title", "1"),
            ("###[slug](\"T\"){.c} Title", "2"),
            ("######[]() Title", "5"),
            // An extras head attaches to the run without claiming a bracket.
            ("###@@t{.c} Title", "3"),
        ] {
            assert_eq!(
                attr(
                    &process(&heading(source), &HeadingOptions::default()),
                    "depth"
                )
                .as_deref(),
                Some(depth),
                "{source}"
            );
        }
    }

    /// §7.4: `inner_text` is the node's text, never an attribute, and `slug`
    /// falls back to a slug of it.
    #[test]
    fn the_slug_falls_back_to_the_heading_text() {
        let out = process(&heading("## A Fine Head!"), &HeadingOptions::default());
        assert_eq!(attr(&out, "slug").as_deref(), Some("a-fine-head"));
        assert_eq!(attr(&out, "id").as_deref(), Some("a-fine-head"));
        assert_eq!(attr(&out, "inner_text"), None);
        assert_eq!(attr(&out, "raw_title"), None);
        assert!(out
            .iter()
            .any(|e| matches!(e, Event::Text(text) if text == "A Fine Head!")));
    }

    /// §9.5: when the section owns the ids, the heading emits neither `id` nor
    /// `slug`.
    #[test]
    fn the_section_owns_the_id() {
        let options = HeadingOptions {
            section_owns_id: true,
            ..HeadingOptions::default()
        };
        let out = process(&heading("##[own] Title"), &options);
        assert_eq!(attr(&out, "id"), None);
        assert_eq!(attr(&out, "slug"), None);
        assert_eq!(attr(&out, "depth").as_deref(), Some("1"));
    }

    #[test]
    fn process_emits_extras_attributes_and_flags() {
        let out = process(
            &heading("##[foo]@@heading{.c,isFoo} Title"),
            &HeadingOptions::default(),
        );
        assert_eq!(attr(&out, "id").as_deref(), Some("foo"));
        assert_eq!(attr(&out, "class").as_deref(), Some("c"));
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
        // §7.4: the paren needs its bracket (`[]`); `("Boom")` alone is text.
        let out = process(&heading("###[](\"Boom\") Body"), &HeadingOptions::default());
        assert_eq!(attr(&out, "title").as_deref(), Some("Boom"));
        assert_eq!(attr(&out, "depth").as_deref(), Some("2"));
        assert_eq!(attr(&out, "inner_text"), None);
        assert!(out
            .iter()
            .any(|e| matches!(e, Event::Text(text) if text == "Body")));
    }

    /// ADR-0010: numbering belongs to `number`. No heading code path may
    /// emit a `number` attribute or inject a number into the heading text any
    /// more (the legacy `auto_number` / `number_style` behaviour is gone).
    #[test]
    fn no_heading_path_produces_a_number() {
        let out = process(&heading("## Title"), &HeadingOptions::default());
        assert!(!out
            .iter()
            .any(|event| matches!(event, Event::Attribute { name, .. } if name == "number")));
        assert!(out
            .iter()
            .any(|event| matches!(event, Event::Text(text) if text == "Title")));
        // …and the §11 layer tag is emitted in standard mode too.
        assert!(out.iter().any(
            |event| matches!(event, Event::Attribute { name, value } if name == "__layer" && value == HEADING_LAYER)
        ));
    }

    /// A heading event stream as `plugin-markdown` writes it: the core's `depth`
    /// attribute plus the `#` run as text.
    fn heading(source: &str) -> Vec<Event> {
        let depth = source.chars().take_while(|c| *c == '#').count();
        vec![
            Event::StartNode(NodeKind::Heading),
            Event::Attribute {
                name: "depth".to_string(),
                value: depth.to_string(),
            },
            Event::Text(source.to_string()),
            Event::EndNode(NodeKind::Heading),
        ]
    }

    /// The first `Attribute` of `name`.
    fn attr(events: &[Event], name: &str) -> Option<String> {
        events.iter().find_map(|event| match event {
            Event::Attribute { name: key, value } if key == name => Some(value.clone()),
            _ => None,
        })
    }
}
