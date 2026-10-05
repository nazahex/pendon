//! Attribute values produced by the typed extras head
//! (`docs/spec/SYNTAX.md` §6.3).

/// A single parsed attribute value, exactly as declared in spec §6.3.
///
/// Numbers are parsed as numbers so a custom component receives `12` rather
/// than `"12"`, but they render back into literal text (`literal`) so golden
/// fixtures stay stable.
#[derive(Debug, Clone, PartialEq)]
pub enum AttrValue {
    /// A quoted (or backticked) value.
    Str(String),
    /// An unquoted integer.
    Int(i64),
    /// An unquoted decimal.
    Float(f64),
    /// An unquoted `true` / `false`.
    Bool(bool),
    /// Any other unquoted text.
    Raw(String),
}

impl AttrValue {
    /// Literal text used by `Event::Attribute`, by interpolations
    /// (`{attrs.key}`) and by the stringly-typed DOM fallback path.
    pub fn literal(&self) -> String {
        match self {
            Self::Str(text) | Self::Raw(text) => text.clone(),
            Self::Int(number) => number.to_string(),
            Self::Float(number) => float_literal(*number),
            Self::Bool(true) => "true".to_string(),
            Self::Bool(false) => "false".to_string(),
        }
    }
}

/// Formats a float so that an authored `6.0` does not collapse into `6`.
fn float_literal(number: f64) -> String {
    let text = number.to_string();
    if text.contains('.') || text.contains('e') || text.contains('E') {
        text
    } else {
        format!("{text}.0")
    }
}

/// Classifies an unquoted scalar the way spec §5 does:
/// `true`/`false` → `Bool`, integer → `Int`, decimal → `Float`, else `Raw`.
pub fn classify_scalar(text: &str) -> AttrValue {
    match text {
        "true" => return AttrValue::Bool(true),
        "false" => return AttrValue::Bool(false),
        _ => {}
    }
    match number_kind(text) {
        Some(NumberKind::Integer) => match text.parse::<i64>() {
            // Out of range numbers keep their literal form instead of wrapping.
            Err(_) => AttrValue::Raw(text.to_string()),
            Ok(number) => AttrValue::Int(number),
        },
        Some(NumberKind::Float) => match text.parse::<f64>() {
            Err(_) => AttrValue::Raw(text.to_string()),
            Ok(number) => AttrValue::Float(number),
        },
        None => AttrValue::Raw(text.to_string()),
    }
}

/// Whether an unquoted scalar looks like an integer or a decimal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberKind {
    Integer,
    Float,
}

/// `-?DIGIT+(.DIGIT+)?` — the only shapes treated as numbers.
pub fn number_kind(text: &str) -> Option<NumberKind> {
    let digits = text.strip_prefix('-').unwrap_or(text);
    if digits.is_empty() {
        return None;
    }
    let (whole, fraction) = match digits.split_once('.') {
        Some((whole, fraction)) => (whole, Some(fraction)),
        None => (digits, None),
    };
    if !whole.chars().all(|character| character.is_ascii_digit()) {
        return None;
    }
    match fraction {
        None if whole.is_empty() => None,
        None => Some(NumberKind::Integer),
        Some(fraction) if fraction.is_empty() || !fraction.chars().all(|c| c.is_ascii_digit()) => {
            None
        }
        Some(_) => Some(NumberKind::Float),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_scalars() {
        assert_eq!(classify_scalar("true"), AttrValue::Bool(true));
        assert_eq!(classify_scalar("false"), AttrValue::Bool(false));
        assert_eq!(classify_scalar("12"), AttrValue::Int(12));
        assert_eq!(classify_scalar("-3"), AttrValue::Int(-3));
        assert_eq!(classify_scalar("6.0"), AttrValue::Float(6.0));
        assert_eq!(classify_scalar("-2.5"), AttrValue::Float(-2.5));
        assert_eq!(classify_scalar("6px"), AttrValue::Raw("6px".to_string()));
        assert_eq!(classify_scalar("v1"), AttrValue::Raw("v1".to_string()));
        assert_eq!(
            classify_scalar("1.2.3"),
            AttrValue::Raw("1.2.3".to_string())
        );
        assert_eq!(classify_scalar("True"), AttrValue::Raw("True".to_string()));
        assert_eq!(classify_scalar("1."), AttrValue::Raw("1.".to_string()));
    }

    #[test]
    fn literals_keep_their_authored_shape() {
        assert_eq!(AttrValue::Int(12).literal(), "12");
        assert_eq!(AttrValue::Float(6.0).literal(), "6.0");
        assert_eq!(AttrValue::Float(6.25).literal(), "6.25");
        assert_eq!(AttrValue::Bool(true).literal(), "true");
        assert_eq!(AttrValue::Raw("red".to_string()).literal(), "red");
        assert_eq!(AttrValue::Str("a b".to_string()).literal(), "a b");
    }
}
