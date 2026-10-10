//! Showing diagnostics: readable text with source snippets, one line each,
//! and JSON.
//!
//! The renderers read a [`FileTable`], which holds what a diagnostic's
//! locations point into (the path to show and the text), and turn a
//! `ascribe_check::Diagnostic` into lines and columns the same way, so text
//! and JSON always agree.

pub mod concise;
pub mod json;
pub mod text;

use ascribe_check::{Diagnostic, Project, Reported, Severity};
use ascribe_core::{FileId, LineIndex, Location, WideEncoding};

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
    /// The files of a loaded project: the content model, the lock, every
    /// source, and every code file the checks read for a snippet. Made after
    /// the checks, so the code files are there.
    pub fn of_project(project: &Project) -> FileTable {
        let mut ids = vec![FileId::new(0), ascribe_check::LOCK_FILE_ID];
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
                path: ascribe_check::MODEL_FILE.to_owned(),
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
        match self.entry(id) {
            Some(entry) => position_in(&entry.index, offset),
            None => Position {
                line: 1,
                column: 1,
                offset,
            },
        }
    }

    /// The start and end positions of a location.
    pub fn range(&self, at: Location) -> (Position, Position) {
        (
            self.position(at.file, at.span.start()),
            self.position(at.file, at.span.end()),
        )
    }
}

/// The position of a byte offset in the text `index` indexes.
pub fn position_in(index: &LineIndex, offset: usize) -> Position {
    index.wide_line_col(WideEncoding::Utf32, offset).map_or(
        Position {
            line: 1,
            column: 1,
            offset,
        },
        |p| Position {
            line: p.line + 1,
            column: p.col + 1,
            offset,
        },
    )
}

/// How many errors, warnings, and advice a list of diagnostics has.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub errors: usize,
    pub warnings: usize,
    pub advice: usize,
}

impl Counts {
    /// Counts a list.
    pub fn of(diagnostics: &[Diagnostic]) -> Counts {
        let mut counts = Counts::default();
        for d in diagnostics {
            counts.add(d.severity);
        }
        counts
    }

    /// Counts a report.
    pub fn of_reported(reported: &[Reported]) -> Counts {
        let mut counts = Counts::default();
        for r in reported {
            counts.add(r.diagnostic.severity);
        }
        counts
    }

    fn add(&mut self, severity: Severity) {
        match severity {
            Severity::Error => self.errors += 1,
            Severity::Warning => self.warnings += 1,
            Severity::Advice => self.advice += 1,
        }
    }

    /// Errors, warnings, and advice together.
    pub fn total(self) -> usize {
        self.errors + self.warnings + self.advice
    }
}

/// How many diagnostics have one code, for `--summary`.
pub struct ByCode {
    pub code: &'static str,
    pub slug: String,
    pub severity: Severity,
    pub count: usize,
}

/// How many diagnostics are in one file, for `--summary`.
pub struct ByFile {
    pub file: String,
    pub counts: Counts,
}

/// A report's diagnostics counted by code and by file, most first; a tie
/// keeps the order the report first shows them in.
pub fn tally(files: &FileTable, reported: &[Reported]) -> (Vec<ByCode>, Vec<ByFile>) {
    let mut codes: Vec<ByCode> = Vec::new();
    let mut by_file: Vec<ByFile> = Vec::new();
    for r in reported {
        let d = &r.diagnostic;
        match codes.iter_mut().find(|c| c.code == d.code) {
            Some(c) => c.count += 1,
            None => codes.push(ByCode {
                code: d.code,
                slug: d.slug.to_string(),
                severity: d.severity,
                count: 1,
            }),
        }
        let file = files.path(d.location.file);
        match by_file.iter_mut().find(|f| f.file == file) {
            Some(f) => f.counts.add(d.severity),
            None => {
                let mut counts = Counts::default();
                counts.add(d.severity);
                by_file.push(ByFile { file, counts });
            }
        }
    }
    // Stable sorts: ties stay in the report's order.
    codes.sort_by_key(|c| std::cmp::Reverse(c.count));
    by_file.sort_by_key(|f| std::cmp::Reverse(f.counts.total()));
    (codes, by_file)
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
