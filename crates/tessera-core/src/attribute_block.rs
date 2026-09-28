//! Parsed attribute blocks (SPEC §3.3): the output of the attribute parser.
//!
//! One grammar serves directives and images, so one set of types does too.
//! Phase 05 writes the parser (`attributes.rs`, producing these types), and
//! phase 07 reuses it for images. The types record each value's *form*
//! (token, quoted string, or value set), never its type: types come from the
//! schema (SPEC §3.3), and the checks (phase 10) interpret values against it.
//!
//! Every part keeps its span, in the coordinates of the file being parsed,
//! so the formatter can rewrite spacing and quoting and the language server
//! can point at a single key or value.

use crate::Span;

/// An attribute block: `{` pairs `}`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttributeBlock {
    /// From `{` through `}`, inclusive.
    pub span: Span,
    /// The attributes in source order. A key given twice appears twice;
    /// reporting it is up to the checks. `{}` has none.
    pub attributes: Vec<Attribute>,
}

impl AttributeBlock {
    /// The first attribute with this key.
    pub fn get(&self, key: &str) -> Option<&Attribute> {
        self.attributes.iter().find(|a| a.key == key)
    }
}

/// One `key=value` pair.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attribute {
    /// The key (ABNF rule `key`).
    pub key: String,
    /// The key's span.
    pub key_span: Span,
    /// The value, or `None` for a bare key with no `=` and value, which is an
    /// error (SPEC §3.3) that the parser has reported. Keeping the pair lets
    /// the language server complete the value and the checks still see the key.
    pub value: Option<AttributeValue>,
    /// From the start of the key to the end of the value (or of the key, for
    /// a bare key). Excludes the separating comma and surrounding spaces.
    pub span: Span,
}

/// An attribute value, in the form it was written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AttributeValue {
    /// An unquoted value (ABNF rule `token`): `caution`, `3.4`, `600px`.
    Token(Token),
    /// A double-quoted value (ABNF rule `quoted`).
    Quoted {
        /// The value with its escapes (`\"`, `\\`) resolved and the quotes removed.
        text: String,
        /// From the opening quote through the closing quote.
        span: Span,
    },
    /// Tokens joined by `|` (ABNF rule `value-set`): `cloud|on-prem`.
    /// Only schema keys typed `set(…)` accept one; the parser records the
    /// form regardless, and the checks report it where it isn't allowed.
    Set {
        /// The members, in source order. At least two.
        members: Vec<Token>,
        /// From the first member through the last.
        span: Span,
    },
}

/// A token: an unquoted value, or a member of a value set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    /// The token's text, exactly as written.
    pub text: String,
    /// Its span.
    pub span: Span,
}

impl AttributeValue {
    /// The value's span.
    pub fn span(&self) -> Span {
        match self {
            AttributeValue::Token(t) => t.span,
            AttributeValue::Quoted { span, .. } | AttributeValue::Set { span, .. } => *span,
        }
    }

    /// The value's text, for a token or quoted string. `None` for a set.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            AttributeValue::Token(t) => Some(&t.text),
            AttributeValue::Quoted { text, .. } => Some(text),
            AttributeValue::Set { .. } => None,
        }
    }

    /// The values the attribute names: a set's members, or the single value.
    /// This is how a `set(…)` key reads either form (a single token is a
    /// one-member set).
    pub fn members(&self) -> Vec<&str> {
        match self {
            AttributeValue::Set { members, .. } => {
                members.iter().map(|t| t.text.as_str()).collect()
            }
            other => other.as_text().into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token(text: &str, at: usize) -> Token {
        Token {
            text: text.into(),
            span: Span::new(at, at + text.len()),
        }
    }

    #[test]
    fn reads_values_in_every_form() {
        // {platform=cloud|on-prem, label="A \"b\"", heading}
        let block = AttributeBlock {
            span: Span::new(0, 48),
            attributes: vec![
                Attribute {
                    key: "platform".into(),
                    key_span: Span::new(1, 9),
                    value: Some(AttributeValue::Set {
                        members: vec![token("cloud", 10), token("on-prem", 16)],
                        span: Span::new(10, 23),
                    }),
                    span: Span::new(1, 23),
                },
                Attribute {
                    key: "label".into(),
                    key_span: Span::new(25, 30),
                    value: Some(AttributeValue::Quoted {
                        text: "A \"b\"".into(),
                        span: Span::new(31, 40),
                    }),
                    span: Span::new(25, 40),
                },
                Attribute {
                    key: "heading".into(),
                    key_span: Span::new(42, 49),
                    value: None,
                    span: Span::new(42, 49),
                },
            ],
        };
        let platform = block.get("platform").and_then(|a| a.value.as_ref());
        assert_eq!(
            platform.map(|v| v.members()),
            Some(vec!["cloud", "on-prem"])
        );
        assert_eq!(platform.and_then(|v| v.as_text()), None);
        let label = block.get("label").and_then(|a| a.value.as_ref());
        assert_eq!(label.and_then(|v| v.as_text()), Some("A \"b\""));
        assert_eq!(label.map(|v| v.members()), Some(vec!["A \"b\""]));
        assert_eq!(label.map(|v| v.span()), Some(Span::new(31, 40)));
        assert!(block.get("heading").is_some_and(|a| a.value.is_none()));
        assert!(block.get("missing").is_none());
    }
}
