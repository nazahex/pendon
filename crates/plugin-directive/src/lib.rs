//! §10.2/§10.3: the `::type…::` inline and `==type…==` block directives.
//!
//! A directive is a *construct container*: the head names a `type` (mandatory,
//! §10.2/§10.3) plus the usual `[…]` / `(…)` / `@@type{…}` extras, and the
//! content between the fences is parsed as inline (colon sigil) or block (equal
//! sigil) content.
//!
//! This plugin runs **before** `plugin-markdown` (§12.1). It turns every
//! directive into a `Custom` node carrying:
//!
//! * `__plugin_kind` = `inline` / `block`, so `plugin-markdown` re-lexes the
//!   content in the matching mode (§11 rule 3 placement);
//! * `type`, the routing key of the §11 component set, exactly like
//!   `plugin-marker` (§11 rule 3);
//! * `name`, the selected component (the node name the renderer matches on);
//! * every extra of the head (§6.3), flags included.
//!
//! The `directive` layer is the plugin's primary layer, so
//! `[[task.directive.custom]]` addresses it (§11).
//!
//! When no component claims a type the node is named after the directive `type`
//! itself and the renderer falls back to its child-rendering path (§11 rule 3).
//!
//! Code fences, inline code and raw HTML keep their text: a `::` / `==` inside
//! them is literal (§4.3, same rule as every other construct plugin).

use pendon_core::{Event, NodeKind, Severity};
use pendon_extra::{
    parse_extras, to_attributes, warning_message, AttrValue, Attrs, DirectiveHead, ExtrasAttr,
    ExtrasMatch, ExtrasOptions,
};
use pendon_renderer_solid::{ComponentSet, ComponentTemplate, ImportEntry, SolidRenderHints};
use serde::{Deserialize, Serialize};

mod block;
mod inline;

/// §11 primary layer of `plugin-directive`: the node a directive renders.
const DIRECTIVE_LAYER: &str = "directive";
/// §10.2: key the head `[…]` slot maps to.
const BRACKET_KEY: &str = "slug";
/// §10.2: key the head `("…")` slot maps to.
const PARENTHESES_KEY: &str = "title";

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DirectiveCustomNode {
    pub name: String,
    pub template: String,
    #[serde(default)]
    pub imports: Vec<ImportEntry>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DirectiveOptions {
    /// §11 component set of the `directive` layer: typed entries plus at most
    /// one default, selected by the directive `type` (§11 rule 3).
    pub custom: ComponentSet<DirectiveCustomNode>,
}

/// One parsed directive: its mandatory type and the attributes of its head
/// (including the optional adjacent extras head).
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct ParsedDirective {
    pub(crate) type_name: String,
    pub(crate) attrs: Vec<(String, ExtrasAttr)>,
    pub(crate) warnings: Vec<String>,
}

/// §10.2/§10.3: preprocess inline and block directives into `Custom` nodes.
///
/// The block pass runs first so a block body is collected as a whole; the inline
/// pass then walks the result, so `::…::` inside a block body is bound as well.
pub fn process(events: &[Event], options: &DirectiveOptions) -> Vec<Event> {
    let blocked = block::process(events, options);
    inline::process(&blocked, options)
}

pub fn solid_hints(options: &DirectiveOptions) -> Option<SolidRenderHints> {
    if options.custom.is_empty() {
        return None;
    }
    let mut hints = SolidRenderHints::default();
    // §11 rule 3: every entry answers the directive types it declares.
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

/// The layer key `[[task.directive.custom]]` addresses (always the primary).
pub fn primary_layer() -> &'static str {
    DIRECTIVE_LAYER
}

/// Nodes whose text is copied verbatim (code, raw HTML): a `::` / `==` inside
/// them is literal text.
pub(crate) fn is_verbatim(kind: &NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::CodeFence | NodeKind::InlineCode | NodeKind::HtmlBlock | NodeKind::HtmlInline
    )
}

