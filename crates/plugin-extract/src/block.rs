//! One extraction block: what it selects, where it writes, which fields it keeps.
//!
//! The block is the unit of configuration (`[[task.<element>.extract]]`) and the
//! unit `pendon-types` resolves: every destination it names is one rule target,
//! so two blocks writing the same path override each other (P.1.c) while two
//! blocks writing different paths both run.

use pendon_types::Selector;

use crate::format::Format;

/// A config error: the key that is wrong and what is wrong with it.
///
/// `key` is the dotted path the user wrote (`task.cite.extract[0].format`), so a
/// diagnostic points at the line to fix.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtractError {
    pub key: String,
    pub message: String,
}

impl ExtractError {
    pub fn new(key: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ExtractError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.key, self.message)
    }
}

impl std::error::Error for ExtractError {}

/// The raw block as the task table carries it — every value still a string.
#[derive(Clone, Debug, Default)]
pub struct BlockSpec {
    pub element: String,
    pub types: Vec<String>,
    pub output: Vec<String>,
    pub format: Vec<String>,
    pub only: Vec<String>,
    pub omit: Vec<String>,
    pub recommended: bool,
    pub export: Option<String>,
}

/// A validated block.
#[derive(Clone, Debug)]
pub struct ExtractBlock {
    pub element: &'static str,
    pub selectors: Vec<Selector>,
    pub outputs: Vec<(String, Format)>,
    pub only: Vec<String>,
    pub omit: Vec<String>,
    pub recommended: bool,
    pub export: Option<String>,
}

impl ExtractBlock {
    /// Validates one `[[task.<element>.extract]]` entry.
    ///
    /// Errors name the element and the entry index (`task.cite.extract[0].format`)
    /// so a multi-block task says which block is wrong.
    pub fn build(spec: BlockSpec, index: usize) -> Result<Self, ExtractError> {
        let key = |suffix: String| format!("task.{}.extract[{index}].{suffix}", spec.element);
        let element = crate::fields::element(&spec.element).ok_or_else(|| {
            ExtractError::new(
                key("element".to_string()),
                format!(
                    "`{}` has no extractable node (docs/spec/EXTRAS-INVENTORY.md §1)",
                    spec.element
                ),
            )
        })?;

        let selectors = Selector::parse_all(spec.types.iter().map(String::as_str))
            .map_err(|error| ExtractError::new(key("type".to_string()), error.to_string()))?;

        if spec.output.is_empty() && spec.export.is_none() {
            return Err(ExtractError::new(
                key("output".to_string()),
                "a block needs `output`, `export`, or both",
            ));
        }
        if !spec.output.is_empty() && spec.format.is_empty() {
            return Err(ExtractError::new(
                key("format".to_string()),
                "`format` is required when `output` is set",
            ));
        }
        if spec.format.len() != spec.output.len() {
            return Err(ExtractError::new(
                key("format".to_string()),
                format!(
                    "{} format(s) for {} output(s): give exactly one format per output",
                    spec.format.len(),
                    spec.output.len()
                ),
            ));
        }

        let mut outputs = Vec::with_capacity(spec.output.len());
        for (slot, path) in spec.output.iter().enumerate() {
            let name = spec.format[slot].as_str();
            let format = Format::parse(name).ok_or_else(|| {
                ExtractError::new(
                    format!("task.{}.extract[{index}].format[{slot}]", spec.element),
                    format!(
                        "unknown format `{name}`; expected one of {}",
                        Format::NAMES.join(", ")
                    ),
                )
            })?;
            if path.trim().is_empty() {
                return Err(ExtractError::new(
                    format!("task.{}.extract[{index}].output[{slot}]", spec.element),
                    "an output path cannot be empty",
                ));
            }
            outputs.push((path.clone(), format));
        }

        Ok(Self {
            element,
            selectors,
            outputs,
            only: spec.only,
            omit: spec.omit,
            recommended: spec.recommended,
            export: spec.export,
        })
    }

