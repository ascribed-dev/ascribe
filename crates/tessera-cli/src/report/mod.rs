//! Showing diagnostics: readable text with source snippets, and JSON.
//!
//! Both renderers read a [`FileTable`], which holds what a diagnostic's
//! locations point into (the path to show and the text), and turn a
//! `tessera_check::Diagnostic` into lines and columns the same way, so text
//! and JSON always agree.

pub mod json;
pub mod text;

use tessera_check::{Diagnostic, Project, Severity};
use tessera_core::{FileId, LineIndex, Location, WideEncoding};

/// One file a diagnostic can point into.
struct Entry {
    id: FileId,
    /// The path to show: relative to the project root, `/`-separated.
    path: String,
    text: String,
    index: LineIndex,
}

/// The files diagnostics point into.
pub struct FileTable {
    entries: Vec<Entry>,
}

/// A position: 1-based line and column (in Unicode scalar values), and the
/// byte offset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    pub line: u32,
    pub column: u32,
    pub offset: usize,
}

impl FileTable {
    /// The files of a loaded project: the content model, every source, and
    /// every code file the checks read for a snippet. Made after the checks,
    /// so the code files are there.
    pub fn of_project(project: &Project) -> FileTable {
        let mut ids = vec![FileId::new(0)];
        ids.extend(project.sources().iter().map(|s| s.id));
        let mut entries: Vec<Entry> = ids
            .into_iter()
            .filter_map(|id| project.file(id))
            .map(|f| Entry {
                id: f.id,
                path: f.display_path.clone(),
                index: LineIndex::new(f.text),
                text: f.text.to_owned(),
            })
            .collect();
        entries.extend(project.code_files().all().iter().map(|f| Entry {
            id: f.id,
            path: f.path.to_string(),
            index: LineIndex::new(&f.text),
            text: f.text.clone(),
        }));
        FileTable { entries }
    }

    /// Just the content model, for a project that didn't load.
    pub fn of_model(text: String) -> FileTable {
        FileTable {
            entries: vec![Entry {
                id: FileId::new(0),
                path: tessera_check::MODEL_FILE.to_owned(),
                index: LineIndex::new(&text),
                text,
            }],
        }
    }

    fn entry(&self, id: FileId) -> Option<&Entry> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// The path to show for a file.
    pub fn path(&self, id: FileId) -> String {
        self.entry(id)
            .map_or_else(|| format!("<file {}>", id.index()), |e| e.path.clone())
    }

    /// The position of a byte offset in a file.
    pub fn position(&self, id: FileId, offset: usize) -> Position {
        let unknown = Position {
            line: 1,
            column: 1,
            offset,
        };
        let Some(entry) = self.entry(id) else {
            return unknown;
        };
        entry
            .index
            .wide_line_col(WideEncoding::Utf32, offset)
            .map_or(unknown, |p| Position {
                line: p.line + 1,
                column: p.col + 1,
                offset,
            })
    }

    /// The start and end positions of a location.
    pub fn range(&self, at: Location) -> (Position, Position) {
        (
            self.position(at.file, at.span.start()),
            self.position(at.file, at.span.end()),
        )
    }
}

/// How many errors and warnings a list of diagnostics has.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub errors: usize,
    pub warnings: usize,
}

impl Counts {
    /// Counts a list.
    pub fn of(diagnostics: &[Diagnostic]) -> Counts {
        let mut counts = Counts::default();
        for d in diagnostics {
            match d.severity {
                Severity::Error => counts.errors += 1,
                Severity::Warning => counts.warnings += 1,
            }
        }
        counts
    }
}

impl FileTable {
    /// A file's text.
    pub fn text(&self, id: FileId) -> Option<&str> {
        self.entry(id).map(|e| e.text.as_str())
    }

    /// The text of the file shown as `path`, for the snippet renderer.
    pub fn text_at(&self, path: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|e| e.path == path)
            .map(|e| e.text.as_str())
    }
}
