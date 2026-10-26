//! Computing diagnostics: the worker's side.
//!
//! A [`Job`] is what one round has to do, taken from the state under the lock.
//! [`compute`] runs without the lock, from a [`Snapshot`], calling the same
//! functions `ascribe check` does (`ascribe_check::check_file`, and the
//! page-level checks of the editor's build); [`Core::finish`] publishes under
//! the lock, and only what is still current.
//!
//! The content checks across the project (`page-orphan`, the unused
//! fragments and content model entries, `title-duplicate`,
//! `description-duplicate`) need every page
//! of the build, so they don't run at every round: a round runs them when
//! the project is loaded, when a file is saved or changes on disk, and when
//! the content model changes ([`Job::across`]). What they found is kept
//! ([`Across`]) and added to each file's diagnostics while the file's text
//! is still the one it was found in; an edit hides them until the next save.
//!
//! It doesn't call `ascribe_check::diagnose`, the entry `ascribe check`
//! uses: that checks every build of a whole project at once, and the server
//! checks only the files an edit affects, in the editor's one build, over the
//! index it keeps current. `crates/ascribe-cli/tests/all/lsp_parity.rs` holds
//! the two to the same diagnostics.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use ascribe_check::identity::Identity;
use ascribe_check::{
    ADDRESSES_FILE_ID, Acknowledgements, Diagnostic, NAV_FILE_ID, PERMALINKS_FILE_ID, PageChecker,
    Project, ReadFailure, Registry, Severity, SourceFile, apply_levels, check_file,
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
    /// Whether the round runs the content checks across the project.
    pub across: bool,
    /// What they found when they last ran.
    pub found: Arc<Across>,
    /// The day the checks run on.
    pub today: Option<ascribe_core::Date>,
    /// `ascribe.baseline`, whose advice isn't published.
    pub baseline: Arc<crate::baseline::Now>,
}

/// What the content checks across the project found when they last ran,
/// each with the text it's located in, levels not yet applied.
#[derive(Default, PartialEq)]
pub(crate) struct Across {
    /// Each source file's, and its text.
    pub sources: BTreeMap<RelPath, (Arc<str>, Vec<lsp_types::Diagnostic>)>,
    /// The content model's, and its text.
    pub model: (String, Vec<Diagnostic>),
    /// The permalinks list's, `ascribe.permalinks`, which isn't a source.
    pub list: Vec<lsp_types::Diagnostic>,
    /// The navigation file's, `ascribe.nav`, which isn't a source either.
    pub nav: Vec<lsp_types::Diagnostic>,
    /// `ascribe.addresses`' permalinks' records', checked against the
    /// content (SPEC §7.6). Its other problems are the addresses worker's,
    /// which it sends on its own.
    pub addresses: Vec<lsp_types::Diagnostic>,
    /// Each memo file's, committed and personal (SPEC §7.7), by its path
    /// from the project root, levels applied: every memo file the checks
    /// read, with nothing for one that has no problem.
    pub memos: BTreeMap<RelPath, Vec<lsp_types::Diagnostic>>,
    /// What `ascribe.baseline` holds back of each source file's, and its
    /// text.
    pub held: BTreeMap<RelPath, (Arc<str>, Vec<lsp_types::Diagnostic>)>,
    /// The identities of what they found that an entry of the baseline
    /// matches, held back or not.
    pub matched: Vec<Identity>,
}

impl Across {
    /// What's kept for a source file whose text is now `text`: nothing once
    /// it's been edited since the checks ran.
    fn of_source(&self, path: &RelPath, text: &str) -> &[lsp_types::Diagnostic] {
        match self.sources.get(path) {
            Some((was, found)) if **was == *text => found,
            _ => &[],
        }
    }

    /// What the baseline holds back of a source file's, while its text is
    /// still `text`.
    fn of_held(&self, path: &RelPath, text: &str) -> &[lsp_types::Diagnostic] {
        match self.held.get(path) {
            Some((was, held)) if **was == *text => held,
            _ => &[],
        }
    }

    /// What's kept for the content model, when its text is still `text`.
    pub(crate) fn of_model(&self, text: &str) -> &[Diagnostic] {
        if self.model.0 == text {
            &self.model.1
        } else {
            &[]
        }
    }
}

