//! §16: golden fixtures for the unified extras syntax (`docs/spec/golden/`).
//!
//! Every fixture is a small project checked into the repo:
//!
//! ```text
//! docs/spec/golden/NN-name.mdp  input markdown
//! docs/spec/golden/NN-name.toml the task config that renders it (§16.1)
//! docs/spec/golden/NN-name.jsx  the frozen Solid output
//! ```
//!
//! The test renders the input with the fixture config in a temp dir and
//! compares the output byte for byte. Nothing here is `#[ignore]`d: the Phase 1
//! fixtures (01–07) cover the grammar that is already implemented.

use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use std::path::{Path, PathBuf};

/// Where the fixtures live, relative to the workspace root.
fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("apps dir")
        .parent()
        .expect("workspace root")
        .join("docs/spec/golden")
}

/// Writes the fixture project into a temp dir and returns it.
///
/// The fixture lives in the repo as `NN-name.{mdp,toml,jsx}`; inside the temp
/// project it is a regular project (`src/NN-name.mdp`, `pendon.toml`, …), so the
/// harness exercises the real CLI path, config loading included.
fn write_project(stem: &str) -> PathBuf {
    let golden = golden_dir();
    let dir = std::env::temp_dir().join(format!("pendon-golden-{stem}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("src")).expect("src dir");

    fs::copy(
        golden.join(format!("{stem}.mdp")),
        dir.join("src").join(format!("{stem}.mdp")),
    )
    .expect("fixture source");
    fs::copy(golden.join(format!("{stem}.toml")), dir.join("pendon.toml")).expect("fixture config");
    copy_data_dir(stem, &dir);
    dir
}

/// Copies a fixture's payload directory (`NN-name.data/`) into the temp project
/// as `data/`, when it has one.
///
/// A file payload (`{{{csv[rows](../data/rows.csv)}}}`) must be resolved against
/// the **source file's** directory, so the payload has to sit *outside* `src/`
/// for `../` to be part of what the fixture proves (§2.7).
fn copy_data_dir(stem: &str, dir: &Path) {
    let from = golden_dir().join(format!("{stem}.data"));
    if !from.is_dir() {
        return;
    }
    let to = dir.join("data");
    fs::create_dir_all(&to).expect("data dir");
    for entry in fs::read_dir(&from).expect("read data dir") {
        let entry = entry.expect("data entry");
        fs::copy(entry.path(), to.join(entry.file_name())).expect("copy data file");
    }
}

fn render(stem: &str) -> (PathBuf, String) {
    let dir = write_project(stem);
    let mut cmd = cargo_bin_cmd!("pendon");
    cmd.current_dir(&dir)
        .arg("run")
        // The cache is keyed per project directory; force the render so a
        // leftover `.pendon` can never hide a broken fixture.
        .arg("-F")
        .assert()
        .success();
    let out =
        fs::read_to_string(dir.join("out").join(format!("{stem}.jsx"))).expect("rendered output");
    (dir, out)
}

/// Compares a fixture's side files with what the render wrote beside the page.
///
/// An `[[task.<element>.extract]]` block writes files, not markup, so the golden
/// for one is the file itself: `NN-name.out/` mirrors the project's `out/`
/// directory (the page's own `.jsx` is compared separately). A fixture without
/// that directory promises no side file.
fn assert_side_outputs(stem: &str, dir: &Path) {
    let from = golden_dir().join(format!("{stem}.out"));
    if !from.is_dir() {
        return;
    }

    let mut entries: Vec<PathBuf> = fs::read_dir(&from)
        .expect("read golden side dir")
        .map(|entry| entry.expect("side entry").path())
        .collect();
    entries.sort();

    for expected_path in entries {
        let name = expected_path
            .file_name()
            .expect("a file name")
            .to_string_lossy()
            .into_owned();
        let expected = fs::read_to_string(&expected_path)
            .unwrap_or_else(|error| panic!("{stem}.out/{name}: {error}"));
        let actual_path = dir.join("out").join(&name);
        let actual = fs::read_to_string(&actual_path).unwrap_or_else(|error| {
            panic!("{stem}.out/{name}: {error} (the render wrote no such side file)")
        });
        assert_eq!(actual, expected, "{stem}.out/{name} does not match");
    }
}

/// Fails when a rendered `.jsx` spells a **dotted attribute name** (`specs.tick="…"`),
/// which is not JSX: `a.b=` parses as a member expression.
///
/// §6.3 fixes how a dotted path is written — the one-key spread
/// `{...{ "specs.tick": "…" }}` — for the built-in element path and the component
/// `{...attrs}` path alike (S2-3.6), so a regression in either one is caught here
/// on **every** fixture rather than only where a dotted field happens to be
/// asserted.
fn assert_no_dotted_attribute_name(stem: &str, jsx: &str) {
    // An attribute position is whitespace (or `<`) followed by a name and `=`.
    for (index, line) in jsx.lines().enumerate() {
        let mut rest = line;
        while let Some(at) = rest.find([' ', '<']) {
            let candidate = rest[at + 1..].trim_start();
            let name: String = candidate
                .chars()
                .take_while(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | ':' | '.'))
                .collect();
            let after = &candidate[name.len()..];
            // `=` is required: a dotted path is never a bare flag, and a prose
            // mention (`… see specs.tick here`) is not an attribute position.
            if name.contains('.') && after.starts_with('=') {
                panic!(
                    "{stem}.jsx line {}: `{name}` is not a JSX attribute name — a dotted \
                     field is written as `{{...{{ \"{name}\": … }}}}` (§6.3):\n  {line}",
                    index + 1
                );
            }
            rest = &rest[at + 1..];
        }
    }
}

/// Renders the fixture and compares it — page and side files — with the
/// checked-in golden.
fn assert_fixture(stem: &str) {
    let golden_path = golden_dir().join(format!("{stem}.jsx"));
    let expected =
        fs::read_to_string(&golden_path).unwrap_or_else(|error| panic!("{stem}.jsx: {error}"));
    let (dir, actual) = render(stem);
    assert_no_dotted_attribute_name(stem, &actual);

    if actual == expected {
        assert_side_outputs(stem, &dir);
        return;
    }

    // A byte-level diff message: the first line that differs.
    let first_diff = actual
        .lines()
        .zip(expected.lines())
        .enumerate()
        .find(|(_, (got, want))| got != want)
        .map(|(index, (got, want))| {
            format!(
                "\n  line {}:\n    got:  {}\n    want: {}",
                index + 1,
                got,
                want
            )
        })
        .unwrap_or_else(|| "\n  (outputs differ in length)".to_string());
    panic!(
        "golden mismatch for {stem}:{first_diff}\n\n\
         expected (docs/spec/golden/{stem}.jsx):\n{expected}\n\
         actual:\n{actual}\n\n\
         Run `pendon run -F` inside a copy of the fixture project to refresh the golden, \
         then re-read the diff to make sure the new output is the intended behaviour."
    );
}

/// The guard itself: the one-key spread and a prose mention pass, an attribute
/// spelled with a dot does not.
#[test]
fn the_dotted_attribute_guard_catches_the_invalid_spelling() {
    assert_no_dotted_attribute_name("probe", "<p {...{ \"specs.tick\": \"x\" }} class=\"c\">");
    assert_no_dotted_attribute_name("probe", "<p>see specs.tick here</p>");

    let caught = std::panic::catch_unwind(|| {
        assert_no_dotted_attribute_name("probe", "<X specs.tick=\"x\" />");
    });
    assert!(caught.is_err(), "the guard missed `specs.tick=\"x\"`");
}

/// §16.3 fixture 01: all item kinds, ordering, duplicates, escaping (§5–§6).
#[test]
fn golden_01_extras_head() {
    assert_fixture("01-extras-head");
}

/// §16.3 fixture 02: malformed heads stay literal text, adjacency failures (§4.3).
#[test]
fn golden_02_extras_literal() {
    assert_fixture("02-extras-literal");
}

/// §16.3 fixture 03: `!` tag + preset `alt`/`src`, bare slot fields, the retired
/// marker run as literal text (§7.1).
#[test]
fn golden_03_image() {
    assert_fixture("03-image");
}

/// §16.3 fixture 04: head `("title")` vs extras `"title"` priority (§7.2).
#[test]
fn golden_04_anchor() {
    assert_fixture("04-anchor");
}

/// §16.3 fixture 05: `[^^](ref "loc")` + extras, `loc=` prop form (§7.3).
#[test]
fn golden_05_cite() {
    assert_fixture("05-cite");
}

/// §16.3 fixture 06: `[slug]` + `("title")` + extras, auto-number interaction
/// (§7.4).
#[test]
fn golden_06_heading() {
    assert_fixture("06-heading");
}

/// §16.3 fixture 07: `[[…]]` + extras, `href` not overridable (§7.5).
#[test]
fn golden_07_wiki() {
    assert_fixture("07-wiki");
}

/// §16.3 fixture 08: the `<table>` layer's decorator head, plus the caption line
/// (§8).
#[test]
fn golden_08_table_head() {
    assert_fixture("08-table-head");
}

/// §16.3 fixture 09: column/th/td/tr/tbody/tfoot extras routing (§8).
#[test]
fn golden_09_table_layers() {
    assert_fixture("09-table-layers");
}

/// §16.3 fixture 10: general block decorators (§9.1) — a paragraph and a code
/// fence each decorated by the line above them, the typed head's marker landing
/// as a `type` attribute (§11 rule 3). A *heading* decorator belongs to
/// `plugin-section` (§9.5), so it is not a `markdown` case.
#[test]
fn golden_10_decorator_blocks() {
    assert_fixture("10-decorator-blocks");
}

/// §16.3 fixture 14: inline + block forms, `<span>`/`<div>` fallback (§10.1).
#[test]
fn golden_14_marker() {
    assert_fixture("14-marker");
}

/// §16.3 fixture 15: `::…::` nesting 2..7, `field.*` naming the head slots
/// (§10.2).
#[test]
fn golden_15_directive_inline() {
    assert_fixture("15-directive-inline");
}

/// §16.3 fixture 16: `==type`/`==` LIFO closing, nesting, EOF warning (§10.3).
#[test]
fn golden_16_directive_block() {
    assert_fixture("16-directive-block");
}

/// §16.3 fixture 21: bare/typed/type-only/empty heads and strict adjacency across
/// anchor, heading and table (§3–§4, §7.4, §8).
#[test]
fn golden_21_extras_forms() {
    assert_fixture("21-extras-forms");
}

/// §16.3 fixture 11: the blockquote layer (§9.2) — inner head, decorator line,
/// inner-wins, and the D8 element fallback for an unclaimed type.
#[test]
fn golden_11_blockquote() {
    assert_fixture("11-blockquote");
}

/// §16.3 fixture 12: the list container merge (§9.3 L1, §9.4) — typed/untyped
/// container decorators, marker-decides-layer, and the layer default.
#[test]
fn golden_12_list_container() {
    assert_fixture("12-list-container");
}

/// §16.3 fixture 22: the `plugin-section` outline (§9.5) — the section decorator
/// line binds to the section (not the heading), the id transfers off the heading
/// through the priority chain, and the `<--->` / `>---<` level markers deepen and
/// close the outline.
#[test]
fn golden_22_section() {
    assert_fixture("22-section");
}

/// §16.3 fixture 23: `plugin-bind` (§19) — a `{{{lang[var] … }}}` block is read
/// before the lexer, `$var` binds a real JS value in an extras head (JSON,
/// JSONC, YAML merge keys, TOML, CSV, nested values), and an unbound `$var`
/// stays literal with a warning.
#[test]
fn golden_23_bind() {
    assert_fixture("23-bind");
}

/// §16.3 fixture 24: bind **paths** (§2.5), a **spread/merge** item (§2.6) and
/// an **external file** payload (§2.7) resolved against the source file's own
/// directory (the fixture source sits in `src/`, its payload in `../data/`).
#[test]
fn golden_24_bind_paths() {
    assert_fixture("24-bind-paths");
}

/// §16.3 fixture 27: `mdp` is a **sub-document** (§19) — its payload runs
/// through the page's own chain, so a heading, a wiki link, a citation, inline
/// math, a figure with a caption and a table all render inside the fragment, and
/// the payload's citation shares the page's numbering. The payload reaches the
/// prop as a fragment, which Solid carries as `(<>{…}</>)`, and the inline form
/// drops the payload's own paragraph wrapper.
#[test]
fn golden_27_bind_mdp() {
    assert_fixture("27-bind-mdp");
}

/// §16.3 fixture 25: nested objects/arrays in extras values plus JSX brackets
/// (`{valueA}` → `name={valueA}`), including a bind-resolved `$theme` inside a
/// nested value combined with non-bind nested values in the same head (§5, §6.3).
#[test]
fn golden_25_extras_nested() {
    assert_fixture("25-extras-nested");
}

/// §16.3 fixture 28: `type` is a **selector** and the renderer resolves the whole
/// layer once per document (§11, ADR-0008 `D-3a`). Reproduces R.1–R.4 of
/// `docs/issues/type-upgrade.md`: the literal beats `*`, `_` is the remainder of
/// the occurring types minus what the layer defines (so `?` removes untyped from
/// `_` even though it is written later), `?` answers the untyped nodes, and a
/// `*` entry that loses everywhere contributes no import line.
#[test]
fn golden_28_type_selectors() {
    assert_fixture("28-type-selectors");
}

/// §16.3 fixture 26: the reference export scope (§2.6 of RFC-0002) — frontmatter
/// references default to `all`, so an uncited entry still reaches the
/// `references` binding, while `citations` only lists what was cited.
#[test]
fn golden_26_cite_scope() {
    assert_fixture("26-cite-scope");
}

/// §16.3 fixture 30: `plugin-number` numbers headings from
/// `[[task.heading.number]]` (ADR-0010, `D-1`/`D-5`). One counter answers the
/// `HeadingA`+`HeadingB` selector, `style[n]` formats depth n, a depth without a
/// style falls back to decimal, and a type no block selects stays bare. The
/// number is an **attribute** (`number={["1","2","a"]}`), never text.
#[test]
fn golden_30_number_heading() {
    assert_fixture("30-number-heading");
}

/// §16.3 fixture 31: the `%%numberType` marker (ADR-0010, `D-4`/`D-9`). `%`-count
/// − 1 is the depth, the type matches a `[[task.number.custom]]` block
/// case-insensitively, the marker always beats the element's own numbering, and
/// the same counter runs across headings, anchors and images.
#[test]
fn golden_31_number_custom() {
    assert_fixture("31-number-custom");
}

/// §16.3 fixture 32: the `headings` export without `plugin-extract-heading`
/// (T4.5/T4.7). `renderer-solid` builds the **nested** outline from the AST, so
/// the data the deleted plugin injected is unchanged, while
/// `[[task.heading.extract]]` writes the same headings as a **flat** array with
/// `depth` — to a side file and to `export const outline`.
#[test]
fn golden_32_extract_heading() {
    assert_fixture("32-extract-heading");
}

/// §16.3 fixture 33: P.1.a/b/c of `plugin-extract` (T4.7) — two `type`-scoped
/// blocks writing two files in two formats, a `_` block for the untyped
/// remainder, and one `export` name registered twice where the later block wins
/// entirely (D-6).
#[test]
fn golden_33_extract_scopes() {
    assert_fixture("33-extract-scopes");
}

/// §16.3 fixture 34: `field.specs.*` as a **move** (S2-3.5, §6.1/§6.3). One entry
/// assigns all five positions, so its head lands on the author's paths
/// (`slug`, `title`, `foo.class`, `bar.id`, `foo.style`) with the default paths
/// left empty — no `class` / `id` / `style` rename and no id fallback — while an
/// entry that assigns nothing keeps every default in the same document.
#[test]
fn golden_34_specs_assignment() {
    assert_fixture("34-specs-assignment");
}

/// §16.3 fixture 35: cite's fields under a `field.*` block (S2-4.4, §7.3). The
/// entry moves `reference_id` to the tick and `locator` to the quote, and
/// publishes them as `ref_id` / `loc` and the number as `no` (D-4) — the
/// processor still numbers off its own names, so the identity and the ordinals
/// are unchanged.
#[test]
fn golden_35_cite_fields() {
    assert_fixture("35-cite-fields");
}

/// §16.3 fixture 36: one head grammar for every table layer (S2-4.11, §8). The
/// `<table>`, the `<caption>`, a column's `<th>`, the `<tbody>`, a `<tr>`, a
/// `<td>` and the `<tfoot>` each carry a `[…]("…")` pair, `.class` items and a
/// `%%numberType` marker — and the column's extras stay on the `<th>`.
#[test]
fn golden_36_table_slots() {
    assert_fixture("36-table-slots");
}

/// §16.3 fixture 37: the **inline** element group reads its head whole
/// (S2-4.12, D-7). Anchor, image and citation each carry all six positions —
/// bare, quote and tick in both groups — so every value is either a preset field
/// (`inner_text` / `href` / `title`, `alt` / `src`, `reference_id` / `locator`)
/// or a slot-qualified bare field (`bracket.tick`, `paren.quote`, …). The
/// built-in `<sup>` writes the four positions it has no prop for, so not one
/// value is lost for want of a component.
#[test]
fn golden_37_head_slots_inline() {
    assert_fixture("37-head-slots-inline");
}

/// §16.3 fixture 38: the **block** element group, same claim (S2-4.12, D-7) —
/// a heading (whose fourth position is the `#` run, D-11) and a block image.
#[test]
fn golden_38_head_slots_block() {
    assert_fixture("38-head-slots-block");
}

/// §16.3 fixture 39: the decorator's head order (S2-5.1, D-10, §9.1) —
/// `@@type` → slot → `{specs}` → `%%numbering`, with every part landing on the
/// decorated quote. The untyped `@@[slot]%%numberA` spelling proves the sigil,
/// not the type, is what makes the line a decorator.
#[test]
fn golden_39_decorator_order() {
    assert_fixture("39-decorator-order");
}

/// §16.3 fixture 40: `@@` is **mandatory** (S2-5.1, D-10, §9.1/§14) — a bare
/// `{.lead, #nope}` line is literal text and never configures the block below
/// it, while the same head with the sigil decorates the next quote.
#[test]
fn golden_40_decorator_needs_at() {
    assert_fixture("40-decorator-needs-at");
}

/// §16.3 fixture 41: D-10 rule 2 (S2-5.2) — the claim table gives each decorator
/// exactly one owner, and the owner is the **block**, not the plugin's position
/// in the chain: the section takes the heading's decorator, the list its own, the
/// quote its own and the paragraph its own.
#[test]
fn golden_41_decorator_claim() {
    assert_fixture("41-decorator-claim");
}

/// §16.3 fixture 42: D-10 rule 3 (S2-5.2) — every decorator no pass can claim is
/// removed with a `Warning`: a thematic break can never receive one, a heading /
/// list / quote whose plugin is missing from the chain belongs to an absent
/// receiver, and the earlier line of a run is overridden.
#[test]
fn golden_42_decorator_rejected() {
    assert_fixture("42-decorator-rejected");
}

/// §16.3 fixture 43: §5.4 / D-9 (S2-6) — the bracket's `|…` run is pure
/// configuration: a symbol contributes the fields its entry's
/// `bracket.modifier."…"` table names, two contributions to one path merge into
/// an array in written order, `class` accumulates, the head wins over a run, an
/// unconfigured symbol is a no-op, a `true` contribution is a bare flag, and a
/// config-only entry (D-8) needs no custom component to be read.
#[test]
fn golden_43_slot_modifiers() {
    assert_fixture("43-slot-modifiers");
}
