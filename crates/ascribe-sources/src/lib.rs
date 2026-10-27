//! Sources in another repository (SPEC §7.4): copying the files a project's
//! snippets use into it, and moving a source's pin.
//!
//! A project reads a source in another repository through copies, kept in
//! `sources/<name>/` and pinned to a commit by `ascribe.lock`, so checking
//! and building need no network. This crate is what makes and moves them:
//!
//! - [`fetch`] makes the copies match the lock: it copies, at each source's
//!   pin, the files snippets name that have no copy or a wrong one, removes
//!   the copies no snippet names, and pins a source that has no pin yet.
//! - [`update`] moves a source's pin to the head of its branch, or to a
//!   given revision, copies the files again there, and says what changed.
//! - [`status`] says, without the network, each source's pin and the state
//!   of its copies.
//!
//! `fetch` and `update` are the only code in Ascribe that reaches another
//! repository, through the `git` executable and its own credentials
//! (`remote`). No git, HTTP, or async library is linked.

mod copies;
mod remote;

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use ascribe_core::RelPath;
use ascribe_model::examples::{Status, is_examples_file};
use ascribe_model::{LOCK_FILE, Lock, LockedFile, LockedSource, Source, file_hash};
use ascribe_resolve::{DiskFs, FileSystem, Layout};
use ascribe_workspace::WorkspaceSnapshot;
use serde::Serialize;

pub use ascribe_model::short_commit as short;
pub use copies::SIZE_LIMIT;
pub use remote::{CommitLine, Commits, NEWEST, cache_root};

use copies::{Folder, not_text};
use remote::{Cache, History, Remote};

/// Why a command couldn't do its work.
#[derive(Debug, thiserror::Error)]
pub enum SourcesError {
    /// `git` isn't installed, or isn't on the path.
    #[error(
        "git not found: copying a source from another repository runs `git`, so it needs git installed and on the path"
    )]
    GitNotFound,
    /// `git` failed for a source: its own message.
    #[error("source `{name}`: git failed: {message}")]
    Git {
        /// The source.
        name: String,
        /// What `git` said.
        message: String,
    },
    /// A name given isn't a declared source.
    #[error("ascribe.toml has no source `{0}`")]
    UnknownSource(String),
    /// A name given is a source in this repository.
    #[error(
        "source `{0}` is a folder in this repository, not another repository, so it has nothing to copy"
    )]
    NotRemote(String),
    /// `ascribe.lock` can't be read.
    #[error("ascribe.lock can't be read: {0}; `ascribe check` says where")]
    Lock(String),
    /// A revision given with `--to` isn't one.
    #[error("`{0}` isn't a revision: give a commit's full hash, a branch, or a tag")]
    BadRevision(String),
    /// `--to` with more than one source.
    #[error("--to moves one source's pin: name the source")]
    ToNeedsOneSource,
    /// There's no cache folder.
    #[error("there's no folder to keep fetched repositories in: set ASCRIBE_CACHE_DIR, or HOME")]
    NoCache,
    /// The cache couldn't be made.
    #[error("can't make the cache at {path}: {message}")]
    Cache {
        /// The folder.
        path: String,
        /// The operating system's reason.
        message: String,
    },
    /// A copy or the lock couldn't be written.
    #[error("can't write {path}: {message}")]
    Write {
        /// The file.
        path: String,
        /// The operating system's reason.
        message: String,
    },
}

impl ascribe_core::Coded for SourcesError {
    fn code(&self) -> &'static str {
        match self {
            SourcesError::GitNotFound => "git_not_found",
            SourcesError::Git { .. } => "git_failed",
            SourcesError::UnknownSource(_) => "unknown_source",
            SourcesError::NotRemote(_) => "source_not_remote",
            SourcesError::Lock(_) => "lock_unreadable",
            SourcesError::BadRevision(_) => "unknown_revision",
            SourcesError::ToNeedsOneSource => "to_needs_one_source",
            SourcesError::NoCache => "no_cache",
            SourcesError::Cache { .. } => "cache_unwritable",
            SourcesError::Write { .. } => "copy_unwritable",
        }
    }
}

