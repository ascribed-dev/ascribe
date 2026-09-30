//! Files and byte spans.

use std::fmt;
use std::ops::Range;

use serde::Serialize;

/// Identifies one file within a session: a source file, or `ascribe.toml`.
///
/// A `FileId` is an opaque handle. Whoever owns the set of files a session
/// reads assigns the ids: the project's source index and the language
/// server's file table do, and a tool that reads a single
/// file can use any id. An id is meaningful only within the table that
/// assigned it, and says nothing about the file's path.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct FileId(u32);

impl FileId {
    /// A file id from its raw number.
    pub const fn new(raw: u32) -> FileId {
        FileId(raw)
    }

    /// The raw number, for use as an index into the owner's file table.
    pub const fn index(self) -> u32 {
        self.0
    }
}

impl fmt::Display for FileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "file#{}", self.0)
    }
}

/// A half-open range `[start, end)` of UTF-8 byte offsets into one file's text.
///
/// Invariant: `start <= end`. The constructors keep it; the fields are private
/// so nothing else can break it. Offsets count bytes from the start of the
/// file, including any frontmatter. Spans produced by Ascribe crates always
/// fall on `char` boundaries.
///
/// A span doesn't record its file; pair it with a [`FileId`] in a
/// [`Location`] when the file isn't implied.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct Span {
    start: usize,
    end: usize,
}

impl Span {
    /// The span between two offsets, in either order.
    pub fn new(a: usize, b: usize) -> Span {
        Span {
            start: a.min(b),
            end: a.max(b),
        }
    }

    /// An empty span at `offset`, as used for an insertion point.
    pub fn empty(offset: usize) -> Span {
        Span {
            start: offset,
            end: offset,
        }
    }

    /// The first byte offset in the span.
    pub fn start(self) -> usize {
        self.start
    }

    /// The offset just past the span's last byte.
    pub fn end(self) -> usize {
        self.end
    }

    /// The span's length in bytes.
    pub fn len(self) -> usize {
        self.end - self.start
    }

    /// Whether the span covers no bytes.
    pub fn is_empty(self) -> bool {
        self.start == self.end
    }

    /// Whether `offset` lies in `[start, end)`.
    pub fn contains(self, offset: usize) -> bool {
        self.start <= offset && offset < self.end
    }

    /// Whether `other` lies entirely within this span.
    pub fn contains_span(self, other: Span) -> bool {
        self.start <= other.start && other.end <= self.end
    }

    /// The smallest span covering both spans.
    pub fn cover(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    /// The span as a range, for slicing the file's text.
    pub fn range(self) -> Range<usize> {
        self.start..self.end
    }
}

impl From<Range<usize>> for Span {
    fn from(range: Range<usize>) -> Span {
        Span::new(range.start, range.end)
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

/// A span in a particular file.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct Location {
    /// The file.
    pub file: FileId,
    /// The bytes in that file.
    pub span: Span,
}

impl Location {
    /// A location from a file and a span.
    pub fn new(file: FileId, span: impl Into<Span>) -> Location {
        Location {
            file,
            span: span.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spans_keep_start_before_end() {
        let s = Span::new(7, 3);
        assert_eq!((s.start(), s.end(), s.len()), (3, 7, 4));
        assert_eq!(Span::from(3..7), s);
        assert!(Span::empty(4).is_empty());
    }

    #[test]
    fn containment_and_cover() {
        let s = Span::new(2, 6);
        assert!(s.contains(2) && s.contains(5) && !s.contains(6));
        assert!(s.contains_span(Span::new(3, 6)));
        assert!(!s.contains_span(Span::new(1, 3)));
        assert_eq!(s.cover(Span::new(8, 9)), Span::new(2, 9));
        assert_eq!(s.range(), 2..6);
    }
}
