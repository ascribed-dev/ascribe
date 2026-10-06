//! Helpers over the spanned TOML tree.

use ascribe_core::Span;
use serde_yaml_ng::Value;
use toml::Spanned;
use toml::de::{DeTable, DeValue};

/// A spanned TOML value.
pub type V<'i> = Spanned<DeValue<'i>>;

/// The span of a spanned node.
pub fn sp<T>(v: &Spanned<T>) -> Span {
    let r = v.span();
    Span::new(r.start, r.end)
}

/// A key's text.
pub fn key_str<'a>(k: &'a Spanned<std::borrow::Cow<'_, str>>) -> &'a str {
    k.get_ref().as_ref()
}

/// How a value is described in a "but it's …" message.
pub fn describe(v: &DeValue<'_>) -> &'static str {
    match v {
        DeValue::String(_) => "a string",
        DeValue::Integer(_) | DeValue::Float(_) => "a number",
        DeValue::Boolean(_) => "a boolean",
        DeValue::Datetime(_) => "a date",
        DeValue::Array(_) => "an array",
        DeValue::Table(_) => "a table",
    }
}

/// Appends a key to a dotted path, quoting it if it isn't a bare key.
pub fn join(path: &str, key: &str) -> String {
    let bare = !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    let key = if bare {
        key.to_owned()
    } else {
        format!("\"{key}\"")
    };
    if path.is_empty() {
        key
    } else {
        format!("{path}.{key}")
    }
}

/// Converts a TOML value to YAML, for checking defaults with the same code
/// that checks frontmatter.
pub fn to_yaml(v: &DeValue<'_>) -> Value {
    match v {
        DeValue::String(s) => Value::String(s.to_string()),
        DeValue::Boolean(b) => Value::Bool(*b),
        DeValue::Integer(i) => {
            let text = i.as_str().replace('_', "");
            let parsed = match i.radix() {
                10 => text.parse::<i64>().ok(),
                radix => {
                    let (neg, rest) = match text.strip_prefix('-') {
                        Some(r) => (true, r),
                        None => (false, text.trim_start_matches('+')),
                    };
                    let digits = rest.get(2..).unwrap_or("");
                    i64::from_str_radix(digits, radix)
                        .ok()
                        .map(|n| if neg { -n } else { n })
                }
            };
            match parsed {
                Some(n) => Value::Number(n.into()),
                None => Value::Number(0.into()),
            }
        }
        DeValue::Float(f) => match f.as_str().replace('_', "").parse::<f64>() {
            Ok(n) => Value::Number(n.into()),
            Err(_) => Value::Number(0.into()),
        },
        DeValue::Datetime(d) => Value::String(d.to_string()),
        DeValue::Array(a) => Value::Sequence(a.iter().map(|x| to_yaml(x.get_ref())).collect()),
        DeValue::Table(t) => Value::Mapping(
            t.iter()
                .map(|(k, v)| (Value::String(key_str(k).to_owned()), to_yaml(v.get_ref())))
                .collect(),
        ),
    }
}

/// Iterates a table's entries as `(key, key span, value)`.
pub fn entries<'a, 'i>(
    t: &'a DeTable<'i>,
) -> impl Iterator<Item = (&'a str, Span, &'a V<'i>)> + 'a {
    t.iter().map(|(k, v)| (key_str(k), sp(k), v))
}
