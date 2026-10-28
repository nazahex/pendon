//! Preset assignment: where each slot position of a built-in element lands
//! (`docs/spec/SYNTAX.md` §6.1/§6.5, proposal §7, D-7).
//!
//! Every element that contracts a slot head has three pieces of static data:
//!
//! * its [`SlotContract`] — which positions it claims (§6.1, ADR-0012);
//! * its **preset** — which field each of those positions lands in when the
//!   user configures nothing (`@[Inner Text](href "title")` → `inner_text`,
//!   `href`, `title`);
//! * its **owned** fields — the ones the element writes itself, so a head item
//!   that spells one is an `Error` (D-12 rule 5).
//!
//! All three are data, and all of them live in the owning plugin's crate
//! ([`ElementHead`] is just the triple). That is deliberate: the assignment is
//! part of the element's contract, not of a plugin's private code, so Phase 4 can
//! delete the hand-written `insert("href", …)` calls without moving the truth
//! (D-16, ADR-0014).
//!
//! **The preset is void as a whole.** One `field.<slot>.<kind>` key in the
//! matched `custom` entry disables every assignment here for that entry — there
//! is no half-preset mode — and each unassigned position falls back to its
//! slot-qualified bare name ([`fallback_path`]: `bracket.bare`, `paren.quote`, …)
//! per D-7. The trigger is a *slot* key, not any `field.*`: the preset maps slot
//! positions, so `field.specs.*` / `field.override.*` leave it alone. The voiding
//! itself is the assignment step's job (S2-2.6); this module only holds the data
//! and the lookups.

use crate::field::{FieldConfig, FieldMap, FieldMapError, FieldName, Path};
use crate::slot::{Slot, SlotContract, SlotKind, SlotPair};
use crate::value::AttrValue;

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

/// D-12's last rule, as a predicate: does `key` name a field the construct
/// writes itself (`href` on an anchor, `src` on an image)?
///
/// The list is [`ElementHead::owned`] for an element that contracts slots, and a
/// plain `const` for one that does not (`plugin-wiki`'s `[[…]]` link). The
/// collision itself is reported by the element as
/// [`ExtrasIssue::owned_key`](crate::typed::ExtrasIssue::owned_key) — an `Error`
/// pointing at `field.override.<key>` (D-4) — because the head must never write a
/// field the construct owns.
pub fn owns_field(owned: &[FieldName], key: &str) -> bool {
    owned.iter().any(|name| name.as_str() == key)
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
    /// The fields the element writes **itself** (D-12 rule 5): a head item that
    /// spells one of them is an `Error`, and `field.override.<k>` (D-4) is the
    /// way to publish it under another name.
    pub owned: &'static [FieldName],
}

impl ElementHead {
    /// Pairs a contract with its preset table.
    ///
    /// The owned list starts empty; an element that writes fields itself adds
    /// them with [`with_owned`](Self::with_owned), so the collision rule is the
    /// element's own data rather than a hardcoded `if key == "…"` in the reader.
    pub const fn new(contract: SlotContract, preset: &'static [Preset]) -> Self {
        Self {
            contract,
            preset,
            owned: &[],
        }
    }

    /// Declares the fields the element writes itself (D-12 rule 5).
    pub const fn with_owned(mut self, owned: &'static [FieldName]) -> Self {
        self.owned = owned;
        self
    }

