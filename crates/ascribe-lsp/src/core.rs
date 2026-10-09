//! The server's state: open documents, the project, and what to compute.
//!
//! Everything here runs under one lock (`server::Shared`). Handlers change the
//! state and queue work; the worker (`compute`) takes a snapshot of what to do,
//! computes without the lock, and comes back to publish under it, so a change
//! and the check that a result is still current are never interleaved.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use ascribe_check::Diagnostic;
use ascribe_core::path::{normalize, relative_path};
use ascribe_core::{FileId, LineIndex, RelPath};
use ascribe_model::ContentModel;
use ascribe_resolve::{
    Affected, ApplyError, Change, DiskFs, FileSystem, IncrementalProject, Layout, ResolvedCache,
    in_nested_project, is_source_path,
};
use crossbeam_channel::Sender;
use lsp_server::{Message, Notification, Request};
use lsp_types::{
    DidChangeWatchedFilesRegistrationOptions, FileChangeType, FileEvent, FileSystemWatcher,
    GlobPattern, PublishDiagnosticsParams, Registration, RegistrationParams,
    TextDocumentContentChangeEvent, Uri,
};

use crate::compute::to_lsp;
use crate::docs::Doc;
use crate::fsx::{BufferFs, LayerFs};
use crate::nav::Ctx;
use crate::position::Encoding;
use crate::uri::{path_to_uri, uri_to_path};

/// The content model's file name, at the project root.
const MODEL_FILE: &str = "ascribe.toml";

/// Directories nothing in a documentation set lives in, whose changes are
/// ignored.
const IGNORED_DIRS: [&str; 3] = [".git", "node_modules", ".hg"];

/// A model that didn't load: its text and its problems.
#[derive(Clone, Debug)]
pub(crate) struct ModelProblem {
    pub text: String,
    pub diagnostics: Vec<Diagnostic>,
}

/// The last diagnostics published for a file.
struct Published {
    diagnostics: Vec<lsp_types::Diagnostic>,
    version: Option<i32>,
}

/// A loaded project.
pub(crate) struct Loaded {
    /// Which load this is: a job computed for an earlier one is dropped.
    pub epoch: u64,
    pub root: PathBuf,
    pub layout: Layout,
    pub inc: IncrementalProject,
    /// The latest model that loaded.
    pub model: Arc<ContentModel>,
    /// Its text.
    pub model_text: String,
    /// What the file-level checks probe for non-source files.
    pub fs: Arc<LayerFs>,
    /// Files whose diagnostics need computing.
    pub dirty: BTreeSet<RelPath>,
    /// The pages resolved for the editor's build, kept between rounds. Only the
    /// worker uses it, and first applies `pending`.
    pub cache: Arc<Mutex<ResolvedCache>>,
    /// What updates affected since the last round, in order, for `cache`.
    pub pending: Vec<Affected>,
    /// What each file included at the last round (see `compute::with_included`).
    pub direct_includes: BTreeMap<RelPath, BTreeSet<RelPath>>,
}

impl Loaded {
    /// The absolute path of a source file.
    pub(crate) fn source_path(&self, path: &RelPath) -> PathBuf {
        normalize(
            &self
                .root
                .join(self.layout.content_root.as_str())
                .join(path.as_str()),
        )
    }

    /// What a path on disk is to this project.
    fn classify(&self, abs: &Path) -> Option<Kind> {
        let project_rel = relative_path(&self.root, abs)?;
        if project_rel.as_str() == MODEL_FILE {
            return Some(Kind::Model);
        }
        if project_rel
            .segments()
            .any(|s| IGNORED_DIRS.contains(&s) || s == "node_modules")
        {
            return None;
        }
        let content = normalize(&self.root.join(self.layout.content_root.as_str()));
        // A file in a nested project's folder is no source of this one, open
        // or not, as a file in a hidden directory isn't.
        if let Some(content_path) = relative_path(&content, abs)
            && content_path.is_inside()
            && self.inc.snapshot().is_source(&content_path)
        {
            return Some(Kind::Source(content_path, project_rel));
        }
        Some(Kind::Asset(project_rel))
    }
}

