use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

use pendon_core::{Event, NodeKind, Severity};
use pendon_extra::{
    issue_event, scan_extras_chars, to_attributes, ElementHead, ExtrasAttr, ExtrasHead,
    ExtrasIssue, FieldName, Preset, Slot, SlotContract, SlotKind, SlotPos, SlotTag, SpecsKey,
};
use pendon_renderer_solid::{
    layer_templates, ComponentSet, DataExport, ImportEntry, SolidRenderHints,
};
use serde_json::{Map, Value};

// --- Slot head (§7.3, §6.1) ---

/// Every position cite claims: `[^reference_id "locator" `bracket tick`](paren …)`
/// — a claimed head is read whole, so an unnamed content becomes a bare field
/// (`bracket.tick`, `paren.bare`, …) rather than being dropped (D-7).
const SLOTS: &[SlotPos] = &[
    (Slot::Bracket, SlotKind::Bare),
    (Slot::Bracket, SlotKind::Quote),
    (Slot::Bracket, SlotKind::Tick),
    (Slot::Paren, SlotKind::Bare),
    (Slot::Paren, SlotKind::Quote),
    (Slot::Paren, SlotKind::Tick),
];

/// §7.3's preset assignment: `bracket.bare` → `reference_id`,
/// `bracket.quote` → `locator`.
///
/// These two are the only *slot* presets; `citation_number` is computed by this
/// plugin and is therefore not a position. The processor reads `reference_id` and
/// `locator` **by name**, so a user may move them to any position in the head
/// (§7.3) as long as the names survive — or override the names and keep the
/// processor reading its own (D-4).
const PRESET: &[Preset] = &[
    Preset::new(
        Slot::Bracket,
        SlotKind::Bare,
        FieldName::new("reference_id"),
    ),
    Preset::new(Slot::Bracket, SlotKind::Quote, FieldName::new("locator")),
];

/// The tag that claims cite's head (D-11), the positions it claims, and their
/// fallback-to-preset assignment (§6.1, §6.5, D-7).
pub const HEAD: ElementHead = ElementHead::new(
    SlotContract {
        accepted: SLOTS,
        tag: SlotTag::Cite,
    },
    PRESET,
);

// --- Public Types ---

#[derive(Clone, Debug, Default)]
pub struct CiteCustomNode {
    pub name: String,
    pub template: String,
    pub imports: Vec<ImportEntry>,
}

/// §11: the layer `[task.cite.custom]` declares.
const CITE_LAYER: &str = "cite";

/// The layer key `[task.cite.custom.<layer>]` addresses.
pub fn primary_layer() -> &'static str {
    CITE_LAYER
}

#[derive(Clone, Debug)]
pub struct CiteOptions {
    pub ref_id_prefix: String,
    pub citation_class: String,
    pub citation_id_prefix: String,
    /// Export variable name for references data (Solid/JS output).
    pub references_export: String,
    /// Export variable name for citations data (Solid/JS output).
    pub citations_export: String,
    /// Which references the `references` export carries (§2.6 of RFC-0002).
    /// `None` keeps the source-dependent default
    /// ([`ReferencesScope::default_for_source`]).
    pub references_scope: Option<ReferencesScope>,
    /// §11 component set of the `cite` layer: typed entries plus at most one
    /// default, resolved for the whole document by the renderer against the
    /// `@@type{…}` markers it occurs (rule 3, ADR-0008 `D-3a`).
    pub custom: ComponentSet<CiteCustomNode>,
    pub external_references: Option<Value>,
}

impl Default for CiteOptions {
    fn default() -> Self {
        Self {
            ref_id_prefix: "citeref-".to_string(),
            citation_class: "cite-ref".to_string(),
            citation_id_prefix: "cite-".to_string(),
            references_export: "references".to_string(),
            citations_export: "citations".to_string(),
            references_scope: None,
            custom: ComponentSet::new(),
            external_references: None,
        }
    }
}

/// Which references the `references` export carries (§2.6 of RFC-0002).
///
/// The scope is a **filter on the export only**: `citations`, the
/// `citation_number` / `instance_index` assignment and the diagnostics are
/// unaffected by it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferencesScope {
    /// Only the references the document actually cites.
    Cited,
    /// Every reference the document knows about (the merged
    /// frontmatter/external map), cited or not.
    All,
}

impl ReferencesScope {
    /// The configured spelling of the variant (`cited` / `all`).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cited => "cited",
            Self::All => "all",
        }
    }

    /// Parses the configured spelling; `None` for anything else, so callers can
    /// fail loudly with the list of valid options.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "cited" => Some(Self::Cited),
            "all" => Some(Self::All),
            _ => None,
        }
    }

    /// The default when `references_scope` is not configured (§2.6).
    ///
    /// Frontmatter references are authored for the document itself, so they
    /// default to `all`; an external catalog may be large and mostly unrelated,
    /// so it defaults to `cited`.
    pub fn default_for_source(external: bool) -> Self {
        if external {
            Self::Cited
        } else {
            Self::All
        }
    }
}

/// Extra attributes parsed from the §7.3 `{…}` / `@@type{…}` extras head after
/// the cite args. The retired `[.class,#id]{key: val}` block is literal text
/// (§14).
#[derive(Debug, Clone, Default)]
struct CiteExtraAttrs {
    classes: Vec<String>,
    id: Option<String>,
    data: Vec<(String, String)>,
    /// The `style` value the extras head merged from `--var` items (§6.3).
    extras_style: Option<String>,
    /// Bare flags of the extras head (§6.3), emitted as bare attributes.
    flags: Vec<String>,
    /// §13 diagnostics raised while merging the head.
    issues: Vec<ExtrasIssue>,
    /// The `@@type{…}` marker, when present: the §11 routing key (rule 3).
    type_marker: Option<String>,
}

impl CiteExtraAttrs {
    /// Whether the construct side already sets `key` (extras then lose, §6.2).
    fn has_key(&self, key: &str) -> bool {
        match key {
            "id" => self.id.is_some(),
            "class" => !self.classes.is_empty(),
            "style" => self.extras_style.is_some(),
            _ => self.data.iter().any(|(name, _)| name == key),
        }
    }
}

