use assert_cmd::cargo::cargo_bin_cmd;
use predicates::str::contains;
use predicates::Predicate;
use std::fs;
use std::path::{Path, PathBuf};

/// A real inline custom plugin (`kind = "inline"`), the same spec shape used by
/// `sandbox/universal`: `::[ep1] reason::` becomes an `<Epis>` component.
const INLINE_SPEC: &str = include_str!("fixtures/inline-epis.toml");

const PENDON_TOML: &str = r#"[[task]]
name = "Nested Component Demo"
input = "./src/[slug].md"
output = "./out/[slug].jsx"
plugin = "table,img,epis,markdown"
format = "solid"

[plugin-custom]
source = ["./plugins/index.toml"]
"#;

const PLUGIN_INDEX: &str = r#"[[plugin]]
id = "epis"
props = { name = "Epis" }
path = "epis.toml"
enabled = true
"#;

const SOURCE: &str = "!![Alt](https://x.test/a.webp) Caption with ::[ep1] caption reason:: end.\n\n| Produk | Stok |\n| --- | --- |\n| ::[ep2] cell reason:: | 15 |\n";

fn write_project(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("pendon-nested-component-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("src")).expect("src dir");
    fs::create_dir_all(dir.join("plugins")).expect("plugins dir");
    fs::write(dir.join("pendon.toml"), PENDON_TOML).expect("config");
    fs::write(dir.join("plugins/index.toml"), PLUGIN_INDEX).expect("plugin index");
    fs::write(dir.join("plugins/epis.toml"), INLINE_SPEC).expect("plugin spec");
    fs::write(dir.join("src/demo.md"), SOURCE).expect("source");
    dir
}

fn run_task(dir: &Path) {
    let mut cmd = cargo_bin_cmd!("pendon");
    cmd.current_dir(dir).arg("run").assert().success();
}

/// `true` when `needle` appears between an `open` tag and its matching `close`.
fn between(haystack: &str, open: &str, close: &str, needle: &str) -> bool {
    let mut rest = haystack;
    while let Some(start) = rest.find(open) {
        let after = &rest[start + open.len()..];
        let Some(end) = after.find(close) else {
            return false;
        };
        if after[..end].contains(needle) {
            return true;
        }
        rest = &after[end + close.len()..];
    }
    false
}

#[test]
fn custom_inline_component_nests_inside_figure_caption() {
    let dir = write_project("figure-caption");
    run_task(&dir);
    let out = fs::read_to_string(dir.join("out/demo.jsx")).expect("output file");

    assert!(contains("<figure").eval(&out), "figure emitted: {out}");
    assert!(
        contains("<figcaption>").eval(&out),
        "figcaption emitted: {out}"
    );
    assert!(
        between(&out, "<figcaption>", "</figcaption>", "<Epis"),
        "custom component must nest inside <figcaption>: {out}"
    );
    assert!(
        !out.contains("::[ep1]"),
        "inline marker must be consumed: {out}"
    );
}

#[test]
fn custom_inline_component_nests_inside_table_cell() {
    let dir = write_project("table-cell");
    run_task(&dir);
    let out = fs::read_to_string(dir.join("out/demo.jsx")).expect("output file");

    assert!(contains("<table").eval(&out), "table emitted: {out}");
    assert!(contains("<td").eval(&out), "cells emitted: {out}");
    assert!(
        between(&out, "<td", "</td>", "<Epis"),
        "custom component must nest inside <td>: {out}"
    );
    assert!(
        !out.contains("::[ep2]"),
        "inline marker must be consumed: {out}"
    );
    // The table is structured markup, not one escaped HTML string.
    assert!(
        !out.contains("&lt;td"),
        "table must not be escaped HTML: {out}"
    );
}
