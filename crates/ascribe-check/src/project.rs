//! The project the checks run on: the content model and the source files.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use ascribe_core::{AttributeValue, Date, FileId, Issue, LineIndex, RelPath, Span};
use ascribe_model::{
    AddressFile, ContentModel, Lock, LockedSource, NavFile, PermalinkList, SourceBound,
};
use ascribe_resolve::openapi::ApiSpecs;
use ascribe_resolve::side_file::{self, SideFile};
use ascribe_resolve::{CodeFile, CodeFiles, DiskFs, FileSystem, Layout, MemoFile, SourceSet};
use ascribe_syntax::{Block, BlockKind, DirectiveLine, ParseOptions, parse};
use ascribe_workspace::{
    LoadError, Loader, ModelFile, SharedFs, SourceFile, Workspace, WorkspaceSnapshot,
};

use crate::Diagnostic;
use crate::baseline::{BASELINE_FILE_ID, Baseline, BaselineError};
use crate::checks::template::Template;
use crate::side_files::{SIDE_FILES, side_file_by_id};

pub use ascribe_model::MODEL_FILE;

/// The id of `ascribe.lock` (SPEC §7.4), for locations in it.
pub const LOCK_FILE_ID: FileId = side_file::Lock::ID;

/// The id of `ascribe.permalinks` (SPEC §7.5), for locations in it.
pub const PERMALINKS_FILE_ID: FileId = side_file::Permalinks::ID;

/// The id of `ascribe.nav` (SPEC §7.8), for locations in it.
pub const NAV_FILE_ID: FileId = side_file::Nav::ID;

/// The id of `ascribe.addresses` (SPEC §7.10), for locations in it.
pub const ADDRESSES_FILE_ID: FileId = side_file::Addresses::ID;

/// The id of the first image file under the content root, for locations
/// in image files: the image checks list them in path order, and number them
/// from here ([`Project::images`]).
pub const IMAGE_FILE_IDS: u32 = 0x4000_0000;

/// The id of the first memo file (SPEC §7.7), for locations in memo files:
/// [`Project::checked_memos`] lists them, and numbers them from here.
pub const MEMO_FILE_IDS: u32 = 0x2000_0000;

/// What a file id names, for reporting.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileEntry<'a> {
    /// The id.
    pub id: FileId,
    /// The path relative to the project root, `/`-separated: how a person
    /// would type it. `ascribe.toml` for the content model.
    pub display_path: String,
    /// The text.
    pub text: &'a str,
    /// The path relative to the content root, for a source file or an image
    /// file under it; `None` for the content model and the side files.
    pub content_path: Option<&'a RelPath>,
}

/// What loading a project for the checks takes of this crate: the content
/// model's loader, which checks the files its types name
/// ([`type_files::load_model`](crate::type_files::load_model)), and the side
/// files ([`SIDE_FILES`]). Every surface loads a [`Workspace`] with it, so
/// each has the same side files.
pub const LOADER: Loader = Loader {
    model: crate::type_files::load_model,
    side_files: SIDE_FILES,
};

/// Loads the project whose content model is at `config` with [`LOADER`]:
/// the model, then every source file under its content root, as
/// [`ascribe_resolve::FileSystem::sources`] finds them, indexed. A command
/// checks [`Project::new`] of its snapshot.
///
/// A source file or directory that can't be read, or a source file that
/// isn't valid UTF-8, is still a source file, with its
/// [`ReadFailure`](ascribe_workspace::ReadFailure); `check_files` reports it
/// and checks the rest.
///
/// # Errors
///
/// As [`Workspace::load`].
pub fn load(config: &Path) -> Result<Workspace, LoadError> {
    Workspace::load(&LOADER, config)
}

/// [`load`], with a local source bounded by `bound` when no git repository
/// is around the project ([`SourceBound`]).
///
/// # Errors
///
/// As [`load`].
pub fn load_bounded(config: &Path, bound: SourceBound<'_>) -> Result<Workspace, LoadError> {
    Workspace::load_bounded(&LOADER, config, bound)
}

