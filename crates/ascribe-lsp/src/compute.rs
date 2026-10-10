//! Computing diagnostics: the worker's side.
//!
//! A [`Job`] is what one round has to do, taken from the state under the lock.
//! [`compute`] runs without the lock, from a [`Snapshot`], calling the same
//! functions `ascribe check` does (`ascribe_check::check_file`, and the
//! page-level checks of the editor's build); [`Core::finish`] publishes under
//! the lock, and only what is still current.
//!
//! It doesn't call `ascribe_check::diagnose`, the entry `ascribe check`
//! uses: that checks every build of a whole project at once, and the server
//! checks only the files an edit affects, in the editor's one build, over the
//! index it keeps current. `crates/ascribe-cli/tests/all/lsp_parity.rs` holds
//! the two to the same diagnostics.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};

use ascribe_check::{
    Acknowledgements, Diagnostic, PageChecker, Project, ReadFailure, Registry, Severity,
    SourceFile, apply_levels, check_file,
};
use ascribe_core::path::normalize;
use ascribe_core::{FileId, LineIndex, RelPath};
use ascribe_model::ContentModel;
use ascribe_resolve::{Affected, DefaultRouter, FileKind, ResolvedCache, ResolvedPage, Snapshot};
use lsp_types::{
    CodeDescription, DiagnosticRelatedInformation, DiagnosticSeverity, Location, NumberOrString,
};