/// Shared citation state that persists across multiple process_calls within
/// the same document. This allows cite to work correctly in sub-pipelines
/// (e.g., image captions) while maintaining a global index counter and
/// identity deduplication map.
pub struct CitationContext {
    references: Value,
    options: CiteOptions,
    /// Every **physical** citation occurrence, in document order. This is the
    /// data exported as `export const citations = [...]` (§2.3): entries repeat
    /// when the same `(ref_id, loc)` is cited more than once.
    citations: Vec<Value>,
    /// `(ref_id, loc)` → display ordinal (`citation_number`).
    identities: HashMap<String, usize>,
    /// Next display ordinal handed out to a new `(ref_id, loc)` tuple.
    next_citation_number: usize,
    /// Absolute counter for physical citation occurrences (never deduplicates).
    instance_counter: usize,
    diagnostics: Vec<Event>,
}

impl CitationContext {
    /// Creates a new context from references and options.
    /// Call this once per document, then use process_events() for each
    /// event stream (main document, captions, etc.).
    pub fn new(references: Value, options: CiteOptions) -> Self {
        Self {
            references,
            options,
            citations: Vec::new(),
            identities: HashMap::new(),
            next_citation_number: 0,
            instance_counter: 0,
            diagnostics: Vec::new(),
        }
    }

    /// Processes an event stream, transforming citation syntax into citation nodes.
    /// Mutates internal state (cites, identities, diagnostics) so subsequent calls
    /// continue numbering from where the previous call left off.
    pub fn process_events(&mut self, events: &[Event]) -> Vec<Event> {
        let mut out = Vec::with_capacity(events.len());
        let mut excluded = 0usize;
        let mut in_frontmatter = false;

        for event in events {
            match event {
                Event::StartNode(kind) => {
                    if matches!(
                        kind,
                        NodeKind::CodeFence
                            | NodeKind::InlineCode
                            | NodeKind::HtmlBlock
                            | NodeKind::HtmlInline
                    ) {
                        excluded += 1;
                    }
                    if *kind == NodeKind::Frontmatter {
                        in_frontmatter = true;
                    }
                    out.push(event.clone());
                }
                Event::EndNode(kind) => {
                    if *kind == NodeKind::Frontmatter {
                        in_frontmatter = false;
                    }
                    if matches!(
                        kind,
                        NodeKind::CodeFence
                            | NodeKind::InlineCode
                            | NodeKind::HtmlBlock
                            | NodeKind::HtmlInline
                    ) {
                        excluded = excluded.saturating_sub(1);
                    }
                    out.push(event.clone());
                }
                Event::Text(text) if excluded == 0 && !in_frontmatter => {
                    emit_text(text, self, &mut out);
                }
                _ => out.push(event.clone()),
            }
        }

        out
    }

    /// Returns every physical citation occurrence as a JSON array.
    ///
    /// Each entry carries `ref_id`, `citation_number`, `instance_index` and the
    /// optional `loc`. A duplicated `(ref_id, loc)` keeps the same
    /// `citation_number` but gets its own `instance_index` (§2.2).
    pub fn get_citations(&self) -> Value {
        Value::Array(self.citations.clone())
    }

    /// Returns only the references that were actually cited.
    pub fn get_used_references(&self) -> Value {
        let mut used = Map::new();
        if let Some(refs) = self.references.as_object() {
            for cite in &self.citations {
                if let Some(id) = cite.get("ref_id").and_then(Value::as_str) {
                    if let Some(reference) = refs.get(id) {
                        used.insert(id.to_string(), reference.clone());
                    }
                }
            }
        }
        if let Some(ext) = self
            .options
            .external_references
            .as_ref()
            .and_then(Value::as_object)
        {
            for cite in &self.citations {
                if let Some(id) = cite.get("ref_id").and_then(Value::as_str) {
                    if !used.contains_key(id) {
                        if let Some(reference) = ext.get(id) {
                            used.insert(id.to_string(), reference.clone());
                        }
                    }
                }
            }
        }
        Value::Object(used)
    }

    /// The effective export scope: an explicit `references_scope` always wins,
    /// otherwise it follows the reference source (§2.6 of RFC-0002).
    pub fn references_scope(&self) -> ReferencesScope {
        self.options.references_scope.unwrap_or_else(|| {
            ReferencesScope::default_for_source(self.options.external_references.is_some())
        })
    }

    /// The map the `references` export carries, filtered by
    /// [`CitationContext::references_scope`].
    ///
    /// `Scope::all` keeps the merged map as-is — including entries the document
    /// never cites — while `Scope::cited` keeps only what `get_used_references`
    /// resolves (the only behaviour the plugin had before §2.6).
    pub fn get_exported_references(&self) -> Value {
        match self.references_scope() {
            ReferencesScope::Cited => self.get_used_references(),
            ReferencesScope::All => self.references.clone(),
        }
    }

    /// Drains accumulated diagnostics.
    pub fn drain_diagnostics(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.diagnostics)
    }

    /// Returns a reference to the options.
    pub fn options(&self) -> &CiteOptions {
        &self.options
    }
}

// --- Legacy API (backward compatible) ---

pub fn load_references(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path).map_err(|e| {
        format!(
            "cannot read citation reference file '{}': {}",
            path.display(),
            e
        )
    })?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&text).map_err(|e| {
        format!(
            "invalid citation reference YAML '{}': {}",
            path.display(),
            e
        )
    })?;
    yaml_to_json(&yaml)
}

