# STATUS — Live Plan of Record

**This is the single live source of truth for "what is happening now."**
It is intentionally small and changes every commit. History and rationale live
elsewhere — do **not** restate them here:

- **Normative grammar** → [`docs/spec/SYNTAX.md`](docs/spec/SYNTAX.md)
- **Why a decision was made (D1–D10)** → [`docs/decisions/`](docs/decisions/)
- **Completed work / historical essays** → [`docs/archive/`](docs/archive/)
- **How to run tests, goldens, sandboxes** → [`CONTRIBUTORS.md`](CONTRIBUTORS.md) & [`docs/guides/testing.md`](docs/guides/testing.md)

Update this file whenever you start, block, or finish a piece of work. When a
line is done, delete it (the git history is the record) — keep this file short.

---

## Current focus

Phase 4 wiring of `plugin-list` L2/L3 (the `<li>` item layer) and the docs
restructure. The container layers (`list`/`unordered`/`ordered`, blockquote)
landed; the item layer is next.

## Open blockers

- **L37 truncation (`sandbox/ultimate`).** A heading whose line carries an extras
  head inside a list item over-scans past the line and re-emits the swallowed
  text inside the `<h*>`; the frozen baseline currently encodes a truncated body.
  Minimal repro:
  ```text
  ##[s-a] V1

  - item

    # [x]("T"){.d}body

  D01 after
  ```
  Owner: shared head reader (`crates/extra`) + the plugin that scans it
  (`plugin-anchor` / `plugin-heading`). After the fix: re-render, review the newly
  visible tail sections, re-freeze, add a regression case to the E section.
  **Blocks:** Step 6b (list-container sandbox cases L53–L55), the D/B/N/E sections
  of `sandbox/ultimate`, and golden `13-list-item`.

## In progress

- [ ] **Step 3 — L2/L3 item layer.** `plugin-list` item wrapper; `plugin-markdown`
      gains `CustomPlacement::ListItem` + `PendingListItem` / `arm_list_item` /
      `take_list_item`; L4 `start:` rejection warning; L5 nesting test; golden
      `13-list-item`. (blocked by L37)
- [ ] **Step 6b — ultimate sandbox (list container).** L53/L54/L55 container
      decorator + bracket group, unclaimed container type, pair lost to the anchor
      layer. (blocked by L37)
- [ ] **Step 7 — full gates.** `cargo test --workspace`, `cargo build --workspace`,
      `cargo fmt --check`, `git diff` review of every golden / baseline move.

## Deferred

- **§11 threading gaps** (from the positional-keys pass):
  - `plugin-table`: `extras_to_layer(head)` still uses `ExtrasOptions::default()`;
    thread `TableOptions` into the 7-layer head scan.
  - `plugin-wiki`: `WikiOptions` carries no `ComponentSet`, so extras stay on the
    built-in `slug`/`title`; add `[task.wiki.custom]` to lift this.
  - `extract-heading`: `process(events)` takes no options; thread heading options
    if `[task.heading.custom]` renames `backtick_key`.
- **Phase-4 cleanup** (Step 7): OPEN-R1 `:::` deprecated alias with warning; archive
  `docs/rfc/unified-syntax.md`; rewrite `sandbox/unified`; trim
  `docs/todo/syntaxes.md`.
- **plugin-markdown gaps** (Step 5): fenced code inside a blockquote (Q14);
  `parse_blockquote_prefix` indentation + blank-`>` loose-list split.
- **Decorator space exception:** a decorator on a list item / blockquote may allow
  one space between marker and head (specified §4.1); exercised once L2/L3 lands.

## Golden & freeze status (see [`docs/spec/golden/README.md`](docs/spec/golden/README.md))

- Golden fixtures `01–12`, `14–16`, `21`, `22` are frozen and green; none
  `#[ignore]`d. `cargo test -p pendon --test syntax_spec` (17 fixtures) and
  `cargo test -p pendon --test ultimate_freeze_spec` are the gates.
- `sandbox/ultimate` frozen baseline `out.frozen/` is gated by
  `ultimate_freeze_spec` — re-freeze deliberately only.
