//! The directive-head parser (SPEC §3.1, Appendix A `directive-line`).
//!
//! The head of a directive line is everything on its first line:
//! `@name {attributes}: primary`. The block parser in `comrak-ascribe`
//! recognizes the line (a known keyword after `@`) and, for keywords with a
//! text primary, finds where that primary starts. This module parses the
//! rest: the attribute block, the colon, and where a primary begins, all
//! with exact spans. It never panics on any input.
//!
//! The two parsers must agree on where a text primary starts. They do by
//! construction: both find the end of an attribute block with the rule in
//! `ascribe_core::attributes`, and both skip spaces and tabs around the
//! colon. A property test (`tests/all/agreement.rs`) checks it against the fork
//! on arbitrary lines.

use ascribe_core::{FileId, ParsedAttributes, parse_attribute_block};

/// The parsed head of one directive line. Offsets are into the line's raw
/// text, which starts at the `@`; attribute spans are in file coordinates.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Head {
    /// Where the name ends: just past its last character.
    pub name_end: usize,
    /// The attribute block, if the line has one.
    pub attributes: Option<ParsedAttributes>,
    /// Where the block starts and ends in the raw text.
    pub attributes_range: Option<(usize, usize)>,
    /// The offset of the colon.
    pub colon: Option<usize>,
    /// Where a non-empty primary starts (after spaces and tabs).
    pub primary_start: Option<usize>,
    /// The end of the line's content, without trailing spaces and tabs.
    pub content_end: usize,
    /// Where text that fits nothing in the head starts, if any.
    pub unexpected_start: Option<usize>,
}

/// Parses the head of `raw`, the line from `@` to its end without the line
/// ending. `base` is the file offset of the `@`.
pub(crate) fn parse_head(raw: &str, base: usize, file: FileId) -> Head {
    let bytes = raw.as_bytes();
    let content_end = raw.trim_end_matches([' ', '\t']).len();
    // Names are lowercase letters, digits, and hyphens (Appendix A `name`).
    let name_end = 1 + bytes
        .get(1..)
        .unwrap_or_default()
        .iter()
        .take_while(|&&b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        .count();
    let mut head = Head {
        name_end,
        attributes: None,
        attributes_range: None,
        colon: None,
        primary_start: None,
        content_end,
        unexpected_start: None,
    };

    let mut i = skip_ows(bytes, name_end, content_end);
    if raw.as_bytes().get(i) == Some(&b'{') && i < content_end {
        // Trailing whitespace can't close a block, so it's left out; an
        // unclosed block then ends where the line's content does.
        let text = &raw[i..content_end];
        if let Some(parsed) = parse_attribute_block(text, base + i, file) {
            let end = i + parsed.len;
            let closed = parsed.closed;
            head.attributes_range = Some((i, end));
            head.attributes = Some(parsed);
            if !closed {
                // The rest of the line belongs to the unclosed block.
                return head;
            }
            i = skip_ows(bytes, end, content_end);
        }
    }

    match bytes.get(i) {
        _ if i >= content_end => {}
        Some(b':') => {
            head.colon = Some(i);
            let after = skip_ows(bytes, i + 1, content_end);
            if after < content_end {
                head.primary_start = Some(after);
            }
        }
        Some(_) => head.unexpected_start = Some(i),
        None => {}
    }
    head
}

/// `key=value` pairs written where an attribute block goes, without its
/// braces, as in the `type=warning` of `@note type=warning: text`.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BarePairs<'a> {
    /// Each pair's key and value, the value as written (quotes included).
    pub pairs: Vec<(&'a str, &'a str)>,
    /// How many bytes of the text the pairs cover, through the last value.
    pub len: usize,
}

/// The `key=value` pairs at the start of `text`, separated by spaces or
/// commas, if it starts with one. An unquoted value ends at whitespace, a
/// comma, or a `:` that ends the text or has whitespace after it, so the
/// colon before a primary isn't taken as part of the value.
pub(crate) fn bare_pairs(text: &str) -> Option<BarePairs<'_>> {
    let bytes = text.as_bytes();
    let mut pairs = Vec::new();
    let mut len = 0;
    let mut i = 0;
    loop {
        let key_end = i + bytes
            .get(i..)
            .unwrap_or_default()
            .iter()
            .take_while(|&&b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            .count();
        if !bytes.get(i).is_some_and(u8::is_ascii_lowercase) || bytes.get(key_end) != Some(&b'=') {
            break;
        }
        let start = key_end + 1;
        let end = if bytes.get(start) == Some(&b'"') {
            let mut j = start + 1;
            loop {
                match bytes.get(j) {
                    None => return None,
                    Some(b'\\') => j += 2,
                    Some(b'"') => break j + 1,
                    Some(_) => j += 1,
                }
            }
        } else {
            let mut j = start;
            while let Some(&b) = bytes.get(j) {
                let ends_value = match b {
                    b' ' | b'\t' | b',' | b'{' | b'}' | b'"' | b'=' => true,
                    b':' => matches!(bytes.get(j + 1), None | Some(b' ' | b'\t')),
                    _ => false,
                };
                if ends_value {
                    break;
                }
                j += 1;
            }
            j
        };
        if end == start {
            break;
        }
        pairs.push((&text[i..key_end], &text[start..end]));
        len = end;
        i = skip_ows(bytes, end, bytes.len());
        if bytes.get(i) == Some(&b',') {
            i = skip_ows(bytes, i + 1, bytes.len());
        }
    }
    (!pairs.is_empty()).then_some(BarePairs { pairs, len })
}