/// A project built in memory with [`LOADER`]'s side files: its source files
/// are `files`, `(content path, text)` pairs, numbered from 1 in path order,
/// and everything else, the side files among it, is read from the disk under
/// `root`. A test's project, or one whose pages are given.
pub fn in_memory(
    root: PathBuf,
    content_root: RelPath,
    model: ContentModel,
    model_text: String,
    files: impl IntoIterator<Item = (RelPath, String)>,
) -> Workspace {
    // The disk doesn't change while a project built in memory is checked,
    // so listings are kept.
    let layout = Layout {
        content_root: content_root.clone(),
        output_dir: RelPath::parse(&model.project.output_dir).unwrap_or_default(),
    };
    let fs = Arc::new(DiskFs::with_listing_cache(&root, &layout));
    in_memory_with(root, content_root, model, model_text, files, fs)
}

/// [`in_memory`], with everything that isn't one of `files` read through
/// `fs`: `ascribe explain`'s examples, or the language server's files.
pub fn in_memory_with(
    root: PathBuf,
    content_root: RelPath,
    model: ContentModel,
    model_text: String,
    files: impl IntoIterator<Item = (RelPath, String)>,
    fs: SharedFs,
) -> Workspace {
    let model = ModelFile {
        root,
        text: model_text,
        model,
    };
    Workspace::from_texts(LOADER.side_files, model, content_root, files, fs)
}

/// A documentation set as the checks see it: a [`WorkspaceSnapshot`], and
/// the checks' own settings, the day they run on and whether personal memos
/// count.
///
/// The content model is file id 0, and the source files have the ids of the
/// snapshot's source index: 1, 2, … in path order when a project is loaded,
/// or the ids an `ascribe_resolve::Snapshot` gives in the language server,
/// which are stable per path across updates and have gaps where files were
/// deleted (`ascribe_resolve::incremental`). It finds a file by id, not by
/// position. Files that aren't sources (images, other downloads) are looked
/// for through the snapshot's [`FileSystem`] when a reference needs them.
///
/// Which files are sources, and every rule about what a reference names, are
/// `ascribe_resolve`'s, so this crate and the source index can't disagree.
#[derive(Clone, Debug)]
pub struct Project {
    snapshot: WorkspaceSnapshot,
    model_warnings: Vec<Diagnostic>,
    /// The source index keeping a base revision's specs
    /// ([`Project::with_base_apis`]), in place of the snapshot's.
    based_index: Option<Arc<ascribe_resolve::Project>>,
    /// The day the checks run on, which `recheck-overdue` compares with.
    today: Option<Date>,
    /// The templates the content types name, by type, read the first time a
    /// page of one is checked.
    templates: Arc<OnceLock<HashMap<String, Arc<Template>>>>,
    /// Whether the checks read the personal memos too: the editor's do, and
    /// `ascribe check`'s don't ([`Project::with_personal_memos`]).
    personal_memos: bool,
    /// The values the pages' `@variant` arms give each dimension, read the
    /// first time a check asks.
    variant_values: OnceLock<BTreeMap<String, Vec<String>>>,
}

impl Project {
    /// The checks' view of `snapshot`, with no day and only the committed
    /// memos.
    pub fn new(snapshot: WorkspaceSnapshot) -> Project {
        let model_warnings = snapshot
            .model()
            .warnings
            .iter()
            .map(Diagnostic::from_issue)
            .collect();
        Project {
            snapshot,
            model_warnings,
            based_index: None,
            today: None,
            templates: Arc::default(),
            personal_memos: false,
            variant_values: OnceLock::new(),
        }
    }

    /// The snapshot the checks read.
    pub fn snapshot(&self) -> &WorkspaceSnapshot {
        &self.snapshot
    }

    /// The project root: the directory containing `ascribe.toml`.
    pub fn root(&self) -> &Path {
        self.snapshot.root()
    }