enum Kind {
    /// `ascribe.toml`.
    Model,
    /// A source file: its content path and its project path.
    Source(RelPath, RelPath),
    /// Any other file: its project path.
    Asset(RelPath),
}

/// The server's state. It serves one project, found by looking upward from the
/// workspace folder.
pub(crate) struct Core {
    pub out: Sender<Message>,
    pub encoding: Encoding,
    pub folders: Vec<PathBuf>,
    pub docs: HashMap<PathBuf, Doc>,
    /// The project's `ascribe.toml`, once found.
    pub config: Option<PathBuf>,
    pub loaded: Option<Loaded>,
    /// Set while the current text of `ascribe.toml` doesn't load.
    pub model_problem: Option<ModelProblem>,
    pub shutdown: bool,
    pub can_watch: bool,
    /// What `ascribe/preview` keeps between requests (`preview.rs`).
    pub(crate) preview_routes: crate::preview::RouteCache,
    /// The review base, while review is on (`review.rs`).
    pub(crate) review: Option<Arc<crate::review::ReviewBase>>,
    epoch: u64,
    published: HashMap<PathBuf, Published>,
    /// Closed files changed on disk, published at their next diagnostics
    /// even when those didn't change, so a client waiting on the change
    /// learns the server has caught up.
    forced: HashSet<PathBuf>,
    next_id: i32,
}

impl Core {
    pub(crate) fn new(out: Sender<Message>) -> Core {
        Core {
            out,
            encoding: Encoding::Utf16,
            folders: Vec::new(),
            docs: HashMap::new(),
            config: None,
            loaded: None,
            model_problem: None,
            shutdown: false,
            can_watch: false,
            preview_routes: Default::default(),
            review: None,
            epoch: 0,
            published: HashMap::new(),
            forced: HashSet::new(),
            next_id: 0,
        }
    }

    pub(crate) fn send(&self, message: impl Into<Message>) {
        // A closed channel means the client is gone; the main loop notices.
        let _ = self.out.send(message.into());
    }

    fn log(&self, text: &str) {
        crate::log::line(format_args!("{text}"));
    }

    // -- Startup ------------------------------------------------------------

    /// Finds the project's `ascribe.toml` at or above the workspace folders
    /// and loads it.
    pub(crate) fn start(&mut self) {
        if self.config.is_none() {
            self.config = find_config(&self.folders).map(|p| normalize(&p));
        }
        self.log(&start_message(self.config.as_deref(), &self.folders));
        if self.config.is_some() {
            self.sync_model();
        }
    }

    /// Asks the client to watch files, when it can.
    pub(crate) fn register_watchers(&mut self) {
        if !self.can_watch {
            return;
        }
        let options = DidChangeWatchedFilesRegistrationOptions {
            watchers: vec![FileSystemWatcher {
                glob_pattern: GlobPattern::String("**/*".to_owned()),
                kind: None,
            }],
        };
        self.next_id += 1;
        let params = RegistrationParams {
            registrations: vec![Registration {
                id: "ascribe-watched-files".to_owned(),
                method: "workspace/didChangeWatchedFiles".to_owned(),
                register_options: serde_json::to_value(options).ok(),
            }],
        };
        self.send(Request::new(
            lsp_server::RequestId::from(self.next_id),
            "client/registerCapability".to_owned(),
            params,
        ));
    }

    // -- The model ----------------------------------------------------------

    /// The current text of `ascribe.toml`: the open buffer, or the file.
    fn model_text(&self) -> Option<String> {
        let config = self.config.as_ref()?;
        match self.docs.get(config) {
            Some(doc) => Some(doc.text.clone()),
            // Outside FileSystem: the content model, which says where the
            // content root is.
            None => std::fs::read_to_string(config).ok(),
        }
    }

