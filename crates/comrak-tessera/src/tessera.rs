//! TESSERA: Tessera's additions to comrak. This whole file is Tessera's; see
//! `FORK.md`.
//!
//! Tessera (see the repository's `SPEC.md`) adds one block to CommonMark: the
//! **Tessera line**, a directive line (`@note {type=caution}: text`) or an end
//! line (`@end`). This module holds its option, its node, and the scanner that
//! recognizes it. The parser hooks that call the scanner are in
//! `parser/tessera.rs`.
//!
//! The fork only finds Tessera lines and gives them the right block structure.
//! It doesn't parse the directive head (the name's attributes, form, and
//! primary kind); `tessera-syntax` does that from [`NodeTesseraLine::raw`].
//!
//! Doctests are off in this crate (see `FORK.md`), so this example is also a
//! test, `module_example` in `tests/spike.rs`.
//!
//! ```
//! use std::sync::Arc;
//! use comrak_tessera::{Arena, Options, parse_document};
//! use comrak_tessera::nodes::NodeValue;
//! use comrak_tessera::tessera::TesseraOptions;
//!
//! let mut options = Options::default();
//! options.extension.tessera = Some(Arc::new(
//!     TesseraOptions::new().keyword("note", true).keyword("end", false),
//! ));
//!
//! let arena = Arena::new();
//! let root = parse_document(&arena, "Intro text\n@note: Careful.\n", &options);
//! let second = root.last_child().unwrap();
//! match &second.data().value {
//!     NodeValue::TesseraLine(line) => assert_eq!(line.raw, "@note: Careful."),
//!     other => panic!("expected a Tessera line, got {other:?}"),
//! }
//! ```

use std::collections::HashMap;
use std::fmt::{self, Write};

use crate::html::{ChildRendering, Context, render_sourcepos};
use crate::nodes::Node;

/// The Tessera option: the known directive keywords.
///
/// A line is a Tessera line only when its name is one of these keywords
/// (SPEC §3.2). The set is closed per project: the built-in directives, `end`,
/// and the project widgets the content model declares. The fork doesn't know
/// the built-in set; the caller supplies every keyword.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TesseraOptions {
    keywords: HashMap<String, TesseraKeyword>,
}

/// What the block parser needs to know about one directive keyword.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TesseraKeyword {
    /// Whether the directive takes a text primary (SPEC §3.4). A text primary
    /// continues onto following lines as a paragraph does, so the parser needs
    /// to know which keywords have one.
    pub text_primary: bool,
}

impl TesseraOptions {
    /// An empty keyword set. With no keywords, no line is a Tessera line.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a keyword, replacing any earlier entry for the same name, and
    /// returns the options.
    pub fn keyword(mut self, name: impl Into<String>, text_primary: bool) -> Self {
        self.insert(name, text_primary);
        self
    }

    /// Adds a keyword, replacing any earlier entry for the same name.
    pub fn insert(&mut self, name: impl Into<String>, text_primary: bool) {
        self.keywords
            .insert(name.into(), TesseraKeyword { text_primary });
    }

    /// Looks up a keyword by name, without the `@`.
    pub fn get(&self, name: &str) -> Option<TesseraKeyword> {
        self.keywords.get(name).copied()
    }

    /// The number of known keywords.
    pub fn len(&self) -> usize {
        self.keywords.len()
    }

    /// Whether no keywords are known.
    pub fn is_empty(&self) -> bool {
        self.keywords.is_empty()
    }
}

/// A Tessera line: a directive line or an end line (SPEC §3.1).
///
/// The node's sourcepos covers the whole line, and, when the directive has a
/// text primary, every line the primary continues onto. When there's a text
/// primary, the node has exactly one child, a [`Paragraph`] holding the
/// primary's inline content from its first character to the end of its last
/// line. Otherwise the node has no children.
///
/// [`Paragraph`]: crate::nodes::NodeValue::Paragraph
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NodeTesseraLine {
    /// The line from its `@` to the end of the line, not including the line
    /// ending. Container indentation and blockquote markers aren't included;
    /// the node's sourcepos gives the column of the `@`.
    pub raw: String,

    /// The directive's name: the keyword after `@`, such as `note` or `end`.
    pub name: String,

    /// When the line has a non-empty text primary, the byte offset in
    /// [`raw`](Self::raw) where it starts.
    pub text_primary: Option<usize>,
}

/// The result of recognizing a Tessera line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ScannedLine {
    /// Where the name ends: the byte after its last character.
    pub name_end: usize,
    /// Where the line's content ends, before the line ending.
    pub content_end: usize,
    /// Where a non-empty text primary starts, if the keyword takes one and
    /// the line has one.
    pub text_primary: Option<usize>,
}

