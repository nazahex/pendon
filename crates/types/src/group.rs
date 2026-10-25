//! Rules, definition groups, and the two resolution entry points.
//!
//! A **definition group** is the smallest set of rules that can override each
//! other: one element section of one task (decision D-2). `defined` is computed
//! from the whole group *before* any resolution, so the meaning of `_` never
//! depends on registration order.

use std::collections::BTreeSet;

use crate::selector::Selector;
use crate::type_set::{TypeId, TypeSet};

/// One rule: the selectors it answers, its payload (component, number block,
/// extract block — `pendon-types` never interprets it) and the destination it
/// writes to (§6).
///
/// A rule contributes **one target per destination it names**; a producer with
/// two destinations registers two rules with the same selectors and payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule<P, T> {
    pub selectors: Vec<Selector>,
    pub payload: P,
    pub target: T,
}

impl<P, T> Rule<P, T> {
    /// A rule with the given selectors, destination and payload.
    pub fn new(selectors: Vec<Selector>, target: T, payload: P) -> Self {
        Self {
            selectors,
            payload,
            target,
        }
    }

    /// The literal marker names this rule selects (wildcards contribute none).
    pub fn literal_types(&self) -> impl Iterator<Item = &str> {
        self.selectors.iter().filter_map(Selector::literal)
    }
}

/// What the whole group *names*, computed once (§2.1).
///
/// `names` = every literal any rule selects; `untyped` = some rule wrote `?`.
/// `_` and `*` add nothing here — that is what makes two `_` rules see the same
/// remainder.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Definition {
    names: BTreeSet<String>,
    untyped: bool,
}

impl Definition {
    /// Collects `defined` from every rule of the group, before any resolution.
    pub fn of<P, T>(rules: &[Rule<P, T>]) -> Self {
        let mut names = BTreeSet::new();
        let mut untyped = false;
        for rule in rules {
            for selector in &rule.selectors {
                match selector {
                    Selector::Exact(name) => {
                        names.insert(name.clone());
                    }
                    Selector::Question => untyped = true,
                    Selector::Underscore | Selector::Star => {}
                }
            }
        }
        Self { names, untyped }
    }

    /// The literal names any rule of the group selected.
    pub fn names(&self) -> &BTreeSet<String> {
        &self.names
    }

    /// Did any rule write `?`? If so, `untyped` is defined and `_` excludes it.
    pub fn defines_untyped(&self) -> bool {
        self.untyped
    }

    /// Is `id` defined by the group (i.e. excluded from `_`)?
    pub fn contains(&self, id: TypeId<'_>) -> bool {
        match id {
            TypeId::Literal(name) => self.names.contains(name),
            TypeId::Untyped => self.untyped,
        }
    }
}

/// The specificity of `rule` for `id`, or `None` when it does not cover it.
///
/// Specificity is **per selector**: the max over the selectors of the rule that
/// match, so `["*", "TypeB"]` still scores 2 for `TypeB`.
fn score<P, T>(
    rule: &Rule<P, T>,
    types: &TypeSet,
    defined: &Definition,
    id: TypeId<'_>,
) -> Option<u8> {
    rule.selectors
        .iter()
        .filter_map(|selector| match selector {
            Selector::Exact(name) => match id {
                TypeId::Literal(id_name) if id_name == name => Some(selector.specificity()),
                _ => None,
            },
            Selector::Question => id.is_untyped().then(|| selector.specificity()),
            Selector::Underscore => {
                (types.contains(id) && !defined.contains(id)).then(|| selector.specificity())
            }
            Selector::Star => types.contains(id).then(|| selector.specificity()),
        })
        .max()
}

/// Exclusive consumers: the rule that owns `id`, or `None` when no rule covers
/// it (the caller then applies its default — a layer default component, a
/// built-in element, a skipped block).
///
/// Highest specificity wins; equal specificity is broken by **later
/// registration**. When a group has several destinations, use [`matching`]
/// instead — this answers "the best rule overall".
pub fn winner<'a, P, T>(
    rules: &'a [Rule<P, T>],
    types: &TypeSet,
    id: TypeId<'_>,
) -> Option<&'a Rule<P, T>> {
    let defined = Definition::of(rules);
    winner_with_definition(rules, types, &defined, id)
}

