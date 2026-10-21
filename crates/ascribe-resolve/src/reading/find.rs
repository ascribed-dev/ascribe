//! The places a query matches in the readings of many pages and builds,
//! each a line of the source to edit, listed once however many pages or
//! builds show it.

use std::collections::HashMap;
use std::ops::Range;

use ascribe_core::{FileId, LineIndex, RelPath, Span};

use super::{In, Origin, PieceKind, Query, ReadBlock, Reading, Under, line_col};
use crate::Project;
use crate::fs::MODEL_FILE;
use crate::snippet::{Snippet, parse_address};

/// What [`find`] found.
#[derive(Clone, Debug, Default)]
pub struct Found {
    /// The places, in order: titles, then headings, then the rest; in each,
    /// the places written in pages, then those in fragments and code files,
    /// each in path order and in order within a file.
    pub places: Vec<Place>,
    /// How many pages show a place.
    pub pages: usize,
}

/// A place the query matches: a line of the source.
#[derive(Clone, Debug)]
pub struct Place {
    /// Where to edit.
    pub at: At,
    /// What it is.
    pub kind: In,
    /// The text of the block it matched in, as a reader reads it in the
    /// first build that has it; for a phrase, its value.
    pub text: String,
    /// The match, in bytes of `text`.
    pub matched: Range<usize>,
    /// The heading the block is under, or for a heading, itself.
    pub under: Option<Under>,
    /// The arm the block is written in.
    pub arm: Option<String>,
    /// What the match reads through that the source doesn't spell.
    pub through: Vec<Through>,
    /// The builds it matched in, in the order searched.
    pub builds: Vec<String>,
    /// The pages that show it, in path order.
    pub shown: Vec<Shown>,
    /// Whether it's written where other pages can show it too: in a
    /// fragment, or in a snippet's code file.
    pub shared: bool,
    /// For a snippet's code that can't be lined up with its file, the last
    /// line it shows: the place is the lines shown, from the first, not the
    /// line the match is on.
    pub end_line: Option<u32>,
}

/// A page that shows a place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shown {
    /// The page's content path.
    pub page: RelPath,
    /// The heading the place is under on the page, in the first build that
    /// shows it there.
    pub under: Option<Under>,
    /// The builds that show it on the page, in the order searched.
    pub builds: Vec<String>,
}

/// Where a place is written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum At {
    /// In a source file.
    Source {
        /// The file.
        file: FileId,
        /// The byte where the match's first character is written.
        offset: usize,
        /// Its line, from 1.
        line: u32,
        /// Its column, from 1, in Unicode characters.
        column: u32,
    },
    /// In a snippet's code file.
    Code {
        /// The code file's id.
        file: FileId,
        /// Its path from the project root, or, for a file outside the
        /// project's folder, `<source>:<path>`, as the snippet names it.
        path: String,
        /// Its path from the project root, outside the project's folder or
        /// not.
        from_root: RelPath,
        /// The byte where the match's first character is written.
        offset: usize,
        /// Its line, from 1.
        line: u32,
        /// Its column, from 1, in Unicode characters.
        column: u32,
    },
    /// A phrase's value in `ascribe.toml`, when the match is wholly in it.
    Phrase {
        /// The phrase's key.
        key: String,
        /// The key in `ascribe.toml`.
        span: Span,
    },
}

/// Text a match reads through that its line doesn't spell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Through {
    /// A phrase's value.
    Phrase(String),
    /// The title of a page a link with no text takes, or the text of the
    /// heading it names.
    Title {
        /// The page.
        page: RelPath,
        /// The heading's id, when the link names one.
        id: Option<String>,
    },
    /// A snippet's code: its address, as written.
    Code(String),
}

/// What identifies a place: its line, or its phrase.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Key {
    Line(FileId, u32),
    Phrase(String),
}