    /// The content root, relative to the project root.
    pub fn content_root(&self) -> &RelPath {
        self.snapshot.content_root()
    }

    /// Where the content root and the output directory are.
    pub fn layout(&self) -> &Layout {
        self.snapshot.layout()
    }

    /// The source index: what the page-level checks read, a broken link's
    /// closest targets are looked for in, and a command that resolves builds
    /// (`build`, `diff`, `drift`, `sources`) works on. It's the snapshot's,
    /// or, with a base revision's specs ([`Project::with_base_apis`]), a
    /// copy of it keeping them. Images and other files are read through
    /// [`Project::file_system`].
    pub fn index(&self) -> &Arc<ascribe_resolve::Project> {
        self.based_index
            .as_ref()
            .unwrap_or_else(|| self.snapshot.index())
    }

    /// The content model's text.
    pub fn model_text(&self) -> &str {
        self.snapshot.model_text()
    }

    /// The files the checks probe for what isn't a source file: the file
    /// system the snapshot was read through.
    pub fn file_system(&self) -> &dyn FileSystem {
        self.snapshot.file_system()
    }

    /// The code files the checks have read for snippets (SPEC §4.8), so far.
    /// A diagnostic's related location can be in one: its id is the code
    /// file's, from [`ascribe_resolve::snippet::CODE_FILE_IDS`] up.
    pub fn code_files(&self) -> &CodeFiles {
        self.snapshot.code_files()
    }

    /// The code file with this id, if the checks have read it.
    pub fn code_file(&self, id: FileId) -> Option<Arc<CodeFile>> {
        self.snapshot.code_files().by_id(id)
    }

    /// The content model.
    pub fn model(&self) -> &ContentModel {
        self.snapshot.model()
    }

    /// The source files, in path order.
    pub fn sources(&self) -> &[SourceFile] {
        self.snapshot.sources()
    }

    /// The diagnostics for the content model's warnings.
    pub fn model_warnings(&self) -> &[Diagnostic] {
        &self.model_warnings
    }

    /// A source file by content path.
    pub fn source_at(&self, path: &RelPath) -> Option<&SourceFile> {
        self.snapshot.source_at(path)
    }

    /// The side file `F` read, when the project has it ([`SIDE_FILES`]).
    pub fn side_file<F: SideFile>(&self) -> Option<&F::Read> {
        self.snapshot.side_file::<F>()
    }

    /// The text of the side file at `path`, from the project root, when the
    /// project has it.
    pub fn side_text(&self, path: &str) -> Option<&str> {
        self.snapshot.side_text(path)
    }

    /// The text of `ascribe.lock`, when the project has one.
    pub fn lock_text(&self) -> Option<&str> {
        self.side_text(side_file::Lock::PATH)
    }

    /// `ascribe.lock` read, or its problems, when the project has one.
    pub(crate) fn lock(&self) -> Option<&Result<Lock, Vec<Issue>>> {
        self.side_file::<side_file::Lock>()
    }

    /// The text of `ascribe.permalinks`, when the project has one.
    pub fn permalinks_text(&self) -> Option<&str> {
        self.side_text(side_file::Permalinks::PATH)
    }

    /// `ascribe.permalinks` read, or its problems, when the project has one.
    pub fn permalinks(&self) -> Option<&Result<PermalinkList, Vec<Issue>>> {
        self.side_file::<side_file::Permalinks>()
    }

    /// `ascribe.nav` read, or its problems, when the project has one.
    pub fn nav(&self) -> Option<&Result<NavFile, Vec<Issue>>> {
        self.side_file::<side_file::Nav>()
    }

    /// The text of `ascribe.addresses`, when the project has one.
    pub fn addresses_text(&self) -> Option<&str> {
        self.side_text(side_file::Addresses::PATH)
    }

    /// `ascribe.addresses` read, or its problems, when the project has one.
    pub fn addresses(&self) -> Option<&Result<AddressFile, Vec<Issue>>> {
        self.side_file::<side_file::Addresses>()
    }

