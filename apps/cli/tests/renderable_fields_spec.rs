//! Renderable fields end to end (`;;…;;` / `;;;…;;;`, §6.7, ADR-0021).
//!
//! The grammar layer is pinned in `crates/extra/tests/renderable_fields_spec.rs`;
//! this file pins what the *chain* makes of it: the resolve stage renders the
//! payload through the page's own chain, and the field carries a fragment — the
//! same value a `{{{mdp[var] …}}}` binding produces — so every renderer shows
//! nodes, never the transport sentinel.

use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use std::path::PathBuf;

/// A one-page project; `body` is the page, `plugins` the chain.
fn write_project(name: &str, plugins: &str, body: &str) -> PathBuf {
    write_project_as(name, plugins, "solid", body)
}

/// The same project rendered in another format (`html`, `json`, …).
fn write_project_as(name: &str, plugins: &str, format: &str, body: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("pendon-renderable-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("src")).expect("src dir");
    fs::create_dir_all(dir.join("out")).expect("out dir");
    fs::write(dir.join("src/doc.mdp"), body).expect("source");
    let extension = if format == "html" { "html" } else { "jsx" };
    fs::write(
        dir.join("pendon.toml"),
        format!(
            r#"[[task]]
name = "Doc"
input = "./src/doc.mdp"
output = "./out/doc.{extension}"
plugin = "{plugins}"
format = "{format}"
"#
        ),
    )
    .expect("config");
    dir
}

/// Renders the project and returns the emitted page.
fn render(dir: &PathBuf) -> String {
    cargo_bin_cmd!("pendon")
        .current_dir(dir)
        .arg("run")
        .assert()
        .success();
    let page = if dir.join("out/doc.html").exists() {
        "out/doc.html"
    } else {
        "out/doc.jsx"
    };
    fs::read_to_string(dir.join(page)).expect("output")
}

const FRONTMATTER: &str =
    "---\nreferences:\n  book: {id: book, type: book, title: A Book}\n---\n\n";

#[test]
fn an_inline_renderable_becomes_a_fragment_prop() {
    let dir = write_project(
        "inline",
        "frontmatter,anchor,cite,wiki,markdown",
        &format!("{FRONTMATTER}@[Card](/docs)@@anchorA{{note: ;;lorem *ipsum* and [[Wiki]];;}}\n"),
    );
    let jsx = render(&dir);

    // §6.7: the value is nodes, not the authored `;;` spelling.
    assert!(
        jsx.contains("note={(<>lorem <em>ipsum</em> and <a href=\"/Wiki\""),
        "{jsx}"
    );
    // The inline form drops the payload's own `<p>` wrapper.
    assert!(!jsx.contains("note={(<><p>"), "{jsx}");
    // The transport sentinel never reaches the output.
    assert!(!jsx.contains('\u{E003}'), "{jsx}");
    assert!(!jsx.contains(";;lorem"), "{jsx}");
}

#[test]
fn a_block_renderable_keeps_its_block_structure() {
    let dir = write_project(
        "block",
        "anchor,markdown",
        "@[Card](/docs)@@anchorA{body: \";;;## Head\\n\\n- one;;;\"}\n",
    );
    let jsx = render(&dir);

    assert!(jsx.contains("body={(<><h2>Head</h2>"), "{jsx}");
    assert!(jsx.contains("<li>one</li>"), "{jsx}");
}

#[test]
fn a_builtin_element_attribute_carries_a_fragment() {
    // §6.5's other target: a **built-in** element writes the value as an
    // attribute expression, so the fragment must not be stringified — it is a
    // value, and `note={(<>…</>)}` is how Solid spells one.
    let dir = write_project(
        "builtin",
        "directive,markdown",
        "::plain[a](b)@@plain{note: ;;hi *x*;;} body::\n",
    );
    let jsx = render(&dir);

    assert!(jsx.contains("note={(<>hi <em>x</em></>)}"), "{jsx}");
    assert!(!jsx.contains('\u{E003}'), "{jsx}");
}

#[test]
fn a_nested_position_carries_a_fragment_too() {
    // D-RF-6: every position is legal, `specs` objects and arrays included. Their
    // value travels as JSON, so the field is a `{"$mdp": …}` wrapper *inside* it —
    // resolved in place, and the fragment nests the way a bound `mdp` does.
    let dir = write_project(
        "nested-position",
        "anchor,markdown",
        "@[Card](/docs)@@anchorA{specs: {deep: ;;x *y*;;}, list: [;;a *b*;;, \"plain\"]}\n",
    );
    let jsx = render(&dir);

    assert!(jsx.contains("deep: (<>x <em>y</em></>)"), "{jsx}");
    assert!(jsx.contains("[(<>a <em>b</em></>), \"plain\"]"), "{jsx}");
    assert!(!jsx.contains("$mdp"), "the transport leaked: {jsx}");
    assert!(!jsx.contains('\u{E003}'), "{jsx}");
}

#[test]
fn a_renderable_in_html_is_escaped_markup_in_the_attribute() {
    // Neither HTML mode can put markup *in* an attribute (an attribute is text by
    // definition), so the fragment renders compactly and is escaped — the same
    // contract a bound `mdp` value has (`renderer-html`'s own test).
    let dir = write_project_as(
        "html",
        "anchor,markdown",
        "html",
        "@[Card](/docs)@@anchorA{note: ;;hi *there*;;}\n",
    );
    let html = render(&dir);

    assert!(
        html.contains("note=\"hi &lt;em&gt;there&lt;/em&gt;\""),
        "{html}"
    );
    assert!(!html.contains("$fragment"), "{html}");
}

#[test]
fn a_renderable_is_a_node_list_in_json() {
    // The JSON/AST renderers keep the list: a fragment is structure, and a
    // consumer that wants text joins the nodes itself.
    let dir = write_project_as(
        "json",
        "anchor,markdown",
        "json",
        "@[Card](/docs)@@anchorA{note: ;;hi *there*;;}\n",
    );
    let json = render(&dir);

    assert!(json.contains(r#""$fragment":["#), "{json}");
    assert!(json.contains(r#""type":"Emphasis""#), "{json}");
    assert!(!json.contains('\u{E003}'), "{json}");
}

#[test]
fn a_renderable_inside_a_renderable_resolves_too() {
    // Nesting is spelled with escapes, because a bare `;;` would close the outer
    // span (a `;` run ≥ the opener always closes — §6.7).
    let dir = write_project(
        "nested",
        "anchor,markdown",
        "@[Card](/docs)@@anchorA{note: ;;outer *o* \\;\\;@[in](/u)@@anchorB{v: \\;\\;inner *i*\\;\\;} \\;\\; tail;;}\n",
    );
    let jsx = render(&dir);

    assert!(jsx.contains("outer <em>o</em>"), "{jsx}");
    assert!(jsx.contains("v={(<>inner <em>i</em></>)}"), "{jsx}");
    assert!(!jsx.contains('\u{E003}'), "{jsx}");
}

#[test]
fn a_literal_consumer_sees_the_authored_text() {
    // `.class` is not a field value, so `;;` there stays literal — nothing
    // disappears and nothing becomes a fragment where a string is required.
    let dir = write_project(
        "literal",
        "anchor,markdown",
        "@[Card](/docs)@@anchorA{class: ;;x;;, note: ;;y;;}\n",
    );
    let jsx = render(&dir);

    assert!(jsx.contains("class=\";;x;;\""), "{jsx}");
    assert!(jsx.contains("note={(<>y</>)}"), "{jsx}");
}

#[test]
fn a_citation_inside_a_field_is_numbered_by_source_position() {
    // D-RF-8 (option B): the field is rendered after the chain, but its citation
    // must still take the number of its **source** position. `[^a]` is written on
    // the head's line, before the body's `[^b]`, so `a` is 1 and `b` is 2.
    let dir = write_project(
        "numbering",
        "frontmatter,anchor,cite,markdown",
        "---\nreferences:\n  a: {id: a, type: book, title: A}\n  b: {id: b, type: book, title: B}\n---\n\n\
         @[Card](/docs)@@anchorA{note: ;;see [^a];;}\n\n\
         Body text cites [^b] here.\n",
    );
    let jsx = render(&dir);

    assert!(jsx.contains("href=\"#1-a\""), "{jsx}");
    assert!(jsx.contains("href=\"#2-b\""), "{jsx}");
    // The number is a display ordinal: the field renders last, so its backlink
    // anchor is the third physical occurrence even though its number is 1.
    assert!(!jsx.contains('\u{E003}'), "{jsx}");
}

#[test]
fn a_preset_position_keeps_the_authored_text() {
    // `slug` is the heading's own field *by default*, but the position is not
    // reserved (D-RF-6): the value is legal, and the element — which needs text
    // for its id — reads the authored spelling instead of a fragment. Nothing
    // is refused, and no sentinel leaks.
    let dir = write_project("preset", "heading,markdown", "##[;;a *b*;;](\"T\")\n");
    let jsx = render(&dir);

    assert!(!jsx.contains('\u{E003}'), "{jsx}");
    assert!(jsx.contains(";;a *b*;;"), "{jsx}");
}
