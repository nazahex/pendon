//! `[[task.<element>.extract]]` — the CLI side of `plugin-extract`
//! (P.1 of `docs/issues/plugin-extract.md`, T4.2–T4.4 of
//! [`docs/workflows/TASKS.md`](../../docs/workflows/TASKS.md)).
//!
//! Extraction is a **reader**: the page renders exactly as it would without the
//! blocks, and the rows land in side files (`output` + `format`) and in the
//! automatic `export const` registrations of the Solid renderer (`export`).
//!
//! The elements are only addressable through the event stream, so a `cite`
//! block needs the citation layer (`[task.cite.custom]`) that turns a citation
//! into a structured node — the same prerequisite `plugin-cite`'s own custom
//! component has.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::str::contains;
use predicates::Predicate;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

/// A reference file in the shape the cite plugin reads: `id: { … }`, not a list.
const REFERENCES: &str = r#"alpha:
  type: book
  title: Alpha
  authors:
    - family: A
      given: A
  issuedDate:
    year: 2020
beta:
  type: article
  title: Beta
  authors:
    - family: B
      given: B
  issuedDate:
    year: 2021
gamma:
  type: book
  title: Gamma
  authors:
    - family: G
      given: G
  issuedDate:
    year: 2022
"#;

/// The citation layer plus its reference source — appended to every task below.
const CITE: &str = r#"
[task.cite]
reference_source = "external"
reference_file = "./src/reference.yaml"

[[task.cite.custom]]
name = "Cite"
template = "<Cite {...attrs} />"
"#;