    /// The text of `ascribe.baseline`, when the project has one.
    pub fn baseline_text(&self) -> Option<&str> {
        self.side_text(crate::side_files::Baseline::PATH)
    }

    /// `ascribe.baseline` read, or the line that doesn't parse, when the
    /// project has one.
    pub fn baseline(&self) -> Option<&Result<Baseline, BaselineError>> {
        self.side_file::<crate::side_files::Baseline>()
    }

    /// The pin of the source `name` in another repository, when
    /// `ascribe.lock` can be read and pins it to the repository the content
    /// model names.
    pub(crate) fn pin(&self, name: &str) -> Option<&LockedSource> {
        let url = &self.model().source(name)?.git.as_ref()?.url;
        // A lock that can't be read is reported by the sources check.
        let lock = self.lock()?.as_ref().ok()?;
        lock.source(name).filter(|l| &l.git == url)
    }

    /// The image files under the content root, by path from the project
    /// root and from the content root, in path order, listed by `list` the
    /// first time they're asked for. The image at position `i` has the id
    /// `IMAGE_FILE_IDS + i`.
    pub(crate) fn images_or(
        &self,
        list: impl FnOnce() -> Vec<(RelPath, RelPath)>,
    ) -> &[(RelPath, RelPath)] {
        self.snapshot.images_or(list)
    }

