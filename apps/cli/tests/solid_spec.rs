use assert_cmd::cargo::cargo_bin_cmd;
use predicates::str::contains;
use predicates::Predicate;

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
