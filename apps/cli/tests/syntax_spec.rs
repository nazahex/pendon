//! §16: golden fixtures for the unified extras syntax (`docs/spec/golden/`).
//!
//! Every fixture is a small project checked into the repo:
//!
//! ```text
//! docs/spec/golden/NN-name.md   input markdown
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
/// The fixture lives in the repo as `NN-name.{md,toml,jsx}`; inside the temp
/// project it is a regular project (`src/NN-name.md`, `pendon.toml`, …), so the
/// harness exercises the real CLI path, config loading included.
fn write_project(stem: &str) -> PathBuf {
    let golden = golden_dir();
    let dir = std::env::temp_dir().join(format!("pendon-golden-{stem}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("src")).expect("src dir");

    fs::copy(
        golden.join(format!("{stem}.md")),
        dir.join("src").join(format!("{stem}.md")),
    )
    .expect("fixture source");
    fs::copy(golden.join(format!("{stem}.toml")), dir.join("pendon.toml")).expect("fixture config");
    dir
}

fn render(stem: &str) -> String {
    let dir = write_project(stem);
    let mut cmd = cargo_bin_cmd!("pendon");
    cmd.current_dir(&dir)
        .arg("run")
        // The cache is keyed per project directory; force the render so a
        // leftover `.pendon` can never hide a broken fixture.
        .arg("-F")
        .assert()
        .success();
    fs::read_to_string(dir.join("out").join(format!("{stem}.jsx"))).expect("rendered output")
}

/// Renders the fixture and compares it with the checked-in golden.
fn assert_fixture(stem: &str) {
    let golden_path = golden_dir().join(format!("{stem}.jsx"));
    let expected =
        fs::read_to_string(&golden_path).unwrap_or_else(|error| panic!("{stem}.jsx: {error}"));
    let actual = render(stem);

    if actual == expected {
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

/// §16.3 fixture 03: `~?!` / `~?!!` + extras + caption (§7.1).
#[test]
fn golden_03_img_figure() {
    assert_fixture("03-img-figure");
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

/// §16.3 fixture 08: `|-…-|`, `|| caption ||`, decl head `[slug]` (§8).
#[test]
fn golden_08_table_decl() {
    assert_fixture("08-table-decl");
}

/// §16.3 fixture 09: column/th/td/tr/tbody/tfoot extras routing (§8).
#[test]
fn golden_09_table_layers() {
    assert_fixture("09-table-layers");
}
