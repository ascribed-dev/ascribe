//! ASCRIBE: Ascribe's additions to comrak. This whole file is Ascribe's; see
//! `FORK.md`.
//!
//! Ascribe (see the repository's `SPEC.md`) adds one block to CommonMark: the
//! **Ascribe line**, a directive line (`@note {type=caution}: text`) or an end
//! line (`@end`). This module holds its option, its node, and the scanner that
//! recognizes it. The parser hooks that call the scanner are in
//! `parser/ascribe.rs`.
//!
//! The fork only finds Ascribe lines and gives them the right block structure.
//! It doesn't parse the directive head (the name's attributes, form, and
//! primary kind); `ascribe-syntax` does that from [`NodeAscribeLine::raw`].
//!
//! Doctests are off in this crate (see `FORK.md`), so this example is also a
//! test, `module_example` in `tests/spike.rs`.
//!
//! ```
//! use std::sync::Arc;
//! use comrak_ascribe::{Arena, Options, parse_document};
//! use comrak_ascribe::nodes::NodeValue;
//! use comrak_ascribe::ascribe::AscribeOptions;
//!
//! let mut options = Options::default();
//! options.extension.ascribe = Some(Arc::new(
//!     AscribeOptions::new().keyword("note", true).keyword("end", false),
//! ));
//!
//! let arena = Arena::new();
//! let root = parse_document(&arena, "Intro text\n@note: Careful.\n", &options);
//! let second = root.last_child().unwrap();
//! match &second.data().value {
//!     NodeValue::AscribeLine(line) => assert_eq!(line.raw, "@note: Careful."),
//!     other => panic!("expected an Ascribe line, got {other:?}"),
//! }
//! ```

use std::collections::HashMap;
use std::fmt::{self, Write};

use std::ops::Range;

use crate::html::{ChildRendering, Context, render_sourcepos};
use crate::nodes::{LineColumn, Node, Sourcepos};

/// The Ascribe option: the known directive keywords.
///
/// A line is an Ascribe line only when its name is one of these keywords
/// (SPEC §3.2). The set is closed per project: the built-in directives, `end`,
/// and the project widgets the content model declares. The fork doesn't know
/// the built-in set; the caller supplies every keyword.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AscribeOptions {
    keywords: HashMap<String, AscribeKeyword>,
}

/// What the block parser needs to know about one directive keyword.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AscribeKeyword {
    /// Whether the directive takes a text primary (SPEC §3.4). A text primary
    /// continues onto following lines as a paragraph does, so the parser needs
    /// to know which keywords have one.
    pub text_primary: bool,
}

impl AscribeOptions {
    /// An empty keyword set. With no keywords, no line is an Ascribe line.
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
            .insert(name.into(), AscribeKeyword { text_primary });
    }

    /// Looks up a keyword by name, without the `@`.
    pub fn get(&self, name: &str) -> Option<AscribeKeyword> {
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

/// An Ascribe line: a directive line or an end line (SPEC §3.1).
///
/// The node's sourcepos covers the whole line, and, when the directive has a
/// text primary, every line the primary continues onto. When there's a text
/// primary, the node has exactly one child, a [`Paragraph`] holding the
/// primary's inline content from its first character to the end of its last
/// line. Otherwise the node has no children.
///
/// [`Paragraph`]: crate::nodes::NodeValue::Paragraph
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NodeAscribeLine {
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

/// A link reference definition (`[label]: destination "title"`), which comrak
/// consumes while parsing and leaves out of the tree. Get them from
/// [`parse_document_with_definitions`](crate::parse_document_with_definitions).
///
/// Every position is a [`Sourcepos`] in comrak's convention: 1-based lines and
/// byte columns, the end being the last byte (inclusive). Every part is
/// non-empty. A definition can span several lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkDefinition {
    /// The whole definition, from the `[` through the last character of the
    /// title, or of the destination when there is no title.
    pub sourcepos: Sourcepos,
    /// The label as written, trimmed, between the brackets.
    pub label: Sourcepos,
    /// The label normalized as CommonMark matches labels (case folded, white
    /// space collapsed). The first definition of a label wins; later ones are
    /// parsed but never used.
    pub normalized_label: String,
    /// The destination as written, `<>` included.
    pub destination: Sourcepos,
    /// The destination with `<>` removed and escapes and entities decoded.
    pub url: String,
    /// The title as written, quotes or parentheses included, and its decoded
    /// text, if the definition has one.
    pub title: Option<(Sourcepos, String)>,
}

/// A definition's parts as offsets into the content the parser was reading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawDefinition {
    pub label: Range<usize>,
    pub normalized_label: String,
    pub destination: Range<usize>,
    pub url: String,
    pub title: Option<(Range<usize>, String)>,
}