/// Legacy entry point. For new code, prefer CitationContext::new() + process_events().
pub fn process(events: &[Event], options: &CiteOptions) -> Vec<Event> {
    let frontmatter = extract_frontmatter(events);
    let front_references = frontmatter
        .as_ref()
        .and_then(|data| data.get("references"))
        .cloned()
        .unwrap_or_else(|| Value::Object(Map::new()));
    let references = merge_references(&front_references, options.external_references.as_ref());

    let mut ctx = CitationContext::new(references, options.clone());
    let processed = ctx.process_events(events);

    let diagnostics = ctx.drain_diagnostics();
    let mut out = processed;
    if !diagnostics.is_empty() {
        let insert_at = out
            .iter()
            .position(|event| matches!(event, Event::StartNode(NodeKind::Document)))
            .map(|idx| idx + 1)
            .unwrap_or(0);
        for (offset, diagnostic) in diagnostics.into_iter().enumerate() {
            out.insert(insert_at + offset, diagnostic);
        }
    }

    // `cites`/`references` no longer ride in frontmatter (§2.4); they are
    // exported as dedicated JS bindings by the solid renderer.
    out
}

/// Builds the Solid render hints for a document that ran the `cite` layer.
///
/// Besides the §11 component templates this registers the two data exports the
/// RFC-0002 upgrade introduced (§2.3): every physical citation occurrence and
/// the references they point at. They are pushed as `DataExport`s in
/// **reference-then-citations** order — a citation names a reference, so a
/// reader must be able to resolve one without a forward reference (ADR-0009).
pub fn solid_hints(
    options: &CiteOptions,
    citations: &Value,
    references: &Value,
) -> SolidRenderHints {
    let mut hints = SolidRenderHints {
        templates: layer_templates(CITE_LAYER, &options.custom, |custom| {
            (custom.template.clone(), custom.imports.clone())
        }),
        ..SolidRenderHints::default()
    };
    hints.exports.push(DataExport::plugin(
        "cite",
        options.references_export.clone(),
        references.clone(),
    ));
    hints.exports.push(DataExport::plugin(
        "cite",
        options.citations_export.clone(),
        citations.clone(),
    ));
    hints
}

// --- Internal Processing ---

fn emit_text(text: &str, ctx: &mut CitationContext, out: &mut Vec<Event>) {
    let chars: Vec<char> = text.chars().collect();
    let mut cursor = 0usize;
    let mut normal = String::new();

    while cursor < chars.len() {
        if chars[cursor..].starts_with(&['[', '^', '^', ']', '(']) {
            if let Some((end, id, props, mut extra)) = parse_citation(&chars, cursor, ctx.options())
            {
                flush_text(&mut normal, out);

                // §7.3/§6.2: the cite args are the construct head and win over
                // a same-named extras prop.
                let head_keys: Vec<String> = props.keys().cloned().collect();
                let mut dropped: Vec<String> = Vec::new();
                extra.data.retain(|(key, _)| {
                    if head_keys.contains(key) {
                        dropped.push(key.clone());
                        false
                    } else {
                        true
                    }
                });
                for key in dropped {
                    extra.issues.push(ExtrasIssue::warning(format!(
                        "`{key}` was dropped because the construct head already sets it"
                    )));
                }
                for issue in &extra.issues {
                    out.push(issue_event("cite", issue));
                }

                if reference_exists(&ctx.references, &id) {
                    let mut citation = Map::new();
                    citation.insert("ref_id".to_string(), Value::String(id.clone()));
                    for (key, value) in props {
                        citation.insert(key, Value::String(value));
                    }

                    // Identity for deduplication: the `(ref_id, loc)` tuple.
                    // `loc` may be absent, so use a sentinel for "no loc".
                    let loc = citation.get("loc").and_then(Value::as_str).unwrap_or("");
                    let identity = format!("{id}\x1F{loc}");

                    // `citation_number`: the display ordinal. A repeated
                    // `(ref_id, loc)` keeps the number it was first given.
                    let citation_number = match ctx.identities.get(&identity) {
                        Some(&existing) => existing,
                        None => {
                            ctx.next_citation_number += 1;
                            let new_number = ctx.next_citation_number;
                            ctx.identities.insert(identity, new_number);
                            new_number
                        }
                    };

                    // `instance_index`: absolute per physical occurrence; it
                    // never deduplicates (§2.2).
                    ctx.instance_counter += 1;
                    let instance_index = ctx.instance_counter;

                    // Record the physical occurrence for the `citations` export.
                    let mut record = citation.clone();
                    record.insert(
                        "citation_number".to_string(),
                        Value::Number(citation_number.into()),
                    );
                    record.insert(
                        "instance_index".to_string(),
                        Value::Number(instance_index.into()),
                    );
                    ctx.citations.push(Value::Object(record));

                    // The emitted node carries both numbers.
                    citation.insert(
                        "citation_number".to_string(),
                        Value::Number(citation_number.into()),
                    );
                    citation.insert(
                        "instance_index".to_string(),
                        Value::Number(instance_index.into()),
                    );

                    emit_citation(
                        out,
                        &ctx.options,
                        &citation,
                        citation_number,
                        instance_index,
                        &extra,
                    );
                } else {
                    // Reference not found: emit raw syntax as plain text and log diagnostic
                    let raw: String = chars[cursor..end].iter().collect();
                    normal.push_str(&raw);
                    ctx.diagnostics.push(Event::Diagnostic {
                        severity: Severity::Error,
                        message: format!("[cite] reference '{}' was not found", id),
                        span: None,
                    });
                }

                // CRITICAL: Always advance cursor past the parsed citation,
                // regardless of whether the reference existed or not.
                cursor = end;
                continue;
            }
        }

        // Fallback: character is not part of a valid citation syntax
        normal.push(chars[cursor]);
        cursor += 1;
    }

    flush_text(&mut normal, out);
}

