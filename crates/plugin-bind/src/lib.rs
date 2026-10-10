//! `plugin-bind` — bind `{{{lang[var] … }}}` data blocks into Solid props.
//!
//! RFC: [`docs/rfc/plugin-bind.md`](../../../docs/rfc/plugin-bind.md) ·
//! decision: [`docs/decisions/0004-pre-parse-bind-stage.md`](../../../docs/decisions/0004-pre-parse-bind-stage.md).
//!
//! The plugin has two stages and no middle:
//!
//! * [`extract`] runs **before** `pendon_core::parse` — a payload's lines look
//!   like markup (`#` → heading, `-` → list, blank → block separator), so the
//!   block must leave the text before the lexer sees it.
//! * [`resolve`] runs **after** every other plugin — the extras heads have by
//!   then emitted their literal `$var` attribute values.
//!
//! Structured values travel on [`Event::Attribute`] as
//! [`pendon_core::JSON_ATTR_PREFIX`] + compact JSON; `renderer-ast` re-hydrates
//! them and `renderer-solid` emits `name={…}`.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use pendon_core::{Event, Severity};
use serde_json::Value;

/// The languages a data block may declare (§2.1 of the RFC).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Json,
    Jsonc,
    Yaml,
    Toml,
    Csv,
    /// Pendon Markdown — specified, phase 2 (a phase-1 build warns and drops).
    Mdp,
}

impl Lang {
    /// Maps a block's `lang` token to a language, `None` when it is unknown.
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "json" => Some(Self::Json),
            "jsonc" => Some(Self::Jsonc),
            "yaml" | "yml" => Some(Self::Yaml),
            "toml" => Some(Self::Toml),
            "csv" => Some(Self::Csv),
            "mdp" => Some(Self::Mdp),
            _ => None,
        }
    }

    /// The canonical spelling used in diagnostics.
    pub fn name(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Jsonc => "jsonc",
            Self::Yaml => "yaml",
            Self::Toml => "toml",
            Self::Csv => "csv",
            Self::Mdp => "mdp",
        }
    }
}

/// The document's bound values, addressed by `var`. Later blocks win (§2.1).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Registry {
    values: BTreeMap<String, Value>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    /// `true` when no block bound a value.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// The value bound to `name`, if any.
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.values.get(name)
    }

    /// Binds `name`; a re-bind replaces the previous value (last wins).
    pub fn insert(&mut self, name: impl Into<String>, value: Value) {
        self.values.insert(name.into(), value);
    }

    /// The bound names, in document order.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.values.keys().map(String::as_str)
    }
}

/// The result of the pre-parse stage.
#[derive(Debug, Clone)]
pub struct Extract {
    /// The source with every data block replaced by one blank line.
    pub text: String,
    /// The values the blocks bound.
    pub registry: Registry,
    /// `Warning` / `Error` events raised while scanning.
    pub diagnostics: Vec<Event>,
    /// Every file a `(path)` block read, in scan order (§2.7).
    ///
    /// A caller that caches work **must** add these to its dependency set, or an
    /// edit to the file will not re-render the page. Recorded as soon as the file
    /// is read, even when the payload then fails to parse.
    pub external_files: Vec<PathBuf>,
}

/// `true` for a legal `var`: `ALPHA { ALPHA | DIGIT | "-" | "_" }` (§2.1).
///
/// The leading `ALPHA` is load-bearing, not cosmetic: it is what makes `$100`
/// ordinary text and therefore what makes "no `$` escape hatch is needed"
/// sound (§2.2).
pub fn is_valid_var(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|character| character.is_ascii_alphanumeric() || character == '-' || character == '_')
}

/// One step of a reference path (§2.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Segment<'a> {
    /// `.key` — an object key (same character set as a `var`).
    Key(&'a str),
    /// `[0]` — an array index.
    Index(usize),
}

/// A reference: the bound name it starts at, plus the path walked from it
/// (§2.2, §2.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ref<'a> {
    /// The `var` the reference starts at.
    pub root: &'a str,
    /// The steps walked from the root, left to right.
    pub path: Vec<Segment<'a>>,
}

impl Ref<'_> {
    /// `true` when the reference is a bare `$var` with no path.
    pub fn is_bare(&self) -> bool {
        self.path.is_empty()
    }

    /// The reference as it was written, for diagnostics (§2.5).
    pub fn render(&self) -> String {
        let mut text = format!("${}", self.root);
        for segment in &self.path {
            match segment {
                Segment::Key(key) => text.push_str(&format!(".{key}")),
                Segment::Index(index) => text.push_str(&format!("[{index}]")),
            }
        }
        text
    }
}

/// Parses `text` as a reference when it is **exactly** one (§2.2, §2.5):
/// `$var` followed by zero or more `.key`, `[index]` or `["key"]` steps.
///
/// Anything else — `$100`, `costs $5`, `a$a`, `$a.`, `$a[*]` — is `None`, so the
/// caller leaves the value as text. A miss **inside** a well-formed reference is
/// not a parse failure; that is [`lookup`]'s job.
pub fn parse_ref(text: &str) -> Option<Ref<'_>> {
    let mut rest = text.trim().strip_prefix('$')?;
    let end = rest
        .find(|character: char| !is_ref_char(character))
        .unwrap_or(rest.len());
    let root = &rest[..end];
    if !is_valid_var(root) {
        return None;
    }

    let mut path = Vec::new();
    rest = &rest[end..];
    while !rest.is_empty() {
        if let Some(after_dot) = rest.strip_prefix('.') {
            let end = after_dot
                .find(|character: char| !is_ref_char(character))
                .unwrap_or(after_dot.len());
            let key = &after_dot[..end];
            if !is_valid_var(key) {
                return None;
            }
            path.push(Segment::Key(key));
            rest = &after_dot[end..];
            continue;
        }
        if let Some(inner) = rest.strip_prefix('[') {
            let close = inner.find(']')?;
            let token = &inner[..close];
            match token
                .strip_prefix('"')
                .and_then(|token| token.strip_suffix('"'))
            {
                // A quoted key holds any text, so `["a-b"]` reaches a key a
                // dotted step cannot spell. It cannot contain `]`.
                Some(quoted) if !quoted.is_empty() => path.push(Segment::Key(quoted)),
                Some(_) => return None,
                None => path.push(Segment::Index(token.parse::<usize>().ok()?)),
            }
            rest = &inner[close + 1..];
            continue;
        }
        return None;
    }
    Some(Ref { root, path })
}

/// The bytes a `var` or a dotted `key` may use.
fn is_ref_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '-' || character == '_'
}

