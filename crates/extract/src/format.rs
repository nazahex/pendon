//! The three destination formats, and what happens when the file is already there.
//!
//! P.1 of [`docs/archive/fulfilled/plugin-extract.md`](../../../docs/archive/fulfilled/plugin-extract.md)
//! makes `format` mandatory and independent of the file extension: `format = "yaml"`
//! writing `out/x.json` produces YAML.
//!
//! JSON and YAML carry the flat array the element produced. TOML cannot express a
//! top-level array, so the same rows are written as an array of tables under the
//! element's name:
//!
//! ```toml
//! [[cite]]
//! citation_number = 1
//! ```
//!
//! **A destination accumulates.** Two blocks — or two pages — resolving to one
//! path used to overwrite it, so the last one to render won and every other row
//! was lost. [`Format::extend`] reads whatever the file already holds and
//! **appends** the rows it does not have yet:
//!
//! - a re-run of the same input adds nothing (row equality is structural), so
//!   append is idempotent instead of duplicating a page's data;
//! - the file's format is **detected from its content**, never from the
//!   extension, and a file written in another format is an error rather than a
//!   silent mixed document (§21.6).

use serde_json::{Map, Value};
use std::collections::BTreeMap;

/// One output format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Json,
    Yaml,
    Toml,
}

impl Format {
    /// Every name `format` accepts, in the order an error message lists them.
    pub const NAMES: [&'static str; 3] = ["json", "yaml", "toml"];

    /// The format named `name`, or `None` when the config misspelled it.
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "json" => Some(Self::Json),
            "yaml" => Some(Self::Yaml),
            "toml" => Some(Self::Toml),
            _ => None,
        }
    }

    /// The name the config used (and the one a diagnostic quotes).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Yaml => "yaml",
            Self::Toml => "toml",
        }
    }

    /// Serializes `rows` — the flat array of one element — for this format.
    ///
    /// `element` names the TOML array-of-tables key; JSON and YAML ignore it.
    pub fn render(self, element: &str, rows: &[Value]) -> Result<String, String> {
        self.render_document(&self.document(element, rows))
    }

    /// The text to write for `rows` when the target already holds `existed`.
    ///
    /// `None` (or a blank file) is a fresh write, exactly [`render`](Self::render).
    /// Anything else is read as a document of this format and the rows are
    /// **appended** to it — JSON and YAML keep one array, TOML keeps one array of
    /// tables per element, so a second element's block leaves the first one's
    /// behind.
    ///
    /// Appending is idempotent: a row the file already holds is not written
    /// twice, which is what makes a re-rendered page safe to run again.
    ///
    /// The format of the existing file is **detected from its content**
    /// ([`detect`](Self::detect)) and compared with `self`: writing TOML into a
    /// file that holds JSON is an error, never a mixed document.
    pub fn extend(
        self,
        element: &str,
        rows: &[Value],
        existed: Option<&str>,
    ) -> Result<String, String> {
        let Some(existed) = existed.filter(|text| !text.trim().is_empty()) else {
            return self.render(element, rows);
        };
        let Some(found) = Self::detect(existed) else {
            return Err(format!(
                "the file already exists and is not valid {} — delete it to rewrite it",
                Self::NAMES.join(", ")
            ));
        };
        if found != self {
            return Err(format!(
                "the file already holds {} data, but this block writes {} — one file, one format \
                 (delete the file to rewrite it)",
                found.as_str(),
                self.as_str()
            ));
        }
        let mut document = self.parse_document(existed)?;
        append_missing(self.rows_of(&mut document, element)?, rows);
        self.render_document(&document)
    }

    /// The format `text` is written in, or `None` when it is not a data document
    /// at all.
    ///
    /// Detection order is JSON, TOML, YAML, because a JSON document is also valid
    /// YAML and the stricter reading has to win: a `format = "yaml"` block may
    /// not quietly grow a `.json` file. The extension is never consulted (§21.6).
    pub fn detect(text: &str) -> Option<Self> {
        if serde_json::from_str::<Value>(text).is_ok() {
            return Some(Self::Json);
        }
        if toml::from_str::<toml::Value>(text).is_ok() {
            return Some(Self::Toml);
        }
        if serde_yaml::from_str::<Value>(text).is_ok() {
            return Some(Self::Yaml);
        }
        None
    }

    /// The whole document `element`'s rows live in: the array itself for JSON and
    /// YAML, one table keyed by the element for TOML.
    fn document(self, element: &str, rows: &[Value]) -> Value {
        match self {
            Self::Json | Self::Yaml => Value::Array(rows.to_vec()),
            Self::Toml => {
                let mut table = Map::new();
                table.insert(element.to_string(), Value::Array(rows.to_vec()));
                Value::Object(table)
            }
        }
    }

    /// Reads a document of this format.
    fn parse_document(self, text: &str) -> Result<Value, String> {
        match self {
            Self::Json => serde_json::from_str(text)
                .map_err(|error| format!("cannot read the existing json: {error}")),
            Self::Yaml => serde_yaml::from_str(text)
                .map_err(|error| format!("cannot read the existing yaml: {error}")),
            Self::Toml => toml::from_str(text)
                .map_err(|error| format!("cannot read the existing toml: {error}")),
        }
    }

    /// The rows inside a document of this format, ready to be appended to.
    ///
    /// A document of the right format but the wrong shape — an object where rows
    /// belong, a string under the element's TOML key — is an error: something
    /// else wrote that file, and appending to it would corrupt it.
    fn rows_of<'a>(
        self,
        document: &'a mut Value,
        element: &str,
    ) -> Result<&'a mut Vec<Value>, String> {
        match self {
            Self::Json | Self::Yaml => match document {
                Value::Array(rows) => Ok(rows),
                other => Err(format!(
                    "the file already holds {}, not an array of rows — delete it to rewrite it",
                    kind(other)
                )),
            },
            Self::Toml => {
                let Value::Object(table) = document else {
                    return Err(format!(
                        "the file already holds {}, not a table of rows — delete it to rewrite it",
                        kind(document)
                    ));
                };
                let entry = table
                    .entry(element.to_string())
                    .or_insert_with(|| Value::Array(Vec::new()));
                match entry {
                    Value::Array(rows) => Ok(rows),
                    other => Err(format!(
                        "the file already holds `{element}` as {}, not an array of tables — \
                         delete it to rewrite it",
                        kind(other)
                    )),
                }
            }
        }
    }

    /// Serializes a whole document of this format.
    fn render_document(self, document: &Value) -> Result<String, String> {
        match self {
            Self::Json => serde_json::to_string_pretty(document)
                .map(|text| format!("{text}\n"))
                .map_err(|error| error.to_string()),
            Self::Yaml => serde_yaml::to_string(document).map_err(|error| error.to_string()),
            Self::Toml => toml::to_string_pretty(document).map_err(|error| error.to_string()),
        }
    }
}