fn project(name: &str, config: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("pendon-extract-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("src")).expect("src dir");
    fs::write(dir.join("src/reference.yaml"), REFERENCES).expect("references");
    fs::write(dir.join("pendon.toml"), config).expect("config");
    dir
}

fn run_in(dir: &Path) {
    cargo_bin_cmd!("pendon")
        .current_dir(dir)
        .arg("run")
        .assert()
        .success();
}

fn read(dir: &Path, path: &str) -> String {
    fs::read_to_string(dir.join(path)).unwrap_or_else(|error| panic!("{path}: {error}"))
}

/// Append the citation layer to the task the test just wrote.
fn with_cite(dir: &Path) {
    fs::write(
        dir.join("pendon.toml"),
        format!("{}{CITE}", read(dir, "pendon.toml")),
    )
    .expect("config");
}

/// P.1.a/b/c in one task: a type-scoped block writing two files, and two blocks
/// sharing one export name, where the later block wins.
#[test]
fn p1_scoped_files_and_an_overridden_export() {
    let dir = project(
        "p1",
        r#"[[task]]
name = "Extract"
input = "./src/doc.mdp"
output = "./out/doc.jsx"
plugin = "frontmatter,cite,extract,markdown"
format = "solid"

[[task.cite.extract]]
type = ["CiteA", "CiteB"]
output = ["./out/scoped.json", "./out/scoped.yaml"]
format = ["json", "yaml"]
only = ["reference_id", "citation_number"]

[[task.cite.extract]]
export = "citationsC"

[[task.cite.extract]]
output = "./out/rest.yaml"
format = "yaml"
export = "citationsC"
recommended = true
only = ["reference_id", "citation_number"]
omit = ["reference_id"]
"#,
    );
    fs::write(
        dir.join("src/doc.mdp"),
        "A [^alpha]@@CiteA, B [^beta]@@CiteB, C [^gamma].\n",
    )
    .expect("source");
    with_cite(&dir);

    run_in(&dir);

    // P.1.a — one flat array of the two scoped rows, only the `only` fields.
    let scoped: Value =
        serde_json::from_str(&read(&dir, "out/scoped.json")).expect("scoped json parses");
    assert_eq!(
        scoped,
        serde_json::json!([
            { "citation_number": "1", "reference_id": "alpha" },
            { "citation_number": "2", "reference_id": "beta" },
        ])
    );
    // `format` wins over the extension: the second destination is YAML.
    let yaml = read(&dir, "out/scoped.yaml");
    assert!(contains("reference_id: alpha").eval(&yaml), "yaml = {yaml}");
    assert!(contains("reference_id: beta").eval(&yaml), "yaml = {yaml}");

    // P.1.b — no `type` is `_`: the safety net for the types no block names, so
    // only the untyped `gamma` citation.
    assert_eq!(read(&dir, "out/rest.yaml"), "- citation_number: '3'\n");

    // P.1.c — the later block owns `citationsC`: one binding, its value.
    let out = read(&dir, "out/doc.jsx");
    assert_eq!(
        out.matches("export const citationsC").count(),
        1,
        "out = {out}"
    );
    assert!(
        contains(r#"export const citationsC = [{"citation_number":"3"}];"#).eval(&out),
        "out = {out}"
    );
    // P.2 — extraction never rewrites the stream: the page still renders.
    assert!(contains("<Cite").eval(&out), "out = {out}");
}

/// `output` is project-root-relative like the task `output`, and its `[segment]`
/// placeholders are substituted per page.
#[test]
fn a_dynamic_side_path_is_written_per_page() {
    let dir = project(
        "dynamic",
        r#"[[task]]
name = "Extract"
input = "./src/[slug]/doc.mdp"
output = "./out/[slug]/doc.jsx"
plugin = "frontmatter,cite,extract,markdown"
format = "solid"

[[task.cite.extract]]
type = "CiteA"
output = "./out/[slug].cite.json"
format = "json"
only = ["reference_id"]
"#,
    );
    fs::create_dir_all(dir.join("src/alpha")).expect("alpha dir");
    fs::create_dir_all(dir.join("src/beta")).expect("beta dir");
    fs::write(dir.join("src/alpha/doc.mdp"), "A [^alpha]@@CiteA.\n").expect("alpha");
    fs::write(dir.join("src/beta/doc.mdp"), "B [^beta]@@CiteB.\n").expect("beta");
    with_cite(&dir);

    run_in(&dir);

    assert_eq!(
        serde_json::from_str::<Value>(&read(&dir, "out/alpha.cite.json")).expect("alpha json"),
        serde_json::json!([{ "reference_id": "alpha" }])
    );
    // `CiteB` is not `CiteA`, so this page's block matches nothing at all.
    assert!(
        !dir.join("out/beta.cite.json").exists(),
        "a block that matches nothing writes no file"
    );
}

/// A path without a placeholder is one file for the whole task: every page that
/// resolves to it replaces the previous content, so the file holds one page's
/// rows — never a merge (P.1). Pages are walked in directory order, so *which*
/// page wins is not alphabetical; only "one of them" is promised.
#[test]
fn a_static_side_path_is_overwritten_not_merged() {
    let dir = project(
        "static",
        r#"[[task]]
name = "Extract"
input = "./src/[slug]/doc.mdp"
output = "./out/[slug]/doc.jsx"
plugin = "frontmatter,cite,extract,markdown"
format = "solid"

[[task.cite.extract]]
output = "./out/all-cite.json"
format = "json"
only = ["reference_id"]
"#,
    );
    fs::create_dir_all(dir.join("src/alpha")).expect("alpha dir");
    fs::create_dir_all(dir.join("src/beta")).expect("beta dir");
    fs::write(dir.join("src/alpha/doc.mdp"), "A [^alpha].\n").expect("alpha");
    fs::write(dir.join("src/beta/doc.mdp"), "B [^beta].\n").expect("beta");
    with_cite(&dir);

    run_in(&dir);

    let rows: Value =
        serde_json::from_str(&read(&dir, "out/all-cite.json")).expect("the file parses");
    let rows = rows.as_array().expect("an array of rows");
    assert_eq!(rows.len(), 1, "the shared path holds one page: {rows:?}");
    let id = rows[0]["reference_id"].as_str().expect("a reference_id");
    assert!(
        id == "alpha" || id == "beta",
        "the file is written by a real page: {id}"
    );
}

/// ADR-0009 rule 5: only `solid` has module scope. The side file is still
/// written, the `export const` is dropped.
#[test]
fn export_is_dropped_for_a_non_solid_task() {
    let dir = project(
        "html",
        r#"[[task]]
name = "Extract"
input = "./src/doc.mdp"
output = "./out/doc.html"
plugin = "frontmatter,cite,extract,markdown"
format = "html"

[[task.cite.extract]]
output = "./out/cites.json"
format = "json"
export = "citationsC"
"#,
    );
    fs::write(dir.join("src/doc.mdp"), "A [^alpha].\n").expect("source");
    with_cite(&dir);

    run_in(&dir);

    assert!(
        dir.join("out/cites.json").exists(),
        "the file destination is format-independent"
    );
    let out = read(&dir, "out/doc.html");
    assert!(!out.contains("export const citationsC"), "out = {out}");
}

/// P.2 / §6.3: `export = "frontmatter"` is an ordinary registration, and the
/// automatic one is emitted **first**, so one line survives and the block's value
/// wins (ADR-0009 D-6: last wins, value and position alike). The document keeps its
/// frontmatter node; only the `export const` it used to carry is replaced.
#[test]
fn an_extract_export_overrides_the_automatic_frontmatter() {
    let dir = project(
        "frontmatter",
        r#"[[task]]
name = "Extract"
input = "./src/doc.mdp"
output = "./out/doc.jsx"
plugin = "frontmatter,section,heading,number,markdown,extract"
format = "solid"

[[task.heading.extract]]
export = "frontmatter"
only = ["inner_text"]
"#,
    );
    fs::write(dir.join("src/doc.mdp"), "---\ntitle: Demo\n---\n\n# One\n").expect("source");

    run_in(&dir);

    let out = read(&dir, "out/doc.jsx");
    let lines: Vec<&str> = out
        .lines()
        .filter(|line| line.starts_with("export const frontmatter"))
        .collect();
    assert_eq!(lines.len(), 1, "one line for one name: {out}");
    let line = lines[0];
    assert!(
        contains(r#""inner_text":"One""#).eval(line),
        "the block's rows are the value: {line}"
    );
    assert!(
        !line.contains("Demo"),
        "the automatic `{{title: Demo}}` is gone: {line}"
    );
    let at = out.find(line).expect("the export line");
    let body = out.find("export default function").expect("the component");
    assert!(at < body, "exports precede the component: {out}");
}

/// T4.6: a side file is an **output**, so the cache guards it like the page. A run
/// that finds the page fresh still has to restore a deleted side file — otherwise
/// the data is silently gone until someone touches the source.
#[test]
fn a_deleted_side_file_is_rewritten_on_the_next_run() {
    let dir = project(
        "cache",
        r#"[[task]]
name = "Extract"
input = "./src/doc.mdp"
output = "./out/doc.jsx"
plugin = "frontmatter,cite,extract,markdown"
format = "solid"

[[task.cite.extract]]
output = "./out/cites.json"
format = "json"
only = ["reference_id"]
"#,
    );
    fs::write(dir.join("src/doc.mdp"), "A [^alpha].\n").expect("source");
    with_cite(&dir);
    run_in(&dir);

    let side = read(&dir, "out/cites.json");
    assert!(contains("alpha").eval(&side), "side = {side}");
    fs::remove_file(dir.join("out/cites.json")).expect("delete the side file");

    run_in(&dir);

    assert_eq!(
        read(&dir, "out/cites.json"),
        side,
        "the cache must rebuild the side file, not skip the page"
    );
}