    /// Reads `ascribe.toml` again and brings the project in line with it.
    pub(crate) fn sync_model(&mut self) {
        let Some(config) = self.config.clone() else {
            return;
        };
        let Some(text) = self.model_text() else {
            self.log(&format!("can't read {}", config.display()));
            return;
        };
        let root = config
            .parent()
            .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
        match ascribe_model::load_str_in(&text, FileId::new(0), &root) {
            Err(issues) => {
                // The project keeps the last model that
                // loaded, and the problems go on `ascribe.toml`.
                self.model_problem = Some(ModelProblem {
                    text,
                    diagnostics: issues.iter().map(Diagnostic::from_issue).collect(),
                });
            }
            Ok(model) => {
                self.model_problem = None;
                self.adopt_model(Arc::new(model), text);
            }
        }
        self.publish_model_diagnostics();
    }

    fn adopt_model(&mut self, model: Arc<ContentModel>, text: String) {
        let Some(loaded) = self.loaded.as_mut() else {
            self.load_project(model, text);
            return;
        };
        match loaded.inc.apply([Change::Model(model.clone())]) {
            Ok(affected) => {
                loaded.model = model;
                loaded.model_text = text;
                self.absorb(&affected);
            }
            Err(ApplyError::LayoutChanged | ApplyError::NestedProjectChanged) => {
                self.load_project(model, text);
            }
        }
    }

    /// Loads the project again with the model it has, after a change the
    /// incremental update can't apply in place.
    fn reload(&mut self) {
        let Some(loaded) = self.loaded.as_ref() else {
            return;
        };
        let (model, text) = (loaded.model.clone(), loaded.model_text.clone());
        self.load_project(model, text);
    }

    /// Loads the project from scratch, with the open buffers over the disk:
    /// the first load, after a model change that moves the content root or
    /// the output directory, and when a nested project's `ascribe.toml`
    /// appears or goes.
    fn load_project(&mut self, model: Arc<ContentModel>, text: String) {
        let Some(config) = self.config.clone() else {
            return;
        };
        let root = config
            .parent()
            .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
        let layout = Layout::from_model(&model);
        let disk = DiskFs::new(&root, &layout);
        let content = normalize(&root.join(layout.content_root.as_str()));
        let buffers: BTreeMap<RelPath, String> = self
            .docs
            .iter()
            .filter_map(|(path, doc)| {
                let rel = relative_path(&content, path)?;
                (rel.is_inside() && is_source_path(&rel)).then(|| (rel, doc.text.clone()))
            })
            .collect();
        let inc = IncrementalProject::load(
            model.clone(),
            layout.clone(),
            BufferFs::new(disk.clone(), buffers),
        );
        let snapshot = inc.snapshot();
        let mut dirty: BTreeSet<RelPath> = snapshot.files().map(|f| f.path.clone()).collect();
        dirty.extend(snapshot.unreadable().iter().map(|u| u.path.clone()));
        self.epoch += 1;
        let loaded = Loaded {
            epoch: self.epoch,
            root,
            layout,
            inc,
            model,
            model_text: text,
            fs: Arc::new(LayerFs::new(disk)),
            dirty,
            cache: Arc::new(Mutex::new(ResolvedCache::new())),
            pending: Vec::new(),
            direct_includes: BTreeMap::new(),
        };
        // What was published for files the new project doesn't have goes.
        let keep: BTreeSet<PathBuf> = snapshot
            .files()
            .map(|f| loaded.source_path(&f.path))
            .chain(
                snapshot
                    .unreadable()
                    .iter()
                    .map(|u| loaded.source_path(&u.path)),
            )
            .chain(self.config.clone())
            .collect();
        let gone: Vec<PathBuf> = self
            .published
            .keys()
            .filter(|p| !keep.contains(*p))
            .cloned()
            .collect();
        self.loaded = Some(loaded);
        for path in gone {
            self.clear(&path);
        }
    }