/// Walks a reference through the bound data, `None` on the first miss (§2.5).
///
/// An index addresses an **array** only: `$m[0]` is a miss even when `$m` has
/// the string key `"0"`, so a typo cannot silently read the wrong leaf.
pub fn lookup<'a>(registry: &'a Registry, reference: &Ref<'_>) -> Option<&'a Value> {
    let mut value = registry.get(reference.root)?;
    for segment in &reference.path {
        value = match segment {
            Segment::Key(key) => value.get(*key)?,
            Segment::Index(index) => value.get(*index)?,
        };
    }
    Some(value)
}

/// A `{{{lang[var] … }}}` head split from the text that follows it.
struct Head<'a> {
    lang: Option<Lang>,
    /// The raw `lang` token, for diagnostics.
    lang_token: &'a str,
    var: Option<&'a str>,
    /// A `(path)` group: `true` when one was written, legal or not (§2.7).
    file_group: bool,
    /// The `(path)` text, when the group was well formed.
    file: Option<&'a str>,
    /// `true` when the body sits on the head line (only legal for `mdp`).
    inline: bool,
    /// Text after the closing `]` (or after the `(path)` group).
    after: &'a str,
    error: Option<String>,
}

/// Splits `rest` (the text after `{{{`) into a head; malformed heads carry the
/// reason in `error` so the caller can warn and still consume the block.
fn parse_head(rest: &str) -> Head<'_> {
    let Some(open) = rest.find('[') else {
        let token = rest.split_whitespace().next().unwrap_or("");
        return Head {
            lang: Lang::parse(token),
            lang_token: token,
            var: None,
            file_group: false,
            file: None,
            inline: false,
            after: "",
            error: Some("a data block is missing its `[var]` name".to_string()),
        };
    };
    let token = rest[..open].trim();
    let lang = Lang::parse(token);
    let tail = &rest[open + 1..];
    let Some(close) = tail.find(']') else {
        return Head {
            lang,
            lang_token: token,
            var: None,
            file_group: false,
            file: None,
            inline: false,
            after: "",
            error: Some(format!("a `{token}` data block is missing its closing `]`")),
        };
    };
    let var = tail[..close].trim();
    let mut after = &tail[close + 1..];

    // §2.7: a `(path)` group after `]` takes the payload from a file. A `(` is a
    // path only for a data language — for `mdp` the head-line body keeps its
    // meaning (an mdp file payload is a phase-2 question, with `mdp` itself).
    let mut file_group = false;
    let mut file = None;
    let mut file_error = None;
    if matches!(lang, Some(lang) if lang != Lang::Mdp) {
        if let Some(inner) = after.trim_start().strip_prefix('(') {
            file_group = true;
            match inner.find(')') {
                Some(end) => {
                    let candidate = inner[..end].trim();
                    if candidate.is_empty() {
                        file_error = Some("a `(…)` file path is empty".to_string());
                    } else {
                        file = Some(candidate);
                    }
                    after = &inner[end + 1..];
                }
                None => {
                    file_error = Some("a `(…)` file path is missing its closing `)`".to_string());
                    after = "";
                }
            }
        }
    }

    let error = if lang.is_none() {
        Some(format!("`{token}` is not a known data-block language"))
    } else if !is_valid_var(var) {
        Some(format!("`{var}` is not a valid data-block name"))
    } else {
        file_error
    };
    Head {
        lang,
        lang_token: token,
        var: is_valid_var(var).then_some(var),
        file_group,
        file,
        inline: !file_group && !after.trim().is_empty(),
        after,
        error,
    }
}

/// The pre-parse stage (ADR-0004) for a document with **no** base directory —
/// stdin, or a string built in memory. A `(path)` block is a hard error here:
/// a relative path has nothing to resolve against (§2.7).
pub fn extract(source: &str) -> Extract {
    extract_impl(source, None)
}

/// The pre-parse stage for a document read from the directory `base`.
///
/// `base` is the **source file's** directory, never the process CWD (§2.7), so a
/// `(path)` resolves the same way whatever directory the build runs in, and a
/// source file nested at any depth can walk up with `../`.
pub fn extract_with_base(source: &str, base: &Path) -> Extract {
    extract_impl(source, Some(base))
}

