//! S2-4.1: **extras are universal** — every element head and every decorator
//! reads the same `{…}` grammar, so the five specs positions (§6.1) and the
//! `%%numberType` marker (§10.5) reach every construct.
//!
//! The blockquote appears once, as a decorator: it has no head of its own, so a
//! `@@type{…}` inside the `>` is content and stays literal (§9.2, `S2-4.3`).
//!
//! One document per row, one project, one render: the table below is the audit,
//! and each row asserts what its element's own projection (§6.3, §7.x) makes of
//! the shared head. The head text is identical everywhere —
//! `` {`slug-x`, "Title X", .cls, #elem-x, style: "--w: 1px", %%%numberA} `` —
//! so a row that loses `class` / the id / the style / the quote is a head the
//! element did not read, and `@@` surviving in the output means the head fell
//! back to literal text (§4.3).
//!
//! Element-specific projection is *not* what this test checks (a citation renames
//! `#id` to `cite-id`, an image carries the quote as `data-title`, a heading lets
//! its own head win the title): the per-row expectations are the element's
//! documented output, while the *same* two checks — the positions are accepted,
//! nothing is literal — run for every row.

use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use std::path::{Path, PathBuf};

/// The shared head: all five specs positions plus a numbering marker.
const HEAD: &str = r#"{`slug-x`, "Title X", .cls, #elem-x, style: "--w: 1px", %%%numberA}"#;

/// One element head (or decorator) under audit.
struct Case {
    /// File stem, also the label of the row.
    name: &'static str,
    /// The task's `plugin` list — precedence differs per element (§12.1).
    plugins: &'static str,
    /// The fixture source; `{HEAD}` is replaced by [`HEAD`].
    source: &'static str,
    /// Extra TOML placed directly after this task's header
    /// (`[[task.cite.custom]]`, `[[task.number.custom]]`, …). Blocks must follow
    /// their own task: a `[[task.<x>.*]]` table attaches to the most recent
    /// `[[task]]`.
    extra: &'static str,
    /// What the element's projection makes of the shared head (verified against
    /// a real render before being written down).
    expect: &'static [&'static str],
}