    // -- Documents ----------------------------------------------------------

    fn doc_path(uri: &Uri) -> Option<PathBuf> {
        uri_to_path(uri).map(|p| normalize(&p))
    }

    pub(crate) fn did_open(&mut self, uri: Uri, version: i32, text: String) {
        let Some(path) = Core::doc_path(&uri) else {
            return;
        };
        self.docs.insert(
            path.clone(),
            Doc {
                uri,
                version,
                text: text.clone(),
            },
        );
        self.document_changed(&path, text);
    }

    pub(crate) fn did_change(
        &mut self,
        uri: &Uri,
        version: i32,
        changes: Vec<TextDocumentContentChangeEvent>,
    ) {
        let Some(path) = Core::doc_path(uri) else {
            return;
        };
        let encoding = self.encoding;
        let Some(doc) = self.docs.get_mut(&path) else {
            self.log(&format!(
                "change for a document that isn't open: {}",
                uri.as_str()
            ));
            return;
        };
        doc.apply(version, changes, encoding);
        let text = doc.text.clone();
        self.document_changed(&path, text);
    }

    pub(crate) fn did_close(&mut self, uri: &Uri) {
        let Some(path) = Core::doc_path(uri) else {
            return;
        };
        if self.docs.remove(&path).is_none() {
            return;
        }
        // Closing reverts to the disk.
        if self.config.as_ref() == Some(&path) {
            self.sync_model();
            return;
        }
        self.refresh_from_disk(&path);
    }

    /// An open document's text changed.
    fn document_changed(&mut self, path: &Path, text: String) {
        if self.config.as_deref() == Some(path) {
            self.sync_model();
            return;
        }
        let Some(loaded) = self.loaded.as_ref() else {
            return;
        };
        if let Some(Kind::Source(content, project)) = loaded.classify(path) {
            self.apply(
                vec![Change::Edited {
                    path: content,
                    text,
                }],
                &[(project, true)],
            );
        }
    }

    // -- Files on disk --------------------------------------------------------

    /// Follows what the file watcher reports: files and directories created,
    /// changed, or deleted by anything else.
    pub(crate) fn did_change_watched(&mut self, events: Vec<FileEvent>) {
        let mut sync_model = false;
        let mut changes: Vec<Change> = Vec::new();
        let mut mirror: Vec<(RelPath, bool)> = Vec::new();
        for event in events {
            let Some(path) = Core::doc_path(&event.uri) else {
                continue;
            };
            if self.config.is_none()
                && path.file_name().is_some_and(|n| n == MODEL_FILE)
                && event.typ != FileChangeType::DELETED
                && self
                    .folders
                    .iter()
                    .any(|f| path.parent() == Some(f.as_path()))
            {
                self.config = Some(path.clone());
                self.log(&start_message(Some(&path), &self.folders));
                sync_model = true;
                continue;
            }
            if self.config.as_deref() == Some(path.as_path()) {
                // An open buffer wins over the file on disk.
                if !self.docs.contains_key(&path) {
                    sync_model = true;
                }
                continue;
            }
            if event.typ == FileChangeType::DELETED {
                self.collect_deleted(&path, &mut changes, &mut mirror);
            } else {
                self.collect_present(&path, &mut changes, &mut mirror);
            }
        }
        if !changes.is_empty() {
            let written: Vec<RelPath> = changes
                .iter()
                .filter_map(|change| match change {
                    Change::Edited { path, .. } | Change::Unreadable { path, .. } => {
                        Some(path.clone())
                    }
                    _ => None,
                })
                .collect();
            self.apply(changes, &mirror);
            if let Some(loaded) = self.loaded.as_mut() {
                for path in written {
                    self.forced.insert(loaded.source_path(&path));
                    // Checked again even when its text is what it was.
                    loaded.dirty.insert(path);
                }
            }
        }
        if sync_model {
            self.sync_model();
        }
    }

