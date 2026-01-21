use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use serde_json::Value;
use std::fs::File;
use std::io::Write;
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
fn ast_format_heading_has_level_attr() {
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
                .and(predicate::str::contains("\"level\":\"2\"")),
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
        .arg("micomatter,markdown")
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
        .arg("micomatter,markdown")
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
