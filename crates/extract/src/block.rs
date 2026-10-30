//! One extraction block: what it selects, where it writes, which fields it keeps.
//!
//! The block is the unit of configuration (`[[task.<element>.extract]]`) and the
//! unit `pendon-types` resolves: every destination it names is one rule target,
//! so two blocks naming one path are one target (the later block's rows are the
//! ones appended to that file, D-20) while two blocks writing different paths
//! both run.

use pendon_extra::Path;
use pendon_types::Selector;

use crate::format::Format;
use crate::select::Selection;

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

/// Which field list a block starts from (D-6).
///
/// The **preset** is a starting point, not a filter: [`Pendon`](Self::Pendon) is
/// the element's documented list
/// ([`docs/spec/FIELDS-INVENTORY.md`](../../../docs/spec/FIELDS-INVENTORY.md)
/// §12), `include` adds names to it, `only` keeps nothing else and `omit`
/// subtracts last. `All` starts from every field the occurrence carries, which
/// is what a block with no `preset` did before Stage 2.
///
/// `Default` is [`All`](Self::All) until `S2-8.6` flips it to `Pendon` (the
/// order is deliberate: the flip is one line plus a golden re-freeze, not a
/// behaviour change hidden inside a refactor).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Preset {
    /// `preset = "pendon"` — the element's own fields, and nothing an author
    /// added through extras, a modifier run or a custom assignment.
    Pendon,
    /// `preset = "all"` — every field the occurrence carries, internals aside.
    #[default]
    All,
}

impl Preset {
    /// The two preset names, in the order a diagnostic lists them.
    pub const NAMES: [&'static str; 2] = ["pendon", "all"];

    /// The preset a config value names, or `None` when it names neither.
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "pendon" => Some(Self::Pendon),
            "all" => Some(Self::All),
            _ => None,
        }
    }

    /// The value as it is written in a config.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pendon => "pendon",
            Self::All => "all",
        }
    }

    /// The fields the preset starts from: `Some(list)` for [`Pendon`](Self::Pendon),
    /// `None` for [`All`](Self::All) — "no restriction", not "no fields".
    pub fn fields(self, element: &str) -> Option<&'static [&'static str]> {
        match self {
            Self::Pendon => Some(crate::fields::pendon_preset(element)),
            Self::All => None,
        }
    }
}

/// The raw block as the task table carries it — every value still a string.
#[derive(Clone, Debug, Default)]
pub struct BlockSpec {
    pub element: String,
    pub types: Vec<String>,
    pub output: Vec<String>,
    pub format: Vec<String>,
    /// `preset` as authored; absent is [`Preset::default`].
    pub preset: Option<String>,
    pub include: Vec<String>,
    pub only: Vec<String>,
    pub omit: Vec<String>,
    pub export: Option<String>,
}

/// A validated block.
#[derive(Clone, Debug)]
pub struct ExtractBlock {
    pub element: &'static str,
    pub selectors: Vec<Selector>,
    pub outputs: Vec<(String, Format)>,
    pub preset: Preset,
    /// `include`, `only` and `omit` as **paths**: `price.details.amount` is
    /// three segments, and a prefix selects the whole subtree (§21 rule 4).
    pub include: Vec<Path>,
    pub only: Vec<Path>,
    pub omit: Vec<Path>,
    pub export: Option<String>,
}