/// Where fetched repositories are kept.
#[derive(Clone, Debug)]
pub struct Options {
    /// The cache folder: one bare repository per URL is kept in it.
    pub cache: PathBuf,
}

impl Options {
    /// The cache in the user's cache folder (`ASCRIBE_CACHE_DIR` overrides
    /// it).
    ///
    /// # Errors
    ///
    /// [`SourcesError::NoCache`] when there's no cache folder to use.
    pub fn from_env() -> Result<Options, SourcesError> {
        remote::cache_root()
            .map(|cache| Options { cache })
            .ok_or(SourcesError::NoCache)
    }
}

/// A project, as these commands see it: its folder, its sources, its lock,
/// and the files its snippets name.
#[derive(Clone, Debug)]
pub struct SourcesView {
    root: PathBuf,
    /// The project's files.
    files: DiskFs,
    sources: Vec<Source>,
    lock: Lock,
    /// The lock's text as it is on disk, so it's only written when it
    /// changes.
    lock_text: Option<String>,
    /// For each source, the paths snippets name in it.
    used: BTreeMap<String, BTreeSet<String>>,
    /// Whether `fetch` and `update` write nothing, and only say what they
    /// would write.
    dry_run: bool,
    /// On a dry run, the files `fetch` or `update` would write or remove.
    planned: RefCell<Vec<PathBuf>>,
}