fn emit_citation(
    out: &mut Vec<Event>,
    options: &CiteOptions,
    citation: &Map<String, Value>,
    citation_number: usize,
    instance_index: usize,
    extra: &CiteExtraAttrs,
) {
    // §11 rule 3 / D-3a: with a configured `cite` layer every citation becomes a
    // structured, layer-tagged node and the renderer picks the entry; with none
    // configured no template can ever apply, so the built-in `<sup>` markup is
    // emitted directly, exactly as before.
    if !options.custom.is_empty() {
        let node = NodeKind::Element("sup".to_string());
        out.push(Event::StartNode(node.clone()));
        out.push(Event::Attribute {
            name: "name".to_string(),
            value: "sup".to_string(),
        });
        out.push(Event::Attribute {
            name: "__layer".to_string(),
            value: CITE_LAYER.to_string(),
        });
        if let Some(marker) = &extra.type_marker {
            out.push(Event::Attribute {
                name: "type".to_string(),
                value: marker.clone(),
            });
        }
        // Emit core citation attributes
        for (key, value) in citation {
            out.push(Event::Attribute {
                name: key.clone(),
                value: value_to_string(value),
            });
        }
        // Emit extra attributes (class, id, props, style)
        emit_extra_attrs(out, extra);
        out.push(Event::EndNode(node));
        return;
    }

    // Default HTML output
    emit_citation_html(
        out,
        options,
        citation,
        citation_number,
        instance_index,
        extra,
    );
}

/// Default HTML rendering with extra class support
fn emit_citation_html(
    out: &mut Vec<Event>,
    options: &CiteOptions,
    citation: &Map<String, Value>,
    citation_number: usize,
    instance_index: usize,
    extra: &CiteExtraAttrs,
) {
    let ref_id = citation
        .get("ref_id")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let target = format!("{}{}-{}", options.ref_id_prefix, citation_number, ref_id);
    let anchor = format!("{}{}", options.citation_id_prefix, instance_index);

    // Build class list: base citation_class + extra classes
    let mut classes = vec![options.citation_class.clone()];
    classes.extend(extra.classes.iter().cloned());
    let class_attr = classes.join(" ");

    let mut html = format!(
        "<sup class=\"{}\"><a href=\"#{}\" id=\"{}\"",
        escape_html(&class_attr),
        escape_html(&target),
        escape_html(&anchor),
    );

    // Add extra id if present (appended to anchor id)
    if let Some(extra_id) = &extra.id {
        html.push_str(&format!(" data-cite-id=\"{}\"", escape_html(extra_id)));
    }

    // Add data attributes
    for (k, v) in &extra.data {
        html.push_str(&format!(" data-{}=\"{}\"", escape_html(k), escape_html(v)));
    }

    // Add style attributes
    let style = cite_style(extra);
    if !style.is_empty() {
        html.push_str(&format!(" style=\"{}\"", escape_html(&style)));
    }

    // §6.3: a bare flag stays a bare attribute in the markup too.
    for name in &extra.flags {
        html.push_str(&format!(" {}", escape_html(name)));
    }

    html.push_str(&format!(">[{}]</a></sup>", citation_number));

    out.push(Event::StartNode(NodeKind::HtmlInline));
    out.push(Event::Text(html));
    out.push(Event::EndNode(NodeKind::HtmlInline));
}

/// Emits extra attributes on a custom component node.
///
/// Unlike the default HTML path (which renders arbitrary props as `data-*`
/// attributes), a custom Solid component receives plain props, so the keys are
/// passed through verbatim. The extra `#id` is emitted as `cite-id` to keep it
/// distinguishable from the citation reference `id`.
fn emit_extra_attrs(out: &mut Vec<Event>, extra: &CiteExtraAttrs) {
    if let Some(id) = &extra.id {
        out.push(Event::Attribute {
            name: "cite-id".to_string(),
            value: id.clone(),
        });
    }
    if !extra.classes.is_empty() {
        out.push(Event::Attribute {
            name: "class".to_string(),
            value: extra.classes.join(" "),
        });
    }
    for (k, v) in &extra.data {
        out.push(Event::Attribute {
            name: k.clone(),
            value: v.clone(),
        });
    }
    let style = cite_style(extra);
    if !style.is_empty() {
        out.push(Event::Attribute {
            name: "style".to_string(),
            value: style,
        });
    }
    // §6.3: a bare flag stays a bare attribute on custom components too.
    for name in &extra.flags {
        out.push(Event::AttributeFlag { name: name.clone() });
    }
}

/// The `style` value of a citation: the `--var` items and the `style:` prop the
/// extras head merged (§6.3).
fn cite_style(extra: &CiteExtraAttrs) -> String {
    let mut styles = String::new();
    if let Some(value) = &extra.extras_style {
        styles.push_str(value);
        if !styles.ends_with(';') {
            styles.push(';');
        }
    }
    styles
}

// --- Parsing Helpers ---

/// Parses `[^^](ref "loc")` (§7.3) plus an adjacent extras head.
///
/// The reference is an **unquoted** token and the location is an optional
/// **quoted** string. Everything else stays literal text: a quoted reference
/// (`[^^](book)`), the retired `("ref", "loc")` form, a `loc=` prop and an
/// empty `[^^]()` are not citations.
///
/// Returns (total_end_position, cite_id, cite_props, extra_attrs).
fn parse_citation(
    chars: &[char],
    start: usize,
    options: &CiteOptions,
) -> Option<(usize, String, BTreeMap<String, String>, CiteExtraAttrs)> {
    // `[^^](` is five characters.
    let mut cursor = start + 5;

    while matches!(chars.get(cursor), Some(ch) if ch.is_whitespace()) {
        cursor += 1;
    }
    let reference_start = cursor;
    while let Some(&ch) = chars.get(cursor) {
        if ch == ')' || ch.is_whitespace() {
            break;
        }
        if ch == '"' || ch == ',' {
            return None;
        }
        cursor += 1;
    }
    if cursor == reference_start {
        return None;
    }
    let id: String = chars[reference_start..cursor].iter().collect();
    let mut props = BTreeMap::new();

    // An optional quoted location, separated from the reference by whitespace.
    let mut after_space = cursor;
    while matches!(chars.get(after_space), Some(ch) if ch.is_whitespace()) {
        after_space += 1;
    }
    if after_space > cursor {
        if chars.get(after_space) != Some(&'"') {
            return None;
        }
        let mut quote_end = after_space + 1;
        while let Some(&ch) = chars.get(quote_end) {
            if ch == '"' {
                break;
            }
            quote_end += 1;
        }
        if chars.get(quote_end) != Some(&'"') {
            return None;
        }
        let loc: String = chars[after_space + 1..quote_end].iter().collect();
        props.insert("loc".to_string(), loc);
        cursor = quote_end + 1;
        while matches!(chars.get(cursor), Some(ch) if ch.is_whitespace()) {
            cursor += 1;
        }
    }

    if chars.get(cursor) != Some(&')') {
        return None;
    }
    cursor += 1;

    let extra = parse_cite_extra_attrs(chars, &mut cursor, options);
    Some((cursor, id, props, extra))
}

