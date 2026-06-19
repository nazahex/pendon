use std::fs;
use std::path::{Path, PathBuf};

// Generate human-friendly class lists derived from syntect highlighting of sample code.
// Languages: TypeScript, HTML, Rust.
fn main() {
    let ss = syntect::parsing::SyntaxSet::load_defaults_newlines();
    let langs = [("ts", "TypeScript"), ("html", "HTML"), ("rust", "Rust")];
    // Resolve docs/grammar relative to current working directory; try multiple levels up
    let doc_candidates = [
        PathBuf::from("../docs/grammar"),
        PathBuf::from("../../docs/grammar"),
        PathBuf::from("../../../docs/grammar"),
        PathBuf::from("../../../../docs/grammar"),
        PathBuf::from("./docs/grammar"),
    ];
    let mut out_dir = None;
    for p in doc_candidates.iter() {
        let parent = p.parent().unwrap_or(Path::new("."));
        if parent.exists() {
            out_dir = Some(p.clone());
            break;
        }
    }
    let out_dir = out_dir.unwrap_or_else(|| PathBuf::from("./docs/grammar"));
    if let Err(e) = fs::create_dir_all(&out_dir) {
        eprintln!("Error: cannot create docs/grammar: {}", e);
        std::process::exit(2);
    }

    for (slug, token) in langs.iter() {
        let path = out_dir.join(format!("{}.md", slug));
        let classes = if *token == "TypeScript" {
            // Prefer extracting scope tokens directly from the .tmLanguage grammar
            match find_ts_tmlanguage() {
                Some(grammar_path) => match collect_tokens_from_tmlanguage(&grammar_path) {
                    Ok(set) => set,
                    Err(err) => {
                        eprintln!(
                            "Warn: failed to parse {}: {} — falling back to sample highlighting",
                            grammar_path.display(),
                            err
                        );
                        collect_classes_via_sample(&ss, token)
                    }
                },
                None => {
                    eprintln!("Warn: typescript.tmLanguage not found — using sample highlighting");
                    collect_classes_via_sample(&ss, token)
                }
            }
        } else {
            collect_classes_via_sample(&ss, token)
        };

        let mut buf = String::new();
        buf.push_str(&format!("# {} Scopes\n\n", token));
        if *token == "TypeScript" {
            buf.push_str("- Note: Derived from parsing the TypeScript .tmLanguage grammar (captures, names).\n");
        } else {
            buf.push_str("- Note: Derived from highlighting sample content using syntect ClassedHTMLGenerator.\n");
        }
        buf.push_str("- Format: Unique CSS classes (sorted) representing scope categories used during HTML generation.\n\n");
        for sc in classes.iter() {
            buf.push_str("- ");
            buf.push_str(sc);
            buf.push('\n');
        }
        if let Err(e) = fs::write(&path, buf) {
            eprintln!("Error: cannot write {}: {}", path.display(), e);
            std::process::exit(2);
        }
    }
}

fn collect_classes_via_sample(
    ss: &syntect::parsing::SyntaxSet,
    token: &str,
) -> std::collections::BTreeSet<String> {
    use syntect::html::ClassStyle;
    let mut output_classes: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let syntax = if let Some(s) = ss.find_syntax_by_token(token) {
        s
    } else {
        let try_exts: &[&str] = match token {
            "TypeScript" => &["ts", "tsx", "js"],
            "HTML" => &["html", "htm"],
            "Rust" => &["rs"],
            _ => &[],
        };
        let mut found: Option<&syntect::parsing::SyntaxReference> = None;
        for ext in try_exts {
            if let Some(s) = ss.find_syntax_by_extension(ext) {
                found = Some(s);
                break;
            }
        }
        if let Some(s) = found {
            s
        } else {
            return output_classes;
        }
    };
    let sample = match token {
        "TypeScript" => sample_ts(),
        "HTML" => sample_html(),
        "Rust" => sample_rust(),
        _ => String::new(),
    };
    let mut gen =
        syntect::html::ClassedHTMLGenerator::new_with_class_style(syntax, ss, ClassStyle::Spaced);
    for line in sample.lines() {
        let _ = gen.parse_html_for_line_which_includes_newline(&format!("{}\n", line));
    }
    let classed = gen.finalize();
    let re = regex::Regex::new("class=\"([^\"]+)\"").unwrap();
    for caps in re.captures_iter(&classed) {
        if let Some(m) = caps.get(1) {
            for cls in m.as_str().split(' ') {
                let c = cls.trim();
                if !c.is_empty() {
                    output_classes.insert(c.to_string());
                }
            }
        }
    }
    output_classes
}

