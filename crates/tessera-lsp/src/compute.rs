//! Computing diagnostics: the worker's side.
//!
//! A [`Job`] is what one round has to do, taken from the state under the lock.
//! [`compute`] runs without the lock, from a [`Snapshot`], calling the same
//! functions `tessera check` does (`tessera_check::check_file`, and the
//! page-level checks of the editor's build); [`Core::finish`] publishes under
//! the lock, and only what is still current.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::Arc;

use lsp_types::{
    DiagnosticRelatedInformation, DiagnosticSeverity, Location, NumberOrString,
};
use tessera_check::{Diagnostic, PageChecker, Project, ReadFailure, Severity, SourceFile, check_file};
use tessera_core::{FileId, LineIndex, RelPath};
use tessera_model::ContentModel;
use tessera_resolve::Snapshot;

use crate::core::Core;
use crate::fsx::LayerFs;
use crate::position::Encoding;
use crate::uri::{normalize, path_to_uri};

/// What one round of diagnostics has to do.
pub(crate) struct Job {
    pub epoch: u64,
    pub snapshot: Snapshot,
    /// The files to compute.
    pub files: BTreeSet<RelPath>,
    pub root: PathBuf,
    pub model: Arc<ContentModel>,
    pub model_text: String,
    pub fs: Arc<LayerFs>,
    pub encoding: Encoding,
    /// The version of each open document among `files`, when the job was
    /// taken: what the results are for.
    pub versions: BTreeMap<RelPath, i32>,
}

/// What a round produced: the diagnostics of each file it computed.
pub(crate) enum Outcome {
    Done(Vec<(RelPath, Vec<lsp_types::Diagnostic>)>),
    /// A newer update overtook the round before it finished; its files are
    /// queued again.
    Abandoned,
}

impl Core {
    /// Takes the work to do, if there is any.
    pub(crate) fn plan(&mut self) -> Option<Job> {
        let encoding = self.encoding;
        let loaded = self.loaded.as_mut()?;
        if loaded.dirty.is_empty() {
            return None;
        }
        let files = std::mem::take(&mut loaded.dirty);
        let versions = files
            .iter()
            .filter_map(|path| {
                let doc = self.docs.get(&loaded.source_path(path))?;
                Some((path.clone(), doc.version))
            })
            .collect();
        Some(Job {
            epoch: loaded.epoch,
            snapshot: loaded.inc.snapshot(),
            files,
            root: loaded.root.clone(),
            model: loaded.model.clone(),
            model_text: loaded.model_text.clone(),
            fs: loaded.fs.clone(),
            encoding,
            versions,
        })
    }

    /// Publishes a round's results: each file's diagnostics, when the file's
    /// text and everything its diagnostics depend on are what they were when
    /// the round started, and, for an open document, the version is still the
    /// editor's. Anything else is dropped and queued again.
    pub(crate) fn finish(&mut self, job: &Job, outcome: Outcome) {
        let Some(loaded) = self.loaded.as_mut() else {
            return;
        };
        if loaded.epoch != job.epoch {
            // The project was loaded again since; every file is queued there.
            return;
        }
        let results = match outcome {
            Outcome::Done(results) => results,
            Outcome::Abandoned => {
                loaded.dirty.extend(job.files.iter().cloned());
                return;
            }
        };
        let mut ready = Vec::new();
        for (path, diagnostics) in results {
            let abs = loaded.source_path(&path);
            let current = loaded.inc.is_file_current(&job.snapshot, &path);
            let version = self.docs.get(&abs).map(|d| d.version);
            if current && version == job.versions.get(&path).copied() {
                ready.push((abs, diagnostics));
            } else {
                loaded.dirty.insert(path);
            }
        }
        for (abs, diagnostics) in ready {
            self.publish(&abs, diagnostics);
        }
    }
}