    /// The ids of the image files the image checks listed; none when they
    /// haven't run.
    pub fn image_ids(&self) -> impl Iterator<Item = FileId> + '_ {
        (0..self.snapshot.images().map_or(0, <[_]>::len)).map(image_id)
    }

    /// What a file id names.
    pub fn file(&self, id: FileId) -> Option<FileEntry<'_>> {
        // A side file's id can be in the images' range, so it's looked for
        // first.
        if let Some(entry) = side_file_by_id(id) {
            return self.side_text(entry.path).map(|text| FileEntry {
                id,
                display_path: entry.path.to_owned(),
                text,
                content_path: None,
            });
        }
        if let Some(i) = id.index().checked_sub(MEMO_FILE_IDS)
            && id.index() < IMAGE_FILE_IDS
        {
            let memo = self.checked_memos().get(i as usize)?;
            return Some(FileEntry {
                id,
                display_path: memo.path.to_string(),
                text: &memo.text,
                content_path: None,
            });
        }
        if let Some(i) = id.index().checked_sub(IMAGE_FILE_IDS)
            && id.index() < BASELINE_FILE_ID.index()
        {
            let (path, content) = self.snapshot.images()?.get(i as usize)?;
            return Some(FileEntry {
                id,
                display_path: path.to_string(),
                text: "",
                content_path: Some(content),
            });
        }
        if id == FileId::new(0) {
            return Some(FileEntry {
                id,
                display_path: MODEL_FILE.to_owned(),
                text: self.model_text(),
                content_path: None,
            });
        }
        let source = self.sources().iter().find(|s| s.id == id)?;
        Some(FileEntry {
            id,
            display_path: self
                .content_root()
                .join(source.path.as_str())
                .map_or_else(|_| source.path.to_string(), |p| p.to_string()),
            text: &source.text,
            content_path: Some(&source.path),
        })
    }

    /// The path a report shows for a file id: [`FileEntry::display_path`] for
    /// the content model, a side file, and a source file, and the code file's
    /// path for a code file a snippet read.
    pub fn display_path(&self, id: FileId) -> Option<String> {
        match self.file(id) {
            Some(entry) => Some(entry.display_path),
            None => self.code_file(id).map(|code| code.path.to_string()),
        }
    }

    /// This project with the source file at `path` (a content path) holding
    /// `text` instead of what was read, or added when there's none: a file
    /// checked before it's saved. The ids are numbered again, in path order
    /// ([`WorkspaceSnapshot::with_source`]).
    pub fn with_source(&self, path: &RelPath, text: String) -> Project {
        self.with_snapshot(self.snapshot.with_source(path, text))
    }

    /// The checks' view of `snapshot`, another of the same project's, with
    /// this one's day and the templates it has read, since a page's text
    /// doesn't change them.
    pub fn with_snapshot(&self, snapshot: WorkspaceSnapshot) -> Project {
        Project {
            templates: self.templates.clone(),
            ..Project::new(snapshot).with_today(self.today)
        }
    }

    /// The committed memo files (SPEC §7.7), in path order, read through the
    /// project's [`FileSystem`] the first time they're asked for. None when
    /// `memos/` is inside the content root, where its files are pages.
    pub fn memos(&self) -> &[MemoFile] {
        self.snapshot.memos()
    }

    /// The committed memo files, then the personal ones in `.ascribe/memos/`,
    /// each in path order: what `ascribe outline` and `ascribe refs` list.
    pub fn all_memos(&self) -> &[MemoFile] {
        self.snapshot.all_memos()
    }

    /// This project, with the checks reading the personal memos as well as
    /// the committed ones: the editor's, which reports a personal memo's
    /// problems as it reports a team memo's.
    pub fn with_personal_memos(self) -> Project {
        Project {
            personal_memos: true,
            ..self
        }
    }

    /// The memo files the checks read: the committed ones, and the personal
    /// ones after them when [`Project::with_personal_memos`] says so. The
    /// memo at position `i` has the id `MEMO_FILE_IDS + i`.
    pub fn checked_memos(&self) -> &[MemoFile] {
        if self.personal_memos {
            self.all_memos()
        } else {
            self.memos()
        }
    }

    /// The ids of the memo files the checks read.
    pub fn memo_ids(&self) -> impl Iterator<Item = FileId> + '_ {
        (0..self.checked_memos().len()).map(memo_id)
    }

    /// The template content type `name` names, read through the project's
    /// [`FileSystem`]. None when it names none, or when the file can't be
    /// read, which is the content model's error (`model-type-template`).
    pub(crate) fn template(&self, name: &str) -> Option<Arc<Template>> {
        let templates = self.templates.get_or_init(|| {
            let mut out = HashMap::new();
            for t in &self.model().types {
                let Some(path) = t
                    .template
                    .as_ref()
                    .and_then(|f| RelPath::parse(&f.path).ok())
                else {
                    continue;
                };
                let Some(text) = self
                    .file_system()
                    .read_file(&path)
                    .ok()
                    .and_then(|bytes| String::from_utf8(bytes).ok())
                else {
                    continue;
                };
                out.insert(
                    t.name.clone(),
                    Arc::new(Template::read(&text, self.model())),
                );
            }
            out
        });
        templates.get(name).cloned()
    }

    /// Where `text`, a page of type `type_name`, still has what the type's
    /// template left to fill in, in source order: each frontmatter field
    /// whose value is the template's, and each placeholder of the body,
    /// whole, from the first piece of one that runs across blocks to its
    /// last. They're found as `template-placeholder-left` finds them, whether
    /// or not the check is on. Empty when the type has no template, or its
    /// template can't be read.
    pub fn template_places(&self, type_name: &str, text: &str) -> Vec<Span> {
        self.template(type_name)
            .map(|template| crate::checks::template::places(&template, text, self.model()))
            .unwrap_or_default()
    }

    /// This project, checked as on `today`: the day `recheck-overdue`
    /// compares a page's recheck date with. A library doesn't read the clock,
    /// so a project has no day until its caller gives it one, and without
    /// one, no recheck is overdue.
    #[must_use]
    pub fn with_today(mut self, today: Option<Date>) -> Project {
        self.today = today;
        self
    }

    /// The project with a base revision's specs, so a link to an
    /// `operationId` renamed since is offered the operation now at the
    /// method and path it had there, first and as a safe fix
    /// ([`ascribe_resolve::openapi::ApiSpecs::renamed`]). Its source index
    /// is a copy of the snapshot's, keeping them.
    #[must_use]
    pub fn with_base_apis(self, base: Arc<ApiSpecs>) -> Project {
        let mut index = ascribe_resolve::Project::clone(self.snapshot.index());
        index.remember_base_apis(&base);
        Project {
            based_index: Some(Arc::new(index)),
            ..self
        }
    }

    /// The values the pages' `@variant` arms give the dimension `name`, in
    /// the order they're first given: what a `[dimensions.<name>]` table
    /// for them would list. Read from every source file the first time any
    /// dimension is asked for, which only a project that doesn't declare
    /// the dimension does.
    pub(crate) fn variant_values(&self, name: &str) -> &[String] {
        let all = self.variant_values.get_or_init(|| {
            let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
            // A file without the word has no arm to read, and a mention of
            // it is cheap to look for.
            for file in self
                .sources()
                .iter()
                .filter(|f| f.text.contains("@variant"))
            {
                let options = ParseOptions::new(self.model().directive_schemas());
                variant_values_in(&parse(&file.text, &options).blocks, &mut out);
            }
            out
        });
        all.get(name).map_or(&[], Vec::as_slice)
    }

    /// The day the checks run on, if the caller gave one.
    pub fn today(&self) -> Option<Date> {
        self.today
    }

    /// A line index for a file's text, for turning spans into lines and columns.
    pub fn line_index(&self, id: FileId) -> Option<LineIndex> {
        self.file(id).map(|f| LineIndex::new(f.text))
    }
}

