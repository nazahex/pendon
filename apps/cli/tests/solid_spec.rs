use assert_cmd::cargo::cargo_bin_cmd;
use predicates::str::contains;
use predicates::Predicate;
use serde_json::Value;

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
fn solid_component_signature() {
    let input = "# Title\n";
    let (_code, out, _err) = run_cli(input, &["--plugin", "markdown", "--format", "solid"]);
    assert!(contains("export default function PendonView()").eval(&out));
    assert!(contains("<h1>Title</h1>").eval(&out));
}

#[test]
fn solid_inline_and_lists() {
    let input = "Text with **strong** and __bold__, *em* and _italic_, `code`, and [link](https://example.com).\n\n- a\n  - b\n    - c\n";
    let (_code, out, _err) = run_cli(input, &["--plugin", "markdown", "--format", "solid"]);
    assert!(contains("<strong>strong</strong>").eval(&out));
    assert!(contains("<b>bold</b>").eval(&out));
    assert!(contains("<em>em</em>").eval(&out));
    assert!(contains("<i>italic</i>").eval(&out));
    assert!(contains("<code>code</code>").eval(&out));
    assert!(contains("<a href=\"https://example.com\">link</a>").eval(&out));
    assert!(contains("<ul>").eval(&out));
    assert!(contains("<li>a").eval(&out));
    assert!(contains("<li>b").eval(&out));
    assert!(contains("<li>c").eval(&out));
}

#[test]
fn solid_blockquote_and_table() {
    let input = "> quoted\n> block\n\n| A | B |\n| C | D |\n";
    let (_code, out, _err) = run_cli(input, &["--plugin", "markdown", "--format", "solid"]);
    assert!(contains("<blockquote>").eval(&out));
    assert!(contains("quoted").eval(&out));
    assert!(contains("<table>").eval(&out));
    assert!(contains("<thead>").eval(&out));
    assert!(contains("<tbody>").eval(&out));
    assert!(contains("<th>A</th>").eval(&out));
    assert!(contains("<td>C</td>").eval(&out));
}

#[test]
fn solid_exports_frontmatter_object() {
    let input = "---\ntitle: \"Demo\"\ndraft: false\nscore: 95.5\n---\n\n# Hello\n";
    let (_code, out, _err) = run_cli(
        input,
        &["--plugin", "micomatter,markdown", "--format", "solid"],
    );
    let fm_line = out
        .lines()
        .find(|l| l.starts_with("export const frontmatter"))
        .expect("frontmatter export present");
    let json_str = fm_line
        .trim_start_matches("export const frontmatter = ")
        .trim_end_matches(';');
    let fm: Value = serde_json::from_str(json_str).expect("valid frontmatter JSON");
    assert_eq!(fm.get("title").and_then(|v| v.as_str()), Some("Demo"));
    assert_eq!(fm.get("draft").and_then(|v| v.as_bool()), Some(false));
    assert_eq!(fm.get("score").and_then(|v| v.as_f64()), Some(95.5));
    assert!(contains("<h1>Hello</h1>").eval(&out));
    assert!(!contains("---").eval(&out));
}
