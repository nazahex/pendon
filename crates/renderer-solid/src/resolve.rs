//! §11 `type` resolution — decided once per document, not once per element.
//!
//! ADR-0008 (`D-3a`) moved the decision out of the element plugins: a plugin
//! emits its built-in node, tags it with the layer it belongs to (`__layer`) and
//! keeps the `type` marker attribute it already emitted. This module answers,
//! for the whole document at once, which entry of each layer answers each
//! *occurring* type — so `_` (the remainder) and `*` (everything) expand against
//! the document's real type set instead of a single element's marker.
//!
//! A **layer** is one component set of one element section (§11 rule 1):
//! `heading`, `anchor`, `figure`, `img`, `table`, `row`, `cell`, … The plugin
//! names it in the internal `__layer` attribute, which every renderer strips
//! (`__`-prefixed keys are internal markers, §11 rule 3). Components that are
//! not §11 layers (`plugin-custom`'s `toml:` specs, `plugin-vicado`) keep the
//! older identity: the AST node type plus the `name` attribute.

use std::collections::{BTreeMap, BTreeSet};

use pendon_types::{Group, Rule, TypeId, TypeSet};
use serde_json::{Map, Value};

use crate::imports::{ComponentTemplate, SolidRenderHints};

/// Everything `render_node` needs besides the node itself.
#[derive(Clone, Copy)]
pub(crate) struct Ctx<'a> {
    pub hints: Option<&'a SolidRenderHints>,
    pub resolution: &'a Resolution,
}

/// What a component set answers.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum LayerKey {
    /// A §11 layer, named by the element plugin in `__layer`.
    Layer(String),
    /// A node identity: the AST node type and, when the node carries one, the
    /// `name` attribute.
    Identity(String, Option<String>),
}

/// One layer's resolved winners and its default slot.
#[derive(Clone, Debug, Default)]
struct Layer {
    /// Occurring identity (marker name; `None` = untyped) → winning template.
    winners: BTreeMap<Option<String>, usize>,
    /// The slot for identities no rule covers (§11 rule 3).
    default: Option<usize>,
}

/// The document-wide answer: one entry per layer, plus the templates that won
/// at least one occurring type (the only ones whose imports are emitted).
#[derive(Clone, Debug, Default)]
pub(crate) struct Resolution {
    layers: BTreeMap<LayerKey, Layer>,
    used: BTreeSet<usize>,
}

impl Resolution {
    /// Resolves every layer of `hints` against the types `ast` occurs.
    ///
    /// Occurring types are collected from the whole JSON tree — including
    /// `$fragment` payloads nested in attribute values (a bound `mdp` body is
    /// rendered as a layer like any other).
    pub(crate) fn build(hints: Option<&SolidRenderHints>, ast: &Value) -> Self {
        let Some(hints) = hints else {
            return Self::default();
        };

        let mut grouped: BTreeMap<LayerKey, Vec<usize>> = BTreeMap::new();
        for (index, template) in hints.templates.iter().enumerate() {
            grouped.entry(layer_key(template)).or_default().push(index);
        }
        let mut occurring: BTreeMap<LayerKey, TypeSet> = BTreeMap::new();
        collect_types(ast, &grouped, &mut occurring);

        let mut resolution = Self::default();
        for (key, indices) in grouped {
            let types = occurring.remove(&key).unwrap_or_default();
            let default = indices
                .iter()
                .copied()
                .find(|index| hints.templates[*index].selectors.is_empty());
            let rules: Vec<Rule<usize, ()>> = indices
                .iter()
                .filter(|index| !hints.templates[**index].selectors.is_empty())
                .map(|index| Rule::new(hints.templates[*index].selectors.clone(), (), *index))
                .collect();
            let group = Group::new(rules);

            let mut layer = Layer {
                winners: BTreeMap::new(),
                default,
            };
            for id in types.occurring() {
                match group.winner(&types, id) {
                    Some(rule) => {
                        layer.winners.insert(identity(id), rule.payload);
                        resolution.used.insert(rule.payload);
                    }
                    // §11 rule 3: an identity no rule names falls to the layer
                    // default, so its imports are needed.
                    None => {
                        if let Some(default) = default {
                            resolution.used.insert(default);
                        }
                    }
                }
            }
            resolution.layers.insert(key, layer);
        }
        resolution
    }

