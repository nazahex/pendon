//! Preset assignment: where each slot position of a built-in element lands
//! (`docs/spec/SYNTAX.md` §6.1/§6.5, proposal §7, D-7).
//!
//! Every element that contracts a slot head has two pieces of static data:
//!
//! * its [`SlotContract`] — which positions it claims (§6.1, ADR-0012);
//! * its **preset** — which field each of those positions lands in when the
//!   user configures nothing (`@[Inner Text](href "title")` → `inner_text`,
//!   `href`, `title`).
//!
//! Both are data, and both live in the owning plugin's crate
//! ([`ElementHead`] is just the pair). That is deliberate: the assignment is part
//! of the element's contract, not of a plugin's private code, so Phase 4 can
//! delete the hand-written `insert("href", …)` calls without moving the truth
//! (D-16, ADR-0014).
//!
//! **The preset is void as a whole.** One `field.*` key in the matched `custom`
//! entry disables every assignment here for that entry — there is no half-preset
//! mode — and each unassigned position falls back to its slot-qualified bare name
//! ([`fallback_path`]: `bracket.bare`, `paren.quote`, …) per D-7. The voiding
//! itself is the assignment step's job (S2-2.6); this module only holds the data
//! and the lookups.

use crate::field::{FieldName, Path};
use crate::slot::{Slot, SlotContract, SlotKind};

/// One preset assignment: the content at `slot`.`kind` lands in field `name`.
///
/// A position appears at most once: [`ElementHead::preset_for`] returns the first
/// match, so a duplicate would simply be unreachable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Preset {
    /// Which of the two slots the content sits in.
    pub slot: Slot,
    /// Which of the three contents it is.
    pub kind: SlotKind,
    /// The field it lands in (`inner_text`, `href`, `title`).
    pub name: FieldName,
}

impl Preset {
    /// A preset for `slot`.`kind` landing in `name`.
    ///
    /// `const` so an element can declare its table in a `static`/`const`
    /// initializer, where [`FieldName::new`] checks the name at compile time.
    pub const fn new(slot: Slot, kind: SlotKind, name: FieldName) -> Self {
        Self { slot, kind, name }
    }

    /// The position this preset fills, as a [`Path`] — `bracket.bare`.
    pub fn position(&self) -> Path {
        fallback_path(self.slot, self.kind)
    }
}

/// The bare name a position falls back to when no preset assigns it (D-7).
///
/// Slot-qualified and dotted — `bracket.bare`, `paren.quote` — so an unassigned
/// value is never lost and never collides with a preset name (a single segment
/// cannot contain a `.`). Both segments come from [`Slot::name`] /
/// [`SlotKind::name`], so the literal is written in exactly one place.
pub fn fallback_path(slot: Slot, kind: SlotKind) -> Path {
    Path::from_segments(&[slot.name(), kind.name()])
        .expect("slot and kind names are valid path segments")
}

/// An element's declarative head: what it claims, and where each position lands.
///
/// The two halves are declared together because they answer the same question —
/// the contract says *which* contents the element takes, the preset says *where*
/// they go — and because a preset for a position the contract does not accept
/// would be dead data ([`ElementHead::is_consistent`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ElementHead {
    /// The slots the element accepts (§6.1).
    pub contract: SlotContract,
    /// The built-in assignment, in declaration order. Empty for an element that
    /// contracts slots without naming any of them (its values stay bare fields).
    pub preset: &'static [Preset],
}

impl ElementHead {
    /// Pairs a contract with its preset table.
    pub const fn new(contract: SlotContract, preset: &'static [Preset]) -> Self {
        Self { contract, preset }
    }

    /// The declared assignment table.
    pub fn presets(&self) -> &'static [Preset] {
        self.preset
    }

    /// The field `slot`.`kind` lands in, if the preset assigns it.
    ///
    /// `None` means "not part of the preset" — either the position is not
    /// contracted at all, or it is contracted and stays a bare field. Callers
    /// must not treat `None` as an error: the position may still carry a value
    /// that the assignment step stores as [`fallback_path`].
    pub fn preset_for(&self, slot: Slot, kind: SlotKind) -> Option<FieldName> {
        self.preset
            .iter()
            .find(|preset| preset.slot == slot && preset.kind == kind)
            .map(|preset| preset.name)
    }

    /// True when every preset position is one the contract accepts.
    ///
    /// The invariant that keeps the two halves honest; each element asserts it in
    /// its own test, so a preset can never silently outlive its contract.
    pub fn is_consistent(&self) -> bool {
        self.preset
            .iter()
            .all(|preset| self.contract.accepts(preset.slot, preset.kind))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::slot::SlotTag;

    const CONTRACT: SlotContract = SlotContract {
        accepted: &[
            (Slot::Bracket, SlotKind::Bare),
            (Slot::Bracket, SlotKind::Quote),
            (Slot::Paren, SlotKind::Bare),
            (Slot::Paren, SlotKind::Quote),
        ],
        tag: SlotTag::Anchor,
    };

    const PRESET: &[Preset] = &[
        Preset::new(Slot::Bracket, SlotKind::Bare, FieldName::new("inner_text")),
        Preset::new(Slot::Paren, SlotKind::Bare, FieldName::new("href")),
        Preset::new(Slot::Paren, SlotKind::Quote, FieldName::new("title")),
    ];

    #[test]
    fn a_preset_names_the_field_its_position_lands_in() {
        let head = ElementHead::new(CONTRACT, PRESET);
        assert_eq!(
            head.preset_for(Slot::Bracket, SlotKind::Bare),
            Some(FieldName::new("inner_text"))
        );
        assert_eq!(
            head.preset_for(Slot::Paren, SlotKind::Quote),
            Some(FieldName::new("title"))
        );
        // Contracted but not preset: the value stays a bare field (D-7).
        assert_eq!(head.preset_for(Slot::Bracket, SlotKind::Quote), None);
        // Not contracted here at all.
        assert_eq!(head.preset_for(Slot::Paren, SlotKind::Tick), None);
    }

    #[test]
    fn a_preset_position_is_always_a_position_the_element_accepts() {
        assert!(ElementHead::new(CONTRACT, PRESET).is_consistent());

        // The checker is not vacuous.
        const OFF_CONTRACT: &[Preset] = &[Preset::new(
            Slot::Bracket,
            SlotKind::Tick,
            FieldName::new("inner_text"),
        )];
        assert!(!ElementHead::new(CONTRACT, OFF_CONTRACT).is_consistent());

        // An element with no preset at all is consistent, not broken: blockquote
        // and list contract nothing and name nothing (§9.2, §9.3).
        const EMPTY: SlotContract = SlotContract {
            accepted: &[],
            tag: SlotTag::None,
        };
        assert!(ElementHead::new(EMPTY, &[]).is_consistent());
    }

    #[test]
    fn every_position_has_a_slot_qualified_bare_name() {
        // D-7's fallbacks, spelled exactly as the spec writes them.
        let all: Vec<String> = Slot::ALL
            .into_iter()
            .flat_map(|slot| SlotKind::ALL.map(|kind| fallback_path(slot, kind).dotted()))
            .collect();
        assert_eq!(
            all,
            [
                "bracket.bare",
                "bracket.quote",
                "bracket.tick",
                "paren.bare",
                "paren.quote",
                "paren.tick",
            ]
        );
    }

    #[test]
    fn a_preset_knows_the_position_it_fills() {
        assert_eq!(PRESET[1].position().dotted(), "paren.bare");
        assert_eq!(PRESET[1].name.as_str(), "href");
    }
}
