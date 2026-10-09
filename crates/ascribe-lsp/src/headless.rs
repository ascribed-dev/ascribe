//! A project kept checked without an editor: the server's state and its
//! incremental index, driven by a caller that says which files changed, and
//! asks for a file's problems. `ascribe agents hook-server` keeps one per
//! project, so the check after an agent's edit redoes only what the edit
//! affects, as the server does after a keystroke.
//!
//! The problems of a file are those `ascribe check <file> --editor-build`
//! reports: what the server would publish for it (its file-level
//! diagnostics and the page-level ones of the editor's build located in
//! it), and those located elsewhere that name a place in it, such as a
//! repeated `@id` in an included fragment, once for each place.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use ascribe_core::path::normalize;
use crossbeam_channel::Receiver;
use lsp_server::Message;
use lsp_types::{Diagnostic, DiagnosticSeverity, FileChangeType, FileEvent, NumberOrString};

use crate::compute::compute;
use crate::core::Core;
use crate::uri::{path_to_uri, uri_to_path};

/// A problem the server found in a file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problem {
    /// The file it's located in: the one asked about, or another that
    /// names a place in it.
    pub file: PathBuf,
    /// Its line, from 1.
    pub line: u32,
    /// Its diagnostic's code, `ASC036`.
    pub code: String,
    /// Its message.
    pub message: String,
    /// Whether it's an error; otherwise a warning.
    pub error: bool,
}

/// One project, loaded and kept current.
pub struct Watched {
    core: Core,
    // What the server would send a client: drained, and dropped.
    sent: Receiver<Message>,
}

impl Watched {
    /// Loads the project of `config`, an `ascribe.toml`.
    pub fn load(config: &Path) -> Watched {
        let (out, sent) = crossbeam_channel::unbounded();
        let mut core = Core::new(out);
        let config = normalize(config);
        core.folders = config.parent().map(Path::to_owned).into_iter().collect();
        core.config = Some(config);
        core.start();
        Watched { core, sent }
    }

    /// Takes in files created or changed (`present`) and deleted (`gone`) on
    /// disk since the last call, as a file watcher reports them.
    pub fn changed(&mut self, present: &[PathBuf], gone: &[PathBuf]) {
        let event = |path: &PathBuf, typ| Some(FileEvent::new(path_to_uri(path)?, typ));
        let events: Vec<FileEvent> = present
            .iter()
            .filter_map(|p| event(p, FileChangeType::CHANGED))
            .chain(
                gone.iter()
                    .filter_map(|p| event(p, FileChangeType::DELETED)),
            )
            .collect();
        if !events.is_empty() {
            self.core.did_change_watched(events);
        }
    }

    /// The problems of `file` now: its own in line order, then those
    /// located elsewhere, after computing what the changes so far affect.
    pub fn problems(&mut self, file: &Path) -> Vec<Problem> {
        while let Some(job) = self.core.plan() {
            let outcome = compute(&job, &|| true);
            self.core.finish(&job, outcome);
        }
        self.sent.try_iter().for_each(drop);
        let file = normalize(file);
        let mut own: Vec<Problem> = self
            .core
            .published(&file)
            .iter()
            .map(|d| problem(&file, d))
            .collect();
        own.sort_by_key(|p| p.line);
        // As `ascribe_check::Scope::report` has them: a problem located
        // elsewhere counts when a place it names is in the file, and its
        // repeats (the same code at the same places) count once.
        let mut seen: HashSet<(String, Vec<(u32, u32)>)> = HashSet::new();
        let mut elsewhere: Vec<Problem> = Vec::new();
        let mut others: Vec<(&PathBuf, &[Diagnostic])> = self
            .core
            .all_published()
            .filter(|(path, _)| **path != file)
            .collect();
        others.sort_by(|a, b| a.0.cmp(b.0));
        for (path, diagnostics) in others {
            for d in diagnostics {
                let places: Vec<(u32, u32)> = d
                    .related_information
                    .iter()
                    .flatten()
                    .filter(|r| uri_to_path(&r.location.uri).is_some_and(|p| normalize(&p) == file))
                    .map(|r| {
                        (
                            r.location.range.start.line,
                            r.location.range.start.character,
                        )
                    })
                    .collect();
                if places.is_empty() {
                    continue;
                }
                let p = problem(path, d);
                if seen.insert((p.code.clone(), places)) {
                    elsewhere.push(p);
                }
            }
        }
        own.extend(elsewhere);
        own
    }

    /// The editor's build, which the problems are of; `None` before the
    /// project has loaded.
    pub fn build(&self) -> Option<String> {
        let loaded = self.core.loaded.as_ref()?;
        Some(loaded.model.editor_default_build().name.clone())
    }

    /// The project root, `ascribe.toml`'s folder; `None` before the project
    /// has loaded.
    pub fn root(&self) -> Option<&Path> {
        self.core.loaded.as_ref().map(|l| l.root.as_path())
    }
}

/// `d`, published for `file`, as a problem.
fn problem(file: &Path, d: &Diagnostic) -> Problem {
    Problem {
        file: file.to_owned(),
        line: d.range.start.line + 1,
        code: match &d.code {
            Some(NumberOrString::String(code)) => code.clone(),
            Some(NumberOrString::Number(n)) => n.to_string(),
            None => String::new(),
        },
        message: d.message.clone(),
        error: d.severity != Some(DiagnosticSeverity::WARNING),
    }
}