/// Parses the blocks attached to a citation: the adjacent §7.3 `{…}` /
/// `@@type{…}` extras head. It attaches to the citation node; the construct head
/// (the citation args) wins (§6.2). The retired `[.class,#id]{key: val}` block is
/// literal text (§14).
fn parse_cite_extra_attrs(
    chars: &[char],
    cursor: &mut usize,
    options: &CiteOptions,
) -> CiteExtraAttrs {
    let mut extra = CiteExtraAttrs::default();

    // §7.3/§4.1: the extras head must be adjacent (`[^^](book)@@cite{…}`).
    let rest: Vec<char> = chars[*cursor..].to_vec();
    if let Some((head, next)) = scan_extras_chars(&rest, 0) {
        *cursor += next;
        merge_cite_extras(&mut extra, &head, options);
    }

    extra
}

/// Merges a §7.3 extras head into the citation's extra attributes.
///
/// The construct side wins (§6.2): a slot the head or the deprecated block
/// already set is kept and the extras value is reported as dropped. `class`
/// accumulates (§6.4) and bare flags stay bare attributes (§6.3).
fn merge_cite_extras(extra: &mut CiteExtraAttrs, head: &ExtrasHead, options: &CiteOptions) {
    // §6.1/§11 rule 5: the entry answering this marker names the extras specs.
    let specs = options.custom.specs_for(head.type_marker.as_deref());
    let parsed = to_attributes(head, &specs);
    extra.issues.extend(parsed.issues.iter().cloned());

    // §6.2/§6.3: `#id` > extras `slug`; both feed the `cite-id` slot **while the
    // position keeps its default path** — a `field.specs.*` assignment publishes it
    // as a field of the author's own, which the citation no longer reads.
    let id = specs.element_path(SpecsKey::Id);
    let tick = specs.element_path(SpecsKey::Tick);
    let extras_id = id
        .and_then(|path| parsed.value(path))
        .map(|value| value.literal());
    let extras_slug = tick
        .and_then(|path| parsed.value(path))
        .map(|value| value.literal());

    // §6.3: the citation's own positions — `class` / `style` are the universal
    // DOM fields, `#id` feeds `cite-id` and an extras `"title"` is the `title`.
    for (key, value) in specs.dom_fields(
        &parsed.items,
        &[(SpecsKey::Id, "cite-id"), (SpecsKey::Quote, "title")],
    ) {
        if key == "cite-id" || tick == Some(key) {
            continue;
        }
        match value {
            ExtrasAttr::Flag => {
                if extra.has_key(key) {
                    extra.issues.push(ExtrasIssue::warning(format!(
                        "`{key}` was dropped because the construct head already sets it"
                    )));
                } else {
                    extra.flags.push(key.to_string());
                }
            }
            ExtrasAttr::Value(value) => {
                let text = value.literal();
                if key == "class" {
                    for token in text.split_whitespace() {
                        extra.classes.push(token.to_string());
                    }
                    continue;
                }
                if key == "style" {
                    if extra.has_key("style") {
                        extra.issues.push(ExtrasIssue::warning(
                            "`style` was dropped because the construct head already sets it",
                        ));
                    } else {
                        extra.extras_style = Some(text);
                    }
                    continue;
                }
                if extra.has_key(key) {
                    extra.issues.push(ExtrasIssue::warning(format!(
                        "`{key}` was dropped because the construct head already sets it"
                    )));
                    continue;
                }
                extra.data.push((key.to_string(), text));
            }
        }
    }

    match extras_id.or(extras_slug) {
        Some(id) if extra.id.is_none() => extra.id = Some(id),
        Some(_) => extra.issues.push(ExtrasIssue::warning(
            "extras `id` was dropped because the citation already has an id (§6.2)",
        )),
        None => {}
    }

    // §11 rule 3: the type marker is the routing key of the citation; it is
    // carried as a `type` attribute so a `{attrs.type}` template can read it
    // back. An explicit `type:` prop keeps its own value.
    extra.type_marker = head.type_marker.clone();
    if let Some(marker) = &head.type_marker {
        if !extra.has_key("type") {
            extra.data.push(("type".to_string(), marker.clone()));
        }
    }
}

// --- Frontmatter Helpers ---

fn extract_frontmatter(events: &[Event]) -> Option<Value> {
    let mut inside = false;
    for event in events {
        match event {
            Event::StartNode(NodeKind::Frontmatter) => inside = true,
            Event::EndNode(NodeKind::Frontmatter) => inside = false,
            Event::Attribute { name, value } if inside && name == "data" => {
                if let Ok(data) = serde_json::from_str(value) {
                    return Some(data);
                }
            }
            _ => {}
        }
    }
    None
}

fn merge_references(front: &Value, external: Option<&Value>) -> Value {
    let mut merged = Map::new();
    if let Some(map) = external.and_then(Value::as_object) {
        for (key, value) in map {
            merged.insert(key.clone(), value.clone());
        }
    }
    if let Some(map) = front.as_object() {
        for (key, value) in map {
            merged.insert(key.clone(), value.clone());
        }
    }
    Value::Object(merged)
}

fn reference_exists(references: &Value, id: &str) -> bool {
    references
        .as_object()
        .is_some_and(|map| map.contains_key(id))
}

fn value_to_string(value: &Value) -> String {
    value
        .as_str()
        .map(str::to_string)
        .unwrap_or_else(|| value.to_string())
}

