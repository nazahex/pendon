use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

use pendon_core::{Event, NodeKind, Severity};
use pendon_extra::{
    issue_event, modifier_contributions, parse_extras, parse_slots, to_attributes, ElementHead,
    ExtrasAttr, ExtrasHead, ExtrasIssue, ExtrasMatch, FieldName, ModifierConfig, Preset, Slot,
    SlotContract, SlotKind, SlotMatch, SlotPair, SlotPos, SlotTag, SpecsKey,
};
use pendon_renderer_solid::{
    layer_templates, ComponentSet, DataExport, ImportEntry, SolidRenderHints,
};
use serde_json::{Map, Value};

// --- Slot head (§7.3, §6.1) ---

/// The two characters that open cite's head: the bracket and its tag (§5.3,
/// D-11).
///
/// The caret sits **inside** the bracket (`SlotTag::Cite`), so the content starts
/// after those two characters. That caret is the only thing that makes a bracket
/// a citation — the old `[^^](ref "loc")` form put a second caret and a paren
/// there instead (§14).
const HEAD_OPEN: &str = "[^";

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
/// plugin and is therefore not a position ([`OWNED`]). The processor reads
/// `reference_id` and `locator` **by name**, so a user may move them to any
/// position in the head (`field.bracket.tick = "reference_id"`) as long as the
/// names survive — or rename them publicly and keep the processor reading its own
/// (D-4).
const PRESET: &[Preset] = &[
    Preset::new(
        Slot::Bracket,
        SlotKind::Bare,
        FieldName::new("reference_id"),
    ),
    Preset::new(Slot::Bracket, SlotKind::Quote, FieldName::new("locator")),
];

/// D-12 rule 5: the field the processor writes itself, so an extras item that
/// spells it is an `Error` (`field.override.<k>` publishes it under another name).
const OWNED: &[FieldName] = &[FieldName::new("citation_number")];

/// The tag that claims cite's head (D-11), the positions it claims, and their
/// fallback-to-preset assignment (§6.1, §6.6, D-7).
pub const HEAD: ElementHead = ElementHead::new(
    SlotContract {
        accepted: SLOTS,
        tag: SlotTag::Cite,
    },
    PRESET,
)
.with_owned(OWNED);

// --- Built-in markup (§7.3) ---

/// The class of the fallback `<sup>` when no component answers the citation.
///
/// Not configurable (`citation_class` is gone, §14): a site that wants its own
/// class names writes a `Cite` component instead — the reason the config key was
/// removed. The three constants below are what the built-in markup has always
/// produced by default.
const CITATION_CLASS: &str = "cite-ref";

/// The `href` prefix of a citation's backlink into the bibliography
/// (`#citeref-1-book`).
const REF_HREF_PREFIX: &str = "citeref-";

