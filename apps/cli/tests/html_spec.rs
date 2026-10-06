use assert_cmd::cargo::cargo_bin_cmd;
use predicates::str::contains;
use predicates::Predicate;
use std::fs;
use tempfile::tempdir;

fn run_cli(input: &str, args: &[&str]) -> (i32, String, String) {
    let mut cmd = cargo_bin_cmd!("pendon");
    cmd.args(args).write_stdin(input).assert().success();
    let output = cmd.output().unwrap();
    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (code, stdout, stderr)
}

#[test]
fn html_heading_and_inline() {
    let input = "# Title\n\nText with **strong** and __bold__, *em* and _italic_, `code`, and [link](https://example.com).\n";
    let (_code, out, _err) = run_cli(input, &["--plugin", "markdown", "--format", "html"]);
    assert!(contains("<h1>Title</h1>").eval(&out));
    assert!(contains("<strong>strong</strong>").eval(&out));
    assert!(contains("<b>bold</b>").eval(&out));
    assert!(contains("<em>em</em>").eval(&out));
    assert!(contains("<i>italic</i>").eval(&out));
    assert!(contains("<code>code</code>").eval(&out));
    assert!(contains("<a href=\"https://example.com\">link</a>").eval(&out));
}

#[test]
fn html_nested_lists() {
    let input = "- a\n  - b\n    - c\n- d\n";
    let (_code, out, _err) = run_cli(input, &["--plugin", "markdown", "--format", "html"]);
    assert!(contains("<ul>").eval(&out));
    assert!(contains("<li>a").eval(&out));
    assert!(contains("<li>b").eval(&out));
    assert!(contains("<li>c").eval(&out));
}

#[test]
fn html_blockquote_and_table() {
    let input = "> quoted\n> block\n\n| A | B |\n| C | D |\n";
    let (_code, out, _err) = run_cli(input, &["--plugin", "markdown", "--format", "html"]);
    assert!(contains("<blockquote>").eval(&out));
    assert!(contains("quoted").eval(&out));
    assert!(contains("<table>").eval(&out));
    assert!(contains("<thead>").eval(&out));
    assert!(contains("<tbody>").eval(&out));
    assert!(contains("<th>A</th>").eval(&out));
    assert!(contains("<td>C</td>").eval(&out));
}

#[test]
fn html_ignores_frontmatter_block() {
    let input = "---\ntitle: \"Demo\"\ndraft: false\n---\n\n# Hello\n";
    let (_code, out, _err) = run_cli(
        input,
        &["--plugin", "micromatter,markdown", "--format", "html"],
    );
    assert!(!contains("---").eval(&out));
    assert!(contains("<h1>Hello</h1>").eval(&out));
}

/// The CLI must expose the second HTML mode: `--pretty` used to be accepted and
/// then ignored on the stdin path, because only the TOML task key (`Task.pretty`)
/// reached `render_html_pretty`.
#[test]
fn html_pretty_flag_indents_the_output() {
    let input = "# Title\n\nText with **strong**.\n";
    let (_code, compact, _err) = run_cli(input, &["--plugin", "markdown", "--format", "html"]);
    let (_code, pretty, _err) = run_cli(
        input,
        &["--plugin", "markdown", "--format", "html", "--pretty"],
    );

    // The compact mode keeps the paragraph text on the opening tag's line.
    assert!(contains("<p>Text with ").eval(&compact));
    // The pretty mode puts every child on its own indented line.
    assert!(contains("<p>\n  Text with ").eval(&pretty));
    assert!(contains("\n  <strong>").eval(&pretty));
    assert_ne!(compact, pretty);
}

/// The same second mode has to stay reachable from the TOML path
/// (`pretty = true` on a task with `format = "html"`).
#[test]
fn config_pretty_key_indents_the_html_task_output() {
    let dir = tempdir().expect("temp dir");
    fs::write(
        dir.path().join("doc.md"),
        "# Title\n\nText with **strong**.\n",
    )
    .expect("source");
    fs::write(
        dir.path().join("pendon.toml"),
        r#"[[task]]
name = "Doc (HTML)"
input = "./doc.md"
output = "./out/doc.html"
plugin = "markdown"
format = "html"
pretty = true
"#,
    )
    .expect("config");

    let mut cmd = cargo_bin_cmd!("pendon");
    cmd.current_dir(dir.path()).arg("run").assert().success();

    let out = fs::read_to_string(dir.path().join("out/doc.html")).expect("task output");
    assert!(contains("<p>\n  Text with ").eval(&out));
    assert!(contains("\n  <strong>").eval(&out));
}