    /// A file or directory exists on disk: its files are created or changed.
    fn collect_present(
        &self,
        path: &Path,
        changes: &mut Vec<Change>,
        mirror: &mut Vec<(RelPath, bool)>,
    ) {
        let Some(loaded) = self.loaded.as_ref() else {
            return;
        };
        // Outside FileSystem: what the editor says changed, which can be any
        // path in the workspace. A source file in it is read through DiskFs
        // below.
        let Ok(meta) = std::fs::metadata(path) else {
            return self.collect_deleted(path, changes, mirror);
        };
        if meta.is_dir() {
            let Ok(entries) = std::fs::read_dir(path) else {
                return;
            };
            for entry in entries.flatten() {
                self.collect_present(&entry.path(), changes, mirror);
            }
            return;
        }
        match loaded.classify(path) {
            Some(Kind::Source(content, project)) => {
                // An open buffer wins over the file on disk.
                if self.docs.contains_key(path) {
                    return;
                }
                // Read as the source index reads it, so a link out of the
                // content root is refused here too.
                match DiskFs::new(&loaded.root, &loaded.layout).read(&content) {
                    Ok(text) => {
                        changes.push(Change::Edited {
                            path: content,
                            text,
                        });
                        mirror.push((project, true));
                    }
                    // Gone between the event and the read: a deletion.
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                        changes.push(Change::Deleted { path: content });
                        mirror.push((project, false));
                    }
                    Err(e) => {
                        // A source that exists but can't be
                        // read (not UTF-8, or refused) is reported as
                        // `source-unreadable`, as `ascribe check` does.
                        self.log(&format!("can't read {}: {e}", path.display()));
                        changes.push(Change::Unreadable {
                            path: content,
                            reason: e.to_string(),
                        });
                        mirror.push((project, true));
                    }
                }
            }
            Some(Kind::Asset(project)) => {
                changes.push(Change::AssetCreated {
                    path: project.clone(),
                });
                mirror.push((project, true));
            }
            Some(Kind::Model) | None => {}
        }
    }

    /// A file or directory is gone from disk: it, and everything the project
    /// knew under it, is deleted.
    fn collect_deleted(
        &self,
        path: &Path,
        changes: &mut Vec<Change>,
        mirror: &mut Vec<(RelPath, bool)>,
    ) {
        let Some(loaded) = self.loaded.as_ref() else {
            return;
        };
        match loaded.classify(path) {
            Some(Kind::Source(content, project)) => {
                if !self.docs.contains_key(path) {
                    changes.push(Change::Deleted { path: content });
                    mirror.push((project, false));
                }
            }
            Some(Kind::Asset(project)) => {
                changes.push(Change::AssetDeleted {
                    path: project.clone(),
                });
                mirror.push((project, false));
            }
            Some(Kind::Model) | None => return,
        }
        // The path may have been a directory: everything known below it goes.
        let Some(dir) = relative_path(&loaded.root, path) else {
            return;
        };
        let prefix = format!("{dir}/");
        let snapshot = loaded.inc.snapshot();
        let mut assets: BTreeSet<RelPath> = BTreeSet::new();
        // A nested project's folder, or one holding it, going takes its
        // `ascribe.toml` with it.
        for folder in snapshot.nested_projects() {
            if let Ok(model) = folder.join(MODEL_FILE) {
                let project = loaded.layout.project_path(&model);
                if project.as_str().starts_with(&prefix) {
                    assets.insert(project);
                }
            }
        }
        for file in snapshot.files() {
            let project = loaded.layout.project_path(&file.path);
            if project.as_str().starts_with(&prefix)
                && !self.docs.contains_key(&loaded.source_path(&file.path))
            {
                changes.push(Change::Deleted {
                    path: file.path.clone(),
                });
                mirror.push((project, false));
            }
            for reference in &file.references {
                if let ascribe_resolve::Target::Local(local) = &reference.target
                    && !local.source
                    && let Some(asset) = &local.path
                {
                    let project = loaded.layout.project_path(asset);
                    if project.as_str().starts_with(&prefix) {
                        assets.insert(project);
                    }
                }
            }
        }
        for asset in assets {
            changes.push(Change::AssetDeleted {
                path: asset.clone(),
            });
            mirror.push((asset, false));
        }
    }

    /// Brings one file in line with the disk, after its buffer is closed.
    fn refresh_from_disk(&mut self, path: &Path) {
        let mut changes = Vec::new();
        let mut mirror = Vec::new();
        // Outside FileSystem: whether the file whose buffer closed is still
        // there; `collect_present` reads it through DiskFs.
        if path.is_file() {
            self.collect_present(path, &mut changes, &mut mirror);
        } else {
            self.collect_deleted(path, &mut changes, &mut mirror);
        }
        if !changes.is_empty() {
            self.apply(changes, &mirror);
        }
    }

    // -- Applying changes -----------------------------------------------------

    /// Applies a batch to the project, and queues what it affects.
    fn apply(&mut self, changes: Vec<Change>, mirror: &[(RelPath, bool)]) {
        let Some(loaded) = self.loaded.as_mut() else {
            return;
        };
        for (path, present) in mirror {
            loaded.fs.set(path, *present);
        }
        match loaded.inc.apply(changes) {
            Ok(affected) => self.absorb(&affected),
            // A nested project's `ascribe.toml` came or went, which changes
            // which files are sources: the disk and the open buffers are read
            // again.
            Err(ApplyError::NestedProjectChanged) => {
                self.log("a nested project appeared or went away; loading the project again");
                self.reload();
            }
            // Only a model change moves the layout, and those go through
            // `sync_model`.
            Err(ApplyError::LayoutChanged) => self.log("unexpected layout change"),
        }
    }

    /// Queues the files an update affects, and clears the diagnostics of the
    /// ones it removed.
    fn absorb(&mut self, affected: &Affected) {
        let Some(loaded) = self.loaded.as_mut() else {
            return;
        };
        if !affected.is_empty() {
            loaded.pending.push(affected.clone());
        }
        loaded.dirty.extend(affected.recheck.iter().cloned());
        let mut cleared = Vec::new();
        for path in &affected.removed {
            loaded.dirty.remove(path);
            // What a deleted page included no longer has it as an includer.
            if let Some(targets) = loaded.direct_includes.remove(path) {
                loaded.dirty.extend(targets);
            }
            cleared.push(loaded.source_path(path));
        }
        // A file that can't be read left the index like a deleted one, but it
        // still exists: its diagnostic is `source-unreadable`.
        for path in &affected.unreadable {
            if let Some(targets) = loaded.direct_includes.remove(path) {
                loaded.dirty.extend(targets);
            }
            loaded.dirty.insert(path.clone());
        }
        for path in cleared {
            self.clear(&path);
        }
    }

    // -- Publishing -------------------------------------------------------------

    /// Publishes the diagnostics of `ascribe.toml`: the model's own problems
    /// when it doesn't load, otherwise its warnings.
    fn publish_model_diagnostics(&mut self) {
        let Some(config) = self.config.clone() else {
            return;
        };
        let (text, diagnostics): (&str, Vec<Diagnostic>) = match (&self.model_problem, &self.loaded)
        {
            (Some(problem), _) => (&problem.text, problem.diagnostics.clone()),
            (None, Some(loaded)) => (
                &loaded.model_text,
                loaded
                    .model
                    .warnings
                    .iter()
                    .map(Diagnostic::from_issue)
                    .collect(),
            ),
            (None, None) => return,
        };
        let index = LineIndex::new(text);
        let lsp = diagnostics
            .iter()
            .map(|d| to_lsp(d, &index, self.encoding, &|_| None))
            .collect();
        self.publish(&config, lsp);
    }

    // A closed file keeps its diagnostics.
    /// Publishes a file's diagnostics when they, or the version of the document
    /// they're for, changed since the last time, or when the file was changed
    /// on disk since.
    pub(crate) fn publish(&mut self, path: &Path, diagnostics: Vec<lsp_types::Diagnostic>) {
        let doc = self.docs.get(path);
        let version = doc.map(|d| d.version);
        let is_open = doc.is_some();
        let changed = match self.published.get(path) {
            None => !diagnostics.is_empty() || is_open,
            Some(last) => last.diagnostics != diagnostics || last.version != version,
        };
        let forced = self.forced.remove(path);
        if !changed && !forced {
            return;
        }
        let uri = match doc {
            Some(doc) => Some(doc.uri.clone()),
            None => path_to_uri(path),
        };
        let Some(uri) = uri else {
            return;
        };
        self.send(Notification::new(
            "textDocument/publishDiagnostics".to_owned(),
            PublishDiagnosticsParams {
                uri,
                diagnostics: diagnostics.clone(),
                version,
            },
        ));
        self.published.insert(
            path.to_path_buf(),
            Published {
                diagnostics,
                version,
            },
        );
    }

    /// Clears a file's diagnostics, when it had any.
    fn clear(&mut self, path: &Path) {
        let Some(last) = self.published.remove(path) else {
            return;
        };
        if last.diagnostics.is_empty() {
            return;
        }
        let uri = match self.docs.get(path) {
            Some(doc) => Some(doc.uri.clone()),
            None => path_to_uri(path),
        };
        if let Some(uri) = uri {
            self.send(Notification::new(
                "textDocument/publishDiagnostics".to_owned(),
                PublishDiagnosticsParams {
                    uri,
                    diagnostics: Vec::new(),
                    version: None,
                },
            ));
        }
    }
}