/// What a file's problems put on the code files its `@snippet`s show: each
/// file's absolute path, and the diagnostic published there.
pub(crate) type Shown = Vec<(PathBuf, lsp_types::Diagnostic)>;

/// What a round found in one file: its diagnostics, what they put on code
/// files, what the baseline holds back of them, and the identities of
/// those an entry of the baseline matches, held back or not.
pub(crate) type FileOutcome = (
    RelPath,
    Vec<lsp_types::Diagnostic>,
    Shown,
    Vec<lsp_types::Diagnostic>,
    Vec<Identity>,
);

/// What a round produced: what it found in each file it computed, and what
/// the content checks across the project found when the round ran them.
pub(crate) enum Outcome {
    Done(Vec<FileOutcome>, Option<Box<Across>>),
    /// A newer update overtook the round before it finished; its files are
    /// queued again.
    Abandoned,
}

impl Core {
    /// Takes the work to do, if there is any.
    pub(crate) fn plan(&mut self) -> Option<Job> {
        let encoding = self.encoding;
        let today = self.today();
        // Nothing is published while the model doesn't load; what's queued
        // waits for it.
        if self.model_problem.is_some() {
            return None;
        }
        let loaded = self.loaded.as_mut()?;
        if loaded.dirty.is_empty() && !loaded.across_dirty {
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
            across: std::mem::take(&mut loaded.across_dirty),
            found: loaded.across.clone(),
            today,
            baseline: loaded.baseline.clone(),
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
        let (results, across) = match outcome {
            Outcome::Done(results, across) => (results, across),
            Outcome::Abandoned => {
                loaded.dirty.extend(job.files.iter().cloned());
                loaded.across_dirty |= job.across;
                return;
            }
        };
        // What the content checks across the project found replaces what
        // they found before; a file whose share changed and that this round
        // didn't compute is computed again, and `ascribe.toml` is published.
        let mut model_changed = false;
        let mut list_changed = false;
        let mut nav_changed = false;
        let mut addresses_changed = false;
        let mut memos_changed: Vec<(RelPath, Vec<lsp_types::Diagnostic>)> = Vec::new();
        if let Some(across) = across
            && *across != *loaded.across
        {
            let paths: BTreeSet<&RelPath> = across
                .sources
                .keys()
                .chain(loaded.across.sources.keys())
                .collect();
            for path in paths {
                if across.sources.get(path) != loaded.across.sources.get(path)
                    && !job.files.contains(path)
                {
                    loaded.dirty.insert(path.clone());
                }
            }
            model_changed = across.model != loaded.across.model;
            list_changed = across.list != loaded.across.list;
            nav_changed = across.nav != loaded.across.nav;
            addresses_changed = across.addresses != loaded.across.addresses;
            // A memo file that's gone is published empty, once.
            for path in loaded.across.memos.keys() {
                if !across.memos.contains_key(path) {
                    memos_changed.push((path.clone(), Vec::new()));
                }
            }
            for (path, found) in &across.memos {
                if loaded.across.memos.get(path) != Some(found) {
                    memos_changed.push((path.clone(), found.clone()));
                }
            }
            loaded.across = Arc::new(*across);
        }
        let mut ready = Vec::new();
        for (path, diagnostics, shown, held, matched) in results {
            let abs = loaded.source_path(&path);
            let current = loaded.inc.is_file_current(&job.snapshot, &path);
            let version = self.docs.get(&abs).map(|d| d.version);
            if current && version == job.versions.get(&path).copied() {
                loaded.matched.insert(path, matched);
                ready.push((abs, diagnostics, shown, held, version));
            } else {
                loaded.dirty.insert(path);
            }
        }
        for (abs, diagnostics, shown, held, version) in ready {
            self.publish(&abs, diagnostics);
            self.show_on_code_files(&abs, shown);
            self.set_held(&abs, held, version);
        }
        if model_changed {
            self.publish_model_diagnostics();
        }
        if let Some(loaded) = self.loaded.as_ref() {
            let root = loaded.root.clone();
            for (path, found) in memos_changed {
                self.publish(&normalize(&root.join(path.as_str())), found);
            }
        }
        if list_changed && let Some(loaded) = self.loaded.as_ref() {
            let path = loaded.root.join(ascribe_model::PERMALINKS_FILE);
            let found = loaded.across.list.clone();
            self.publish(&path, found);
        }
        if addresses_changed && let Some(loaded) = self.loaded.as_ref() {
            let path = loaded.root.join(ascribe_model::ADDRESSES_FILE);
            let found = loaded.across.addresses.clone();
            self.publish(&path, found);
        }
        if nav_changed && let Some(loaded) = self.loaded.as_ref() {
            let path = loaded.root.join(ascribe_model::NAV_FILE);
            let found = loaded.across.nav.clone();
            self.publish(&path, found);
        }
        self.publish_baseline();
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
    // A versioned build is checked in every line, the editor's line first,
    // and a problem that isn't in every line says which it's in.
    let mut lines: Vec<&ascribe_model::Build> = vec![&build];
    if build.line.is_some() {
        lines.extend(job.model.builds.iter().filter(|b| {
            b.line.is_some() && b.declared_name() == build.declared_name() && b.name != build.name
        }));
    }
    let router = DefaultRouter::from_consumer(&job.model.consumer);
    let resolved: Vec<Vec<Arc<ResolvedPage>>> = {
        let mut cache = job.cache.lock().unwrap_or_else(PoisonError::into_inner);
        for affected in &job.affected {
            cache.apply(affected);
        }
        lines
            .iter()
            .map(|line| {
                pages
                    .iter()
                    .filter_map(|path| cache.resolve_page(snapshot.project(), path, line, &router))
                    .collect()
            })
            .collect()
    };
    let per_line: Vec<(&ascribe_model::Build, Vec<&ResolvedPage>)> = lines
        .iter()
        .zip(&resolved)
        .map(|(line, pages)| (*line, pages.iter().map(|p| &**p).collect()))
        .collect();
    let checker = PageChecker::with_index(&project, snapshot.project());
    for mut d in checker.check_resolved_builds(&per_line) {
        if lines.len() > 1
            && let Some(note) = d.builds_note_in(&lines)
        {
            d.message = format!("{} ({note})", d.message);
        }
        if let Some(list) = by_file.get_mut(&d.location.file) {
            list.push(d);
        }
    }
    let across = if job.across {
        if !still_wanted() {
            return Outcome::Abandoned;
        }
        Some(Box::new(run_across(
            job, &project, &checker, &build, &router,
        )))
    } else {
        None
    };
    let found: &Across = across.as_deref().unwrap_or(&job.found);
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
                ascribe_check::generated::mark(
                    &project,
                    apply_levels(
                        &job.model.checks,
                        by_file.remove(&file.id).unwrap_or_default(),
                    ),
                ),
                false,
            )
            .diagnostics;
        let index = LineIndex::new(&file.text);
        // What `ascribe.baseline` lists isn't published, and is kept for
        // the hover and the quick fix that moves it to an `@intended`.
        let held = crate::baseline::hold(&job.baseline, &project, diagnostics);
        let mut held_lsp = crate::baseline::held_to_lsp(&held, &index, job.encoding);
        held_lsp.extend_from_slice(found.of_held(path, &file.text));
        let diagnostics = held.diagnostics;
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
        let mut lsp: Vec<lsp_types::Diagnostic> = diagnostics
            .iter()
            .map(|d| to_lsp(d, &index, job.encoding, &related))
            .collect();
        let shown = on_code_files(&diagnostics, &lsp, &project, job, file.id);
        lsp.extend_from_slice(found.of_source(path, &file.text));
        results.push((path.clone(), lsp, shown, held_lsp, held.matched));
    }
    Outcome::Done(results, across)
}

