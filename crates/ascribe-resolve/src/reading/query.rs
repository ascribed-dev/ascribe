//! What `ascribe search` looks for, and where in a block's text it is.
//!
//! A query is words, which must all be in one block in any order, and
//! phrases in double quotes, matched as written, a run of spaces matching
//! any run of white space. Each is a substring: a word matches inside a
//! longer one, unless the query keeps to whole words. Case is ignored
//! unless the query matches it, and nothing else is folded.
//!
//! A stop word, a common English word such as `how` or `the` written
//! outside quotes, isn't needed: `how do I publish llms.txt` matches where
//! `publish llms.txt` does. A query of only stop words needs them all. When
//! no block has every word a match needs, [`Query::partial_from`] says
//! which of them a block has, so the closest blocks can be listed.

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

/// The words a query doesn't need a block to have, unless they're all it
/// has: common English words, which a question in natural words carries.
/// There's no setting for a project's language yet, so this is English's.
const STOP_WORDS: &[&str] = &[
    "a", "about", "am", "an", "and", "are", "as", "at", "be", "by", "can", "could", "did", "do",
    "does", "for", "from", "how", "i", "if", "in", "into", "is", "it", "its", "me", "my", "of",
    "on", "or", "our", "should", "so", "that", "the", "their", "them", "then", "there", "these",
    "this", "those", "to", "us", "was", "we", "were", "what", "when", "where", "which", "who",
    "why", "will", "with", "would", "you", "your",
];

/// A query, read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Query {
    /// The words and phrases, in the order written.
    terms: Vec<Term>,
    /// Whether case is matched as written.
    case: bool,
    /// Whether a word or a phrase matches only where it isn't inside a
    /// longer word.
    word: bool,
}

/// A word or a phrase of a query.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Term {
    /// Its characters, as written, with a phrase's white space collapsed.
    chars: Vec<char>,
    /// Whether it's a phrase, written in quotes.
    phrase: bool,
    /// Whether it's a stop word: a word, not a phrase, in [`STOP_WORDS`].
    stop: bool,
}

impl Term {
    fn word(word: &str) -> Term {
        let folded: String = word.chars().map(fold_char).collect();
        Term {
            chars: word.chars().collect(),
            phrase: false,
            stop: STOP_WORDS.contains(&folded.as_str()),
        }
    }

    fn phrase(phrase: &str) -> Term {
        Term {
            chars: phrase.chars().collect(),
            phrase: true,
            stop: false,
        }
    }

    /// The term as the query writes it: a phrase in its quotes.
    fn written(&self) -> String {
        let text: String = self.chars.iter().collect();
        if self.phrase {
            format!("\"{text}\"")
        } else {
            text
        }
    }
}

/// Which of the words a match needs a block has, when it doesn't have them
/// all: what [`Query::partial_from`] finds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Partial {
    /// From the first character a word or a phrase it has covers to the
    /// last, in bytes; the stop words it has aren't in it.
    pub matched: Range<usize>,
    /// For each word or phrase a match needs, in [`Query::needed`]'s order,
    /// whether the block has it.
    pub has: Vec<bool>,
    /// How many of the query's stop words the block has.
    pub stop_words: usize,
}

