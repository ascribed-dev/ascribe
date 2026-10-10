//! The prose, checked with Vale (`[checks.vale]`), on open and on save.
//!
//! Vale is a program of its own and can take seconds, so it never runs on the
//! request path or in the worker that computes the other diagnostics: a
//! thread of its own takes one open document at a time ([`Core::take_prose`])
//! and runs [`ascribe_check::prose::lint_pages`] on its text, without the
//! lock. What it finds is kept for the file and published with the file's
//! other diagnostics ([`Core::publish`] adds it), so a round of those never
//! drops it, and Vale's results never wait for one.
//!
//! A result is dropped when the document changed since it was asked for, or
//! closed, or the project or `[checks.vale]` changed. An edit drops what's
//! kept below the place it starts, which would now be in the wrong place, and
//! keeps what's above it; the next save checks the file again.

use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ascribe_check::prose::{self, FILE_TIMEOUT, Linter, Page};
use ascribe_core::{FileId, LineIndex, diagnostics};
use ascribe_model::{ContentModel, ValeSettings};
use lsp_types::{Position, TextDocumentContentChangeEvent};

use crate::compute::to_lsp;
use crate::core::Core;
use crate::position::Encoding;

/// What the prose thread keeps.
#[derive(Default)]
pub(crate) struct State {
    /// The open documents to check, oldest first, each once.
    queue: VecDeque<PathBuf>,
    /// Whether a check is running.
    running: bool,
    /// What Vale found, by file: a source's alerts, or, under `ascribe.toml`,
    /// the advice that it couldn't run.
    found: HashMap<PathBuf, Vec<lsp_types::Diagnostic>>,
}

impl State {
    /// Whether a check is queued or running.
    pub(crate) fn has_work(&self) -> bool {
        self.running || !self.queue.is_empty()
    }

    /// What Vale found in a file, when it found anything.
    pub(crate) fn found(&self, path: &Path) -> &[lsp_types::Diagnostic] {
        self.found.get(path).map_or(&[], Vec::as_slice)
    }
}

/// One check: a document's text, as it was when the check was taken.
pub(crate) struct Job {
    epoch: u64,
    root: PathBuf,
    model: Arc<ContentModel>,
    model_text: String,
    config: PathBuf,
    path: PathBuf,
    /// Its path from the project root, `/`-separated.
    display: String,
    id: FileId,
    version: i32,
    text: String,
    encoding: Encoding,
}

impl Job {
    /// Runs Vale on the document: its diagnostics, converted for the
    /// protocol, by the file they're on.
    pub(crate) fn run(&self, linter: &dyn Linter) -> Vec<(PathBuf, Vec<lsp_types::Diagnostic>)> {
        let page = Page {
            id: self.id,
            path: &self.display,
            text: &self.text,
        };
        let found = prose::lint_pages(&self.root, &self.model, &[page], linter, FILE_TIMEOUT);
        let page_index = LineIndex::new(&self.text);
        let model_index = LineIndex::new(&self.model_text);
        let mut on_page = Vec::new();
        let mut on_model = Vec::new();
        for d in &found {
            if d.slug == diagnostics::PROSE_NOT_CHECKED {
                on_model.push(to_lsp(d, &model_index, self.encoding, &|_| None));
            } else {
                on_page.push(to_lsp(d, &page_index, self.encoding, &|_| None));
            }
        }
        vec![
            (self.path.clone(), on_page),
            (self.config.clone(), on_model),
        ]
    }
}

impl Core {
    /// Queues a check of an open document, when the project checks its prose
    /// and the document is one of its sources.
    pub(crate) fn queue_prose(&mut self, path: &Path) {
        let Some(loaded) = self.loaded.as_ref() else {
            return;
        };
        if loaded.model.checks.vale.is_none()
            || !self.docs.contains_key(path)
            || !matches!(loaded.classify(path), Some(crate::core::Kind::Source(..)))
        {
            return;
        }
        if !self.prose.queue.iter().any(|p| p == path) {
            self.prose.queue.push_back(path.to_path_buf());
        }
    }

    /// Queues a check of every open source document.
    fn queue_all_prose(&mut self) {
        let mut open: Vec<PathBuf> = self.docs.keys().cloned().collect();
        open.sort();
        for path in open {
            self.queue_prose(&path);
        }
    }

