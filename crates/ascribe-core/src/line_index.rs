//! Converting byte offsets to lines and columns, and back.
//!
//! Ascribe works in UTF-8 byte offsets ([`Span`]). People and
//! protocols want lines and columns, counted in different units:
//!
//! | Consumer | Line | Column unit |
//! |---|---|---|
//! | Internal, and LSP clients that negotiate UTF-8 | 0-based | UTF-8 bytes ([`LineCol`]) |
//! | LSP's default position encoding | 0-based | UTF-16 code units ([`WideEncoding::Utf16`]) |
//! | `ascribe check` output and conformance cases | 1-based | Unicode scalar values ([`WideEncoding::Utf32`]), plus 1 |
//!
//! Lines end the way CommonMark ends them: at `\n`, at `\r\n`, or at a `\r`
//! not followed by `\n`. So line numbers here always agree with the parser's.

use serde::Serialize;

use crate::Span;

/// A 0-based line and a 0-based column counted in UTF-8 bytes from the start
/// of the line.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct LineCol {
    /// The 0-based line.
    pub line: u32,
    /// The 0-based column, in UTF-8 bytes.
    pub col: u32,
}

/// A unit for counting columns other than UTF-8 bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WideEncoding {
    /// UTF-16 code units: one per character in the Basic Multilingual Plane,
    /// two for characters outside it. LSP's default.
    Utf16,
    /// Unicode scalar values: one per `char`.
    Utf32,
}

/// A 0-based line and a 0-based column counted in a [`WideEncoding`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct WideLineCol {
    /// The 0-based line.
    pub line: u32,
    /// The 0-based column, in the encoding's units.
    pub col: u32,
}

/// One line: where it starts and where its text ends, before the line ending.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Line {
    start: usize,
    content_end: usize,
}

/// A character that isn't ASCII, located within its line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WideChar {
    /// Byte offset of the character from the start of its line.
    start: u32,
    /// The character's length in UTF-8 bytes: 2, 3, or 4.
    len_utf8: u32,
}

impl WideChar {
    fn len(self, enc: WideEncoding) -> u32 {
        match enc {
            WideEncoding::Utf16 if self.len_utf8 == 4 => 2,
            _ => 1,
        }
    }
}

/// Line starts and non-ASCII characters of one file's text, for converting
/// between byte offsets and line/column positions without keeping the text.
///
/// Build one per version of a file's text; it describes only that text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineIndex {
    len: usize,
    lines: Vec<Line>,
    /// For each line, its non-ASCII characters in order. Empty for ASCII lines.
    wide: Vec<Vec<WideChar>>,
}

impl LineIndex {
    /// Indexes `text`.
    pub fn new(text: &str) -> LineIndex {
        let mut lines = Vec::new();
        let mut wide = Vec::new();
        let mut line_start = 0;
        let mut line_wide = Vec::new();
        let mut chars = text.char_indices().peekable();
        while let Some((i, c)) = chars.next() {
            match c {
                '\n' | '\r' => {
                    let mut next = i + 1;
                    if c == '\r' && chars.peek().is_some_and(|&(_, n)| n == '\n') {
                        chars.next();
                        next += 1;
                    }
                    lines.push(Line {
                        start: line_start,
                        content_end: i,
                    });
                    wide.push(std::mem::take(&mut line_wide));
                    line_start = next;
                }
                c if !c.is_ascii() => {
                    // A line longer than u32::MAX bytes can't be addressed by
                    // LineCol anyway, so its characters past that point are
                    // simply not recorded.
                    if let Ok(start) = u32::try_from(i - line_start) {
                        line_wide.push(WideChar {
                            start,
                            len_utf8: c.len_utf8() as u32,
                        });
                    }
                }
                _ => {}
            }
        }
        lines.push(Line {
            start: line_start,
            content_end: text.len(),
        });
        wide.push(line_wide);
        LineIndex {
            len: text.len(),
            lines,
            wide,
        }
    }

    /// The length of the indexed text in bytes.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether the indexed text is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The number of lines. Text with no line endings has one line, and text
    /// ending in a line ending has an empty last line after it.
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// The text of a 0-based line, without its line ending.
    pub fn line_span(&self, line: u32) -> Option<Span> {
        let l = self.lines.get(line as usize)?;
        Some(Span::new(l.start, l.content_end))
    }