/// The nearest `ascribe.toml` at or above a workspace folder, trying the
/// folders in order. It never looks below a folder.
fn find_config(folders: &[PathBuf]) -> Option<PathBuf> {
    folders
        .iter()
        .find_map(|folder| ascribe_check::Project::find_config(folder))
}

/// The line logged at startup: the project chosen, or why there is none.
fn start_message(config: Option<&Path>, folders: &[PathBuf]) -> String {
    match config {
        Some(config) => format!("using the project at {}", config.display()),
        None if folders.is_empty() => "no workspace folder, so no project".to_owned(),
        None => {
            let listed: Vec<String> = folders.iter().map(|f| f.display().to_string()).collect();
            format!("no ascribe.toml at or above {}", listed.join(", "))
        }
    }
}

impl Core {
    /// The diagnostics last published for a file: none when nothing was.
    pub(crate) fn published(&self, path: &Path) -> &[lsp_types::Diagnostic] {
        self.published
            .get(path)
            .map_or(&[], |p| p.diagnostics.as_slice())
    }

    /// The diagnostics last published, for each file that has any.
    pub(crate) fn all_published(
        &self,
    ) -> impl Iterator<Item = (&PathBuf, &[lsp_types::Diagnostic])> {
        self.published
            .iter()
            .map(|(path, p)| (path, p.diagnostics.as_slice()))
            .filter(|(_, d)| !d.is_empty())
    }

