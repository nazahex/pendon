use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use serde_json::Value;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn plain_text_from_stdin() {
    let mut cmd = cargo_bin_cmd!("pendon");
    let input = "Hello";
    let output = cmd
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).expect("valid JSON");
    assert_eq!(v.get("type").and_then(|t| t.as_str()), Some("Document"));
    let children = v
        .get("children")
        .and_then(|c| c.as_array())
        .expect("children array");
    let para = children.first().expect("paragraph node");
    assert_eq!(para.get("type").and_then(|t| t.as_str()), Some("Paragraph"));
    let text = para
        .get("children")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|n| n.get("text"))
        .and_then(|t| t.as_str());
    assert_eq!(text, Some("Hello"));
}

#[test]
fn multiline_text_from_stdin() {
    let mut cmd = cargo_bin_cmd!("pendon");
    let input = "A\nB";
    let output = cmd
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).expect("valid JSON");
    let children = v
        .get("children")
        .and_then(|c| c.as_array())
        .expect("children array");
    let para = children.first().expect("paragraph node");
    let text = para
        .get("children")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|n| n.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("");
    assert_eq!(text, "A\nB");
}

#[test]
fn reads_from_file_path() {
    let dir = tempdir().expect("temp dir");
    let file_path = dir.path().join("input.md");
    let mut f = File::create(&file_path).expect("create file");
    write!(f, "Hello from file").expect("write file");

    let mut cmd = cargo_bin_cmd!("pendon");
    let output = cmd
        .arg("--input")
        .arg(&file_path)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).expect("valid JSON");
    let para_text = v
        .get("children")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|n| n.get("children"))
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|n| n.get("text"))
        .and_then(|t| t.as_str());
    assert_eq!(para_text, Some("Hello from file"));
}

#[test]
fn unknown_block_treated_as_text_in_default_mode() {
    let mut cmd = cargo_bin_cmd!("pendon");
    let input = ":::alert type=warning\nHello";
    let output = cmd
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).expect("valid JSON");
    let text = v
        .get("children")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|n| n.get("children"))
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|n| n.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("");
    assert_eq!(text, ":::alert type=warning\nHello");
}

#[test]
fn file_not_found_errors() {
    let mut cmd = cargo_bin_cmd!("pendon");
    cmd.arg("--input")
        .arg("/definitely/not/found.md")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error: cannot read file"));
}

#[test]
fn strict_mode_exits_non_zero_on_blank_run_exceed() {
    let mut cmd = cargo_bin_cmd!("pendon");
    let input = "A\n\n\nB";
    let output = cmd
        .arg("--strict")
        .arg("--max-blank-run")
        .arg("1")
        .write_stdin(input)
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).expect("valid JSON");
    let diagnostics = v
        .get("diagnostics")
        .and_then(|d| d.as_array())
        .expect("diagnostics array");
    assert!(diagnostics
        .iter()
        .any(|d| d.get("severity").and_then(|s| s.as_str()) == Some("Error")));

    // Ensure both A and B text chunks are present in the document
    let mut texts: Vec<String> = Vec::new();
    fn collect_texts(node: &Value, out: &mut Vec<String>) {
        if let Some(t) = node.get("text").and_then(|t| t.as_str()) {
            out.push(t.to_string());
        }
        if let Some(children) = node.get("children").and_then(|c| c.as_array()) {
            for ch in children {
                collect_texts(ch, out);
            }
        }
    }
    collect_texts(&v, &mut texts);
    let joined = texts.join("");
    assert!(joined.contains("A"));
    assert!(joined.contains("B"));
}

#[test]
fn ast_format_basic_from_stdin() {
    let mut cmd = cargo_bin_cmd!("pendon");
    let input = "Hello";

    cmd.arg("--format")
        .arg("ast")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("\"type\":\"Document\"")
                .and(predicate::str::contains("\"type\":\"Paragraph\""))
                .and(predicate::str::contains(
                    "\"type\":\"Text\",\"text\":\"Hello\"",
                ))
                .and(
                    predicate::str::is_match("\\\"type\\\":\\\"Paragraph\\\",\\\"text\\\":")
                        .expect("regex")
                        .not(),
                ),
        );
}