impl SourcesView {
    /// The sources' view of the project `snapshot` holds: its folder, its
    /// content model's sources, and the paths its source index's snippets
    /// name.
    ///
    /// # Errors
    ///
    /// [`SourcesError::Lock`] when `ascribe.lock` is there and can't be
    /// read.
    pub fn new(snapshot: &WorkspaceSnapshot) -> Result<SourcesView, SourcesError> {
        let (root, model, project) = (snapshot.root(), snapshot.model(), snapshot.index());
        let files = DiskFs::new(root, &Layout::from_model(model));
        let lock_text = RelPath::parse(LOCK_FILE)
            .ok()
            .and_then(|path| files.read_file(&path).ok())
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned());
        let lock = match &lock_text {
            None => Lock::default(),
            Some(text) => Lock::parse(text, ascribe_core::FileId::new(0)).map_err(|issues| {
                let detail = issues
                    .first()
                    .and_then(|i| i.args.iter().find(|a| a.name == "detail"))
                    .map(|a| a.value.clone())
                    .unwrap_or_default();
                SourcesError::Lock(detail)
            })?,
        };
        let mut used: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for file in project.files() {
            for snippet in &file.snippets {
                if let Some(Ok(address)) = &snippet.address {
                    used.entry(address.source.clone())
                        .or_default()
                        .insert(address.path.clone());
                }
            }
        }
        Ok(SourcesView {
            root: root.to_path_buf(),
            files,
            sources: model.sources.clone(),
            lock,
            lock_text,
            used,
            dry_run: false,
            planned: RefCell::new(Vec::new()),
        })
    }

    /// The same view, where `fetch` and `update` write nothing when
    /// `dry_run`, and say what they would write: the copies they'd copy and
    /// remove, and whether the lock would change.
    #[must_use]
    pub fn dry_run(mut self, dry_run: bool) -> SourcesView {
        self.dry_run = dry_run;
        self.planned = RefCell::new(Vec::new());
        self
    }

    /// The files the dry runs so far would write or remove, absolute, in
    /// the order they'd be changed: the copies, then `ascribe.lock`.
    pub fn planned(&self) -> Vec<PathBuf> {
        self.planned.borrow().clone()
    }

    fn planned_list(&self) -> Option<&RefCell<Vec<PathBuf>>> {
        self.dry_run.then_some(&self.planned)
    }

    /// The sources in other repositories that `names` names, in declaration
    /// order; every one when `names` is empty.
    fn selected(&self, names: &[String]) -> Result<Vec<&Source>, SourcesError> {
        for name in names {
            match self.sources.iter().find(|s| &s.name == name) {
                None => return Err(SourcesError::UnknownSource(name.clone())),
                Some(s) if s.git.is_none() => return Err(SourcesError::NotRemote(name.clone())),
                Some(_) => {}
            }
        }
        Ok(self
            .sources
            .iter()
            .filter(|s| s.git.is_some())
            .filter(|s| names.is_empty() || names.contains(&s.name))
            .collect())
    }

    /// A source's pin, when it's to the repository `ascribe.toml` names now.
    fn pin(&self, source: &Source) -> Option<&LockedSource> {
        let url = &source.git.as_ref()?.url;
        self.lock.source(&source.name).filter(|l| &l.git == url)
    }

    /// The files snippets name in a source that it makes readable: the ones
    /// to copy. One it doesn't make readable is `ascribe check`'s to report.
    fn needed(&self, source: &Source) -> BTreeSet<String> {
        self.used
            .get(&source.name)
            .map(|paths| paths.iter().filter(|p| source.reads(p)).cloned().collect())
            .unwrap_or_default()
    }

    fn folder(&self, source: &Source) -> Folder<'_> {
        Folder::new(
            &self.root,
            &self.files,
            RelPath::parse(&source.path).unwrap_or_default(),
            self.planned_list(),
        )
    }

    /// Makes a source's copies those of `commit`: keeps each copy `previous`
    /// (a pin at the same commit) vouches for, copies the rest from the
    /// repository, and removes the copies nothing names.
    ///
    /// Beside the files snippets name, the source's tested examples' config
    /// and observations are copied, whatever its `include` says
    /// ([`is_examples_file`]). Which there are is read from the commit's
    /// tree whenever the repository is reached; when every copy is in
    /// order, from the pin, so a source whose copies are all in order still
    /// never runs `git`.
    fn sync(
        &self,
        source: &Source,
        commit: &str,
        previous: Option<&LockedSource>,
        remote: &mut Lazy<'_>,
    ) -> Result<Synced, SourcesError> {
        let folder = self.folder(source);
        let mut synced = Synced::default();
        let mut wanted = Vec::new();
        // The examples' files the pin lists: a commit's files don't change.
        let pinned_examples = previous
            .map(|p| {
                p.files
                    .keys()
                    .filter(|path| is_examples_file(path))
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        for path in self.needed(source).into_iter().chain(pinned_examples) {
            // A commit's files don't change: one the pin found missing still is.
            if previous.is_some_and(|p| p.missing.contains(&path)) {
                synced.not_at(path, commit);
                continue;
            }
            let kept = previous.and_then(|p| p.files.get(&path)).filter(|locked| {
                folder
                    .read(&path)
                    .is_some_and(|bytes| file_hash(&bytes) == locked.hash)
            });
            match kept {
                Some(locked) => {
                    synced.files.insert(path, locked.hash.clone());
                }
                None => wanted.push(path),
            }
        }
        if !wanted.is_empty() {
            let remote = remote.get()?;
            if !remote.has_commit(commit) {
                remote.fetch(commit, true)?;
            }
            let tree = remote.tree(commit)?;
            let mut objects: Vec<(String, String)> = Vec::new();
            for path in wanted {
                match tree.get(&path) {
                    // The examples' files are whatever the commit has.
                    None if is_examples_file(&path) => {}
                    Some(file) if is_examples_file(&path) && !file.regular => {}
                    None => synced.not_at(path, commit),
                    Some(file) if !file.regular => synced.failed.push(Failure {
                        path,
                        reason: "it's a symbolic link or a submodule, not a file".into(),
                    }),
                    Some(file) => objects.push((path, file.object.clone())),
                }
            }
            // The examples' files the pin didn't list, or that are new at
            // this commit.
            for (path, file) in &tree {
                if is_examples_file(path)
                    && file.regular
                    && !synced.files.contains_key(path)
                    && !objects.iter().any(|(p, _)| p == path)
                {
                    objects.push((path.clone(), file.object.clone()));
                }
            }
            let names: Vec<&str> = objects.iter().map(|(_, o)| o.as_str()).collect();
            let contents = remote.read(&names, SIZE_LIMIT)?;
            for ((path, _), bytes) in objects.into_iter().zip(contents) {
                // An examples' file that can't be copied is left out: its
                // examples read as unobserved, and no snippet named it.
                let unusable = bytes.as_deref().is_none_or(|b| not_text(b).is_some());
                if is_examples_file(&path) && unusable {
                    continue;
                }
                let Some(bytes) = bytes else {
                    synced.failed.push(Failure {
                        path,
                        reason: format!(
                            "it's larger than {} KB, the most Ascribe copies",
                            SIZE_LIMIT / 1024
                        ),
                    });
                    continue;
                };
                if let Some(reason) = not_text(&bytes) {
                    synced.failed.push(Failure {
                        path,
                        reason: reason.into(),
                    });
                    continue;
                }
                let written = folder
                    .write(&path, &bytes)
                    .map_err(|e| SourcesError::Write {
                        path: format!("{}/{path}", source.path),
                        message: e.to_string(),
                    })?;
                if written {
                    synced.copied.push(path.clone());
                }
                synced.files.insert(path, file_hash(&bytes));
            }
        }
        for path in folder.files() {
            if !synced.files.contains_key(&path) {
                folder.remove(&path).map_err(|e| SourcesError::Write {
                    path: format!("{}/{path}", source.path),
                    message: e.to_string(),
                })?;
                synced.removed.push(path);
            }
        }
        Ok(synced)
    }

    /// Writes the lock, when its text changed. A project with no pins and
    /// no lock gets none.
    fn write_lock(&self, lock: &Lock) -> Result<bool, SourcesError> {
        if lock.sources.is_empty() && self.lock_text.is_none() {
            return Ok(false);
        }
        let text = lock.to_toml();
        if self.lock_text.as_deref() == Some(text.as_str()) {
            return Ok(false);
        }
        let path = self.root.join(LOCK_FILE);
        // A link where the lock goes is neither written through nor replaced.
        if ascribe_resolve::is_link(&path) {
            return Err(SourcesError::Write {
                path: LOCK_FILE.into(),
                message: "it's a link; replace it with a file, or remove it".into(),
            });
        }
        if let Some(planned) = self.planned_list() {
            planned.borrow_mut().push(path);
            return Ok(true);
        }
        // Library write: the lock file, after a fetch that changed it.
        std::fs::write(&path, text).map_err(|e| SourcesError::Write {
            path: LOCK_FILE.into(),
            message: e.to_string(),
        })?;
        Ok(true)
    }
}

/// The cache's repository for a source, opened when it's first needed, so a
/// source whose copies are all in order never runs `git`.
struct Lazy<'a> {
    cache: &'a Cache,
    name: &'a str,
    url: &'a str,
    remote: Option<Remote>,
}