    /// Every destination the block writes, in config order.
    pub fn destination_count(&self) -> usize {
        self.outputs.len() + usize::from(self.export.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(element: &str) -> BlockSpec {
        BlockSpec {
            element: element.to_string(),
            ..BlockSpec::default()
        }
    }

    fn error(spec: BlockSpec, index: usize) -> String {
        ExtractBlock::build(spec, index)
            .expect_err("the spec is invalid")
            .to_string()
    }

    #[test]
    fn a_block_needs_a_destination() {
        let message = error(spec("cite"), 0);
        assert!(
            message.starts_with("task.cite.extract[0].output: "),
            "{message}"
        );
        assert!(message.contains("`output`, `export`, or both"), "{message}");
    }

    #[test]
    fn a_format_is_required_with_an_output() {
        let message = error(
            BlockSpec {
                output: vec!["out/c.json".to_string()],
                ..spec("cite")
            },
            1,
        );
        assert!(
            message.starts_with("task.cite.extract[1].format: "),
            "{message}"
        );
        assert!(message.contains("required"), "{message}");
    }

    /// One format per output, and the message says which block of which element.
    #[test]
    fn a_format_count_must_match_the_output_count() {
        let message = error(
            BlockSpec {
                output: vec!["out/a.json".to_string(), "out/b.json".to_string()],
                format: vec!["json".to_string()],
                ..spec("cite")
            },
            0,
        );
        assert!(
            message.starts_with("task.cite.extract[0].format: "),
            "{message}"
        );
        assert!(message.contains("1 format(s) for 2 output(s)"), "{message}");
    }

    #[test]
    fn an_unknown_format_lists_the_names() {
        let message = error(
            BlockSpec {
                output: vec!["out/a.xml".to_string()],
                format: vec!["xml".to_string()],
                ..spec("cite")
            },
            0,
        );
        assert!(
            message.starts_with("task.cite.extract[0].format[0]: "),
            "{message}"
        );
        assert!(message.contains("json, yaml, toml"), "{message}");
    }

    #[test]
    fn an_empty_path_is_rejected() {
        let message = error(
            BlockSpec {
                output: vec!["  ".to_string()],
                format: vec!["json".to_string()],
                ..spec("cite")
            },
            0,
        );
        assert!(
            message.starts_with("task.cite.extract[0].output[0]: "),
            "{message}"
        );
    }

    /// `wiki` is listed by §8.6 but has no layer tag yet: the config is rejected
    /// rather than silently extracting nothing (EXTRAS-INVENTORY §1).
    #[test]
    fn an_element_without_a_layer_is_rejected() {
        let message = error(
            BlockSpec {
                export: Some("links".to_string()),
                ..spec("wiki")
            },
            0,
        );
        assert!(
            message.starts_with("task.wiki.extract[0].element: "),
            "{message}"
        );
        assert!(message.contains("EXTRAS-INVENTORY"), "{message}");
    }

    #[test]
    fn a_bad_type_selector_names_the_key() {
        let message = error(
            BlockSpec {
                types: vec![String::new()],
                export: Some("rows".to_string()),
                ..spec("cite")
            },
            0,
        );
        assert!(
            message.starts_with("task.cite.extract[0].type: "),
            "{message}"
        );
    }

    /// Every destination is one rule target, so both a file and an export count.
    #[test]
    fn destinations_are_counted() {
        let block = ExtractBlock::build(
            BlockSpec {
                output: vec!["out/a.json".to_string(), "out/b.yaml".to_string()],
                format: vec!["json".to_string(), "yaml".to_string()],
                export: Some("cites".to_string()),
                ..spec("cite")
            },
            0,
        )
        .expect("a valid block");
        assert_eq!(block.destination_count(), 3);
        assert_eq!(block.outputs[1].1, Format::Yaml);
    }
}
