use assert_cmd::cargo::cargo_bin_cmd;
use predicates::str::contains;
use predicates::Predicate;
use std::fs;
use std::path::PathBuf;

/// Real custom plugin specs, the same shape used by `sandbox/universal`: an
/// inline (`kind = "inline"`) and a block (`kind = "block"`) component.
const INLINE_SPEC: &str = include_str!("fixtures/inline-epis.toml");
const BLOCK_SPEC: &str = include_str!("fixtures/block-aside.toml");

const PENDON_TOML: &str = r#"[[task]]
name = "List Render Demo"
input = "./src/[slug].md"
output = "./out/[slug].jsx"
plugin = "table,img,epis,aside,markdown"
format = "solid"

[plugin-custom]
source = ["./plugins/index.toml"]
"#;

const PLUGIN_INDEX: &str = r#"[[plugin]]
id = "epis"
props = { name = "Epis" }
path = "epis.toml"
enabled = true

[[plugin]]
id = "aside"
props = { name = "Aside" }
path = "aside.toml"
enabled = true
"#;

/// One plain list, one ordered list, one quote list, one list inside a block
/// component, one list inside a table cell (`\n` splits one cell into several
/// lines) and one list item whose first child is an inline component followed by
/// a nested list.
const SOURCE: &str = r#"- Alpha
- Beta

1. Satu
2. Dua

> Quote intro
> - Quoted alpha
> - Quoted beta

:::note

- Aside alpha
- Aside beta

:::

| Kolom | Isi |
| --- | --- |
| Plain | - Cell alpha\n- Cell beta |
| Nested | - ::[ep2] cell reason::\n  - Nested cell |
| Code | `cell code` |

- ::[ep1] item reason::
  - Nested under component

`inline code` starts the line.
"#;

fn write_project(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("pendon-list-render-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("src")).expect("src dir");
    fs::create_dir_all(dir.join("plugins")).expect("plugins dir");
    fs::write(dir.join("pendon.toml"), PENDON_TOML).expect("config");
    fs::write(dir.join("plugins/index.toml"), PLUGIN_INDEX).expect("plugin index");
    fs::write(dir.join("plugins/epis.toml"), INLINE_SPEC).expect("inline spec");
    fs::write(dir.join("plugins/aside.toml"), BLOCK_SPEC).expect("block spec");
    fs::write(dir.join("src/demo.md"), SOURCE).expect("source");
    dir
}

fn render(name: &str) -> String {
    let dir = write_project(name);
    let mut cmd = cargo_bin_cmd!("pendon");
    cmd.current_dir(&dir).arg("run").assert().success();
    fs::read_to_string(dir.join("out/demo.jsx")).expect("output file")
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
fn lists_render_as_structured_markup() {
    let out = render("structured");

    assert!(contains("<ul>").eval(&out), "bullet list emitted: {out}");
    assert!(contains("<ol").eval(&out), "ordered list emitted: {out}");
    assert!(contains("<blockquote>").eval(&out), "quote emitted: {out}");
    assert!(
        contains("<Aside").eval(&out),
        "block component emitted: {out}"
    );
    assert!(
        contains("<Epis").eval(&out),
        "inline component emitted: {out}"
    );

    // One `<li>` per item across every list (plain, ordered, quote, block
    // component, table cells) and balanced open/close tags.
    let items = out.matches("<li>").count();
    assert_eq!(items, 14, "one <li> per list item: {out}");
    assert_eq!(out.matches("</li>").count(), items, "balanced <li>: {out}");
    assert_eq!(out.matches("<ul>").count(), 8, "nested <ul> count: {out}");
    assert_eq!(out.matches("</ul>").count(), 8, "balanced <ul>: {out}");

    assert!(
        out.contains("<ol start={1}>\n<li>Satu</li>\n<li>Dua</li>\n</ol>"),
        "ordered list keeps its items: {out}"
    );
}

/// A code span may open at column 0, both in a paragraph and in a table cell
/// (a cell is parsed as its own document, so a cell that starts with a code span
/// hits the same path).
#[test]
fn inline_code_at_line_start_is_not_left_literal() {
    let out = render("inline-code");

    assert!(
        out.contains("<p><code>inline code</code> starts the line."),
        "paragraph code span: {out}"
    );
    assert!(
        out.contains("<td><code>cell code</code></td>"),
        "cell code span: {out}"
    );
    assert!(!out.contains('`'), "literal backticks leaked: {out}");
}

#[test]
fn list_markers_never_leak_into_text() {
    let out = render("markers");

    for leaked in [
        "- Alpha",
        "- Beta",
        "1. Satu",
        "2. Dua",
        "- Quoted alpha",
        "- Aside alpha",
        "- Cell alpha",
        "- Nested cell",
        "- Nested under component",
        "> Quote intro",
        "::[ep1]",
        "::[ep2]",
    ] {
        assert!(!out.contains(leaked), "marker `{leaked}` leaked: {out}");
    }
    // The `\n` cell escape is consumed by the table, never rendered.
    assert!(!out.contains("\\n"), "cell escape leaked: {out}");
}

#[test]
fn lists_nest_inside_quotes_components_and_cells() {
    let out = render("nesting");

    assert!(
        between(
            &out,
            "<blockquote>",
            "</blockquote>",
            "<li>Quoted alpha</li>"
        ),
        "quote list nests inside <blockquote>: {out}"
    );
    assert!(
        between(&out, "<Aside", "</Aside>", "<li>Aside alpha</li>"),
        "list nests inside the block component: {out}"
    );
    assert!(
        between(&out, "<td", "</td>", "<li>Cell alpha</li>"),
        "cell list nests inside <td>: {out}"
    );
    assert!(
        between(&out, "<td", "</td>", "<Epis level={2}>cell reason</Epis>"),
        "inline component nests inside the cell list: {out}"
    );
}

#[test]
fn inline_component_siblings_keep_following_nested_list() {
    let out = render("siblings");

    // The nested list must stay a sibling of the component inside the same
    // `<li>` instead of being flattened into its text.
    assert!(
        out.contains(
            "<li><Epis level={1}>item reason</Epis><ul>\n<li>Nested under component</li>\n</ul>\n</li>"
        ),
        "nested list must follow the inline component: {out}"
    );
    assert!(
        out.contains("<td><ul>\n<li><Epis level={2}>cell reason</Epis><ul>\n<li>Nested cell</li>\n</ul>\n</li>\n</ul>\n</td>"),
        "cell list keeps its nested list: {out}"
    );
}