impl ExtractBlock {
    /// Validates one `[[task.<element>.extract]]` entry.
    ///
    /// Errors name the element and the entry index (`task.cite.extract[0].format`)
    /// so a multi-block task says which block is wrong.
    pub fn build(spec: BlockSpec, index: usize) -> Result<Self, ExtractError> {
        let key = |suffix: String| format!("task.{}.extract[{index}].{suffix}", spec.element);
        let element = crate::fields::any_element(&spec.element).ok_or_else(|| {
            ExtractError::new(
                key("element".to_string()),
                format!(
                    "`{}` has no extractable node (docs/spec/FIELDS-INVENTORY.md §1)",
                    spec.element
                ),
            )
        })?;

        let selectors = Selector::parse_all(spec.types.iter().map(String::as_str))
            .map_err(|error| ExtractError::new(key("type".to_string()), error.to_string()))?;

        // D-14: frontmatter is one record per document and has **no** preset — its
        // fields are whatever the author wrote, so `only` / `omit` are the whole
        // selection. A `preset` (even `"all"`), an `include` or a `type` would be
        // a key that silently does nothing, which is exactly what D-16 forbids.
        let frontmatter = element == crate::fields::FRONTMATTER;
        if frontmatter {
            if spec.preset.is_some() || !spec.include.is_empty() {
                return Err(ExtractError::new(
                    key("preset".to_string()),
                    "frontmatter has no preset; use only/omit",
                ));
            }
            if !spec.types.is_empty() {
                return Err(ExtractError::new(
                    key("type".to_string()),
                    "frontmatter is one record per document, so it has no `type` to select",
                ));
            }
            if spec.export.as_deref() == Some(crate::fields::FRONTMATTER) {
                return Err(ExtractError::new(
                    key("export".to_string()),
                    "`export = \"frontmatter\"` is already defined by the renderer (§20); pick another name",
                ));
            }
        }

        // D-6: the vocabulary is closed — a typo in `preset` is an error naming
        // both values, never a silent fall back to the default.
        let preset = match spec.preset.as_deref() {
            // A frontmatter block always starts from every key the author wrote:
            // `All` here is "no restriction", and it is also what keeps the
            // record untouched when the default preset flips (S2-8.6).
            None if frontmatter => Preset::All,
            None => Preset::default(),
            Some(name) => Preset::parse(name).ok_or_else(|| {
                ExtractError::new(
                    key("preset".to_string()),
                    format!(
                        "unknown preset `{name}`; expected one of {}",
                        Preset::NAMES.join(", ")
                    ),
                )
            })?,
        };

        // D-6: `include` adds to the preset and `only` keeps nothing but its own
        // names, so together they are one command with two readings — an error
        // rather than a precedence rule nobody would remember.
        if !spec.only.is_empty() && !spec.include.is_empty() {
            return Err(ExtractError::new(
                key("only".to_string()),
                "`only` cannot be combined with `include`: `include` adds fields to the preset, `only` keeps nothing else (use `include` with `omit`, or `only` alone)",
            ));
        }

        let include = paths(&spec.include, &key("include".to_string()))?;
        let only = paths(&spec.only, &key("only".to_string()))?;
        let omit = paths(&spec.omit, &key("omit".to_string()))?;

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
            preset,
            include,
            only,
            omit,
            export: spec.export,
        })
    }

    /// Every destination the block writes, in config order.
    pub fn destination_count(&self) -> usize {
        self.outputs.len() + usize::from(self.export.is_some())
    }

    /// The field algebra this block's four keys describe (§21 rule 4).
    pub fn selection(&self) -> Selection {
        Selection::new(self)
    }
}