    /// The line and UTF-8 column of a byte offset.
    ///
    /// Returns `None` if the offset is past the end of the text or inside a
    /// multi-byte character. An offset inside a line ending (between the
    /// `\r` and `\n` of `\r\n`) gets a column past the line's text.
    pub fn line_col(&self, offset: usize) -> Option<LineCol> {
        if offset > self.len {
            return None;
        }
        let line = self.lines.partition_point(|l| l.start <= offset) - 1;
        let col = u32::try_from(offset - self.lines[line].start).ok()?;
        if self.wide[line]
            .iter()
            .any(|c| c.start < col && col < c.start + c.len_utf8)
        {
            return None;
        }
        Some(LineCol {
            line: u32::try_from(line).ok()?,
            col,
        })
    }

    /// The byte offset of a line and UTF-8 column.
    ///
    /// Returns `None` if the line doesn't exist, the column is past the end
    /// of the line's text, or the column is inside a multi-byte character.
    /// The column just past the line's text (where the line ending starts) is
    /// valid.
    pub fn offset(&self, pos: LineCol) -> Option<usize> {
        let line = self.lines.get(pos.line as usize)?;
        let offset = line.start + pos.col as usize;
        if offset > line.content_end {
            return None;
        }
        if self.wide[pos.line as usize]
            .iter()
            .any(|c| c.start < pos.col && pos.col < c.start + c.len_utf8)
        {
            return None;
        }
        Some(offset)
    }

    /// Converts a UTF-8 position to a column counted in `enc`.
    ///
    /// Returns `None` if the line doesn't exist, or the column is past the
    /// line's text or inside a multi-byte character.
    pub(crate) fn to_wide(&self, enc: WideEncoding, pos: LineCol) -> Option<WideLineCol> {
        self.offset(pos)?;
        let mut col = pos.col;
        for c in &self.wide[pos.line as usize] {
            if c.start >= pos.col {
                break;
            }
            col -= c.len_utf8 - c.len(enc);
        }
        Some(WideLineCol {
            line: pos.line,
            col,
        })
    }

    /// Converts a column counted in `enc` to a UTF-8 position.
    ///
    /// Returns `None` if the line doesn't exist, or the column is past the
    /// line's text or in the middle of a character (such as between the two
    /// UTF-16 code units of a character outside the Basic Multilingual Plane).
    pub(crate) fn to_utf8(&self, enc: WideEncoding, pos: WideLineCol) -> Option<LineCol> {
        let line = self.lines.get(pos.line as usize)?;
        let mut utf8 = 0u32;
        let mut wide = 0u32;
        let mut found = None;
        for c in &self.wide[pos.line as usize] {
            // The ASCII run before this character.
            let run = c.start - utf8;
            if pos.col <= wide + run {
                found = Some(utf8 + (pos.col - wide));
                break;
            }
            wide += run;
            utf8 = c.start;
            // The character itself.
            if pos.col < wide + c.len(enc) {
                return None;
            }
            wide += c.len(enc);
            utf8 += c.len_utf8;
        }
        let col = match found {
            Some(col) => col,
            None => utf8.checked_add(pos.col - wide)?,
        };
        if line.start + col as usize > line.content_end {
            return None;
        }
        Some(LineCol {
            line: pos.line,
            col,
        })
    }

    /// The position of a byte offset, with the column counted in `enc`.
    pub fn wide_line_col(&self, enc: WideEncoding, offset: usize) -> Option<WideLineCol> {
        self.to_wide(enc, self.line_col(offset)?)
    }