impl Lazy<'_> {
    fn get(&mut self) -> Result<&Remote, SourcesError> {
        if self.remote.is_none() {
            self.remote = Some(self.cache.open(self.name, self.url)?);
        }
        self.remote.as_ref().ok_or(SourcesError::NoCache)
    }
}

#[derive(Debug, Default)]
struct Synced {
    /// Every copy now, with its hash.
    files: BTreeMap<String, String>,
    copied: Vec<String>,
    removed: Vec<String>,
    failed: Vec<Failure>,
    /// The files that aren't in the repository at the commit.
    missing: BTreeSet<String>,
}

impl Synced {
    /// `path` isn't in the repository at `commit`.
    fn not_at(&mut self, path: String, commit: &str) {
        self.failed.push(Failure {
            reason: format!("it isn't in the repository at {}", short(commit)),
            path: path.clone(),
        });
        self.missing.insert(path);
    }

    fn locked(&self, source: &Source, url: &str, commit: &str) -> LockedSource {
        LockedSource {
            name: source.name.clone(),
            git: url.to_owned(),
            commit: commit.to_owned(),
            files: self
                .files
                .iter()
                .map(|(path, hash)| {
                    (
                        path.clone(),
                        LockedFile {
                            hash: hash.clone(),
                            span: ascribe_core::Span::empty(0),
                        },
                    )
                })
                .collect(),
            missing: self.missing.clone(),
            span: ascribe_core::Span::empty(0),
        }
    }
}