#[test]
fn ast_format_with_markdown_plugin_heading() {
    let mut cmd = cargo_bin_cmd!("pendon");
    let input = "# Title\n";

    cmd.arg("--plugin")
        .arg("markdown")
        .arg("--format")
        .arg("ast")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("\"type\":\"Heading\"")
                .and(predicate::str::contains("\"text\":\"Title\"")),
        );
}

#[test]
fn ast_format_heading_has_depth_attr() {
    let mut cmd = cargo_bin_cmd!("pendon");
    let input = "## Subtitle\n";

    cmd.arg("--plugin")
        .arg("markdown")
        .arg("--format")
        .arg("ast")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("\"type\":\"Heading\"")
                .and(predicate::str::contains("\"depth\":\"2\"")),
        );
}

#[test]
fn ast_format_code_fence_has_lang_attr() {
    let mut cmd = cargo_bin_cmd!("pendon");
    let input = "```ts\nconsole.log(1)\n```\n";

    cmd.arg("--plugin")
        .arg("markdown")
        .arg("--format")
        .arg("ast")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("\"type\":\"CodeFence\"")
                .and(predicate::str::contains("\"lang\":\"ts\"")),
        );
}

#[test]
fn ast_format_bullet_list() {
    let mut cmd = cargo_bin_cmd!("pendon");
    let input = "- one\n- two\n";

    cmd.arg("--plugin")
        .arg("markdown")
        .arg("--format")
        .arg("ast")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("\"type\":\"BulletList\"")
                .and(predicate::str::contains("\"type\":\"ListItem\""))
                .and(predicate::str::contains("one"))
                .and(predicate::str::contains("two")),
        );
}

#[test]
fn ast_format_ordered_list_with_start_attr() {
    let mut cmd = cargo_bin_cmd!("pendon");
    let input = "3. three\n4) four\n";

    cmd.arg("--plugin")
        .arg("markdown")
        .arg("--format")
        .arg("ast")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("\"type\":\"OrderedList\"")
                .and(predicate::str::contains("\"start\":\"3\""))
                .and(predicate::str::contains("three"))
                .and(predicate::str::contains("four")),
        );
}

#[test]
fn ast_format_nested_lists() {
    let mut cmd = cargo_bin_cmd!("pendon");
    let input = "- a\n  - b\n    - c\n- d\n";

    cmd.arg("--plugin")
        .arg("markdown")
        .arg("--format")
        .arg("ast")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("\"type\":\"BulletList\"")
                .and(predicate::str::contains("\"type\":\"ListItem\""))
                .and(predicate::str::contains("a"))
                .and(predicate::str::contains("b"))
                .and(predicate::str::contains("c"))
                .and(predicate::str::contains("d")),
        );
}

#[test]
fn ast_format_list_item_multiline_continuation() {
    let mut cmd = cargo_bin_cmd!("pendon");
    let input = "- first line\n  continuation line\n- second item\n";

    cmd.arg("--plugin")
        .arg("markdown")
        .arg("--format")
        .arg("ast")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("\"type\":\"BulletList\"")
                .and(predicate::str::contains("first line"))
                .and(predicate::str::contains("continuation line"))
                .and(predicate::str::contains("second item")),
        );
}

#[test]
fn ast_paragraph_children_only_with_inline() {
    let mut cmd = cargo_bin_cmd!("pendon");
    let input = "This is _italic_ and **bold**.";

    cmd.arg("--plugin")
        .arg("markdown")
        .arg("--format")
        .arg("ast")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("\"type\":\"Paragraph\"")
                .and(predicate::str::contains("\"type\":\"Italic\""))
                .and(predicate::str::contains("\"type\":\"Strong\""))
                .and(
                    predicate::str::is_match("\\\"type\\\":\\\"Paragraph\\\",\\\"text\\\":")
                        .expect("regex")
                        .not(),
                ),
        );
}

#[test]
fn ast_heading_children_only_when_inline_present() {
    let mut cmd = cargo_bin_cmd!("pendon");
    let input = "# Hello *world*\n";

    cmd.arg("--plugin")
        .arg("markdown")
        .arg("--format")
        .arg("ast")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("\"type\":\"Heading\"")
                .and(predicate::str::contains("\"type\":\"Emphasis\""))
                .and(
                    predicate::str::is_match("\\\"type\\\":\\\"Heading\\\",\\\"text\\\":")
                        .expect("regex")
                        .not(),
                ),
        );
}

