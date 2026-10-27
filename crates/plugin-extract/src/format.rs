//! The three destination formats.
//!
//! P.1 of [`docs/issues/plugin-extract.md`](../../../docs/issues/plugin-extract.md)
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

use serde_json::Value;

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
        let value = Value::Array(rows.to_vec());
        match self {
            Self::Json => serde_json::to_string_pretty(&value)
                .map(|text| format!("{text}\n"))
                .map_err(|error| error.to_string()),
            Self::Yaml => serde_yaml::to_string(&value).map_err(|error| error.to_string()),
            Self::Toml => {
                let mut table = serde_json::Map::new();
                table.insert(element.to_string(), value);
                toml::to_string_pretty(&Value::Object(table)).map_err(|error| error.to_string())
            }
        }
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
}
