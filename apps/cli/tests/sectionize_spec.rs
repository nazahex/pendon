use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

fn parse_ast(output: &[u8]) -> Value {
    serde_json::from_slice(output).expect("valid JSON")
}

#[test]
fn sectionizes_headings_and_ids() {
    let input = r#"# Title

Intro paragraph.

## Foo Bar

Some body.

### Child Piece

Child body.

## Bar Baz {#custom-id}

Tail text.
"#;

    let mut cmd = cargo_bin_cmd!("pendon");
    let output = cmd
        .arg("--plugin")
        .arg("markdown,sectionize")
        .arg("--format")
        .arg("ast")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v = parse_ast(&output);
    let sections = v
        .get("children")
        .and_then(|c| c.as_array())
        .expect("sections");

    // Preface section exists even before first h2
    let preface = sections.first().expect("preface section");
    assert_eq!(
        preface.get("type").and_then(|t| t.as_str()),
        Some("Section")
    );

    let foo = sections.get(1).expect("foo section");
    let foo_id = foo
        .get("attrs")
        .and_then(|a| a.get("id"))
        .and_then(|v| v.as_str());
    assert_eq!(foo_id, Some("foo-bar"));

    let foo_children = foo
        .get("children")
        .and_then(|c| c.as_array())
        .expect("foo children");

    let child_section = foo_children
        .iter()
        .find(|n| n.get("type").and_then(|t| t.as_str()) == Some("Section"))
        .expect("child section");
    let child_id = child_section
        .get("attrs")
        .and_then(|a| a.get("id"))
        .and_then(|v| v.as_str());
    assert_eq!(child_id, Some("child-piece"));

    let bar = sections.get(2).expect("bar section");
    let bar_id = bar
        .get("attrs")
        .and_then(|a| a.get("id"))
        .and_then(|v| v.as_str());
    assert_eq!(bar_id, Some("custom-id"));
}

#[test]
fn renders_heading_ids_in_html() {
    let input = "## Foo Bar\n\nBody\n\n## Bar {#custom-id}\n";

    let mut cmd = cargo_bin_cmd!("pendon");
    let output = cmd
        .arg("--plugin")
        .arg("markdown,sectionize")
        .arg("--format")
        .arg("html")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let html = String::from_utf8_lossy(&output);
    assert!(html.contains("<section id=\"foo-bar\">"));
    assert!(html.contains("<section id=\"custom-id\">"));
    assert!(
        !html.contains("<h2 id=\""),
        "ids should be on sections, not headings"
    );
}

#[test]
fn maintains_heading_hierarchy_outside_lists() {
    let input = r#"## Lists

### Unordered

- a
- b

### Ordered

1. one
2. two

### After

Tail

## Next

Text
"#;

    let mut cmd = cargo_bin_cmd!("pendon");
    let output = cmd
        .arg("--plugin")
        .arg("markdown,sectionize")
        .arg("--format")
        .arg("ast")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v = parse_ast(&output);
    let sections = v
        .get("children")
        .and_then(|c| c.as_array())
        .expect("sections");

    let lists = sections.first().expect("lists section");
    let list_id = lists
        .get("attrs")
        .and_then(|a| a.get("id"))
        .and_then(|v| v.as_str());
    assert_eq!(list_id, Some("lists"));

    let list_sections: Vec<&Value> = lists
        .get("children")
        .and_then(|c| c.as_array())
        .expect("list children")
        .iter()
        .filter(|n| n.get("type").and_then(|t| t.as_str()) == Some("Section"))
        .collect();

    let ids: Vec<Option<&str>> = list_sections
        .iter()
        .map(|s| {
            s.get("attrs")
                .and_then(|a| a.get("id"))
                .and_then(|v| v.as_str())
        })
        .collect();

    assert_eq!(ids, vec![Some("unordered"), Some("ordered"), Some("after")]);

    let next = sections.get(1).expect("next section");
    let next_id = next
        .get("attrs")
        .and_then(|a| a.get("id"))
        .and_then(|v| v.as_str());
    assert_eq!(next_id, Some("next"));
}

#[test]
fn lifts_headings_out_of_lists() {
    let input = r#"1. Intro item

### After

Tail text
"#;

    let mut cmd = cargo_bin_cmd!("pendon");
    let output = cmd
        .arg("--plugin")
        .arg("markdown,sectionize")
        .arg("--format")
        .arg("ast")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v = parse_ast(&output);
    let sections = v
        .get("children")
        .and_then(|c| c.as_array())
        .expect("sections");

    // Preface holds the list content, next section should be the heading outside the list
    let after = sections.get(1).expect("after section");
    let after_id = after
        .get("attrs")
        .and_then(|a| a.get("id"))
        .and_then(|v| v.as_str());
    assert_eq!(after_id, Some("after"));
}

