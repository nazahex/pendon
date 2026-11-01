//! Shared parsing primitives for plugin attribute syntax.
//!
//! Plugins remain responsible for interpreting parsed properties. This crate
//! only defines the common syntax and preserves the unprefixed property key.
//!
//! Four pieces live here:
//!
//! * [`parse_extras`] / [`parse_directive_head`] / [`to_attributes`] — the
//!   `{…}` / `@@type{…}` extras head defined by `docs/spec/SYNTAX.md` §4–§6.
//! * [`parse_slots`] / [`SlotPair`] / [`SlotTag`] — a `[…]` / `(…)` head
//!   (§5.2–§5.4, ADR-0012), parsed before Markdown sees it, plus
//!   [`parse_slot_parts`] for one slot's content and [`SlotContract`] for the
//!   positions an element accepts. [`parse_attached_slots`] is the same grammar
//!   for the groups of a head this crate does not own (`::`, `==`, `{{…}}`,
//!   `@@…`, ADR-0020).
//! * [`to_attributes_with_modifiers`] / [`apply_modifier_run`] — the same mapping
//!   with a §5.4 run applied last, plus [`HeadConfig`] / [`HeadResolver`], which
//!   answer an entry's `field.*` and `bracket.modifier.*` together.
//! * [`scan_extras_chars`] / [`emit_attrs`] / [`issue_event`] — the glue that
//!   binds a parsed head to the event IR, shared by the construct plugins.
//! * [`AttrValue`] / [`classify_scalar`] — typed attribute values (§6.3).
//! * [`validate_field_name`] / [`Path`] / [`FieldMap`] / [`project`] — the public
//!   field vocabulary the projection emits and the map it fills (ADR-0014,
//!   proposal §7), plus [`FieldName`] / [`Preset`] / [`ElementHead`], the
//!   built-in slot→field assignment each element declares as data (§6.6, D-7).
//! * [`modifier_contributions`] / [`ModifierConfig`] — the §5.4 slot modifier
//!   run: what each symbol is configured to contribute and D-9's merge
//!   (ADR-0017). Pendon ships no modifier of its own.
//!
//! The pre-§11 `[.class,#id]{key: value}` form is **removed** (§14): it is
//! plain literal text and this crate has no parser for it.

mod decorator;
mod emit;
mod field;
mod modifier;
mod preset;
mod slot;
mod typed;
mod value;

pub use emit::{emit_attr_issues, emit_attrs, issue_event, scan_extras_chars};

pub use modifier::{
    append_class, contributions as modifier_contributions, Contribution, ModifierConfig,
    ModifierError,
};

pub use field::{
    project, validate_field_name, FieldConfig, FieldConfigError, FieldMap, FieldMapError,
    FieldName, FieldNameError, Path, SPECS_KEYS, SPECS_NAMESPACE,
};

pub use preset::{fallback_path, owns_field, ElementHead, Preset};

pub use slot::{
    parse_attached_slots, parse_slot_parts, parse_slots, Slot, SlotContract, SlotError, SlotKind,
    SlotMatch, SlotPair, SlotParts, SlotPos, SlotTag,
};

pub use decorator::{
    bind_decorators, classify_block, decorator_diagnostics, parse_decorator_line, Bindings,
    BoundDecorator, Claim, DecoratorDrop, DecoratorLine, DroppedDecorator, DroppedText, Receiver,
    ReceiverSet, Target,
};

pub use typed::{
    apply_modifier_run, parse_directive_head, parse_extras, parse_extras_body, parse_type_marker,
    read_number_mark, to_attributes, to_attributes_with_modifiers, Attrs, DirectiveHead,
    DirectiveMatch, DirectiveSigil, ExtrasAttr, ExtrasError, ExtrasHead, ExtrasIssue, ExtrasItem,
    ExtrasMatch, IssueSeverity, NumberMark, Specs, SpecsKey,
};

/// What a head's `type` marker resolves to: where its contents land and what its
/// `|…` run contributes.
///
/// A construct that reads an extras head needs both halves for the **same**
/// entry — the `field.*` of §6.1 name the positions and the
/// `bracket.modifier."<symbol>"` table of §5.4 says what a symbol adds — so they
/// are resolved together rather than as two independent lookups that could
/// disagree about which entry answered the marker (§11 rule 3).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct HeadConfig {
    /// The public field each position lands in (§6.1).
    pub specs: Specs,
    /// The `bracket.modifier."<symbol>"` tables of the same entry (D-9); `None`
    /// when it defines no symbol, which is the common case.
    pub modifier: Option<ModifierConfig>,
}

impl HeadConfig {
    /// The specs alone — a head whose entry defines no modifier symbol.
    pub fn new(specs: Specs) -> Self {
        Self {
            specs,
            modifier: None,
        }
    }
}

/// Resolves the §6.1 [`HeadConfig`] for a node's `type` marker (§11 rule 5).
///
/// The construct plugins own the component set that answers the marker, so they
/// provide this callback: an exact `type` match resolves to that entry's
/// `field.*` / `bracket.modifier.*`, an unclaimed marker to the layer default's,
/// and a marker with no entry at all to the built-in `bracket.bare` … `style`.
pub type HeadResolver<'a> = &'a dyn Fn(Option<&str>) -> HeadConfig;
pub use value::{classify_scalar, number_kind, AttrValue, NumberKind, SPREAD_KEY};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolidImportSpec {
    pub module: String,
    pub default: Option<String>,
    pub names: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_and_prefixed_heads_are_equivalent() {
        let ExtrasMatch::Head { head, rest } = parse_extras("{.hero} tail") else {
            panic!("expected a bare head");
        };
        assert_eq!(head.type_marker, None);
        assert_eq!(rest, " tail");

        let ExtrasMatch::Head { head, rest } = parse_extras("@@{.hero} tail") else {
            panic!("expected a prefixed head");
        };
        assert_eq!(head.type_marker, None);
        assert_eq!(rest, " tail");
    }

    #[test]
    fn empty_heads_are_valid() {
        for source in ["{}", "@@{}", "@@note{}"] {
            let ExtrasMatch::Head { head, rest } = parse_extras(source) else {
                panic!("expected a head for {source:?}");
            };
            assert!(head.items.is_empty(), "{source:?}");
            assert_eq!(rest, "", "{source:?}");
        }
    }

    #[test]
    fn a_type_only_head_stops_at_the_first_symbol() {
        let ExtrasMatch::Head { head, rest } = parse_extras("@@anchorA. tail") else {
            panic!("expected a type-only head");
        };
        assert_eq!(head.type_marker.as_deref(), Some("anchorA"));
        assert!(head.items.is_empty());
        assert_eq!(rest, ". tail");

        let ExtrasMatch::Head { head, rest } = parse_extras("@@anchorA") else {
            panic!("expected a type-only head");
        };
        assert_eq!(head.type_marker.as_deref(), Some("anchorA"));
        assert_eq!(rest, "");
    }

    #[test]
    fn whitespace_breaks_type_adjacency() {
        assert!(matches!(
            parse_extras("@@anchorA {.hero}"),
            ExtrasMatch::Malformed { .. }
        ));
        assert!(matches!(
            parse_extras("@@anchorA {.hero}\n"),
            ExtrasMatch::Malformed { .. }
        ));
        // …but a symbol on the same line ends the head instead (§4.1).
        assert!(matches!(
            parse_extras("@@anchorA-x{.x}"),
            ExtrasMatch::Head { .. }
        ));
    }
}
