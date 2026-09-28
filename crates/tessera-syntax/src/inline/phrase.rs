//! Phrase candidates (SPEC §5.1).
//!
//! A candidate is `{key}` with a key from Appendix A's `key` rule: a
//! lowercase letter, then lowercase letters, digits, or `-`. Everything here
//! reads the *source*, because comrak's decoded text can't tell `\{key}` (an
//! escape, SPEC §2.3) or `&#123;key}` (an entity) from `{key}`.

use tessera_core::Span;

use super::Pass;
use crate::convert::matching_bracket;
use crate::tree::*;

/// One find in a run of source text.
enum Found {
    /// A candidate, `{key}`.
    Candidate(Phrase),
    /// An escaped one, `\{key}`; the span starts at the backslash.
    Escaped(Phrase),
}

/// Finds candidates in `text`, which starts at `base` in the file. With
/// `escapes`, a backslash before ASCII punctuation escapes it (CommonMark's
/// rule, which holds in text and in destinations, not in code).
fn scan(text: &str, base: usize, escapes: bool) -> Vec<Found> {
    let bytes = text.as_bytes();
    let mut found = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' if escapes && bytes.get(i + 1).is_some_and(u8::is_ascii_punctuation) => {
                if bytes[i + 1] == b'{'
                    && let Some(end) = candidate_end(bytes, i + 1)
                {
                    found.push(Found::Escaped(phrase(text, base, i, i + 1, end)));
                }
                i += 2;
            }
            b'{' => match candidate_end(bytes, i) {
                Some(end) => {
                    found.push(Found::Candidate(phrase(text, base, i, i, end)));
                    i = end;
                }
                None => i += 1,
            },
            _ => i += 1,
        }
    }
    found
}

/// A phrase whose text is `text[start..end]` and whose brace is at `brace`.
fn phrase(text: &str, base: usize, start: usize, brace: usize, end: usize) -> Phrase {
    Phrase {
        key: text[brace + 1..end - 1].to_owned(),
        key_span: Span::new(base + brace + 1, base + end - 1),
        span: Span::new(base + start, base + end),
    }
}

/// Where the candidate that starts at the `{` at `open` ends, if there is one.
fn candidate_end(bytes: &[u8], open: usize) -> Option<usize> {
    let mut i = open + 1;
    if !bytes.get(i)?.is_ascii_lowercase() {
        return None;
    }
    while bytes
        .get(i)
        .is_some_and(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'-')
    {
        i += 1;
    }
    (bytes.get(i) == Some(&b'}')).then_some(i + 1)
}

/// The decoded text of a piece of source: backslash escapes and entities, as
/// CommonMark reads them.
fn decode(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut out = String::new();
    let mut plain = 0;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' && bytes.get(i + 1).is_some_and(u8::is_ascii_punctuation) {
            out.push_str(&comrak_tessera::tessera::unescape_entities(&raw[plain..i]));
            out.push(char::from(bytes[i + 1]));
            i += 2;
            plain = i;
        } else {
            i += 1;
        }
    }
    out.push_str(&comrak_tessera::tessera::unescape_entities(&raw[plain..]));
    out.replace('\0', "\u{fffd}")
}

impl Pass<'_> {
    /// Splits `inline`, a text, around its candidates and pushes the pieces.
    pub(super) fn split_text(&mut self, inline: Inline, out: &mut Vec<Inline>) {
        let span = inline.span;
        let Some(text) = self.source.get(span.start()..span.end()) else {
            out.push(inline);
            return;
        };
        if !text.contains('{') {
            out.push(inline);
            return;
        }
        let mut at = 0;
        let mut split = false;
        for found in scan(text, span.start(), true) {
            match found {
                Found::Escaped(phrase) => self.escaped.push(phrase),
                Found::Candidate(phrase) => {
                    split = true;
                    let start = phrase.span.start() - span.start();
                    push_text(text, span.start(), at, start, out);
                    at = phrase.span.end() - span.start();
                    out.push(Inline {
                        span: phrase.span,
                        kind: InlineKind::Phrase(phrase),
                    });
                }
            }
        }
        if split {
            push_text(text, span.start(), at, text.len(), out);
        } else {
            out.push(inline);
        }
    }

    /// The candidates in the destination of the link or image at `span`, when
    /// it's an inline one, `[text](destination "title")`.
    pub(super) fn destination_phrases(&mut self, span: Span, form: LinkForm) -> Vec<Phrase> {
        if form != LinkForm::Inline {
            return Vec::new();
        }
        let Some(text) = self.source.get(span.start()..span.end()) else {
            return Vec::new();
        };
        let open = usize::from(text.starts_with('!'));
        let Some(dest) = matching_bracket(text, open).and_then(|close| destination(text, close))
        else {
            return Vec::new();
        };
        let mut phrases = Vec::new();
        let base = span.start() + dest.start;
        for found in scan(&text[dest.clone()], base, true) {
            match found {
                Found::Escaped(phrase) => self.escaped.push(phrase),
                Found::Candidate(phrase) => phrases.push(phrase),
            }
        }
        phrases
    }

    /// Records the candidates in a fence that opts in with `phrases=true`.
    // SPEC-QUESTION(Q42): backslashes don't escape in code, so `\{key}` is a
    // candidate here.
    pub(super) fn code_block(&mut self, span: Span, code: &mut CodeBlock) {
        if !code.fenced || !code.info.split_whitespace().any(|w| w == "phrases=true") {
            return;
        }
        let text = self.source.get(span.start()..span.end()).unwrap_or("");
        // The opening fence's line holds the info string, not code.
        let body = text.find(['\n', '\r']).unwrap_or(text.len());
        let phrases = scan(&text[body..], span.start() + body, false)
            .into_iter()
            .filter_map(|f| match f {
                Found::Candidate(phrase) => Some(phrase),
                Found::Escaped(_) => None,
            })
            .collect();
        code.phrases = Some(phrases);
    }
}