/// What a [`Value`] is, for the shape errors above.
fn kind(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "a boolean",
        Value::Number(_) => "a number",
        Value::String(_) => "a string",
        Value::Array(_) => "an array",
        Value::Object(_) => "an object",
    }
}

/// Appends the rows `existing` does not already hold, in their own order.
///
/// Equality is structural ([`canonical`]) and it is **counted**: a file holding a
/// row once and a block carrying it twice appends the second copy, so two
/// identical occurrences still produce two rows. That is what makes a re-run a
/// no-op without collapsing real duplicates.
fn append_missing(existing: &mut Vec<Value>, incoming: &[Value]) {
    let mut present: BTreeMap<String, usize> = BTreeMap::new();
    for row in existing.iter() {
        *present.entry(canonical(row)).or_insert(0) += 1;
    }
    for row in incoming {
        let key = canonical(row);
        match present.get_mut(&key) {
            Some(count) if *count > 0 => *count -= 1,
            _ => existing.push(row.clone()),
        }
    }
}

/// A row's comparison key: the same text for two structurally equal rows,
/// whatever order their keys were written in.
///
/// `serde_json` may keep insertion order instead of sorting, so the order is
/// imposed here rather than assumed.
fn canonical(value: &Value) -> String {
    match value {
        Value::Array(items) => {
            let parts: Vec<String> = items.iter().map(canonical).collect();
            format!("[{}]", parts.join(","))
        }
        Value::Object(map) => {
            // Keys are compared in sorted order, not insertion order.
            let sorted: BTreeMap<&String, &Value> = map.iter().collect();
            let parts: Vec<String> = sorted
                .into_iter()
                .map(|(key, value)| format!("{}:{}", Value::String(key.clone()), canonical(value)))
                .collect();
            format!("{{{}}}", parts.join(","))
        }
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows() -> Vec<Value> {
        vec![serde_json::json!({"citation_number": 1, "loc": "hlm. 45"})]
    }

    #[test]
    fn every_name_round_trips() {
        for name in Format::NAMES {
            let format = Format::parse(name).expect("a listed name parses");
            assert_eq!(format.as_str(), name);
        }
        assert_eq!(Format::parse("xml"), None);
    }

    #[test]
    fn json_and_yaml_write_the_flat_array() {
        let json = Format::Json.render("cite", &rows()).expect("json renders");
        assert!(json.trim_start().starts_with('['), "{json}");
        let yaml = Format::Yaml.render("cite", &rows()).expect("yaml renders");
        assert!(yaml.contains("citation_number: 1"), "{yaml}");
    }

    /// TOML has no top-level array, so the rows become an array of tables named
    /// after the element.
    #[test]
    fn toml_writes_an_array_of_tables_under_the_element_name() {
        let toml = Format::Toml.render("cite", &rows()).expect("toml renders");
        assert!(toml.starts_with("[[cite]]\n"), "{toml}");
        assert!(toml.contains("citation_number = 1"), "{toml}");
    }

    // --- a destination accumulates (§21.6) ------------------------------------

    fn row(number: u32) -> Value {
        serde_json::json!({ "citation_number": number })
    }

    /// The three formats agree on one thing: a second write **appends**.
    #[test]
    fn a_second_write_appends_instead_of_replacing() {
        for format in [Format::Json, Format::Yaml, Format::Toml] {
            let first = format.render("cite", &[row(1)]).expect("first render");
            let merged = format
                .extend("cite", &[row(2)], Some(&first))
                .unwrap_or_else(|error| panic!("{}: {error}", format.as_str()));
            let mut document = format
                .parse_document(&merged)
                .unwrap_or_else(|error| panic!("{}: {error}", format.as_str()));
            assert_eq!(
                format
                    .rows_of(&mut document, "cite")
                    .expect("the rows of a written document"),
                &vec![row(1), row(2)],
                "{}: {merged}",
                format.as_str()
            );
        }
    }

    /// The point of appending: a page that renders **again** must not duplicate
    /// its data, so the second write of the same rows is byte-identical.
    #[test]
    fn writing_the_same_rows_twice_is_a_no_op() {
        for format in [Format::Json, Format::Yaml, Format::Toml] {
            let first = format
                .render("cite", &[row(1), row(2)])
                .expect("first render");
            let again = format
                .extend("cite", &[row(1), row(2)], Some(&first))
                .expect("the rows are already there");
            assert_eq!(again, first, "{}: a re-run moved a byte", format.as_str());
        }
    }

    /// Equality is counted, not a set: a file holding a row once and a block
    /// carrying it twice appends exactly one copy, so two identical occurrences
    /// stay two rows.
    #[test]
    fn a_row_the_file_holds_once_is_appended_once_more() {
        let first = Format::Json
            .render("cite", &[row(1)])
            .expect("first render");
        let merged = Format::Json
            .extend("cite", &[row(1), row(1)], Some(&first))
            .expect("append");
        let mut document = Format::Json.parse_document(&merged).expect("parses");
        assert_eq!(
            Format::Json
                .rows_of(&mut document, "cite")
                .expect("rows")
                .len(),
            2,
            "{merged}"
        );
    }

    /// Two elements in one TOML file are two arrays of tables, so a `cite` block
    /// appending to a file a `heading` block wrote keeps the headings.
    #[test]
    fn toml_keeps_the_other_elements_tables() {
        let heading = Format::Toml
            .render("heading", &[serde_json::json!({ "id": "intro" })])
            .expect("heading renders");
        let merged = Format::Toml
            .extend("cite", &[row(1)], Some(&heading))
            .expect("append");
        assert!(merged.contains("[[heading]]"), "{merged}");
        assert!(merged.contains("[[cite]]"), "{merged}");
    }

    /// The file's format comes from its content, so a block may not pour TOML
    /// into a JSON file (or YAML into a JSON one): the error names both sides.
    #[test]
    fn a_file_of_another_format_is_an_error() {
        let yaml = Format::Yaml.render("cite", &[row(1)]).expect("yaml");
        let error = Format::Json
            .extend("cite", &[row(2)], Some(&yaml))
            .expect_err("json rows into a yaml file");
        assert!(error.contains("holds yaml data"), "{error}");
        assert!(error.contains("writes json"), "{error}");

        let json = Format::Json.render("cite", &[row(1)]).expect("json");
        let error = Format::Yaml
            .extend("cite", &[row(2)], Some(&json))
            .expect_err("yaml rows into a json file");
        assert!(error.contains("holds json data"), "{error}");

        let toml = Format::Toml.render("cite", &[row(1)]).expect("toml");
        let error = Format::Json
            .extend("cite", &[row(2)], Some(&toml))
            .expect_err("json rows into a toml file");
        assert!(error.contains("holds toml data"), "{error}");
    }

    /// Right format, wrong shape: something else wrote that file, and appending
    /// to it would corrupt it rather than extend it.
    #[test]
    fn a_document_of_the_wrong_shape_is_an_error() {
        let error = Format::Json
            .extend("cite", &[row(1)], Some("{\n  \"cite\": []\n}"))
            .expect_err("an object is not a row array");
        assert!(error.contains("an object, not an array of rows"), "{error}");

        let error = Format::Toml
            .extend("cite", &[row(1)], Some("[cite]\nid = \"x\"\n"))
            .expect_err("a table is not an array of tables");
        assert!(
            error.contains("`cite` as an object, not an array of tables"),
            "{error}"
        );
    }

    /// A file that is none of the three is not adopted either, and the message
    /// says how to start over.
    #[test]
    fn a_file_that_is_no_data_document_is_an_error() {
        let error = Format::Json
            .extend("cite", &[row(1)], Some("{ unclosed\n"))
            .expect_err("not json, yaml or toml");
        assert!(error.contains("is not valid json, yaml, toml"), "{error}");
    }

    /// Detection reads the content, never the extension, and a blank file is a
    /// fresh write.
    #[test]
    fn detection_reads_the_content_and_a_blank_file_is_fresh() {
        let json = Format::Json.render("cite", &[row(1)]).expect("json");
        let yaml = Format::Yaml.render("cite", &[row(1)]).expect("yaml");
        let toml = Format::Toml.render("cite", &[row(1)]).expect("toml");
        assert_eq!(Format::detect(&json), Some(Format::Json));
        assert_eq!(Format::detect(&yaml), Some(Format::Yaml));
        assert_eq!(Format::detect(&toml), Some(Format::Toml));
        assert_eq!(Format::detect("{ unclosed"), None);

        assert_eq!(
            Format::Json
                .extend("cite", &[row(1)], Some("\n"))
                .expect("a blank file is a fresh write"),
            json
        );
    }
}