/// A file a snippet names that couldn't be copied.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct Failure {
    /// Its path in the source.
    pub path: String,
    /// Why.
    pub reason: String,
}

/// What [`fetch`] did.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct FetchReport {
    /// Each source looked at, in declaration order.
    pub sources: Vec<SourceFetch>,
    /// The pins removed, of sources ascribe.toml no longer declares in
    /// another repository.
    pub unpinned: Vec<String>,
    /// Whether `ascribe.lock` was written.
    pub lock_written: bool,
}

/// What [`fetch`] did for one source.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct SourceFetch {
    /// The source.
    pub name: String,
    /// Its pin, when it has one.
    pub commit: Option<String>,
    /// Whether this run pinned it, to the head of its branch.
    pub pinned: bool,
    /// The copies written.
    pub copied: Vec<String>,
    /// The copies removed.
    pub removed: Vec<String>,
    /// The files snippets name that couldn't be copied.
    pub failed: Vec<Failure>,
    /// Whether these are the first files copied from the source, which
    /// publishes them to everyone who can read the project.
    pub first_copy: bool,
}

impl FetchReport {
    /// Whether a file a snippet names couldn't be copied.
    pub fn has_failures(&self) -> bool {
        self.sources.iter().any(|s| !s.failed.is_empty())
    }
}

/// Makes the copies of the sources `names` names (every source in another
/// repository when it's empty) match the lock.
///
/// # Errors
///
/// [`SourcesError`] when a name isn't a source in another repository, `git`
/// fails or isn't there, or a file can't be written. The pins of the sources
/// done before it are written.
pub fn fetch(
    view: &SourcesView,
    names: &[String],
    options: &Options,
) -> Result<FetchReport, SourcesError> {
    let selected = view.selected(names)?;
    let cache = Cache::new(options.cache.clone());
    let mut lock = view.lock.clone();
    let mut report = FetchReport::default();
    let mut outcome = Ok(());
    for source in selected {
        match fetch_one(view, source, &cache) {
            Ok(Some((fetched, pin))) => {
                replace(&mut lock, pin);
                report.sources.push(fetched);
            }
            Ok(None) => {}
            Err(e) => {
                outcome = Err(e);
                break;
            }
        }
    }
    if names.is_empty() && outcome.is_ok() {
        // Pins of sources that are gone, or no longer in another repository.
        let stale: Vec<LockedSource> = lock
            .sources
            .iter()
            .filter(|l| {
                !view
                    .sources
                    .iter()
                    .any(|s| s.name == l.name && s.git.is_some())
            })
            .cloned()
            .collect();
        for locked in stale {
            let folder = Source::copies_folder(&locked.name);
            // A source in this repository may use that folder itself.
            if !view.sources.iter().any(|s| s.path == folder) {
                let folder = Folder::new(
                    &view.root,
                    &view.files,
                    RelPath::parse(&folder).unwrap_or_default(),
                    view.planned_list(),
                );
                for path in locked.files.keys() {
                    folder.remove(path).map_err(|e| SourcesError::Write {
                        path: path.clone(),
                        message: e.to_string(),
                    })?;
                }
            }
            lock.sources.retain(|l| l.name != locked.name);
            report.unpinned.push(locked.name);
        }
    }
    report.lock_written = view.write_lock(&lock)?;
    outcome.map(|()| report)
}