use crate::core::Core;
use crate::fsx::LayerFs;
use crate::position::Encoding;
use crate::uri::path_to_uri;

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
    /// The resolved pages of the editor's build, and the updates to apply to
    /// them before using them.
    pub cache: Arc<Mutex<ResolvedCache>>,
    pub affected: Vec<Affected>,
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
        let snapshot = loaded.inc.snapshot();
        let files = with_included(
            &snapshot,
            &mut loaded.direct_includes,
            std::mem::take(&mut loaded.dirty),
        );
        let versions = files
            .iter()
            .filter_map(|path| {
                let doc = self.docs.get(&loaded.source_path(path))?;
                Some((path.clone(), doc.version))
            })
            .collect();
        Some(Job {
            epoch: loaded.epoch,
            snapshot,
            files,
            root: loaded.root.clone(),
            model: loaded.model.clone(),
            model_text: loaded.model_text.clone(),
            fs: loaded.fs.clone(),
            encoding,
            versions,
            cache: loaded.cache.clone(),
            affected: std::mem::take(&mut loaded.pending),
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

/// `files` and everything they include, now or at the last round, directly or
/// through other files. `direct` is what each file included at the last round;
/// it's updated for `files`.
///
/// A page that starts (or stops) including a fragment changes the page-level
/// diagnostics located *in that fragment* (an `include-cycle` is located where
/// the cycle closes, and a fragment nobody includes has none), yet
/// `Affected::recheck` lists the page and not the fragment, whose own text
/// didn't change. So a round covers what the files in it reach, and what they
/// reached before.
fn with_included(
    snapshot: &Snapshot,
    direct: &mut BTreeMap<RelPath, BTreeSet<RelPath>>,
    files: BTreeSet<RelPath>,
) -> BTreeSet<RelPath> {
    let mut all = files.clone();
    let mut queue: Vec<RelPath> = Vec::new();
    for path in &files {
        let now: BTreeSet<RelPath> = snapshot
            .file(path)
            .map(|file| {
                file.includes
                    .iter()
                    .filter_map(|i| i.target.clone())
                    .filter(|t| snapshot.file(t).is_some())
                    .collect()
            })
            .unwrap_or_default();
        let before = direct.insert(path.clone(), now.clone()).unwrap_or_default();
        for target in now.into_iter().chain(before) {
            if all.insert(target.clone()) {
                queue.push(target);
            }
        }
    }
    while let Some(path) = queue.pop() {
        for target in direct.get(&path).into_iter().flatten() {
            if all.insert(target.clone()) {
                queue.push(target.clone());
            }
        }
    }
    // What the files reached in turn is now known for the next round.
    for path in &all {
        if !direct.contains_key(path) {
            let now = snapshot
                .file(path)
                .map(|file| {
                    file.includes
                        .iter()
                        .filter_map(|i| i.target.clone())
                        .filter(|t| snapshot.file(t).is_some())
                        .collect()
                })
                .unwrap_or_default();
            direct.insert(path.clone(), now);
        }
    }
    all
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
    // Page level: the editor's build, for the pages that can have a diagnostic
    // located in a file of the round: the file itself when it's a page, and
    // the pages that include it (a cycle in a fragment is located in the
    // fragment). Their resolved forms come from the cache, which first forgets
    // what the updates since the last round affected; the checks read the
    // snapshot's own index. Keeping what's located in the files of the round.
    // Only the editor's build, not content no build
    // publishes.
    let build = job.model.editor_default_build().clone();
    let mut pages: BTreeSet<RelPath> = BTreeSet::new();
    for path in &job.files {
        if snapshot
            .file(path)
            .is_some_and(|f| f.kind == FileKind::Page)
        {
            pages.insert(path.clone());
        }
        pages.extend(snapshot.including_pages(path));
    }
    let router = DefaultRouter::from_consumer(&job.model.consumer);
    let resolved: Vec<Arc<ResolvedPage>> = {
        let mut cache = job.cache.lock().unwrap_or_else(PoisonError::into_inner);
        for affected in &job.affected {
            cache.apply(affected);
        }
        pages
            .iter()
            .filter_map(|path| cache.resolve_page(snapshot.project(), path, &build, &router))
            .collect()
    };
    let refs: Vec<&ResolvedPage> = resolved.iter().map(|p| &**p).collect();
    for d in PageChecker::with_index(&project, snapshot.project()).check_resolved(&build, &refs) {
        if let Some(list) = by_file.get_mut(&d.location.file) {
            list.push(d);
        }
    }
    // The acknowledgements that can cover them: in the content model, in
    // their files, and in the files of their related places (a block that
    // includes a fragment covers a problem in what it includes).
    let mut written: BTreeSet<FileId> = by_file.keys().copied().collect();
    written.extend(
        by_file
            .values()
            .flatten()
            .flat_map(|d| d.related.iter().map(|r| r.location.file)),
    );
    let written: Vec<FileId> = written.into_iter().collect();
    let acknowledgements = Acknowledgements::in_files(&project, &written);
    let mut results = Vec::new();
    for path in &job.files {
        let Some(file) = project.source_at(path) else {
            continue;
        };
        // Whether an acknowledgement covers nothing takes every build, so
        // the editor never says.
        let diagnostics = acknowledgements
            .apply(
                &job.model,
                apply_levels(
                    &job.model.checks,
                    by_file.remove(&file.id).unwrap_or_default(),
                ),
                false,
            )
            .diagnostics;
        let index = LineIndex::new(&file.text);
        let related = |r: &ascribe_check::RelatedInfo| -> Option<Location> {
            // A related location is in a source, the content model, or a code
            // file a snippet reads.
            let (path, index) = match project.file(r.location.file) {
                Some(entry) => (entry.display_path, LineIndex::new(entry.text)),
                None => {
                    let code = project.code_file(r.location.file)?;
                    (code.path.to_string(), LineIndex::new(&code.text))
                }
            };
            let abs = normalize(&job.root.join(&path));
            let uri = path_to_uri(&abs)?;
            let range = job.encoding.range(&index, r.location.span);
            Some(Location { uri, range })
        };
        let lsp = diagnostics
            .iter()
            .map(|d| to_lsp(d, &index, job.encoding, &related))
            .collect();
        results.push((path.clone(), lsp));
    }
    Outcome::Done(results)
}

/// The checked project of a snapshot: its sources with the snapshot's ids, and
/// the files the editor and the watcher reported over the disk.
fn check_project_of(job: &Job) -> Project {
    checked_project(
        &job.snapshot,
        job.root.clone(),
        &job.model,
        &job.model_text,
        &job.fs,
        &[],
    )
}

/// The checked project of `snapshot`, with each of `replaced`'s files holding
/// the text given instead of its own: what `ascribe/edit` checks an edit's
/// result in.
pub(crate) fn checked_project(
    snapshot: &Snapshot,
    root: PathBuf,
    model: &ContentModel,
    model_text: &str,
    fs: &Arc<LayerFs>,
    replaced: &[(&RelPath, &str)],
) -> Project {
    let mut sources: Vec<SourceFile> = snapshot
        .files()
        .map(|f| SourceFile {
            id: f.file,
            path: f.path.clone(),
            text: match replaced.iter().find(|(path, _)| **path == f.path) {
                Some((_, text)) => (*text).to_owned(),
                None => f.source.to_string(),
            },
            unreadable: None,
        })
        .collect();
    // Files the project couldn't read are still sources, reported and not
    // checked, with ids past every other.
    let first = snapshot
        .files()
        .map(|f| f.file.index())
        .max()
        .map_or(1, |max| max + 1);
    for (id, unreadable) in (first..).zip(snapshot.unreadable()) {
        sources.push(SourceFile {
            id: FileId::new(id),
            path: unreadable.path.clone(),
            text: String::new(),
            unreadable: Some(ReadFailure {
                reason: unreadable.reason.clone(),
                not_utf8: unreadable.reason.contains("valid UTF-8"),
            }),
        });
    }
    sources.sort_by(|a, b| a.path.cmp(&b.path));
    Project::from_parts_with_fs(
        root,
        snapshot.layout().content_root.clone(),
        model.clone(),
        model_text.to_owned(),
        sources,
        fs.clone(),
    )
}

/// A diagnostic as the protocol has it, positions in `encoding` over `index`
/// (the text of the diagnostic's own file).
pub(crate) fn to_lsp(
    d: &Diagnostic,
    index: &LineIndex,
    encoding: Encoding,
    related: &dyn Fn(&ascribe_check::RelatedInfo) -> Option<Location>,
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
            Severity::Advice => DiagnosticSeverity::INFORMATION,
        }),
        code: Some(NumberOrString::String(d.code.to_owned())),
        // The code links to its entry in the diagnostics reference.
        code_description: Registry::global()
            .get(d.slug)
            .and_then(|entry| entry.docs().parse().ok())
            .map(|href| CodeDescription { href }),
        source: Some("ascribe".to_owned()),
        message: d.message.clone(),
        related_information: (!related_information.is_empty()).then_some(related_information),
        tags: None,
        // What `ascribe check --format json` says besides the protocol's
        // fields, for the extension's tool that reports problems to agents
        // in that shape.
        data: Some(serde_json::json!({
            "slug": d.slug.as_str(),
            "next": Registry::global().get(d.slug).and_then(|entry| entry.next).map(ascribe_check::Next::as_str),
            "builds": d.builds,
            "unpublished": d.unpublished,
            "help": Registry::global().get(d.slug).and_then(|entry| entry.fix.as_deref()).unwrap_or_default(),
            "fixes": d.fixes.iter().filter(|fix| fix.file == d.location.file).map(|fix| {
                serde_json::json!({
                    "title": fix.title,
                    "applicability": fix.applicability.as_str(),
                    "edits": fix.edits.iter().map(|edit| serde_json::json!({
                        "range": encoding.range(index, edit.span),
                        "newText": edit.new_text,
                    })).collect::<Vec<_>>(),
                })
            }).collect::<Vec<_>>(),
        }))
        .map(|mut data| {
            // The entry the problem names, for the quick fix that
            // acknowledges it in the content model.
            if let (Some(subject), Some(map)) = (&d.subject, data.as_object_mut()) {
                map.insert("subject".to_owned(), subject.clone().into());
            }
            data
        }),
    }
}
