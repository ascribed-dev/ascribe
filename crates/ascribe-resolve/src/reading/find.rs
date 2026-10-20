//! The places a query matches in the readings of many pages and builds,
//! each a line of the source to edit, listed once however many pages or
//! builds show it.

use std::collections::HashMap;
use std::ops::Range;

use ascribe_core::{FileId, LineIndex, RelPath, Span};

use super::{In, PieceKind, Query, ReadBlock, Reading, Under, line_col};
use crate::Project;
use crate::fs::MODEL_FILE;

/// What [`find`] found.
#[derive(Clone, Debug, Default)]
pub struct Found {
    /// The places, in order: headings, then the rest, each in path order
    /// and in order within a file.
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
    pub pages: Vec<RelPath>,
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
/// with each of them; a block matches once, at its first match.
pub fn find(project: &Project, readings: &[Reading], query: &Query) -> Found {
    let mut lines: HashMap<FileId, LineIndex> = HashMap::new();
    let mut places: Vec<Place> = Vec::new();
    let mut keys: HashMap<Key, usize> = HashMap::new();
    for reading in readings {
        for block in &reading.blocks {
            let Some(range) = query.find_in(&block.text) else {
                continue;
            };
            let place = locate(project, &mut lines, block, range);
            let key = match &place.at {
                At::Source { file, line, .. } => Key::Line(*file, *line),
                At::Phrase { key, .. } => Key::Phrase(key.clone()),
            };
            let i = *keys.entry(key).or_insert_with(|| {
                places.push(place);
                places.len() - 1
            });
            let Some(place) = places.get_mut(i) else {
                continue;
            };
            if !place.builds.contains(&reading.build) {
                place.builds.push(reading.build.clone());
            }
            if !place.pages.contains(&reading.page) {
                place.pages.push(reading.page.clone());
            }
        }
    }
    let path = |at: &At| match at {
        At::Source { file, .. } => project
            .path_of(*file)
            .map(|p| project.layout().project_path(p).to_string())
            .unwrap_or_default(),
        At::Phrase { .. } => MODEL_FILE.to_owned(),
    };
    let offset = |at: &At| match at {
        At::Source { offset, .. } => *offset,
        At::Phrase { span, .. } => span.start(),
    };
    places.sort_by_cached_key(|p| (p.kind != In::Heading, path(&p.at), offset(&p.at)));
    for place in &mut places {
        place.pages.sort();
    }
    let mut pages: Vec<&RelPath> = places.iter().flat_map(|p| &p.pages).collect();
    pages.sort();
    pages.dedup();
    let pages = pages.len();
    Found { places, pages }
}

/// The place a match in a block is.
fn locate(
    project: &Project,
    lines: &mut HashMap<FileId, LineIndex>,
    block: &ReadBlock,
    matched: Range<usize>,
) -> Place {
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
        return Place {
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
            pages: Vec::new(),
        };
    }
    let offset = block.source_offset(matched.start);
    let index = lines.entry(block.file).or_insert_with(|| {
        LineIndex::new(project.file_by_id(block.file).map_or("", |f| &f.source))
    });
    let (line, column) = line_col(index, offset);
    Place {
        at: At::Source {
            file: block.file,
            offset,
            line,
            column,
        },
        kind: block.kind,
        text: block.text.clone(),
        matched,
        under: block.under.clone(),
        arm: block.arm.clone(),
        through,
        builds: Vec::new(),
        pages: Vec::new(),
    }
}
