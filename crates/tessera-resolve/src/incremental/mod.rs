//! Incremental updates: keeping the project index and resolved pages correct as
//! files change, redoing only what each change affects.
//!
//! An [`IncrementalProject`] owns a project and a stream of changes
//! ([`Change`]): a file's contents updated, created, deleted, or renamed, a
//! non-source file appearing or disappearing, and the content model
//! changing. [`IncrementalProject::apply`] takes a batch and returns an
//! [`Affected`]: the files whose file-level diagnostics may have changed and
//! the pages whose resolved form may have. The new state is a [`Snapshot`],
//! which is an ordinary [`Project`] tagged with versions.
//!
//! **Correctness is the whole contract.** After any sequence of changes, the
//! snapshot equals a [`Project::load_with_ids`] of the same files with the
//! same ids, and everything not in [`Affected`] is what it was before. The
//! differential property test (`tests/incremental_differential.rs`) checks
//! both after every step of thousands of random sequences.
//!
//! # What each change invalidates
//!
//! | Change | Invalidated |
//! |---|---|
//! | **A source file's contents** | That file's parse and index. Every page that includes it, directly or through other files ([`Affected::re_resolve`]). If what others see of it changed (its frontmatter, headings, directives; see below), also the pages that link to it or to a page that includes it (their link targets, page ids, and empty-text link titles), and the re-check of those files, of the files that include it, and of the pages that include a fragment that links to it. If it is a glossary target, every page. |
//! | **A file created, deleted, or renamed** | The same as a content change of that file, and every reference that resolves, or used to resolve, to that path: links, images, includes, route-like links (which look for `route.md` and `route/index.md`), and case-differing names, found through an index from each file to the paths its references can depend on. A rename is a deletion and a creation. |
//! | **A source file that can't be read** ([`Change::Unreadable`]) | The same as deleting it: it leaves the index. It is listed in [`Project::unreadable`] (and [`Affected::unreadable`]) until a later change to its path, as a file that can't be read at load is. |
//! | **A non-source file created or deleted** ([`Change::AssetCreated`]) | The references to it (the resolution of each changes between an asset and a missing file), and so the pages that contain them. |
//! | **A nested project's `ascribe.toml` created or deleted** | Nothing: it changes which files are sources (a nested project's files aren't), so the batch is refused ([`ApplyError::NestedProjectChanged`]) and the caller loads the project again. A file created, changed, or deleted inside a nested project's folder is a non-source file. |
//! | **The content model** | See [`ModelImpact`]: a new directive keyword or note type reparses every file, including ones no one has open; changed phrases, fragment patterns, slugger, or sources re-index every file (parses are reused); anything else re-checks every file and re-resolves every page. |
//!
//! "What others see of a file" is [`structure_signature`]'s hash: the kind
//! (page or fragment), frontmatter as written, headings, and every directive
//! line with how blocks nest. Editing a paragraph doesn't change it, so it
//! doesn't re-resolve the pages that link to the file. Anything that could
//! change what a link resolves to does.
//!
//! # Caching
//!
//! - **Parse and index**: a file is parsed only when its text or the model's
//!   parse-relevant part (directive schemas and note types) differs from what
//!   the cached parse was made from, by content hash and then by comparison. A
//!   file is indexed when its text, path, or the model's index-relevant part
//!   changed. Unchanged files are never touched: [`Affected::parsed`] and
//!   [`Affected::indexed`] say what was redone, and [`IncrementalProject::stats`]
//!   counts.
//! - **Expansion**: [`Project::expansion`] keeps each page's expanded includes;
//!   an update drops only the pages that (transitively) include a file that
//!   changed.
//! - **Reverse edges** ([`Project::includers`], [`Project::links_to`],
//!   [`Project::asset_users`]) are updated by removing and adding what the
//!   files involved contribute.
//! - **Resolved pages**: [`ResolvedCache`] keeps them per build until an update
//!   lists them in [`Affected::re_resolve`].
//!
//! # Snapshots and versions
//!
//! Every update that changes anything produces a new [`Snapshot`] with a
//! [`Version`], one more than the last. A snapshot never changes; results
//! computed from it (diagnostics, resolved pages) are tagged by its version.
//! Whether a result is still good:
//!
//! - **Coarse:** [`Snapshot::is_current`] is true while no later update
//!   exists. The language server drops a result whose snapshot isn't current
//!   before publishing it.
//! - **Per file:** [`IncrementalProject::is_file_current`] says whether a file's
//!   results from an older snapshot are still right: it is true unless a
//!   later update listed the file in [`Affected::recheck`] or
//!   [`Affected::removed`], or changed the model beyond
//!   [`ModelImpact::Warnings`]. So a result for a file the newest update
//!   didn't touch can be kept, and one it did is recomputed.
//! - [`Snapshot::file_version`] is the version at which a file's text last
//!   changed, and [`Snapshot::model_revision`] counts model changes.
//!
//! An update that changes nothing ([`Affected::is_empty`]) produces no new
//! version.
//!
//! # File ids
//!
//! See [`ids`]: an id names a path for the life of the project, is never
//! reused for another path, and a rename is a deletion plus a creation.
//! `tessera_check::Project` numbers its sources the same way on a fresh load
//! and accepts the ids a snapshot gives.