/// Resolves a directive head into attributes.
///
/// The `[…]` slot maps to `bracket_key` (`slug`) and the `("…")` slot to
/// `parentheses_key` (`title`), both winning over an extras head that sets the
/// same key (§10.4). Returns the parsed directive and how many bytes of `rest`
/// the adjacent extras head consumed (zero when there is none, §4.1/§4.3).
pub(crate) fn resolve_head(head: &DirectiveHead, rest: &str) -> (ParsedDirective, usize) {
    let mut parsed = ParsedDirective {
        type_name: head.type_marker.clone().unwrap_or_default(),
        ..ParsedDirective::default()
    };

    let mut head_attrs = Attrs::default();
    if let Some(bracket) = &head.bracket {
        head_attrs.push(
            BRACKET_KEY,
            ExtrasAttr::Value(AttrValue::Str(bracket.clone())),
        );
    }
    if let Some(parentheses) = &head.parentheses {
        head_attrs.push(
            PARENTHESES_KEY,
            ExtrasAttr::Value(AttrValue::Str(parentheses.clone())),
        );
    }

    let mut consumed = 0usize;
    let extras = match parse_extras(rest) {
        ExtrasMatch::Head {
            head: extras_head,
            rest: after,
        } => {
            consumed = rest.len() - after.len();
            to_attributes(&extras_head, &ExtrasOptions::default())
        }
        ExtrasMatch::Malformed { error, .. } => {
            // §4.3: a malformed head is never dropped nor partially applied.
            parsed.warnings.push(format!(
                "the extras head is malformed ({error:?}); it stays literal text (§4.3)"
            ));
            Attrs::default()
        }
        ExtrasMatch::Absent { .. } => Attrs::default(),
    };

    let merged = head_attrs.merge_with(extras, &[]);
    parsed.attrs = merged.items;
    for warning in &merged.warnings {
        parsed.warnings.push(warning_message(warning));
    }
    (parsed, consumed)
}

/// Emits the `Custom` node of one directive (§10.2/§10.3, §11 rule 3).
pub(crate) fn emit_directive(
    parsed: &ParsedDirective,
    is_block: bool,
    options: &DirectiveOptions,
    children: &[Event],
    out: &mut Vec<Event>,
) {
    for message in &parsed.warnings {
        out.push(Event::Diagnostic {
            severity: Severity::Warning,
            message: format!("[directive] {message}"),
            span: None,
        });
    }

    let custom = options.custom.select(Some(&parsed.type_name));
    let fallback = if is_block { "div" } else { "span" };
    let node = match &custom {
        Some(custom) => NodeKind::Custom(custom.name.clone()),
        // §10.2/§10.3 + D8: an unclaimed/default-less type renders the built-in
        // element, exactly like a marker, so no extras are dropped (§6).
        None => NodeKind::Element(fallback.to_string()),
    };

    out.push(Event::StartNode(node.clone()));
    // First attribute so `plugin-markdown`'s placement look-ahead sees it.
    out.push(Event::Attribute {
        name: "__plugin_kind".to_string(),
        value: if is_block { "block" } else { "inline" }.to_string(),
    });
    out.push(Event::Attribute {
        name: "name".to_string(),
        value: match &custom {
            Some(custom) => custom.name.clone(),
            None => fallback.to_string(),
        },
    });
    out.push(Event::Attribute {
        name: "type".to_string(),
        value: parsed.type_name.clone(),
    });
    push_attrs(parsed, out);
    out.extend(children.iter().cloned());
    out.push(Event::EndNode(node));
}