fn fetch_one(
    view: &SourcesView,
    source: &Source,
    cache: &Cache,
) -> Result<Option<(SourceFetch, LockedSource)>, SourcesError> {
    let Some(git) = &source.git else {
        return Ok(None);
    };
    let previous = view.pin(source);
    if previous.is_none() && view.needed(source).is_empty() {
        // Nothing to copy, and nothing to pin it to yet.
        return Ok(None);
    }
    let mut remote = Lazy {
        cache,
        name: &source.name,
        url: &git.url,
        remote: None,
    };
    let (commit, pinned) = match previous {
        Some(pin) => (pin.commit.clone(), false),
        None => {
            let rev = git.branch.as_deref().unwrap_or("HEAD");
            (remote.get()?.fetch(rev, true)?, true)
        }
    };
    let synced = view.sync(source, &commit, previous, &mut remote)?;
    let had_files = previous.is_some_and(|p| !p.files.is_empty());
    let pin = synced.locked(source, &git.url, &commit);
    Ok(Some((
        SourceFetch {
            name: source.name.clone(),
            commit: Some(commit),
            pinned,
            first_copy: !had_files && !synced.files.is_empty(),
            copied: synced.copied,
            removed: synced.removed,
            failed: synced.failed,
        },
        pin,
    )))
}

fn replace(lock: &mut Lock, pin: LockedSource) {
    match lock.sources.iter_mut().find(|l| l.name == pin.name) {
        Some(slot) => *slot = pin,
        None => lock.sources.push(pin),
    }
}

/// What [`update`] did.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct UpdateReport {
    /// Each source, in declaration order.
    pub sources: Vec<SourceUpdate>,
    /// Whether any file changed: a copy, or the lock.
    pub changed: bool,
}

/// What [`update`] did for one source.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct SourceUpdate {
    /// The source.
    pub name: String,
    /// Its repository.
    pub git: String,
    /// The revision followed: `--to`'s, the branch, or `HEAD`.
    pub followed: String,
    /// The pin before, if it had one.
    pub from: Option<String>,
    /// The pin now.
    pub to: String,
    /// Whether the pin moved.
    pub moved: bool,
    /// Whether the new pin isn't after the old one: it moved back, or to
    /// another line of history. No commits are counted then.
    pub back: bool,
    /// The commits after the old pin up to the new one, when it moved
    /// forward from one and the repository could say.
    pub commits: Option<Commits>,
    /// The copies that changed.
    pub files: Vec<FileUpdate>,
    /// The tested examples snippets use whose status the new pin changed:
    /// each one that went stale, lost its observation, or became current,
    /// by path.
    pub examples: Vec<ExampleUpdate>,
    /// The files snippets name that couldn't be copied.
    pub failed: Vec<Failure>,
    /// Whether these are the first files copied from the source.
    pub first_copy: bool,
}

/// A tested example whose status an update changed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct ExampleUpdate {
    /// Its path in the source.
    pub path: String,
    /// Its status at the new pin.
    pub status: ExampleState,
    /// Its status before; `null` when it wasn't an example, or had no
    /// copy.
    pub was: Option<ExampleState>,
}

/// Where a tested example stands against the observation copied with it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum ExampleState {
    /// The observation lists the file as it is: the page says what it was
    /// tested with.
    Current,
    /// The file changed, and the observation didn't follow.
    Stale,
    /// No observation lists the file.
    Unobserved,
}

impl From<Status> for ExampleState {
    fn from(status: Status) -> ExampleState {
        match status {
            Status::Current => ExampleState::Current,
            Status::Stale => ExampleState::Stale,
            Status::Unobserved => ExampleState::Unobserved,
        }
    }
}

/// How a copy changed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct FileUpdate {
    /// Its path in the source.
    pub path: String,
    /// What happened to it.
    pub change: FileChange,
}

/// What happened to a copy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum FileChange {
    /// It's new.
    Added,
    /// Its contents changed.
    Changed,
    /// It's gone.
    Removed,
}

