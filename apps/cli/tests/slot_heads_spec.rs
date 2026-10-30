//! S2-4.12: **the slot head is read whole** — the counterpart of
//! `extras_heads_spec.rs`, one layer down.
//!
//! `extras_heads_spec` proves that the `{…}` / `@@type{…}` head reaches every
//! element. This file proves the same for the two **positional groups** (§5.2):
//! a head an element claims is not a shape it picks the parts it likes out of —
//! every content the head carries lands in a field, either the element's preset
//! name or its slot-qualified bare name (`bracket.tick`, `paren.quote`, …,
//! D-7, §6.6). An element that contracted only the positions it had a name for
//! would drop the rest, which is what the two tests here make impossible:
//!
//! * [`every_slot_head_claims_every_position`] audits the **contracts** — the
//!   four elements that read a slot head claim all six positions the grammar can
//!   produce, so nothing is uncontracted in the first place;
//! * [`no_element_drops_a_claimed_position`] audits the **pipeline** — one head
//!   with six distinct values per element comes out with all six still there,
//!   and with no diagnostic, so nothing is dropped silently or otherwise.
//!
//! The exact bytes are the goldens' job (`37-head-slots-inline`,
//! `38-head-slots-block`); this file asserts the property, not the output.

use pendon_core::{Event, Options};
use pendon_extra::{ElementHead, Slot, SlotKind};
use pendon_plugin_anchor::HEAD as ANCHOR;
use pendon_plugin_cite::HEAD as CITE;
use pendon_plugin_heading::HEAD as HEADING;
use pendon_plugin_image::HEAD as IMAGE;

/// Every position a head can carry, derived from the grammar's own enumerations
/// (`SlotKind::ALL` × `Slot::ALL`) so this audit grows with §5.2 rather than
/// drifting from it.
fn all_positions() -> Vec<(Slot, SlotKind)> {
    Slot::ALL
        .into_iter()
        .flat_map(|slot| SlotKind::ALL.map(move |kind| (slot, kind)))
        .collect()
}

/// The four elements that contract a slot head: anchor, cite, heading, image
/// (§7.1–§7.4). They are the only callers of `parse_slots` in the workspace.
fn slot_heads() -> [(&'static str, ElementHead); 4] {
    [
        ("anchor", ANCHOR),
        ("cite", CITE),
        ("heading", HEADING),
        ("image", IMAGE),
    ]
}

/// D-7/§6.6: a claimed head is read whole, so no element may narrow its contract
/// to the positions its preset happens to name. If one ever did, the rest of the
/// head would be no field at all — the "uncontracted content" this stage exists
/// to rule out.
#[test]
fn every_slot_head_claims_every_position() {
    let positions = all_positions();
    assert_eq!(positions.len(), 6, "two groups × three contents (§5.2)");

    for (element, head) in slot_heads() {
        for (slot, kind) in &positions {
            assert!(
                head.contract.accepts(*slot, *kind),
                "`{element}` does not claim {}.{} — a head carrying it would lose it",
                slot.name(),
                kind.name()
            );
        }
        assert!(
            head.is_consistent(),
            "`{element}` presets a position its contract does not claim"
        );
    }
}

/// Everything a consumer of a run can read: the node's attribute values, bare
/// flag names and text (the last one matters because a construct with no
/// component writes its markup as raw text — the built-in citation, §7.3), plus
/// the side exports the layer publishes.
fn harvest(events: &[Event], exports: &[&serde_json::Value]) -> String {
    let mut out = String::new();
    for event in events {
        match event {
            Event::Attribute { name, value } => {
                out.push_str(name);
                out.push('=');
                out.push_str(value);
                out.push('\n');
            }
            Event::AttributeFlag { name } => {
                out.push_str(name);
                out.push('\n');
            }
            Event::Text(text) => {
                out.push_str(text);
                out.push('\n');
            }
            Event::Diagnostic { message, .. } => {
                out.push_str("diagnostic: ");
                out.push_str(message);
                out.push('\n');
            }
            Event::StartNode(_) | Event::EndNode(_) => {}
        }
    }
    for export in exports {
        out.push_str(&export.to_string());
        out.push('\n');
    }
    out
}

/// `(element, source, the six values its head carries)` — the same six positions
/// in the same order for every element, so a row that loses one is an element
/// that did not read its head whole.
fn cases() -> [(&'static str, &'static str, [&'static str; 6]); 4] {
    const SIX: [&str; 6] = [
        "v-bare",
        "v-quote",
        "v-tick",
        "v-paren",
        "v-paren-quote",
        "v-paren-tick",
    ];
    [
        (
            "anchor",
            r#"@[v-bare "v-quote" `v-tick`](/v-paren "v-paren-quote" `v-paren-tick`)"#,
            SIX,
        ),
        (
            "image",
            r#"![v-bare "v-quote" `v-tick`](/v-paren "v-paren-quote" `v-paren-tick`)"#,
            SIX,
        ),
        (
            "heading",
            r#"##[v-bare "v-quote" `v-tick`](/v-paren "v-paren-quote" `v-paren-tick`) Text"#,
            SIX,
        ),
        (
            "cite",
            r#"[^v-bare "v-quote" `v-tick`](/v-paren "v-paren-quote" `v-paren-tick`)"#,
            SIX,
        ),
    ]
}

/// Runs the element's own plugin over its head, the way the chain does, and
/// returns everything the run published.
fn published(element: &str, source: &str) -> String {
    let events = pendon_core::parse(source, &Options::default());
    match element {
        "anchor" => harvest(
            &pendon_plugin_anchor::process(&events, &Default::default()),
            &[],
        ),
        "image" => harvest(
            &pendon_plugin_image::process(&events, &Default::default()),
            &[],
        ),
        "heading" => harvest(
            &pendon_plugin_heading::process(&events, &Default::default()),
            &[],
        ),
        "cite" => {
            // §7.3: a citation needs a reference. It is injected as the merged
            // reference set the real chain would have collected, so the case
            // stays one legal head and nothing else — and the `citations`
            // binding the solid renderer exports is read alongside the node,
            // because that is where `locator` lives.
            let mut ctx = pendon_plugin_cite::CitationContext::new(
                serde_json::json!({ "v-bare": { "id": "v-bare", "title": "Sentinel" } }),
                pendon_plugin_cite::CiteOptions::default(),
            );
            let out = ctx.process_events(&events);
            let citations = ctx.get_citations();
            harvest(&out, &[&citations])
        }
        other => panic!("no slot head for `{other}`"),
    }
}

/// The exit of S2-4.12: an element never drops slot content — and never has to
/// warn about it either, because D-7 gives every position a field.
#[test]
fn no_element_drops_a_claimed_position() {
    for (element, source, values) in cases() {
        let published = published(element, source);
        for value in values {
            assert!(
                published.contains(value),
                "`{element}` dropped `{value}` from {source}\n---\n{published}"
            );
        }
        assert!(
            !published.contains("diagnostic:"),
            "`{element}` warned about a legal head {source}\n---\n{published}"
        );
    }
}
