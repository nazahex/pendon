use crate::info::ParsedInfo;
use crate::inline::{mark_inline, replace_markers};
use crate::mappers;
use crate::wrap::wrap_lines;
use syntect::html::ClassedHTMLGenerator;
use syntect::parsing::SyntaxSet;

pub fn highlight_output(code: &str, info: &ParsedInfo, debug_mode: Option<&str>) -> String {
    let marked = mark_inline(code, &info.inline_patterns, MARK_OPEN, MARK_CLOSE);
    let ss = load_syntax_sets();
    let (syntax, ss_for_gen) = select_syntax(&ss, info.lang.as_deref());

    let mut gen = ClassedHTMLGenerator::new_with_class_style(
        syntax,
        ss_for_gen,
        syntect::html::ClassStyle::Spaced,
    );
    for line in marked.lines() {
        let _ = gen.parse_html_for_line_which_includes_newline(&format!("{}\n", line));
    }
    let classed = gen.finalize();

    if matches!(debug_mode, Some("classes")) {
        let classed = replace_markers(&classed, MARK_OPEN, MARK_CLOSE);
        return wrap_lines(&classed, &info.line_ranges);
    }

    let mapper = match info.lang.as_deref().map(|l| l.to_lowercase()) {
        Some(ref l) if l == "html" || l == "xml" => mappers::html::map_classes_to_tag,
        Some(ref l) if l == "ts" || l == "typescript" || l == "tsx" => {
            mappers::default::map_classes_to_tag
        }
        _ => mappers::default::map_classes_to_tag,
    };
    let minimized = minimize_html_with_mapper(classed, mapper);
    let inline = replace_markers(&minimized, MARK_OPEN, MARK_CLOSE);
    wrap_lines(&inline, &info.line_ranges)
}

const MARK_OPEN: &str = "__PSTRONG_OPEN__";
const MARK_CLOSE: &str = "__PSTRONG_CLOSE__";

fn select_syntax<'a>(
    ss: &'a SyntaxSet,
    lang: Option<&str>,
) -> (&'a syntect::parsing::SyntaxReference, &'a SyntaxSet) {
    match lang {
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
                "TypeScript" | "TSX" => {
                    let s = ss
                        .find_syntax_by_token("JavaScript")
                        .or_else(|| {
                            ext_candidates
                                .iter()
                                .find_map(|e| ss.find_syntax_by_extension(e))
                        })
                        .unwrap_or_else(|| ss.find_syntax_plain_text());
                    (s, ss)
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
                    (s, ss)
                }
            };
            (syntax, ss_ref)
        }
        None => (ss.find_syntax_plain_text(), ss),
    }
}

fn normalize_lang_token(l: &str) -> &str {
    match l.to_lowercase().as_str() {
        "html" => "HTML",
        "xml" => "XML",
        "ts" | "typescript" => "TypeScript",
        "tsx" => "TSX",
        "js" | "javascript" => "JavaScript",
        "py" | "python" => "Python",
        "rs" | "rust" => "Rust",
        "text" | "plain" => "Plain Text",
        _ => l,
    }
}

fn minimize_html_with_mapper(classed: String, mapper: fn(&str) -> Option<&'static str>) -> String {
    let mut out = String::new();
    let mut i = 0usize;
    let bytes = classed.as_bytes();
    let mut stack: Vec<Option<&'static str>> = Vec::new();
    while i < bytes.len() {
        if bytes[i] == b'<' {
            if i + 6 < bytes.len() && &bytes[i..i + 6] == b"<span " {
                if let Some(cls_idx) = classed[i..].find("class=\"") {
                    let start = i + cls_idx + "class=\"".len();
                    if let Some(end_rel) = classed[start..].find('"') {
                        let end = start + end_rel;
                        let classes = &classed[start..end];
                        if let Some(gt_rel) = classed[end..].find('>') {
                            let gt = end + gt_rel;
                            let tag = mapper(classes);
                            stack.push(tag);
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
            if i + 7 <= bytes.len() && &bytes[i..i + 7] == b"</span>" {
                if let Some(tag) = stack.pop().flatten() {
                    out.push_str("</");
                    out.push_str(tag);
                    out.push('>');
                }
                i += 7;
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

fn load_syntax_sets() -> SyntaxSet {
    SyntaxSet::load_defaults_newlines()
}
