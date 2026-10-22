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
use ascribe_core::{LineIndex, RelPath, Visited};
use ascribe_model::ContentModel;
use ascribe_resolve::{
    Affected, ApplyError, Change, DiskFs, FileSystem, IncrementalProject, Layout, ResolvedCache,
    in_nested_project, is_source_path,
};
use crossbeam_channel::Sender;
use lsp_server::{Message, Notification, Request};
use lsp_types::{
    DiagnosticSeverity, DidChangeWatchedFilesRegistrationOptions, FileChangeType, FileEvent,
    FileSystemWatcher, GlobPattern, LogMessageParams, MessageType, PublishDiagnosticsParams,
    Registration, RegistrationParams, TextDocumentContentChangeEvent, Uri,
};

use crate::compute::{Across, to_lsp};
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
    /// Each build's reading of each page, for `ascribe/search`.
    pub readings: Arc<crate::search::Readings>,
    /// What each file included at the last round (see `compute::with_included`).
    pub direct_includes: BTreeMap<RelPath, BTreeSet<RelPath>>,
    /// Whether the content checks across the project are to run again.
    pub across_dirty: bool,
    /// What they found when they last ran.
    pub across: Arc<Across>,
    /// `ascribe.baseline`, as the editor has it (`baseline.rs`).
    pub baseline: Arc<crate::baseline::Now>,
    /// The identities of each source file's findings at its last round
    /// that an entry of the baseline matches, so the entry isn't stale.
    pub matched: BTreeMap<RelPath, Vec<ascribe_check::identity::Identity>>,
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

    /// Whether `abs` is `ascribe.baseline`.
    pub(crate) fn is_baseline(&self, abs: &Path) -> bool {
        relative_path(&self.root, abs)
            .is_some_and(|p| p.as_str() == ascribe_check::baseline::BASELINE_FILE)
    }

    /// `ascribe.nav`'s path from the project root, when `abs` is it.
    pub(crate) fn nav_file(&self, abs: &Path) -> Option<RelPath> {
        relative_path(&self.root, abs).filter(|p| p.as_str() == ascribe_model::NAV_FILE)
    }

    /// What a path on disk is to this project.
    pub(crate) fn classify(&self, abs: &Path) -> Option<Kind> {
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
        if ascribe_resolve::is_memo_path(&project_rel, &self.layout) {
            return Some(Kind::Memo(project_rel));
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

pub(crate) enum Kind {
    /// `ascribe.toml`.
    Model,
    /// A source file: its content path and its project path.
    Source(RelPath, RelPath),
    /// A memo file (SPEC §7.7), committed or personal: its project path.
    Memo(RelPath),
    /// Any other file: its project path.
    Asset(RelPath),
}

/// The server's state. It serves one project, found by looking upward from the
/// workspace folder.
pub(crate) struct Core {
    pub out: Sender<Message>,
    pub encoding: Encoding,
    pub folders: Vec<PathBuf>,
    /// Whether, with no repository around the project, a source is read
    /// only inside `folders`: what an editor opened. Off for the hook's
    /// headless server, which a command runs (decision 77).
    pub bound_sources: bool,
    pub docs: HashMap<PathBuf, Doc>,
    /// The project's `ascribe.toml`, once found.
    pub config: Option<PathBuf>,
    pub loaded: Option<Loaded>,
    /// Set while the current text of `ascribe.toml` doesn't load.
    pub model_problem: Option<ModelProblem>,
    pub shutdown: bool,
    pub can_watch: bool,
    /// Whether the client registers lenses dynamically, so the server asks
    /// for them in its pages and its sources' files ([`Core::register_lenses`]).
    pub lens_registration: bool,
    /// Whether the client asks for every lens again when told to.
    pub lens_refresh: bool,
    /// The document filters the lenses are registered for now.
    lens_selector: Option<Vec<lsp_types::DocumentFilter>>,
    /// Whether the worker is computing a round.
    pub computing: bool,
    /// What `ascribe/preview` keeps between requests (`preview.rs`).
    pub(crate) preview_routes: crate::preview::RouteCache,
    /// What `ascribe/page` keeps between requests (`page.rs`).
    pub(crate) page_uses: crate::page::PageCache,
    /// The review base, while review is on (`review.rs`).
    pub(crate) review: Option<Arc<crate::review::ReviewBase>>,
    /// What Vale found, and what's queued for it (`prose.rs`).
    pub(crate) prose: crate::prose::State,
    epoch: u64,
    published: HashMap<PathBuf, Published>,
    /// The diagnostics last published for a file, without Vale's or what
    /// `@snippet`s put on it: what a change to either publishes again.
    checked: HashMap<PathBuf, Vec<lsp_types::Diagnostic>>,
    /// Closed files changed on disk, published at their next diagnostics
    /// even when those didn't change, so a client waiting on the change
    /// learns the server has caught up.
    forced: HashSet<PathBuf>,
    /// What each file's problems put on the code files its `@snippet`s show
    /// ([`Core::show_on_code_files`]), by the file's absolute path.
    shown: BTreeMap<PathBuf, crate::compute::Shown>,
    /// What `ascribe.baseline` holds back of each source file's problems,
    /// by its absolute path, and the version of the document it's for:
    /// what the hover and the quick fix that moves one to an `@intended`
    /// read (`baseline.rs`).
    held: HashMap<PathBuf, (Vec<lsp_types::Diagnostic>, Option<i32>)>,
    /// Whether a code file gets its `snippet-tags` problems: an editor's,
    /// not the hook's headless server, whose problems are `ascribe check`'s.
    pub show_on_code: bool,
    next_id: i32,
    /// What gives the day the checks run on.
    pub(crate) clock: Option<crate::Clock>,
}

impl Core {
    pub(crate) fn new(out: Sender<Message>) -> Core {
        Core {
            out,
            encoding: Encoding::Utf16,
            folders: Vec::new(),
            bound_sources: true,
            docs: HashMap::new(),
            config: None,
            loaded: None,
            model_problem: None,
            shutdown: false,
            can_watch: false,
            lens_registration: false,
            lens_refresh: false,
            lens_selector: None,
            computing: false,
            preview_routes: Default::default(),
            page_uses: Default::default(),
            review: None,
            prose: crate::prose::State::default(),
            epoch: 0,
            published: HashMap::new(),
            checked: HashMap::new(),
            forced: HashSet::new(),
            shown: BTreeMap::new(),
            held: HashMap::new(),
            show_on_code: true,
            next_id: 0,
            clock: None,
        }
    }

    /// The day the checks run on, if the server was given a clock.
    pub(crate) fn today(&self) -> Option<ascribe_core::Date> {
        self.clock.as_ref().and_then(|clock| clock())
    }

    pub(crate) fn send(&self, message: impl Into<Message>) {
        // A closed channel means the client is gone; the main loop notices.
        let _ = self.out.send(message.into());
    }

    fn log(&self, text: &str) {
        crate::log::line(format_args!("{text}"));
    }

    /// Tells the client which project the server uses, with the server's
    /// version. It isn't a problem, so it's `window/logMessage` at Info, not
    /// the log on standard error, which a client shows as errors.
    fn log_start(&self, config: Option<&Path>) {
        self.send(Notification::new(
            "window/logMessage".to_owned(),
            LogMessageParams {
                typ: MessageType::INFO,
                message: format!(
                    "ascribe-lsp {}: {}",
                    env!("CARGO_PKG_VERSION"),
                    start_message(config, &self.folders)
                ),
            },
        ));
    }

    // -- Startup ------------------------------------------------------------

    /// Finds the project's `ascribe.toml` at or above the workspace folders
    /// and loads it.
    pub(crate) fn start(&mut self) {
        if self.config.is_none() {
            self.config = find_config(&self.folders).map(|p| normalize(&p));
        }
        self.log_start(self.config.as_deref());
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

    /// Registers lenses for the project's pages and its sources' files
    /// (`shown.rs`), when the client registers them dynamically, and again
    /// when the content model moves the sources.
    fn register_lenses(&mut self) {
        if !self.lens_registration {
            return;
        }
        let Some(loaded) = self.loaded.as_ref() else {
            return;
        };
        let selector = crate::shown::selector(&loaded.root, &loaded.model);
        if self.lens_selector.as_ref() == Some(&selector) {
            return;
        }
        const ID: &str = "ascribe-code-lens";
        const METHOD: &str = "textDocument/codeLens";
        if self.lens_selector.is_some() {
            self.next_id += 1;
            self.send(Request::new(
                lsp_server::RequestId::from(self.next_id),
                "client/unregisterCapability".to_owned(),
                lsp_types::UnregistrationParams {
                    unregisterations: vec![lsp_types::Unregistration {
                        id: ID.to_owned(),
                        method: METHOD.to_owned(),
                    }],
                },
            ));
        }
        // `CodeLensRegistrationOptions`, which lsp-types doesn't have.
        let options = serde_json::json!({
            "documentSelector": selector,
            "resolveProvider": false,
        });
        self.next_id += 1;
        self.send(Request::new(
            lsp_server::RequestId::from(self.next_id),
            "client/registerCapability".to_owned(),
            RegistrationParams {
                registrations: vec![Registration {
                    id: ID.to_owned(),
                    method: METHOD.to_owned(),
                    register_options: Some(options),
                }],
            },
        ));
        self.lens_selector = Some(selector);
    }

    /// Asks the client for every lens again, when it can be asked: after a
    /// code file a page shows, or what the pages show, changed on disk, and
    /// after a memo file changed, and with it the lenses on what it's about.
    fn refresh_lenses(&mut self) {
        if !self.lens_refresh {
            return;
        }
        self.next_id += 1;
        self.send(Request::new(
            lsp_server::RequestId::from(self.next_id),
            "workspace/codeLens/refresh".to_owned(),
            serde_json::Value::Null,
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
        // With no repository around the project, a source is read only
        // inside a folder the editor opened (decision 77).
        let bound = if self.bound_sources {
            ascribe_model::SourceBound::Within(&self.folders)
        } else {
            ascribe_model::SourceBound::Anywhere
        };
        match ascribe_check::type_files::load_model(&text, &root, bound) {
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
                let vale = self
                    .loaded
                    .as_ref()
                    .and_then(|l| l.model.checks.vale.clone());
                self.adopt_model(Arc::new(model), text);
                self.prose_settings_changed(vale.as_ref());
                self.register_lenses();
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
                loaded.across_dirty = true;
                self.absorb(&affected);
                self.publish_open_configs();
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
        let fs = Arc::new(LayerFs::new(disk));
        // The open memo files, `ascribe.nav`, and `ascribe.baseline`, as
        // the editor has them.
        for (path, doc) in &self.docs {
            if let Some(project) = relative_path(&root, path)
                && (ascribe_resolve::is_memo_path(&project, &layout)
                    || project.as_str() == ascribe_model::NAV_FILE
                    || project.as_str() == ascribe_check::baseline::BASELINE_FILE)
            {
                fs.set_buffer(&project, Some(doc.text.clone()));
            }
        }
        let baseline = Arc::new(crate::baseline::Now::of(&fs));
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
            fs,
            dirty,
            cache: Arc::new(Mutex::new(ResolvedCache::new())),
            readings: Arc::default(),
            pending: Vec::new(),
            direct_includes: BTreeMap::new(),
            across_dirty: true,
            across: Arc::default(),
            baseline,
            matched: BTreeMap::new(),
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
        // Each file's share on the code files comes again with its
        // diagnostics; the code files are among `gone`.
        self.shown.clear();
        self.held.clear();
        for path in gone {
            self.clear(&path);
        }
        // The project read `ascribe.nav` from the disk; an open buffer wins.
        self.refresh_nav();
        self.publish_open_configs();
    }

    /// Gives the long-lived project `ascribe.nav` as the editor has it now,
    /// its open buffer over the disk, so the navigation it gives, which the
    /// Pages view draws, follows the file. Every change to the file comes
    /// here, as each sets `across_dirty` for the checks.
    fn refresh_nav(&mut self) {
        let Some(loaded) = self.loaded.as_mut() else {
            return;
        };
        let nav = RelPath::parse(ascribe_model::NAV_FILE)
            .ok()
            .and_then(|path| loaded.fs.read_file(&path).ok())
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .and_then(|text| {
                ascribe_model::NavFile::parse(&text, ascribe_core::FileId::new(0)).ok()
            })
            .map(Arc::new);
        if let Ok(affected) = loaded.inc.apply([Change::Nav(nav)]) {
            self.absorb(&affected);
        }
    }

    // -- `ascribe.baseline` ----------------------------------------------------

    /// Gives the project `ascribe.baseline` as the editor has it, when
    /// `abs` is it: its open buffer, or the disk's once `text` is `None`.
    /// Whether it was.
    fn set_baseline_buffer(&mut self, abs: &Path, text: Option<String>) -> bool {
        let Some(loaded) = self.loaded.as_ref() else {
            return false;
        };
        if !loaded.is_baseline(abs) {
            return false;
        }
        if let Ok(project) = RelPath::parse(ascribe_check::baseline::BASELINE_FILE) {
            loaded.fs.set_buffer(&project, text);
        }
        self.refresh_baseline();
        true
    }

    /// Reads `ascribe.baseline` again, its open buffer over the disk. When
    /// it changed, every file is checked again, so what it holds back
    /// follows it, and what's published on it with them.
    fn refresh_baseline(&mut self) {
        let Some(loaded) = self.loaded.as_mut() else {
            return;
        };
        let now = crate::baseline::Now::of(&loaded.fs);
        if now == *loaded.baseline {
            return;
        }
        loaded.baseline = Arc::new(now);
        let snapshot = loaded.inc.snapshot();
        loaded
            .dirty
            .extend(snapshot.files().map(|f| f.path.clone()));
        loaded.across_dirty = true;
        // Vale's alerts are held back as they're found.
        self.queue_all_prose();
    }

    /// Keeps what the baseline holds back of a source file's problems, for
    /// the version of the document they were found in.
    pub(crate) fn set_held(
        &mut self,
        abs: &Path,
        held: Vec<lsp_types::Diagnostic>,
        version: Option<i32>,
    ) {
        if held.is_empty() {
            self.held.remove(abs);
        } else {
            self.held.insert(abs.to_path_buf(), (held, version));
        }
    }

    /// How many of a file's problems the baseline holds back.
    pub(crate) fn held_count(&self, abs: &Path) -> usize {
        self.held.get(abs).map_or(0, |(held, _)| held.len())
    }

    /// Publishes what's on `ascribe.baseline` (`baseline.rs`): the line that
    /// doesn't parse, or each entry no finding matches. Its entries are
    /// matched once every file is checked, so it waits until no round is
    /// due.
    pub(crate) fn publish_baseline(&mut self) {
        let Some(loaded) = self.loaded.as_mut() else {
            return;
        };
        if !loaded.dirty.is_empty() || loaded.across_dirty {
            return;
        }
        // A file that's gone matches nothing.
        let snapshot = loaded.inc.snapshot();
        loaded
            .matched
            .retain(|path, _| snapshot.file(path).is_some());
        let matched: HashSet<&ascribe_check::identity::Identity> = loaded
            .matched
            .values()
            .flatten()
            .chain(&loaded.across.matched)
            .collect();
        let found = crate::baseline::published(
            &loaded.baseline,
            &loaded.model,
            &matched,
            crate::baseline::every_build(&loaded.model),
            self.encoding,
        );
        let path = loaded.root.join(ascribe_check::baseline::BASELINE_FILE);
        if found.is_empty() && !self.published.contains_key(&path) {
            return;
        }
        self.publish(&path, found);
    }

    // -- A source's `ascribe.examples.toml` -------------------------------------

    /// The source whose `ascribe.examples.toml` is at `path`, and the file's
    /// path from the project root.
    fn examples_config(&self, path: &Path) -> Option<(ascribe_model::Source, RelPath)> {
        let loaded = self.loaded.as_ref()?;
        let Some(Kind::Asset(project)) = loaded.classify(path) else {
            return None;
        };
        let source = crate::examples::config_source(&loaded.model, &project)?;
        Some((source.clone(), project))
    }

    /// Publishes the problems of an open `ascribe.examples.toml` whose text
    /// is `text`; `false` when `path` isn't a source's config.
    fn publish_config(&mut self, path: &Path, text: &str) -> bool {
        let Some((source, project)) = self.examples_config(path) else {
            return false;
        };
        let Some(loaded) = self.loaded.as_ref() else {
            return false;
        };
        let found = crate::examples::config_diagnostics(
            &loaded.model,
            &source,
            project.as_str(),
            text,
            self.encoding,
        );
        self.publish(path, found);
        true
    }

    /// Publishes the problems of every open `ascribe.examples.toml`, after
    /// the project or its sources change.
    fn publish_open_configs(&mut self) {
        let open: Vec<(PathBuf, String)> = self
            .docs
            .iter()
            .map(|(path, doc)| (path.clone(), doc.text.clone()))
            .collect();
        for (path, text) in open {
            self.publish_config(&path, &text);
        }
    }

    /// The keys that can be written at `position` of an open
    /// `ascribe.examples.toml`; `None` for any other document.
    pub(crate) fn config_completion(
        &self,
        uri: &Uri,
        position: lsp_types::Position,
    ) -> Option<Vec<lsp_types::CompletionItem>> {
        let path = Core::doc_path(uri)?;
        self.examples_config(&path)?;
        let text = &self.docs.get(&path)?.text;
        let index = ascribe_core::LineIndex::new(text);
        let offset = self.encoding.offset_lenient(&index, text, position);
        Some(crate::examples::complete_config(text, offset))
    }

    /// What's at `position` of the open `ascribe.toml`: on a type's name,
    /// where its template and guide come from (`hover::type_markdown`), when
    /// the file is the text the project's model loaded from; on a frontmatter
    /// field's key, which table declares it (`hover::model_field_markdown`),
    /// which finds the key in the text as it is and the field by its name.
    /// `None` for any other document.
    pub(crate) fn model_hover(
        &self,
        uri: &Uri,
        position: lsp_types::Position,
    ) -> Option<lsp_types::Hover> {
        let path = Core::doc_path(uri)?;
        if self.config.as_ref() != Some(&path) {
            return None;
        }
        let text = &self.docs.get(&path)?.text;
        let loaded = self.loaded.as_ref()?;
        let index = ascribe_core::LineIndex::new(text);
        let offset = self.encoding.offset_lenient(&index, text, position);
        let markdown = |span: ascribe_core::Span, value: String| {
            Some(lsp_types::Hover {
                contents: lsp_types::HoverContents::Markup(lsp_types::MarkupContent {
                    kind: lsp_types::MarkupKind::Markdown,
                    value,
                }),
                range: Some(lsp_types::Range {
                    start: self.encoding.position(&index, span.start())?,
                    end: self.encoding.position(&index, span.end())?,
                }),
            })
        };
        // A type's span is the loaded text's.
        if loaded.model_text == *text
            && let Some(t) = loaded
                .model
                .types
                .iter()
                .find(|t| !t.implicit && t.span.start() <= offset && offset <= t.span.end())
        {
            return markdown(t.span, crate::hover::type_markdown(t));
        }
        let (span, value) = crate::hover::model_field_markdown(&loaded.model, text, offset)?;
        markdown(span, value)
    }

    /// What can be written at `position` of an open `ascribe.nav`: its keys,
    /// the project's sections, and a section's pages (`nav_file.rs`).
    /// `None` for any other document.
    pub(crate) fn nav_completion(
        &self,
        uri: &Uri,
        position: lsp_types::Position,
    ) -> Option<Vec<lsp_types::CompletionItem>> {
        let path = Core::doc_path(uri)?;
        let loaded = self.loaded.as_ref()?;
        loaded.nav_file(&path)?;
        let text = &self.docs.get(&path)?.text;
        let index = ascribe_core::LineIndex::new(text);
        let offset = self.encoding.offset_lenient(&index, text, position);
        let snapshot = loaded.inc.snapshot();
        let navigation = ascribe_resolve::Navigation::of(snapshot.project(), None);
        let sections = crate::nav_file::sections(&navigation);
        let range = |start: usize, end: usize| {
            Some(lsp_types::Range {
                start: self.encoding.position(&index, start)?,
                end: self.encoding.position(&index, end)?,
            })
        };
        Some(crate::nav_file::complete(text, offset, &sections, &range))
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
        self.queue_prose(&path);
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
        if !self.docs.contains_key(&path) {
            self.log(&format!(
                "change for a document that isn't open: {}",
                uri.as_str()
            ));
            return;
        }
        self.prose_edited(&path, &changes);
        let Some(doc) = self.docs.get_mut(&path) else {
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
        self.prose_closed(&path);
        // A config's problems are shown while it's open; `ascribe check`
        // reports them at the source.
        if self.examples_config(&path).is_some() {
            self.clear(&path);
        }
        // Closing reverts to the disk.
        if self.config.as_ref() == Some(&path) {
            self.sync_model();
            return;
        }
        if let Some(loaded) = self.loaded.as_mut()
            && let Some(Kind::Memo(project)) = loaded.classify(&path)
        {
            loaded.fs.set_buffer(&project, None);
            loaded.across_dirty = true;
            self.page_uses.forget_memos();
            self.refresh_lenses();
            return;
        }
        if let Some(loaded) = self.loaded.as_mut()
            && let Some(project) = loaded.nav_file(&path)
        {
            loaded.fs.set_buffer(&project, None);
            loaded.across_dirty = true;
            self.refresh_nav();
            return;
        }
        if self.set_baseline_buffer(&path, None) {
            return;
        }
        self.refresh_from_disk(&path);
    }

    /// A document was saved: the content checks across the project run
    /// again, and Vale checks its prose.
    pub(crate) fn did_save(&mut self, uri: &Uri) {
        let Some(path) = Core::doc_path(uri) else {
            return;
        };
        if let Some(loaded) = self.loaded.as_mut()
            && matches!(loaded.classify(&path), Some(Kind::Source(..)))
        {
            loaded.across_dirty = true;
            // What its `@snippet`s show is counted in the code files' lenses.
            self.refresh_lenses();
        }
        self.queue_prose(&path);
    }

    /// An open document's text changed.
    fn document_changed(&mut self, path: &Path, text: String) {
        if self.config.as_deref() == Some(path) {
            self.sync_model();
            return;
        }
        if self.publish_config(path, &text) {
            return;
        }
        let Some(loaded) = self.loaded.as_mut() else {
            return;
        };
        // A memo file isn't a source: the checks across the project, which
        // place each memo and check it, run again with its text.
        if let Some(Kind::Memo(project)) = loaded.classify(path) {
            loaded.fs.set_buffer(&project, Some(text));
            loaded.across_dirty = true;
            self.page_uses.forget_memos();
            self.refresh_lenses();
            return;
        }
        // Nor is `ascribe.nav`: the checks across the project hold it to the
        // project as it's typed.
        if let Some(project) = loaded.nav_file(path) {
            loaded.fs.set_buffer(&project, Some(text));
            loaded.across_dirty = true;
            self.refresh_nav();
            return;
        }
        // Nor is `ascribe.baseline`: what it holds back follows it as it's
        // typed.
        if loaded.is_baseline(path) {
            self.set_baseline_buffer(path, Some(text));
            return;
        }
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
        // This page's rows read files the snapshot doesn't hold.
        self.page_uses.forget();
        let mut sync_model = false;
        let mut memos = false;
        let mut changes: Vec<Change> = Vec::new();
        let mut mirror: Vec<(RelPath, bool)> = Vec::new();
        let mut visited = Visited::default();
        let mut refresh = false;
        let mut nav = false;
        let mut baseline = false;
        let mut redrawn = BTreeSet::new();
        for event in events {
            let Some(path) = Core::doc_path(&event.uri) else {
                continue;
            };
            if let Some(loaded) = self.loaded.as_ref() {
                if crate::shown::used_by_a_source(&loaded.root, &loaded.model, &path) {
                    refresh = true;
                } else if !relative_path(&loaded.root, &path).is_some_and(|p| p.is_inside()) {
                    // Outside the project, only a source's files matter: a
                    // build's output in a source's folder isn't walked.
                    continue;
                }
            }
            if self.config.is_none()
                && path.file_name().is_some_and(|n| n == MODEL_FILE)
                && event.typ != FileChangeType::DELETED
                && self
                    .folders
                    .iter()
                    .any(|f| path.parent() == Some(f.as_path()))
            {
                self.config = Some(path.clone());
                self.log_start(Some(&path));
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
            if let Some(loaded) = self.loaded.as_ref()
                && path
                    == loaded
                        .root
                        .join(ascribe_check::type_files::lock_path().as_str())
            {
                // A type that names no template or guide takes them from
                // the lock, so the model is loaded again.
                sync_model = true;
                continue;
            }
            if let Some(loaded) = self.loaded.as_ref()
                && loaded.is_baseline(&path)
            {
                // Read again, as `ascribe baseline write` and `prune` change
                // it; an open buffer of it wins.
                baseline = true;
                continue;
            }
            if let Some(loaded) = self.loaded.as_mut()
                && (path == loaded.root.join(ascribe_model::PERMALINKS_FILE)
                    || loaded.nav_file(&path).is_some())
            {
                // The permalinks list and the navigation file aren't
                // sources: the checks across the project compare them with
                // the content. An open buffer of `ascribe.nav` wins.
                loaded.across_dirty = true;
                nav |= loaded.nav_file(&path).is_some();
                continue;
            }
            memos |= self.loaded.as_ref().is_some_and(|loaded| {
                relative_path(&loaded.root, &path).is_some_and(|project| {
                    [
                        ascribe_resolve::MEMOS_FOLDER,
                        ascribe_resolve::PERSONAL_MEMOS_FOLDER,
                    ]
                    .into_iter()
                    .filter_map(|folder| RelPath::parse(folder).ok())
                    .any(|folder| project.starts_with(&folder))
                })
            });
            if event.typ == FileChangeType::DELETED {
                self.collect_deleted(&path, &mut changes, &mut mirror);
            } else {
                self.collect_present(&path, &mut visited, &mut changes, &mut mirror);
            }
        }
        if !changes.is_empty() {
            refresh |= changes.iter().any(|change| {
                matches!(
                    change,
                    Change::Edited { .. } | Change::Deleted { .. } | Change::Unreadable { .. }
                )
            });
            let written: Vec<RelPath> = changes
                .iter()
                .filter_map(|change| match change {
                    Change::Edited { path, .. } | Change::Unreadable { path, .. } => {
                        Some(path.clone())
                    }
                    _ => None,
                })
                .collect();
            // A deleted source's diagnostics are cleared even when it had
            // none, so a client waiting on the deletion learns the server
            // has caught up.
            if let Some(loaded) = self.loaded.as_ref() {
                for change in &changes {
                    if let Change::Deleted { path } = change {
                        self.forced.insert(loaded.source_path(path));
                    }
                }
            }
            redrawn = self.apply(changes, &mirror);
            if let Some(loaded) = self.loaded.as_mut() {
                loaded.across_dirty = true;
                for path in written {
                    self.forced.insert(loaded.source_path(&path));
                    // Checked again even when its text is what it was.
                    loaded.dirty.insert(path);
                }
            }
        }
        if memos {
            self.refresh_lenses();
        }
        if nav {
            self.refresh_nav();
        }
        if baseline {
            self.refresh_baseline();
        }
        if sync_model {
            // Published again even when its diagnostics are what they were,
            // as a changed source's are.
            if let Some(config) = self.config.clone() {
                self.forced.insert(config);
            }
            self.sync_model();
        }
        if refresh {
            self.refresh_lenses();
        }
        self.pages_changed(&redrawn);
    }

    /// Tells the client which pages read differently because files changed
    /// on disk, so a preview of one draws it again: a page, a fragment it
    /// includes, or a file a `@snippet` in either shows
    /// ([`crate::preview::CHANGED_NOTIFICATION`]). The editor's own edits it
    /// follows itself.
    fn pages_changed(&self, pages: &BTreeSet<RelPath>) {
        let Some(loaded) = self.loaded.as_ref() else {
            return;
        };
        let uris: Vec<String> = pages
            .iter()
            .filter_map(|page| path_to_uri(&loaded.source_path(page)))
            .map(|uri| uri.as_str().to_owned())
            .collect();
        if !uris.is_empty() {
            self.send(Notification::new(
                crate::preview::CHANGED_NOTIFICATION.to_owned(),
                serde_json::json!({ "pages": uris }),
            ));
        }
    }

    /// A file or directory exists on disk: its files are created or changed.
    /// A directory is walked as the source index walks the content root,
    /// going into each folder once, by its real path, so a link back to one
    /// already walked isn't followed.
    fn collect_present(
        &self,
        path: &Path,
        visited: &mut Visited,
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
            if !matches!(visited.enter(path), Ok(Some(_))) {
                return;
            }
            let Ok(entries) = std::fs::read_dir(path) else {
                return;
            };
            for entry in entries.flatten() {
                self.collect_present(&entry.path(), visited, changes, mirror);
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
            Some(Kind::Asset(project) | Kind::Memo(project)) => {
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
            Some(Kind::Asset(project) | Kind::Memo(project)) => {
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
            self.collect_present(path, &mut Visited::default(), &mut changes, &mut mirror);
        } else {
            self.collect_deleted(path, &mut changes, &mut mirror);
        }
        if !changes.is_empty() {
            self.apply(changes, &mirror);
        }
    }

    // -- Applying changes -----------------------------------------------------

    /// Applies a batch to the project, queues what it affects, and returns
    /// the pages whose resolved form it may change ([`Affected::re_resolve`]).
    fn apply(&mut self, changes: Vec<Change>, mirror: &[(RelPath, bool)]) -> BTreeSet<RelPath> {
        let Some(loaded) = self.loaded.as_mut() else {
            return BTreeSet::new();
        };
        for (path, present) in mirror {
            loaded.fs.set(path, *present);
        }
        match loaded.inc.apply(changes) {
            Ok(affected) => {
                self.absorb(&affected);
                return affected.re_resolve;
            }
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
        BTreeSet::new()
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
        loaded.readings.absorb(affected);
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
    /// when it doesn't load, otherwise its warnings and what the content
    /// checks across the project found in it, at the levels `[checks]` sets.
    pub(crate) fn publish_model_diagnostics(&mut self) {
        let Some(config) = self.config.clone() else {
            return;
        };
        let Some((text, diagnostics)) = self.model_diagnostics() else {
            return;
        };
        let index = LineIndex::new(text);
        let lsp = diagnostics
            .iter()
            .map(|d| to_lsp(d, &index, self.encoding, &|_| None))
            .collect();
        self.publish(&config, lsp);
    }

    /// The text of `ascribe.toml` as the server has it now, and its
    /// diagnostics as they're published: the model's own problems when it
    /// doesn't load, otherwise its warnings and what the content checks
    /// across the project found in it, at the levels `[checks]` sets.
    fn model_diagnostics(&self) -> Option<(&str, Vec<Diagnostic>)> {
        match (&self.model_problem, &self.loaded) {
            (Some(problem), _) => Some((&problem.text, problem.diagnostics.clone())),
            (None, Some(loaded)) => {
                let mut found: Vec<Diagnostic> = loaded
                    .model
                    .warnings
                    .iter()
                    .map(Diagnostic::from_issue)
                    .collect();
                found.extend_from_slice(loaded.across.of_model(&loaded.model_text));
                Some((
                    &loaded.model_text,
                    ascribe_check::apply_levels(&loaded.model.checks, found),
                ))
            }
            (None, None) => None,
        }
    }

    /// `ascribe.toml` as it is now, for Project Settings, asked through any
    /// of the project's files: its text, even while it doesn't load, the
    /// version of its open buffer, and its problems.
    pub(crate) fn model_now(&self, uri: &Uri) -> Option<crate::settings::ModelNow> {
        self.project_target(uri)?;
        let config = self.config.clone()?;
        let (text, diagnostics) = self.model_diagnostics()?;
        Some(crate::settings::ModelNow {
            text: text.to_owned(),
            version: self.docs.get(&config).map(|d| d.version),
            config,
            diagnostics,
            encoding: self.encoding,
        })
    }

    // A closed file keeps its diagnostics.
    /// Publishes a file's diagnostics, with Vale's and what `@snippet`s put
    /// on it, when they, or the version of the document they're for, changed
    /// since the last time, or when the file was changed on disk since.
    pub(crate) fn publish(&mut self, path: &Path, diagnostics: Vec<lsp_types::Diagnostic>) {
        let prose = self.prose.found(path);
        let mut all = Vec::with_capacity(diagnostics.len() + prose.len());
        all.extend(diagnostics.iter().cloned());
        all.extend(prose.iter().cloned());
        // What `@snippet`s that show it put on it, when it's a code file.
        all.extend(self.code_share(path));
        self.checked.insert(path.to_path_buf(), diagnostics);
        let diagnostics = all;
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

    /// Clears a file's diagnostics, when it had any, or when it was deleted
    /// on disk since ([`Core::did_change_watched`]).
    fn clear(&mut self, path: &Path) {
        if self.shown.contains_key(path) {
            self.show_on_code_files(path, Vec::new());
        }
        self.checked.remove(path);
        self.held.remove(path);
        self.prose_closed_quietly(path);
        let forced = self.forced.remove(path);
        let had = self
            .published
            .remove(path)
            .is_some_and(|last| !last.diagnostics.is_empty());
        if !had && !forced {
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

impl Core {
    /// Keeps what a file's problems put on code files, `shown`, in place of
    /// what they put there before, and publishes each code file either
    /// touches again, its own diagnostics, if it has any, with every file's
    /// share ([`Core::code_share`]).
    pub(crate) fn show_on_code_files(&mut self, file: &Path, shown: crate::compute::Shown) {
        if !self.show_on_code {
            return;
        }
        let before = if shown.is_empty() {
            self.shown.remove(file)
        } else {
            self.shown.insert(file.to_path_buf(), shown.clone())
        };
        let before = before.unwrap_or_default();
        if before == shown {
            return;
        }
        let code_files: BTreeSet<PathBuf> = before
            .into_iter()
            .chain(shown)
            .map(|(path, _)| path)
            .collect();
        for code in code_files {
            // A code file can be a source the server checks too: what it
            // found there stays.
            let own = self.checked.get(&code).cloned().unwrap_or_default();
            self.publish(&code, own);
        }
    }

    /// What the files' problems put on `code`, a problem that several
    /// `@snippet`s show, in one file or several, once, naming each of them,
    /// at the most severe level any of them has it.
    fn code_share(&self, code: &Path) -> Vec<lsp_types::Diagnostic> {
        let rank = |d: &lsp_types::Diagnostic| match d.severity {
            Some(DiagnosticSeverity::ERROR) => 0,
            Some(DiagnosticSeverity::WARNING) => 1,
            Some(DiagnosticSeverity::INFORMATION) => 2,
            _ => 3,
        };
        let mut all: Vec<lsp_types::Diagnostic> = Vec::new();
        for (path, d) in self.shown.values().flatten() {
            if path != code {
                continue;
            }
            let same = all
                .iter_mut()
                .find(|o| o.range == d.range && o.code == d.code && o.message == d.message);
            match same {
                Some(same) => {
                    if rank(d) < rank(same) {
                        same.severity = d.severity;
                    }
                    let have = same.related_information.get_or_insert_with(Vec::new);
                    for r in d.related_information.iter().flatten() {
                        if !have.contains(r) {
                            have.push(r.clone());
                        }
                    }
                }
                None => all.push(d.clone()),
            }
        }
        all
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
    /// The diagnostics last published for a file without Vale's.
    pub(crate) fn checked(&self, path: &Path) -> &[lsp_types::Diagnostic] {
        self.checked.get(path).map_or(&[], Vec::as_slice)
    }

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

    /// Whether anything is queued or running: a round of diagnostics, or a
    /// check of the prose.
    pub(crate) fn busy(&self) -> bool {
        self.has_work() || self.computing || self.prose.has_work()
    }

    /// Whether files are waiting to have their diagnostics computed.
    pub(crate) fn has_work(&self) -> bool {
        self.loaded
            .as_ref()
            .is_some_and(|l| !l.dirty.is_empty() || l.across_dirty)
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
            Kind::Model | Kind::Asset(_) | Kind::Memo(_) => None,
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

    /// What a code action request works from: a source file's context, as
    /// [`Core::nav_target`] gives it, or the project's for `ascribe.toml`.
    pub(crate) fn code_action_target(&self, uri: &Uri) -> Option<Ctx> {
        self.nav_target(uri).or_else(|| {
            let path = Core::doc_path(uri)?;
            let is_model = self.config.as_ref() == Some(&path);
            if is_model {
                return self.project_target(uri);
            }
            // `ascribe.permalinks`, beside it: its fixes edit `ascribe.toml`.
            // And `ascribe.nav`, whose fixes edit itself.
            let loaded = self.loaded.as_ref()?;
            if path == loaded.root.join(ascribe_model::PERMALINKS_FILE)
                || loaded.nav_file(&path).is_some()
            {
                let content = RelPath::parse(Ctx::PROJECT).ok()?;
                return self.ctx(loaded, loaded.inc.snapshot(), content, &path);
            }
            None
        })
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
            Kind::Asset(project_rel) | Kind::Memo(project_rel) => {
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

    /// What a request about a memo file works from: the project's context,
    /// and the memo as the editor has it. `None` for any other document.
    pub(crate) fn memo_target(&self, uri: &Uri) -> Option<(Ctx, ascribe_resolve::MemoFile)> {
        let path = Core::doc_path(uri)?;
        let loaded = self.loaded.as_ref()?;
        let Kind::Memo(project) = loaded.classify(&path)? else {
            return None;
        };
        let text = match self.docs.get(&path) {
            Some(doc) => doc.text.clone(),
            None => String::from_utf8(loaded.fs.read_file(&project).ok()?).ok()?,
        };
        let ctx = self.project_target(uri)?;
        Some((ctx, ascribe_resolve::MemoFile::parse(project, text)))
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
            today: self.today(),
            held: self.held_now(path),
            baseline: loaded.baseline.clone(),
        })
    }

    /// What the baseline holds back of a file's problems, when they're for
    /// the document as the editor has it now.
    fn held_now(&self, path: &Path) -> Vec<lsp_types::Diagnostic> {
        match self.held.get(path) {
            Some((held, version)) if *version == self.docs.get(path).map(|d| d.version) => {
                held.clone()
            }
            _ => Vec::new(),
        }
    }

    /// What a lens request for a file that isn't a page works from: the
    /// project's context, for a code file in one of its sources, which may
    /// be outside the project's folder (`shown.rs`), and the file's path.
    pub(crate) fn lens_target(&self, uri: &Uri) -> Option<(Ctx, PathBuf)> {
        let path = Core::doc_path(uri)?;
        let loaded = self.loaded.as_ref()?;
        if !crate::shown::in_a_source(&loaded.root, &loaded.model, &path) {
            return None;
        }
        let content = RelPath::parse(Ctx::PROJECT).ok()?;
        let ctx = self.ctx(loaded, loaded.inc.snapshot(), content, &path)?;
        Some((ctx, path))
    }

    /// The diagnostics last published for a file without Vale's, and the
    /// version of the document they're for, when it's open.
    pub(crate) fn checked_with_version(
        &self,
        path: &Path,
    ) -> (Vec<lsp_types::Diagnostic>, Option<i32>) {
        (
            self.checked(path).to_vec(),
            self.published.get(path).and_then(|p| p.version),
        )
    }

    /// What `ascribe/projectEdit` works from, asked through any of the
    /// project's files: the project's context, as [`Core::project_target`]
    /// gives it, and each source file of its snapshot, in path order, with
    /// its open document's URI and version and the diagnostics last
    /// published for it.
    pub(crate) fn project_edit_target(
        &self,
        uri: &Uri,
    ) -> Option<(Ctx, Vec<crate::project_edit::Source>)> {
        let ctx = self.project_target(uri)?;
        let loaded = self.loaded.as_ref()?;
        let sources = ctx
            .snapshot
            .files()
            .filter_map(|file| {
                let path = loaded.source_path(&file.path);
                let doc = self.docs.get(&path);
                let uri = match doc {
                    Some(doc) => doc.uri.clone(),
                    None => path_to_uri(&path)?,
                };
                Some(crate::project_edit::Source {
                    path: file.path.clone(),
                    uri,
                    version: doc.map(|d| d.version),
                    published: self.checked_with_version(&path),
                })
            })
            .collect();
        Some((ctx, sources))
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