/// The pre-parse stage: removes every data block from `source` and returns the
/// values it bound.
///
/// The text is scanned line by line; a block must start at the beginning of a
/// line. Each removed block is replaced by exactly one blank line so the
/// surrounding blocks stay separate.
fn extract_impl(source: &str, base: Option<&Path>) -> Extract {
    let lines: Vec<&str> = source.split('\n').collect();
    let mut out: Vec<&str> = Vec::with_capacity(lines.len());
    let mut registry = Registry::new();
    let mut diagnostics = Vec::new();
    let mut external_files = Vec::new();
    let mut i = 0usize;

    while i < lines.len() {
        let line = lines[i];
        let Some(rest) = line.trim_start().strip_prefix("{{{") else {
            out.push(line);
            i += 1;
            continue;
        };
        let head = parse_head(rest);

        // §2.7: a `(path)` block takes its payload from a file and is closed by
        // `}}}` on the head line itself.
        if head.file_group {
            if head.after.trim() != "}}}" {
                // A path beside a body, trailing text, or an unclosed head.
                // Consume through `}}}` when there is one, so a payload never
                // leaks into the document as markup (§2.1's drop rule).
                match find_terminator(&lines, i + 1) {
                    Some(end) => {
                        // The head's own problem wins over the shape, so an
                        // unclosed `(` is reported as such.
                        let reason = match &head.error {
                            Some(error) => error.clone(),
                            None if head.after.trim().is_empty() => {
                                "a data block with a `(path)` must not also carry a body"
                                    .to_string()
                            }
                            None => "a `(path)` data block must close with `}}}` on its head line"
                                .to_string(),
                        };
                        diagnostics
                            .push(warning(format!("[bind] {reason}; the block was dropped")));
                        out.push("");
                        i = end + 1;
                    }
                    None => {
                        diagnostics.push(warning(
                            "[bind] a data block is missing its closing `}}}` before the end of the document; it was left as text",
                        ));
                        out.push(line);
                        i += 1;
                    }
                }
                continue;
            }
            if let Some(error) = head.error {
                diagnostics.push(warning(format!("[bind] {error}; the block was dropped")));
                out.push("");
                i += 1;
                continue;
            }
            let (Some(lang), Some(name), Some(file)) = (head.lang, head.var, head.file) else {
                unreachable!("a clean file head has a language, a name and a path")
            };
            match read_external(file, base) {
                Ok((path, contents)) => {
                    if let Some(notice) = extension_warning(lang, &path) {
                        diagnostics.push(notice);
                    }
                    // Recorded even when the payload fails to parse below: the
                    // file is an input of this page either way (§2.7 decision 8).
                    external_files.push(path);
                    match parse_body(lang, &contents) {
                        Ok(value) => {
                            if registry.get(name).is_some() {
                                diagnostics.push(warning(format!(
                                    "[bind] `{name}` is bound more than once; the last block wins"
                                )));
                            }
                            registry.insert(name, value);
                        }
                        Err(error) => diagnostics.push(warning(format!(
                            "[bind] `{name}` is not valid {}: {error}; the block was dropped",
                            lang.name()
                        ))),
                    }
                }
                Err(diagnostic) => diagnostics.push(diagnostic),
            }
            out.push("");
            i += 1;
            continue;
        }

        if head.inline {
            let Some(_close) = head.after.rfind("}}}") else {
                diagnostics.push(warning(format!(
                    "[bind] a `{{{{{{{}[{}]` block is missing its closing `}}}}}}`; the line was left as text",
                    head.lang_token,
                    head.var.unwrap_or("")
                )));
                out.push(line);
                i += 1;
                continue;
            };
            match (head.error, head.lang) {
                (Some(error), _) => {
                    diagnostics.push(warning(format!("[bind] {error}; the block was dropped")))
                }
                (None, Some(Lang::Mdp)) => diagnostics.push(warning(
                    "[bind] `mdp` binding is not implemented yet; the block was dropped",
                )),
                (None, Some(lang)) => diagnostics.push(warning(format!(
                    "[bind] `{}` data must start on the line after the head; the block was dropped",
                    lang.name()
                ))),
                (None, None) => unreachable!("a parsed head always has a language"),
            }
            out.push("");
            i += 1;
            continue;
        }

        let Some(end) = find_terminator(&lines, i + 1) else {
            diagnostics.push(warning(
                "[bind] a data block is missing its closing `}}}` before the end of the document; it was left as text",
            ));
            out.push(line);
            i += 1;
            continue;
        };

        let body = lines[i + 1..end].join("\n");
        let name = head.var.unwrap_or("");
        match (head.error, head.lang) {
            (Some(error), _) => {
                diagnostics.push(warning(format!("[bind] {error}; the block was dropped")))
            }
            (None, Some(Lang::Mdp)) => diagnostics.push(warning(
                "[bind] `mdp` binding is not implemented yet; the block was dropped",
            )),
            (None, Some(lang)) => match parse_body(lang, &body) {
                Ok(value) => {
                    if registry.get(name).is_some() {
                        diagnostics.push(warning(format!(
                            "[bind] `{name}` is bound more than once; the last block wins"
                        )));
                    }
                    registry.insert(name, value);
                }
                Err(error) => diagnostics.push(warning(format!(
                    "[bind] `{name}` is not valid {}: {error}; the block was dropped",
                    lang.name()
                ))),
            },
            (None, None) => unreachable!("a parsed head always has a language"),
        }
        out.push("");
        i = end + 1;
    }

    Extract {
        text: out.join("\n"),
        registry,
        diagnostics,
        external_files,
    }
}

/// The index of the line that closes a block — trimmed content exactly `}}}` —
/// at or after `from`.
fn find_terminator(lines: &[&str], from: usize) -> Option<usize> {
    (from..lines.len()).find(|index| lines[*index].trim() == "}}}")
}

/// Reads a `(path)` payload (§2.7).
///
/// `base` is the source file's directory; `None` means the document came from
/// stdin, which is a hard `Error`: silently resolving against the process CWD
/// would make the same document mean different files in different shells.
fn read_external(file: &str, base: Option<&Path>) -> Result<(PathBuf, String), Event> {
    let Some(base) = base else {
        return Err(error(format!(
            "[bind] `{file}` cannot be read: the document came from stdin, so a relative path has no base directory; pass the source as a file"
        )));
    };
    let path = base.join(file);
    match fs::read_to_string(&path) {
        Ok(contents) => Ok((path, contents)),
        Err(cause) => Err(warning(format!(
            "[bind] cannot read `{}`: {cause}; the block was dropped",
            path.display()
        ))),
    }
}

/// Warns when a known data extension disagrees with the declared `lang` (§2.7).
///
/// `lang` is authoritative — the payload may live in a `.txt` file — so this is a
/// typo guard, not a rule, and an unknown extension is silent.
fn extension_warning(lang: Lang, path: &Path) -> Option<Event> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    let agrees = match extension.as_str() {
        "json" | "jsonc" => matches!(lang, Lang::Json | Lang::Jsonc),
        "yaml" | "yml" => matches!(lang, Lang::Yaml),
        "toml" => matches!(lang, Lang::Toml),
        "csv" => matches!(lang, Lang::Csv),
        _ => return None,
    };
    (!agrees).then(|| {
        warning(format!(
            "[bind] `{}` does not match the `.{}` extension of `{}`; `{}` was used",
            lang.name(),
            extension,
            path.display(),
            lang.name()
        ))
    })
}

/// A `Severity::Warning` diagnostic event.
fn warning(message: impl Into<String>) -> Event {
    Event::Diagnostic {
        severity: Severity::Warning,
        message: message.into(),
        span: None,
    }
}

/// A `Severity::Error` diagnostic event — reserved for a `(path)` block that has
/// no base directory to resolve against (§2.7).
fn error(message: impl Into<String>) -> Event {
    Event::Diagnostic {
        severity: Severity::Error,
        message: message.into(),
        span: None,
    }
}

/// Parses a block body into a JSON value, per the RFC's §4 table.
///
/// An empty body binds `null` with no warning; every other failure is reported
/// to the caller so the block can be dropped.
fn parse_body(lang: Lang, body: &str) -> Result<Value, String> {
    if body.trim().is_empty() {
        return Ok(Value::Null);
    }
    match lang {
        Lang::Json => serde_json::from_str(body).map_err(|error| error.to_string()),
        Lang::Jsonc => {
            serde_json::from_str(&strip_jsonc_comments(body)).map_err(|error| error.to_string())
        }
        Lang::Yaml => {
            let value: serde_yaml::Value =
                serde_yaml::from_str(body).map_err(|error| error.to_string())?;
            yaml_to_json(&value)
        }
        Lang::Toml => {
            let value: toml::Value = toml::from_str(body).map_err(|error| error.to_string())?;
            Ok(toml_to_json(&value))
        }
        Lang::Csv => parse_csv_value(body),
        Lang::Mdp => Err("mdp is Markdown, not a data serialization".to_string()),
    }
}