/// A file's `snippet-tags` problems (ASC132) as they're shown on the code
/// files whose tags they're about, at the tag's line: the same problem as the
/// file's own, `lsp` in `diagnostics`' order, with the other places in the
/// code file it names and the `@snippet` that shows it as related places, and
/// no fixes, which are the file's. `ascribe check` reports nothing on a code
/// file; this is the editor's alone.
fn on_code_files(
    diagnostics: &[Diagnostic],
    lsp: &[lsp_types::Diagnostic],
    project: &Project,
    job: &Job,
    id: FileId,
) -> Shown {
    let Some(page) = project
        .file(id)
        .and_then(|entry| path_to_uri(&normalize(&job.root.join(&entry.display_path))))
    else {
        return Vec::new();
    };
    let mut shown = Vec::new();
    for (d, on_page) in diagnostics.iter().zip(lsp) {
        if d.slug != ascribe_core::diagnostics::SNIPPET_TAGS {
            continue;
        }
        // The tag itself is the last place a `snippet-tags` problem names.
        let Some((here, before)) = d.related.split_last() else {
            continue;
        };
        let Some(code) = project.code_file(here.location.file) else {
            continue;
        };
        let abs = normalize(&job.root.join(code.path.as_str()));
        let Some(uri) = path_to_uri(&abs) else {
            continue;
        };
        let index = LineIndex::new(&code.text);
        let mut related: Vec<DiagnosticRelatedInformation> = before
            .iter()
            .filter(|r| r.location.file == code.id)
            .map(|r| DiagnosticRelatedInformation {
                location: Location {
                    uri: uri.clone(),
                    range: job.encoding.range(&index, r.location.span),
                },
                message: r.message.clone(),
            })
            .collect();
        related.push(DiagnosticRelatedInformation {
            location: Location {
                uri: page.clone(),
                range: on_page.range,
            },
            message: "the `@snippet` that shows it".to_owned(),
        });
        let mut d = on_page.clone();
        d.range = job.encoding.range(&index, here.location.span);
        d.related_information = Some(related);
        if let Some(map) = d.data.as_mut().and_then(serde_json::Value::as_object_mut) {
            map.insert("fixes".to_owned(), serde_json::json!([]));
        }
        shown.push((abs, d));
    }
    shown
}