// Attempt to locate the TypeScript .tmLanguage grammar in the repo
fn find_ts_tmlanguage() -> Option<PathBuf> {
    let rels = [
        "crates/plugin-codeblock-syntect/src/grammar/typescript.tmLanguage",
        "sandbox/syntect/syntaxes/typescript.tmLanguage",
    ];
    let prefixes = ["./", "../", "../../", "../../../", "../../../../"]; // try up to 4 levels up
    let mut candidates: Vec<PathBuf> = Vec::new();
    for pre in prefixes.iter() {
        for rel in rels.iter() {
            candidates.push(PathBuf::from(format!("{}{}", pre, rel)));
        }
    }
    for p in candidates.iter() {
        if p.exists() {
            return Some(p.clone());
        }
    }
    None
}

// Parse a .tmLanguage (XML plist) and extract all scope name tokens
fn collect_tokens_from_tmlanguage(
    path: &Path,
) -> Result<std::collections::BTreeSet<String>, String> {
    use plist::Value;
    let val = Value::from_file(path).map_err(|e| format!("plist parse error: {}", e))?;
    let mut out: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    fn push_segments(s: &str, out: &mut std::collections::BTreeSet<String>) {
        for seg in s.split('.') {
            let seg = seg.trim();
            if seg.is_empty() {
                continue;
            }
            // Keep segments concise; avoid overly-specific vendor fragments
            out.insert(seg.to_string());
        }
    }

    fn walk_dict(dict: &plist::Dictionary, out: &mut std::collections::BTreeSet<String>) {
        // Top-level or context-level scope names
        if let Some(plist::Value::String(scope)) = dict.get("scopeName") {
            push_segments(scope, out);
        }
        if let Some(plist::Value::String(name)) = dict.get("name") {
            push_segments(name, out);
        }
        if let Some(plist::Value::String(cname)) = dict.get("contentName") {
            push_segments(cname, out);
        }

        // Captures: numeric keys -> { name: "..." }
        for key in ["captures", "beginCaptures", "endCaptures"].iter() {
            if let Some(plist::Value::Dictionary(caps)) = dict.get(*key) {
                for (_idx, v) in caps.iter() {
                    if let plist::Value::Dictionary(capdict) = v {
                        if let Some(plist::Value::String(n)) = capdict.get("name") {
                            push_segments(n, out);
                        }
                    }
                }
            }
        }

        // Patterns array: recurse into each sub-dict
        if let Some(plist::Value::Array(arr)) = dict.get("patterns") {
            for v in arr.iter() {
                match v {
                    plist::Value::Dictionary(d) => walk_dict(d, out),
                    _ => {}
                }
            }
        }

        // Repository: nested named contexts
        if let Some(plist::Value::Dictionary(repo)) = dict.get("repository") {
            for (_name, v) in repo.iter() {
                if let plist::Value::Dictionary(d) = v {
                    walk_dict(d, out);
                }
            }
        }
    }

    match val {
        plist::Value::Dictionary(d) => {
            walk_dict(&d, &mut out);
            Ok(out)
        }
        _ => Err("unexpected plist root (expected dictionary)".to_string()),
    }
}

fn sample_ts() -> String {
    let s = r#"// TypeScript sample covering keywords, types, strings, numbers, comments, operators, decorators
import { useState } from 'react';
// comment line
/* block comment */
@sealed class Foo<T extends number> implements IFoo { constructor(public x: T) {} }
function bar<T>(a: number, b: string): Promise<T> { return new Promise<T>((resolve) => resolve(a as unknown as T)); }
const baz: string = `Hello ${'world'} ${42}`; let x = 10; if (x < 20 && x !== 15) { x++; }
type U = string | number & boolean; interface I { a: string; }
enum E { A = 1, B = 2 }
"#;
    s.to_string()
}

fn sample_html() -> String {
    let s = r#"<!DOCTYPE html>
<!-- HTML sample: tags, attributes, comments, entities -->
<html lang=\"en\">
  <head>
    <meta charset=\"utf-8\" />
    <title>Sample &amp; Test</title>
    <style>/* inline style */ body { color: #333; }</style>
  </head>
  <body>
    <div id=\"app\" class=\"container\">Hello <span>world</span></div>
    <!-- comment -->
    <script>const x = 1 + 2; // inline script</script>
  </body>
</html>
"#;
    s.to_string()
}

fn sample_rust() -> String {
    let s = r##"//! Rust sample: keywords, types, lifetimes, attrs, macros, strings, raw strings, numbers, operators, comments
#![allow(dead_code)]
use std::fmt::{self, Display};
/// doc comment
#[derive(Debug)]
struct Point<'a> { x: i32, y: i32, name: &'a str }
impl<'a> Point<'a> { fn new(x: i32, y: i32, name: &'a str) -> Self { Self { x, y, name } } }
fn main() {
    let mut n = 42i32; let s = "hello"; let rs = r#"raw string"#; let bs = b"bytes"; // comment
    if n < 100 && n != 0 { n += 1; }
    println!("{} {:?}", s, n);
}
"##;
    s.to_string()
}