fn winner_with_definition<'a, P, T>(
    rules: &'a [Rule<P, T>],
    types: &TypeSet,
    defined: &Definition,
    id: TypeId<'_>,
) -> Option<&'a Rule<P, T>> {
    let mut best: Option<(u8, &'a Rule<P, T>)> = None;
    for rule in rules {
        let Some(value) = score(rule, types, defined, id) else {
            continue;
        };
        // `>=` (not `>`) is the tie-break: equal specificity means the later
        // rule wins, so a same-specificity rule always replaces the earlier one.
        if best
            .map(|(best_score, _)| value >= best_score)
            .unwrap_or(true)
        {
            best = Some((value, rule));
        }
    }
    best.map(|(_, rule)| rule)
}

/// Additive consumers: every rule that owns a destination for `id`, one entry
/// per destination, in registration order.
///
/// Two rules naming the same destination are the §2.3 override: the better one
/// (specificity, then later registration) is yielded and the other disappears,
/// so extraction never writes the same file twice.
pub fn matching<'a, P, T: PartialEq>(
    rules: &'a [Rule<P, T>],
    types: &TypeSet,
    id: TypeId<'_>,
) -> Vec<&'a Rule<P, T>> {
    let defined = Definition::of(rules);
    matching_with_definition(rules, types, &defined, id)
}

fn matching_with_definition<'a, P, T: PartialEq>(
    rules: &'a [Rule<P, T>],
    types: &TypeSet,
    defined: &Definition,
    id: TypeId<'_>,
) -> Vec<&'a Rule<P, T>> {
    // (rule index, score) per destination, replaced in place by a better rule.
    let mut kept: Vec<(usize, u8)> = Vec::new();
    for (index, rule) in rules.iter().enumerate() {
        let Some(value) = score(rule, types, defined, id) else {
            continue;
        };
        match kept
            .iter_mut()
            .find(|(kept_index, _)| rules[*kept_index].target == rule.target)
        {
            // Equal specificity: the later rule wins, so `>=` replaces.
            Some(entry) if value >= entry.1 => *entry = (index, value),
            Some(_) => {}
            None => kept.push((index, value)),
        }
    }
    kept.sort_by_key(|(index, _)| *index);
    kept.into_iter().map(|(index, _)| &rules[index]).collect()
}

/// A resolved set of rules: the definition cache plus the registration order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Group<P, T> {
    rules: Vec<Rule<P, T>>,
    defined: Definition,
}

impl<P, T: PartialEq> Group<P, T> {
    /// Builds a group and computes `defined` up front (never incrementally).
    pub fn new(rules: Vec<Rule<P, T>>) -> Self {
        let defined = Definition::of(&rules);
        Self { rules, defined }
    }

    pub fn rules(&self) -> &[Rule<P, T>] {
        &self.rules
    }

