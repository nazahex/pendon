//! Attribute values produced by the typed extras head
//! (`docs/spec/SYNTAX.md` §6.3).

/// The reserved object key that carries `...$var` spread items
/// (RFC `docs/rfc/plugin-bind.md` §2.6).
///
/// `to_attributes` cannot express "merge that object in" as a key/value pair, so
/// a spread is transported as this key whose value is a JSON **array** of the
/// reference strings, in source order. `...` can never be an authored key
/// (`is_key` requires a leading `ALPHA` or `_`), so the key is unreachable from
/// a document and free to carry the marker.
pub const SPREAD_KEY: &str = "...";

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
    /// A nested `{ … }` object; entries keep source order.
    Object(Vec<(String, AttrValue)>),
    /// A nested `[ … ]` array.
    Array(Vec<AttrValue>),
    /// A `{ … }` group whose body is a single JSX expression token — a
    /// dot-separated identifier path such as `valueA` or `frontmatter.fm1` —
    /// that must be emitted unquoted as `name={expr}` rather than as a string
    /// literal. Transported through [`pendon_core::JSX_ATTR_PREFIX`].
    JsxExpr(String),
    /// `null` — **not authorable**: §6.3 has no null literal, so the extras head
    /// never produces this. It exists so the projection is a total bridge back
    /// to the AST (`crate::project`): `plugin-bind` can resolve a `$var` to
    /// `null`, and every renderer already renders that value (§19).
    Null,
}

impl AttrValue {
    /// Literal text used by `Event::Attribute`, by interpolations
    /// (`{attrs.key}`) and by the stringly-typed DOM fallback path.
    ///
    /// A structured value (`Object` / `Array`) renders as compact JSON so a
    /// renderer that only understands strings still receives valid text.
    pub fn literal(&self) -> String {
        match self {
            Self::Str(text) | Self::Raw(text) => text.clone(),
            Self::Int(number) => number.to_string(),
            Self::Float(number) => float_literal(*number),
            Self::Bool(true) => "true".to_string(),
            Self::Bool(false) => "false".to_string(),
            Self::Object(_) | Self::Array(_) => {
                serde_json::to_string(&self.to_json()).unwrap_or_else(|_| "null".to_string())
            }
            Self::JsxExpr(expr) => {
                format!("{}{}", pendon_core::JSX_ATTR_PREFIX, expr)
            }
            Self::Null => "null".to_string(),
        }
    }

    /// Reads an AST value back into the typed form — the half of the bridge the
    /// projection ([`crate::project`]) uses, the inverse of [`AttrValue::to_json`].
    ///
    /// Total on purpose: every JSON value the pipeline can carry has an
    /// `AttrValue`, so a renderer reading the projection cannot silently lose an
    /// attribute. The one asymmetry is a `u64` above `i64::MAX`, which keeps its
    /// literal form (`Raw`) exactly as [`classify_scalar`] keeps an out-of-range
    /// integer authored in a document — no wrapping, no precision loss, at the
    /// cost of re-serialising as a string.
    pub fn from_json(value: &serde_json::Value) -> Self {
        use serde_json::Value;
        match value {
            Value::Null => Self::Null,
            Value::Bool(flag) => Self::Bool(*flag),
            Value::Number(number) => match number.as_i64() {
                Some(integer) => Self::Int(integer),
                // Neither branch below may round: an integer too large for
                // `i64` is still an integer, so it is kept as text.
                None if number.is_u64() => Self::Raw(number.to_string()),
                None => match number.as_f64() {
                    Some(decimal) => Self::Float(decimal),
                    None => Self::Raw(number.to_string()),
                },
            },
            Value::String(text) => Self::Str(text.clone()),
            Value::Array(items) => Self::Array(items.iter().map(Self::from_json).collect()),
            Value::Object(entries) => Self::Object(
                entries
                    .iter()
                    .map(|(key, item)| (key.clone(), Self::from_json(item)))
                    .collect(),
            ),
        }
    }

    /// The JSON form of this value, used by the data-binding renderer path
    /// (`docs/rfc/plugin-bind.md`).
    ///
    /// A structured value becomes a real JSON object/array; a scalar keeps its
    /// JSON type so the emitted `{…}` expression is valid JavaScript.
    pub fn to_json(&self) -> serde_json::Value {
        use serde_json::Value;
        match self {
            Self::Str(text) | Self::Raw(text) => Value::String(text.clone()),
            Self::Int(number) => Value::Number((*number).into()),
            Self::Float(number) => serde_json::Number::from_f64(*number)
                .map(Value::Number)
                .unwrap_or(Value::Null),
            Self::Bool(flag) => Value::Bool(*flag),
            Self::Object(entries) => {
                let mut map = serde_json::Map::new();
                for (key, value) in entries {
                    map.insert(key.clone(), value.to_json());
                }
                Value::Object(map)
            }
            Self::Array(items) => Value::Array(items.iter().map(AttrValue::to_json).collect()),
            Self::JsxExpr(expr) => Value::String(expr.clone()),
            Self::Null => Value::Null,
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
        assert_eq!(AttrValue::Null.literal(), "null");
    }

    /// The bridge is the projection's (`S2-2.7`): every AST value it reads must
    /// come back out unchanged, or a renderer would re-emit different bytes.
    #[test]
    fn the_json_bridge_is_lossless() {
        for ast in [
            serde_json::json!(null),
            serde_json::json!(true),
            serde_json::json!(false),
            serde_json::json!(12),
            serde_json::json!(-3),
            serde_json::json!(6.0),
            serde_json::json!(-2.5),
            serde_json::json!("text"),
            // A `JSX_ATTR_PREFIX` payload is a plain string here: the sentinel
            // rides inside the text, so the renderer still sees it.
            serde_json::json!(format!("{}valueA", pendon_core::JSX_ATTR_PREFIX)),
            serde_json::json!([1, "two", [true]]),
            serde_json::json!({ "b": 1, "a": { "c": null } }),
        ] {
            let typed = AttrValue::from_json(&ast);
            assert_eq!(typed.to_json(), ast, "{ast}");
        }
    }

    /// A numeric AST value becomes the narrowest typed form that keeps its
    /// value: `Int` when it fits, `Float` when it does not, and `Raw` only for
    /// an integer too large for `i64` (the same fallback `classify_scalar` uses
    /// for an authored out-of-range integer).
    #[test]
    fn json_numbers_pick_their_typed_form() {
        assert_eq!(
            AttrValue::from_json(&serde_json::json!(12)),
            AttrValue::Int(12)
        );
        assert_eq!(
            AttrValue::from_json(&serde_json::json!(6.5)),
            AttrValue::Float(6.5)
        );
        let huge = serde_json::json!(serde_json::Number::from(u64::MAX));
        assert_eq!(
            AttrValue::from_json(&huge),
            AttrValue::Raw(u64::MAX.to_string())
        );
        assert_eq!(
            AttrValue::from_json(&serde_json::json!(null)),
            AttrValue::Null
        );
    }
}