/// Pushes the text `text[from..to]` if it isn't empty.
fn push_text(text: &str, base: usize, from: usize, to: usize, out: &mut Vec<Inline>) {
    if from < to {
        out.push(Inline {
            span: Span::new(base + from, base + to),
            kind: InlineKind::Text(decode(&text[from..to])),
        });
    }
}

/// The range of the destination of the inline link whose text closes at
/// `close` (the `]`), as written: without the `<` and `>` of the pointy form.
fn destination(text: &str, close: usize) -> Option<std::ops::Range<usize>> {
    let bytes = text.as_bytes();
    if bytes.get(close + 1) != Some(&b'(') {
        return None;
    }
    let mut i = close + 2;
    while bytes.get(i).is_some_and(u8::is_ascii_whitespace) {
        i += 1;
    }
    let start;
    if bytes.get(i) == Some(&b'<') {
        i += 1;
        start = i;
        while let Some(&b) = bytes.get(i) {
            match b {
                b'\\' => i += 1,
                b'>' | b'\n' | b'\r' => break,
                _ => {}
            }
            i += 1;
        }
    } else {
        start = i;
        let mut depth = 0usize;
        while let Some(&b) = bytes.get(i) {
            match b {
                b'\\' => i += 1,
                b'(' => depth += 1,
                b')' if depth == 0 => break,
                b')' => depth -= 1,
                b if b.is_ascii_whitespace() => break,
                _ => {}
            }
            i += 1;
        }
    }
    let end = i.min(bytes.len());
    (start < end).then_some(start..end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_candidates_and_escapes() {
        let found = scan(r"a {b} \{c} {D} {e_f} \\{g} {}", 10, true);
        let kinds: Vec<_> = found
            .iter()
            .map(|f| match f {
                Found::Candidate(p) => ("candidate", p.key.as_str(), p.span),
                Found::Escaped(p) => ("escaped", p.key.as_str(), p.span),
            })
            .collect();
        assert_eq!(
            kinds,
            [
                ("candidate", "b", Span::new(12, 15)),
                ("escaped", "c", Span::new(16, 20)),
                ("candidate", "g", Span::new(33, 36)),
            ]
        );
    }

    #[test]
    fn keys_follow_the_grammar() {
        for (text, ok) in [
            ("{a}", true),
            ("{a-b2}", true),
            ("{2a}", false),
            ("{-a}", false),
            ("{A}", false),
            ("{a b}", false),
            ("{a=b}", false),
            ("{a", false),
        ] {
            assert_eq!(candidate_end(text.as_bytes(), 0).is_some(), ok, "{text}");
        }
    }

    #[test]
    fn decodes_like_commonmark() {
        assert_eq!(decode(r"a \* &amp; \&amp; &#35; \\"), r"a * & &amp; # \");
        assert_eq!(decode("plain"), "plain");
    }

    #[test]
    fn finds_destinations() {
        let dest = |text: &str| {
            let close = matching_bracket(text, 0).unwrap();
            destination(text, close).map(|r| text[r].to_owned())
        };
        assert_eq!(dest("[a]({api}x)").as_deref(), Some("{api}x"));
        assert_eq!(dest("[a](  {api}x \"t\")").as_deref(), Some("{api}x"));
        assert_eq!(dest("[a](<{api} x>)").as_deref(), Some("{api} x"));
        assert_eq!(dest("[a](x(y){z})").as_deref(), Some("x(y){z}"));
        assert_eq!(dest("[a]()"), None);
        assert_eq!(dest("[a][b]"), None);
    }
}
