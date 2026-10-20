//! What `ascribe search` looks for, and where in a block's text it is.
//!
//! A query is words, which must all be in one block in any order, and
//! phrases in double quotes, matched as written, a run of spaces matching
//! any run of white space. Each is a substring: a word matches inside a
//! longer one. Case is ignored, and nothing else is folded.

use std::fmt;
use std::ops::Range;

use ascribe_core::Coded;

/// A query that can't be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BadQuery {
    /// It has no words.
    Empty,
    /// A `"` isn't closed.
    UnclosedQuote {
        /// The query, as given.
        query: String,
    },
}

impl fmt::Display for BadQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BadQuery::Empty => f.write_str("the query has no words to search for"),
            BadQuery::UnclosedQuote { query } => {
                write!(f, "a quote in `{query}` isn't closed")
            }
        }
    }
}

impl std::error::Error for BadQuery {}

impl Coded for BadQuery {
    fn code(&self) -> &'static str {
        match self {
            BadQuery::Empty => "empty_query",
            BadQuery::UnclosedQuote { .. } => "unclosed_quote",
        }
    }
}

/// A query, read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Query {
    /// The words and phrases, each folded and with its white space
    /// collapsed.
    terms: Vec<Vec<char>>,
}

impl Query {
    /// Reads a query: words, and phrases in double quotes.
    ///
    /// # Errors
    ///
    /// The query has no words, or a quote isn't closed.
    pub fn parse(query: &str) -> Result<Query, BadQuery> {
        let mut terms = Vec::new();
        let mut quoted = false;
        for (n, part) in query.split('"').enumerate() {
            quoted = n % 2 == 1;
            if quoted {
                let phrase: Vec<&str> = part.split_whitespace().collect();
                if !phrase.is_empty() {
                    terms.push(fold(&phrase.join(" ")));
                }
            } else {
                terms.extend(part.split_whitespace().map(fold));
            }
        }
        if quoted {
            return Err(BadQuery::UnclosedQuote {
                query: query.to_owned(),
            });
        }
        if terms.is_empty() {
            return Err(BadQuery::Empty);
        }
        Ok(Query { terms })
    }

    /// Where the query matches `text`, a block's text with its white space
    /// collapsed: from the first character any word or phrase covers to the
    /// last, in bytes. `None` unless every one of them is in it.
    pub fn find_in(&self, text: &str) -> Option<Range<usize>> {
        let chars: Vec<(usize, char)> = text
            .char_indices()
            .map(|(i, c)| (i, fold_char(c)))
            .collect();
        let mut first = usize::MAX;
        let mut last = 0;
        for term in &self.terms {
            let at = chars
                .windows(term.len())
                .position(|w| w.iter().map(|(_, c)| c).eq(term.iter()))?;
            first = first.min(at);
            last = last.max(at + term.len());
        }
        let start = chars.get(first).map_or(text.len(), |(i, _)| *i);
        let end = chars.get(last).map_or(text.len(), |(i, _)| *i);
        Some(start..end)
    }
}

/// A character with its case ignored: its lowercase, when that's one
/// character, so a block's folded text has the characters it has.
fn fold_char(c: char) -> char {
    let mut lower = c.to_lowercase();
    match (lower.next(), lower.next()) {
        (Some(l), None) => l,
        _ => c,
    }
}

fn fold(text: &str) -> Vec<char> {
    text.chars().map(fold_char).collect()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn found<'t>(query: &str, text: &'t str) -> Option<&'t str> {
        Query::parse(query).unwrap().find_in(text).map(|r| &text[r])
    }

    #[test]
    fn words_match_in_any_order_from_the_first_to_the_last() {
        assert_eq!(
            found("key api", "Copy an API key from Settings"),
            Some("API key")
        );
        assert_eq!(found("key zebra", "Copy an API key"), None);
    }

    #[test]
    fn a_phrase_matches_as_written_with_any_white_space() {
        assert_eq!(
            found("\"sign   in to\"", "Sign in to Quill Cloud"),
            Some("Sign in to")
        );
        assert_eq!(found("\"in sign\"", "Sign in to Quill Cloud"), None);
    }

    #[test]
    fn case_is_ignored_and_a_word_matches_inside_a_longer_one() {
        assert_eq!(found("QUILL", "the quill agent"), Some("quill"));
        assert_eq!(found("sync", "Streaming synchronization"), Some("sync"));
        assert_eq!(found("ça", "Ça va"), Some("Ça"));
    }

    #[test]
    fn nothing_else_is_folded() {
        assert_eq!(found("cafe", "un café"), None);
    }

    #[test]
    fn an_empty_query_or_an_open_quote_is_an_error() {
        assert_eq!(Query::parse("  "), Err(BadQuery::Empty));
        assert_eq!(Query::parse("\"\""), Err(BadQuery::Empty));
        assert_eq!(
            Query::parse("\"rate limit"),
            Err(BadQuery::UnclosedQuote {
                query: "\"rate limit".to_owned()
            })
        );
        assert_eq!(BadQuery::Empty.code(), "empty_query");
    }
}