/// The `id` prefix of a citation's own anchor (`cite-1`).
const CITATION_ANCHOR_PREFIX: &str = "cite-";

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
/// `citation_number` assignment and the diagnostics are unaffected by it.
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
    /// when the same `(reference_id, locator)` is cited more than once.
    citations: Vec<Value>,
    /// `(reference_id, locator)` → display ordinal (`citation_number`).
    identities: HashMap<String, usize>,
    /// Next display ordinal handed out to a new `(reference_id, locator)` tuple.
    next_citation_number: usize,
    /// Absolute counter for physical citation occurrences (never deduplicates).
    /// Not a field (§7.3): only the built-in markup's backlink anchor uses it.
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
    /// Each entry carries `reference_id`, `citation_number` and, when the head
    /// names one, `locator` (§7.3). A duplicated `(reference_id, locator)` keeps
    /// its `citation_number` and is recorded again — every physical occurrence is
    /// a row, which is what makes `citations` a citation list rather than a
    /// bibliography.
    pub fn get_citations(&self) -> Value {
        Value::Array(self.citations.clone())
    }

    /// Returns only the references that were actually cited.
    pub fn get_used_references(&self) -> Value {
        let mut used = Map::new();
        if let Some(refs) = self.references.as_object() {
            for cite in &self.citations {
                if let Some(id) = cite.get("reference_id").and_then(Value::as_str) {
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
                if let Some(id) = cite.get("reference_id").and_then(Value::as_str) {
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
    let mut cursor = 0usize;
    let mut normal = String::new();

    while cursor < text.len() {
        let rest = &text[cursor..];
        match read_head(rest, ctx) {
            Read::Cite { parts, consumed } => {
                flush_text(&mut normal, out);
                for issue in &parts.extra.issues {
                    out.push(issue_event("cite", issue));
                }
                if reference_exists(&ctx.references, &parts.reference_id) {
                    emit_citation(*parts, ctx, out);
                } else {
                    // Reference not found: the text stays as written and the
                    // document gets an `Error` (§7.3).
                    normal.push_str(&text[cursor..cursor + consumed]);
                    ctx.diagnostics.push(Event::Diagnostic {
                        severity: Severity::Error,
                        message: format!("[cite] reference '{}' was not found", parts.reference_id),
                        span: None,
                    });
                }
                cursor += consumed;
            }
            // §4.3 / D-2: slot-shaped but illegal, or a citation without a
            // reference. The `Error` is reported and the text stays as written.
            Read::Malformed(message) => {
                out.push(Event::Diagnostic {
                    severity: Severity::Error,
                    message: format!("[cite] {message}"),
                    span: None,
                });
                normal.push('[');
                cursor += '['.len_utf8();
            }
            Read::Literal => {
                let ch = rest.chars().next().expect("cursor is in bounds");
                normal.push(ch);
                cursor += ch.len_utf8();
            }
        }
    }

    flush_text(&mut normal, out);
}

/// Emits one citation: assigns its `citation_number`, records the physical
/// occurrence and renders it — a §11 component when the `cite` layer is
/// configured, the built-in `<sup>` markup otherwise.
fn emit_citation(parts: CiteParts, ctx: &mut CitationContext, out: &mut Vec<Event>) {
    // Identity for deduplication: the `(reference_id, locator)` tuple. The
    // locator may be absent, so use a sentinel for "no locator".
    let identity = format!(
        "{}\x1F{}",
        parts.reference_id,
        parts.locator.as_deref().unwrap_or("")
    );

    // `citation_number`: the display ordinal. A repeated
    // `(reference_id, locator)` keeps the number it was first given.
    let citation_number = match ctx.identities.get(&identity) {
        Some(&existing) => existing,
        None => {
            ctx.next_citation_number += 1;
            let new_number = ctx.next_citation_number;
            ctx.identities.insert(identity, new_number);
            new_number
        }
    };

    // `instance_count`: absolute per physical occurrence; it never deduplicates.
    // It is **not** a field (§7.3 — `number` replaces it): the built-in
    // markup uses it for its own backlink anchor, nothing else.
    ctx.instance_counter += 1;
    let instance = ctx.instance_counter;

    // Record the physical occurrence for the `citations` export (§20).
    let mut record = Map::new();
    record.insert(
        "reference_id".to_string(),
        Value::String(parts.reference_id.clone()),
    );
    if let Some(locator) = &parts.locator {
        record.insert("locator".to_string(), Value::String(locator.clone()));
    }
    record.insert(
        "citation_number".to_string(),
        Value::Number(citation_number.into()),
    );
    ctx.citations.push(Value::Object(record));

    let options = &ctx.options;
    // §11 rule 3 / D-3a: with a configured `cite` layer every citation becomes a
    // structured, layer-tagged node and the renderer picks the entry; with none
    // configured no template can ever apply, so the built-in `<sup>` markup is
    // emitted directly.
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
        if let Some(marker) = &parts.extra.type_marker {
            out.push(Event::Attribute {
                name: "type".to_string(),
                value: marker.clone(),
            });
        }
        // The processor's own fields (§7.3): the preset names, so a
        // `field.override.*` can rename them on the way out (D-4).
        out.push(Event::Attribute {
            name: "reference_id".to_string(),
            value: parts.reference_id.clone(),
        });
        if let Some(locator) = &parts.locator {
            out.push(Event::Attribute {
                name: "locator".to_string(),
                value: locator.clone(),
            });
        }
        out.push(Event::Attribute {
            name: "citation_number".to_string(),
            value: citation_number.to_string(),
        });
        // Every other claimed position keeps its (possibly moved) field name.
        for (name, value) in &parts.values {
            out.push(Event::Attribute {
                name: name.clone(),
                value: value.clone(),
            });
        }
        emit_extra_attrs(out, &parts.extra);
        out.push(Event::EndNode(node));
        return;
    }

    emit_citation_html(out, &parts, citation_number, instance);
}

/// The built-in markup: `<sup class="cite-ref"><a href="#citeref-1-book"
/// id="cite-1">[1]</a></sup>` plus the extras the head carries.
///
/// The class and the two prefixes are constants (§7.3): a site that wants its own
/// spelling writes a `Cite` component, which is why `citation_class`,
/// `ref_id_prefix` and `citation_id_prefix` are gone (§14).
///
/// The markup is fixed, but the *fields* are not: every position the head carried
/// beyond `reference_id` / `locator` (D-7's `bracket.tick`, `paren.bare`, …) is
/// written under the `data-` prefix — `bracket.tick` becomes
/// `data-bracket-tick`, the spelling a dotted path gets when it cannot be a prop
/// (§6.3). A cite head never loses content for want of a component.
fn emit_citation_html(
    out: &mut Vec<Event>,
    parts: &CiteParts,
    citation_number: usize,
    instance: usize,
) {
    let extra = &parts.extra;
    let target = format!(
        "{}{}-{}",
        REF_HREF_PREFIX, citation_number, parts.reference_id
    );
    let anchor = format!("{}{}", CITATION_ANCHOR_PREFIX, instance);

    // Build class list: the built-in class + extra classes
    let mut classes = vec![CITATION_CLASS.to_string()];
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

    // §7.3/D-7: every other claimed position is an ordinary field, and this
    // markup has no prop of its own for one — so it writes the field the way the
    // built-in `<img>` writes a key it has no attribute for (§6.3, §7.1): under
    // the `data-` prefix. A dotted path is *not* an attribute name in JSX
    // (§6.3 writes it as a one-key spread, which raw markup cannot do), so
    // `bracket.tick` is spelled `data-bracket-tick` here. Without this the head's
    // remaining contents would be dropped, and only the custom-component path
    // would keep them.
    for (name, value) in &parts.values {
        let attribute = format!("data-{}", name.replace('.', "-"));
        html.push_str(&format!(
            " {}=\"{}\"",
            escape_html(&attribute),
            escape_html(value)
        ));
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

/// What one `[^` in the text turned out to be (§7.3).
enum Read {
    /// Not a citation: no `[^…]` at all, or a bracket another tag claims.
    Literal,
    /// Slot-shaped but illegal (§4.3, D-2), or a citation whose `reference_id`
    /// is missing: an `Error`, and the text stays as written.
    Malformed(String),
    /// A complete citation: its fields and the bytes the head occupied. Boxed
    /// because a citation carries far more than a literal does.
    Cite {
        parts: Box<CiteParts>,
        consumed: usize,
    },
}

/// What one citation carries: the two fields the processor reads, every other
/// claimed position, and the §11 extras head.
#[derive(Debug, Default)]
struct CiteParts {
    /// `reference_id` — the reference the citation points at. Mandatory (§7.3):
    /// a head that does not assign it is not a citation.
    reference_id: String,
    /// `locator` — optional, and part of the citation's identity (§7.3).
    locator: Option<String>,
    /// Every other claimed position, under its field name: a position the preset
    /// does not name keeps its slot-qualified path (`bracket.tick`,
    /// `paren.bare`, …, D-7), and a moved position keeps the author's name.
    values: BTreeMap<String, String>,
    /// The extras head, merged onto the citation (§6.2/§6.3).
    extra: CiteExtraAttrs,
}

/// Reads a citation head at the start of `rest` — the `^` tag, the slots (§5.2–
/// §5.3) and the §11 extras head attached to the closing delimiter.
fn read_head(rest: &str, ctx: &CitationContext) -> Read {
    if !rest.starts_with(HEAD_OPEN) {
        return Read::Literal;
    }
    let (pair, mut consumed) = match parse_slots(rest) {
        SlotMatch::Slots { pair, rest: after } if pair.tag == SlotTag::Cite => {
            (pair, rest.len() - after.len())
        }
        // §4.3 / D-2: tagged and slot-shaped but illegal.
        SlotMatch::Malformed { error, .. } => return Read::Malformed(error.to_string()),
        // `^[` with no usable head, or a bracket another tag owns.
        _ => return Read::Literal,
    };

    // §4.1: the extras head must touch the closing delimiter.
    let extras = match parse_extras(&rest[consumed..]) {
        ExtrasMatch::Head { head, rest: after } => {
            consumed = rest.len() - after.len();
            Some(head)
        }
        ExtrasMatch::Absent { .. } | ExtrasMatch::Malformed { .. } => None,
    };

    let options = ctx.options();
    // §6.6 / D-7: the preset is voided by the **matched** entry, never by another
    // entry's `field.*`. No extras head means no `type` marker, which is exactly
    // the entry the renderer will pick (the layer default), so its `field.*`
    // applies too — otherwise a default `Cite` component could never assign or
    // unassign a position.
    let marker = extras.as_ref().and_then(|head| head.type_marker.as_deref());
    let matched = options.custom.field_for(marker);

    let mut parts = CiteParts::default();
    // §6.6: `bracket.bare → reference_id`, `bracket.quote → locator`; every
    // other position is a bare field (D-7).
    let mut issues = Vec::new();
    let fields = HEAD.assign(&pair, matched).unwrap_or_else(|error| {
        // The entry's assignments contradict each other (one path is a prefix of
        // another), so they cannot all be published. A config clash is the
        // author's bug: say so and keep the preset.
        issues.push(ExtrasIssue::error(format!(
            "the matched entry's `field.*` assignments clash ({error}); the preset is kept"
        )));
        HEAD.assign(&pair, None)
            .expect("an untouched preset cannot clash")
    });
    // The names this construct's own head produced; an extras item spelling one
    // loses (§6.2).
    let mut head_keys: Vec<String> = Vec::new();
    for (path, value) in fields.iter() {
        let name = path.dotted();
        let text = value.literal();
        head_keys.push(name.clone());
        match name.as_str() {
            "reference_id" => parts.reference_id = text,
            "locator" => parts.locator = Some(text),
            _ => {
                parts.values.insert(name, text);
            }
        }
    }
    // §7.3: without a reference there is nothing to number, so the head is not a
    // citation — the `Error` names the cause.
    if parts.reference_id.is_empty() {
        if fields.is_empty() {
            return Read::Malformed(
                "a citation needs a `reference_id` (`[^reference_id]`)".to_string(),
            );
        }
        return Read::Malformed(
            "no position assigns `reference_id`; the cite preset is void (see `field.*`)"
                .to_string(),
        );
    }

    if let Some(head) = &extras {
        merge_cite_extras(&mut parts.extra, head, options, &head_keys);
    }
    // §5.4 / D-9: the bracket's modifier run contributes last, so every field the
    // head or the extras already produced wins (§6.2).
    merge_modifiers(&mut parts.extra, &pair, options.custom.modifier_for(marker));
    parts.extra.issues.extend(issues);

    Read::Cite {
        parts: Box::new(parts),
        consumed,
    }
}

/// Merges a §7.3 extras head into the citation's extra attributes.
///
/// The construct side wins (§6.2): an item that spells a name the head already
/// produced (`reference_id`, `locator`, a slot field) is dropped and reported,
/// `class` accumulates (§6.4) and bare flags stay bare attributes (§6.3).
fn merge_cite_extras(
    extra: &mut CiteExtraAttrs,
    head: &ExtrasHead,
    options: &CiteOptions,
    head_keys: &[String],
) {
    // §6.1/§11 rule 5: the entry answering this marker names the extras specs.
    let specs = options.custom.specs_for(head.type_marker.as_deref());
    let parsed = to_attributes(head, &specs);
    extra.issues.extend(parsed.issues.iter().cloned());

    // §6.2: an extras item that spells a name the citation's own head produced
    // loses — the head is the construct, the extras only decorate it.
    let mut items = Vec::with_capacity(parsed.items.len());
    for (key, value) in parsed.items.iter() {
        if head_keys.iter().any(|head| head == key) {
            extra.issues.push(ExtrasIssue::warning(format!(
                "`{key}` was dropped because the construct head already sets it"
            )));
            continue;
        }
        items.push((key.clone(), value.clone()));
    }

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
        &items,
        &[(SpecsKey::Id, "cite-id"), (SpecsKey::Quote, "title")],
    ) {
        if key == "cite-id" || tick == Some(key) {
            continue;
        }
        if HEAD.owns(key) {
            // D-12 rule 5: the processor writes `citation_number`, so an extras
            // item that spells it is an `Error`, not a silent overwrite.
            extra.issues.push(ExtrasIssue::owned_key(key));
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
                let text = value.transport_for(key);
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

/// Merges `pair`'s modifier run into the citation's fields (§5.4, D-9).
///
/// `class` accumulates (§6.4); every other contributed field is dropped with a
/// warning when the element already carries it, and an unusable merge (D-9's
/// `Severity::Error` tier) drops the whole run.
fn merge_modifiers(extra: &mut CiteExtraAttrs, pair: &SlotPair, config: Option<&ModifierConfig>) {
    let fields = match modifier_contributions(config, pair) {
        Ok(fields) => fields,
        Err(error) => {
            extra
                .issues
                .push(ExtrasIssue::error(error.message().to_string()));
            return;
        }
    };
    for field in fields {
        if field.is_flag() {
            if extra.has_key(&field.name) {
                extra
                    .issues
                    .push(ExtrasIssue::modifier_dropped(&field.name));
            } else {
                extra.flags.push(field.name);
            }
            continue;
        }
        let text = field.text();
        match field.name.as_str() {
            "class" => {
                for token in text.split_whitespace() {
                    if !extra.classes.iter().any(|known| known == token) {
                        extra.classes.push(token.to_string());
                    }
                }
            }
            "style" if !extra.has_key("style") => extra.extras_style = Some(text),
            _ if extra.has_key(&field.name) => {
                extra
                    .issues
                    .push(ExtrasIssue::modifier_dropped(&field.name));
            }
            "id" => extra.id = Some(text),
            _ => extra.data.push((field.name, text)),
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
    use pendon_extra::{AttrValue, Path};
    use pendon_renderer_solid::TypedComponent;

    /// §7.3's preset, position by position (S2-4.4): the bracket carries the
    /// reference and the locator, the paren is claimed but never preset.
    #[test]
    fn the_head_declares_the_spec_preset() {
        assert_eq!(HEAD.contract.tag, SlotTag::Cite);
        // The contract's sigil is the character the reader looks for.
        assert_eq!(HEAD.contract.tag.sigil(), Some('^'));
        assert_eq!(HEAD_OPEN, "[^");
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
        // §7.3: the processor writes `citation_number` itself, so an extras item
        // spelling it is an `Error` (D-12 rule 5).
        assert!(HEAD.owns("citation_number"));
        assert!(!HEAD.owns("reference_id"));
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
            Event::Text(r#"A [^book "p. 1"] B [^book "p. 1"] C [^book "p. 2"]."#.into()),
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
        // Three physical occurrences: `p. 1`, `p. 1`, `p. 2` (§2.2). The record
        // carries the fields (§7.3) — `instance_index` is gone.
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0]["reference_id"], "book");
        assert_eq!(arr[0]["citation_number"], 1);
        assert!(arr[0].get("instance_index").is_none(), "{}", arr[0]);
        assert_eq!(arr[1]["locator"], "p. 1");
        assert_eq!(arr[1]["citation_number"], 1);
        assert_eq!(arr[2]["locator"], "p. 2");
        assert_eq!(arr[2]["citation_number"], 2);
    }

    #[test]
    fn shared_context_continues_numbering_across_calls() {
        let references: Value = serde_json::json!({"book": {"title": "Book"}});
        let options = CiteOptions::default();
        let mut ctx = CitationContext::new(references, options);

        let events_a = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Paragraph),
            Event::Text(r#"[^book "p. 1"]"#.into()),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ];
        let _out_a = ctx.process_events(&events_a);

        let events_b = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Paragraph),
            Event::Text(r#"[^book "p. 2"]"#.into()),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ];
        let _out_b = ctx.process_events(&events_b);

        let citations = ctx.get_citations();
        let arr = citations.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["citation_number"], 1);
        assert_eq!(arr[0]["locator"], "p. 1");
        assert_eq!(arr[1]["citation_number"], 2);
        assert_eq!(arr[1]["locator"], "p. 2");
    }

    /// §2.1: the inline anchor is keyed by the physical occurrence, its `href` by
    /// `citation_number` + `reference_id`, so a duplicate citation gets a fresh
    /// anchor id but reuses the number. Neither is a field any more (§7.3): the
    /// anchor is built-in markup, the number is `citation_number`.
    #[test]
    fn duplicate_citations_keep_the_number_but_get_a_fresh_anchor() {
        let result = process(
            &events_with_text(r#"[^book "p. 1"] and [^book "p. 1"]"#),
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
        let _ = ctx.process_events(&events_with_text(r#"[^book]"#));
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
            all.get_citations()[0]["reference_id"],
            serde_json::json!("book")
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

    /// A context over one reference and no components, for `read_head` tests.
    fn reading_context() -> CitationContext {
        CitationContext::new(
            serde_json::json!({"book": {"title": "Book"}}),
            CiteOptions::default(),
        )
    }

    /// The `Read::Cite` parts of `source`, which must be one citation head.
    fn cite_parts(source: &str) -> (CiteParts, usize) {
        match read_head(source, &reading_context()) {
            Read::Cite { parts, consumed } => (*parts, consumed),
            _ => panic!("{source} is not a citation head"),
        }
    }

    #[test]
    fn parses_extra_attrs_after_cite_args() {
        let (parts, end) = cite_parts(
            r#"[^book "p. 1"]{.highlight, .urgent, #my-cite, foo: "bar", --color: "red"}"#,
        );
        assert_eq!(parts.reference_id, "book");
        assert_eq!(parts.locator.as_deref(), Some("p. 1"));
        assert_eq!(parts.extra.classes, vec!["highlight", "urgent"]);
        assert_eq!(parts.extra.id, Some("my-cite".to_string()));
        assert_eq!(
            parts.extra.data,
            vec![("foo".to_string(), "bar".to_string())]
        );
        assert_eq!(parts.extra.extras_style.as_deref(), Some("--color: red"));
        assert!(end > 0);
    }

    /// §7.3/D-16: the tag, not a second caret, is what makes a citation. The old
    /// spellings are no longer citations — `[^^](book)` now reads `^` as the
    /// reference (`bracket.bare`) and `book` as a paren field, so it cites a
    /// reference named `^`; an empty head has no `reference_id` at all.
    #[test]
    fn the_retired_cite_forms_are_not_citations() {
        let (parts, _) = cite_parts("[^^](book)");
        assert_eq!(parts.reference_id, "^");
        assert_eq!(
            parts.values.get("paren.bare").map(String::as_str),
            Some("book")
        );

        // A head without a `reference_id` is an `Error`, and the text stays.
        for source in ["[^]", "[^\"book\"]", "[^`tick`]"] {
            match read_head(source, &reading_context()) {
                Read::Malformed(message) => {
                    assert!(message.contains("reference_id"), "{source}: {message}");
                }
                _ => panic!("{source} must not be a citation"),
            }
        }

        let result = process(
            &events_with_text("[^^](book) and [^] and [^\"book\"]"),
            &CiteOptions::default(),
        );
        // Nothing renders as a citation: no `<sup>`, and the text survives.
        assert!(html_text(&result).is_empty(), "{}", html_text(&result));
        assert!(result.iter().any(|event| matches!(
            event,
            Event::Text(text) if text.contains("[^^](book)")
        )));
        assert!(result.iter().any(|event| matches!(
            event,
            Event::Diagnostic { message, .. } if message.contains("reference '^' was not found")
        )));
        assert!(result.iter().any(|event| matches!(
            event,
            Event::Diagnostic { message, .. } if message.contains("needs a `reference_id`")
        )));
    }

    /// §5.4 / D-9: the bracket's modifier run contributes its configured fields
    /// to the citation's node — `class` accumulates (§6.4), a bare `true` is a
    /// flag (§6.3) — and the run never leaks into the reference or the locator.
    #[test]
    fn a_modifier_run_contributes_configured_fields() {
        let mut modifier = ModifierConfig::new();
        modifier.define(
            '?',
            vec![
                (
                    Path::from_dotted("rel").unwrap(),
                    AttrValue::Str("nofollow".into()),
                ),
                (
                    Path::from_dotted("class").unwrap(),
                    AttrValue::Str("hero".into()),
                ),
                (Path::from_dotted("compact").unwrap(), AttrValue::Bool(true)),
            ],
        );
        let mut options = CiteOptions::default();
        options.custom =
            ComponentSet::from_entries([TypedComponent::default_component(CiteCustomNode {
                name: "Citation".into(),
                template: "<Citation />".into(),
                imports: Vec::new(),
            })
            .with_modifier(modifier)]);

        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Frontmatter),
            Event::Attribute {
                name: "data".into(),
                value: r#"{"references":{"book":{"title":"Book"}}}"#.into(),
            },
            Event::EndNode(NodeKind::Frontmatter),
            Event::StartNode(NodeKind::Paragraph),
            Event::Text(r#"[^book |?]{.hero}"#.into()),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ];

        let result = process(&events, &options);
        assert!(
            result.iter().any(|event| matches!(
                event,
                Event::Attribute { name, value } if name == "rel" && value == "nofollow"
            )),
            "{result:?}"
        );
        // The extras `class` and the modifier's merge without repeating a token.
        assert!(
            result.iter().any(|event| matches!(
                event,
                Event::Attribute { name, value } if name == "class" && value == "hero"
            )),
            "{result:?}"
        );
        assert!(
            result.iter().any(|event| matches!(
                event,
                Event::AttributeFlag { name } if name == "compact"
            )),
            "{result:?}"
        );
        // The head still resolves: the run is configuration, not the reference.
        assert!(
            result.iter().any(|event| matches!(
                event,
                Event::Attribute { name, value }
                    if name == "reference_id" && value == "book"
            )),
            "{result:?}"
        );
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
            Event::Text(r#"[^book]{.hero, foo: "bar"}"#.into()),
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
        let source = r#"[^book]@@cite{.highlight, #short, note: "x", --color: "red"} tail"#;
        let (parts, end) = cite_parts(source);
        assert_eq!(parts.reference_id, "book");
        assert!(parts.locator.is_none());
        assert_eq!(parts.extra.classes, vec!["highlight"]);
        assert_eq!(parts.extra.id.as_deref(), Some("short"));
        // §11 rule 3: the marker rides along as the `type` attribute.
        assert_eq!(parts.extra.type_marker.as_deref(), Some("cite"));
        assert_eq!(
            parts.extra.data,
            vec![
                ("note".to_string(), "x".to_string()),
                ("type".to_string(), "cite".to_string()),
            ]
        );
        assert_eq!(parts.extra.extras_style.as_deref(), Some("--color: red"));

        // Only the citation and its head are consumed, the trailing text stays.
        assert_eq!(
            &source[..end],
            r#"[^book]@@cite{.highlight, #short, note: "x", --color: "red"}"#
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
            &events_with_text(r#"[^book]@@cite{.hero, citeId: "short", isFoo}"#),
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
        // §7.3: the reference reaches the template as `reference_id`, the field
        // the processor reads (a `field.override.*` renames it publicly, D-4).
        assert!(result.iter().any(|e| matches!(
            e,
            Event::Attribute { name, value } if name == "reference_id" && value == "book"
        )));
    }

    #[test]
    fn cite_args_win_over_extras_props() {
        let result = process(
            &events_with_text(r#"[^book "p. 1"]@@cite{locator: "p. 9", note: "n"}"#),
            &CiteOptions::default(),
        );

        let html = html_text(&result);
        assert!(html.contains("data-note=\"n\""), "{html}");
        assert!(!html.contains("locator"), "{html}");
        assert!(result.iter().any(|event| matches!(
            event,
            Event::Diagnostic { message, .. }
                if message.contains("[cite]") && message.contains("`locator`")
        )));
    }

    #[test]
    fn extras_style_and_flags_reach_the_markup() {
        let result = process(
            &events_with_text(r#"[^book]@@cite{.hero, --color: "red", isFoo}"#),
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
            &events_with_text(r#"[^book]@@cite{`short-slug`}"#),
            &CiteOptions::default(),
        );
        // §6.2: `#id` > extras `slug`.
        assert!(html_text(&result).contains("data-cite-id=\"short-slug\""));
    }

    /// S2-4.12/D-7: the built-in markup is fixed, but the head is not — every
    /// position beyond `reference_id` / `locator` reaches the markup under its
    /// own dotted name, so a head never loses content for want of a component.
    #[test]
    fn the_builtin_markup_keeps_every_other_position() {
        let result = process(
            &events_with_text(r#"[^book "p. 1" `t`](b "q" `u`)"#),
            &CiteOptions::default(),
        );

        let html = html_text(&result);
        for (name, value) in [
            ("data-bracket-tick", "t"),
            ("data-paren-bare", "b"),
            ("data-paren-quote", "q"),
            ("data-paren-tick", "u"),
        ] {
            assert!(
                html.contains(&format!("{name}=\"{value}\"")),
                "`{name}` was dropped: {html}"
            );
        }
        // The preset's own positions keep their markup: the reference is the
        // href, and the locator has no attribute of its own (§7.3).
        assert!(html.contains("href=\"#citeref-1-book\""), "{html}");
        assert!(!html.contains("locator="), "{html}");
    }

    /// §14/D3: the retired `[.class,#id]{k:v}` block is literal text.
    #[test]
    fn the_retired_block_is_literal_text() {
        let result = process(
            &events_with_text(r#"[^book][#short]"#),
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

    /// §7.3: a user may move the two fields to any position; the processor reads
    /// them by name, so numbering still works. `field.bracket.tick` alone voids
    /// the preset (D-7), so `locator` must be assigned too.
    #[test]
    fn a_moved_position_keeps_the_processor_working() {
        let mut field = pendon_extra::FieldConfig::new();
        field.assign_slot(
            Slot::Bracket,
            SlotKind::Tick,
            Some(pendon_extra::Path::from_dotted("reference_id").expect("valid path")),
        );
        field.assign_slot(
            Slot::Bracket,
            SlotKind::Bare,
            Some(pendon_extra::Path::from_dotted("locator").expect("valid path")),
        );
        let mut entry = TypedComponent::default_component(CiteCustomNode {
            name: "Citation".into(),
            template: "<Citation />".into(),
            imports: Vec::new(),
        });
        entry.field = field;
        let options = CiteOptions {
            custom: ComponentSet::from_entries([entry]),
            ..CiteOptions::default()
        };

        // The bare content carries the locator, the tick the reference (§5.2
        // orders the contents bare → quote → tick).
        let result = process(&events_with_text("[^p. 1 `book`]"), &options);
        assert!(result.iter().any(|event| matches!(
            event,
            Event::Attribute { name, value } if name == "reference_id" && value == "book"
        )));
        assert!(result.iter().any(|event| matches!(
            event,
            Event::Attribute { name, value } if name == "locator" && value == "p. 1"
        )));
        assert!(result.iter().any(|event| matches!(
            event,
            Event::Attribute { name, value } if name == "citation_number" && value == "1"
        )));
        assert!(
            !result.iter().any(|event| matches!(
                event,
                Event::Diagnostic { message, .. }
                    if message.contains("clash") || message.contains("reference")
            )),
            "no diagnostic expected: {result:?}"
        );
    }

    /// D-4: an override renames a field for public consumption only — the
    /// processor still numbers off `reference_id` / `locator`.
    #[test]
    fn an_override_does_not_break_numbering() {
        let mut field = pendon_extra::FieldConfig::new();
        field
            .add_override(
                pendon_extra::Path::from_dotted("reference_id").expect("valid path"),
                pendon_extra::Path::from_dotted("ref_id").expect("valid path"),
            )
            .expect("one override");
        field
            .add_override(
                pendon_extra::Path::from_dotted("citation_number").expect("valid path"),
                pendon_extra::Path::from_dotted("no").expect("valid path"),
            )
            .expect("unique target");
        let mut entry = TypedComponent::default_component(CiteCustomNode {
            name: "Citation".into(),
            template: "<Citation />".into(),
            imports: Vec::new(),
        });
        entry.field = field;
        let options = CiteOptions {
            custom: ComponentSet::from_entries([entry]),
            ..CiteOptions::default()
        };

        let result = process(&events_with_text("[^book] and [^book \"p. 1\"]"), &options);
        // The node still speaks the internal names (the renderer renames them).
        assert!(result.iter().any(|event| matches!(
            event,
            Event::Attribute { name, value } if name == "reference_id" && value == "book"
        )));
        let citations = result
            .iter()
            .filter(
                |event| matches!(event, Event::Attribute { name, .. } if name == "citation_number"),
            )
            .count();
        assert_eq!(citations, 2, "both citations were numbered");
    }

    /// D-12 rule 5: `citation_number` is the processor's own field, so an extras
    /// item that spells it is an `Error` — it never silently overwrites the
    /// ordinal (`field.override.citation_number` publishes the ordinal under
    /// another name instead, D-4).
    #[test]
    fn an_extras_item_cannot_write_the_citation_number() {
        let result = process(
            &events_with_text(r#"[^book]@@cite{citation_number: "9", note: "n"}"#),
            &CiteOptions::default(),
        );
        assert!(result.iter().any(|event| matches!(
            event,
            Event::Diagnostic { severity: Severity::Error, message, .. }
                if message.contains("citation_number")
        )));
        // The processor's number still wins.
        let html = html_text(&result);
        assert!(html.contains("[1]</a>"), "{html}");
    }

    /// §7.3: `reference_id` is mandatory. A head that leaves it unassigned is an
    /// `Error` naming the cause, and the text stays as written.
    #[test]
    fn a_head_without_a_reference_id_is_an_error() {
        // `field.bracket.bare = ""` unassigns the preset (OBJ-10), so the head
        // carries a value no field claims.
        let mut field = pendon_extra::FieldConfig::new();
        field.assign_slot(Slot::Bracket, SlotKind::Bare, None);
        let mut entry = TypedComponent::default_component(CiteCustomNode {
            name: "Citation".into(),
            template: "<Citation />".into(),
            imports: Vec::new(),
        });
        entry.field = field;
        let options = CiteOptions {
            custom: ComponentSet::from_entries([entry]),
            ..CiteOptions::default()
        };

        let result = process(&events_with_text("[^book] tail"), &options);
        assert!(result.iter().any(|event| matches!(
            event,
            Event::Diagnostic { message, .. }
                if message.contains("[cite]") && message.contains("reference_id")
        )));
        assert!(result.iter().any(|event| matches!(
            event,
            Event::Text(text) if text.contains("[^book]")
        )));
        assert!(!result
            .iter()
            .any(|event| matches!(event, Event::Attribute { name, .. } if name == "__layer")));
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
            Event::Text(r#"[^book]{#short}"#.into()),
            Event::EndNode(NodeKind::Paragraph),
            Event::EndNode(NodeKind::Document),
        ];

        let result = process(&events, &options);
        assert!(result.iter().any(|event| matches!(
            event,
            Event::Attribute { name, value } if name == "reference_id" && value == "book"
        )));
        assert!(result.iter().any(|event| matches!(
            event,
            Event::Attribute { name, value } if name == "cite-id" && value == "short"
        )));
    }
}
