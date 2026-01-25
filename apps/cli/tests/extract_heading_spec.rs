use assert_cmd::cargo::cargo_bin_cmd;
use regex::Regex;
use serde_json::Value;

fn parse_headings_export(output: &str) -> Value {
    let re = Regex::new(r"(?s)export const headings = (\[.*?\]);").expect("valid regex");
    let caps = re.captures(output).expect("headings export present");
    let json = caps.get(1).expect("payload").as_str();
    serde_json::from_str(json).expect("valid headings json")
}

#[test]
fn exports_headings_metadata_in_solid() {
    let input = r#"---
title: Demo
---

## Foo

### Bar

## Baz {#custom-baz}
"#;

    let mut cmd = cargo_bin_cmd!("pendon");
    let output = cmd
        .arg("--plugin")
        .arg("micomatter,markdown,sectionize,extract-heading")
        .arg("--format")
        .arg("solid")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let out = String::from_utf8_lossy(&output);

    let fm_pos = out
        .find("export const frontmatter =")
        .expect("frontmatter export");
    let headings_pos = out
        .find("export const headings =")
        .expect("headings export");
    assert!(fm_pos < headings_pos, "headings should follow frontmatter");

    let headings = parse_headings_export(&out);
    let arr = headings.as_array().expect("headings array");
    assert_eq!(arr.len(), 2);

    let foo = &arr[0];
    assert_eq!(foo.get("id").and_then(|v| v.as_str()), Some("foo"));
    assert_eq!(foo.get("text").and_then(|v| v.as_str()), Some("Foo"));
    assert_eq!(foo.get("level").and_then(|v| v.as_u64()), Some(2));
    let foo_sub = foo
        .get("subheadings")
        .and_then(|v| v.as_array())
        .expect("foo subheadings");
    assert_eq!(foo_sub.len(), 1);
    let bar = &foo_sub[0];
    assert_eq!(bar.get("id").and_then(|v| v.as_str()), Some("bar"));
    assert_eq!(bar.get("text").and_then(|v| v.as_str()), Some("Bar"));
    assert_eq!(bar.get("level").and_then(|v| v.as_u64()), Some(3));

    let baz = &arr[1];
    assert_eq!(baz.get("id").and_then(|v| v.as_str()), Some("custom-baz"));
    assert_eq!(baz.get("text").and_then(|v| v.as_str()), Some("Baz"));
    assert_eq!(baz.get("level").and_then(|v| v.as_u64()), Some(2));
}