    /// Whether files are waiting to have their diagnostics computed.
    pub(crate) fn has_work(&self) -> bool {
        self.loaded.as_ref().is_some_and(|l| !l.dirty.is_empty())
    }

    /// What a semantic tokens request needs for a document: the snapshot, the
    /// file's content path, and the model.
    pub(crate) fn tokens_target(
        &self,
        uri: &Uri,
    ) -> Option<(ascribe_resolve::Snapshot, RelPath, Arc<ContentModel>)> {
        let path = Core::doc_path(uri)?;
        let loaded = self.loaded.as_ref()?;
        match loaded.classify(&path)? {
            Kind::Source(content, _) => {
                let snapshot = loaded.inc.snapshot();
                snapshot.file(&content)?;
                Some((snapshot, content, loaded.model.clone()))
            }
            Kind::Model | Kind::Asset(_) => None,
        }
    }

    /// What a navigation request needs for a document: the current snapshot,
    /// the file's content path, the model and its text. `None` for a document
    /// that isn't a source file of the project.
    pub(crate) fn nav_target(&self, uri: &Uri) -> Option<Ctx> {
        let path = Core::doc_path(uri)?;
        let loaded = self.loaded.as_ref()?;
        let Kind::Source(content, _) = loaded.classify(&path)? else {
            return None;
        };
        let snapshot = loaded.inc.snapshot();
        snapshot.file(&content)?;
        self.ctx(loaded, snapshot, content, &path)
    }