fn skip_ows(bytes: &[u8], mut i: usize, end: usize) -> usize {
    while i < end && matches!(bytes.get(i), Some(b' ' | b'\t')) {
        i += 1;
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;

    fn head(raw: &str) -> Head {
        parse_head(raw, 0, FileId::new(0))
    }

    #[test]
    fn splits_the_head_into_its_parts() {
        let h = head("@note {type=caution}: Back up first");
        assert_eq!(h.name_end, 5);
        assert_eq!(h.attributes_range, Some((6, 20)));
        assert_eq!(h.colon, Some(20));
        assert_eq!(h.primary_start, Some(22));
        assert_eq!(h.content_end, 35);
        assert_eq!(h.unexpected_start, None);
    }

    #[test]
    fn accepts_any_spacing() {
        for raw in [
            "@note{type=tip}:x",
            "@note {type=tip}:x",
            "@note\t{type=tip}\t:\tx",
            "@note   {type=tip}   :   x",
        ] {
            let h = head(raw);
            assert_eq!(h.primary_start, Some(raw.len() - 1), "{raw:?}");
            assert!(h.attributes.is_some());
        }
        let h = head("@note :x");
        assert_eq!(h.colon, Some(6));
        assert_eq!(h.primary_start, Some(7));
    }

    #[test]
    fn a_colon_with_nothing_after_it_has_no_primary() {
        for raw in ["@note:", "@note:  \t", "@note {type=tip}:  ", "@note :"] {
            let h = head(raw);
            assert!(h.colon.is_some(), "{raw:?}");
            assert_eq!(h.primary_start, None, "{raw:?}");
        }
    }

    #[test]
    fn a_primary_ending_in_a_colon_is_still_a_primary() {
        let raw = "@note: Important:";
        let h = head(raw);
        assert_eq!(h.colon, Some(5));
        assert_eq!(h.primary_start, Some(7));
        assert_eq!(&raw[7..h.content_end], "Important:");
    }

    #[test]
    fn quoted_strings_hide_colons_and_braces() {
        let raw = r#"@note {label="a: } \" b"}: text"#;
        let h = head(raw);
        assert_eq!(&raw[h.primary_start.unwrap()..], "text");
    }

    #[test]
    fn keeps_text_that_fits_nothing() {
        let h = head("@note hello: text");
        assert_eq!(h.unexpected_start, Some(6));
        assert_eq!(h.colon, None);
        assert_eq!(h.primary_start, None);
        let h = head("@steps foo");
        assert_eq!(h.unexpected_start, Some(7));
        let h = head("@note {type=tip} extra");
        assert_eq!(h.unexpected_start, Some(17));
    }

    #[test]
    fn an_unclosed_block_takes_the_rest_of_the_line() {
        let h = head("@note {type=tip: text");
        assert!(h.attributes.is_some_and(|a| !a.closed));
        assert_eq!(h.colon, None);
        assert_eq!(h.primary_start, None);
    }

    #[test]
    fn finds_attributes_written_without_braces() {
        let p = bare_pairs("type=warning: You need Node 20.").unwrap();
        assert_eq!(p.pairs, [("type", "warning")]);
        assert_eq!(p.len, 12);
        let text = r#"type=tip, label="A: b" rest"#;
        let p = bare_pairs(text).unwrap();
        assert_eq!(p.pairs, [("type", "tip"), ("label", r#""A: b""#)]);
        assert_eq!(&text[..p.len], r#"type=tip, label="A: b""#);
        let p = bare_pairs("href=https://x.dev/a: text").unwrap();
        assert_eq!(p.pairs, [("href", "https://x.dev/a")]);
        let p = bare_pairs("type=warning:").unwrap();
        assert_eq!(p.len, 12);
        for text in [
            "hello: text",
            "foo",
            "type= x",
            "Type=x",
            "=x",
            r#"label="open"#,
        ] {
            assert_eq!(bare_pairs(text), None, "{text:?}");
        }
    }

    #[test]
    fn bare_and_trailing_whitespace_lines() {
        let h = head("@end  \t");
        assert_eq!(h.name_end, 4);
        assert_eq!(h.content_end, 4);
        assert_eq!(h.colon, None);
        assert_eq!(h.unexpected_start, None);
        let h = head("@");
        assert_eq!(h.name_end, 1);
    }
}
