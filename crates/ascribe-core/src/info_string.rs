//! The info string of a fenced code block: its language, then words, some of
//! them `key=value` pairs (SPEC §3.3.1). Every reader of a pair in an info
//! string reads it here, so `phrases`, `title`, `tested`, `request`, and
//! `response` are read the same way everywhere.
//!
//! The info string is the one CommonMark gives, with its Markdown escapes
//! taken off. A quoted value then has the escapes of an attribute block's
//! (§3.3): `\"` and `\\`.

use std::borrow::Cow;

/// A code block's info string, read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InfoString<'a> {
    /// The first word, when it isn't a pair.
    pub language: Option<&'a str>,
    /// The words after the language, or every word when there's none.
    pub words: Vec<InfoWord<'a>>,
}

/// A word of an info string: a pair, or any other word.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InfoWord<'a> {
    /// The word as written. A pair's runs from its key to the end of its
    /// value, quotes included.
    pub text: &'a str,
    /// Where it starts in the text it was read from.
    pub start: usize,
    /// Its key and value, when it's a pair.
    pub pair: Option<InfoPair<'a>>,
}

/// A `key=value` word.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InfoPair<'a> {
    /// The key.
    pub key: &'a str,
    /// The value, with a quoted one's quotes and escapes taken off.
    pub value: Cow<'a, str>,
    /// Where the value starts in the text it was read from, inside its
    /// quotes when it's quoted.
    pub value_start: usize,
    /// The value's length as written, without its quotes.
    pub value_len: usize,
}

impl<'a> InfoString<'a> {
    /// Reads an info string: its first word is its language unless it's a
    /// pair.
    pub fn parse(info: &'a str) -> InfoString<'a> {
        let mut words = words(info);
        let language = match words.first() {
            Some(first) if first.pair.is_none() => Some(words.remove(0).text),
            _ => None,
        };
        InfoString { language, words }
    }

    /// The first pair with `key`.
    pub fn pair(&self, key: &str) -> Option<&InfoPair<'a>> {
        self.words
            .iter()
            .filter_map(|w| w.pair.as_ref())
            .find(|p| p.key == key)
    }

    /// The value of the first pair with `key`.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.pair(key).map(|p| p.value.as_ref())
    }

    /// Whether the block opts in to phrases: `phrases=true` (SPEC §5.1).
    pub fn phrases(&self) -> bool {
        self.get("phrases") == Some("true")
    }

    /// The info string again, without the pairs whose keys are in `keys`,
    /// its words parted by one space.
    pub fn without(&self, keys: &[&str]) -> String {
        let kept = self
            .words
            .iter()
            .filter(|w| w.pair.as_ref().is_none_or(|p| !keys.contains(&p.key)))
            .map(|w| w.text);
        self.language
            .into_iter()
            .chain(kept)
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// The words of `text`, parted by spaces and tabs, with no language taken
/// out: what follows the language, such as the `data-meta` a renderer
/// writes.
pub fn words(text: &str) -> Vec<InfoWord<'_>> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if is_space(bytes[i]) {
            i += 1;
            continue;
        }
        let start = i;
        let (end, pair) = match pair_at(text, start) {
            Some((end, pair)) => (end, Some(pair)),
            None => (word_end(bytes, start), None),
        };
        out.push(InfoWord {
            text: &text[start..end],
            start,
            pair,
        });
        i = end;
    }
    out
}

/// A value as a quoted value of an info string: in double quotes, with `"`
/// and `\` escaped.
pub fn quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

/// The keys of the pairs a processor reads, SPEC §3.3.1's table.
pub const KEYS: &[&str] = &["phrases", "title", "tested", "request", "response"];

