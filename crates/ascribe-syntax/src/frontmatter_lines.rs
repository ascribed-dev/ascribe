//! Frontmatter read line by line, as the editor reads it to find and change
//! what's written in a field without parsing the YAML: each line's key and
//! value, with where the value starts, its comment left out, and the lines
//! of a block scalar (`|` or `>`) kept as text.
//!
//! It follows frontmatter as it's written: `key: value` lines, nested by
//! indentation, `- item` lines, and block scalars. A value in a form this
//! doesn't follow, such as a quoted string over several lines, is read one
//! line at a time.

/// A line of frontmatter that isn't blank or a comment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontmatterLine<'a> {
    /// Where the line starts in the text read.
    pub start: usize,
    /// The column its content starts at: after the `- ` of an item.
    pub indent: usize,
    /// For an item, the column of its `-`.
    pub item: Option<usize>,
    /// Its key, unquoted; `None` for a line that's only a value, such as an
    /// item, a continuation, or a line of a block scalar.
    pub key: Option<&'a str>,
    /// Its value: what follows the key, or the whole content, without a
    /// comment or the whitespace around it. Empty when there's none.
    pub value: &'a str,
    /// Where the value starts in the text read.
    pub value_start: usize,
    /// Whether it's a line of a block scalar: its text is all value.
    pub block: bool,
}

/// The lines of `text`, a frontmatter's content, that aren't blank or
/// comments.
pub fn frontmatter_lines(text: &str) -> Vec<FrontmatterLine<'_>> {
    let mut out = Vec::new();
    // The indentation of the key whose block scalar the next lines may be in.
    let mut scalar: Option<usize> = None;
    let mut at = 0;
    for raw in text.split_inclusive('\n') {
        let start = at;
        at += raw.len();
        let line = raw.trim_end_matches(['\n', '\r']);
        let rest = line.trim_start();
        let indent = line.len() - rest.len();
        if let Some(owner) = scalar {
            if rest.is_empty() {
                continue;
            }
            if indent > owner {
                let value = rest.trim_end();
                out.push(FrontmatterLine {
                    start,
                    indent,
                    item: None,
                    key: None,
                    value,
                    value_start: start + indent,
                    block: true,
                });
                continue;
            }
            scalar = None;
        }
        if rest.is_empty() || rest.starts_with('#') {
            continue;
        }
        let (indent, rest, item) = match rest.strip_prefix('-') {
            Some(after) if after.is_empty() || after.starts_with([' ', '\t']) => {
                let after = after.trim_start();
                (line.len() - after.len(), after, Some(indent))
            }
            _ => (indent, rest, None),
        };
        let (key, value) = match key_value(rest) {
            Some((key, value)) => (Some(key), value),
            None => (None, rest),
        };
        let value = without_comment(value).trim_end();
        if key.is_some() && is_block_indicator(value) {
            scalar = Some(item.unwrap_or(indent));
        }
        out.push(FrontmatterLine {
            start,
            indent,
            item,
            key,
            value,
            value_start: start + offset_in(line, value),
            block: false,
        });
    }
    out
}

/// Where `part`, a slice of `line`, starts in it.
fn offset_in(line: &str, part: &str) -> usize {
    (part.as_ptr() as usize).saturating_sub(line.as_ptr() as usize)
}

/// `key: value`, with the key unquoted and the value's leading whitespace
/// left out.
fn key_value(line: &str) -> Option<(&str, &str)> {
    if line.starts_with(['"', '\'', '[', '{']) && !quoted_key(line) {
        return None;
    }
    let colon = line
        .find(": ")
        .or_else(|| line.strip_suffix(':').map(|k| k.len()))?;
    let key = line[..colon].trim();
    if key.is_empty() || key.contains('#') {
        return None;
    }
    let key = key.trim_matches(['"', '\'']);
    Some((key, line[colon + 1..].trim_start()))
}

/// Whether a line starting with a quote is a quoted key: `"a b": value`.
fn quoted_key(line: &str) -> bool {
    let Some(quote) = line.chars().next().filter(|c| matches!(c, '"' | '\'')) else {
        return false;
    };
    line[1..]
        .find(quote)
        .is_some_and(|close| line[close + 2..].starts_with(':'))
}

/// The text before a comment: a `#` at the start or after whitespace, outside
/// quotes.
fn without_comment(value: &str) -> &str {
    let mut quote = None;
    let mut previous = ' ';
    for (i, c) in value.char_indices() {
        match quote {
            Some(q) if c == q => quote = None,
            Some(_) => {}
            None if matches!(c, '"' | '\'')
                && (i == 0 || previous.is_whitespace() || matches!(previous, '[' | '{' | ',')) =>
            {
                quote = Some(c);
            }
            None if c == '#' && previous.is_whitespace() => return &value[..i],
            None if c == '#' && i == 0 => return &value[..0],
            None => {}
        }
        previous = c;
    }
    value
}

/// Whether a value starts a block scalar: `|` or `>`, with any indicators.
fn is_block_indicator(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some('|' | '>'))
        && chars.all(|c| matches!(c, '+' | '-') || c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(text: &str) -> Vec<(Option<&str>, &str, usize, bool)> {
        frontmatter_lines(text)
            .into_iter()
            .map(|l| {
                assert_eq!(&text[l.value_start..l.value_start + l.value.len()], l.value);
                (l.key, l.value, l.indent, l.block)
            })
            .collect()
    }

    #[test]
    fn keys_values_and_items() {
        assert_eq!(
            read(
                "title: Use it # a note\nvariant:\n  deployment: [a, b] # a\nkeywords:\n  - one\n  - \"two # not a comment\"\n# a comment\n"
            ),
            [
                (Some("title"), "Use it", 0, false),
                (Some("variant"), "", 0, false),
                (Some("deployment"), "[a, b]", 2, false),
                (Some("keywords"), "", 0, false),
                (None, "one", 4, false),
                (None, "\"two # not a comment\"", 4, false),
            ]
        );
    }

    #[test]
    fn a_block_scalar_is_text() {
        assert_eq!(
            read("title: |\n  Note: {product} # kept\n\n  More\ndescription: >-\n  x\nnext: y\n"),
            [
                (Some("title"), "|", 0, false),
                (None, "Note: {product} # kept", 2, true),
                (None, "More", 2, true),
                (Some("description"), ">-", 0, false),
                (None, "x", 2, true),
                (Some("next"), "y", 0, false),
            ]
        );
    }

    #[test]
    fn crlf_and_urls() {
        assert_eq!(
            read("link: https://x.dev/a#b\r\n\"a key\": 'v' # c\r\n"),
            [
                (Some("link"), "https://x.dev/a#b", 0, false),
                (Some("a key"), "'v'", 0, false),
            ]
        );
    }
}