/// Computes a round. Calls `still_wanted` before each stage; when it says a
/// newer update has overtaken the round, the round is abandoned.
pub(crate) fn compute(job: &Job, still_wanted: &dyn Fn() -> bool) -> Outcome {
    let snapshot = &job.snapshot;
    if !still_wanted() {
        return Outcome::Abandoned;
    }
    let project = check_project_of(job);
    let mut by_file: BTreeMap<FileId, Vec<Diagnostic>> = BTreeMap::new();
    for path in &job.files {
        let Some(file) = project.source_at(path) else {
            continue;
        };
        by_file.entry(file.id).or_default();
    }
    // File level: each file in the round, on its own.
    for path in &job.files {
        let Some(file) = project.source_at(path) else {
            continue;
        };
        by_file
            .entry(file.id)
            .or_default()
            .extend(check_file(&project, file));
    }
    if !still_wanted() {
        return Outcome::Abandoned;
    }
    // Page level: the editor's build, over the whole project (the source index
    // is rebuilt from the snapshot's texts), keeping what is located in the
    // files of the round.
    let build = job.model.editor_default_build().clone();
    for d in PageChecker::new(&project).check(&build) {
        if let Some(list) = by_file.get_mut(&d.location.file) {
            list.push(d);
        }
    }
    let mut results = Vec::new();
    for path in &job.files {
        let Some(file) = project.source_at(path) else {
            continue;
        };
        let diagnostics = by_file.remove(&file.id).unwrap_or_default();
        let index = LineIndex::new(&file.text);
        let related = |r: &tessera_check::RelatedInfo| -> Option<Location> {
            let entry = project.file(r.location.file)?;
            let abs = normalize(&job.root.join(&entry.display_path));
            let uri = path_to_uri(&abs)?;
            let range = job
                .encoding
                .range(&LineIndex::new(entry.text), r.location.span);
            Some(Location { uri, range })
        };
        let lsp = diagnostics
            .iter()
            .map(|d| to_lsp(d, &index, job.encoding, &related))
            .collect();
        results.push((path.clone(), lsp));
    }
    let _ = snapshot;
    Outcome::Done(results)
}

/// The checked project of a snapshot: its sources with the snapshot's ids, and
/// the files the editor and the watcher reported over the disk.
fn check_project_of(job: &Job) -> Project {
    let snapshot = &job.snapshot;
    let mut sources: Vec<SourceFile> = snapshot
        .files()
        .map(|f| SourceFile {
            id: f.file,
            path: f.path.clone(),
            text: f.source.to_string(),
            unreadable: None,
        })
        .collect();
    // Files the project couldn't read are still sources, reported and not
    // checked (Q52), with ids past every other.
    let mut next = snapshot
        .files()
        .map(|f| f.file.index())
        .max()
        .map_or(1, |max| max + 1);
    for unreadable in snapshot.unreadable() {
        sources.push(SourceFile {
            id: FileId::new(next),
            path: unreadable.path.clone(),
            text: String::new(),
            unreadable: Some(ReadFailure {
                reason: unreadable.reason.clone(),
                not_utf8: unreadable.reason.contains("valid UTF-8"),
            }),
        });
        next += 1;
    }
    sources.sort_by(|a, b| a.path.cmp(&b.path));
    Project::from_parts_with_fs(
        job.root.clone(),
        snapshot.layout().content_root.clone(),
        (*job.model).clone(),
        job.model_text.clone(),
        sources,
        job.fs.clone(),
    )
}

/// A diagnostic as the protocol has it, positions in `encoding` over `index`
/// (the text of the diagnostic's own file).
pub(crate) fn to_lsp(
    d: &Diagnostic,
    index: &LineIndex,
    encoding: Encoding,
    related: &dyn Fn(&tessera_check::RelatedInfo) -> Option<Location>,
) -> lsp_types::Diagnostic {
    let related_information: Vec<DiagnosticRelatedInformation> = d
        .related
        .iter()
        .filter_map(|r| {
            Some(DiagnosticRelatedInformation {
                location: related(r)?,
                message: r.message.clone(),
            })
        })
        .collect();
    lsp_types::Diagnostic {
        range: encoding.range(index, d.location.span),
        severity: Some(match d.severity {
            Severity::Error => DiagnosticSeverity::ERROR,
            Severity::Warning => DiagnosticSeverity::WARNING,
        }),
        code: Some(NumberOrString::String(d.code.to_owned())),
        code_description: None,
        source: Some("tessera".to_owned()),
        message: d.message.clone(),
        related_information: (!related_information.is_empty()).then_some(related_information),
        tags: None,
        data: Some(serde_json::json!({
            "slug": d.slug.as_str(),
            "builds": d.builds,
            "unpublished": d.unpublished,
        })),
    }
}
