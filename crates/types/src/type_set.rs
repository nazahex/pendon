//! The document's type identities (§1 "occurring types").
//!
//! `occurring = {untyped} ∪ {every literal marker that appears in the
//! document}`. The set is what `_` and `*` are expanded against, so it must be
//! an *occurrence* set: a document with no markers at all is empty, never a
//! wildcard.

use std::collections::BTreeSet;

/// One occurring type identity: a literal marker name, or the untyped element.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeId<'a> {
    /// A `@@type{…}` / `{{type}}` / `::type` / `==type` marker.
    Literal(&'a str),
    /// An element with no marker. Distinct from a literal named `untyped`.
    Untyped,
}

impl<'a> TypeId<'a> {
    /// The identity's name (`untyped` for the untyped identity).
    pub fn name(&self) -> &'a str {
        match self {
            Self::Literal(name) => name,
            Self::Untyped => "untyped",
        }
    }

    /// `true` for the untyped identity — the only one `?` answers.
    pub fn is_untyped(&self) -> bool {
        matches!(self, Self::Untyped)
    }
}

impl std::fmt::Display for TypeId<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

/// The document's occurring types, ordered for deterministic output.
///
/// Literals iterate in `BTreeSet` order (lexicographic, byte-wise); the
/// untyped identity always comes last, so a resolution that expands `*` produces
/// the same sequence on every run.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TypeSet {
    names: BTreeSet<String>,
    untyped: bool,
}

impl TypeSet {
    /// An empty set: no literal occurs and nothing is untyped.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds a set from the literal markers a document carries.
    ///
    /// Duplicates collapse; order is normalized to the `BTreeSet`'s. Whether the
    /// document also contains untyped elements is the caller's decision
    /// ([`with_untyped`](Self::with_untyped)), because only the caller knows
    /// whether it really saw an element without a marker.
    pub fn from_types<I, S>(types: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut set = Self::new();
        for name in types {
            set.insert(name);
        }
        set
    }

    /// Adds one literal marker name (deduplicated).
    pub fn insert(&mut self, name: impl Into<String>) {
        self.names.insert(name.into());
    }

    /// Marks the untyped identity as occurring.
    pub fn insert_untyped(&mut self) {
        self.untyped = true;
    }

    /// Builder form of [`insert_untyped`](Self::insert_untyped).
    pub fn with_untyped(mut self) -> Self {
        self.untyped = true;
        self
    }

    /// Every occurring identity: literals in sorted order, then `untyped`.
    pub fn occurring(&self) -> Vec<TypeId<'_>> {
        let mut out: Vec<TypeId<'_>> = self
            .names
            .iter()
            .map(|name| TypeId::Literal(name))
            .collect();
        if self.untyped {
            out.push(TypeId::Untyped);
        }
        out
    }

    /// Does this identity occur in the document?
    pub fn contains(&self, id: TypeId<'_>) -> bool {
        match id {
            TypeId::Literal(name) => self.names.contains(name),
            TypeId::Untyped => self.untyped,
        }
    }

    /// The literal names, sorted. Excludes the untyped identity.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.names.iter().map(String::as_str)
    }

    /// Does the untyped identity occur?
    pub fn has_untyped(&self) -> bool {
        self.untyped
    }

    /// `true` when nothing occurs — the document has no markers and no untyped
    /// element was recorded.
    pub fn is_empty(&self) -> bool {
        self.names.is_empty() && !self.untyped
    }

    /// Number of occurring identities, the untyped one included.
    pub fn len(&self) -> usize {
        self.names.len() + usize::from(self.untyped)
    }
}

impl FromIterator<String> for TypeSet {
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        Self::from_types(iter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_set_dedupes_and_sorts() {
        let set = TypeSet::from_types(["TypeC", "TypeA", "TypeC", "TypeB"]);
        assert_eq!(
            set.occurring(),
            vec![
                TypeId::Literal("TypeA"),
                TypeId::Literal("TypeB"),
                TypeId::Literal("TypeC"),
            ]
        );
        assert_eq!(set.len(), 3);
        assert!(!set.has_untyped());
    }

    #[test]
    fn type_set_puts_untyped_last() {
        let set = TypeSet::from_types(["TypeA"]).with_untyped();
        assert_eq!(
            set.occurring(),
            vec![TypeId::Literal("TypeA"), TypeId::Untyped]
        );
        assert!(set.contains(TypeId::Untyped));
        assert!(set.contains(TypeId::Literal("TypeA")));
        assert!(!set.contains(TypeId::Literal("TypeB")));
    }

    /// A document with no types yields an empty set, not a wildcard — `*` over
    /// it matches nothing, so an untyped document never "matches everything".
    #[test]
    fn type_set_of_an_untyped_marker_free_document_is_empty() {
        let set = TypeSet::from_types([] as [&str; 0]);
        assert!(set.is_empty());
        assert!(set.occurring().is_empty());
        assert!(!set.contains(TypeId::Untyped));
        assert!(!set.contains(TypeId::Literal("untyped")));
    }

    #[test]
    fn literal_named_untyped_is_distinct_from_the_untyped_identity() {
        let set = TypeSet::from_types(["untyped"]);
        assert!(set.contains(TypeId::Literal("untyped")));
        assert!(!set.contains(TypeId::Untyped));
        assert_eq!(set.occurring(), vec![TypeId::Literal("untyped"),]);
        assert_eq!(TypeId::Untyped.name(), "untyped");
    }
}