/// A word after the language that starts like a pair a processor reads but
/// isn't one (SPEC §3.3.1): one of [`KEYS`], in any case, then optional
/// spaces and `=`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NearMiss {
    /// The key it starts like, as [`KEYS`] has it.
    pub key: &'static str,
    /// Where what was meant as the pair starts in the text it was read from.
    pub start: usize,
    /// Where it ends: past its value, which may take in the words after it,
    /// as `phrases = true` and `title="say "hi""` do.
    pub end: usize,
    /// What keeps it from being a pair.
    pub problem: NearMissProblem,
    /// The pair it was meant to be, when that's clear, as an info string
    /// holds it.
    pub fixed: Option<String>,
}

/// What keeps a [`NearMiss`] from being a pair.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NearMissProblem {
    /// The key isn't lowercase, as `Title=x`.
    Case,
    /// Spaces around `=`, as `phrases = true`.
    Spaced,
    /// A quote inside the value, as `title="say "hi""` or `title=a"b`.
    Quote,
    /// A quoted value that doesn't close, as `title="connect.ts`.
    Unclosed,
    /// Anything else, such as no value: `title=`.
    Other,
}

impl NearMissProblem {
    /// The name a diagnostic's message variant has for it.
    pub fn as_str(self) -> &'static str {
        match self {
            NearMissProblem::Case => "case",
            NearMissProblem::Spaced => "spaced",
            NearMissProblem::Quote => "quote",
            NearMissProblem::Unclosed => "unclosed",
            NearMissProblem::Other => "other",
        }
    }
}

/// The words after an info string's language that start like a pair a
/// processor reads and aren't one, in order. A word inside what one of them
/// meant as its value isn't looked at again.
pub fn near_misses(info: &str) -> Vec<NearMiss> {
    let parsed = InfoString::parse(info);
    let mut out: Vec<NearMiss> = Vec::new();
    for word in &parsed.words {
        if word.pair.is_some() || out.last().is_some_and(|m| word.start < m.end) {
            continue;
        }
        if let Some(miss) = near_miss_at(info, word.start) {
            out.push(miss);
        }
    }
    out
}

/// The key in [`KEYS`] that `text` starts with at `start`, in any case,
/// followed by optional spaces and `=`, and where the `=` is.
fn key_at(text: &str, start: usize) -> Option<(&'static str, usize)> {
    let rest = text.get(start..)?;
    KEYS.iter().find_map(|key| {
        let written = rest.get(..key.len())?;
        if !written.eq_ignore_ascii_case(key) {
            return None;
        }
        let after = &rest.as_bytes()[key.len()..];
        let spaces = after.iter().take_while(|&&b| is_space(b)).count();
        (after.get(spaces) == Some(&b'=')).then_some((*key, start + key.len() + spaces))
    })
}

/// A quoted value's text with its escapes, `\\"` and `\\\\`, taken off.
fn unescaped(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
        match (c, chars.peek()) {
            ('\\', Some(&next @ ('"' | '\\'))) => {
                out.push(next);
                chars.next();
            }
            _ => out.push(c),
        }
    }
    out
}

/// Whether a word starts at `at` that ends what a near miss before it
/// meant as its value: a pair, or another near miss.
fn starts_another(text: &str, at: usize) -> bool {
    pair_at(text, at).is_some() || key_at(text, at).is_some()
}

fn near_miss_at(text: &str, start: usize) -> Option<NearMiss> {
    let bytes = text.as_bytes();
    let (key, equals) = key_at(text, start)?;
    let mut value_start = equals + 1;
    while bytes.get(value_start).is_some_and(|&b| is_space(b)) {
        value_start += 1;
    }
    let spaced = equals > start + key.len() || value_start > equals + 1;
    let named = |otherwise| {
        if &text[start..start + key.len()] != key {
            NearMissProblem::Case
        } else if spaced {
            NearMissProblem::Spaced
        } else {
            otherwise
        }
    };
    // No value, or only a word after the spaces that's a pair of its own.
    if value_start == bytes.len() || (spaced && starts_another(text, value_start)) {
        return Some(NearMiss {
            key,
            start,
            end: equals + 1,
            problem: named(NearMissProblem::Other),
            fixed: None,
        });
    }
    let (end, value, problem) = if bytes[value_start] == b'"' {
        quoted_value(text, value_start)
    } else {
        let end = word_end(bytes, value_start);
        let token = &text[value_start..end];
        if token.contains('"') {
            (end, quote(token), NearMissProblem::Quote)
        } else {
            (end, token.to_owned(), NearMissProblem::Other)
        }
    };
    let fixed = format!("{key}={value}");
    let fixed = match words(&fixed).as_slice() {
        [only] if only.pair.is_some() && only.text.len() == fixed.len() => Some(fixed),
        _ => None,
    };
    Some(NearMiss {
        key,
        start,
        end,
        problem: named(problem),
        fixed,
    })
}