fn flush_text(text: &mut String, out: &mut Vec<Event>) {
    if !text.is_empty() {
        out.push(Event::Text(std::mem::take(text)));
    }
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn yaml_to_json(value: &serde_yaml::Value) -> Result<Value, String> {
    match value {
        serde_yaml::Value::Null => Ok(Value::Null),
        serde_yaml::Value::Bool(value) => Ok(Value::Bool(*value)),
        serde_yaml::Value::Number(value) => serde_json::to_value(value).map_err(|e| e.to_string()),
        serde_yaml::Value::String(value) => Ok(Value::String(value.clone())),
        serde_yaml::Value::Sequence(values) => values
            .iter()
            .map(yaml_to_json)
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array),
        serde_yaml::Value::Mapping(values) => {
            let mut object = Map::new();
            for (key, value) in values {
                let key = match key {
                    serde_yaml::Value::String(key) => key.clone(),
                    _ => return Err("citation reference mapping keys must be strings".to_string()),
                };
                object.insert(key, yaml_to_json(value)?);
            }
            Ok(Value::Object(object))
        }
        serde_yaml::Value::Tagged(tagged) => yaml_to_json(&tagged.value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pendon_renderer_solid::TypedComponent;

    /// §7.3's preset, position by position (S2-2.5): cite presets the bracket,
    /// not the paren — the old `[^^](ref "locator")` form is pre-D-16 history.
    #[test]
    fn the_head_declares_the_spec_preset() {
        assert_eq!(HEAD.contract.tag, SlotTag::Cite);
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
        assert_eq!(
            got,
            ["bracket.bare -> reference_id", "bracket.quote -> locator"]
        );
        // The paren is claimed (its contents are fields) but never preset.
        assert!(HEAD.contract.accepts(Slot::Paren, SlotKind::Bare));
        assert_eq!(HEAD.preset_for(Slot::Paren, SlotKind::Bare), None);
        assert_eq!(HEAD.preset_for(Slot::Bracket, SlotKind::Tick), None);
    }

    fn events_with_refs() -> Vec<Event> {
        vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Frontmatter),
            Event::Attribute {
                name: "data".into(),
                value: r#"{"references":{"book":{"title":"Book"}}}"#.into(),
            },
            Event::EndNode(NodeKind::Frontmatter),
            Event::StartNode(NodeKind::Paragraph),
            Event::Text(r#"A [^^](book "p. 1") B [^^](book "p. 1") C [^^](book "p. 2")."#.into()),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ]
    }

    #[test]
    fn numbers_duplicate_details_once_and_different_details_separately() {
        let references: Value = serde_json::json!({"book": {"title": "Book"}});
        let mut ctx = CitationContext::new(references, CiteOptions::default());
        let _ = ctx.process_events(&events_with_refs());

        let citations = ctx.get_citations();
        let arr = citations.as_array().unwrap();
        // Three physical occurrences: `p. 1`, `p. 1`, `p. 2` (§2.2).
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0]["ref_id"], "book");
        assert_eq!(arr[0]["citation_number"], 1);
        assert_eq!(arr[0]["instance_index"], 1);
        assert_eq!(arr[1]["loc"], "p. 1");
        assert_eq!(arr[1]["citation_number"], 1);
        assert_eq!(arr[1]["instance_index"], 2);
        assert_eq!(arr[2]["loc"], "p. 2");
        assert_eq!(arr[2]["citation_number"], 2);
        assert_eq!(arr[2]["instance_index"], 3);
    }

    #[test]
    fn shared_context_continues_numbering_across_calls() {
        let references: Value = serde_json::json!({"book": {"title": "Book"}});
        let options = CiteOptions::default();
        let mut ctx = CitationContext::new(references, options);

        let events_a = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Paragraph),
            Event::Text(r#"[^^](book "p. 1")"#.into()),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ];
        let _out_a = ctx.process_events(&events_a);

        let events_b = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Paragraph),
            Event::Text(r#"[^^](book "p. 2")"#.into()),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ];
        let _out_b = ctx.process_events(&events_b);

        let citations = ctx.get_citations();
        let arr = citations.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["citation_number"], 1);
        assert_eq!(arr[0]["instance_index"], 1);
        assert_eq!(arr[1]["citation_number"], 2);
        assert_eq!(arr[1]["instance_index"], 2);
    }

    /// §2.1: the inline anchor is keyed by `instance_index`, its `href` by
    /// `citation_number` + `ref_id`, so a duplicate citation gets a fresh
    /// anchor id but reuses the number.
    #[test]
    fn duplicate_citations_keep_the_number_but_get_a_fresh_anchor() {
        let result = process(
            &events_with_text(r#"[^^](book "p. 1") and [^^](book "p. 1")"#),
            &CiteOptions::default(),
        );

        let html = html_text(&result);
        assert!(
            html.contains(r##"href="#citeref-1-book" id="cite-1""##),
            "{html}"
        );
        assert!(
            html.contains(r##"href="#citeref-1-book" id="cite-2""##),
            "{html}"
        );
        assert_eq!(html.matches("[1]</a>").count(), 2, "{html}");
    }

    /// §2.4: the plugin no longer writes `cites`/`references` into frontmatter.
    #[test]
    fn frontmatter_is_left_untouched() {
        let result = process(&events_with_refs(), &CiteOptions::default());
        let data = result
            .iter()
            .find_map(|event| match event {
                Event::Attribute { name, value } if name == "data" => {
                    serde_json::from_str::<Value>(value).ok()
                }
                _ => None,
            })
            .unwrap();
        assert_eq!(
            data,
            serde_json::json!({"references": {"book": {"title": "Book"}}})
        );
    }

    /// A document whose reference map holds a cited and an uncited entry.
    fn catalog() -> Value {
        serde_json::json!({
            "book": {"title": "Book"},
            "paper": {"title": "Paper"},
        })
    }

    fn context(references: Value, options: CiteOptions) -> CitationContext {
        let mut ctx = CitationContext::new(references, options);
        let _ = ctx.process_events(&events_with_text(r#"[^^](book)"#));
        ctx
    }

    /// §2.6: frontmatter references are authored for this document, so the
    /// default export is the whole map — the uncited `paper` ships too.
    #[test]
    fn frontmatter_references_are_exported_in_full_by_default() {
        let ctx = context(catalog(), CiteOptions::default());

        assert_eq!(ctx.references_scope(), ReferencesScope::All);
        let exported = ctx.get_exported_references();
        assert_eq!(exported["book"]["title"], "Book");
        assert_eq!(exported["paper"]["title"], "Paper", "{exported}");
    }

    /// §2.6: an external catalog may be large, so it defaults to cited-only.
    #[test]
    fn external_references_are_exported_only_when_cited_by_default() {
        let mut options = CiteOptions::default();
        options.external_references = Some(catalog());
        let ctx = context(catalog(), options);

        assert_eq!(ctx.references_scope(), ReferencesScope::Cited);
        let exported = ctx.get_exported_references();
        assert_eq!(exported["book"]["title"], "Book");
        assert!(exported.get("paper").is_none(), "{exported}");
    }

    /// §2.6: an explicit scope always wins over the source-dependent default.
    #[test]
    fn an_explicit_references_scope_overrides_the_source_default() {
        let mut options = CiteOptions::default();
        options.references_scope = Some(ReferencesScope::Cited);
        let exported = context(catalog(), options).get_exported_references();
        assert_eq!(exported.as_object().unwrap().len(), 1, "{exported}");
        assert!(exported.get("paper").is_none(), "{exported}");

        let mut options = CiteOptions::default();
        options.references_scope = Some(ReferencesScope::All);
        options.external_references = Some(catalog());
        let exported = context(catalog(), options).get_exported_references();
        assert!(exported.get("paper").is_some(), "{exported}");
    }

    /// §2.6: the scope filters the export only — numbering, anchors and the
    /// citation list are identical under either scope.
    #[test]
    fn the_scope_does_not_change_citations_or_numbering() {
        let mut all = context(catalog(), CiteOptions::default());
        let mut options = CiteOptions::default();
        options.references_scope = Some(ReferencesScope::Cited);
        let cited = context(catalog(), options);

        assert_eq!(all.get_citations(), cited.get_citations());
        assert_eq!(
            all.get_citations()[0]["citation_number"],
            serde_json::json!(1)
        );
        assert_eq!(
            all.get_citations()[0]["instance_index"],
            serde_json::json!(1)
        );
        assert!(all.drain_diagnostics().is_empty());
    }

    #[test]
    fn renders_layer_tagged_node() {
        let mut options = CiteOptions::default();
        options.custom =
            ComponentSet::from_entries([TypedComponent::default_component(CiteCustomNode {
                name: "Citation".into(),
                template: "<Citation>{children}</Citation>".into(),
                imports: Vec::new(),
            })]);
        let result = process(&events_with_refs(), &options);
        // §11 rule 3 / D-3a: the node stays an element tagged with the layer; the
        // renderer decides which entry answers it.
        assert!(result.iter().any(|event| matches!(
            event,
            Event::Attribute { name, value } if name == "__layer" && value == "cite"
        )));
        assert!(!result
            .iter()
            .any(|event| matches!(event, Event::StartNode(NodeKind::Custom(_)))));
    }

    #[test]
    fn parses_extra_attrs_after_cite_args() {
        let chars: Vec<char> =
            r#"[^^](book "p. 1"){.highlight, .urgent, #my-cite, foo: "bar", --color: "red"}"#
                .chars()
                .collect();
        let (end, id, props, extra) = parse_citation(&chars, 0, &CiteOptions::default()).unwrap();
        assert_eq!(id, "book");
        assert_eq!(props.get("loc").map(|s| s.as_str()), Some("p. 1"));
        assert_eq!(extra.classes, vec!["highlight", "urgent"]);
        assert_eq!(extra.id, Some("my-cite".to_string()));
        assert_eq!(extra.data, vec![("foo".to_string(), "bar".to_string())]);
        assert_eq!(extra.extras_style.as_deref(), Some("--color: red"));
        assert!(end > 0);
    }

    /// §7.3/D5: the only accepted forms are an unquoted ref and an optional
    /// quoted loc. Everything else is literal text.
    #[test]
    fn the_retired_cite_forms_are_literal_text() {
        for source in [
            concat!("[^^](", "\"book\"", ")"),
            concat!("[^^](book, ", "\"p. 1\"", ")"),
            concat!("[^^](book, loc=", "\"p. 1\"", ")"),
            "[^^]()",
        ] {
            let chars: Vec<char> = source.chars().collect();
            assert!(
                parse_citation(&chars, 0, &CiteOptions::default()).is_none(),
                "{source} must stay literal"
            );
        }

        // …and none of them renders a citation.
        let result = process(
            &events_with_text(concat!(
                "[^^](",
                "\"book\"",
                ") and [^^](book, ",
                "\"p. 1\"",
                ") and [^^]()"
            )),
            &CiteOptions::default(),
        );
        assert!(html_text(&result).is_empty(), "{}", html_text(&result));
        assert!(result.iter().any(|event| matches!(
            event,
            Event::Text(text) if text.contains("book")
        )));
    }

    #[test]
    fn emits_extra_attrs_on_custom_node() {
        let mut options = CiteOptions::default();
        options.custom =
            ComponentSet::from_entries([TypedComponent::default_component(CiteCustomNode {
                name: "Citation".into(),
                template: "<Citation />".into(),
                imports: Vec::new(),
            })]);

        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Frontmatter),
            Event::Attribute {
                name: "data".into(),
                value: r#"{"references":{"book":{"title":"Book"}}}"#.into(),
            },
            Event::EndNode(NodeKind::Frontmatter),
            Event::StartNode(NodeKind::Paragraph),
            Event::Text(r#"[^^](book){.hero, foo: "bar"}"#.into()),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ];

        let result = process(&events, &options);
        assert!(result.iter().any(|e| matches!(
            e,
            Event::Attribute { name, value } if name == "class" && value == "hero"
        )));
        assert!(result.iter().any(|e| matches!(
            e,
            Event::Attribute { name, value } if name == "foo" && value == "bar"
        )));
    }

    /// The raw HTML the default (non-custom) citation path emits.
    fn html_text(events: &[Event]) -> String {
        events
            .iter()
            .filter_map(|event| match event {
                Event::Text(text) if text.starts_with("<sup") => Some(text.clone()),
                _ => None,
            })
            .collect()
    }

    fn events_with_text(text: &str) -> Vec<Event> {
        vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Frontmatter),
            Event::Attribute {
                name: "data".into(),
                value: r#"{"references":{"book":{"title":"Book"}}}"#.into(),
            },
            Event::EndNode(NodeKind::Frontmatter),
            Event::StartNode(NodeKind::Paragraph),
            Event::Text(text.to_string()),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ]
    }

    #[test]
    fn parses_the_extras_head_after_cite_args() {
        let chars: Vec<char> =
            r#"[^^](book)@@cite{.highlight, #short, note: "x", --color: "red"} tail"#
                .chars()
                .collect();
        let (end, id, props, extra) = parse_citation(&chars, 0, &CiteOptions::default()).unwrap();
        assert_eq!(id, "book");
        assert!(props.is_empty());
        assert_eq!(extra.classes, vec!["highlight"]);
        assert_eq!(extra.id.as_deref(), Some("short"));
        // §11 rule 3: the marker rides along as the `type` attribute.
        assert_eq!(extra.type_marker.as_deref(), Some("cite"));
        assert_eq!(
            extra.data,
            vec![
                ("note".to_string(), "x".to_string()),
                ("type".to_string(), "cite".to_string()),
            ]
        );
        assert_eq!(extra.extras_style.as_deref(), Some("--color: red"));

        // Only the citation and its head are consumed, the trailing text stays.
        let consumed: String = chars[..end].iter().collect();
        assert_eq!(
            consumed,
            r#"[^^](book)@@cite{.highlight, #short, note: "x", --color: "red"}"#
        );
    }

    #[test]
    fn extras_merge_into_the_custom_citation_node() {
        let mut options = CiteOptions::default();
        options.custom =
            ComponentSet::from_entries([TypedComponent::default_component(CiteCustomNode {
                name: "Citation".into(),
                template: "<Citation />".into(),
                imports: Vec::new(),
            })]);

        let result = process(
            &events_with_text(r#"[^^](book)@@cite{.hero, citeId: "short", isFoo}"#),
            &options,
        );

        assert!(result.iter().any(|e| matches!(
            e,
            Event::Attribute { name, value } if name == "class" && value == "hero"
        )));
        assert!(result.iter().any(|e| matches!(
            e,
            Event::Attribute { name, value } if name == "citeId" && value == "short"
        )));
        // §6.3: bare flags stay bare attributes.
        assert!(result
            .iter()
            .any(|e| matches!(e, Event::AttributeFlag { name } if name == "isFoo")));
        // §7.3/§2.5: the reference id reaches the template as `ref_id`.
        assert!(result.iter().any(|e| matches!(
            e,
            Event::Attribute { name, value } if name == "ref_id" && value == "book"
        )));
    }

    #[test]
    fn cite_args_win_over_extras_props() {
        let result = process(
            &events_with_text(r#"[^^](book "p. 1")@@cite{loc: "p. 9", note: "n"}"#),
            &CiteOptions::default(),
        );

        let html = html_text(&result);
        assert!(html.contains("data-note=\"n\""), "{html}");
        assert!(!html.contains("loc"), "{html}");
        assert!(result.iter().any(|event| matches!(
            event,
            Event::Diagnostic { message, .. }
                if message.contains("[cite]") && message.contains("`loc`")
        )));
    }

    #[test]
    fn extras_style_and_flags_reach_the_markup() {
        let result = process(
            &events_with_text(r#"[^^](book)@@cite{.hero, --color: "red", isFoo}"#),
            &CiteOptions::default(),
        );

        let html = html_text(&result);
        assert!(html.contains("class=\"cite-ref hero\""), "{html}");
        assert!(html.contains("style=\"--color: red;\""), "{html}");
        assert!(html.contains(" isFoo>"), "{html}");
    }

    #[test]
    fn cites_extras_id_lands_in_the_cite_id_slot() {
        let result = process(
            &events_with_text(r#"[^^](book)@@cite{`short-slug`}"#),
            &CiteOptions::default(),
        );
        // §6.2: `#id` > extras `slug`.
        assert!(html_text(&result).contains("data-cite-id=\"short-slug\""));
    }

    /// §14/D3: the retired `[.class,#id]{k:v}` block is literal text.
    #[test]
    fn the_retired_block_is_literal_text() {
        let result = process(
            &events_with_text(r#"[^^](book)[#short]"#),
            &CiteOptions::default(),
        );
        assert!(
            html_text(&result).starts_with("<sup"),
            "{:?}",
            html_text(&result)
        );
        assert!(!html_text(&result).contains("[#short]"));
        assert!(result.iter().any(|event| matches!(
            event,
            Event::Text(text) if text.contains("[#short]")
        )));
    }

    #[test]
    fn extra_id_does_not_replace_citation_id_on_custom_node() {
        let mut options = CiteOptions::default();
        options.custom =
            ComponentSet::from_entries([TypedComponent::default_component(CiteCustomNode {
                name: "Citation".into(),
                template: "<Citation />".into(),
                imports: Vec::new(),
            })]);

        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Frontmatter),
            Event::Attribute {
                name: "data".into(),
                value: r#"{"references":{"book":{"title":"Book"}}}"#.into(),
            },
            Event::EndNode(NodeKind::Frontmatter),
            Event::StartNode(NodeKind::Paragraph),
            Event::Text(r#"[^^](book){#short}"#.into()),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ];

        let result = process(&events, &options);
        assert!(result.iter().any(|event| matches!(
            event,
            Event::Attribute { name, value } if name == "ref_id" && value == "book"
        )));
        assert!(result.iter().any(|event| matches!(
            event,
            Event::Attribute { name, value } if name == "cite-id" && value == "short"
        )));
    }
}
