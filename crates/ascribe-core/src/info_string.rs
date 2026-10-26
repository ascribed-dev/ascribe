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
}