/// The quoted value that starts at `open`, its quote: where it ends, the
/// value quoted again, and what was wrong with it. A value whose first
/// closing quote is followed by a space or the end is right as it is.
/// Otherwise it was meant to close at the last quote before a space or the
/// end, short of a word that starts another pair; with no such quote, it
/// doesn't close.
fn quoted_value(text: &str, open: usize) -> (usize, String, NearMissProblem) {
    let bytes = text.as_bytes();
    let value = |end: usize| quote(&unescaped(&text[open + 1..end]));
    let closes_at = |j: usize| bytes.get(j + 1).is_none_or(|&b| is_space(b));
    // The quotes after the opening one that aren't escaped, with whether a
    // word starting another pair comes before each.
    let mut quotes = Vec::new();
    let mut stop = None;
    let mut j = open + 1;
    while j < bytes.len() {
        match bytes[j] {
            b'\\' if matches!(bytes.get(j + 1), Some(b'"' | b'\\')) => j += 1,
            b'"' => quotes.push((j, stop.is_some())),
            b if is_space(b) && stop.is_none() && starts_another(text, j + 1) => stop = Some(j),
            _ => {}
        }
        j += 1;
    }
    if let Some(&(first, _)) = quotes.first()
        && closes_at(first)
    {
        return (first + 1, value(first), NearMissProblem::Other);
    }
    let stop = stop.unwrap_or(bytes.len());
    let quotes: Vec<usize> = quotes
        .into_iter()
        .filter_map(|(q, past)| (!past).then_some(q))
        .collect();
    match quotes.iter().rev().find(|&&q| closes_at(q)) {
        Some(&last) => (last + 1, value(last), NearMissProblem::Quote),
        None => {
            let end = text[..stop].trim_end_matches([' ', '\t']).len();
            (end, value(end), NearMissProblem::Unclosed)
        }
    }
}

fn is_space(b: u8) -> bool {
    b == b' ' || b == b'\t'
}

fn word_end(bytes: &[u8], from: usize) -> usize {
    bytes[from..]
        .iter()
        .position(|&b| is_space(b))
        .map_or(bytes.len(), |n| from + n)
}