/// Removes `//` line and `/* … */` block comments that sit outside a string, so
/// the result is plain JSON (`jsonc`, RFC §4).
fn strip_jsonc_comments(source: &str) -> String {
    let chars: Vec<char> = source.chars().collect();
    let mut out = String::with_capacity(source.len());
    let mut i = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    while i < chars.len() {
        let c = chars[i];
        if in_string {
            out.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        if c == '"' {
            in_string = true;
            out.push(c);
            i += 1;
            continue;
        }
        if c == '/' && chars.get(i + 1) == Some(&'/') {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if c == '/' && chars.get(i + 1) == Some(&'*') {
            i += 2;
            while i < chars.len() && !(chars[i] == '*' && chars.get(i + 1) == Some(&'/')) {
                i += 1;
            }
            i = (i + 2).min(chars.len());
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

/// Converts a `serde_yaml::Value` to JSON. Non-finite floats (`inf`/`nan`) and
/// unresolved tags become `null` rather than failing the whole block.
fn yaml_to_json(value: &serde_yaml::Value) -> Result<Value, String> {
    use serde_yaml::Value as Yaml;
    Ok(match value {
        Yaml::Null => Value::Null,
        Yaml::Bool(flag) => Value::Bool(*flag),
        Yaml::Number(number) => yaml_number(number),
        Yaml::String(text) => Value::String(text.clone()),
        Yaml::Sequence(items) => Value::Array(
            items
                .iter()
                .map(yaml_to_json)
                .collect::<Result<Vec<_>, _>>()?,
        ),
        Yaml::Mapping(mapping) => {
            let mut object = serde_json::Map::new();
            // `serde_yaml` keeps YAML merge keys (`<<`) literal, so expand them
            // here: an explicit key always wins, and an earlier source wins over
            // a later one (YAML 1.1 merge semantics).
            for (key, item) in mapping {
                if !yaml_key_is_merge(key) {
                    continue;
                }
                for source in merge_sources(item)? {
                    if let Value::Object(entries) = source {
                        for (name, value) in entries {
                            object.entry(name).or_insert(value);
                        }
                    }
                }
            }
            for (key, item) in mapping {
                if yaml_key_is_merge(key) {
                    continue;
                }
                object.insert(yaml_key(key)?, yaml_to_json(item)?);
            }
            Value::Object(object)
        }
        Yaml::Tagged(tagged) => yaml_to_json(&tagged.value)?,
    })
}

/// `true` for the YAML merge key (`<<`).
fn yaml_key_is_merge(key: &serde_yaml::Value) -> bool {
    matches!(key, serde_yaml::Value::String(text) if text == "<<")
}

/// The mappings a `<<` merge key pulls in: one mapping or a sequence of them.
fn merge_sources(item: &serde_yaml::Value) -> Result<Vec<Value>, String> {
    use serde_yaml::Value as Yaml;
    match item {
        Yaml::Mapping(_) => Ok(vec![yaml_to_json(item)?]),
        Yaml::Sequence(items) => items.iter().map(yaml_to_json).collect(),
        _ => Err("a `<<` merge key expects a mapping or a sequence of mappings".to_string()),
    }
}

/// A finite float becomes a JSON number; `inf` / `nan` become `null`.
fn yaml_number(number: &serde_yaml::Number) -> Value {
    if let Some(int) = number.as_i64() {
        return Value::Number(int.into());
    }
    if let Some(uint) = number.as_u64() {
        return Value::Number(uint.into());
    }
    match number.as_f64() {
        Some(float) => serde_json::Number::from_f64(float)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        None => Value::Null,
    }
}

/// A YAML mapping key, which JSON requires to be a string.
fn yaml_key(key: &serde_yaml::Value) -> Result<String, String> {
    use serde_yaml::Value as Yaml;
    Ok(match key {
        Yaml::String(text) => text.clone(),
        Yaml::Bool(flag) => flag.to_string(),
        Yaml::Number(number) => match (number.as_i64(), number.as_u64(), number.as_f64()) {
            (Some(int), _, _) => int.to_string(),
            (_, Some(uint), _) => uint.to_string(),
            (_, _, Some(float)) => float.to_string(),
            _ => return Err("a mapping key is not a finite number".to_string()),
        },
        Yaml::Null => "null".to_string(),
        _ => return Err("only scalar mapping keys are supported".to_string()),
    })
}

/// Converts a `toml::Value` to JSON. Datetimes become RFC 3339 strings and
/// non-finite floats become `null` (JSON has no `inf`/`nan`).
fn toml_to_json(value: &toml::Value) -> Value {
    match value {
        toml::Value::String(text) => Value::String(text.clone()),
        toml::Value::Integer(int) => Value::Number((*int).into()),
        toml::Value::Float(float) => serde_json::Number::from_f64(*float)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        toml::Value::Boolean(flag) => Value::Bool(*flag),
        toml::Value::Datetime(datetime) => Value::String(datetime.to_string()),
        toml::Value::Array(items) => Value::Array(items.iter().map(toml_to_json).collect()),
        toml::Value::Table(table) => {
            let mut object = serde_json::Map::new();
            for (key, item) in table {
                object.insert(key.clone(), toml_to_json(item));
            }
            Value::Object(object)
        }
    }
}

/// Parses CSV into an array of objects: the header row names the keys, and a
/// cell that is valid JSON becomes a value (§4).
fn parse_csv_value(body: &str) -> Result<Value, String> {
    let mut rows = parse_csv_rows(body)?.into_iter();
    let Some(header) = rows.next() else {
        return Ok(Value::Array(Vec::new()));
    };
    let mut records = Vec::new();
    for row in rows {
        let mut object = serde_json::Map::new();
        for (index, key) in header.iter().enumerate() {
            let cell = row.get(index).map(String::as_str).unwrap_or("");
            object.insert(key.clone(), csv_cell(cell));
        }
        records.push(Value::Object(object));
    }
    Ok(Value::Array(records))
}

/// One CSV cell: `NULL` is null, a cell that is valid JSON (and not a JSON
/// string) keeps its type, everything else stays a string.
fn csv_cell(cell: &str) -> Value {
    if cell == "NULL" {
        return Value::Null;
    }
    match serde_json::from_str::<Value>(cell) {
        Ok(value) if !matches!(value, Value::String(_)) => value,
        _ => Value::String(cell.to_string()),
    }
}

/// RFC 4180 rows: comma separated, `"` quoted fields with `""` escapes, records
/// split on `\n` / `\r\n`, embedded newlines preserved inside quotes.
fn parse_csv_rows(body: &str) -> Result<Vec<Vec<String>>, String> {
    let chars: Vec<char> = body.chars().collect();
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut row: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    let mut i = 0usize;

    while i < chars.len() {
        let c = chars[i];
        if in_quotes {
            if c == '"' {
                if chars.get(i + 1) == Some(&'"') {
                    field.push('"');
                    i += 2;
                    continue;
                }
                in_quotes = false;
                i += 1;
                continue;
            }
            field.push(c);
            i += 1;
            continue;
        }
        match c {
            '"' if field.is_empty() => {
                in_quotes = true;
                i += 1;
            }
            ',' => {
                row.push(std::mem::take(&mut field));
                i += 1;
            }
            '\r' | '\n' => {
                if c == '\r' && chars.get(i + 1) == Some(&'\n') {
                    i += 1;
                }
                row.push(std::mem::take(&mut field));
                rows.push(std::mem::take(&mut row));
                i += 1;
            }
            _ => {
                field.push(c);
                i += 1;
            }
        }
    }

    if in_quotes {
        return Err("a quoted CSV field is never closed".to_string());
    }
    if !field.is_empty() || !row.is_empty() {
        row.push(field);
        rows.push(row);
    }
    // A blank line yields a single empty field; drop those rows.
    rows.retain(|row| !(row.len() == 1 && row[0].is_empty()));
    Ok(rows)
}

/// The post-markdown stage: replaces `$var` extras values with the bound data.
///
/// Runs after every other plugin, so the extras heads have already emitted
/// their literal `$var` attribute values. A resolved value is transported as
/// [`pendon_core::JSON_ATTR_PREFIX`] + compact JSON; an unresolved reference is
/// reported and left as text.
pub fn resolve(events: &[Event], registry: &Registry) -> Vec<Event> {
    let mut out = Vec::with_capacity(events.len() + 4);
    let mut diagnostics = Vec::new();
    for event in events {
        match event {
            Event::Attribute { name, value } => out.push(Event::Attribute {
                name: name.clone(),
                value: resolve_value(value, registry, name, &mut diagnostics),
            }),
            other => out.push(other.clone()),
        }
    }
    out.extend(diagnostics);
    out
}

/// Resolves one attribute value, warning on every reference that does not
/// resolve.
fn resolve_value(
    value: &str,
    registry: &Registry,
    name: &str,
    diagnostics: &mut Vec<Event>,
) -> String {
    if let Some(reference) = parse_ref(value) {
        return match lookup(registry, &reference) {
            Some(bound) => encode_json(bound),
            None => {
                diagnostics.push(unresolved(&reference, name));
                value.to_string()
            }
        };
    }

    let Ok(mut parsed) = serde_json::from_str::<Value>(value) else {
        return value.to_string();
    };
    if !matches!(parsed, Value::Object(_) | Value::Array(_)) {
        return value.to_string();
    }

    let changed = substitute(&mut parsed, registry, name, diagnostics);
    if changed {
        encode_json(&parsed)
    } else {
        value.to_string()
    }
}

/// Replaces every string leaf that is exactly a reference (§2.2, §2.5) and
/// expands every `...$var` spread (§2.6), returning `true` when anything changed.
///
/// A spread's keys are **copied**, never walked: a `$…`-looking string inside
/// bound data is data, not a reference. The author's own keys are resolved
/// first, then merged over the spread, so an explicitly written key wins.
fn substitute(
    value: &mut Value,
    registry: &Registry,
    name: &str,
    diagnostics: &mut Vec<Event>,
) -> bool {
    match value {
        Value::String(text) => match parse_ref(text) {
            Some(reference) => match lookup(registry, &reference) {
                Some(bound) => {
                    *value = bound.clone();
                    true
                }
                None => {
                    diagnostics.push(unresolved(&reference, name));
                    false
                }
            },
            None => false,
        },
        Value::Array(items) => {
            let mut changed = false;
            for item in items {
                changed |= substitute(item, registry, name, diagnostics);
            }
            changed
        }
        Value::Object(object) => {
            // Take the spread **before** walking: the marker's own value is a
            // list of reference strings, not authored data to resolve.
            let spreads = object.remove(pendon_extra::SPREAD_KEY);
            let mut changed = spreads.is_some();
            for item in object.values_mut() {
                changed |= substitute(item, registry, name, diagnostics);
            }
            let Some(spreads) = spreads else {
                return changed;
            };
            let mut merged = serde_json::Map::new();
            for text in spread_refs(&spreads) {
                let Some(reference) = parse_ref(&text) else {
                    diagnostics.push(not_a_reference(&text, name));
                    continue;
                };
                match lookup(registry, &reference) {
                    Some(Value::Object(entries)) => {
                        for (key, item) in entries {
                            merged.insert(key.clone(), item.clone());
                        }
                    }
                    Some(_) => diagnostics.push(not_an_object(&text, name)),
                    None => diagnostics.push(unresolved(&reference, name)),
                }
            }
            for (key, item) in std::mem::take(object) {
                merged.insert(key, item);
            }
            *object = merged;
            true
        }
        _ => false,
    }
}

/// The reference strings a spread item carried, in source order (§2.6).
fn spread_refs(spreads: &Value) -> Vec<String> {
    match spreads {
        Value::Array(items) => items
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect(),
        Value::String(text) => vec![text.clone()],
        _ => Vec::new(),
    }
}

/// `JSON_ATTR_PREFIX` + compact JSON — the transport encoding (§4).
fn encode_json(value: &Value) -> String {
    let payload = serde_json::to_string(value).unwrap_or_else(|_| "null".to_string());
    format!("{}{}", pendon_core::JSON_ATTR_PREFIX, payload)
}

/// The "a reference did not resolve" warning (§2.5): an unbound `$var` when the
/// reference is bare, or a path step that missed.
fn unresolved(reference: &Ref<'_>, name: &str) -> Event {
    if reference.is_bare() {
        warning(format!(
            "[bind] `{}` is not bound to any data block; `{name}` was left as text",
            reference.render()
        ))
    } else {
        warning(format!(
            "[bind] `{}` found no value in `${}`; `{name}` was left as text",
            reference.render(),
            reference.root
        ))
    }
}

/// The "a spread's text is not a reference" warning (§2.6).
fn not_a_reference(text: &str, name: &str) -> Event {
    warning(format!(
        "[bind] `{text}` is not a reference; the spread was ignored in `{name}`"
    ))
}

/// The "a spread's value is not an object" warning (§2.6).
fn not_an_object(text: &str, name: &str) -> Event {
    warning(format!(
        "[bind] `{text}` is not an object; the spread was ignored in `{name}`"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pendon_core::NodeKind;
    use serde_json::json;

    /// Formats a block-form data block, so tests read like the author's source.
    fn block(lang: &str, var: &str, body: &str) -> String {
        format!("{{{{{{{}[{}]\n{}\n}}}}}}\n", lang, var, body)
    }

    fn messages(events: &[Event]) -> Vec<String> {
        events
            .iter()
            .filter_map(|event| match event {
                Event::Diagnostic { message, .. } => Some(message.clone()),
                _ => None,
            })
            .collect()
    }

    fn attr(name: &str, value: &str) -> Event {
        Event::Attribute {
            name: name.to_string(),
            value: value.to_string(),
        }
    }

    fn decode(value: &str) -> Value {
        let payload = pendon_core::json_attr_payload(value).expect("a JSON attr");
        serde_json::from_str(payload).expect("valid JSON payload")
    }

    /// The `value` of an `Attribute` event, panicking on any other event.
    fn value_of(event: &Event) -> String {
        match event {
            Event::Attribute { value, .. } => value.clone(),
            other => panic!("unexpected {other:?}"),
        }
    }

    /// A fresh temp directory for the external-file tests.
    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("pendon-bind-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    #[test]
    fn binds_json_and_removes_the_block() {
        let source = format!(
            "# Title\n\n{}\nbody\n",
            block("json", "a1", "{\n \"n\": 1\n}")
        );
        let result = extract(&source);
        assert_eq!(result.registry.get("a1"), Some(&json!({ "n": 1 })));
        assert!(
            result.diagnostics.is_empty(),
            "{:?}",
            messages(&result.diagnostics)
        );
        assert!(
            !result.text.contains("{{"),
            "block leaked:\n{}",
            result.text
        );
        assert!(result.text.contains("# Title"));
        assert!(result.text.contains("body"));
    }

    #[test]
    fn binds_every_phase_one_language() {
        let source = format!(
            "{}{}{}{}{}",
            block("jsonc", "j", "{\n // note\n \"a\": 1\n}"),
            block("yaml", "y", "base: &b\n  a: 1\ndev:\n  <<: *b\n  c: 2"),
            block("toml", "t", "[primitives]\nfloats = [1.5, inf, nan]"),
            block(
                "csv",
                "c",
                "id,nama,meta,aktif\n1,\"Budi, S.T.\",{\"role\":\"admin\"},true"
            ),
            block("json", "e", ""),
        );
        let result = extract(&source);
        assert!(
            result.diagnostics.is_empty(),
            "{:?}",
            messages(&result.diagnostics)
        );
        assert_eq!(result.registry.get("j"), Some(&json!({ "a": 1 })));
        assert_eq!(
            result.registry.get("y"),
            Some(&json!({ "base": { "a": 1 }, "dev": { "a": 1, "c": 2 } }))
        );
        assert_eq!(
            result.registry.get("t"),
            Some(&json!({ "primitives": { "floats": [1.5, null, null] } }))
        );
        assert_eq!(
            result.registry.get("c"),
            Some(&json!([
                { "id": 1, "nama": "Budi, S.T.", "meta": { "role": "admin" }, "aktif": true }
            ]))
        );
        assert_eq!(result.registry.get("e"), Some(&Value::Null));
    }

    #[test]
    fn a_duplicate_name_warns_and_the_last_block_wins() {
        let source = format!("{}{}", block("json", "x", "1"), block("json", "x", "2"));
        let result = extract(&source);
        assert_eq!(result.registry.get("x"), Some(&json!(2)));
        let messages = messages(&result.diagnostics);
        assert_eq!(messages.len(), 1);
        assert!(messages[0].contains("bound more than once"));
    }

    #[test]
    fn drops_a_block_with_an_unknown_language() {
        let source = format!("keep\n{}", block("xml", "x", "<a/>"));
        let result = extract(&source);
        assert!(result.registry.is_empty());
        assert!(result.text.contains("keep"));
        assert!(!result.text.contains("{{"));
        assert!(messages(&result.diagnostics)[0].contains("not a known data-block language"));
    }

    #[test]
    fn drops_a_block_with_an_invalid_name() {
        let result = extract(&block("json", "bad name", "1"));
        assert!(result.registry.is_empty());
        assert!(messages(&result.diagnostics)[0].contains("not a valid data-block name"));
    }

    #[test]
    fn a_same_line_body_is_only_legal_for_mdp() {
        let result = extract("{{{json[a] {\"n\":1} }}}\n");
        assert!(result.registry.is_empty());
        assert!(result.text.trim().is_empty());
        assert!(messages(&result.diagnostics)[0].contains("must start on the line after"));
    }

    #[test]
    fn mdp_is_reported_as_not_implemented() {
        let result = extract(&block("mdp", "m", "Foo *bar* baz."));
        assert!(result.registry.is_empty());
        assert!(messages(&result.diagnostics)[0].contains("mdp"));

        // A `(…)` group is a path only for a data language: for `mdp` the
        // head-line body keeps its meaning, and an `mdp` file payload stays a
        // phase-2 question alongside `mdp` itself (§2.7).
        let inline = extract("{{{mdp[m](hello) }}}\n");
        assert!(inline.registry.is_empty());
        assert!(messages(&inline.diagnostics)[0].contains("mdp"));
    }

    #[test]
    fn an_unclosed_block_is_left_as_text_with_a_warning() {
        let result = extract("{{{json[a]\n{ \"n\": 1 }\n");
        assert!(result.registry.is_empty());
        assert!(result.text.contains("{{{json[a]"));
        assert!(messages(&result.diagnostics)[0].contains("closing `}}}`"));
    }

    #[test]
    fn a_payload_that_fails_to_parse_drops_the_block() {
        let result = extract(&format!("body\n{}", block("json", "x", "{ not json }")));
        assert!(result.registry.is_empty());
        assert!(result.text.contains("body"));
        assert!(messages(&result.diagnostics)[0].contains("not valid json"));
    }

    #[test]
    fn resolve_replaces_a_scalar_reference() {
        let mut registry = Registry::new();
        registry.insert("card", json!({ "n": [1, 2] }));
        let events = vec![
            Event::StartNode(NodeKind::Document),
            Event::StartNode(NodeKind::Custom("Card".to_string())),
            attr("data", "$card"),
            Event::EndNode(NodeKind::Custom("Card".to_string())),
            Event::EndNode(NodeKind::Document),
        ];
        let out = resolve(&events, &registry);
        match &out[2] {
            Event::Attribute { value, .. } => assert_eq!(decode(value), json!({ "n": [1, 2] })),
            other => panic!("unexpected {other:?}"),
        }
        assert!(out
            .iter()
            .all(|event| !matches!(event, Event::Diagnostic { .. })));
    }

    #[test]
    fn resolve_walks_nested_extras_values() {
        let mut registry = Registry::new();
        registry.insert("foo", json!("bar"));
        registry.insert("rows", json!([{ "id": 1 }]));
        let events = vec![attr(
            "tabel",
            "{\"karyawan\":\"$rows\",\"nested\":{\"fooYi\":\"$foo\",\"n\":1}}",
        )];
        let out = resolve(&events, &registry);
        match &out[0] {
            Event::Attribute { value, .. } => assert_eq!(
                decode(value),
                json!({ "karyawan": [{ "id": 1 }], "nested": { "fooYi": "bar", "n": 1 } })
            ),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn resolve_leaves_unbound_references_as_text_and_warns() {
        let registry = Registry::new();
        let events = vec![attr("x", "$missing"), attr("y", "{\"a\":\"$missing\"}")];
        let out = resolve(&events, &registry);
        match &out[0] {
            Event::Attribute { value, .. } => assert_eq!(value, "$missing"),
            other => panic!("unexpected {other:?}"),
        }
        match &out[1] {
            Event::Attribute { value, .. } => assert_eq!(value, "{\"a\":\"$missing\"}"),
            other => panic!("unexpected {other:?}"),
        }
        let messages = messages(&out);
        assert_eq!(messages.len(), 2);
        assert!(messages[0].contains("`$missing` is not bound"));
    }

    #[test]
    fn resolve_ignores_dollar_text_that_is_not_a_reference() {
        let mut registry = Registry::new();
        registry.insert("a", json!(1));
        let events = vec![
            attr("price", "costs $5"),
            attr("mixed", "a$a"),
            attr("str", "plain"),
        ];
        let out = resolve(&events, &registry);
        assert!(out
            .iter()
            .all(|event| !matches!(event, Event::Diagnostic { .. })));
        match &out[1] {
            Event::Attribute { value, .. } => assert_eq!(value, "a$a"),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn csv_supports_quotes_escapes_and_embedded_newlines() {
        let body =
            "id,nama,catatan\n1,\"Budi, S.T.\",\"Baris 1\nBaris 2\"\n2,\"Siti \"\"B\"\"\",ok";
        let parsed = parse_body(Lang::Csv, body).expect("csv");
        assert_eq!(
            parsed,
            json!([
                { "id": 1, "nama": "Budi, S.T.", "catatan": "Baris 1\nBaris 2" },
                { "id": 2, "nama": "Siti \"B\"", "catatan": "ok" },
            ])
        );
    }

    #[test]
    fn jsonc_strips_comments_only_outside_strings() {
        let body = "{\n  // line\n  \"url\": \"https://x//y\", /* block */\n  \"n\": 1\n}";
        let parsed = parse_body(Lang::Jsonc, body).expect("jsonc");
        assert_eq!(parsed, json!({ "url": "https://x//y", "n": 1 }));
    }

    #[test]
    fn variable_names_are_restricted() {
        assert!(is_valid_var("data-X"));
        assert!(is_valid_var("foo_yi9"));
        assert!(!is_valid_var(""));
        assert!(!is_valid_var("bad name"));
        assert!(!is_valid_var("a.b"));
        // §2.1/§19: a `var` starts with a letter. That is what makes `$100`
        // ordinary text and "no `$` escape hatch is needed" sound (§2.2).
        assert!(!is_valid_var("1a"));
        assert!(!is_valid_var("_a"));

        let bare = parse_ref(" $data-X ").expect("a bare reference");
        assert!(bare.is_bare());
        assert_eq!(bare.root, "data-X");
        assert_eq!(parse_ref("x$y"), None);
        assert_eq!(parse_ref("$100"), None);
        assert_eq!(parse_ref("$"), None);
    }

    #[test]
    fn a_reference_parses_a_path_and_nothing_else() {
        let reference = parse_ref("$rows[1].name").expect("a path reference");
        assert_eq!(reference.root, "rows");
        assert_eq!(
            reference.path,
            vec![Segment::Index(1), Segment::Key("name")]
        );

        let quoted = parse_ref("$m[\"a-b\"]").expect("a quoted key");
        assert_eq!(quoted.path, vec![Segment::Key("a-b")]);

        // Shapes that are not references stay plain text (§2.2, §2.5).
        for text in ["$a.", "$a[*]", "$a[?x]", "$a..b", "$a x", "Total: $sum"] {
            assert_eq!(parse_ref(text), None, "{text}");
        }
    }

    #[test]
    fn paths_walk_objects_arrays_and_quoted_keys() {
        let mut registry = Registry::new();
        registry.insert(
            "config",
            json!({ "db": { "host": "localhost", "port": 5432 } }),
        );
        registry.insert("rows", json!([{ "name": "Budi" }, { "name": "Siti" }]));
        registry.insert("m", json!({ "a-b": "dash" }));

        let events = vec![
            attr("host", "$config.db.host"),
            attr("port", "$config.db.port"),
            attr("second", "$rows[1].name"),
            attr("dash", "$m[\"a-b\"]"),
        ];
        let out = resolve(&events, &registry);
        assert_eq!(decode(&value_of(&out[0])), json!("localhost"));
        assert_eq!(decode(&value_of(&out[1])), json!(5432));
        assert_eq!(decode(&value_of(&out[2])), json!("Siti"));
        assert_eq!(decode(&value_of(&out[3])), json!("dash"));
        assert!(out
            .iter()
            .all(|event| !matches!(event, Event::Diagnostic { .. })));
    }

    #[test]
    fn a_path_miss_warns_and_stays_literal() {
        let mut registry = Registry::new();
        registry.insert("rows", json!([{ "name": "Budi" }]));
        let events = vec![attr("gone", "$rows[9].name"), attr("typo", "$rows[0].nmae")];
        let out = resolve(&events, &registry);
        assert_eq!(value_of(&out[0]), "$rows[9].name");
        assert_eq!(value_of(&out[1]), "$rows[0].nmae");
        let messages = messages(&out);
        assert_eq!(messages.len(), 2);
        assert!(messages[0].contains("`$rows[9].name` found no value in `$rows`"));
    }

    #[test]
    fn an_index_addresses_arrays_and_never_a_numeric_string_key() {
        let mut registry = Registry::new();
        registry.insert("m", json!({ "0": "zero" }));
        let out = resolve(&[attr("x", "$m[0]")], &registry);
        assert_eq!(value_of(&out[0]), "$m[0]");
        assert!(messages(&out)[0].contains("found no value"));
    }

    #[test]
    fn a_spread_supplies_defaults_and_an_authored_key_wins() {
        let mut registry = Registry::new();
        registry.insert(
            "brand",
            json!({ "accent": "#0af", "mode": "dark", "scale": 1.0, "label": "$brand" }),
        );
        let value = format!(
            "{{\"{}\":[\"$brand\"],\"scale\":1.2}}",
            pendon_extra::SPREAD_KEY
        );
        let out = resolve(&[attr("theme", &value)], &registry);
        assert_eq!(
            decode(&value_of(&out[0])),
            // The authored `scale` wins; `label` is *data*, so the `$brand` inside
            // it is never walked as a reference.
            json!({ "accent": "#0af", "mode": "dark", "scale": 1.2, "label": "$brand" })
        );
        assert!(out
            .iter()
            .all(|event| !matches!(event, Event::Diagnostic { .. })));
    }

    #[test]
    fn the_later_of_two_spreads_wins() {
        let mut registry = Registry::new();
        registry.insert("base", json!({ "mode": "light", "n": 1 }));
        registry.insert("override", json!({ "mode": "dark" }));
        let value = format!(
            "{{\"{}\":[\"$base\",\"$override\"]}}",
            pendon_extra::SPREAD_KEY
        );
        let out = resolve(&[attr("theme", &value)], &registry);
        assert_eq!(
            decode(&value_of(&out[0])),
            json!({ "mode": "dark", "n": 1 })
        );
    }

    #[test]
    fn a_spread_that_is_not_an_object_warns_and_is_ignored() {
        let mut registry = Registry::new();
        registry.insert("rows", json!([1, 2]));
        let value = format!("{{\"{}\":[\"$rows\"],\"n\":2}}", pendon_extra::SPREAD_KEY);
        let out = resolve(&[attr("x", &value)], &registry);
        assert_eq!(decode(&value_of(&out[0])), json!({ "n": 2 }));
        assert!(messages(&out)[0].contains("`$rows` is not an object"));
    }

    #[test]
    fn a_spread_from_an_unbound_name_warns_and_is_ignored() {
        let registry = Registry::new();
        let value = format!("{{\"{}\":[\"$ghost\"],\"n\":2}}", pendon_extra::SPREAD_KEY);
        let out = resolve(&[attr("x", &value)], &registry);
        assert_eq!(decode(&value_of(&out[0])), json!({ "n": 2 }));
        assert!(messages(&out)[0].contains("`$ghost` is not bound"));
    }

    #[test]
    fn a_file_block_reads_relative_to_the_source_directory_at_any_depth() {
        let root = temp_dir("depth");
        let source_dir = root.join("src/deep");
        let data_dir = root.join("data");
        std::fs::create_dir_all(&source_dir).expect("source dir");
        std::fs::create_dir_all(&data_dir).expect("data dir");
        std::fs::write(data_dir.join("rows.csv"), "id,nama\n1,Budi\n").expect("csv");

        let result = extract_with_base("{{{csv[rows](../../data/rows.csv)}}}\n", &source_dir);
        assert_eq!(
            result.registry.get("rows"),
            Some(&json!([{ "id": 1, "nama": "Budi" }]))
        );
        assert!(
            result.diagnostics.is_empty(),
            "{:?}",
            messages(&result.diagnostics)
        );
        // The file is a dependency of the page, whatever directory the build runs
        // in (§2.7 decision 8).
        assert_eq!(result.external_files.len(), 1);
        assert!(result.external_files[0].ends_with("data/rows.csv"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_file_block_without_a_base_directory_is_an_error() {
        // stdin: a path resolves against the document's own directory or not at
        // all — a CWD fallback would mean different files per shell (§2.7 (4)).
        let result = extract("{{{csv[rows](../data/rows.csv)}}}\n");
        assert!(result.registry.is_empty());
        assert!(result.external_files.is_empty());
        assert_eq!(result.diagnostics.len(), 1);
        match &result.diagnostics[0] {
            Event::Diagnostic {
                severity, message, ..
            } => {
                assert_eq!(*severity, Severity::Error);
                assert!(message.contains("stdin"));
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn a_missing_external_file_warns_and_drops_the_block() {
        let root = temp_dir("missing");
        let result = extract_with_base("{{{json[x](nope.json)}}}\n", &root);
        assert!(result.registry.is_empty());
        assert!(messages(&result.diagnostics)[0].contains("cannot read"));
        assert!(result.external_files.is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_file_block_may_not_also_carry_a_body() {
        let root = temp_dir("body");
        std::fs::write(root.join("x.json"), "{\"n\":1}").expect("json");
        let source = String::from("keep\n{{{json[x](x.json)\n{\"n\":2}\n}}}\nafter\n");
        let result = extract_with_base(&source, &root);
        assert!(result.registry.is_empty());
        assert!(result.text.contains("keep"));
        assert!(result.text.contains("after"));
        assert!(!result.text.contains("{{"));
        assert!(messages(&result.diagnostics)[0].contains("must not also carry a body"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_malformed_file_head_reports_the_head_and_never_leaks_the_payload() {
        let root = temp_dir("malformed");
        // An unclosed `(`: the message names that, and the body is consumed with
        // the block instead of being parsed as markup.
        let source = String::from("keep\n{{{json[x](x.json\n{\"n\":2}\n}}}\nafter\n");
        let result = extract_with_base(&source, &root);
        assert!(result.registry.is_empty());
        assert!(!result.text.contains("{{"));
        assert!(!result.text.contains("\"n\""));
        assert!(messages(&result.diagnostics)[0].contains("missing its closing `)`"));

        // Trailing text after the group is the same class of mistake.
        let trailing = extract_with_base("{{{json[x](x.json)junk}}}\n", &root);
        assert!(trailing.registry.is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_path_that_is_not_a_file_path_is_left_as_text_when_unterminated() {
        // No `}}}` anywhere: the line stays source text rather than being
        // silently swallowed (§2.1's unterminated rule).
        let result = extract("{{{json[x](x.json)\n{\"n\": 1}\n");
        assert!(result.registry.is_empty());
        assert!(result.text.contains("{{{json[x](x.json)"));
    }

    #[test]
    fn a_known_extension_that_disagrees_with_lang_warns() {
        let root = temp_dir("ext");
        std::fs::write(root.join("brand.json"), "{\"a\": 1}").expect("json");
        std::fs::write(root.join("notes.txt"), "{\"a\": 1}").expect("txt");
        let source =
            String::from("{{{yaml[brand](brand.json)}}}\n\n{{{json[notes](notes.txt)}}}\n");
        let result = extract_with_base(&source, &root);
        assert_eq!(result.registry.get("brand"), Some(&json!({ "a": 1 })));
        assert_eq!(result.registry.get("notes"), Some(&json!({ "a": 1 })));
        // `lang` is authoritative; the extension is a typo guard, and an unknown
        // extension is silent (§2.7 decision 6).
        let messages = messages(&result.diagnostics);
        assert_eq!(messages.len(), 1, "{messages:?}");
        assert!(messages[0].contains("does not match the `.json` extension"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn an_external_file_is_recorded_even_when_its_payload_fails_to_parse() {
        let root = temp_dir("dep");
        std::fs::write(root.join("bad.json"), "{ not json }").expect("json");
        let result = extract_with_base("{{{json[x](bad.json)}}}\n", &root);
        assert!(result.registry.is_empty());
        assert!(messages(&result.diagnostics)[0].contains("not valid json"));
        // Fixing the file must re-render the page, so it stays a dependency even
        // though this build dropped the block.
        assert_eq!(result.external_files.len(), 1);
        let _ = std::fs::remove_dir_all(&root);
    }
}