#[test]
fn ast_heading_text_only_when_plain() {
    let mut cmd = cargo_bin_cmd!("pendon");
    let input = "# Plain\n";

    cmd.arg("--plugin")
        .arg("markdown")
        .arg("--format")
        .arg("ast")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("\"type\":\"Heading\"")
                .and(predicate::str::contains("\"text\":\"Plain\"")),
        );
}

#[test]
fn ast_list_item_children_only_when_inline_present() {
    let mut cmd = cargo_bin_cmd!("pendon");
    let input = "- Hello *world*\n";

    cmd.arg("--plugin")
        .arg("markdown")
        .arg("--format")
        .arg("ast")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("\"type\":\"BulletList\"")
                .and(predicate::str::contains("\"type\":\"ListItem\""))
                .and(predicate::str::contains("\"type\":\"Emphasis\""))
                .and(
                    predicate::str::is_match("\\\"type\\\":\\\"ListItem\\\",\\\"text\\\":")
                        .expect("regex")
                        .not(),
                ),
        );
}

#[test]
fn ast_list_item_text_only_when_plain() {
    let mut cmd = cargo_bin_cmd!("pendon");
    let input = "- Plain\n";

    cmd.arg("--plugin")
        .arg("markdown")
        .arg("--format")
        .arg("ast")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("\"type\":\"BulletList\"")
                .and(predicate::str::contains("\"type\":\"ListItem\""))
                .and(predicate::str::contains("\"text\":\"Plain\"")),
        );
}

#[test]
fn ast_blockquote_and_table_nodes() {
    let mut cmd = cargo_bin_cmd!("pendon");
    let input = "> quoted\n> block\n\n| A | B |\n| C | D |\n";

    cmd.arg("--plugin")
        .arg("markdown")
        .arg("--format")
        .arg("ast")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("\"type\":\"Blockquote\"")
                .and(predicate::str::contains("quoted"))
                .and(predicate::str::contains("\"type\":\"Table\""))
                .and(predicate::str::contains("\"type\":\"TableHead\""))
                .and(predicate::str::contains("\"type\":\"TableBody\""))
                .and(predicate::str::contains("\"header\":\"1\""))
                .and(predicate::str::contains("A"))
                .and(predicate::str::contains("C")),
        );
}

#[test]
fn ast_code_fence_text_only() {
    let mut cmd = cargo_bin_cmd!("pendon");
    let input = "```ts\nconsole.log(1)\n```\n";

    let output = cmd
        .arg("--plugin")
        .arg("markdown")
        .arg("--format")
        .arg("ast")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // Parse JSON and assert structurally
    let v: serde_json::Value = serde_json::from_slice(&output).expect("valid JSON");
    let children = v
        .get("children")
        .and_then(|c| c.as_array())
        .expect("document children");
    let cf = children
        .iter()
        .find(|n| n.get("type").and_then(|t| t.as_str()) == Some("CodeFence"))
        .expect("CodeFence node");
    assert_eq!(
        cf.get("text").and_then(|t| t.as_str()),
        Some("console.log(1)\n")
    );
    assert!(
        cf.get("children").is_none(),
        "CodeFence must not have children"
    );
}

#[test]
fn ast_frontmatter_payload_is_exposed() {
    let mut cmd = cargo_bin_cmd!("pendon");
    let input = "---\ntitle: \"Demo\"\ndraft: false\nviews: 7\n---\n\n# Heading\n";

    let output = cmd
        .arg("--plugin")
        .arg("micromatter,markdown")
        .arg("--format")
        .arg("ast")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).expect("valid JSON");
    let children = v
        .get("children")
        .and_then(|c| c.as_array())
        .expect("document children");
    let fm = children
        .iter()
        .find(|n| n.get("type").and_then(|t| t.as_str()) == Some("Frontmatter"))
        .expect("frontmatter node present");
    let attrs = fm.get("attrs").and_then(|a| a.get("data"));
    let data_str = attrs
        .and_then(|d| d.as_str())
        .expect("data attribute present");
    let data: Value = serde_json::from_str(data_str).expect("frontmatter JSON");
    assert_eq!(data.get("title").and_then(|t| t.as_str()), Some("Demo"));
    assert_eq!(data.get("draft").and_then(|t| t.as_bool()), Some(false));
    assert_eq!(data.get("views").and_then(|t| t.as_i64()), Some(7));
}

