//! The word-level diff inside a changed block of prose.

use similar::{Algorithm, DiffTag, capture_diff_slices};

/// Above this many tokens on either side, a changed block has no word-level
/// diff: it's marked changed as a whole. A token is a word, a punctuation
/// character, or a run of whitespace, so this is roughly 2,500 words.
pub const MAX_TOKENS: usize = 5000;

/// The words that differ between two texts, as ranges of characters
/// (Unicode scalar values) `[start, end)` into each.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WordRanges {
    /// Ranges in the old text: the words removed or replaced.
    pub was: Vec<[usize; 2]>,
    /// Ranges in the new text: the words added or replacing others.
    pub now: Vec<[usize; 2]>,
}

/// A token of text, with where it is in characters.
struct Token<'a> {
    text: &'a str,
    start: usize,
    end: usize,
    space: bool,
}

/// Splits text into words, runs of whitespace, and single characters of
/// anything else. A word is a run of letters, digits, and `_`, and the
/// joiners `.`, `-`, `'`, and `’` between two of them, so `2.4`,
/// `lantern.yaml`, `sign-in`, and `step's` are one word each.
fn tokenize(text: &str) -> Vec<Token<'_>> {
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let word = |c: char| c.is_alphanumeric() || c == '_';
    let joiner = |c: char| matches!(c, '.' | '-' | '\'' | '’');
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i].1;
        let start = i;
        if word(c) {
            i += 1;
            while i < chars.len() {
                let next = chars[i].1;
                let joins = joiner(next) && chars.get(i + 1).is_some_and(|(_, c)| word(*c));
                if word(next) {
                    i += 1;
                } else if joins {
                    i += 2;
                } else {
                    break;
                }
            }
        } else if c.is_whitespace() {
            while i < chars.len() && chars[i].1.is_whitespace() {
                i += 1;
            }
        } else {
            i += 1;
        }
        let byte_end = chars.get(i).map_or(text.len(), |(b, _)| *b);
        out.push(Token {
            text: &text[chars[start].0..byte_end],
            start,
            end: i,
            space: c.is_whitespace(),
        });
    }
    out
}

/// The words that differ between `was` and `now`, or `None` when either is
/// longer than [`MAX_TOKENS`].
pub fn diff_words(was: &str, now: &str) -> Option<WordRanges> {
    let old = tokenize(was);
    let new = tokenize(now);
    if old.len() > MAX_TOKENS || new.len() > MAX_TOKENS {
        return None;
    }
    let old_text: Vec<&str> = old.iter().map(|t| t.text).collect();
    let new_text: Vec<&str> = new.iter().map(|t| t.text).collect();
    let mut old_changed = vec![false; old.len()];
    let mut new_changed = vec![false; new.len()];
    for op in capture_diff_slices(Algorithm::Myers, &old_text, &new_text) {
        let (tag, old_range, new_range) = op.as_tag_tuple();
        if tag == DiffTag::Equal {
            continue;
        }
        old_range.for_each(|i| old_changed[i] = true);
        new_range.for_each(|i| new_changed[i] = true);
    }
    Some(WordRanges {
        was: ranges(&old, &old_changed),
        now: ranges(&new, &new_changed),
    })
}

/// The changed tokens as character ranges: whitespace never starts or ends
/// a range, and changed words with only whitespace between them are one
/// range.
fn ranges(tokens: &[Token<'_>], changed: &[bool]) -> Vec<[usize; 2]> {
    let mut out: Vec<[usize; 2]> = Vec::new();
    // The index of the last token in `out`'s last range.
    let mut last: Option<usize> = None;
    for (i, token) in tokens.iter().enumerate() {
        if !changed[i] || token.space {
            continue;
        }
        let joins = last.is_some_and(|l| tokens[l + 1..i].iter().all(|t| t.space));
        match out.last_mut() {
            Some(range) if joins => range[1] = token.end,
            _ => out.push([token.start, token.end]),
        }
        last = Some(i);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slice(text: &str, [a, b]: [usize; 2]) -> String {
        text.chars().skip(a).take(b - a).collect()
    }

    #[test]
    fn a_replaced_word_is_marked_on_both_sides() {
        let was = "Lantern agent 2.2 or later";
        let now = "Lantern agent 2.4 or later";
        let words = diff_words(was, now).unwrap();
        assert_eq!(
            words.was.iter().map(|r| slice(was, *r)).collect::<Vec<_>>(),
            ["2.2"]
        );
        assert_eq!(
            words.now.iter().map(|r| slice(now, *r)).collect::<Vec<_>>(),
            ["2.4"]
        );
    }

    #[test]
    fn added_words_join_across_spaces_and_leave_the_spaces_out() {
        let was = "Set the percentage.";
        let now = "Set the first step's percentage.";
        let words = diff_words(was, now).unwrap();
        assert!(words.was.is_empty());
        assert_eq!(
            words.now.iter().map(|r| slice(now, *r)).collect::<Vec<_>>(),
            ["first step's"]
        );
    }

    #[test]
    fn a_sentence_end_isnt_part_of_a_word() {
        let was = "Run it. Then stop.";
        let now = "Run it. Then wait.";
        let words = diff_words(was, now).unwrap();
        assert_eq!(
            words.now.iter().map(|r| slice(now, *r)).collect::<Vec<_>>(),
            ["wait"]
        );
    }

    #[test]
    fn ranges_count_characters_not_bytes() {
        let was = "café au lait";
        let now = "café au thé";
        let words = diff_words(was, now).unwrap();
        assert_eq!(words.was, vec![[8, 12]]);
        assert_eq!(words.now, vec![[8, 11]]);
    }

    #[test]
    fn long_texts_have_no_word_diff() {
        let long = "word ".repeat(MAX_TOKENS);
        assert_eq!(diff_words(&long, "word"), None);
    }
}
