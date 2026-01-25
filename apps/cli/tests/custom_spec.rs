use assert_cmd::cargo::cargo_bin_cmd;
use pendon_plugin_custom::load_spec_from_path;
use predicates::prelude::*;
use predicates::str::contains;
use serde_json::Value;
use std::path::Path;

fn repo_path(rel: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("apps dir")
        .parent()
        .expect("workspace root")
        .join(rel)
}

fn plugin_spec() -> String {
    let path = repo_path("sandbox/custom/plugins/foo.toml");
    format!("toml:{},markdown", path.to_string_lossy())
}

fn find_node<'a>(node: &'a Value, kind: &str) -> Option<&'a Value> {
    if node.get("type").and_then(|t| t.as_str()) == Some(kind) {
        return Some(node);
    }
    if let Some(children) = node.get("children").and_then(|c| c.as_array()) {
        for ch in children {
            if let Some(found) = find_node(ch, kind) {
                return Some(found);
            }
        }
    }
    None
}

#[test]
fn custom_plugin_emits_component_ast() {
    let mut cmd = cargo_bin_cmd!("pendon");
    let output = cmd
        .arg("--plugin")
        .arg(plugin_spec())
        .arg("--format")
        .arg("ast")
        .arg("--input")
        .arg(repo_path("sandbox/custom/src/demo.md"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).expect("valid JSON");
    let comp = find_node(&v, "Component").expect("component node present");
    let attrs = comp
        .get("attrs")
        .and_then(|a| a.as_object())
        .expect("attrs map");
    assert_eq!(attrs.get("name").and_then(|v| v.as_str()), Some("Foo"));
    assert_eq!(attrs.get("argument").and_then(|v| v.as_str()), Some("bar"));
    assert_eq!(
        attrs.get("attributes.qux").and_then(|v| v.as_str()),
        Some("fred")
    );
    assert_eq!(
        attrs.get("attributes.waldo").and_then(|v| v.as_str()),
        Some("8")
    );
    assert_eq!(
        attrs.get("attributes.isBar").and_then(|v| v.as_str()),
        Some("true")
    );
}

#[test]
fn solid_renderer_uses_custom_template_and_imports() {
    let spec =
        load_spec_from_path(repo_path("sandbox/custom/plugins/foo.toml")).expect("load spec");
    assert!(
        spec.renderer
            .as_ref()
            .and_then(|r| r.solid.as_ref())
            .and_then(|s| s.component_template.as_ref())
            .is_some(),
        "solid renderer template missing"
    );
    let mut cmd = cargo_bin_cmd!("pendon");
    let out = cmd
        .arg("--plugin")
        .arg(plugin_spec())
        .arg("--format")
        .arg("solid")
        .arg("--input")
        .arg(repo_path("sandbox/custom/src/demo.md"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8_lossy(&out).to_string();

    assert!(stdout
        .lines()
        .any(|l| l.trim_start().starts_with("import Foo from \"./Foo\"")));
    assert!(contains(
        "<Foo argument=\"bar\" attributes={{ qux: \"fred\", waldo: 8, isBar: true }}>"
    )
    .eval(&stdout));
}