#[test]
fn json_renderer_carries_frontmatter_blockquote_and_table() {
    fn find_first<'a>(node: &'a Value, target: &str) -> Option<&'a Value> {
        if node.get("type").and_then(|t| t.as_str()) == Some(target) {
            return Some(node);
        }
        if let Some(children) = node.get("children").and_then(|c| c.as_array()) {
            for ch in children {
                if let Some(found) = find_first(ch, target) {
                    return Some(found);
                }
            }
        }
        None
    }

    let mut cmd = cargo_bin_cmd!("pendon");
    let input = "---\ntitle: \"Demo\"\n---\n\n> quoted\n\n| A | B |\n| --- | --- |\n| 1 | 2 |\n";

    let output = cmd
        .arg("--plugin")
        .arg("micromatter,markdown")
        .arg("--format")
        .arg("json")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).expect("valid JSON");
    assert_eq!(v.get("type").and_then(|t| t.as_str()), Some("Document"));

    let fm = find_first(&v, "Frontmatter").expect("frontmatter node");
    let data_str = fm
        .get("attrs")
        .and_then(|a| a.get("data"))
        .and_then(|d| d.as_str())
        .expect("frontmatter data attr");
    let data: Value = serde_json::from_str(data_str).expect("frontmatter json");
    assert_eq!(data.get("title").and_then(|t| t.as_str()), Some("Demo"));

    let bq = find_first(&v, "Blockquote").expect("blockquote node");
    let bq_text = bq
        .get("children")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|n| n.get("children"))
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|n| n.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("");
    assert!(bq_text.contains("quoted"));

    let table = find_first(&v, "Table").expect("table node");
    let thead = find_first(table, "TableHead").expect("table head");
    let header_cell = find_first(thead, "TableCell").expect("header cell");
    assert_eq!(
        header_cell
            .get("attrs")
            .and_then(|a| a.get("header"))
            .and_then(|h| h.as_str()),
        Some("1")
    );
    let tbody = find_first(table, "TableBody").expect("table body");
    let body_cell = find_first(tbody, "TableCell").expect("body cell");
    let body_text = body_cell
        .get("children")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|n| n.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("");
    assert_eq!(body_text, "1");
}

/// The custom table node covers every layer of the table — caption, the three
/// sections, rows and cells — and its `imports` list is shared by all of them.
#[test]
fn run_config_table_custom_layers_and_shared_imports() {
    let dir = tempdir().expect("temp dir");
    let src_dir = dir.path().join("src");
    let out_dir = dir.path().join("out");
    std::fs::create_dir_all(&src_dir).expect("create src dir");
    std::fs::create_dir_all(&out_dir).expect("create out dir");

    std::fs::write(
        src_dir.join("sales.md"),
        "|| Laporan Penjualan ||\n| Produk | Stok |\n| --- | --- |\n| Laptop Pro | 15 |\n|===|\n| Total | 15 |\n",
    )
    .expect("write markdown file");

    std::fs::write(
        dir.path().join("pendon.toml"),
        r#"[[task]]
input = "./src/[...slug].md"
output = "./out/[...slug].jsx"
plugin = "table,markdown"
format = "solid"


[task.table.custom.table]
name = "CustomTable"
template = "<CustomTable>{children}</CustomTable>"
imports = ["import { TableCaption } from '@comp/table';", { module = "@comp/table", default = "CustomTable" }]

[task.table.custom.caption]
name = "TableCaption"
template = "<TableCaption>{children}</TableCaption>"

[task.table.custom.thead]
name = "TableHead"
template = "<TableHead>{children}</TableHead>"

[[task.table.custom.thead.imports]]
module = "@comp/table"
names = ["TableHead"]

[task.table.custom.tbody]
name = "TableBody"
template = "<TableBody>{children}</TableBody>"

[task.table.custom.tfoot]
name = "TableFoot"
template = "<TableFoot>{children}</TableFoot>"

[task.table.custom.row]
name = "TableRow"
template = "<TableRow>{children}</TableRow>"

[task.table.custom.cell]
name = "TableCell"
template = "<TableCell>{children}</TableCell>"
"#,
    )
    .expect("write config file");

    let mut cmd = cargo_bin_cmd!("pendon");
    cmd.current_dir(dir.path()).arg("run").assert().success();

    let output = std::fs::read_to_string(out_dir.join("sales.jsx")).expect("read output jsx");

    for name in [
        "CustomTable",
        "TableCaption",
        "TableHead",
        "TableBody",
        "TableFoot",
        "TableRow",
        "TableCell",
    ] {
        assert!(
            output.contains(&format!("<{name}")),
            "missing <{name}>: {output}"
        );
    }
    for tag in [
        "<table", "<caption", "<thead", "<tbody", "<tfoot", "<tr", "<td",
    ] {
        assert!(
            !output.contains(tag),
            "plain {tag}> must not remain: {output}"
        );
    }

    // Shared raw line plus each component's own entry, merged per module.
    assert!(
        output.contains("import { TableCaption } from '@comp/table';"),
        "{output}"
    );
    assert!(
        output.contains("import CustomTable, {TableHead} from \"@comp/table\";"),
        "{output}"
    );

    // Cells are pre-rendered, so their words must stay separated.
    assert!(output.contains("Laptop Pro"), "{output}");
    assert!(output.contains("Laporan Penjualan"), "{output}");
}