    /// The byte offset of a position whose column is counted in `enc`.
    pub fn wide_offset(&self, enc: WideEncoding, pos: WideLineCol) -> Option<usize> {
        self.offset(self.to_utf8(enc, pos)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lc(line: u32, col: u32) -> LineCol {
        LineCol { line, col }
    }

    fn wlc(line: u32, col: u32) -> WideLineCol {
        WideLineCol { line, col }
    }

    #[test]
    fn lines_end_at_lf_crlf_and_lone_cr() {
        let text = "a\nbc\r\nd\re";
        let idx = LineIndex::new(text);
        assert_eq!(idx.line_count(), 4);
        let lines: Vec<&str> = (0..4)
            .map(|l| &text[idx.line_span(l).unwrap().range()])
            .collect();
        assert_eq!(lines, ["a", "bc", "d", "e"]);
        assert_eq!(idx.line_col(2), Some(lc(1, 0)));
        assert_eq!(idx.line_col(6), Some(lc(2, 0)));
        assert_eq!(idx.line_col(8), Some(lc(3, 0)));
        // Inside the \r\n: a column past the line's text, which has no offset.
        assert_eq!(idx.line_col(5), Some(lc(1, 3)));
        assert_eq!(idx.offset(lc(1, 3)), None);
        assert_eq!(idx.offset(lc(1, 2)), Some(4));
    }

    #[test]
    fn empty_text_and_trailing_newline() {
        let idx = LineIndex::new("");
        assert_eq!(idx.line_count(), 1);
        assert_eq!(idx.line_col(0), Some(lc(0, 0)));
        assert_eq!(idx.line_col(1), None);

        let idx = LineIndex::new("x\n");
        assert_eq!(idx.line_count(), 2);
        assert_eq!(idx.line_col(2), Some(lc(1, 0)));
        assert_eq!(idx.offset(lc(1, 0)), Some(2));
        assert_eq!(idx.offset(lc(2, 0)), None);
    }

    #[test]
    fn round_trips_every_char_boundary() {
        let text = "héllo 😀 wörld\n\tcafé 𝄞 x\r\n\r\nend";
        let idx = LineIndex::new(text);
        for (offset, _) in text.char_indices().chain([(text.len(), ' ')]) {
            let Some(pos) = idx.line_col(offset) else {
                continue; // inside a line ending
            };
            if idx.offset(pos).is_none() {
                continue; // between \r and \n
            }
            assert_eq!(idx.offset(pos), Some(offset));
            for enc in [WideEncoding::Utf16, WideEncoding::Utf32] {
                let wide = idx.to_wide(enc, pos).unwrap();
                assert_eq!(idx.to_utf8(enc, wide), Some(pos), "{enc:?} at {offset}");
            }
        }
    }

    #[test]
    fn columns_outside_the_basic_multilingual_plane() {
        // 😀 is U+1F600: 4 UTF-8 bytes, 2 UTF-16 code units, 1 scalar value.
        let text = "a😀b";
        let idx = LineIndex::new(text);
        let b = text.find('b').unwrap();
        assert_eq!(b, 5);
        assert_eq!(idx.line_col(b), Some(lc(0, 5)));
        assert_eq!(idx.wide_line_col(WideEncoding::Utf16, b), Some(wlc(0, 3)));
        assert_eq!(idx.wide_line_col(WideEncoding::Utf32, b), Some(wlc(0, 2)));
        assert_eq!(idx.wide_offset(WideEncoding::Utf16, wlc(0, 3)), Some(5));
        assert_eq!(idx.wide_offset(WideEncoding::Utf32, wlc(0, 2)), Some(5));
        // The middle of the character, in each unit.
        assert_eq!(idx.line_col(2), None);
        assert_eq!(idx.offset(lc(0, 3)), None);
        assert_eq!(idx.to_utf8(WideEncoding::Utf16, wlc(0, 2)), None);
        // Past the end of the line.
        assert_eq!(idx.to_utf8(WideEncoding::Utf16, wlc(0, 5)), None);
        assert_eq!(idx.to_utf8(WideEncoding::Utf16, wlc(0, 4)), Some(lc(0, 6)));
    }

    #[test]
    fn columns_inside_the_basic_multilingual_plane() {
        // é is 2 UTF-8 bytes; 中 is 3. Both are one UTF-16 unit.
        let text = "x\né中z";
        let idx = LineIndex::new(text);
        let z = text.find('z').unwrap();
        assert_eq!(idx.line_col(z), Some(lc(1, 5)));
        assert_eq!(idx.wide_line_col(WideEncoding::Utf16, z), Some(wlc(1, 2)));
        assert_eq!(idx.wide_line_col(WideEncoding::Utf32, z), Some(wlc(1, 2)));
    }

    #[test]
    fn conformance_columns_are_one_based_scalar_values() {
        // How `ascribe check` and the conformance harness report a column.
        let text = "## Ünïcode 😀 {key}";
        let idx = LineIndex::new(text);
        let brace = text.find('{').unwrap();
        let col = idx.wide_line_col(WideEncoding::Utf32, brace).unwrap().col + 1;
        assert_eq!(col, text[..brace].chars().count() as u32 + 1);
    }
}
