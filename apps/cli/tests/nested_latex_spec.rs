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
fn latex_renders_in_paragraph_and_escapes_literal_braces() {
    let input = "Irure $H \\rightarrow O$ duis {adipisicing} cupidatat.\n";
    let (_code, out, _err) = run_cli(input, &["--plugin", "latex,markdown", "--format", "solid"]);

    assert!(contains("latex latex-inline").eval(&out));
    assert!(contains("katex").eval(&out));
    // `{...}` in running text must survive JSX embedding as HTML entities.
    assert!(contains("&#123;adipisicing&#125;").eval(&out));
    assert!(!contains("{adipisicing}").eval(&out));
}

#[test]
fn latex_renders_inside_figure_caption() {
    let input = "!![Alt](https://x.test/a.webp)[.foo]{con: jux} Et {foo} labore $x^2$ qui.\n";
    let (_code, out, _err) = run_cli(
        input,
        &["--plugin", "img,markdown,latex", "--format", "solid"],
    );

    assert!(contains("<figcaption>").eval(&out));
    // Math inside the caption is rendered by the nested inline pipeline...
    assert!(contains("latex latex-inline").eval(&out));
    // ...and literal braces are escaped as entities.
    assert!(contains("&#123;foo&#125;").eval(&out));
    assert!(!contains("{foo}").eval(&out));
}

#[test]
fn latex_renders_inside_table_cell() {
    let input = "| Foo | Bar |\n| --- | --- |\n| $H \\rightarrow O$ | {foo} |\n";
    let (_code, out, _err) = run_cli(
        input,
        &["--plugin", "table,markdown,latex", "--format", "solid"],
    );

    assert!(contains("<td>").eval(&out));
    assert!(contains("latex latex-inline").eval(&out));
    assert!(contains("&#123;foo&#125;").eval(&out));
    assert!(!contains("{foo}").eval(&out));
}

#[test]
fn latex_renders_inside_dialog_content() {
    let input = "Revan: \"Maka $x^2$ benar\"\nStevano: \"Setuju\"\n";
    let (_code, out, _err) = run_cli(input, &["--plugin", "dialog,latex", "--format", "solid"]);

    assert!(contains("<dl>").eval(&out));
    assert!(contains("<q>").eval(&out));
    assert!(contains("latex latex-inline").eval(&out));
}

#[test]
fn without_latex_plugin_math_is_left_literal_but_braces_still_escaped() {
    let input = "!![Alt](https://x.test/a.webp) Et {foo} labore $x^2$ qui.\n";
    let (_code, out, _err) = run_cli(input, &["--plugin", "img,markdown", "--format", "solid"]);

    // No `latex` in the plugin list means `$...$` is not interpreted.
    assert!(!contains("latex latex-inline").eval(&out));
    assert!(!contains("katex").eval(&out));
    assert!(contains("$x^2$").eval(&out));
    // Brace escaping is a renderer concern and still applies.
    assert!(contains("&#123;foo&#125;").eval(&out));
}

#[test]
fn dollar_inside_link_destination_is_not_treated_as_math() {
    let input =
        "!![Alt](https://x.test/a.webp) Cap [labore](/foo/bar^--$! \"Buy Foo!\") $x^2$ end.\n";
    let (_code, out, _err) = run_cli(
        input,
        &["--plugin", "img,markdown,latex", "--format", "solid"],
    );

    // The `$!` inside the link destination must not swallow the link syntax.
    assert!(contains("href=\"/foo/bar^--$!\" title=\"Buy Foo!\"").eval(&out));
    assert!(contains(">labore</a>").eval(&out));
    // The real inline math after the link still renders.
    assert!(contains("latex latex-inline").eval(&out));
}