/// Renders a cite project whose reference map holds a cited `book` and an
/// uncited `paper`, and returns the produced `.jsx`.
///
/// `frontmatter` picks the reference source: `true` declares both entries in the
/// document's frontmatter, `false` loads them from an external YAML catalog. The
/// extra `scope` line goes into `[task.cite]` verbatim (`""` keeps the default).
fn render_cite_scope(frontmatter: bool, scope: &str) -> String {
    let dir = tempdir().expect("temp dir");
    let src_dir = dir.path().join("src");
    let out_dir = dir.path().join("out");
    std::fs::create_dir_all(&src_dir).expect("create src dir");
    std::fs::create_dir_all(&out_dir).expect("create out dir");

    let body = "[^book] and [^book \"p. 2\"].\n";
    let source = if frontmatter {
        std::fs::write(
            src_dir.join("chapter.md"),
            format!(
                "---\nreferences:\n  book:\n    title: Book\n  paper:\n    title: Paper\n---\n\n{body}"
            ),
        )
        .expect("write markdown file");
        "reference_source = \"internal\"\n".to_string()
    } else {
        std::fs::write(src_dir.join("chapter.md"), body).expect("write markdown file");
        let refs_dir = dir.path().join("references");
        std::fs::create_dir_all(&refs_dir).expect("create references dir");
        std::fs::write(
            refs_dir.join("chapter.yaml"),
            "book:\n  title: Book\npaper:\n  title: Paper\n",
        )
        .expect("write references file");
        "reference_source = \"external\"\nreference_file = \"./references/[slug].yaml\"\n"
            .to_string()
    };

    std::fs::write(
        dir.path().join("pendon.toml"),
        format!(
            "[[task]]\ninput = \"./src/[slug].md\"\noutput = \"./out/[slug].jsx\"\n\
             plugin = \"micromatter,cite,markdown\"\nformat = \"solid\"\n\n\
             [task.cite]\n{source}{scope}"
        ),
    )
    .expect("write config file");

    let mut cmd = cargo_bin_cmd!("pendon");
    cmd.current_dir(dir.path()).arg("run").assert().success();

    std::fs::read_to_string(out_dir.join("chapter.jsx")).expect("read output jsx")
}

/// The `export const citations = …` line, to prove the scope never touches it.
fn citations_line(output: &str) -> &str {
    output
        .lines()
        .find(|line| line.starts_with("export const citations"))
        .unwrap_or_else(|| panic!("no citations export: {output}"))
}

/// The `export const references = …` line, the only export the scope filters.
fn references_line(output: &str) -> &str {
    output
        .lines()
        .find(|line| line.starts_with("export const references"))
        .unwrap_or_else(|| panic!("no references export: {output}"))
}

/// §2.6: an external catalog may be huge, so it defaults to cited-only — the
/// uncited `paper` must not reach the `references` export.
#[test]
fn run_config_cite_exports_only_cited_external_references_by_default() {
    let output = render_cite_scope(false, "");

    assert!(references_line(&output).contains("\"book\""), "{output}");
    assert!(
        !references_line(&output).contains("paper"),
        "uncited reference leaked: {output}"
    );
}

/// §2.6: `references_scope = "all"` forces the whole external catalog into the
/// export; the `citations` export is byte-identical to the default render.
#[test]
fn run_config_cite_scope_all_keeps_uncited_external_references() {
    let default = render_cite_scope(false, "");
    let all = render_cite_scope(false, "references_scope = \"all\"\n");

    assert!(references_line(&all).contains("\"paper\""), "{all}");
    assert_eq!(citations_line(&default), citations_line(&all));
}