impl SourceSet for Project {
    fn contains(&self, path: &RelPath) -> bool {
        self.snapshot.contains(path)
    }

    fn case_twin(&self, path: &RelPath) -> Option<RelPath> {
        self.snapshot.case_twin(path)
    }

    fn pages(&self) -> Vec<RelPath> {
        self.snapshot.pages()
    }
}

/// Adds the values each `@variant` arm in `blocks` gives its dimensions to
/// `out`, each once.
fn variant_values_in(blocks: &[Block], out: &mut BTreeMap<String, Vec<String>>) {
    for block in blocks {
        match &block.kind {
            BlockKind::Group(group) => {
                for arm in &group.arms {
                    opener_values(&arm.opener, out);
                    variant_values_in(&arm.children, out);
                }
            }
            BlockKind::Container(container) => {
                opener_values(&container.opener, out);
                variant_values_in(&container.children, out);
            }
            BlockKind::Directive(line) => opener_values(line, out),
            BlockKind::BlockQuote(quote) => variant_values_in(&quote.children, out),
            BlockKind::List(list) => {
                for item in &list.items {
                    variant_values_in(&item.children, out);
                }
            }
            _ => {}
        }
    }
}

/// Adds the values a `@variant` opener's attributes give to `out`.
fn opener_values(line: &DirectiveLine, out: &mut BTreeMap<String, Vec<String>>) {
    let Some(block) = line.attributes.as_ref().filter(|_| line.name == "variant") else {
        return;
    };
    for attribute in &block.attributes {
        let values: Vec<String> = match &attribute.value {
            Some(AttributeValue::Set { members, .. }) => {
                members.iter().map(|m| m.text.clone()).collect()
            }
            Some(other) => other.as_text().map(str::to_owned).into_iter().collect(),
            None => Vec::new(),
        };
        let known = out.entry(attribute.key.clone()).or_default();
        for value in values {
            if !known.contains(&value) {
                known.push(value);
            }
        }
    }
}

/// The id of the memo file at position `i` of [`Project::checked_memos`].
pub(crate) fn memo_id(i: usize) -> FileId {
    // A project has far fewer memo files than the ids from MEMO_FILE_IDS to
    // the images'.
    #[allow(clippy::cast_possible_truncation)]
    FileId::new(MEMO_FILE_IDS + i as u32)
}

/// The id of the image at position `i` of [`Project::images`].
pub(crate) fn image_id(i: usize) -> FileId {
    // A project has far fewer images than the ids from IMAGE_FILE_IDS to the
    // navigation file's.
    #[allow(clippy::cast_possible_truncation)]
    FileId::new(IMAGE_FILE_IDS + i as u32)
}