mod cache;
mod ids;
mod overlay;
mod signature;

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use tessera_core::{RelPath, Slugger};
use tessera_model::ContentModel;

pub use cache::ResolvedCache;
pub use ids::FileIds;
pub use signature::ModelImpact;

use cache::ParseCache;
use overlay::Overlay;
use signature::{Fingerprints, structure_signature};

use crate::fs::{FileSystem, MODEL_FILE, in_nested_project, is_source_path};
use crate::index::{FileIndex, FileKind, Target, index_parsed, parse_source};
use crate::layout::Layout;
use crate::project::{Project, Unreadable};
use crate::slug::{default_slugger, slugger_by_name};

/// One change to the project, as the editor or the file watcher reports it.
///
/// Paths of source files are **content paths**. The changes in one call to
/// [`IncrementalProject::apply`] are taken as a batch: only the net effect on
/// each path counts, so a file created and deleted in one batch was never
/// there, and an edit that restores the old text changes nothing.
#[derive(Clone, Debug)]
pub enum Change {
    /// A file appeared, with this text. A source file is an `.md` file under
    /// the content root, not inside a directory whose name starts with `.` or
    /// a nested project's folder ([`Project::is_source`]). A path that isn't
    /// a source is a non-source file appearing, like [`Change::AssetCreated`].
    Created {
        /// The file's content path.
        path: RelPath,
        /// Its text.
        text: String,
    },
    /// A source file's contents changed: the editor's buffer, or a save. A file
    /// the project doesn't have is created.
    Edited {
        /// The file's content path.
        path: RelPath,
        /// Its new text.
        text: String,
    },
    /// A file was deleted. One the project doesn't have is ignored.
    Deleted {
        /// The file's content path.
        path: RelPath,
    },
    /// A source file exists but can't be read: it isn't valid UTF-8, or the
    /// operating system refuses. It leaves the index, as a deletion does, and
    /// is listed in [`Project::unreadable`], so its `source-unreadable`
    /// diagnostic shows, until a later change to its path. A path that
    /// isn't a source is a non-source file that exists, like
    /// [`Change::AssetCreated`].
    Unreadable {
        /// The file's content path.
        path: RelPath,
        /// The operating system's reason.
        reason: String,
    },
    /// A file moved. A source file keeps its text (send [`Change::Edited`] as
    /// well if the move changed it). Moving a non-source file to a source
    /// path, which needs text the project can't know, deletes the old file;
    /// send [`Change::Created`] for the new one.
    Renamed {
        /// Where it was.
        from: RelPath,
        /// Where it is.
        to: RelPath,
    },
    /// A file that isn't a source appeared (an image, a download), at a path
    /// relative to the **project root**, as [`FileSystem::probe`] takes it.
    AssetCreated {
        /// The path.
        path: RelPath,
    },
    /// A file that isn't a source disappeared, at a path relative to the
    /// project root.
    AssetDeleted {
        /// The path.
        path: RelPath,
    },
    /// `ascribe.toml` changed and loaded: this is the new model. One that moves
    /// the content root or the output directory can't be applied in place
    /// ([`ApplyError::LayoutChanged`]).
    Model(Arc<ContentModel>),
}

/// Why a batch of changes wasn't applied. Nothing changed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApplyError {
    /// The new model has a different content root or output directory, which
    /// changes what every path means. Load a new project instead (the
    /// language server does).
    LayoutChanged,
    /// An `ascribe.toml` appeared in, or disappeared from, a directory below
    /// the content root other than the project's own folder
    /// ([`Project::own_folder`]), which makes that directory another
    /// project's folder or stops it being one ([`Project::nested_projects`]),
    /// and so changes which files are sources. Load a new project instead
    /// (the language server does).
    NestedProjectChanged,
}

impl fmt::Display for ApplyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApplyError::LayoutChanged => f.write_str(
                "the content root or the output directory changed; load the project again",
            ),
            ApplyError::NestedProjectChanged => f.write_str(
                "a project nested in the content root appeared or went away; load the project again",
            ),
        }
    }
}

impl std::error::Error for ApplyError {}

/// The order of snapshots: each update that changes something produces the
/// next.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version(u64);

impl Version {
    /// The number.
    pub fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}", self.0)
    }
}