const CASES: &[Case] = &[
    Case {
        name: "anchor",
        plugins: "anchor,markdown",
        source: "@[A](/a)@@anchorX{HEAD}\n",
        extra: "",
        expect: &[
            r#"class="cls""#,
            r#"id="elem-x""#,
            r#"title="Title X""#,
            r#"type="anchorX""#,
        ],
    },
    Case {
        name: "image",
        plugins: "image,markdown",
        source: "![Alt](pic.png)@@imageX{HEAD}\n",
        extra: "",
        expect: &[
            r#"alt="Alt""#,
            r#"class="cls""#,
            r#"data-title="Title X""#,
            r#"id="elem-x""#,
            r#"type="imageX""#,
        ],
    },
    Case {
        name: "cite",
        plugins: "cite,anchor,markdown",
        source: "[^alpha]@@citeX{HEAD}\n",
        // §7.3: the citation layer needs a component to become a structured node.
        extra: r#"
[task.cite]
reference_source = "external"
reference_file = "./src/reference.yaml"

[[task.cite.custom]]
name = "Cite"
imports = "import { Cite } from '@comp/Cite'"
template = "<Cite {...attrs} />"
"#,
        // §7.3/§6.3: the reference the head assigns, the citation's own id field,
        // and the quote as its `title`.
        expect: &[
            r#"reference_id="alpha""#,
            r#"cite-id="elem-x""#,
            r#"class="cls""#,
            r#"title="Title X""#,
        ],
    },
    Case {
        name: "heading",
        plugins: "heading,markdown",
        source: "##[h](\"HT\")@@headingX{HEAD} Heading text\n",
        extra: "",
        // The heading's own `("HT")` wins the title (§6.2); the rest is extras.
        expect: &[
            r#"class="cls""#,
            r#"id="elem-x""#,
            r#"title="HT""#,
            "Heading text",
        ],
    },
    Case {
        name: "wiki",
        plugins: "wiki,markdown",
        source: "[[Page]]@@wikiX{HEAD}\n",
        extra: "",
        // The link owns `href` / `title`; the two specs items keep their paths.
        expect: &[
            r#"class="cls""#,
            r#"id="elem-x""#,
            r#"title="Page""#,
            r#"specs.tick": "slug-x""#,
        ],
    },
    Case {
        name: "marker",
        plugins: "marker,markdown",
        source: "{{markerX}}{HEAD}\n",
        extra: "",
        // A marker contracts nothing, so both specs items stay dotted (§10.1).
        expect: &[
            r#"class="cls""#,
            r#"id="elem-x""#,
            r#"specs.quote": "Title X""#,
            r#"type="markerX""#,
        ],
    },
    Case {
        name: "directive-inline",
        plugins: "directive,markdown",
        source: "::directiveX{HEAD} body::\n",
        extra: "",
        expect: &[
            r#"class="cls""#,
            r#"id="elem-x""#,
            r#"specs.tick": "slug-x""#,
            r#"type="directiveX""#,
        ],
    },
    Case {
        name: "directive-block",
        plugins: "directive,markdown",
        source: "==directiveY{HEAD}\nblock body\n==\n",
        extra: "",
        expect: &[
            r#"class="cls""#,
            r#"specs.quote": "Title X""#,
            r#"type="directiveY""#,
        ],
    },
    Case {
        name: "table-head",
        plugins: "table,markdown",
        source: "@@tableX{HEAD}\n| A | B |\n| :--- | :--- |\n| a | c |\n",
        extra: "",
        // §8/§9.1: the `<table>` head is the decorator line above the table, so
        // the quote lands as `title` and every specs position reaches the node.
        expect: &[
            r#"class="cls""#,
            r#"id="elem-x""#,
            r#"title="Title X""#,
            r#"type="tableX""#,
        ],
    },
    Case {
        name: "table-cell",
        plugins: "table,markdown",
        source: "| A | B |\n| :---@@cellX{HEAD} | :--- |\n| a | c |\n",
        extra: "",
        // §8: a cell head in the alignment row routes the column's `th` and `td`.
        expect: &[
            r#"class="cls""#,
            r#"id="elem-x""#,
            r#"title="Title X""#,
            r#"type="cellX""#,
        ],
    },
    Case {
        name: "quote-decorator",
        plugins: "blockquote,markdown",
        source: "@@quoteY{HEAD}\n> quoted\n",
        extra: "",
        // §9.1/§9.2: the decorator line above the quote decorates it.
        expect: &[r#"class="cls""#, r#"id="elem-x""#, r#"type="quoteY""#],
    },
    Case {
        name: "list-container",
        plugins: "list,markdown",
        source: "@@listX{HEAD}\n- one\n",
        extra: "",
        // §9.3 L1: a container decorator decorates the `<ul>`.
        expect: &[r#"class="cls""#, r#"id="elem-x""#, r#"type="listX""#],
    },
    Case {
        name: "section",
        plugins: "section,heading,markdown",
        source: "@@sectionX{HEAD}\n## Sect\n",
        extra: "",
        // §9.5: a decorator above a heading decorates the **section**.
        expect: &[r#"class="cls""#, r#"id="elem-x""#, r#"type="sectionX""#],
    },
    Case {
        name: "code-fence",
        plugins: "markdown",
        source: "@@fenceX{HEAD}\n```js\nx\n```\n",
        extra: "",
        expect: &[r#"class="cls""#, r#"id="elem-x""#, r#"type="fenceX""#],
    },
    Case {
        name: "paragraph",
        plugins: "markdown",
        source: "@@paraX{HEAD}\nA paragraph.\n",
        extra: "",
        expect: &[r#"class="cls""#, r#"id="elem-x""#, r#"type="paraX""#],
    },
    Case {
        name: "number-marker",
        plugins: "anchor,number,markdown",
        // `%%` → depth 1 (ADR-0010), so one counter segment.
        source: "@[N](/n)@@numberedA{`slug-y`, %%numberA}\n",
        // §10.5: the marker reaches `plugin-number`, which writes `number`.
        extra: r#"
[[task.number.custom]]
type = ["numberA"]
style = "decimal"
"#,
        expect: &[r#"number={["1"]}"#, r#"type="numberedA""#],
    },
];

/// A reference file in the shape the cite plugin reads: `id: { … }`, not a list.
const REFERENCES: &str = "alpha:\n  type: book\n  title: Alpha\n  authors:\n    - family: A\n      given: A\n  issuedDate:\n    year: 2020\n";

/// Writes every row as its own task in one project (§16.1) and returns it.
fn project() -> PathBuf {
    let dir = std::env::temp_dir().join("pendon-extras-heads");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("src")).expect("src dir");
    fs::write(dir.join("src/reference.yaml"), REFERENCES).expect("references");

    let mut config = String::new();
    for case in CASES {
        config.push_str(&format!(
            "[[task]]\nname = \"{name}\"\ninput = \"./src/{name}.mdp\"\n\
             output = \"./out/{name}.jsx\"\nplugin = \"{plugins}\"\nformat = \"solid\"\n{extra}\n",
            name = case.name,
            plugins = case.plugins,
            extra = case.extra,
        ));
        fs::write(
            dir.join("src").join(format!("{}.mdp", case.name)),
            case.source.replace("{HEAD}", HEAD),
        )
        .expect("fixture source");
    }
    fs::write(dir.join("pendon.toml"), config).expect("config");
    dir
}

fn read(dir: &Path, path: &str) -> String {
    fs::read_to_string(dir.join(path)).unwrap_or_else(|error| panic!("{path}: {error}"))
}

/// S2-4.1's exit criterion: every element head and decorator accepts the whole
/// extras grammar — all five specs positions and `%%numbering` — and none of the
/// documents falls back to literal text.
#[test]
fn extras_reach_every_element_head() {
    let dir = project();
    cargo_bin_cmd!("pendon")
        .current_dir(&dir)
        .arg("run")
        .assert()
        .success();

    for case in CASES {
        let out = read(&dir, &format!("out/{}.jsx", case.name));
        for expect in case.expect {
            assert!(
                out.contains(expect),
                "{}: expected `{expect}` in\n{out}",
                case.name
            );
        }
        // The head was *read*: no `@@type{…}` literal fallback (§4.3) and no
        // marker left as text — the same two checks for every row.
        assert!(
            !out.contains("@@"),
            "{}: a head stayed literal:\n{out}",
            case.name
        );
        assert!(
            !out.contains("%%%numberA"),
            "{}: the numbering marker stayed literal:\n{out}",
            case.name
        );
    }
}