/// The pair that starts at `start`, and where it ends: a key, `=`, and a
/// token or a quoted value, then a space, a tab, or the end.
fn pair_at(text: &str, start: usize) -> Option<(usize, InfoPair<'_>)> {
    let bytes = text.as_bytes();
    let mut i = start;
    if !bytes.get(i)?.is_ascii_lowercase() {
        return None;
    }
    while bytes
        .get(i)
        .is_some_and(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'-')
    {
        i += 1;
    }
    let key = &text[start..i];
    if bytes.get(i) != Some(&b'=') {
        return None;
    }
    i += 1;
    let (value, value_start, value_len, end) = if bytes.get(i) == Some(&b'"') {
        let value_start = i + 1;
        let mut j = value_start;
        let mut unescaped: Option<String> = None;
        loop {
            match bytes.get(j)? {
                b'"' => break,
                b'\\' if matches!(bytes.get(j + 1), Some(b'"' | b'\\')) => {
                    let s = unescaped.get_or_insert_with(|| text[value_start..j].to_owned());
                    s.push(char::from(bytes[j + 1]));
                    j += 2;
                }
                _ => {
                    let c = text[j..].chars().next()?;
                    if let Some(s) = unescaped.as_mut() {
                        s.push(c);
                    }
                    j += c.len_utf8();
                }
            }
        }
        let value = match unescaped {
            Some(s) => Cow::Owned(s),
            None => Cow::Borrowed(&text[value_start..j]),
        };
        (value, value_start, j - value_start, j + 1)
    } else {
        let end = word_end(bytes, i);
        if end == i || text[i..end].contains('"') {
            return None;
        }
        (Cow::Borrowed(&text[i..end]), i, end - i, end)
    };
    if bytes.get(end).is_some_and(|&b| !is_space(b)) {
        return None;
    }
    Some((
        end,
        InfoPair {
            key,
            value,
            value_start,
            value_len,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pairs(info: &str) -> Vec<(String, String)> {
        InfoString::parse(info)
            .words
            .iter()
            .filter_map(|w| w.pair.as_ref())
            .map(|p| (p.key.to_owned(), p.value.to_string()))
            .collect()
    }

    #[test]
    fn language_then_pairs_in_any_order() {
        let info = InfoString::parse(r#"json title="Create a user" request="../api.yaml#create""#);
        assert_eq!(info.language, Some("json"));
        assert_eq!(info.get("title"), Some("Create a user"));
        assert_eq!(info.get("request"), Some("../api.yaml#create"));
        let info = InfoString::parse(r#"json request="../api.yaml#create" title="Create""#);
        assert_eq!(info.get("title"), Some("Create"));
    }

    #[test]
    fn a_pair_first_means_no_language() {
        let info = InfoString::parse("phrases=true");
        assert_eq!(info.language, None);
        assert!(info.phrases());
    }

    #[test]
    fn quoted_values_take_the_attribute_escapes() {
        assert_eq!(
            pairs(r#"text title="say \"hi\" now" tested="C:\\dir\x""#),
            [
                ("title".into(), r#"say "hi" now"#.into()),
                ("tested".into(), r"C:\dir\x".into())
            ]
        );
    }

    #[test]
    fn a_word_inside_a_quoted_value_is_not_a_pair() {
        let info = InfoString::parse(r#"yaml title="use phrases=true here""#);
        assert!(!info.phrases());
        assert_eq!(info.words.len(), 1);
    }

    #[test]
    fn spaces_around_the_equals_sign_make_no_pair() {
        let info = InfoString::parse("yaml phrases = true");
        assert!(!info.phrases());
        assert!(info.words.iter().all(|w| w.pair.is_none()));
    }

    #[test]
    fn words_that_are_not_pairs_are_kept() {
        for word in [
            r#"title="unclosed"#,
            r#"title="a"b"#,
            "Title=x",
            "key=",
            r#"key=a"b"#,
            "{1,3}",
            "showLineNumbers",
        ] {
            let text = format!("ts {word}");
            let info = InfoString::parse(&text);
            assert!(
                info.words.iter().all(|w| w.pair.is_none()),
                "{word} is not a pair"
            );
        }
        let info = InfoString::parse(r#"ts title="unclosed and more"#);
        assert_eq!(
            info.words.iter().map(|w| w.text).collect::<Vec<_>>(),
            [r#"title="unclosed"#, "and", "more"]
        );
    }

    #[test]
    fn the_first_pair_with_a_key_is_read() {
        assert_eq!(
            InfoString::parse("x phrases=false phrases=true").get("phrases"),
            Some("false")
        );
    }

    #[test]
    fn without_drops_pairs_and_keeps_the_rest() {
        let info = InfoString::parse(
            "json   title=\"A b\" request=\"../a.yaml#c\" {1}\tphrases=true tested=\"Node 24\"",
        );
        assert_eq!(
            info.without(&["request", "phrases", "tested"]),
            "json title=\"A b\" {1}"
        );
    }

    #[test]
    fn spans_point_into_the_text() {
        let text = r#"json response="../a.yaml#c 201""#;
        let info = InfoString::parse(text);
        let word = &info.words[0];
        assert_eq!(word.start, 5);
        let pair = word.pair.as_ref().expect("a pair");
        assert_eq!(
            &text[pair.value_start..pair.value_start + pair.value_len],
            "../a.yaml#c 201"
        );
    }

    #[test]
    fn quote_round_trips() {
        for value in ["plain", r#"say "hi" now"#, r"C:\dir\", "是非"] {
            let info = format!("text title={}", quote(value));
            assert_eq!(InfoString::parse(&info).get("title"), Some(value));
        }
    }
    /// The near misses of `info`: what each was meant as, what's wrong, and
    /// the pair it was meant to be.
    fn misses(info: &str) -> Vec<(&str, &'static str, Option<String>)> {
        near_misses(info)
            .into_iter()
            .map(|m| (&info[m.start..m.end], m.problem.as_str(), m.fixed))
            .collect()
    }

    #[test]
    fn near_misses_say_what_was_meant() {
        let cases: &[(&str, &str, &str, Option<&str>)] = &[
            (
                "ts phrases = true",
                "phrases = true",
                "spaced",
                Some("phrases=true"),
            ),
            (
                r#"ts title= "x""#,
                r#"title= "x""#,
                "spaced",
                Some(r#"title="x""#),
            ),
            ("ts Title=x", "Title=x", "case", Some("title=x")),
            (
                r#"ts TESTED = "Node""#,
                r#"TESTED = "Node""#,
                "case",
                Some(r#"tested="Node""#),
            ),
            (
                r#"ts title="connect.ts"#,
                r#"title="connect.ts"#,
                "unclosed",
                Some(r#"title="connect.ts""#),
            ),
            (
                r#"ts title="say "hi"""#,
                r#"title="say "hi"""#,
                "quote",
                Some(r#"title="say \"hi\"""#),
            ),
            (
                r#"ts title=a"b"#,
                r#"title=a"b"#,
                "quote",
                Some(r#"title="a\"b""#),
            ),
            ("ts title=", "title=", "other", None),
            ("ts phrases= tested=x", "phrases=", "spaced", None),
        ];
        for (info, meant, problem, fixed) in cases {
            assert_eq!(
                misses(info),
                [(*meant, *problem, fixed.map(str::to_owned))],
                "{info}"
            );
        }
    }

    #[test]
    fn a_near_miss_ends_before_the_next_pair() {
        assert_eq!(
            misses(r#"ts title="say "hi" now" phrases=true"#),
            [(
                r#"title="say "hi" now""#,
                "quote",
                Some(r#"title="say \"hi\" now""#.to_owned())
            )]
        );
        assert_eq!(
            misses(r#"ts title="connect.ts phrases = true"#),
            [
                (
                    r#"title="connect.ts"#,
                    "unclosed",
                    Some(r#"title="connect.ts""#.to_owned())
                ),
                ("phrases = true", "spaced", Some("phrases=true".to_owned())),
            ]
        );
        // A quoted value that closes is right, whatever is inside it.
        assert_eq!(
            misses(r#"ts Title="use title=x" {1,3} "y""#),
            [(
                r#"Title="use title=x""#,
                "case",
                Some(r#"title="use title=x""#.to_owned())
            )]
        );
    }

    #[test]
    fn other_words_are_no_near_misses() {
        for info in [
            r#"ts {1,3} showLineNumbers titles=x foo=bar Foo=bar title"#,
            r#"ts title="a" phrases=true"#,
            r#"ts title="use phrases = true""#,
            // The first word is the language, whatever it looks like.
            "Title=x",
        ] {
            assert_eq!(misses(info), [], "{info}");
        }
    }

    #[test]
    fn the_keys_are_the_specs() {
        let spec = include_str!("../../../SPEC.md");
        let section = spec
            .split("#### 3.3.1 ")
            .nth(1)
            .and_then(|s| s.split("\n### ").next())
            .expect("SPEC §3.3.1");
        let keys: Vec<&str> = section
            .lines()
            .filter_map(|l| l.strip_prefix("| `"))
            .filter_map(|l| l.split('`').next())
            .collect();
        assert_eq!(keys, KEYS);
    }
}