    /// The layer `kind`/`attrs` belongs to, when a component set answers it.
    pub(crate) fn key_for<'a>(
        &'a self,
        kind: &str,
        attrs: Option<&Map<String, Value>>,
    ) -> Option<&'a LayerKey> {
        key_in(&self.layers, kind, attrs)
    }

    /// The winning template of `key` for `marker` (`None` = untyped).
    pub(crate) fn select(&self, key: &LayerKey, marker: Option<&str>) -> Option<usize> {
        let layer = self.layers.get(key)?;
        let identity = marker.map(str::to_string);
        layer.winners.get(&identity).copied().or(layer.default)
    }

    /// The templates that won at least one occurring type (§11 rule 3). Only
    /// these contribute imports; a losing entry's `import` line never appears.
    pub(crate) fn used(&self) -> &BTreeSet<usize> {
        &self.used
    }
}

/// The set a template answers: its layer name, else its node identity.
fn layer_key(template: &ComponentTemplate) -> LayerKey {
    match &template.layer {
        Some(layer) => LayerKey::Layer(layer.clone()),
        None => LayerKey::Identity(template.node_type.clone(), template.node_name.clone()),
    }
}

/// The identity key of an occurring type inside its layer (`None` = untyped).
fn identity(id: TypeId<'_>) -> Option<String> {
    match id {
        TypeId::Literal(name) => Some(name.to_string()),
        TypeId::Untyped => None,
    }
}

/// Looks `(kind, attrs)` up in `layers`, honouring the exact-name preference: a
/// node whose `name` some set declares belongs to that set, otherwise the
/// generic (`node_name = None`) set of the same node type applies.
fn key_in<'a, V>(
    layers: &'a BTreeMap<LayerKey, V>,
    kind: &str,
    attrs: Option<&Map<String, Value>>,
) -> Option<&'a LayerKey> {
    if let Some(layer) = attr_str(attrs, "__layer") {
        if let Some((key, _)) = layers.get_key_value(&LayerKey::Layer(layer.to_string())) {
            return Some(key);
        }
        // An unregistered layer is no layer at all: the node may still answer a
        // `[[plugin]]` identity template of the same node type.
    }
    if let Some(name) = attr_str(attrs, "name") {
        let exact = LayerKey::Identity(kind.to_string(), Some(name.to_string()));
        if let Some((key, _)) = layers.get_key_value(&exact) {
            return Some(key);
        }
    }
    layers
        .get_key_value(&LayerKey::Identity(kind.to_string(), None))
        .map(|(key, _)| key)
}

/// Adds one occurring type per layer-tagged node of the tree.
fn collect_types<V>(
    value: &Value,
    layers: &BTreeMap<LayerKey, V>,
    out: &mut BTreeMap<LayerKey, TypeSet>,
) {
    match value {
        Value::Object(map) => {
            if let Some(kind) = map.get("type").and_then(Value::as_str) {
                let attrs = map.get("attrs").and_then(Value::as_object);
                if let Some(key) = key_in(layers, kind, attrs) {
                    let types = out.entry(key.clone()).or_default();
                    match attr_str(attrs, "type") {
                        Some(marker) => types.insert(marker),
                        None => types.insert_untyped(),
                    }
                }
            }
            // Attribute values may hold a nested document (`$fragment`); its
            // nodes are layers too, so the walk does not stop at `children`.
            for child in map.values() {
                collect_types(child, layers, out);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_types(item, layers, out);
            }
        }
        _ => {}
    }
}