/// Moves the pins of the sources `names` names (every source in another
/// repository when it's empty) to the head of each one's branch, or to `to`,
/// and copies their files again there.
///
/// # Errors
///
/// [`SourcesError`] when a name isn't a source in another repository, `to`
/// is given for several, `git` fails or isn't there (no network, no access,
/// an unknown revision), or a file can't be written. The pins of the
/// sources done before it are written.
pub fn update(
    view: &SourcesView,
    names: &[String],
    to: Option<&str>,
    options: &Options,
) -> Result<UpdateReport, SourcesError> {
    let selected = view.selected(names)?;
    if let Some(to) = to {
        if selected.len() != 1 {
            return Err(SourcesError::ToNeedsOneSource);
        }
        if to.is_empty()
            || to.starts_with('-')
            || to.chars().any(|c| c.is_control() || c.is_whitespace())
        {
            return Err(SourcesError::BadRevision(to.to_owned()));
        }
    }
    let cache = Cache::new(options.cache.clone());
    let mut lock = view.lock.clone();
    let mut report = UpdateReport::default();
    let mut outcome = Ok(());
    for source in selected {
        match update_one(view, source, to, &cache) {
            Ok(Some((updated, pin))) => {
                report.changed |= !updated.files.is_empty();
                replace(&mut lock, pin);
                report.sources.push(updated);
            }
            Ok(None) => {}
            Err(e) => {
                outcome = Err(e);
                break;
            }
        }
    }
    report.changed |= view.write_lock(&lock)?;
    outcome.map(|()| report)
}

fn update_one(
    view: &SourcesView,
    source: &Source,
    to: Option<&str>,
    cache: &Cache,
) -> Result<Option<(SourceUpdate, LockedSource)>, SourcesError> {
    let Some(git) = &source.git else {
        return Ok(None);
    };
    let previous = view.pin(source);
    let followed = to.or(git.branch.as_deref()).unwrap_or("HEAD").to_owned();
    let mut remote = Lazy {
        cache,
        name: &source.name,
        url: &git.url,
        remote: None,
    };
    // Without a pin to count from, the history isn't needed.
    let commit = remote.get()?.fetch(&followed, previous.is_none())?;
    let moved = previous.is_none_or(|p| p.commit != commit);
    let folder = view.folder(source);
    let needed = view.needed(source);
    let examples_before = example_states(&folder, &needed);
    let synced = view.sync(source, &commit, previous.filter(|_| !moved), &mut remote)?;
    let examples_after = example_states(&folder, &needed);
    let examples = needed
        .iter()
        .filter_map(|path| {
            let status = *examples_after.get(path)?;
            let was = examples_before.get(path).copied();
            (was != Some(status)).then(|| ExampleUpdate {
                path: path.clone(),
                status,
                was,
            })
        })
        .collect();
    let before: BTreeMap<&String, &String> = previous
        .map(|p| p.files.iter().map(|(k, v)| (k, &v.hash)).collect())
        .unwrap_or_default();
    let mut files = Vec::new();
    for (path, hash) in &synced.files {
        let change = match before.get(path) {
            None => Some(FileChange::Added),
            Some(old) if *old != hash || synced.copied.contains(path) => Some(FileChange::Changed),
            Some(_) => None,
        };
        if let Some(change) = change {
            files.push(FileUpdate {
                path: path.clone(),
                change,
            });
        }
    }
    for path in before.keys() {
        if !synced.files.contains_key(*path) {
            files.push(FileUpdate {
                path: (*path).clone(),
                change: FileChange::Removed,
            });
        }
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    let (commits, back) = match previous {
        Some(p) if moved => match remote.get()?.history(&p.commit, &commit) {
            History::After(commits) => (Some(commits), false),
            History::Back => (None, true),
            History::Unknown => (None, false),
        },
        _ => (None, false),
    };
    let had_files = previous.is_some_and(|p| !p.files.is_empty());
    let pin = synced.locked(source, &git.url, &commit);
    Ok(Some((
        SourceUpdate {
            name: source.name.clone(),
            git: git.url.clone(),
            followed,
            from: previous.map(|p| p.commit.clone()),
            to: commit,
            moved,
            back,
            commits,
            files,
            examples,
            first_copy: !had_files && !synced.files.is_empty(),
            failed: synced.failed,
        },
        pin,
    )))
}

/// Where each of `paths` that's a tested example stands, read from the
/// copies in `folder`: its config, every observation, and the file, as the
/// pages have it. A path that isn't an example, or has no copy, isn't
/// listed. Read only: nothing in the config is run.
fn example_states(folder: &Folder<'_>, paths: &BTreeSet<String>) -> BTreeMap<String, ExampleState> {
    let examples = folder.examples();
    let Some(Ok(config)) = &examples.config else {
        return BTreeMap::new();
    };
    paths
        .iter()
        .filter(|path| config.is_example(path))
        .filter_map(|path| {
            let bytes = folder.read(path)?;
            let status = Status::across(examples.readable().map(|(_, o)| o.status(path, &bytes)));
            Some((path.clone(), ExampleState::from(status)))
        })
        .collect()
}

/// Each source's pin and the state of its copies, read from the files alone.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct StatusReport {
    /// Each source in another repository, in declaration order.
    pub sources: Vec<SourceStatus>,
}