    /// The group's `defined` set (`_` = occurring minus defined).
    pub fn defined(&self) -> &Definition {
        &self.defined
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// See [`winner`].
    pub fn winner(&self, types: &TypeSet, id: TypeId<'_>) -> Option<&Rule<P, T>> {
        winner_with_definition(&self.rules, types, &self.defined, id)
    }

    /// See [`matching`].
    pub fn matching(&self, types: &TypeSet, id: TypeId<'_>) -> Vec<&Rule<P, T>> {
        matching_with_definition(&self.rules, types, &self.defined, id)
    }

    /// Does any rule cover `id`?
    pub fn covers(&self, types: &TypeSet, id: TypeId<'_>) -> bool {
        self.rules
            .iter()
            .any(|rule| score(rule, types, &self.defined, id).is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn literal(name: &str) -> Selector {
        Selector::Exact(name.to_string())
    }

    fn types() -> TypeSet {
        TypeSet::from_types(["TypeA", "TypeB"]).with_untyped()
    }

    fn owner<'a>(
        rules: &'a [Rule<&'a str, &'a str>],
        id: TypeId<'_>,
        types: &TypeSet,
    ) -> Option<&'a str> {
        winner(rules, types, id).map(|rule| rule.payload)
    }

    #[test]
    fn winner_literal_beats_underscore() {
        let rules = vec![
            Rule::new(vec![Selector::Underscore], "auto", "rest"),
            Rule::new(vec![literal("TypeA")], "auto", "exact"),
        ];
        let types = types();
        assert_eq!(
            owner(&rules, TypeId::Literal("TypeA"), &types),
            Some("exact")
        );
        assert_eq!(
            owner(&rules, TypeId::Literal("TypeB"), &types),
            Some("rest")
        );
    }

    #[test]
    fn winner_underscore_beats_nothing_and_nothing_matches_nothing() {
        let rules = vec![Rule::new(vec![Selector::Underscore], "auto", "rest")];
        let types = types();
        assert_eq!(
            owner(&rules, TypeId::Literal("TypeA"), &types),
            Some("rest")
        );

        let only_literal = vec![Rule::new(vec![literal("TypeA")], "auto", "A")];
        assert_eq!(owner(&only_literal, TypeId::Literal("TypeB"), &types), None);
        assert_eq!(owner(&only_literal, TypeId::Untyped, &types), None);

        // An empty document is empty: `_` and `*` cover nothing.
        let empty = TypeSet::new();
        let any = vec![Rule::new(vec![Selector::Star], "auto", "all")];
        assert_eq!(owner(&any, TypeId::Untyped, &empty), None);
    }

    #[test]
    fn winner_later_underscore_beats_earlier_underscore() {
        let rules = vec![
            Rule::new(vec![Selector::Underscore], "auto", "first"),
            Rule::new(vec![Selector::Underscore], "auto", "second"),
        ];
        let types = types();
        assert_eq!(
            owner(&rules, TypeId::Literal("TypeA"), &types),
            Some("second")
        );
    }

    #[test]
    fn winner_literal_beats_star_and_question_beats_underscore() {
        let types = types();
        let rules = vec![
            Rule::new(vec![Selector::Star], "auto", "star"),
            Rule::new(vec![literal("TypeB")], "auto", "literal"),
        ];
        assert_eq!(
            owner(&rules, TypeId::Literal("TypeB"), &types),
            Some("literal")
        );
        assert_eq!(
            owner(&rules, TypeId::Literal("TypeA"), &types),
            Some("star")
        );

        let with_question = vec![
            Rule::new(vec![Selector::Underscore], "auto", "rest"),
            Rule::new(vec![Selector::Question], "auto", "untyped"),
        ];
        assert_eq!(
            owner(&with_question, TypeId::Untyped, &types),
            Some("untyped")
        );
    }

    #[test]
    fn defined_is_group_wide_not_registration_order() {
        // A `?` registered *after* the `_` still removes `untyped` from it.
        let group: Group<&str, &str> = Group::new(vec![
            Rule::new(vec![Selector::Underscore], "auto", "rest"),
            Rule::new(vec![Selector::Question], "auto", "untyped"),
        ]);
        let types = TypeSet::from_types(["TypeA"]).with_untyped();
        assert!(group.defined().defines_untyped());
        assert_eq!(
            group
                .winner(&types, TypeId::Untyped)
                .map(|rule| rule.payload),
            Some("untyped")
        );

        // A literal named only by the *last* rule is still excluded from `_`.
        let later_literal: Group<&str, &str> = Group::new(vec![
            Rule::new(vec![Selector::Underscore], "auto", "rest"),
            Rule::new(vec![literal("TypeC")], "auto", "explicit"),
        ]);
        let types = TypeSet::from_types(["TypeA", "TypeC"]);
        assert_eq!(
            later_literal
                .winner(&types, TypeId::Literal("TypeC"))
                .map(|rule| rule.payload),
            Some("explicit")
        );
        assert!(later_literal.defined().names().contains("TypeC"));
    }

    #[test]
    fn star_never_defines_anything() {
        let group: Group<&str, &str> = Group::new(vec![
            Rule::new(vec![Selector::Star], "a", "one"),
            Rule::new(vec![Selector::Star], "b", "two"),
            Rule::new(vec![Selector::Underscore], "c", "rest"),
        ]);
        let types = TypeSet::from_types(["TypeA"]).with_untyped();
        assert!(group.defined().names().is_empty());
        assert!(!group.defined().defines_untyped());
        // `_` still sees every occurring type, named or not.
        assert_eq!(
            group
                .winner(&types, TypeId::Literal("TypeA"))
                .map(|rule| rule.payload),
            Some("rest")
        );
        assert_eq!(
            group
                .winner(&types, TypeId::Untyped)
                .map(|rule| rule.payload),
            Some("rest")
        );
    }

    #[test]
    fn matching_yields_one_entry_per_destination() {
        let rules = vec![
            Rule::new(vec![Selector::Underscore], "./out/a.json", "a"),
            Rule::new(vec![Selector::Underscore], "./out/b.json", "b"),
            // Same destination as the first rule: the later one wins it.
            Rule::new(vec![Selector::Underscore], "./out/a.json", "a2"),
        ];
        let types = TypeSet::from_types(["TypeA"]);
        let matched = matching(&rules, &types, TypeId::Literal("TypeA"));
        assert_eq!(
            matched
                .iter()
                .map(|rule| (rule.target, rule.payload))
                .collect::<Vec<_>>(),
            // Registration order of the *winners*: `b` is rule #2, the
            // overriding `a2` is rule #3.
            vec![("./out/b.json", "b"), ("./out/a.json", "a2")]
        );
        assert_eq!(matched.len(), 2);
    }

    #[test]
    fn matching_prefers_the_literal_over_an_underscore_on_one_destination() {
        let rules = vec![
            Rule::new(vec![Selector::Underscore], "./out/a.json", "rest"),
            Rule::new(vec![literal("TypeA")], "./out/a.json", "exact"),
        ];
        let types = TypeSet::from_types(["TypeA"]);
        assert_eq!(
            matching(&rules, &types, TypeId::Literal("TypeA"))[0].payload,
            "exact"
        );
    }

    #[test]
    fn winner_is_always_a_member_of_matching() {
        let rules = vec![
            Rule::new(vec![Selector::Underscore], "files", "rest"),
            Rule::new(vec![literal("TypeA")], "files", "a"),
            Rule::new(vec![Selector::Star], "export", "all"),
        ];
        let types = TypeSet::from_types(["TypeA", "TypeB"]).with_untyped();
        for id in types.occurring() {
            let winner = winner(&rules, &types, id).expect("a rule covers every type");
            assert_contains(&matching(&rules, &types, id), winner);
        }
    }

    fn assert_contains<'a>(
        matched: &[&'a Rule<&'a str, &'a str>],
        winner: &Rule<&'a str, &'a str>,
    ) {
        assert!(
            matched
                .iter()
                .any(|rule| rule.payload == winner.payload && rule.target == winner.target),
            "winner {winner:?} missing from {matched:?}"
        );
    }

    /// An empty document is empty: `_` and `*` cover nothing, because both
    /// expand against the occurring set. A literal or `?` still names its
    /// identity ("occurs 0 times" is valid and produces no elements).
    #[test]
    fn empty_document_matches_no_wildcard() {
        let group: Group<&str, &str> = Group::new(vec![
            Rule::new(vec![Selector::Star], "a", "one"),
            Rule::new(vec![Selector::Underscore], "b", "two"),
        ]);
        let types = TypeSet::new();
        assert!(!group.covers(&types, TypeId::Untyped));
        assert!(!group.covers(&types, TypeId::Literal("TypeA")));
        assert!(group.matching(&types, TypeId::Untyped).is_empty());

        let question: Group<&str, &str> =
            Group::new(vec![Rule::new(vec![Selector::Question], "c", "untyped")]);
        assert!(question.covers(&types, TypeId::Untyped));
        assert!(!question.covers(&types, TypeId::Literal("TypeA")));
    }

    #[test]
    fn group_iterates_in_registration_order() {
        let group: Group<&str, &str> = Group::new(vec![
            Rule::new(vec![literal("TypeB")], "t", "b"),
            Rule::new(vec![literal("TypeA")], "t", "a"),
        ]);
        let payloads: Vec<&str> = group.rules().iter().map(|rule| rule.payload).collect();
        assert_eq!(payloads, vec!["b", "a"]);
    }
}