/// One string attribute of a node.
fn attr_str<'a>(attrs: Option<&'a Map<String, Value>>, name: &str) -> Option<&'a str> {
    attrs?.get(name)?.as_str()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pendon_types::Selector;
    use serde_json::json;

    fn template(layer: &str, selectors: Vec<Selector>, name: &str) -> ComponentTemplate {
        ComponentTemplate {
            layer: Some(layer.to_string()),
            node_type: "Heading".to_string(),
            node_name: None,
            selectors,
            template: format!("<{name} />"),
            imports: Vec::new(),
            field: None,
        }
    }

    fn hints(templates: Vec<ComponentTemplate>) -> SolidRenderHints {
        SolidRenderHints {
            templates,
            ..SolidRenderHints::default()
        }
    }

    /// A tagged node of the document: `__layer` names the layer, `type` the
    /// marker.
    fn node(layer: &str, marker: Option<&str>) -> Value {
        let mut attrs = json!({ "__layer": layer });
        if let Some(marker) = marker {
            attrs["type"] = json!(marker);
        }
        json!({ "type": "Heading", "attrs": attrs })
    }

    fn doc(nodes: Vec<Value>) -> Value {
        json!({ "type": "Document", "children": nodes })
    }

    #[test]
    fn literal_beats_the_default_and_the_default_takes_the_rest() {
        let hints = hints(vec![
            template("heading", vec![Selector::Exact("TypeA".into())], "A"),
            template("heading", Vec::new(), "Default"),
        ]);
        let ast = doc(vec![node("heading", Some("TypeA")), node("heading", None)]);
        let resolution = Resolution::build(Some(&hints), &ast);
        let key = LayerKey::Layer("heading".to_string());

        assert_eq!(resolution.select(&key, Some("TypeA")), Some(0));
        assert_eq!(resolution.select(&key, None), Some(1));
        assert_eq!(resolution.select(&key, Some("TypeB")), Some(1));
    }

    /// `_` is the remainder: it must see the types of *other* elements of the
    /// same layer, which is exactly what a plugin cannot know on its own.
    #[test]
    fn underscore_sees_the_whole_document() {
        let hints = hints(vec![
            template("heading", vec![Selector::Exact("TypeA".into())], "A"),
            template("heading", vec![Selector::Underscore], "Rest"),
        ]);
        let ast = doc(vec![
            node("heading", Some("TypeA")),
            node("heading", Some("TypeB")),
            node("heading", None),
        ]);
        let resolution = Resolution::build(Some(&hints), &ast);
        let key = LayerKey::Layer("heading".to_string());

        assert_eq!(resolution.select(&key, Some("TypeA")), Some(0));
        assert_eq!(resolution.select(&key, Some("TypeB")), Some(1));
        // `untyped` is not defined by `_` either: it is part of the remainder.
        assert_eq!(resolution.select(&key, None), Some(1));
        assert!(resolution.used().contains(&0));
        assert!(resolution.used().contains(&1));
    }

    /// A losing entry never contributes imports (§11 rule 3): only the winner
    /// of an occurring type is "used".
    #[test]
    fn losing_entries_are_not_used() {
        let hints = hints(vec![
            template("heading", vec![Selector::Exact("TypeA".into())], "A"),
            template("heading", vec![Selector::Exact("TypeB".into())], "B"),
        ]);
        let ast = doc(vec![node("heading", Some("TypeA"))]);
        let resolution = Resolution::build(Some(&hints), &ast);

        assert_eq!(
            resolution.used().iter().copied().collect::<Vec<_>>(),
            vec![0]
        );
    }

    /// The layer tag is what a §11 plugin emits; an untagged node keeps the
    /// node-identity behaviour `plugin-custom` relies on.
    #[test]
    fn identity_templates_still_match_a_custom_node() {
        let custom = ComponentTemplate {
            layer: None,
            node_type: "Epis".to_string(),
            node_name: Some("Epis".to_string()),
            selectors: Vec::new(),
            template: "<Epis />".to_string(),
            imports: Vec::new(),
            field: None,
        };
        let hints = hints(vec![custom]);
        let ast = doc(vec![json!({
            "type": "Epis",
            "attrs": { "name": "Epis" },
        })]);
        let resolution = Resolution::build(Some(&hints), &ast);

        let attrs = json!({ "name": "Epis" });
        let key = resolution
            .key_for("Epis", attrs.as_object())
            .expect("the node identity is a layer");
        assert_eq!(resolution.select(key, None), Some(0));
    }

    /// `*` (1) loses to a literal (2) for a named identity, and answers the
    /// untyped one only when `?` does not.
    #[test]
    fn question_and_star_expand_against_the_occurring_set() {
        let hints = hints(vec![
            template("heading", vec![Selector::Question], "Untyped"),
            template("heading", vec![Selector::Star], "All"),
            template("heading", vec![Selector::Exact("TypeB".into())], "B"),
        ]);
        let ast = doc(vec![node("heading", None), node("heading", Some("TypeB"))]);
        let resolution = Resolution::build(Some(&hints), &ast);
        let key = LayerKey::Layer("heading".to_string());

        assert_eq!(resolution.select(&key, Some("TypeB")), Some(2));
        assert_eq!(resolution.select(&key, None), Some(0));
    }

    #[test]
    fn a_document_without_nodes_resolves_nothing() {
        let hints = hints(vec![template("heading", vec![Selector::Star], "All")]);
        let ast = doc(Vec::new());
        let resolution = Resolution::build(Some(&hints), &ast);

        assert!(resolution.used().is_empty());
        let key = LayerKey::Layer("heading".to_string());
        assert_eq!(resolution.select(&key, None), None);
    }
}