    /// What a request about the whole project works from, asked through any
    /// of the project's files: a source file, `ascribe.toml`, or any other
    /// file in its folder. For a file that isn't a source, the context's path
    /// is [`Ctx::PROJECT`], so paths are written from the content root.
    pub(crate) fn project_target(&self, uri: &Uri) -> Option<Ctx> {
        let path = Core::doc_path(uri)?;
        let loaded = self.loaded.as_ref()?;
        let content = match loaded.classify(&path)? {
            Kind::Source(content, _) => content,
            Kind::Model => RelPath::parse(Ctx::PROJECT).ok()?,
            Kind::Asset(project_rel) => {
                let content_dir = normalize(&loaded.root.join(loaded.layout.content_root.as_str()));
                // A file in a nested project's folder is that project's.
                let nested = relative_path(&content_dir, &path).is_some_and(|content| {
                    content.is_inside()
                        && in_nested_project(&content, loaded.inc.snapshot().nested_projects())
                });
                if !project_rel.is_inside() || nested {
                    return None;
                }
                RelPath::parse(Ctx::PROJECT).ok()?
            }
        };
        self.ctx(loaded, loaded.inc.snapshot(), content, &path)
    }

    fn ctx(
        &self,
        loaded: &Loaded,
        snapshot: ascribe_resolve::Snapshot,
        content: RelPath,
        path: &Path,
    ) -> Option<Ctx> {
        Some(Ctx {
            snapshot,
            path: content,
            version: self.docs.get(path).map(|d| d.version),
            model: loaded.model.clone(),
            model_text: loaded.model_text.clone(),
            model_problem: self.model_problem.is_some(),
            config: self.config.clone()?,
            content_dir: normalize(&loaded.root.join(loaded.layout.content_root.as_str())),
            encoding: self.encoding,
            fs: loaded.fs.clone(),
        })
    }

    /// A project snapshot for workspace-level refactorings.
    pub(crate) fn project_nav_target(&self) -> Option<Ctx> {
        let loaded = self.loaded.as_ref()?;
        let snapshot = loaded.inc.snapshot();
        let file = snapshot.files().next()?;
        let uri = path_to_uri(&loaded.source_path(&file.path))?;
        self.nav_target(&uri)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_start_message_names_the_project() {
        let config = PathBuf::from("proj").join("ascribe.toml");
        let text = start_message(Some(&config), &[PathBuf::from("proj")]);
        assert_eq!(text, format!("using the project at {}", config.display()));
    }

    #[test]
    fn the_start_message_says_when_there_is_no_workspace_folder() {
        assert_eq!(
            start_message(None, &[]),
            "no workspace folder, so no project"
        );
    }

    #[test]
    fn the_start_message_says_when_there_is_no_project() {
        let folders = [PathBuf::from("a"), PathBuf::from("b")];
        assert_eq!(
            start_message(None, &folders),
            "no ascribe.toml at or above a, b"
        );
    }
}
