use pendon_core::{Event, NodeKind};
use syntect::html::ClassedHTMLGenerator;
mod mappers;
use syntect::parsing::SyntaxSet;

pub fn process(events: &[Event]) -> Vec<Event> {
    let mut out: Vec<Event> = Vec::with_capacity(events.len());
    let mut i = 0usize;
    while i < events.len() {
        match &events[i] {
            Event::StartNode(NodeKind::CodeFence) => {
                // Collect attrs + inner text until EndNode(CodeFence)
                let mut j = i + 1;
                let mut lang: Option<String> = None;
                let mut debug: Option<String> = None;
                let mut inner = String::new();
                while j < events.len() {
                    match &events[j] {
                        Event::Attribute { name, value } => {
                            if name == "lang" {
                                lang = Some(value.clone());
                            } else if name == "syntect_debug" {
                                debug = Some(value.clone());
                            }
                        }
                        Event::Text(t) => inner.push_str(t),
                        Event::EndNode(NodeKind::CodeFence) => break,
                        _ => {}
                    }
                    j += 1;
                }
                // Default: pass-through if no end found
                if j >= events.len() {
                    out.push(events[i].clone());
                    i += 1;
                    continue;
                }
                // Resolve debug mode: attribute overrides env var
                let env_debug = std::env::var("PENDON_SYNTECT_DEBUG").ok();
                let debug_mode = debug
                    .as_deref()
                    .map(|s| s.to_string())
                    .or(env_debug)
                    .map(|s| s.to_lowercase());
                let highlighted = highlight_output(&inner, lang.as_deref(), debug_mode.as_deref());
                out.push(Event::StartNode(NodeKind::CodeFence));
                if let Some(l) = lang {
                    out.push(Event::Attribute {
                        name: "lang".to_string(),
                        value: l,
                    });
                }
                // Mark as raw HTML payload for downstream renderer
                out.push(Event::Attribute {
                    name: "raw_html".to_string(),
                    value: "1".to_string(),
                });
                out.push(Event::Text(highlighted));
                out.push(Event::EndNode(NodeKind::CodeFence));
                i = j + 1;
            }
            _ => {
                out.push(events[i].clone());
                i += 1;
            }
        }
    }
    out
}

fn highlight_output(code: &str, lang: Option<&str>, debug_mode: Option<&str>) -> String {
    // Load only built-in default syntaxes
    let ss = load_syntax_sets();
    // Select syntax reference from defaults; special-case TypeScript/TSX → JavaScript fallback
    let (syntax, ss_for_gen) = match lang {
        Some(l) => {
            let token = normalize_lang_token(l);
            let lang_lc = l.to_lowercase();
            let ext_candidates: &[&str] = match token {
                "JavaScript" => &["js"],
                "HTML" => &["html", "htm"],
                "XML" => &["xml"],
                "Rust" => &["rs"],
                "Python" => &["py"],
                _ => &[],
            };
            let (syntax, ss_ref) = match token {
                // Fallback TS/TSX to JavaScript
                "TypeScript" | "TSX" => {
                    let s = ss
                        .find_syntax_by_token("JavaScript")
                        .or_else(|| {
                            ext_candidates
                                .iter()
                                .find_map(|e| ss.find_syntax_by_extension(e))
                        })
                        .unwrap_or_else(|| ss.find_syntax_plain_text());
                    (s, &ss)
                }
                _ => {
                    let s = ss
                        .find_syntax_by_token(token)
                        .or_else(|| ss.find_syntax_by_token(&lang_lc))
                        .or_else(|| {
                            ext_candidates
                                .iter()
                                .find_map(|e| ss.find_syntax_by_extension(e))
                        })
                        .unwrap_or_else(|| ss.find_syntax_plain_text());
                    (s, &ss)
                }
            };
            (syntax, ss_ref)
        }
        None => (ss.find_syntax_plain_text(), &ss),
    };
    let mut gen = ClassedHTMLGenerator::new_with_class_style(
        syntax,
        ss_for_gen,
        syntect::html::ClassStyle::Spaced,
    );
    for line in code.lines() {
        let _ = gen.parse_html_for_line_which_includes_newline(&format!("{}\n", line));
    }
    let classed = gen.finalize();
    // Debug mode: output raw classed HTML
    if matches!(debug_mode, Some("classes")) {
        return wrap_lines_p(&classed);
    }
    // Transform classed spans to minimal tags with language-specific mapping
    let mapper = match lang.map(|l| l.to_lowercase()) {
        Some(ref l) if l == "html" || l == "xml" => mappers::html::map_classes_to_tag,
        // For TypeScript/TSX fallback we use default mapper (JavaScript classes)
        Some(ref l) if l == "ts" || l == "typescript" || l == "tsx" => {
            mappers::default::map_classes_to_tag
        }
        _ => mappers::default::map_classes_to_tag,
    };
    let minimized = minimize_html_with_mapper(classed, mapper);
    wrap_lines_p(&minimized)
}

