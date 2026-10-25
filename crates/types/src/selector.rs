//! One entry of a config `type = …` list.
//!
//! `type = "TypeA"` and `type = ["TypeA"]` are the same single-selector list
//! (§11 rule 2); `?`, `_` and `*` are the three wildcards. Everything else is a
//! literal marker name.

use std::fmt;

use serde::{Deserialize, Serialize};

/// A parse failure for a `type` entry. Callers turn it into their own
/// configuration error (the CLI's `ConfigError`, naming layer and key).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectorError {
    token: String,
    reason: &'static str,
}

impl SelectorError {
    /// The offending token, exactly as it appeared after trimming.
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Why it was rejected, phrased to slot into a config error message.
    pub fn reason(&self) -> &'static str {
        self.reason
    }
}

impl fmt::Display for SelectorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`{}` {}", self.token, self.reason)
    }
}

impl std::error::Error for SelectorError {}

/// One `type` entry (§11 selectors).
///
/// Identity is the exact string: case-sensitive, no normalization.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Selector {
    /// A literal marker name (`TypeA`). Matches exactly that type.
    Exact(String),
    /// `?` — the untyped element (no `@@type{…}` marker).
    Question,
    /// `_` — every occurring type no rule in the group named.
    Underscore,
    /// `*` — every occurring type, including the named ones and `untyped`.
    Star,
}

impl Selector {
    /// Parses one config `type` entry: trimmed, non-empty, and either a
    /// wildcard or a literal marker name.
    ///
    /// `??`, `**` and any other all-wildcard run are rejected: they are not
    /// escapes and no element marker can ever be spelled that way, so
    /// accepting them would create a selector that silently never matches.
    pub fn parse(token: &str) -> Result<Self, SelectorError> {
        let token = token.trim();
        if token.is_empty() {
            return Err(SelectorError {
                token: token.to_string(),
                reason: "is not a selector: an empty `type` entry is not allowed",
            });
        }
        let selector = match token {
            "?" => Self::Question,
            "_" => Self::Underscore,
            "*" => Self::Star,
            _ => {
                if token.chars().all(|c| matches!(c, '?' | '_' | '*')) {
                    return Err(SelectorError {
                        token: token.to_string(),
                        reason: "is not a selector: `?`, `_` and `*` stand alone \
                                 and are not escapes",
                    });
                }
                if token.chars().any(char::is_whitespace) {
                    return Err(SelectorError {
                        token: token.to_string(),
                        reason: "is not a type marker: it contains whitespace",
                    });
                }
                Self::Exact(token.to_string())
            }
        };
        Ok(selector)
    }

    /// Parses a config value that already accepts a string or an array.
    ///
    /// `[]` (no tokens) means "no `type` key" (§11 rule 2) and stays the layer
    /// default. An explicit empty entry (`type = [""]`) or any invalid selector
    /// is an error, so a typo never lands as a wildcard.
    pub fn parse_all<I, S>(tokens: I) -> Result<Vec<Self>, SelectorError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let tokens: Vec<S> = tokens.into_iter().collect();
        tokens
            .iter()
            .map(|token| Self::parse(token.as_ref()))
            .collect()
    }

    /// Selector specificity (§2.1): literal / `?` = 2, `_` / `*` = 1.
    pub fn specificity(&self) -> u8 {
        match self {
            Self::Exact(_) | Self::Question => 2,
            Self::Underscore | Self::Star => 1,
        }
    }

    /// `true` when this selector can be answered without the document's types.
    /// Positional keys (§6.1) are literal-only, so they use this.
    pub fn is_literal(&self) -> bool {
        matches!(self, Self::Exact(_))
    }

    /// The literal marker name, when this is an [`Exact`](Self::Exact).
    pub fn literal(&self) -> Option<&str> {
        match self {
            Self::Exact(name) => Some(name),
            _ => None,
        }
    }
}

impl fmt::Display for Selector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exact(name) => f.write_str(name),
            Self::Question => f.write_str("?"),
            Self::Underscore => f.write_str("_"),
            Self::Star => f.write_str("*"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(source: &str) {
        let selector = Selector::parse(source).expect(source);
        assert_eq!(selector.to_string(), source);
    }

    #[test]
    fn selector_round_trips_every_form() {
        round_trip("?");
        round_trip("_");
        round_trip("*");
        round_trip("TypeA");
        round_trip("heading:auto");
    }

    #[test]
    fn selector_trims_whitespace() {
        assert_eq!(
            Selector::parse("  TypeA \n").unwrap(),
            Selector::Exact("TypeA".into())
        );
        assert_eq!(Selector::parse(" ?").unwrap(), Selector::Question);
    }

    #[test]
    fn selector_rejects_empty_tokens() {
        for token in ["", "   ", "\t"] {
            assert!(Selector::parse(token).is_err(), "{token:?} accepted");
        }
        // `[]` is "no `type` key" (the default slot); an explicit empty entry
        // is an error, so `type = [""]` cannot silently become a wildcard.
        let none: [&str; 0] = [];
        assert!(Selector::parse_all(none).unwrap().is_empty());
        assert!(Selector::parse_all([""]).is_err());
    }

    #[test]
    fn selector_rejects_repeated_wildcards() {
        for token in ["??", "**", "__", "?*_"] {
            assert!(Selector::parse(token).is_err(), "{token:?} accepted");
        }
        // Trimming happens first, so `" _ "` is the ordinary `_` wildcard.
        assert_eq!(Selector::parse(" _ ").unwrap(), Selector::Underscore);
    }

    #[test]
    fn selector_accepts_names_that_merely_start_with_a_wildcard() {
        assert_eq!(
            Selector::parse("_private").unwrap(),
            Selector::Exact("_private".into())
        );
        assert_eq!(
            Selector::parse("Type*").unwrap(),
            Selector::Exact("Type*".into())
        );
    }

    #[test]
    fn selector_specificity_is_per_selector() {
        assert_eq!(Selector::Exact("A".into()).specificity(), 2);
        assert_eq!(Selector::Question.specificity(), 2);
        assert_eq!(Selector::Underscore.specificity(), 1);
        assert_eq!(Selector::Star.specificity(), 1);
    }
}