/// What one update touched. Everything not listed is as it was.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Affected {
    /// The snapshot this update produced. Equal to [`Affected::previous`]
    /// when the update changed nothing.
    pub version: Version,
    /// The snapshot it was applied to.
    pub previous: Version,
    /// Files whose parse was redone: their text (or the model's directive
    /// keywords) differed from what the cached parse was made from.
    pub parsed: BTreeSet<RelPath>,
    /// Files whose index was redone (a superset of `parsed`).
    pub indexed: BTreeSet<RelPath>,
    /// Files that exist now and whose file-level diagnostics
    /// (`tessera_check::check_files`, [`Project::problems`], and the
    /// page-level checks located in them) may differ from before. Includes
    /// every file after a model change beyond [`ModelImpact::Warnings`].
    pub recheck: BTreeSet<RelPath>,
    /// Files that no longer exist: their diagnostics go away, and results
    /// located at their ids are stale.
    pub removed: BTreeSet<RelPath>,
    /// Source files that exist but became unreadable, or whose reason
    /// changed: they left the index (results located at their ids are
    /// stale), and each now has a `source-unreadable` diagnostic
    /// ([`Project::unreadable`]).
    pub unreadable: BTreeSet<RelPath>,
    /// Pages whose resolved form, in any build, may differ from before.
    /// Every page after a model change beyond [`ModelImpact::Warnings`].
    pub re_resolve: BTreeSet<RelPath>,
    /// What a model change reaches; `None` when the model didn't change.
    pub model: Option<ModelImpact>,
}

impl Affected {
    /// Whether the update changed nothing: no file, no model, no version.
    pub fn is_empty(&self) -> bool {
        self.version == self.previous
    }
}

/// A consistent state of the project, tagged with the versions it reflects.
///
/// It dereferences to the [`Project`], so every lookup, expansion, and
/// resolution works on it directly. It never changes, and it's cheap to clone
/// and to hold while later updates happen.
#[derive(Clone)]
pub struct Snapshot {
    project: Arc<Project>,
    version: Version,
    model_revision: u64,
    stamps: Arc<BTreeMap<RelPath, Version>>,
    latest: Arc<AtomicU64>,
}

impl fmt::Debug for Snapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Snapshot")
            .field("version", &self.version)
            .field("model_revision", &self.model_revision)
            .field("files", &self.stamps.len())
            .finish()
    }
}

impl std::ops::Deref for Snapshot {
    type Target = Project;

    fn deref(&self) -> &Project {
        &self.project
    }
}

impl Snapshot {
    /// The project as of this snapshot.
    pub fn project(&self) -> &Project {
        &self.project
    }

    /// This snapshot's version.
    pub fn version(&self) -> Version {
        self.version
    }

    /// How many times the model has changed, as of this snapshot (0 at load).
    pub fn model_revision(&self) -> u64 {
        self.model_revision
    }

    /// The version at which a file's text last changed, as of this snapshot.
    /// The file's own contents only: a file's diagnostics also depend on
    /// others (see [`IncrementalProject::is_file_current`]).
    pub fn file_version(&self, path: &RelPath) -> Option<Version> {
        self.stamps.get(path).copied()
    }

    /// Whether no update has been applied since this snapshot. A result
    /// computed from a snapshot that isn't current may be out of date, and
    /// shouldn't be published without a check
    /// ([`IncrementalProject::is_file_current`]).
    pub fn is_current(&self) -> bool {
        self.latest.load(Ordering::Acquire) == self.version.0
    }
}

/// Counts of the work done so far, for tests and profiling.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    /// Files parsed.
    pub parses: u64,
    /// Files indexed.
    pub indexes: u64,
    /// Files whose references were resolved.
    pub resolutions: u64,
}

// How many updates `is_file_current` can look back over.
const HISTORY: usize = 256;

struct HistoryEntry {
    version: Version,
    /// `None`: every file.
    recheck: Option<BTreeSet<RelPath>>,
    removed: BTreeSet<RelPath>,
}

/// Which files' references can depend on which paths, so a file appearing or
/// disappearing finds the references it affects.
#[derive(Default)]
struct Deps {
    /// A lowercased path relative to the project root to the files with a
    /// reference that looks at it (as a file, a case-differing twin, or the
    /// `.md` or `index.md` a route-like link maps to).
    by_key: BTreeMap<String, BTreeSet<RelPath>>,
    keys_of: BTreeMap<RelPath, BTreeSet<String>>,
}

impl Deps {
    fn set(&mut self, layout: &Layout, index: &FileIndex) {
        self.remove(&index.path);
        let mut keys = BTreeSet::new();
        for reference in &index.references {
            let Target::Local(local) = &reference.target else {
                continue;
            };
            let Some(path) = &local.path else {
                continue;
            };
            let mut candidates = vec![path.clone()];
            if path.is_root() {
                candidates.extend(RelPath::parse("index.md").ok());
            } else {
                candidates.extend(RelPath::parse(&format!("{path}.md")).ok());
                candidates.extend(path.join("index.md").ok());
            }
            for candidate in candidates {
                keys.insert(fold(&layout.project_path(&candidate)));
            }
        }
        for key in &keys {
            self.by_key
                .entry(key.clone())
                .or_default()
                .insert(index.path.clone());
        }
        self.keys_of.insert(index.path.clone(), keys);
    }

    fn remove(&mut self, path: &RelPath) {
        for key in self.keys_of.remove(path).unwrap_or_default() {
            if let Some(files) = self.by_key.get_mut(&key) {
                files.remove(path);
                if files.is_empty() {
                    self.by_key.remove(&key);
                }
            }
        }
    }