impl Partial {
    /// How many of the words a match needs the block has.
    pub fn count(&self) -> usize {
        self.has.iter().filter(|h| **h).count()
    }
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
                    terms.push(Term::phrase(&phrase.join(" ")));
                }
            } else {
                terms.extend(part.split_whitespace().map(Term::word));
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
    /// what a whole word is told by. A stop word isn't needed, and isn't in
    /// the range, unless every word is one.
    pub fn find_from(&self, text: &str, from: usize) -> Option<Range<usize>> {
        let chars = self.folded(text);
        let skip = chars.partition_point(|(i, _)| *i < from);
        let mut first = usize::MAX;
        let mut last = 0;
        for term in self.needed_terms() {
            let at = self.position(&chars, skip, term)?;
            first = first.min(at);
            last = last.max(at + term.chars.len());
        }
        Some(byte_range(text, &chars, first, last))
    }

    /// Which of the words and phrases a match needs `text` has at or after
    /// the byte `from`, and how many of the stop words, when it has some of
    /// them but not all: `None` when it has none of them, or every one.
    pub fn partial_from(&self, text: &str, from: usize) -> Option<Partial> {
        let chars = self.folded(text);
        let skip = chars.partition_point(|(i, _)| *i < from);
        let mut first = usize::MAX;
        let mut last = 0;
        let mut has = Vec::new();
        for term in self.needed_terms() {
            let at = self.position(&chars, skip, term);
            if let Some(at) = at {
                first = first.min(at);
                last = last.max(at + term.chars.len());
            }
            has.push(at.is_some());
        }
        if has.iter().all(|h| *h) || !has.iter().any(|h| *h) {
            return None;
        }
        let stop_words = if self.all_stop() {
            0
        } else {
            self.terms
                .iter()
                .filter(|t| t.stop && self.position(&chars, skip, t).is_some())
                .count()
        };
        Some(Partial {
            matched: byte_range(text, &chars, first, last),
            has,
            stop_words,
        })
    }

    /// The words and phrases a match needs, as the query writes them, a
    /// phrase in its quotes: every one but the stop words, or every one
    /// when they're all stop words.
    pub fn needed(&self) -> Vec<String> {
        self.needed_terms().map(Term::written).collect()
    }

    /// The query's stop words, as written, which a match doesn't need:
    /// none when every word is one, since then a match needs them all.
    pub fn stop_words(&self) -> Vec<String> {
        if self.all_stop() {
            return Vec::new();
        }
        self.terms
            .iter()
            .filter(|t| t.stop)
            .map(Term::written)
            .collect()
    }

    fn all_stop(&self) -> bool {
        self.terms.iter().all(|t| t.stop)
    }

    fn needed_terms(&self) -> impl Iterator<Item = &Term> {
        let all = self.all_stop();
        self.terms.iter().filter(move |t| all || !t.stop)
    }

    /// `text`'s characters, each at its byte, with case folded unless the
    /// query matches it.
    fn folded(&self, text: &str) -> Vec<(usize, char)> {
        text.char_indices()
            .map(|(i, c)| (i, if self.case { c } else { fold_char(c) }))
            .collect()
    }

    /// The first character of `chars` at or after `skip` where `term`
    /// matches.
    fn position(&self, chars: &[(usize, char)], skip: usize, term: &Term) -> Option<usize> {
        let term: Vec<char> = if self.case {
            term.chars.clone()
        } else {
            term.chars.iter().copied().map(fold_char).collect()
        };
        chars
            .get(skip..)?
            .windows(term.len())
            .enumerate()
            .position(|(n, w)| {
                w.iter().map(|(_, c)| c).eq(term.iter())
                    && (!self.word || bounded(chars, skip + n, term.len()))
            })
            .map(|n| skip + n)
    }
}

/// The bytes of `text` from the character `first` of `chars` to the one
/// before `last`.
fn byte_range(text: &str, chars: &[(usize, char)], first: usize, last: usize) -> Range<usize> {
    let start = chars.get(first).map_or(text.len(), |(i, _)| *i);
    let end = chars.get(last).map_or(text.len(), |(i, _)| *i);
    start..end
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
    fn stop_words_are_not_needed_unless_every_word_is_one() {
        assert_eq!(
            found("how do I publish llms.txt", "Publish llms.txt for agents"),
            Some("Publish llms.txt")
        );
        assert_eq!(found("how do", "how do you"), Some("how do"));
        assert_eq!(found("how do", "how it works"), None);
        assert_eq!(found("\"the\" key", "a key"), None, "a phrase is needed");
        let query = Query::parse("How do I publish \"llms.txt\"").unwrap();
        assert_eq!(query.needed(), ["publish", "\"llms.txt\""]);
        assert_eq!(query.stop_words(), ["How", "do", "I"]);
        let only = Query::parse("how do").unwrap();
        assert_eq!(only.needed(), ["how", "do"]);
        assert!(only.stop_words().is_empty());
    }

    #[test]
    fn a_partial_match_says_which_needed_words_a_block_has() {
        let query = Query::parse("webhook signature the").unwrap();
        let text = "A key signs the events the agent sends to your webhooks.";
        let partial = query.partial_from(text, 0).unwrap();
        assert_eq!(partial.has, [true, false]);
        assert_eq!((partial.count(), partial.stop_words), (1, 1));
        assert_eq!(&text[partial.matched], "webhook");
        assert_eq!(query.partial_from("the webhook signature", 0), None, "all");
        assert_eq!(query.partial_from("the events", 0), None, "none");
        let word = Query::parse("limit rate").unwrap().whole_words(true);
        assert_eq!(
            word.partial_from("limits, the rate", 0).unwrap().has,
            [false, true]
        );
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