fn normalize_lang_token(l: &str) -> &str {
    match l.to_lowercase().as_str() {
        // HTML/XML
        "html" => "HTML",
        "xml" => "XML",
        // TypeScript/TSX prefer real grammars when available
        "ts" | "typescript" => "TypeScript",
        "tsx" => "TSX",
        // JavaScript aliases (for completeness)
        "js" | "javascript" => "JavaScript",
        // Python
        "py" | "python" => "Python",
        // Rust
        "rs" | "rust" => "Rust",
        // Plain text
        "text" | "plain" => "Plain Text",
        _ => l,
    }
}

fn wrap_lines_p(s: &str) -> String {
    // Wrap each line in a block-level <p> without inserting
    // extra newlines between blocks (pre preserves newlines → extra gaps).
    // For empty lines, insert a zero-width space so the line has height.
    let mut out = String::new();
    for line in s.split('\n') {
        out.push_str("<p>");
        if line.is_empty() {
            out.push_str("&#8203;");
        } else {
            out.push_str(line);
        }
        out.push_str("</p>");
    }
    out
}

fn minimize_html_with_mapper(classed: String, mapper: fn(&str) -> Option<&'static str>) -> String {
    // Robust nested <span class="..."> ... </span> to minimal tag mapping using a stack.
    let mut out = String::new();
    let mut i = 0usize;
    let bytes = classed.as_bytes();
    // Stack holds optional tag for each opened span
    let mut stack: Vec<Option<&'static str>> = Vec::new();
    while i < bytes.len() {
        if bytes[i] == b'<' {
            // Try to match opening span
            if i + 6 < bytes.len() && &bytes[i..i + 6] == b"<span " {
                // find class attribute start
                if let Some(cls_idx) = classed[i..].find("class=\"") {
                    let start = i + cls_idx + "class=\"".len();
                    if let Some(end_rel) = classed[start..].find('"') {
                        let end = start + end_rel;
                        let classes = &classed[start..end];
                        // advance to end of tag '>'
                        if let Some(gt_rel) = classed[end..].find('>') {
                            let gt = end + gt_rel;
                            let tag = mapper(classes);
                            stack.push(tag);
                            // emit opening minimal tag if mapped; drop span wrapper
                            if let Some(t) = tag {
                                out.push('<');
                                out.push_str(t);
                                out.push('>');
                            }
                            i = gt + 1;
                            continue;
                        }
                    }
                }
            }
            // Try to match closing span
            if i + 7 <= bytes.len() && &bytes[i..i + 7] == b"</span>" {
                // pop stack and emit closing tag if present
                if let Some(tag) = stack.pop().flatten() {
                    out.push_str("</");
                    out.push_str(tag);
                    out.push('>');
                }
                i += 7;
                continue;
            }
        }
        // Default: copy current char
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

// Removed legacy inline mapper; use language-specific mapper modules in `mappers/`.

fn load_syntax_sets() -> SyntaxSet {
    // Use only syntect's built-in default syntaxes with newline handling.
    SyntaxSet::load_defaults_newlines()
}