    /// True when `key` names a field this element writes itself (D-12 rule 5).
    pub fn owns(&self, key: &str) -> bool {
        owns_field(self.owned, key)
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

    /// The head's contents as fields, honoring the entry's `field.*` (D-7, D-8).
    ///
    /// Every position the contract accepts is taken in head order, and each
    /// content present lands where:
    ///
    /// | the entry configures…                        | a position lands in                  |
    /// | -------------------------------------------- | ------------------------------------ |
    /// | no `field.<slot>.<kind>` (or no entry)       | its preset name, else its bare name  |
    /// | **any** `field.<slot>.<kind>` (D-7 void)     | the assigned name, else its bare name |
    ///
    /// `config` is the **matched** entry's [`FieldConfig`], never the task's: D-7
    /// voids the preset per `custom` entry, and there is no half-preset mode — one
    /// slot key kills every preset row at once. The trigger is a **slot**
    /// assignment on purpose: `field.specs.*` and `field.override.*` say where an
    /// extras item goes or how a public path is spelled, not where a slot lands,
    /// so they leave the preset alive (`S2-4.2`). `""` (`Some(None)`) is an
    /// *unassign*, so that position falls back to its bare name like any other
    /// unassigned one (OBJ-10), and the order of `fallback_path` values is the
    /// head's own (bracket bare → quote → tick, then paren).
    ///
    /// Values are [`AttrValue::Str`]: slot content is text (§6.3). Nothing here
    /// applies `field.override` — that is `project`'s job (ADR-0014 rules 3–4).
    ///
    /// Two positions assigned to one name are **not** an error here: the later
    /// position in head order replaces the earlier one in place (that is
    /// [`FieldMap::insert`]'s duplicate policy). A load-time error would have to
    /// live in the config layer, where the whole entry is visible.
    pub fn assign(
        &self,
        pair: &SlotPair,
        config: Option<&FieldConfig>,
    ) -> Result<FieldMap, FieldMapError> {
        let preset_alive = config.is_none_or(FieldConfig::is_empty_slots);
        let mut fields = FieldMap::new();
        for &(slot, kind) in self.contract.accepted {
            let Some(value) = pair.get(slot, kind) else {
                continue;
            };
            let path = if preset_alive {
                self.preset_for(slot, kind).map(FieldName::path)
            } else {
                config
                    .and_then(|config| config.assignment(slot, kind))
                    .and_then(|assigned| assigned.clone())
            };
            let path = path.unwrap_or_else(|| fallback_path(slot, kind));
            fields.insert(path, AttrValue::Str(value.to_string()))?;
        }
        Ok(fields)
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
            (Slot::Bracket, SlotKind::Tick),
            (Slot::Paren, SlotKind::Bare),
            (Slot::Paren, SlotKind::Quote),
            (Slot::Paren, SlotKind::Tick),
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

        // The checker is not vacuous: an element that claims only `[bare]` may
        // not preset a paren position.
        const BARE_ONLY: SlotContract = SlotContract {
            accepted: &[(Slot::Bracket, SlotKind::Bare)],
            tag: SlotTag::Anchor,
        };
        assert!(!ElementHead::new(BARE_ONLY, PRESET).is_consistent());

        // An element with no preset at all is consistent, not broken: blockquote
        // and list contract nothing and name nothing (§9.2, §9.3).
        const EMPTY: SlotContract = SlotContract {
            accepted: &[],
            tag: SlotTag::None,
        };
        assert!(ElementHead::new(EMPTY, &[]).is_consistent());
    }

    #[test]
    fn an_element_declares_the_fields_it_writes_itself() {
        // D-12 rule 5: an untouched head owns nothing, so no element inherits a
        // collision rule it did not declare.
        let bare = ElementHead::new(CONTRACT, PRESET);
        assert!(bare.owned.is_empty());
        assert!(!bare.owns("href"));

        const OWNED: &[FieldName] = &[FieldName::new("href")];
        let pinned = ElementHead::new(CONTRACT, PRESET).with_owned(OWNED);
        assert!(pinned.owns("href"));
        assert!(!pinned.owns("inner_text"), "a preset name is not owned");
        assert!(!pinned.owns("HREF"), "the rule is exact, never case-folded");
        // The predicate the whole rule rests on, on its own.
        assert!(owns_field(OWNED, "href"));
        assert!(!owns_field(OWNED, "src"));
        assert!(!owns_field(&[], "href"));
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

    // ---- S2-2.6: what a parsed head lands in (D-7) ----

    use crate::slot::parse_slots;

    /// The head D-7 is written about, parsed by the real grammar so the test
    /// cannot drift from it: `@[foo "Bar" `baz`](soo "Sar" `zaz`)`.
    ///
    /// The decision doc spells the same head without the spaces before each tick,
    /// which §5.2 reads as **one** quote content (a space is what ends a
    /// content), so the spaced spelling is the one that carries the six values
    /// D-7 lists.
    fn d7_pair() -> SlotPair {
        match parse_slots(r#"@[foo "Bar" `baz`](soo "Sar" `zaz`)"#) {
            crate::slot::SlotMatch::Slots { pair, .. } => pair,
            other => panic!("the D-7 head must parse, got {other:?}"),
        }
    }

    /// Every field as `path = value`, in projection order — the shape a user (and
    /// the spec) reads.
    fn rows(fields: &FieldMap) -> Vec<String> {
        fields
            .iter()
            .map(|(path, value)| format!("{} = {}", path.dotted(), literal(value)))
            .collect()
    }

    fn literal(value: &AttrValue) -> String {
        match value {
            AttrValue::Str(text) => text.clone(),
            other => panic!("a slot value is text, got {other:?}"),
        }
    }

    #[test]
    fn one_field_key_voids_the_whole_preset() {
        // D-7's example, asserted exactly.
        let mut config = FieldConfig::new();
        config.assign_slot(
            Slot::Bracket,
            SlotKind::Bare,
            Some(Path::from_dotted("something").unwrap()),
        );

        let fields = ElementHead::new(CONTRACT, PRESET)
            .assign(&d7_pair(), Some(&config))
            .unwrap();
        assert_eq!(
            rows(&fields),
            [
                "something = foo",
                "bracket.quote = Bar",
                "bracket.tick = baz",
                "paren.bare = soo",
                "paren.quote = Sar",
                "paren.tick = zaz",
            ]
        );
        // Not a half-preset: the three preset names are gone for this entry.
        for gone in ["inner_text", "href", "title"] {
            assert!(
                !fields.contains(&Path::from_dotted(gone).unwrap()),
                "{gone}"
            );
        }
    }

    #[test]
    fn a_specs_or_override_assignment_keeps_the_preset() {
        // S2-4.2: only a **slot** assignment voids the preset. Renaming an extras
        // item (`field.specs.*`) or a public path (`field.override.*`) says
        // nothing about where a slot lands, so `inner_text` / `href` / `title`
        // survive — which is what fixture `34-specs-assignment` shows.
        let pair = d7_pair();
        let head = ElementHead::new(CONTRACT, PRESET);
        let kept = rows(&head.assign(&pair, None).unwrap());

        let mut config = FieldConfig::new();
        config
            .assign_specs("tick", Some(Path::from_dotted("slug").unwrap()))
            .unwrap();
        config
            .add_override(
                Path::from_dotted("href").unwrap(),
                Path::from_dotted("url").unwrap(),
            )
            .unwrap();
        assert!(!config.is_empty(), "the entry did configure something");
        assert!(
            config.is_empty_slots(),
            "but it configured no slot position"
        );
        assert_eq!(
            rows(&head.assign(&pair, Some(&config)).unwrap()),
            kept,
            "a non-slot key must not void the preset"
        );
    }

    #[test]
    fn an_entry_that_configures_no_field_keeps_the_preset() {
        let pair = d7_pair();
        let head = ElementHead::new(CONTRACT, PRESET);
        let kept = rows(&head.assign(&pair, Some(&FieldConfig::new())).unwrap());
        // No matched entry at all is the same as an empty one.
        assert_eq!(kept, rows(&head.assign(&pair, None).unwrap()));
        assert_eq!(
            kept,
            [
                "inner_text = foo",
                "bracket.quote = Bar",
                "bracket.tick = baz",
                "href = soo",
                "title = Sar",
                "paren.tick = zaz",
            ]
        );
    }

    #[test]
    fn an_unassign_empties_one_position_instead_of_voicing_the_rest() {
        // OBJ-10: `field.bracket.bare = ""` — the position falls back to its bare
        // name; the preset is void either way, so nothing is `inner_text`.
        let mut config = FieldConfig::new();
        config.assign_slot(Slot::Bracket, SlotKind::Bare, None);

        let fields = ElementHead::new(CONTRACT, PRESET)
            .assign(&d7_pair(), Some(&config))
            .unwrap();
        assert_eq!(
            rows(&fields),
            [
                "bracket.bare = foo",
                "bracket.quote = Bar",
                "bracket.tick = baz",
                "paren.bare = soo",
                "paren.quote = Sar",
                "paren.tick = zaz",
            ]
        );
    }

    #[test]
    fn a_position_may_move_to_another_slot_content() {
        // §7.3's "simpler version": `reference_id` moves from bare to tick, and
        // `locator` has to be redefined because the preset is void.
        const CITE_PRESET: &[Preset] = &[
            Preset::new(
                Slot::Bracket,
                SlotKind::Bare,
                FieldName::new("reference_id"),
            ),
            Preset::new(Slot::Bracket, SlotKind::Quote, FieldName::new("locator")),
        ];
        let cite = ElementHead::new(CONTRACT, CITE_PRESET);
        let pair = match parse_slots(r#"[^book "p. 1" `extra`]"#) {
            crate::slot::SlotMatch::Slots { pair, .. } => pair,
            other => panic!("the cite head must parse, got {other:?}"),
        };

        let mut config = FieldConfig::new();
        config.assign_slot(
            Slot::Bracket,
            SlotKind::Tick,
            Some(Path::from_dotted("reference_id").unwrap()),
        );
        config.assign_slot(
            Slot::Bracket,
            SlotKind::Quote,
            Some(Path::from_dotted("locator").unwrap()),
        );

        let fields = cite.assign(&pair, Some(&config)).unwrap();
        assert_eq!(
            rows(&fields),
            [
                "bracket.bare = book",
                "locator = p. 1",
                "reference_id = extra",
            ]
        );
    }

    #[test]
    fn a_position_the_contract_refuses_is_not_read() {
        // `CONTRACT` claims no tick, so a tick content is not a field here — the
        // element's contract is what decides, not the text that happens to parse.
        const NO_TICK: SlotContract = SlotContract {
            accepted: &[(Slot::Bracket, SlotKind::Bare)],
            tag: SlotTag::Anchor,
        };
        let fields = ElementHead::new(NO_TICK, PRESET)
            .assign(&d7_pair(), None)
            .unwrap();
        assert_eq!(rows(&fields), ["inner_text = foo"]);
    }
}