/// §2.6: frontmatter references are authored for this document, so they default
/// to `all`; an explicit `cited` narrows them again.
#[test]
fn run_config_cite_frontmatter_defaults_to_all_and_honours_cited() {
    let default = render_cite_scope(true, "");
    assert!(references_line(&default).contains("\"paper\""), "{default}");

    let cited = render_cite_scope(true, "references_scope = \"cited\"\n");
    assert!(
        !references_line(&cited).contains("paper"),
        "uncited reference leaked: {cited}"
    );
    assert_eq!(citations_line(&default), citations_line(&cited));
}

/// §2.6: any spelling other than `cited` / `all` is a hard error that names the
/// valid options.
#[test]
fn run_config_cite_rejects_an_unknown_references_scope() {
    let dir = tempdir().expect("temp dir");
    let src_dir = dir.path().join("src");
    std::fs::create_dir_all(&src_dir).expect("create src dir");
    std::fs::write(src_dir.join("chapter.md"), "Body text.\n").expect("write markdown file");
    std::fs::write(
        dir.path().join("pendon.toml"),
        "[[task]]\ninput = \"./src/[slug].md\"\noutput = \"./out/[slug].jsx\"\n\
         plugin = \"cite,markdown\"\nformat = \"solid\"\n\n\
         [task.cite]\nreferences_scope = \"everything\"\n",
    )
    .expect("write config file");

    let mut cmd = cargo_bin_cmd!("pendon");
    cmd.current_dir(dir.path())
        .arg("run")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("unsupported cite.references_scope 'everything'")
                .and(predicate::str::contains("cited or all")),
        );
}

/// §13: an unknown key inside a `[[task]]` table is a **load error** — the build
/// aborts before any file is processed, so a typo can never change the output
/// while the run reports success.
#[test]
fn run_config_rejects_an_unknown_task_key() {
    let dir = tempdir().expect("temp dir");
    let src_dir = dir.path().join("src");
    std::fs::create_dir_all(&src_dir).expect("create src dir");
    std::fs::write(src_dir.join("page.md"), "Body text.\n").expect("write markdown file");
    std::fs::write(
        dir.path().join("pendon.toml"),
        "[[task]]\nname = \"Typo\"\ninput = \"./src/[slug].md\"\noutput = \"./out/[slug].jsx\"\n\
         plugin = \"markdown\"\nformat = \"solid\"\ncolour = \"red\"\n",
    )
    .expect("write config file");

    let mut cmd = cargo_bin_cmd!("pendon");
    cmd.current_dir(dir.path())
        .arg("run")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("unknown key 'colour'")
                .and(predicate::str::contains("pendon.toml")),
        );
    assert!(
        !dir.path().join("out").join("page.jsx").exists(),
        "the build must abort before rendering"
    );
}

// === `no_cache` / `post_hook` (docs/rfc/no-cache-and-post-hook.md) ===

/// Writes an HTML task named `Hooked` over `src/[slug].md` and appends `extra`
/// to the `[[task]]` table. Returns the temp project.
fn hook_project(extra: &str) -> tempfile::TempDir {
    let dir = tempdir().expect("temp dir");
    let src_dir = dir.path().join("src");
    std::fs::create_dir_all(&src_dir).expect("create src dir");
    std::fs::write(src_dir.join("page.md"), "Body text.\n").expect("write markdown file");
    std::fs::write(
        dir.path().join("pendon.toml"),
        format!(
            "[[task]]\nname = \"Hooked\"\ninput = \"./src/[slug].md\"\n\
             output = \"./out/[slug].html\"\nplugin = \"markdown\"\nformat = \"html\"\n{extra}"
        ),
    )
    .expect("write config file");
    dir
}

/// Runs `pendon run` inside the project root.
fn run_pendon(dir: &Path) -> assert_cmd::assert::Assert {
    let mut cmd = cargo_bin_cmd!("pendon");
    cmd.current_dir(dir).arg("run").assert()
}

/// Every cache record under `.pendon/cache`, as raw TOML (empty when nothing is
/// cached).
fn cache_records(dir: &Path) -> Vec<String> {
    let mut records = Vec::new();
    let mut stack = vec![dir.join(".pendon").join("cache")];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|ext| ext == "toml") {
                if let Ok(text) = std::fs::read_to_string(&path) {
                    records.push(text);
                }
            }
        }
    }
    records.sort();
    records
}