    fn files_at(&self, key: &str) -> impl Iterator<Item = &RelPath> {
        self.by_key.get(key).into_iter().flatten()
    }
}

fn fold(path: &RelPath) -> String {
    path.as_str().to_lowercase()
}

/// The folder an `ascribe.toml` at this path (relative to the project root)
/// would make a nested project's, as a content path: a directory below the
/// content root, not the content root itself, whose name and whose parents'
/// names don't start with `.`. `None` for any other file. The project's own
/// `ascribe.toml` gives its own folder, when that's below the content root.
fn nested_folder(layout: &Layout, project_path: &RelPath) -> Option<RelPath> {
    if project_path.file_name() != Some(MODEL_FILE)
        || !project_path.starts_with(&layout.content_root)
    {
        return None;
    }
    let rest: Vec<&str> = project_path
        .segments()
        .skip(layout.content_root.segments().count())
        .collect();
    let (_, dir) = rest.split_last()?;
    if dir.is_empty() || dir.iter().any(|s| s.starts_with('.')) {
        return None;
    }
    RelPath::parse(&dir.join("/"))
        .ok()
        .filter(RelPath::is_inside)
}

/// Which files write an `@include` naming each path, whether or not that path
/// is a file. It's what finds every page an include reaches, through files
/// that are there and files that aren't yet.
#[derive(Default)]
struct IncludeRev {
    by_target: BTreeMap<RelPath, BTreeSet<RelPath>>,
    /// The same, by the target's lowercased path: an include's problem names
    /// a file whose path differs from its target only in case.
    by_folded: BTreeMap<String, BTreeSet<RelPath>>,
    targets_of: BTreeMap<RelPath, BTreeSet<RelPath>>,
}

impl IncludeRev {
    fn set(&mut self, index: &FileIndex) {
        self.remove(&index.path);
        let targets: BTreeSet<RelPath> = index
            .includes
            .iter()
            .filter_map(|i| i.target.clone())
            .collect();
        for target in &targets {
            self.by_target
                .entry(target.clone())
                .or_default()
                .insert(index.path.clone());
            self.by_folded
                .entry(fold(target))
                .or_default()
                .insert(index.path.clone());
        }
        self.targets_of.insert(index.path.clone(), targets);
    }

    fn remove(&mut self, path: &RelPath) {
        for target in self.targets_of.remove(path).unwrap_or_default() {
            if let Some(files) = self.by_target.get_mut(&target) {
                files.remove(path);
                if files.is_empty() {
                    self.by_target.remove(&target);
                }
            }
            let folded = fold(&target);
            if let Some(files) = self.by_folded.get_mut(&folded) {
                files.remove(path);
                if files.is_empty() {
                    self.by_folded.remove(&folded);
                }
            }
        }
    }

    /// The files whose `@include` names this path, or one that differs only
    /// in case.
    fn includers_like(&self, target: &RelPath) -> BTreeSet<RelPath> {
        self.by_folded
            .get(&fold(target))
            .cloned()
            .unwrap_or_default()
    }

    fn includers(&self, target: &RelPath) -> BTreeSet<RelPath> {
        self.by_target.get(target).cloned().unwrap_or_default()
    }

    /// The files that include `start`, directly or through other files,
    /// including through paths that aren't files.
    fn dependents(&self, start: &RelPath) -> BTreeSet<RelPath> {
        let mut seen = BTreeSet::new();
        let mut queue = vec![start.clone()];
        while let Some(next) = queue.pop() {
            for file in self.by_target.get(&next).into_iter().flatten() {
                if seen.insert(file.clone()) {
                    queue.push(file.clone());
                }
            }
        }
        seen
    }
}

/// A project that updates itself as files change. See the [module
/// documentation](self).
pub struct IncrementalProject {
    fs: Overlay,
    ids: FileIds,
    project: Arc<Project>,
    version: Version,
    latest: Arc<AtomicU64>,
    model_revision: u64,
    stamps: Arc<BTreeMap<RelPath, Version>>,
    sigs: BTreeMap<RelPath, u64>,
    deps: Deps,
    include_rev: IncludeRev,
    parse_cache: ParseCache,
    fingerprints: Fingerprints,
    history: VecDeque<HistoryEntry>,
    stats: Stats,
}