/// One source's pin and copies.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct SourceStatus {
    /// The source.
    pub name: String,
    /// Its repository.
    pub git: String,
    /// The branch an update follows; `None` for the repository's default.
    pub branch: Option<String>,
    /// Its pin, when it has one to this repository.
    pub commit: Option<String>,
    /// Each copy, and each file snippets name with none, by path.
    pub files: Vec<CopyStatus>,
}

/// A file of a source's copies.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct CopyStatus {
    /// Its path in the source.
    pub path: String,
    /// Its state.
    pub state: CopyState,
}

/// The state of a copy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum CopyState {
    /// It's the file the lock pins, and a snippet uses it, or it's the
    /// source's tested examples' config or one of their observations.
    Current,
    /// It isn't the file the lock pins.
    Changed,
    /// The lock lists it, and it isn't there.
    Missing,
    /// It's there, and the lock doesn't list it.
    Unlocked,
    /// No snippet names it.
    Unused,
    /// A snippet names it, and it hasn't been copied.
    NotCopied,
    /// A snippet names it, and it isn't in the repository at the pin.
    NotAtPin,
}

impl StatusReport {
    /// Whether every source is pinned and every copy current.
    pub fn in_order(&self) -> bool {
        self.sources
            .iter()
            .all(|s| s.commit.is_some() && s.files.iter().all(|f| f.state == CopyState::Current))
    }
}

/// Each source's pin and the state of its copies. Reads only the project's
/// files: no `git`, no network.
pub fn status(view: &SourcesView) -> StatusReport {
    let mut report = StatusReport::default();
    for source in &view.sources {
        let Some(git) = &source.git else {
            continue;
        };
        let pin = view.pin(source);
        let folder = view.folder(source);
        let needed = view.needed(source);
        let on_disk: BTreeSet<String> = folder.files().into_iter().collect();
        let mut files: BTreeMap<String, CopyState> = BTreeMap::new();
        if let Some(pin) = pin {
            for (path, locked) in &pin.files {
                let state = match folder.read(path) {
                    None if !on_disk.contains(path) => CopyState::Missing,
                    Some(bytes) if file_hash(&bytes) == locked.hash => {
                        if needed.contains(path) || is_examples_file(path) {
                            CopyState::Current
                        } else {
                            CopyState::Unused
                        }
                    }
                    _ => CopyState::Changed,
                };
                files.insert(path.clone(), state);
            }
        }
        for path in &on_disk {
            files.entry(path.clone()).or_insert(CopyState::Unlocked);
        }
        for path in &needed {
            let state = if pin.is_some_and(|p| p.missing.contains(path)) {
                CopyState::NotAtPin
            } else {
                CopyState::NotCopied
            };
            files.entry(path.clone()).or_insert(state);
        }
        report.sources.push(SourceStatus {
            name: source.name.clone(),
            git: git.url.clone(),
            branch: git.branch.clone(),
            commit: pin.map(|p| p.commit.clone()),
            files: files
                .into_iter()
                .map(|(path, state)| CopyStatus { path, state })
                .collect(),
        });
    }
    report
}
