//! Inline markup in frontmatter fields: `inline = "code"` (SPEC §7.2).
//!
//! A string field that sets `inline = "code"` is read as inline Markdown
//! that knows only code spans, so a page title can be "`ascribe.toml`
//! reference". Code spans follow CommonMark: a run of backticks opens one,
//! and the next run of the same length closes it. Outside them, a backslash
//! escapes ASCII punctuation, as it does in CommonMark, so `` \` `` is a
//! literal backtick; everything else, emphasis and links included, is
//! literal text.
//!
//! [`parse`] splits a value into [`Segment`]s, substituting phrases in the
//! text between code spans (never inside them, as in prose, SPEC §5.1), and
//! [`plain_text`] joins them back without the markup: the form for a page's
//! `<title>`, search, and sorting.

/// The inline markup a field's value is read with.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InlineMarkup {
    /// Code spans only.
    Code,
}

impl InlineMarkup {
    /// The setting's value in `ascribe.toml`.
    pub fn name(self) -> &'static str {
        match self {
            InlineMarkup::Code => "code",
        }
    }

    /// The markup a setting's value names, if any.
    pub fn from_name(name: &str) -> Option<InlineMarkup> {
        match name {
            "code" => Some(InlineMarkup::Code),
            _ => None,
        }
    }
}

/// A piece of a formatted value.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Segment {
    /// Text, with escapes applied and phrases substituted.
    Text(String),
    /// A code span's content, as CommonMark normalizes it.
    Code(String),
}

impl Segment {
    /// The segment's text, without markup.
    pub fn text(&self) -> &str {
        match self {
            Segment::Text(t) | Segment::Code(t) => t,
        }
    }
}

/// Splits `value` into text and code spans. `phrase` gives a declared
/// phrase's value by key; it's asked only for `{key}` in text, so a field
/// that doesn't take phrases passes `|_| None`.
pub fn parse(value: &str, phrase: &dyn Fn(&str) -> Option<String>) -> Vec<Segment> {
    let bytes = value.as_bytes();
    let mut out = Vec::new();
    let mut text = String::new();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' if bytes.get(i + 1).is_some_and(u8::is_ascii_punctuation) => {
                text.push(char::from(bytes[i + 1]));
                i += 2;
            }
            b'`' => {
                let n = run(bytes, i);
                match closer(bytes, i + n, n) {
                    Some(close) => {
                        if !text.is_empty() {
                            out.push(Segment::Text(std::mem::take(&mut text)));
                        }
                        out.push(Segment::Code(normalize(&value[i + n..close])));
                        i = close + n;
                    }
                    None => {
                        text.push_str(&value[i..i + n]);
                        i += n;
                    }
                }
            }
            b'{' => {
                let after = &value[i + 1..];
                match after
                    .find('}')
                    .and_then(|close| phrase(&after[..close]).map(|v| (close, v)))
                {
                    Some((close, v)) => {
                        text.push_str(&v);
                        i += close + 2;
                    }
                    None => {
                        text.push('{');
                        i += 1;
                    }
                }
            }
            _ => {
                let ch = value[i..].chars().next().unwrap_or_default();
                text.push(ch);
                i += ch.len_utf8().max(1);
            }
        }
    }
    if !text.is_empty() {
        out.push(Segment::Text(text));
    }
    out
}

/// The segments' text, without markup.
pub fn plain_text(segments: &[Segment]) -> String {
    segments.iter().map(Segment::text).collect()
}

/// The length of the backtick run at `at`.
fn run(bytes: &[u8], at: usize) -> usize {
    bytes[at..].iter().take_while(|b| **b == b'`').count()
}

/// The start of the next run of exactly `n` backticks from `from`.
fn closer(bytes: &[u8], from: usize, n: usize) -> Option<usize> {
    let mut i = from;
    while i < bytes.len() {
        if bytes[i] == b'`' {
            let len = run(bytes, i);
            if len == n {
                return Some(i);
            }
            i += len;
        } else {
            i += 1;
        }
    }
    None
}

/// A code span's content as CommonMark gives it: line endings become spaces,
/// and one space is stripped from each end when both ends have one and the
/// content isn't all spaces.
fn normalize(content: &str) -> String {
    let joined = content.replace("\r\n", " ").replace(['\r', '\n'], " ");
    let b = joined.as_bytes();
    if b.len() >= 2 && b[0] == b' ' && b[b.len() - 1] == b' ' && b.iter().any(|c| *c != b' ') {
        joined[1..joined.len() - 1].to_owned()
    } else {
        joined
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn none(_: &str) -> Option<String> {
        None
    }

    fn code(s: &str) -> Segment {
        Segment::Code(s.into())
    }

    fn text(s: &str) -> Segment {
        Segment::Text(s.into())
    }

    #[test]
    fn code_spans() {
        assert_eq!(
            parse("`ascribe.toml` reference", &none),
            vec![code("ascribe.toml"), text(" reference")]
        );
        assert_eq!(
            parse("a ``b`c`` d", &none),
            vec![text("a "), code("b`c"), text(" d")]
        );
        assert_eq!(parse("`` `x` ``", &none), vec![code("`x`")]);
        assert_eq!(parse("`  `", &none), vec![code("  ")]);
        assert_eq!(parse("`\\`", &none), vec![code("\\")]);
    }

    #[test]
    fn unclosed_runs_and_escapes_are_text() {
        assert_eq!(parse("a `b", &none), vec![text("a `b")]);
        assert_eq!(parse("``a`", &none), vec![text("``a`")]);
        assert_eq!(parse("\\`a`", &none), vec![text("`a`")]);
        assert_eq!(parse("*a* [b](c) \\d", &none), vec![text("*a* [b](c) \\d")]);
    }

    #[test]
    fn phrases_only_in_text() {
        let phrase = |k: &str| (k == "p").then(|| "Quill".to_owned());
        assert_eq!(
            parse("{p} `{p}` \\{p}", &phrase),
            vec![text("Quill "), code("{p}"), text(" {p}")]
        );
        assert_eq!(plain_text(&parse("`a` {p}", &phrase)), "a Quill");
    }
}