/// Parses the `include` / `only` / `omit` entries into field paths.
///
/// §21 rule 4: each entry is a dotted public field path, so a typo such as
/// `price..amount` or `1x` is a load error naming the entry — never a filter
/// that silently matches nothing (D-16).
fn paths(names: &[String], key: &str) -> Result<Vec<Path>, ExtractError> {
    names
        .iter()
        .enumerate()
        .map(|(slot, name)| {
            Path::from_dotted(name).map_err(|error| {
                ExtractError::new(
                    format!("{key}[{slot}]"),
                    format!("`{name}` is not a field path: {error}"),
                )
            })
        })
        .collect()
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

    /// A spec that reaches the field-selection checks: one destination, so the
    /// block passes the "needs a destination" gate first.
    fn written(element: &str) -> BlockSpec {
        BlockSpec {
            output: vec!["out/x.json".to_string()],
            format: vec!["json".to_string()],
            ..spec(element)
        }
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
    /// rather than silently extracting nothing (FIELDS-INVENTORY §1).
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
        assert!(message.contains("FIELDS-INVENTORY"), "{message}");
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

    fn block(overrides: BlockSpec) -> ExtractBlock {
        ExtractBlock::build(overrides, 0).expect("a valid block")
    }

    /// D-6: the preset is a closed vocabulary. `pendon` is the element's own list
    /// and `all` is "no restriction", which is also what an absent key means until
    /// `S2-8.6` flips the default.
    #[test]
    fn a_preset_name_is_parsed_and_absent_is_the_default() {
        assert_eq!(
            block(written("cite")).preset,
            Preset::All,
            "an absent `preset` is the default"
        );
        assert_eq!(
            block(BlockSpec {
                preset: Some("pendon".to_string()),
                ..written("cite")
            })
            .preset,
            Preset::Pendon
        );
        assert_eq!(
            block(BlockSpec {
                preset: Some("all".to_string()),
                ..written("cite")
            })
            .preset,
            Preset::All
        );
        // `All` is the "no restriction" arm, not an empty list.
        assert_eq!(Preset::All.fields("cite"), None);
        assert_eq!(
            Preset::Pendon.fields("cite"),
            Some(crate::fields::pendon_preset("cite"))
        );
        assert_eq!(Preset::NAMES, ["pendon", "all"]);
        assert_eq!(Preset::All.as_str(), "all");
        assert_eq!(Preset::Pendon.as_str(), "pendon");
    }

    /// D-6: a typo is an error naming both values — never a silent fall back to
    /// the default preset (which would quietly change what a file holds).
    #[test]
    fn an_unknown_preset_lists_the_names() {
        let message = error(
            BlockSpec {
                preset: Some("pendon-all".to_string()),
                ..written("cite")
            },
            0,
        );
        assert!(
            message.starts_with("task.cite.extract[0].preset: "),
            "{message}"
        );
        assert!(message.contains("unknown preset `pendon-all`"), "{message}");
        assert!(message.contains("pendon, all"), "{message}");
    }

    /// D-6: `include` adds to the preset, `only` keeps nothing else — one block
    /// cannot mean both, so the pair is a config error.
    #[test]
    fn only_and_include_cannot_be_combined() {
        let message = error(
            BlockSpec {
                include: vec!["class".to_string()],
                only: vec!["href".to_string()],
                ..written("cite")
            },
            0,
        );
        assert!(
            message.starts_with("task.cite.extract[0].only: "),
            "{message}"
        );
        assert!(
            message.contains("cannot be combined with `include`"),
            "{message}"
        );
    }

    /// The dotted spelling of a path list, for assertions.
    fn dotted(paths: &[Path]) -> Vec<String> {
        paths.iter().map(Path::dotted).collect()
    }

    /// D-6: the allowed pairs. `omit` may follow either narrow key, and a preset
    /// may be narrowed by `only` alone (that is PE.4's `[{}]` case).
    #[test]
    fn include_and_only_are_each_allowed_with_omit_and_a_preset() {
        let included = block(BlockSpec {
            preset: Some("pendon".to_string()),
            include: vec!["class".to_string()],
            omit: vec!["title".to_string()],
            ..written("cite")
        });
        assert_eq!(included.preset, Preset::Pendon);
        assert_eq!(dotted(&included.include), ["class"]);
        assert_eq!(dotted(&included.omit), ["title"]);
        assert!(included.only.is_empty());

        let exclusive = block(BlockSpec {
            preset: Some("pendon".to_string()),
            only: vec!["href".to_string()],
            omit: vec!["href".to_string()],
            ..written("cite")
        });
        assert_eq!(dotted(&exclusive.only), ["href"]);
        assert_eq!(dotted(&exclusive.omit), ["href"]);
        assert!(exclusive.include.is_empty());
    }

    /// §21 rule 4: every selection entry is a field path, and a malformed one is
    /// a load error naming the entry — not a filter that silently matches
    /// nothing (D-16). The dotted names a `field.*` key allows are the rule.
    #[test]
    fn a_selection_entry_must_be_a_field_path() {
        let message = error(
            BlockSpec {
                only: vec!["price".to_string(), "price..amount".to_string()],
                ..written("anchor")
            },
            2,
        );
        assert!(
            message.starts_with("task.anchor.extract[2].only[1]: "),
            "{message}"
        );
        assert!(message.contains("`price..amount`"), "{message}");
        assert!(message.contains("segment 2 is empty"), "{message}");

        let message = error(
            BlockSpec {
                omit: vec!["1x".to_string()],
                ..written("anchor")
            },
            0,
        );
        assert!(
            message.starts_with("task.anchor.extract[0].omit[0]: "),
            "{message}"
        );
        assert!(message.contains("starts with `1`"), "{message}");
    }

    /// A dotted entry keeps its segments, so the selection can reach a subtree.
    #[test]
    fn a_dotted_selection_entry_keeps_its_segments() {
        let block = block(BlockSpec {
            include: vec!["price.details.amount".to_string()],
            ..written("anchor")
        });
        assert_eq!(dotted(&block.include), ["price.details.amount"]);
        assert_eq!(block.include[0].segments().len(), 3);
        assert_eq!(block.include[0].segments()[2], "amount");
    }

    /// D-14: frontmatter takes `only` / `omit` and the usual destinations, and
    /// no preset — its starting set is every key the author wrote.
    #[test]
    fn a_frontmatter_block_takes_only_and_omit_but_no_preset() {
        let block = block(BlockSpec {
            element: "frontmatter".to_string(),
            output: vec!["out/fm.json".to_string()],
            format: vec!["json".to_string()],
            only: vec!["title".to_string(), "meta.tags".to_string()],
            export: Some("meta".to_string()),
            ..BlockSpec::default()
        });
        assert_eq!(block.element, crate::fields::FRONTMATTER);
        // No preset key is a starting set of *everything* — and it stays that
        // way when the default preset flips to `pendon` (S2-8.6).
        assert_eq!(block.preset, Preset::All);
        assert_eq!(dotted(&block.only), ["title", "meta.tags"]);
        assert_eq!(block.export.as_deref(), Some("meta"));
    }

    /// D-14: `preset`, `include` and `type` are errors for frontmatter, and so is
    /// exporting the name §20 already owns.
    #[test]
    fn a_frontmatter_block_rejects_what_it_cannot_mean() {
        let frontmatter = || BlockSpec {
            element: "frontmatter".to_string(),
            output: vec!["out/fm.json".to_string()],
            format: vec!["json".to_string()],
            ..BlockSpec::default()
        };

        let message = error(
            BlockSpec {
                preset: Some("all".to_string()),
                ..frontmatter()
            },
            0,
        );
        assert!(
            message.starts_with("task.frontmatter.extract[0].preset: "),
            "{message}"
        );
        assert!(message.contains("frontmatter has no preset"), "{message}");

        let message = error(
            BlockSpec {
                include: vec!["title".to_string()],
                ..frontmatter()
            },
            0,
        );
        assert!(message.contains("frontmatter has no preset"), "{message}");

        let message = error(
            BlockSpec {
                types: vec!["FmA".to_string()],
                ..frontmatter()
            },
            0,
        );
        assert!(
            message.starts_with("task.frontmatter.extract[0].type: "),
            "{message}"
        );
        assert!(message.contains("no `type` to select"), "{message}");

        let message = error(
            BlockSpec {
                output: Vec::new(),
                format: Vec::new(),
                export: Some("frontmatter".to_string()),
                ..frontmatter()
            },
            0,
        );
        assert!(
            message.starts_with("task.frontmatter.extract[0].export: "),
            "{message}"
        );
        assert!(message.contains("already defined"), "{message}");
    }
}
