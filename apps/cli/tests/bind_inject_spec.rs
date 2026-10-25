//! `[[task.bind.inject]]` — the export side of `plugin-bind`
//! (§6.1 of `docs/workflows/numbering-extract-type-export.md`, ADR-0009).
//!
//! A page can publish a data **file** as `export const <name>` without naming it
//! in the Markdown. The conversion is the `{{{lang[var] … }}}` converter, so the
//! value has the same JSON shape whichever way it entered.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::str::contains;
use predicates::Predicate;
use std::fs;
use std::path::{Path, PathBuf};

/// A project with two data files and one page.
fn write_project(name: &str, task_extra: &str, format: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("pendon-bind-inject-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("src/alpha")).expect("src dir");
    fs::create_dir_all(dir.join("src/beta")).expect("src dir");
    fs::write(
        dir.join("src/alpha/rows.yaml"),
        "- name: Alpha\n  score: 1\n- name: Beta\n  score: 2\n",
    )
    .expect("alpha rows");
    fs::write(
        dir.join("src/beta/rows.yaml"),
        "- name: Gamma\n  score: 3\n",
    )
    .expect("beta rows");
    fs::write(dir.join("settings.toml"), "title = \"Demo\"\ncount = 2\n").expect("settings");
    fs::write(dir.join("src/doc.md"), "# Title\n\nBody.\n").expect("source");
    fs::write(
        dir.join("pendon.toml"),
        format!(
            r#"[[task]]
name = "Doc"
input = "./src/doc.md"
output = "./out/doc.jsx"
plugin = "bind,markdown"
format = "{format}"

[task.bind]
{task_extra}
"#
        ),
    )
    .expect("config");
    dir
}

fn read_out(dir: &Path) -> String {
    fs::read_to_string(dir.join("out/doc.jsx")).expect("task output")
}

fn run_in(dir: &Path) {
    let mut cmd = cargo_bin_cmd!("pendon");
    cmd.current_dir(dir).arg("run").assert().success();
}

/// §6.3 acceptance: a `.toml` and a `.yaml` source each become one module-level
/// binding, both before `export default function`.
#[test]
fn injected_yaml_and_toml_sources_become_exports() {
    let dir = write_project(
        "yaml-toml",
        "inject = [\n  { source = \"./src/alpha/rows.yaml\", export = \"fooData\" },\n  { source = \"./settings.toml\" },\n]\n",
        "solid",
    );
    run_in(&dir);

    let out = read_out(&dir);
    // The `.yaml` payload keeps the block converter's shape (an array of objects).
    assert!(
        contains(
            r#"export const fooData = [{"name":"Alpha","score":1},{"name":"Beta","score":2}];"#
        )
        .eval(&out),
        "out = {out}"
    );
    // No `export` key: the file's stem is the binding name.
    assert!(
        contains(r#"export const settings = {"count":2,"title":"Demo"};"#).eval(&out),
        "out = {out}"
    );
    let exports_at = out.find("export const fooData").expect("inject export");
    let body_at = out.find("export default function").expect("component");
    assert!(exports_at < body_at, "out = {out}");
}

/// Dynamic segments resolve per page, so one task can publish a different file —
/// and therefore a different payload — for each match.
#[test]
fn inject_source_substitutes_the_task_capture() {
    let dir = std::env::temp_dir().join("pendon-bind-inject-dynamic");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("src/alpha")).expect("src dir");
    fs::create_dir_all(dir.join("src/beta")).expect("src dir");
    fs::write(dir.join("src/alpha/doc.md"), "# Alpha\n").expect("alpha page");
    fs::write(dir.join("src/beta/doc.md"), "# Beta\n").expect("beta page");
    fs::write(dir.join("src/alpha/rows.json"), r#"[{"name":"Alpha"}]"#).expect("alpha rows");
    fs::write(dir.join("src/beta/rows.json"), r#"[{"name":"Beta"}]"#).expect("beta rows");
    fs::write(
        dir.join("pendon.toml"),
        r#"[[task]]
name = "Doc"
input = "./src/[foo]/doc.md"
output = "./out/[foo]/doc.jsx"
plugin = "bind,markdown"
format = "solid"

[[task.bind.inject]]
source = "./src/[foo]/rows.json"
export = "rows"
"#,
    )
    .expect("config");

    run_in(&dir);

    let alpha = fs::read_to_string(dir.join("out/alpha/doc.jsx")).expect("alpha out");
    let beta = fs::read_to_string(dir.join("out/beta/doc.jsx")).expect("beta out");
    assert!(
        contains(r#"export const rows = [{"name":"Alpha"}];"#).eval(&alpha),
        "alpha = {alpha}"
    );
    assert!(
        contains(r#"export const rows = [{"name":"Beta"}];"#).eval(&beta),
        "beta = {beta}"
    );
}

/// ADR-0009 rule 5: HTML has no module scope, so the payload is dropped
/// silently instead of leaking a stray `export const`.
#[test]
fn inject_exports_are_dropped_for_a_non_solid_task() {
    let dir = write_project(
        "html",
        "inject = [\n  { source = \"./settings.toml\", export = \"settings\" },\n]\n",
        "html",
    );
    let mut cmd = cargo_bin_cmd!("pendon");
    cmd.current_dir(&dir).arg("run").assert().success();

    let out = fs::read_to_string(dir.join("out/doc.jsx")).expect("task output");
    assert!(!out.contains("export const settings"), "out = {out}");
}

/// §6.3 acceptance: an invalid identifier names the key **and** the value.
#[test]
fn an_invalid_export_name_is_a_task_error() {
    let dir = write_project(
        "invalid-name",
        "inject = [\n  { source = \"./settings.toml\", export = \"data-x\" },\n]\n",
        "solid",
    );
    let mut cmd = cargo_bin_cmd!("pendon");
    cmd.current_dir(&dir)
        .arg("run")
        .assert()
        .failure()
        .stderr(contains(
            "bind.inject 'data-x' is not a valid JavaScript identifier",
        ));
    assert!(
        !dir.join("out/doc.jsx").exists(),
        "a task error renders nothing"
    );
}

/// §6.3 acceptance: the injected file is a dependency, so editing it re-renders
/// a page the cache would otherwise skip.
#[test]
fn editing_an_injected_source_invalidates_the_cache() {
    let dir = write_project(
        "cache",
        "inject = [\n  { source = \"./settings.toml\", export = \"settings\" },\n]\n",
        "solid",
    );
    run_in(&dir);
    assert!(
        contains(r#"export const settings = {"count":2,"title":"Demo"};"#).eval(&read_out(&dir)),
        "first run"
    );

    fs::write(
        dir.join("settings.toml"),
        "title = \"Changed\"\ncount = 7\n",
    )
    .expect("settings");
    run_in(&dir);

    let out = read_out(&dir);
    assert!(
        contains(r#"export const settings = {"count":7,"title":"Changed"};"#).eval(&out),
        "the cached page was not re-rendered: {out}"
    );
}