    /// Takes the next check to run, if there is one.
    pub(crate) fn take_prose(&mut self) -> Option<Job> {
        while let Some(path) = self.prose.queue.pop_front() {
            let (Some(loaded), Some(config), Some(doc)) = (
                self.loaded.as_ref(),
                self.config.as_ref(),
                self.docs.get(&path),
            ) else {
                continue;
            };
            let Some(crate::core::Kind::Source(content, project)) = loaded.classify(&path) else {
                continue;
            };
            let id = loaded
                .inc
                .snapshot()
                .file(&content)
                .map_or(FileId::new(1), |f| f.file);
            self.prose.running = true;
            return Some(Job {
                epoch: loaded.epoch,
                root: loaded.root.clone(),
                model: loaded.model.clone(),
                model_text: loaded.model_text.clone(),
                config: config.clone(),
                path,
                display: project.as_str().to_owned(),
                id,
                version: doc.version,
                text: doc.text.clone(),
                encoding: self.encoding,
            });
        }
        None
    }

    /// Keeps and publishes what a check found, when it's still current.
    pub(crate) fn finish_prose(
        &mut self,
        job: &Job,
        results: Vec<(PathBuf, Vec<lsp_types::Diagnostic>)>,
    ) {
        self.prose.running = false;
        let current = self.loaded.as_ref().is_some_and(|loaded| {
            loaded.epoch == job.epoch && Arc::ptr_eq(&loaded.model, &job.model)
        }) && self.encoding == job.encoding
            && self.docs.get(&job.path).map(|d| d.version) == Some(job.version);
        if !current {
            return;
        }
        for (path, found) in results {
            self.set_prose(&path, found);
        }
    }

    /// Replaces what's kept for a file, and publishes the file's
    /// diagnostics again when that changed it.
    fn set_prose(&mut self, path: &Path, found: Vec<lsp_types::Diagnostic>) {
        let before = self.prose.found.get(path).map_or(&[][..], Vec::as_slice);
        if before == found.as_slice() {
            return;
        }
        if found.is_empty() {
            self.prose.found.remove(path);
        } else {
            self.prose.found.insert(path.to_path_buf(), found);
        }
        self.republish(path);
    }

    /// An open document was edited: what's kept for it from where the
    /// first change starts on goes, and what's above stays.
    pub(crate) fn prose_edited(&mut self, path: &Path, changes: &[TextDocumentContentChangeEvent]) {
        let Some(kept) = self.prose.found.get_mut(path) else {
            return;
        };
        let from = changes
            .iter()
            .map(|c| c.range.map_or(Position::new(0, 0), |r| r.start))
            .min();
        let Some(from) = from else {
            return;
        };
        kept.retain(|d| d.range.end < from);
        if kept.is_empty() {
            self.prose.found.remove(path);
        }
        self.prose.queue.retain(|p| p != path);
    }

    /// An open document was closed: what's kept for it goes.
    pub(crate) fn prose_closed(&mut self, path: &Path) {
        if self.prose_closed_quietly(path) {
            self.republish(path);
        }
    }

    /// What's kept for a file goes, without publishing: whether there was
    /// anything.
    pub(crate) fn prose_closed_quietly(&mut self, path: &Path) -> bool {
        self.prose.queue.retain(|p| p != path);
        self.prose.found.remove(path).is_some()
    }

    /// The content model changed: when `[checks.vale]` did, what's kept
    /// goes, and the open documents are checked again with the new
    /// settings. When only where it's written moved, the advice that Vale
    /// couldn't run, which is on it, goes. `ascribe.toml`'s own diagnostics
    /// are published next.
    pub(crate) fn prose_settings_changed(&mut self, before: Option<&ValeSettings>) {
        let now = self
            .loaded
            .as_ref()
            .and_then(|l| l.model.checks.vale.as_ref());
        if same_settings(before, now) {
            if before.map(|b| b.span) != now.map(|n| n.span)
                && let Some(config) = self.config.clone()
            {
                self.prose.found.remove(&config);
            }
            return;
        }
        let kept: Vec<PathBuf> = self.prose.found.keys().cloned().collect();
        self.prose.found.clear();
        self.prose.queue.clear();
        for path in kept {
            self.republish(path.as_path());
        }
        self.queue_all_prose();
    }

    /// Publishes a file's diagnostics again, with what's kept for it now.
    fn republish(&mut self, path: &Path) {
        let own = self.checked(path).to_vec();
        self.publish(path, own);
    }
}

/// Whether two `[checks.vale]` tables check the same way: everything but
/// where they're written.
fn same_settings(a: Option<&ValeSettings>, b: Option<&ValeSettings>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => {
            a.source == b.source
                && a.command == b.command
                && a.in_check == b.in_check
                && a.max_level == b.max_level
                && a.off == b.off
        }
        _ => false,
    }
}
