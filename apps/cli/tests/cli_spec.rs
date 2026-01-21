use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn plain_text_from_stdin() {
    let mut cmd = cargo_bin_cmd!("pendon");
    let input = "Hello";
    let expected = "{\"type\":\"Document\",\"children\":\"Hello\"}\n";

    cmd.write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::eq(expected));
}

#[test]
fn multiline_text_from_stdin() {
    let mut cmd = cargo_bin_cmd!("pendon");
    let input = "A\nB";
    let expected = "{\"type\":\"Document\",\"children\":\"A\\nB\"}\n";

    cmd.write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::eq(expected));
}

#[test]
fn reads_from_file_path() {
    let dir = tempdir().expect("temp dir");
    let file_path = dir.path().join("input.md");
    let mut f = File::create(&file_path).expect("create file");
    write!(f, "Hello from file").expect("write file");

    let mut cmd = cargo_bin_cmd!("pendon");
    let expected = "{\"type\":\"Document\",\"children\":\"Hello from file\"}\n";

    cmd.arg("--input")
        .arg(&file_path)
        .assert()
        .success()
        .stdout(predicate::eq(expected));
}

#[test]
fn unknown_block_treated_as_text_in_default_mode() {
    let mut cmd = cargo_bin_cmd!("pendon");
    let input = ":::alert type=warning\nHello";
    let expected = "{\"type\":\"Document\",\"children\":\":::alert type=warning\\nHello\"}\n";

    cmd.write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::eq(expected));
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
    let expected = "{\"type\":\"Document\",\"children\":\"A\\n\\n\\nB\"}\n";

    cmd.arg("--strict")
        .arg("--max-blank-run")
        .arg("1")
        .write_stdin(input)
        .assert()
        .failure()
        .stdout(predicate::eq(expected));
}

#[test]
fn ast_format_basic_from_stdin() {
    let mut cmd = cargo_bin_cmd!("pendon");
    let input = "Hello";

    cmd.arg("--format").arg("ast")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("\"type\":\"Document\"")
                .and(predicate::str::contains("\"type\":\"Paragraph\""))
                .and(predicate::str::contains("\"type\":\"Text\",\"text\":\"Hello\""))
                .and(predicate::str::is_match("\\\"type\\\":\\\"Paragraph\\\",\\\"text\\\":").expect("regex").not()),
        );
}

#[test]
fn ast_format_with_markdown_plugin_heading() {
    let mut cmd = cargo_bin_cmd!("pendon");
    let input = "# Title\n";

    cmd.arg("--plugin").arg("markdown")
        .arg("--format").arg("ast")
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

    cmd.arg("--plugin").arg("markdown")
        .arg("--format").arg("ast")
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

    cmd.arg("--plugin").arg("markdown")
        .arg("--format").arg("ast")
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

    cmd.arg("--plugin").arg("markdown")
        .arg("--format").arg("ast")
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

    cmd.arg("--plugin").arg("markdown")
        .arg("--format").arg("ast")
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

    cmd.arg("--plugin").arg("markdown")
        .arg("--format").arg("ast")
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

    cmd.arg("--plugin").arg("markdown")
        .arg("--format").arg("ast")
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

    cmd.arg("--plugin").arg("markdown")
        .arg("--format").arg("ast")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("\"type\":\"Paragraph\"")
                .and(predicate::str::contains("\"type\":\"Italic\""))
                .and(predicate::str::contains("\"type\":\"Strong\""))
                .and(predicate::str::is_match("\\\"type\\\":\\\"Paragraph\\\",\\\"text\\\":").expect("regex").not()),
        );
}

#[test]
fn ast_heading_children_only_when_inline_present() {
    let mut cmd = cargo_bin_cmd!("pendon");
    let input = "# Hello *world*\n";

    cmd.arg("--plugin").arg("markdown")
        .arg("--format").arg("ast")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("\"type\":\"Heading\"")
                .and(predicate::str::contains("\"type\":\"Emphasis\""))
                .and(predicate::str::is_match("\\\"type\\\":\\\"Heading\\\",\\\"text\\\":").expect("regex").not()),
        );
}

#[test]
fn ast_heading_text_only_when_plain() {
    let mut cmd = cargo_bin_cmd!("pendon");
    let input = "# Plain\n";

    cmd.arg("--plugin").arg("markdown")
        .arg("--format").arg("ast")
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

    cmd.arg("--plugin").arg("markdown")
        .arg("--format").arg("ast")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("\"type\":\"BulletList\"")
                .and(predicate::str::contains("\"type\":\"ListItem\""))
                .and(predicate::str::contains("\"type\":\"Emphasis\""))
                .and(predicate::str::is_match("\\\"type\\\":\\\"ListItem\\\",\\\"text\\\":").expect("regex").not()),
        );
}

#[test]
fn ast_list_item_text_only_when_plain() {
    let mut cmd = cargo_bin_cmd!("pendon");
    let input = "- Plain\n";

    cmd.arg("--plugin").arg("markdown")
        .arg("--format").arg("ast")
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
fn ast_code_fence_text_only() {
    let mut cmd = cargo_bin_cmd!("pendon");
    let input = "```ts\nconsole.log(1)\n```\n";

    let output = cmd.arg("--plugin").arg("markdown")
        .arg("--format").arg("ast")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout.clone();

    // Parse JSON and assert structurally
    let v: serde_json::Value = serde_json::from_slice(&output).expect("valid JSON");
    let children = v.get("children").and_then(|c| c.as_array()).expect("document children");
    let cf = children.iter().find(|n| n.get("type").and_then(|t| t.as_str()) == Some("CodeFence")).expect("CodeFence node");
    assert_eq!(cf.get("text").and_then(|t| t.as_str()), Some("console.log(1)\n"));
    assert!(cf.get("children").is_none(), "CodeFence must not have children");
}