/// The places `query` matches in `readings`, given in the order to take a
/// place's text from: each build's pages, the builds in the content model's
/// order. A line that matches in several pages or builds is one place,
/// with each of them. A block matches at its first match; when that's
/// wholly in a phrase's value, the phrase is a place and matching carries
/// on in the block after the phrase, until a match is the block's line.
pub fn find<'a>(
    project: &Project,
    readings: impl IntoIterator<Item = &'a Reading>,
    query: &Query,
) -> Found {
    let mut lines: HashMap<FileId, LineIndex> = HashMap::new();
    let mut places: Vec<Place> = Vec::new();
    let mut keys: HashMap<Key, usize> = HashMap::new();
    for reading in readings {
        for block in &reading.blocks {
            let mut from = 0;
            while let Some(range) = query.find_from(&block.text, from) {
                let (place, after) = locate(project, &mut lines, block, range);
                let key = match &place.at {
                    At::Source { file, line, .. } | At::Code { file, line, .. } => {
                        Key::Line(*file, *line)
                    }
                    At::Phrase { key, .. } => Key::Phrase(key.clone()),
                };
                let i = *keys.entry(key).or_insert_with(|| {
                    places.push(place);
                    places.len() - 1
                });
                if let Some(place) = places.get_mut(i) {
                    if !place.builds.contains(&reading.build) {
                        place.builds.push(reading.build.clone());
                    }
                    let at = match place.shown.iter().position(|s| s.page == reading.page) {
                        Some(at) => at,
                        None => {
                            place.shown.push(Shown {
                                page: reading.page.clone(),
                                under: block.under.clone(),
                                builds: Vec::new(),
                            });
                            place.shown.len() - 1
                        }
                    };
                    if let Some(shown) = place.shown.get_mut(at)
                        && !shown.builds.contains(&reading.build)
                    {
                        shown.builds.push(reading.build.clone());
                    }
                }
                match after {
                    Some(after) if after > from => from = after,
                    _ => break,
                }
            }
        }
    }
    let path = |at: &At| match at {
        At::Source { file, .. } => project
            .path_of(*file)
            .map(|p| project.layout().project_path(p).to_string())
            .unwrap_or_default(),
        At::Code { path, .. } => path.clone(),
        At::Phrase { .. } => MODEL_FILE.to_owned(),
    };
    let offset = |at: &At| match at {
        At::Source { offset, .. } | At::Code { offset, .. } => *offset,
        At::Phrase { span, .. } => span.start(),
    };
    let rank = |kind: In| match kind {
        In::Title => 0,
        In::Heading => 1,
        _ => 2,
    };
    // Within each kind, the pages' own places come before those of the
    // files they bring in, fragments and code files.
    places.sort_by_cached_key(|p| (rank(p.kind), p.shared, path(&p.at), offset(&p.at)));
    for place in &mut places {
        place.shown.sort_by(|a, b| a.page.cmp(&b.page));
    }
    let mut pages: Vec<&RelPath> = places
        .iter()
        .flat_map(|p| p.shown.iter().map(|s| &s.page))
        .collect();
    pages.sort();
    pages.dedup();
    let pages = pages.len();
    Found { places, pages }
}

/// The place a match in a block is, and, when it's a phrase's value, where
/// in the block's text the phrase ends.
fn locate(
    project: &Project,
    lines: &mut HashMap<FileId, LineIndex>,
    block: &ReadBlock,
    matched: Range<usize>,
) -> (Place, Option<usize>) {
    let mut through = Vec::new();
    for piece in block.pieces_in(matched.clone()) {
        let item = match &piece.kind {
            PieceKind::Phrase(key) => Through::Phrase(key.clone()),
            PieceKind::Borrowed { page, id } => Through::Title {
                page: page.clone(),
                id: id.clone(),
            },
            PieceKind::Copied | PieceKind::Made => continue,
        };
        if !through.contains(&item) {
            through.push(item);
        }
    }
    // The last line a snippet shows, when its code can't be lined up.
    let (shared, last_shown) = match &block.origin {
        Origin::Page => (false, None),
        Origin::Fragment => (true, None),
        Origin::Snippet { snippet, exact } => {
            through.push(Through::Code(snippet.address.clone()));
            (true, snippet.spans.last().filter(|_| !exact).copied())
        }
    };
    let first = block.piece_at(matched.start);
    let last = block.piece_at(matched.end.max(matched.start + 1) - 1);
    // Wholly inside one phrase's value: the place to edit is the value.
    if let (Some(first), Some(last)) = (first, last)
        && first == last
        && let PieceKind::Phrase(key) = &first.kind
        && let Some(declared) = project.model().phrases.iter().find(|p| &p.key == key)
    {
        let text = block.text[first.range.clone()].to_owned();
        let start = matched.start - first.range.start;
        let place = Place {
            at: At::Phrase {
                key: key.clone(),
                span: declared.span,
            },
            kind: In::Phrase,
            matched: start..start + matched.len(),
            text,
            under: None,
            arm: None,
            through,
            builds: Vec::new(),
            shown: Vec::new(),
            shared: false,
            end_line: None,
        };
        return (place, Some(first.range.end));
    }
    let offset = block.source_offset(matched.start);
    let index = lines
        .entry(block.file)
        .or_insert_with(|| LineIndex::new(block.source(project)));
    let (line, column) = line_col(index, offset);
    let end_line = last_shown.map(|span| line_col(index, span.start()).0);
    let at = match &block.origin {
        Origin::Snippet { snippet, .. } => At::Code {
            file: block.file,
            path: code_path(snippet),
            from_root: snippet.path.clone(),
            offset,
            line,
            column,
        },
        Origin::Page | Origin::Fragment => At::Source {
            file: block.file,
            offset,
            line,
            column,
        },
    };
    let place = Place {
        at,
        kind: block.kind,
        text: block.text.clone(),
        matched,
        under: block.under.clone(),
        arm: block.arm.clone(),
        through,
        builds: Vec::new(),
        shown: Vec::new(),
        shared,
        end_line,
    };
    (place, None)
}

/// A code file's path from the project root, or, when that leaves the
/// project's folder, the file as the snippet's address names it.
fn code_path(snippet: &Snippet) -> String {
    let path = snippet.path.as_str();
    if path != ".." && !path.starts_with("../") {
        return path.to_owned();
    }
    parse_address(&snippet.address).map_or_else(|_| snippet.address.clone(), |a| a.file())
}
