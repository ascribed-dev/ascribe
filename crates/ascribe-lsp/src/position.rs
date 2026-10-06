//! Positions in the negotiated encoding, over `ascribe_core::LineIndex`.

use ascribe_core::{LineCol, LineIndex, Span, WideEncoding, WideLineCol};
use lsp_types::{Position, PositionEncodingKind, Range};

/// How the client counts columns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Encoding {
    /// UTF-8 bytes.
    Utf8,
    /// UTF-16 code units, LSP's default.
    Utf16,
}

impl Encoding {
    /// UTF-8 when the client offers it, otherwise UTF-16 (the protocol's
    /// default, which every client supports).
    pub fn negotiate(offered: Option<&[PositionEncodingKind]>) -> Encoding {
        if offered.is_some_and(|kinds| kinds.contains(&PositionEncodingKind::UTF8)) {
            Encoding::Utf8
        } else {
            Encoding::Utf16
        }
    }

    /// The kind to send back in `positionEncoding`.
    pub fn kind(self) -> PositionEncodingKind {
        match self {
            Encoding::Utf8 => PositionEncodingKind::UTF8,
            Encoding::Utf16 => PositionEncodingKind::UTF16,
        }
    }

    /// The position of a byte offset, or `None` past the end of the text or
    /// inside a character.
    pub fn position(self, index: &LineIndex, offset: usize) -> Option<Position> {
        let (line, character) = match self {
            Encoding::Utf8 => {
                let p = index.line_col(offset)?;
                (p.line, p.col)
            }
            Encoding::Utf16 => {
                let p = index.wide_line_col(WideEncoding::Utf16, offset)?;
                (p.line, p.col)
            }
        };
        Some(Position { line, character })
    }

    /// The range of a span. An offset the index can't place (which a span from
    /// the parser never is) is clamped to the end of the text.
    pub fn range(self, index: &LineIndex, span: Span) -> Range {
        let at = |offset: usize| {
            self.position(index, offset)
                .or_else(|| self.position(index, index.len()))
                .unwrap_or_default()
        };
        Range {
            start: at(span.start()),
            end: at(span.end()),
        }
    }

    /// The byte offset of a position, exactly: `None` when the line doesn't
    /// exist, the column is past the end of its text, or it's inside a
    /// character.
    pub fn offset(self, index: &LineIndex, pos: Position) -> Option<usize> {
        match self {
            Encoding::Utf8 => index.offset(LineCol {
                line: pos.line,
                col: pos.character,
            }),
            Encoding::Utf16 => index.wide_offset(
                WideEncoding::Utf16,
                WideLineCol {
                    line: pos.line,
                    col: pos.character,
                },
            ),
        }
    }

    /// The byte offset of a position, the way an editor edit wants it: a line
    /// past the end is the end of the text, a column past the end of a line is
    /// the end of that line, and a column inside a character is the start of
    /// that character (LSP asks servers to be this lenient).
    pub fn offset_lenient(self, index: &LineIndex, text: &str, pos: Position) -> usize {
        let Some(line) = index.line_span(pos.line) else {
            return text.len();
        };
        let end = self.position(index, line.end()).map_or(0, |p| p.character);
        let mut character = pos.character.min(end);
        loop {
            let at = Position {
                line: pos.line,
                character,
            };
            if let Some(offset) = self.offset(index, at) {
                return offset;
            }
            if character == 0 {
                return line.start();
            }
            // Inside a character: step back to where it starts.
            character -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf16_counts_astral_characters_twice() {
        let text = "a📄b\nsecond";
        let index = LineIndex::new(text);
        let b = text.find('b').expect("a b");
        let p = Encoding::Utf16.position(&index, b).expect("a position");
        assert_eq!((p.line, p.character), (0, 3));
        let p = Encoding::Utf8.position(&index, b).expect("a position");
        assert_eq!((p.line, p.character), (0, 5));
        assert_eq!(Encoding::Utf16.offset(&index, Position::new(0, 3)), Some(b));
        assert_eq!(Encoding::Utf16.offset(&index, Position::new(0, 2)), None);
    }

    #[test]
    fn lenient_offsets_clamp() {
        let text = "a📄b\nsecond";
        let index = LineIndex::new(text);
        let enc = Encoding::Utf16;
        assert_eq!(enc.offset_lenient(&index, text, Position::new(0, 2)), 1);
        assert_eq!(
            enc.offset_lenient(&index, text, Position::new(0, 99)),
            text.find('\n').expect("a newline")
        );
        assert_eq!(
            enc.offset_lenient(&index, text, Position::new(9, 0)),
            text.len()
        );
    }

    #[test]
    fn negotiation_prefers_utf8_only_when_offered() {
        assert_eq!(Encoding::negotiate(None), Encoding::Utf16);
        assert_eq!(
            Encoding::negotiate(Some(&[
                PositionEncodingKind::UTF16,
                PositionEncodingKind::UTF8
            ])),
            Encoding::Utf8
        );
        assert_eq!(
            Encoding::negotiate(Some(&[PositionEncodingKind::UTF16])),
            Encoding::Utf16
        );
    }
}