/// `no_cache = true` must render every run and leave the cache alone.
#[test]
fn run_task_no_cache_always_rebuilds_and_never_touches_the_cache() {
    let dir = hook_project("no_cache = true\n");
    run_pendon(dir.path()).success();

    let out = dir.path().join("out").join("page.html");
    assert!(out.exists(), "first run did not render the output");
    assert!(
        !dir.path().join(".pendon").exists(),
        "no_cache must not create a cache"
    );

    // A cached task would skip a file its manifest calls up to date, leaving the
    // deleted output deleted; a no_cache task has to render it again.
    std::fs::remove_file(&out).expect("remove output");
    run_pendon(dir.path()).success();
    assert!(out.exists(), "no_cache task skipped a missing output");
    assert!(
        !dir.path().join(".pendon").exists(),
        "no_cache wrote a cache entry"
    );
}

/// Caching is disabled only for the task that opts out.
#[test]
fn run_task_no_cache_leaves_other_tasks_cached() {
    let dir = tempdir().expect("temp dir");
    let src_dir = dir.path().join("src");
    std::fs::create_dir_all(&src_dir).expect("create src dir");
    std::fs::write(src_dir.join("cached.md"), "Cached body.\n").expect("write cached source");
    std::fs::write(src_dir.join("uncached.md"), "Uncached body.\n").expect("write uncached source");
    std::fs::write(
        dir.path().join("pendon.toml"),
        "[[task]]\nname = \"Cached\"\ninput = \"./src/cached.md\"\n\
         output = \"./out/cached.html\"\nplugin = \"markdown\"\nformat = \"html\"\n\n\
         [[task]]\nname = \"Uncached\"\ninput = \"./src/uncached.md\"\n\
         output = \"./out/uncached.html\"\nplugin = \"markdown\"\nformat = \"html\"\n\
         no_cache = true\n",
    )
    .expect("write config file");

    run_pendon(dir.path()).success();

    let records = cache_records(dir.path());
    assert_eq!(
        records.len(),
        1,
        "the cached task must record exactly one file: {records:#?}"
    );
    assert!(
        records[0].contains("task = \"Cached\""),
        "wrong record: {}",
        records[0]
    );
    assert!(
        !records[0].contains("Uncached"),
        "no_cache task leaked into the cache: {}",
        records[0]
    );
}

/// One input, two tasks: each keeps its own record, so both stay cached. The
/// single-manifest layout let one task's entry overwrite the other's.
#[test]
fn run_task_two_tasks_over_one_input_both_stay_cached() {
    let dir = tempdir().expect("temp dir");
    let src_dir = dir.path().join("src");
    std::fs::create_dir_all(&src_dir).expect("create src dir");
    std::fs::write(src_dir.join("page.md"), "Body text.\n").expect("write source");
    std::fs::write(
        dir.path().join("pendon.toml"),
        "[[task]]\nname = \"Html\"\ninput = \"./src/[slug].md\"\n\
         output = \"./out/[slug].html\"\nplugin = \"markdown\"\nformat = \"html\"\n\
         post_hook = \"echo html >> hooks.log\"\n\n\
         [[task]]\nname = \"Json\"\ninput = \"./src/[slug].md\"\n\
         output = \"./out/[slug].json\"\nplugin = \"markdown\"\nformat = \"json\"\n\
         post_hook = \"echo json >> hooks.log\"\n",
    )
    .expect("write config file");

    run_pendon(dir.path()).success();
    run_pendon(dir.path()).success();

    assert_eq!(
        cache_records(dir.path()).len(),
        2,
        "one record per (task, input)"
    );
    let log = std::fs::read_to_string(dir.path().join("hooks.log")).expect("hooks.log");
    assert_eq!(
        log.lines().collect::<Vec<_>>(),
        vec!["html", "json"],
        "a shared input re-ran a hook instead of hitting the cache: {log}"
    );
}

/// A record only proves "up to date" while the output is still there.
#[test]
fn run_task_a_deleted_output_is_rebuilt() {
    let dir = hook_project("");
    run_pendon(dir.path()).success();

    let out = dir.path().join("out").join("page.html");
    std::fs::remove_file(&out).expect("remove output");
    run_pendon(dir.path()).success();

    assert!(out.exists(), "a deleted output stayed deleted");
}