#[test]
fn preserves_section_nesting_around_lists() {
    let input = r#"# Demo: Lists and Headings

## UL Sebelum Heading (H2)

Paragraf sebelum daftar tak berurut.

- Item satu
- Item dua
  - Sub-satu
  - Sub-dua

### Setelah UL (H3)

Paragraf setelah ul.

## OL dengan Nested (H2)

Paragraf sebelum daftar berurut.

1. Langkah satu
2. Langkah dua
   - Catatan A
   - Catatan B

### Setelah OL (H3)

Paragraf setelah ol.

## Heading di Dalam List (H2)

- Pra-heading di dalam list
- Masih dalam list

### H3 Seharusnya Keluar (H3)

Paragraf memastikan h3 tidak dinest di dalam list.
"#;

    let mut cmd = cargo_bin_cmd!("pendon");
    let output = cmd
        .arg("--plugin")
        .arg("markdown,sectionize")
        .arg("--format")
        .arg("ast")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v = parse_ast(&output);
    let sections = v
        .get("children")
        .and_then(|c| c.as_array())
        .expect("sections");

    fn find_section<'a>(nodes: &'a [Value], id: &str) -> Option<&'a Value> {
        for node in nodes {
            if node
                .get("attrs")
                .and_then(|a| a.get("id"))
                .and_then(|v| v.as_str())
                == Some(id)
            {
                return Some(node);
            }
            if let Some(children) = node.get("children").and_then(|c| c.as_array()) {
                if let Some(found) = find_section(children, id) {
                    return Some(found);
                }
            }
        }
        None
    }

    let find_section = |id: &str| -> &Value {
        find_section(sections, id).unwrap_or_else(|| panic!("missing section id {}", id))
    };

    let ul_sec = find_section("ul-sebelum-heading-h2");
    let ul_children = ul_sec
        .get("children")
        .and_then(|c| c.as_array())
        .expect("ul children");

    // List content should not wrap the following section
    assert!(ul_children
        .iter()
        .any(|n| n.get("type").and_then(|t| t.as_str()) == Some("BulletList")));
    assert!(ul_children
        .iter()
        .any(|n| n.get("type").and_then(|t| t.as_str()) == Some("Section")));

    let ol_sec = find_section("ol-dengan-nested-h2");
    let ol_children = ol_sec
        .get("children")
        .and_then(|c| c.as_array())
        .expect("ol children");
    assert!(ol_children
        .iter()
        .any(|n| n.get("type").and_then(|t| t.as_str()) == Some("OrderedList")));
    assert!(ol_children
        .iter()
        .any(|n| n.get("type").and_then(|t| t.as_str()) == Some("Section")));

    let h3_out = find_section("h3-seharusnya-keluar-h3");
    let h3_parent = sections
        .iter()
        .find(|s| {
            s.get("children")
                .and_then(|c| c.as_array())
                .map(|arr| arr.iter().any(|n| std::ptr::eq(n, h3_out)))
                .unwrap_or(false)
        })
        .expect("h3 parent section");

    let parent_id = h3_parent
        .get("attrs")
        .and_then(|a| a.get("id"))
        .and_then(|v| v.as_str());
    assert_eq!(parent_id, Some("heading-di-dalam-list-h2"));
}

#[test]
fn skips_empty_preface_when_only_frontmatter_precedes_heading() {
    let input = r#"---
title: Demo
---

# Title
"#;

    let mut cmd = cargo_bin_cmd!("pendon");
    let output = cmd
        .arg("--plugin")
        .arg("markdown,sectionize")
        .arg("--format")
        .arg("ast")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v = parse_ast(&output);
    let sections = v
        .get("children")
        .and_then(|c| c.as_array())
        .expect("sections");

    fn has_heading(node: &Value) -> bool {
        if node.get("type").and_then(|t| t.as_str()) == Some("Heading") {
            return true;
        }
        node.get("children")
            .and_then(|c| c.as_array())
            .map(|arr| arr.iter().any(has_heading))
            .unwrap_or(false)
    }

    let first_heading_idx = sections
        .iter()
        .position(has_heading)
        .expect("heading section");
    assert!(
        first_heading_idx <= 1,
        "heading should appear in first or second section"
    );
}
