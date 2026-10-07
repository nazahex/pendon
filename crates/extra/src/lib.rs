//! Shared parsing primitives for plugin attribute syntax.
//!
//! Plugins remain responsible for interpreting parsed properties. This crate
//! only defines the common syntax and preserves the unprefixed property key.
//!
//! Three pieces live here:
//!
//! * [`parse_extras`] / [`parse_directive_head`] / [`to_attributes`] — the
//!   `{…}` / `@@type{…}` extras head defined by `docs/spec/SYNTAX.md` §4–§6.
//! * [`scan_extras_chars`] / [`emit_attrs`] / [`warning_event`] — the glue that
//!   binds a parsed head to the event IR, shared by the construct plugins.
//! * [`AttrValue`] / [`classify_scalar`] — typed attribute values (§6.3).
//!
//! The pre-§11 `[.class,#id]{key: value}` form is **removed** (§14): it is
//! plain literal text and this crate has no parser for it.

mod bind;
mod decorator;
mod typed;
mod value;

pub use bind::{emit_attr_warnings, emit_attrs, scan_extras_chars, warning_event, warning_message};

pub use decorator::{
    bind_decorators, parse_decorator_line, Bindings, BoundDecorator, DecoratorDrop, DecoratorLine,
    DroppedDecorator,
};

pub use typed::{
    parse_directive_head, parse_extras, parse_extras_body, parse_type_marker, to_attributes, Attrs,
    DirectiveHead, DirectiveMatch, DirectiveSigil, ExtrasAttr, ExtrasError, ExtrasHead, ExtrasItem,
    ExtrasMatch, ExtrasOptions, ExtrasWarning,
};
pub use value::{classify_scalar, number_kind, AttrValue, NumberKind};

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
