//! What `ascribe search` looks for, and where in a block's text it is.
//!
//! A query is words, which must all be in one block in any order, and
//! phrases in double quotes, matched as written, a run of spaces matching
//! any run of white space. Each is a substring: a word matches inside a
//! longer one, unless the query keeps to whole words. Case is ignored
//! unless the query matches it, and nothing else is folded.

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
    /// The words and phrases, each as written, with its white space
    /// collapsed.
    terms: Vec<Vec<char>>,
    /// Whether case is matched as written.
    case: bool,
    /// Whether a word or a phrase matches only where it isn't inside a
    /// longer word.
    word: bool,
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
                    terms.push(phrase.join(" ").chars().collect());
                }
            } else {
                terms.extend(part.split_whitespace().map(|w| w.chars().collect()));
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
        Ok(Query {
            terms,
            case: false,
            word: false,
        })
    }

    /// The query, matching case as written when `case` is true.
    #[must_use]
    pub fn matching_case(mut self, case: bool) -> Query {
        self.case = case;
        self
    }

    /// The query, matching each word or phrase only where the characters on
    /// either side of it aren't letters, digits, or `_`, when `word` is
    /// true.
    #[must_use]
    pub fn whole_words(mut self, word: bool) -> Query {
        self.word = word;
        self
    }

    /// Where the query matches `text`, a block's text with its white space
    /// collapsed: from the first character any word or phrase covers to the
    /// last, in bytes. `None` unless every one of them is in it.
    pub fn find_in(&self, text: &str) -> Option<Range<usize>> {
        self.find_from(text, 0)
    }

    /// Where the query matches `text` at or after the byte `from`, as
    /// [`Query::find_in`] does: the characters before `from` are still
    /// what a whole word is told by.
    pub fn find_from(&self, text: &str, from: usize) -> Option<Range<usize>> {
        let chars: Vec<(usize, char)> = text
            .char_indices()
            .map(|(i, c)| (i, if self.case { c } else { fold_char(c) }))
            .collect();
        let skip = chars.partition_point(|(i, _)| *i < from);
        let mut first = usize::MAX;
        let mut last = 0;
        for term in &self.terms {
            let term: Vec<char> = if self.case {
                term.clone()
            } else {
                term.iter().copied().map(fold_char).collect()
            };
            let at = skip
                + chars
                    .get(skip..)?
                    .windows(term.len())
                    .enumerate()
                    .position(|(n, w)| {
                        w.iter().map(|(_, c)| c).eq(term.iter())
                            && (!self.word || bounded(&chars, skip + n, term.len()))
                    })?;
            first = first.min(at);
            last = last.max(at + term.len());
        }
        let start = chars.get(first).map_or(text.len(), |(i, _)| *i);
        let end = chars.get(last).map_or(text.len(), |(i, _)| *i);
        Some(start..end)
    }
}

/// Whether the `len` characters at `at` of `chars` aren't inside a longer
/// word.
fn bounded(chars: &[(usize, char)], at: usize, len: usize) -> bool {
    let before = at.checked_sub(1).and_then(|i| chars.get(i));
    let after = chars.get(at + len);
    !before.is_some_and(|(_, c)| is_word_char(*c)) && !after.is_some_and(|(_, c)| is_word_char(*c))
}

/// Whether a character can be part of a word, for `--word`: a letter, a
/// digit, or `_`.
fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
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

    fn found_as(query: &str, case: bool, word: bool, text: &str) -> Option<String> {
        let query = Query::parse(query)
            .unwrap()
            .matching_case(case)
            .whole_words(word);
        query.find_in(text).map(|r| text[r].to_owned())
    }

    #[test]
    fn case_is_matched_as_written_when_asked() {
        assert_eq!(
            found_as("Vale", true, false, "run vale, then Vale"),
            Some("Vale".to_owned())
        );
        assert_eq!(found_as("Vale", true, false, "run vale"), None);
        assert_eq!(found_as("vale", true, false, "Vale"), None);
    }

    #[test]
    fn a_whole_word_is_not_inside_a_longer_one() {
        assert_eq!(found_as("limit", false, true, "limits apply"), None);
        assert_eq!(
            found_as("limit", false, true, "the rate limit."),
            Some("limit".to_owned())
        );
        assert_eq!(
            found_as("limit", false, true, "limit"),
            Some("limit".to_owned()),
            "a block's start and end"
        );
        assert_eq!(
            found_as("limit", false, true, "(limit)"),
            Some("limit".to_owned())
        );
        assert_eq!(
            found_as("limit", false, true, "rate_limit"),
            None,
            "`_` is a word's"
        );
        assert_eq!(
            found_as("limit", false, true, "limit2"),
            None,
            "a digit is a word's"
        );
        assert_eq!(
            found_as("limit", false, true, "limits, then the limit"),
            Some("limit".to_owned()),
            "a later whole word is found past a longer one"
        );
        assert_eq!(
            found_as("\"rate limit\"", false, true, "a rate limits, a rate limit"),
            Some("rate limit".to_owned()),
            "a phrase is whole at its ends"
        );
        assert_eq!(
            found_as("\"rate limit\"", false, true, "pirate limit"),
            None
        );
    }

    #[test]
    fn a_whole_word_after_a_start_is_told_by_what_comes_before_it() {
        let query = Query::parse("limit").unwrap().whole_words(true);
        assert_eq!(query.find_from("ratelimit limit", 4), Some(10..15));
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