/// Recognizes a Tessera line (SPEC §3.2). `line` starts at the line's first
/// non-space character, after container indentation, and may end with a line
/// ending.
///
/// A line is a Tessera line when it's `@`, a known keyword, and then
/// whitespace, `{`, `:`, or the end of the line.
pub(crate) fn scan_line(line: &str, options: &TesseraOptions) -> Option<ScannedLine> {
    let bytes = line.as_bytes();
    if bytes.first() != Some(&b'@') {
        return None;
    }

    // Names are lowercase letters, digits, and hyphens (Appendix A, `name`).
    // Taking every such byte means `@note-x` is looked up as `note-x`, never
    // as `note` followed by `-x`.
    let name_end = 1 + bytes[1..]
        .iter()
        .take_while(|&&b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        .count();
    let keyword = options.get(&line[1..name_end])?;

    match bytes.get(name_end) {
        None | Some(b' ' | b'\t' | b'{' | b':' | b'\r' | b'\n') => {}
        Some(_) => return None,
    }

    let content_end = line.trim_end_matches(['\r', '\n']).len();
    let text_primary = if keyword.text_primary {
        find_text_primary(&bytes[..content_end], name_end)
    } else {
        None
    };

    Some(ScannedLine {
        name_end,
        content_end,
        text_primary,
    })
}

/// Finds where a non-empty primary starts: after the optional attribute block
/// and the `:`, skipping spaces and tabs. Returns `None` for a line with no
/// `:`, an empty primary (a container opener, SPEC §3.5), or a head that can't
/// be read, such as an unclosed attribute block. `tessera-syntax` reports
/// malformed heads; here they just get no primary.
fn find_text_primary(bytes: &[u8], mut i: usize) -> Option<usize> {
    let skip_ows = |i: &mut usize| {
        while matches!(bytes.get(*i), Some(b' ' | b'\t')) {
            *i += 1;
        }
    };

    skip_ows(&mut i);
    if bytes.get(i) == Some(&b'{') {
        i = skip_attributes(bytes, i)?;
        skip_ows(&mut i);
    }
    if bytes.get(i) != Some(&b':') {
        return None;
    }
    i += 1;
    skip_ows(&mut i);
    (i < bytes.len()).then_some(i)
}

/// Skips an attribute block starting at the `{` at `start`, honoring quoted
/// strings and their `\"` and `\\` escapes (SPEC §3.3). Returns the offset
/// after the closing `}`, or `None` if the block isn't closed on this line.
fn skip_attributes(bytes: &[u8], start: usize) -> Option<usize> {
    let mut i = start + 1;
    let mut quoted = false;
    while let Some(&b) = bytes.get(i) {
        match (quoted, b) {
            (true, b'\\') => i += 1,
            (_, b'"') => quoted = !quoted,
            (false, b'}') => return Some(i + 1),
            _ => {}
        }
        i += 1;
    }
    None
}

/// Renders a Tessera line as HTML.
///
/// This output exists only so comrak's HTML renderer handles every node; the
/// real output comes from `tessera-emit`. A Tessera line becomes a `div`
/// carrying the raw line, wrapping the primary's paragraph if there is one.
pub(crate) fn render_html<T>(
    context: &mut Context<T>,
    node: Node<'_>,
    entering: bool,
    ntl: &NodeTesseraLine,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.cr()?;
        context.write_str("<div data-tessera-line=\"")?;
        context.escape(&ntl.raw)?;
        context.write_str("\"")?;
        render_sourcepos(context, node)?;
        context.write_str(">")?;
    } else {
        context.cr()?;
        context.write_str("</div>")?;
        context.lf()?;
    }
    Ok(ChildRendering::HTML)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options() -> TesseraOptions {
        TesseraOptions::new()
            .keyword("note", true)
            .keyword("include", false)
            .keyword("end", false)
            .keyword("quill-demo", true)
    }

    fn scan(line: &str) -> Option<ScannedLine> {
        scan_line(line, &options())
    }

    fn primary(line: &str) -> Option<&str> {
        let s = scan(line)?;
        s.text_primary.map(|p| &line[p..s.content_end])
    }

    #[test]
    fn recognizes_known_keywords_followed_by_the_right_characters() {
        for line in [
            "@note",
            "@note\n",
            "@note\r\n",
            "@note ",
            "@note\t{type=tip}",
            "@note{type=tip}",
            "@note:",
            "@note: text",
            "@end",
            "@quill-demo: text",
            "@note hello",
        ] {
            assert!(scan(line).is_some(), "{line:?} should be a Tessera line");
        }
    }

    #[test]
    fn leaves_other_lines_alone() {
        for line in [
            "note",
            " @note",
            "@unknown: text",
            "@astrojs/react",
            "@timestamp",
            "@Note: text",
            "@note-x: text",
            "@note.",
            "@notes",
            "@",
            "@: text",
            "\\@note: text",
            "support@example.com",
        ] {
            assert!(scan(line).is_none(), "{line:?} shouldn't be a Tessera line");
        }
    }

    #[test]
    fn finds_the_text_primary() {
        assert_eq!(primary("@note: text"), Some("text"));
        assert_eq!(primary("@note:text"), Some("text"));
        assert_eq!(primary("@note \t:  text  \n"), Some("text  "));
        assert_eq!(
            primary("@note {type=caution}: Back up your database\n"),
            Some("Back up your database")
        );
        assert_eq!(primary("@note{type=caution}:x"), Some("x"));
        assert_eq!(primary("@note: Important:"), Some("Important:"));
        assert_eq!(
            primary(r#"@note {label="a: } \" b"}: text"#),
            Some("text"),
            "quoted strings hide `:`, `}}`, and escaped quotes"
        );
    }

    #[test]
    fn has_no_text_primary_without_a_non_empty_one() {
        // Container openers.
        assert_eq!(primary("@note:"), None);
        assert_eq!(primary("@note {type=tip}:  \t\n"), None);
        // Line form with no primary.
        assert_eq!(primary("@note"), None);
        assert_eq!(primary("@note {type=tip}"), None);
        // Keywords without a text primary.
        assert_eq!(primary("@include: guides/setup.md"), None);
        assert_eq!(primary("@end"), None);
        // Heads that can't be read.
        assert_eq!(primary("@note {type=tip: text"), None);
        assert_eq!(primary(r#"@note {label="unclosed}: text"#), None);
        assert_eq!(primary("@note hello: text"), None);
    }

    #[test]
    fn records_where_the_name_and_content_end() {
        let s = scan("@note {type=tip}: x\r\n").unwrap();
        assert_eq!(s.name_end, 5);
        assert_eq!(s.content_end, 19);
    }
}