impl IncrementalProject {
    /// Loads a project from `fs` (the source files under the content root, and
    /// what references can probe): the state every later update is relative
    /// to. Files are numbered in path order from 1, as [`Project::load`] does.
    /// `fs` must be `Send`, so the project can move between threads (the
    /// language server computes on a worker).
    pub fn load(
        model: Arc<ContentModel>,
        layout: Layout,
        fs: impl FileSystem + Send + 'static,
    ) -> IncrementalProject {
        let mut ids = FileIds::new();
        let project = Project::load_with_ids(model.clone(), layout, &fs, &mut ids);
        let fingerprints = Fingerprints::of(&model);
        let mut parse_cache = ParseCache::default();
        let mut sigs = BTreeMap::new();
        let mut deps = Deps::default();
        let mut include_rev = IncludeRev::default();
        let mut stamps = BTreeMap::new();
        for index in project.files() {
            parse_cache.put(
                index.file,
                &index.source,
                fingerprints.parse(),
                &index.document,
            );
            sigs.insert(index.path.clone(), structure_signature(index));
            deps.set(project.layout(), index);
            include_rev.set(index);
            stamps.insert(index.path.clone(), Version(0));
        }
        IncrementalProject {
            fs: Overlay::new(Box::new(fs)),
            ids,
            project: Arc::new(project),
            version: Version(0),
            latest: Arc::new(AtomicU64::new(0)),
            model_revision: 0,
            stamps: Arc::new(stamps),
            sigs,
            deps,
            include_rev,
            parse_cache,
            fingerprints,
            history: VecDeque::new(),
            stats: Stats::default(),
        }
    }