/// The node's extras attributes, bare flags included (§6.3).
fn push_attrs(parsed: &ParsedDirective, out: &mut Vec<Event>) {
    for (key, value) in &parsed.attrs {
        match value {
            ExtrasAttr::Flag => out.push(Event::AttributeFlag { name: key.clone() }),
            ExtrasAttr::Value(value) => out.push(Event::Attribute {
                name: key.clone(),
                value: value.literal(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pendon_core::{parse, Options};
    use pendon_renderer_solid::TypedComponent;

    pub(crate) fn component(name: &str) -> DirectiveCustomNode {
        DirectiveCustomNode {
            name: name.to_string(),
            template: format!("<{name}>{{children}}</{name}>"),
            imports: Vec::new(),
        }
    }

    pub(crate) fn typed(types: &[&str], name: &str) -> TypedComponent<DirectiveCustomNode> {
        TypedComponent::typed(types.to_vec(), component(name))
    }

    pub(crate) fn options() -> DirectiveOptions {
        DirectiveOptions {
            custom: ComponentSet::from_entries([
                typed(&["note"], "Note"),
                TypedComponent::default_component(component("DirectiveDefault")),
            ]),
        }
    }

    pub(crate) fn run(src: &str) -> Vec<Event> {
        run_with(src, &options())
    }

    pub(crate) fn run_with(src: &str, options: &DirectiveOptions) -> Vec<Event> {
        let events = parse(src, &Options::default());
        process(&events, options)
    }

    pub(crate) fn attr(events: &[Event], name: &str) -> Option<String> {
        events.iter().find_map(|event| match event {
            Event::Attribute { name: n, value } if n == name => Some(value.clone()),
            _ => None,
        })
    }

    pub(crate) fn all_attrs(events: &[Event], name: &str) -> Vec<String> {
        events
            .iter()
            .filter_map(|event| match event {
                Event::Attribute { name: n, value } if n == name => Some(value.clone()),
                _ => None,
            })
            .collect()
    }

    pub(crate) fn customs(events: &[Event]) -> Vec<String> {
        events
            .iter()
            .filter_map(|event| match event {
                Event::StartNode(NodeKind::Custom(name)) => Some(name.clone()),
                _ => None,
            })
            .collect()
    }

    pub(crate) fn kinds(events: &[Event]) -> Vec<String> {
        all_attrs(events, "__plugin_kind")
    }

    pub(crate) fn text_of(events: &[Event]) -> String {
        events
            .iter()
            .filter_map(|event| match event {
                Event::Text(text) => Some(text.as_str()),
                _ => None,
            })
            .collect()
    }

    pub(crate) fn warnings(events: &[Event]) -> Vec<String> {
        events
            .iter()
            .filter_map(|event| match event {
                Event::Diagnostic { message, .. } => Some(message.clone()),
                _ => None,
            })
            .collect()
    }

    /// §10.2: an inline directive becomes a `Custom` node of its `type` with the
    /// surrounding text kept inline.
    #[test]
    fn inline_directive_becomes_a_custom_node() {
        let out = run("Before ::note some text:: after\n");
        assert_eq!(customs(&out), vec!["Note".to_string()]);
        assert_eq!(kinds(&out), vec!["inline".to_string()]);
        assert_eq!(attr(&out, "type").as_deref(), Some("note"));
        let text = text_of(&out);
        assert!(text.contains("Before"));
        assert!(text.contains("some text"));
        assert!(text.contains("after"));
        assert!(!text.contains("::"));
    }

    /// §10.2/§10.4: `[bracket]` → `slug`, `("title")` → `title`, and the extras
    /// head contributes the rest.
    #[test]
    fn inline_head_slots_and_extras_map_to_attributes() {
        let out = run("::note[slug-1](\"A title\")@@note{.box, #i, isOpen} body::\n");
        assert_eq!(attr(&out, "slug").as_deref(), Some("slug-1"));
        assert_eq!(attr(&out, "title").as_deref(), Some("A title"));
        assert_eq!(attr(&out, "id").as_deref(), Some("i"));
        assert_eq!(attr(&out, "class").as_deref(), Some("box"));
        assert!(out
            .iter()
            .any(|event| matches!(event, Event::AttributeFlag { name } if name == "isOpen")));
    }

    /// §10.3: a block directive consumes its whole body and becomes a `block`
    /// `Custom` node (no `<p>` wrapper around the fences).
    #[test]
    fn block_directive_collects_its_body() {
        let out = run("==note\nfirst para\n\nsecond para\n==\n");
        assert_eq!(customs(&out), vec!["Note".to_string()]);
        assert_eq!(kinds(&out), vec!["block".to_string()]);
        assert_eq!(attr(&out, "type").as_deref(), Some("note"));
        let text = text_of(&out);
        assert!(text.contains("first para"), "{text:?}");
        assert!(text.contains("second para"), "{text:?}");
        assert!(!text.contains("=="), "{text:?}");
    }

    /// §10.3: a fence without a type cannot open; a bare fence with nothing open
    /// stays literal text.
    #[test]
    fn bare_and_untyped_fences_stay_literal() {
        let out = run("==\ntext\n");
        assert!(customs(&out).is_empty());
        assert!(text_of(&out).contains("=="));

        let out = run("==5\nbody\n");
        assert!(customs(&out).is_empty(), "{out:?}");
        assert!(text_of(&out).contains("==5"));
    }

    /// §10.3: an unclosed block directive is closed implicitly with a warning.
    #[test]
    fn unclosed_block_warns_and_still_binds() {
        let out = run("==note\nbody\n");
        assert_eq!(customs(&out), vec!["Note".to_string()]);
        assert!(warnings(&out)
            .iter()
            .any(|message| message.contains("unclosed block directive")));
    }

    /// §10.3: fences nest LIFO; a bare fence closes the innermost open directive.
    #[test]
    fn block_directives_nest() {
        let out = run("==outer\n==note\nbody\n==\ntail\n==\n");
        // The outer node opens first; the inner one nests inside it.
        assert_eq!(
            customs(&out),
            vec!["DirectiveDefault".to_string(), "Note".to_string()],
            "{out:?}"
        );
        assert_eq!(kinds(&out), vec!["block".to_string(), "block".to_string()]);
        let text = text_of(&out);
        assert!(text.contains("body"));
        assert!(text.contains("tail"));
    }

    /// §10.2: inline directives nest, and the closing run must be at least as
    /// long as the opening run.
    #[test]
    fn inline_directives_nest() {
        let out = run("::outer a ::note b:: c::\n");
        assert_eq!(
            customs(&out),
            vec!["DirectiveDefault".to_string(), "Note".to_string()]
        );
        let text = text_of(&out);
        assert!(text.contains('a') && text.contains('c'));
    }

    /// §10.2: a `::name` with no closing run stays literal text.
    #[test]
    fn unterminated_inline_stays_literal() {
        let out = run("::note no close here\n");
        assert!(customs(&out).is_empty(), "{out:?}");
        assert!(text_of(&out).contains("::note"));
    }

    /// A default component answers every unclaimed type (§11 rule 3).
    #[test]
    fn the_type_routes_the_component_set() {
        let out = run("==unknown\nbody\n==\n");
        assert_eq!(customs(&out), vec!["DirectiveDefault".to_string()]);
        assert_eq!(attr(&out, "type").as_deref(), Some("unknown"));
    }

    /// A layer without a default falls back to the built-in element with its
    /// own `type` (§10.2/§10.3, D8).
    #[test]
    fn an_unclaimed_type_falls_back_to_the_builtin_element() {
        let options = DirectiveOptions {
            custom: ComponentSet::from_entries([typed(&["note"], "Note")]),
        };
        let out = run_with("==aside\nbody\n==\n", &options);
        assert!(customs(&out).is_empty(), "{out:?}");
        // Block directives render `<div>`, inline ones `<span>`.
        assert!(out.iter().any(|event| matches!(
            event,
            Event::StartNode(NodeKind::Element(name)) if name == "div"
        )));
        assert_eq!(attr(&out, "type").as_deref(), Some("aside"));
        assert!(text_of(&out).contains("body"));

        let out = run_with("::aside body::\n", &options);
        assert!(out.iter().any(|event| matches!(
            event,
            Event::StartNode(NodeKind::Element(name)) if name == "span"
        )));
        assert_eq!(attr(&out, "type").as_deref(), Some("aside"));
    }

    /// A code fence keeps a directive-looking text literal (§4.3).
    ///
    /// Inline backtick spans are not guarded: the core parser does not lex them
    /// (that is `plugin-markdown`'s job, and this plugin runs before it), the
    /// same limitation `plugin-marker` has.
    #[test]
    fn fenced_code_keeps_its_text() {
        let out = run("```\n==note\nbody\n==\n```\n");
        assert!(customs(&out).is_empty(), "{out:?}");
        assert!(all_attrs(&out, "type").is_empty());
        assert!(text_of(&out).contains("==note"));
    }

    /// Every entry of the set needs a template, typed or not.
    #[test]
    fn hints_cover_every_entry() {
        let hints = solid_hints(&options()).expect("hints");
        assert_eq!(hints.templates.len(), 2);
        assert!(solid_hints(&DirectiveOptions::default()).is_none());
    }

    #[test]
    fn primary_layer_matches_the_config_key() {
        assert_eq!(primary_layer(), DIRECTIVE_LAYER);
    }
}