/// Turns a [`RawDefinition`] into positions. `raw`'s offsets are relative to
/// `content[base..]`. `origin` is the line of the content's first line, and the
/// byte column, in the source, at which each of its lines starts (comrak's
/// `line_offsets`).
pub(crate) fn locate(
    content: &str,
    base: usize,
    raw: RawDefinition,
    origin: (usize, &[usize]),
) -> LinkDefinition {
    // The position of the byte at `offset` in `content`.
    let at = |offset: usize| {
        let mut line = 0;
        let mut line_start = 0;
        let bytes = content.as_bytes();
        let mut i = 0;
        while i < offset {
            match bytes[i] {
                b'\r' if bytes.get(i + 1) == Some(&b'\n') => {
                    i += 1;
                    line += 1;
                    line_start = i + 1;
                }
                b'\n' | b'\r' => {
                    line += 1;
                    line_start = i + 1;
                }
                _ => {}
            }
            i += 1;
        }
        LineColumn {
            line: origin.0 + line,
            column: origin.1.get(line).copied().unwrap_or(0) + (offset - line_start) + 1,
        }
    };
    let range = |r: &Range<usize>| Sourcepos {
        start: at(base + r.start),
        end: at(base + r.end - 1),
    };
    let end = raw.title.as_ref().map_or(&raw.destination, |(r, _)| r);
    LinkDefinition {
        sourcepos: Sourcepos {
            start: at(base), // the `[`
            end: at(base + end.end - 1),
        },
        label: range(&raw.label),
        normalized_label: raw.normalized_label,
        destination: range(&raw.destination),
        url: raw.url,
        title: raw.title.map(|(r, text)| (range(&r), text)),
    }
}

/// The result of recognizing an Ascribe line.
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

/// Recognizes an Ascribe line (SPEC §3.2). `line` starts at the line's first
/// non-space character, after container indentation, and may end with a line
/// ending.
///
/// A line is an Ascribe line when it's `@`, a known keyword, and then
/// whitespace, `{`, `:`, or the end of the line.
pub(crate) fn scan_line(line: &str, options: &AscribeOptions) -> Option<ScannedLine> {
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
/// be read, such as an unclosed attribute block. `ascribe-syntax` reports
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

/// The length of the attribute block directly after an image (SPEC §5.3), if
/// there is one. `rest` is the text right after the image. It starts with the
/// block's `{`; the block runs to the first `}` outside a quoted string (the
/// rule of [`skip_attributes`]) and must close on the same line.
///
/// Whatever is between the braces is the block: whether it's well-formed is
/// for `ascribe-syntax` to report. A `{` with no closing `}` on the line isn't
/// a block here, and stays text. The inline parser skips this many bytes after
/// an image, so the block's contents never take part in emphasis, links, or
/// code spans; `ascribe-syntax` calls it too, to find the same block.
pub fn image_attributes_len(rest: &str) -> Option<usize> {
    let bytes = rest.as_bytes();
    if bytes.first() != Some(&b'{') {
        return None;
    }
    let line_end = rest.find(['\r', '\n']).unwrap_or(rest.len());
    skip_attributes(&bytes[..line_end], 0)
}

/// Decodes the HTML entities (`&amp;`, `&#35;`) in `text` as CommonMark does.
/// `ascribe-syntax` uses it to work out the decoded value of a piece of text
/// from its source.
pub fn unescape_entities(text: &str) -> std::borrow::Cow<'_, str> {
    crate::entity::unescape_html(text)
}

/// Renders an Ascribe line as HTML.
///
/// This output exists only so comrak's HTML renderer handles every node; the
/// real output comes from `ascribe-emit`. An Ascribe line becomes a `div`
/// carrying the raw line, wrapping the primary's paragraph if there is one.
pub(crate) fn render_html<T>(
    context: &mut Context<T>,
    node: Node<'_>,
    entering: bool,
    ntl: &NodeAscribeLine,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.cr()?;
        context.write_str("<div data-ascribe-line=\"")?;
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

    #[test]
    fn image_attribute_blocks() {
        for (rest, len) in [
            ("{w=1}", Some(5)),
            ("{w=1} tail", Some(5)),
            ("{}", Some(2)),
            ("{key}", Some(5)),
            (r#"{a="}"} x"#, Some(7)),
            (r#"{a="\"}"}"#, Some(9)),
            ("{w=1", None),
            ("{w=1\n}", None),
            ("{a=\"}", None),
            (" {w=1}", None),
            ("w=1", None),
            ("", None),
        ] {
            assert_eq!(image_attributes_len(rest), len, "{rest:?}");
        }
    }

    fn options() -> AscribeOptions {
        AscribeOptions::new()
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
            assert!(scan(line).is_some(), "{line:?} should be an Ascribe line");
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
            assert!(
                scan(line).is_none(),
                "{line:?} shouldn't be an Ascribe line"
            );
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