/// Runs the content checks across the project for the editor's build, over
/// every page it publishes, and keeps what they found with the text it's
/// located in, levels applied for a source file.
fn run_across(
    job: &Job,
    project: &Project,
    checker: &PageChecker<'_>,
    build: &ascribe_model::Build,
    router: &DefaultRouter,
) -> Across {
    let snapshot = &job.snapshot;
    let mut paths: Vec<&RelPath> = snapshot
        .files()
        .filter(|f| f.kind == FileKind::Page)
        .map(|f| &f.path)
        .collect();
    paths.sort();
    let pages: Vec<Arc<ResolvedPage>> = {
        let mut cache = job.cache.lock().unwrap_or_else(PoisonError::into_inner);
        paths
            .into_iter()
            .filter_map(|path| cache.resolve_page(snapshot.project(), path, build, router))
            .collect()
    };
    let refs: Vec<&ResolvedPage> = pages.iter().map(|p| &**p).collect();
    let mut by_file: BTreeMap<FileId, Vec<Diagnostic>> = BTreeMap::new();
    for d in checker.check_across(build, &refs) {
        by_file.entry(d.location.file).or_default().push(d);
    }
    // What an acknowledgement covers isn't reported (SPEC §4.9); whether one
    // covers nothing takes every build, so the editor never says.
    let acknowledgements = Acknowledgements::of(project);
    // What `ascribe.baseline` lists isn't published, a file at a time; the
    // identities it matched are kept, so an entry isn't stale.
    let mut matched: Vec<Identity> = Vec::new();
    let mut hold = |diagnostics: Vec<Diagnostic>| {
        let held = crate::baseline::hold(&job.baseline, project, diagnostics);
        matched.extend(held.matched.iter().cloned());
        held
    };
    let mut found = Across {
        model: (
            job.model_text.clone(),
            hold(
                acknowledgements
                    .apply(
                        &job.model,
                        by_file.remove(&FileId::new(0)).unwrap_or_default(),
                        false,
                    )
                    .diagnostics,
            )
            .diagnostics,
        ),
        ..Across::default()
    };
    // The permalinks list's own, the navigation file's, and the permalinks'
    // records' in `ascribe.addresses`, at the levels `[checks]` sets (none
    // of them is configurable, so as they are).
    for (id, out) in [
        (PERMALINKS_FILE_ID, &mut found.list),
        (NAV_FILE_ID, &mut found.nav),
        (ADDRESSES_FILE_ID, &mut found.addresses),
    ] {
        let list = hold(by_file.remove(&id).unwrap_or_default()).diagnostics;
        if let Some(entry) = project.file(id) {
            let index = LineIndex::new(entry.text);
            *out = list
                .iter()
                .map(|d| to_lsp(d, &index, job.encoding, &|_| None))
                .collect();
        }
    }
    // Every memo file gets an entry, so one whose problems are fixed is
    // published empty.
    for id in project.memo_ids() {
        let Some(entry) = project.file(id) else {
            continue;
        };
        let Ok(path) = RelPath::parse(&entry.display_path) else {
            continue;
        };
        let diagnostics = hold(apply_levels(
            &job.model.checks,
            by_file.remove(&id).unwrap_or_default(),
        ))
        .diagnostics;
        let index = LineIndex::new(entry.text);
        let lsp = diagnostics
            .iter()
            .map(|d| to_lsp(d, &index, job.encoding, &|_| None))
            .collect();
        found.memos.insert(path, lsp);
    }
    for (id, diagnostics) in by_file {
        let Some(entry) = project.file(id) else {
            continue;
        };
        let Some(path) = entry.content_path else {
            continue;
        };
        let Some(file) = snapshot.file(path) else {
            continue;
        };
        let held = hold(
            acknowledgements
                .apply(
                    &job.model,
                    ascribe_check::generated::mark(
                        project,
                        apply_levels(&job.model.checks, diagnostics),
                    ),
                    false,
                )
                .diagnostics,
        );
        let index = LineIndex::new(entry.text);
        let held_lsp = crate::baseline::held_to_lsp(&held, &index, job.encoding);
        if !held_lsp.is_empty() {
            found
                .held
                .insert(path.clone(), (file.source.clone(), held_lsp));
        }
        let diagnostics = held.diagnostics;
        if diagnostics.is_empty() {
            continue;
        }
        let related = |r: &ascribe_check::RelatedInfo| -> Option<Location> {
            let other = project.file(r.location.file)?;
            let abs = normalize(&job.root.join(&other.display_path));
            let range = job
                .encoding
                .range(&LineIndex::new(other.text), r.location.span);
            Some(Location {
                uri: path_to_uri(&abs)?,
                range,
            })
        };
        let lsp = diagnostics
            .iter()
            .map(|d| {
                let mut lsp = to_lsp(d, &index, job.encoding, &related);
                fixes_elsewhere(&mut lsp, d, project, &job.root, job.encoding);
                lsp
            })
            .collect();
        found
            .sources
            .insert(path.clone(), (file.source.clone(), lsp));
    }
    found.matched = matched;
    found
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
    .with_today(job.today)
    // The editor reports a personal memo's problems as a team memo's.
    .with_personal_memos()
    // The snapshot's index is of the same files, with the same ids.
    .with_link_index(job.snapshot.shared_project())
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

/// Adds to `lsp`'s data the fixes of `d` that edit another file the project
/// has, as `elsewhere`: each with that file's URI, and its edits in that
/// file's positions. A page's `nav-page-unplaced` is fixed in `ascribe.nav`.
fn fixes_elsewhere(
    lsp: &mut lsp_types::Diagnostic,
    d: &Diagnostic,
    project: &Project,
    root: &Path,
    encoding: Encoding,
) {
    let elsewhere: Vec<serde_json::Value> = d
        .fixes
        .iter()
        .filter(|fix| fix.file != d.location.file)
        .filter_map(|fix| {
            let entry = project.file(fix.file)?;
            let uri = path_to_uri(&normalize(&root.join(&entry.display_path)))?;
            let index = LineIndex::new(entry.text);
            Some(serde_json::json!({
                "title": fix.title,
                "applicability": fix.applicability.as_str(),
                "uri": uri.as_str(),
                "edits": fix.edits.iter().map(|edit| serde_json::json!({
                    "range": encoding.range(&index, edit.span),
                    "newText": edit.new_text,
                })).collect::<Vec<_>>(),
            }))
        })
        .collect();
    if elsewhere.is_empty() {
        return;
    }
    if let Some(map) = lsp.data.as_mut().and_then(serde_json::Value::as_object_mut) {
        map.insert("elsewhere".to_owned(), elsewhere.into());
    }
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
            "next": d.next().map(ascribe_check::Next::as_str),
            "rule": d.rule,
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