    /// The current state.
    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            project: self.project.clone(),
            version: self.version,
            model_revision: self.model_revision,
            stamps: self.stamps.clone(),
            latest: self.latest.clone(),
        }
    }

    /// The latest version.
    pub fn version(&self) -> Version {
        self.version
    }

    /// The ids of every path a file has been at (see [`FileIds`]).
    pub fn ids(&self) -> &FileIds {
        &self.ids
    }

    /// The work done since the project was loaded.
    pub fn stats(&self) -> Stats {
        self.stats
    }

    /// Whether what was computed for a file from `snapshot` is still right
    /// after every update since: no later update listed the file in
    /// [`Affected::recheck`] or [`Affected::removed`], or changed the model
    /// beyond [`ModelImpact::Warnings`]. False when the record of updates no
    /// longer reaches back that far.
    pub fn is_file_current(&self, snapshot: &Snapshot, path: &RelPath) -> bool {
        if snapshot.version == self.version {
            return true;
        }
        let first = snapshot.version.0 + 1;
        if self.history.front().is_none_or(|e| e.version.0 > first) {
            return false;
        }
        self.history
            .iter()
            .filter(|e| e.version > snapshot.version)
            .all(|e| {
                e.recheck.as_ref().is_some_and(|r| !r.contains(path)) && !e.removed.contains(path)
            })
    }

    /// Applies a batch of changes, and says what they affect. See the
    /// [module documentation](self) for what each kind of change invalidates.
    ///
    /// # Errors
    ///
    /// [`ApplyError::LayoutChanged`] when a [`Change::Model`] moves the content
    /// root or the output directory, and [`ApplyError::NestedProjectChanged`]
    /// when the batch adds or removes a nested project's `ascribe.toml`.
    /// Nothing is applied.
    // Longer than the lint allows from before it was on. Split it only while
    // changing it for another reason.
    #[allow(clippy::too_many_lines)]
    pub fn apply(
        &mut self,
        changes: impl IntoIterator<Item = Change>,
    ) -> Result<Affected, ApplyError> {
        let layout = self.project.layout().clone();
        // Which files are sources: the nested projects stay what they were at
        // load, since a batch that would change them is refused below.
        let nested = self.project.nested_projects().to_vec();
        let own_folder = self.project.own_folder().cloned();
        let is_source = |path: &RelPath| is_source_path(path) && !in_nested_project(path, &nested);

        // The net effect of the batch, per path.
        let mut source_state: BTreeMap<RelPath, Option<Arc<str>>> = BTreeMap::new();
        let mut file_state: BTreeMap<RelPath, bool> = BTreeMap::new();
        // A source path's last word on whether it can't be
        // read. Any other change to the path clears it.
        let mut unreadable_state: BTreeMap<RelPath, Option<String>> = BTreeMap::new();
        let mut new_model: Option<Arc<ContentModel>> = None;
        for change in changes {
            match change {
                Change::Unreadable { path, reason } => {
                    if is_source(&path) {
                        source_state.insert(path.clone(), None);
                        unreadable_state.insert(path, Some(reason));
                    } else {
                        file_state.insert(layout.project_path(&path), true);
                    }
                }
                Change::Created { path, text } | Change::Edited { path, text } => {
                    if is_source(&path) {
                        unreadable_state.insert(path.clone(), None);
                        source_state.insert(path, Some(Arc::from(text)));
                    } else {
                        file_state.insert(layout.project_path(&path), true);
                    }
                }
                Change::Deleted { path } => {
                    if is_source(&path) {
                        unreadable_state.insert(path.clone(), None);
                        source_state.insert(path, None);
                    } else {
                        file_state.insert(layout.project_path(&path), false);
                    }
                }
                Change::Renamed { from, to } => {
                    let text = if is_source(&from) {
                        match source_state.get(&from) {
                            Some(state) => state.clone(),
                            None => self.project.file(&from).map(|f| f.source.clone()),
                        }
                    } else {
                        None
                    };
                    if is_source(&from) {
                        unreadable_state.insert(from.clone(), None);
                        source_state.insert(from, None);
                    } else {
                        file_state.insert(layout.project_path(&from), false);
                    }
                    if is_source(&to) {
                        if let Some(text) = text {
                            unreadable_state.insert(to.clone(), None);
                            source_state.insert(to, Some(text));
                        }
                    } else {
                        file_state.insert(layout.project_path(&to), true);
                    }
                }
                Change::AssetCreated { path } => {
                    file_state.insert(path, true);
                }
                Change::AssetDeleted { path } => {
                    file_state.insert(path, false);
                }
                Change::Model(model) => {
                    // A new content root or output
                    // directory isn't applied in place.
                    if Layout::from_model(&model) != layout {
                        return Err(ApplyError::LayoutChanged);
                    }
                    new_model = Some(model);
                }
            }
        }

        // A nested project's `ascribe.toml` arrives as any other file that isn't
        // a source does. One appearing where there is no nested project, or
        // going where there is one, changes which files are sources. The
        // project's own never does.
        let nested_changed = file_state.iter().any(|(path, present)| {
            let folder =
                nested_folder(&layout, path).filter(|dir| Some(dir) != own_folder.as_ref());
            folder.is_some_and(|dir| {
                if *present {
                    !in_nested_project(&dir, &nested)
                } else {
                    nested.contains(&dir)
                }
            })
        });
        if nested_changed {
            return Err(ApplyError::NestedProjectChanged);
        }

        // The batch's net effect, per path.
        let mut created: BTreeMap<RelPath, Arc<str>> = BTreeMap::new();
        let mut edited: BTreeMap<RelPath, Arc<str>> = BTreeMap::new();
        let mut gone: BTreeSet<RelPath> = BTreeSet::new();
        for (path, state) in source_state {
            match (self.project.file(&path), state) {
                (Some(old), Some(text)) if *old.source == *text => {}
                (Some(_), Some(text)) => {
                    edited.insert(path, text);
                }
                (None, Some(text)) => {
                    created.insert(path, text);
                }
                (Some(_), None) => {
                    gone.insert(path);
                }
                (None, None) => {}
            }
        }
        let file_changes: BTreeMap<RelPath, bool> = file_state
            .into_iter()
            .filter(|(path, present)| self.fs.has(path) != *present)
            .collect();
        // The unreadable list after the batch: an entry a change touched goes,
        // and the batch's own entries come in, in path order.
        let old_unreadable = self.project.unreadable();
        let mut new_unreadable: Vec<Unreadable> = old_unreadable
            .iter()
            .filter(|u| !unreadable_state.contains_key(&u.path))
            .cloned()
            .collect();
        for (path, reason) in &unreadable_state {
            if let Some(reason) = reason {
                new_unreadable.push(Unreadable {
                    path: path.clone(),
                    reason: reason.clone(),
                });
            }
        }
        new_unreadable.sort_by(|a, b| a.path.cmp(&b.path));
        let unreadable_changed = new_unreadable != old_unreadable;
        // Files that now can't be read, or can't be read for a new reason.
        let now_unreadable: BTreeSet<RelPath> = new_unreadable
            .iter()
            .filter(|u| !old_unreadable.contains(u))
            .map(|u| u.path.clone())
            .collect();
        // Files that couldn't be read and are gone, without becoming a source
        // the index holds (a deletion): their diagnostic goes.
        let no_longer_unreadable: BTreeSet<RelPath> = old_unreadable
            .iter()
            .filter(|u| {
                !new_unreadable.iter().any(|n| n.path == u.path) && !created.contains_key(&u.path)
            })
            .map(|u| u.path.clone())
            .collect();
        let (new_model, impact) = match new_model {
            Some(model) => {
                let fingerprints = Fingerprints::of(&model);
                let impact = self.fingerprints.impact_of(&fingerprints);
                (impact.map(|_| (model, fingerprints)), impact)
            }
            None => (None, None),
        };

        if created.is_empty()
            && edited.is_empty()
            && gone.is_empty()
            && file_changes.is_empty()
            && impact.is_none()
            && !unreadable_changed
        {
            return Ok(Affected {
                version: self.version,
                previous: self.version,
                parsed: BTreeSet::new(),
                indexed: BTreeSet::new(),
                recheck: BTreeSet::new(),
                removed: BTreeSet::new(),
                unreadable: BTreeSet::new(),
                re_resolve: BTreeSet::new(),
                model: None,
            });
        }

        let previous = self.version;
        let version = Version(previous.0 + 1);
        let everything = impact.is_some_and(|m| m > ModelImpact::Warnings);
        let reindex_all = impact.is_some_and(|m| m >= ModelImpact::Index);

        // Ids: a new path gets the next unused number, in path order.
        for path in created.keys() {
            self.ids.assign(path);
        }
        let touched: BTreeSet<RelPath> = created
            .keys()
            .chain(edited.keys())
            .chain(gone.iter())
            .cloned()
            .collect();

        // The model the update runs against.
        let (model, fingerprints) = match &new_model {
            Some((model, fingerprints)) => (model.clone(), fingerprints.clone()),
            None => (self.project.model.clone(), self.fingerprints.clone()),
        };
        let slugger: Box<dyn Slugger> =
            slugger_by_name(&model.consumer.slugger).unwrap_or_else(default_slugger);

        // Parse and index what needs it.
        let mut to_index: BTreeMap<RelPath, Arc<str>> = BTreeMap::new();
        if reindex_all {
            for file in self.project.files().filter(|f| !gone.contains(&f.path)) {
                to_index.insert(file.path.clone(), file.source.clone());
            }
        }
        for (path, text) in created.iter().chain(&edited) {
            to_index.insert(path.clone(), text.clone());
        }
        let mut parsed = BTreeSet::new();
        let mut indexed = BTreeSet::new();
        let mut new_indexes: BTreeMap<RelPath, Arc<FileIndex>> = BTreeMap::new();
        for (path, text) in &to_index {
            let Some(id) = self.ids.get(path) else {
                continue;
            };
            let document = match self.parse_cache.get(id, text, fingerprints.parse()) {
                Some(document) => document,
                None => {
                    let document = Arc::new(parse_source(id, text, &model));
                    self.parse_cache
                        .put(id, text, fingerprints.parse(), &document);
                    self.stats.parses += 1;
                    parsed.insert(path.clone());
                    document
                }
            };
            let index = index_parsed(id, path, text.clone(), document, &model, slugger.as_ref());
            self.stats.indexes += 1;
            indexed.insert(path.clone());
            new_indexes.insert(path.clone(), Arc::new(index));
        }

        // Which files' references must be resolved again: those re-indexed, and
        // those with a reference that looks at a path where a file appeared or
        // disappeared. (Every file, when the model changed beyond warnings.)
        let mut keys: BTreeSet<String> = BTreeSet::new();
        for path in created.keys().chain(&gone) {
            keys.insert(fold(&layout.project_path(path)));
        }
        for path in file_changes.keys() {
            keys.insert(fold(path));
        }
        let mut resolve_set: BTreeSet<RelPath> = new_indexes.keys().cloned().collect();
        for key in &keys {
            resolve_set.extend(self.deps.files_at(key).cloned());
        }
        if everything {
            resolve_set.extend(self.project.files().map(|f| f.path.clone()));
        }
        resolve_set.retain(|p| !gone.contains(p));

        // The signatures before, for deciding what others can see changed.
        let old_sigs: BTreeMap<RelPath, Option<u64>> = touched
            .iter()
            .map(|p| (p.clone(), self.sigs.get(p).copied()))
            .collect();

        // -- Mutate ---------------------------------------------------------
        let project = Arc::make_mut(&mut self.project);
        let old_resolutions: BTreeMap<RelPath, Arc<Vec<crate::Resolution>>> = resolve_set
            .iter()
            .filter(|p| !new_indexes.contains_key(*p))
            .map(|p| (p.clone(), project.resolution_list(p)))
            .collect();
        for path in resolve_set.iter().chain(&gone).chain(new_indexes.keys()) {
            project.detach_file(path);
        }
        for path in &gone {
            if let Some(id) = self.ids.get(path) {
                self.parse_cache.forget(id);
            }
            project.drop_index(path);
            self.sigs.remove(path);
            self.deps.remove(path);
            self.include_rev.remove(path);
            self.fs.remove(&layout.project_path(path));
        }
        for path in created.keys() {
            self.fs.add(&layout.project_path(path));
        }
        for (path, present) in &file_changes {
            if *present {
                self.fs.add(path);
            } else {
                self.fs.remove(path);
            }
        }
        if let Some((model, fingerprints)) = new_model {
            project.set_model(model);
            self.fingerprints = fingerprints;
            if impact.is_some_and(|m| m >= ModelImpact::Index) {
                project.forget_expansions(None);
            }
        }
        for (path, index) in new_indexes {
            self.sigs.insert(path.clone(), structure_signature(&index));
            self.deps.set(&layout, &index);
            self.include_rev.set(&index);
            project.put_index(index);
        }
        let mut res_changed: BTreeSet<RelPath> = BTreeSet::new();
        for path in &resolve_set {
            let Some(index) = project.file(path).cloned() else {
                continue;
            };
            let resolutions = project.resolve_file(&index, &self.fs);
            let snippets =
                project.resolve_snippets(&index, &self.fs, &crate::snippet::CodeFiles::new());
            project.put_snippets(path, snippets);
            self.stats.resolutions += 1;
            if old_resolutions
                .get(path)
                .is_some_and(|old| **old != resolutions)
            {
                res_changed.insert(path.clone());
            }
            project.put_resolutions(path, resolutions);
            project.attach_file(path);
        }
        for path in created.keys().chain(&gone) {
            let includers: Vec<RelPath> = self.include_rev.includers(path).into_iter().collect();
            project.refresh_include_edges(path, &includers);
        }
        // What a change to these files can reach through includes.
        let mut dirty = touched.clone();
        for path in &touched {
            dirty.extend(self.include_rev.dependents(path));
        }
        project.forget_expansions(Some(&dirty));
        if unreadable_changed {
            project.set_unreadable(new_unreadable);
        }

        // A file appearing that no reference names changes nothing anyone can
        // see: no new snapshot, no new version.
        if created.is_empty()
            && edited.is_empty()
            && gone.is_empty()
            && impact.is_none()
            && res_changed.is_empty()
            && !unreadable_changed
        {
            return Ok(Affected {
                version: previous,
                previous,
                parsed,
                indexed,
                recheck: BTreeSet::new(),
                removed: BTreeSet::new(),
                unreadable: BTreeSet::new(),
                re_resolve: BTreeSet::new(),
                model: None,
            });
        }

        // -- What was affected ----------------------------------------------
        let project: &Project = &self.project;
        let is_page = |p: &RelPath| project.file(p).is_some_and(|f| f.kind == FileKind::Page);
        let mut recheck: BTreeSet<RelPath> = BTreeSet::new();
        let mut re_resolve: BTreeSet<RelPath> = BTreeSet::new();
        if everything {
            recheck.extend(project.files().map(|f| f.path.clone()));
            re_resolve.extend(project.pages().map(|f| f.path.clone()));
        } else {
            let widen = |file: &RelPath,
                         recheck: &mut BTreeSet<RelPath>,
                         re_resolve: &mut BTreeSet<RelPath>| {
                recheck.insert(file.clone());
                if is_page(file) {
                    re_resolve.insert(file.clone());
                }
                for dependent in self.include_rev.dependents(file) {
                    recheck.insert(dependent.clone());
                    if is_page(&dependent) {
                        re_resolve.insert(dependent);
                    }
                }
            };
            for path in created.keys().chain(edited.keys()) {
                widen(path, &mut recheck, &mut re_resolve);
            }
            for path in &gone {
                for dependent in self.include_rev.dependents(path) {
                    if is_page(&dependent) {
                        re_resolve.insert(dependent);
                    }
                }
            }
            for path in &res_changed {
                widen(path, &mut recheck, &mut re_resolve);
            }
            // The files that write an `@include` of a path that appeared or
            // disappeared have a problem (or lost one), including one whose
            // diagnostic names a twin that differs only in case.
            for path in created.keys().chain(&gone) {
                for includer in self.include_rev.includers_like(path) {
                    recheck.insert(includer.clone());
                    if is_page(&includer) {
                        re_resolve.insert(includer);
                    }
                }
            }
            // Others see a change in what a file is: pages that link to it or
            // to a page that includes it, and whatever those pages hold.
            let glossary_targets = glossary_targets(&model);
            let mut all_pages = false;
            for path in &touched {
                let new_sig = self.sigs.get(path).copied();
                if old_sigs.get(path).copied().flatten() == new_sig {
                    continue;
                }
                let mut reached: BTreeSet<RelPath> = self
                    .include_rev
                    .dependents(path)
                    .into_iter()
                    .filter(&is_page)
                    .collect();
                if is_page(path) {
                    reached.insert(path.clone());
                }
                if glossary_targets.contains(path)
                    || reached.iter().any(|p| glossary_targets.contains(p))
                {
                    all_pages = true;
                }
                for page in &reached {
                    recheck.insert(page.clone());
                    for site in project.links_to(page) {
                        widen(&site.file, &mut recheck, &mut re_resolve);
                    }
                }
                for includer in self.include_rev.includers(path) {
                    recheck.insert(includer);
                }
            }
            if all_pages {
                re_resolve.extend(project.pages().map(|f| f.path.clone()));
            }
        }
        recheck.retain(|p| project.file(p).is_some());
        re_resolve.retain(is_page);

        // -- The new snapshot -----------------------------------------------
        {
            let stamps = Arc::make_mut(&mut self.stamps);
            for path in created.keys().chain(edited.keys()) {
                stamps.insert(path.clone(), version);
            }
            for path in &gone {
                stamps.remove(path);
            }
        }
        if impact.is_some() {
            self.model_revision += 1;
        }
        self.version = version;
        self.latest.store(version.0, Ordering::Release);
        self.history.push_back(HistoryEntry {
            version,
            recheck: (!everything).then(|| recheck.clone()),
            removed: gone.iter().chain(&now_unreadable).cloned().collect(),
        });
        while self.history.len() > HISTORY {
            self.history.pop_front();
        }
        Ok(Affected {
            version,
            previous,
            parsed,
            indexed,
            recheck,
            // A file that can't be read still exists: it's listed as
            // unreadable, not removed.
            removed: gone
                .into_iter()
                .filter(|p| !now_unreadable.contains(p))
                .chain(no_longer_unreadable)
                .collect(),
            unreadable: now_unreadable,
            re_resolve,
            model: impact,
        })
    }
}

/// The pages the glossary's terms link to.
fn glossary_targets(model: &ContentModel) -> BTreeSet<RelPath> {
    model
        .glossary
        .terms
        .iter()
        .filter_map(|t| t.link.as_ref())
        .filter_map(|(path, _)| RelPath::parse(path.trim_start_matches('/')).ok())
        .collect()
}