/// `--no-cache` reads nothing *and* records nothing, so the next run is cold.
#[test]
fn run_task_the_no_cache_flag_records_nothing() {
    let dir = hook_project("");

    let mut cmd = cargo_bin_cmd!("pendon");
    cmd.current_dir(dir.path())
        .arg("run")
        .arg("--no-cache")
        .assert()
        .success();

    assert!(
        !dir.path().join(".pendon").exists(),
        "--no-cache created a cache"
    );
}

/// A single `post_hook` string runs after the render, in the project root, with
/// `[slug]` expanded, and its output is forwarded with the task prefix.
#[test]
fn run_task_post_hook_runs_after_render_with_expanded_placeholders() {
    let dir =
        hook_project("post_hook = \"echo formatting [slug] && echo [slug] > hook-[slug].txt\"\n");

    run_pendon(dir.path())
        .success()
        .stderr(predicate::str::contains(
            "[post_hook:Hooked] formatting page",
        ));

    assert!(dir.path().join("out").join("page.html").exists());
    let marker = dir.path().join("hook-page.txt");
    let contents = std::fs::read_to_string(&marker)
        .expect("hook wrote its marker relative to the project root");
    assert_eq!(contents.trim(), "page");
}

/// An array runs in declared order and aborts at the first non-zero exit.
#[test]
fn run_task_post_hook_array_aborts_on_the_first_failure() {
    let dir = hook_project("post_hook = [\"echo first\", \"exit 3\", \"echo third\"]\n");

    run_pendon(dir.path()).failure().stderr(
        predicate::str::contains("[post_hook:Hooked] first")
            .and(predicate::str::contains("exited with status 3"))
            .and(predicate::str::contains("third").not()),
    );

    // A failed hook marks the task failed, so nothing records it as cached.
    assert!(
        !dir.path().join(".pendon").exists(),
        "a failed task must not record a cache"
    );
}

/// An unknown placeholder in a hook is an error, not literal text.
#[test]
fn run_task_post_hook_rejects_an_unknown_placeholder() {
    let dir = hook_project("post_hook = \"echo [missing]\"\n");

    run_pendon(dir.path())
        .failure()
        .stderr(predicate::str::contains("bad placeholder"));
}

/// A task that fails to render never reaches its hooks.
#[test]
fn run_task_post_hook_is_skipped_when_the_task_fails() {
    let dir = hook_project("");
    // Break the task after the helper wrote it: an unsupported renderer fails
    // every file, so no hook may run.
    std::fs::write(
        dir.path().join("pendon.toml"),
        "[[task]]\nname = \"Hooked\"\ninput = \"./src/[slug].md\"\n\
         output = \"./out/[slug].html\"\nplugin = \"markdown\"\nformat = \"bogus\"\n\
         post_hook = \"echo should-not-run\"\n",
    )
    .expect("write config file");

    run_pendon(dir.path())
        .failure()
        .stderr(predicate::str::contains("should-not-run").not());
}

/// Files served from the cache did not run the pipeline, so their hooks do not
/// re-run; editing the hook invalidates the task once so it reaches them.
#[test]
fn run_task_post_hook_does_not_rerun_for_cached_files() {
    let dir = hook_project("post_hook = \"echo hooked >> hooks.log\"\n");
    run_pendon(dir.path()).success();
    run_pendon(dir.path()).success();

    let log_path = dir.path().join("hooks.log");
    let log = std::fs::read_to_string(&log_path).expect("hooks.log");
    assert_eq!(
        log.lines().count(),
        1,
        "a cached file re-ran its post_hook: {log}"
    );

    std::fs::write(
        dir.path().join("pendon.toml"),
        "[[task]]\nname = \"Hooked\"\ninput = \"./src/[slug].md\"\n\
         output = \"./out/[slug].html\"\nplugin = \"markdown\"\nformat = \"html\"\n\
         post_hook = \"echo changed >> hooks.log\"\n",
    )
    .expect("write changed config file");

    run_pendon(dir.path()).success();
    let log = std::fs::read_to_string(&log_path).expect("hooks.log");
    assert_eq!(
        log.lines().collect::<Vec<_>>(),
        vec!["hooked", "changed"],
        "a changed post_hook must reach already-cached files: {log}"
    );
}
