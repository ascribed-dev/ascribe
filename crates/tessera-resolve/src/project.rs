//! The project: every source file's index, what the references in them name,
//! the edges between files, and the problems found on the way.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::{Arc, Mutex};
use tessera_core::{FileId, Issue, Location, RelPath, Slugger, Span, diagnostics};
use tessera_model::ContentModel;

use crate::expand::{ExpandedPage, Expander, IncludeSite};
use crate::fs::FileSystem;
use crate::incremental::FileIds;
use crate::index::{FileIndex, FileKind, Heading, Include, RefKind, Reference, index_file};
use crate::layout::Layout;
use crate::references::{SourceSet, include_issue, reference_issue, resolve_reference};
use crate::slug::{default_slugger, slugger_by_name};

/// What a link or image names, once the project's files are known.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Resolution {
    /// A URL with a scheme. Nothing to check or copy.
    External,
    /// A source file of the project: a page or a fragment.
    Source {
        /// The file's content path.
        target: RelPath,
        /// The heading source id after `#`, if any.
        id: Option<String>,
        /// Whether the file is a fragment, which a link can't target.
        fragment: bool,
    },
    /// A `.md` file inside the content root that isn't in the project.
    SourceMissing {
        /// The path a file with different case has, if one does.
        actual: Option<RelPath>,
    },
    /// A local file a build copies: an image source, or a link target that
    /// isn't a source file. It exists, with exactly this name, and is inside
    /// the boundary.
    Asset {
        /// Its content path, starting with `..` when it's outside the content
        /// root.
        path: RelPath,
        /// The part after `#`, kept after the rewritten reference.
        fragment: Option<String>,
    },
    /// A local file that doesn't exist, or that a build may not copy.
    AssetMissing(Missing),
    /// A link that looks like a published route rather than a file path, and
    /// names no file (SPEC §5.2).
    Route {
        /// The page it probably means, as a content path.
        page: String,
        /// The link to write instead, as a file path.
        suggestion: String,
        /// Whether that page is a source file of the project, so the
        /// suggestion can be offered as a fix.
        page_exists: bool,
    },
}

/// Why a local file isn't found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Missing {
    /// No such file.
    Absent,
    /// A file exists whose name differs only in case. Names match exactly on
    /// every platform (SPEC §9.4). Holds the real path, as a content path.
    Case(RelPath),
    /// The file is outside the project, or inside the output directory, so it
    /// is treated as not existing (SPEC §9.4).
    Outside,
}

/// A source file, or a directory, the project couldn't read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unreadable {
    /// The file's content path.
    pub path: RelPath,
    /// The operating system's reason.
    pub reason: String,
}

/// A file that includes another.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IncludeEdge {
    /// The including file.
    pub file: RelPath,
    /// The `@include` directive there.
    pub span: Span,
    /// The section it includes, by source id; `None` for the whole file.
    pub section: Option<String>,
}

/// A file that links to another.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkSite {
    /// The linking file.
    pub file: RelPath,
    /// The link.
    pub span: Span,
    /// The source id it names in the target, if any.
    pub id: Option<String>,
}

/// A file that uses an asset.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetSite {
    /// The file the reference is written in.
    pub file: RelPath,
    /// The link or image.
    pub span: Span,
    /// Whether it's an image or a link.
    pub kind: RefKind,
}

/// The edges between files, in the reverse direction: who points here.
#[derive(Clone, Debug, Default)]
pub(crate) struct ReverseEdges {
    includes: BTreeMap<RelPath, Vec<IncludeEdge>>,
    links: BTreeMap<RelPath, Vec<LinkSite>>,
    assets: BTreeMap<RelPath, Vec<AssetSite>>,
}

impl ReverseEdges {
    /// Records what one file contributes: its includes whose target is in
    /// `files`, the links that name a source file, and its asset references.
    /// Each list stays sorted by the linking file and the span, so the result
    /// doesn't depend on the order files were added in.
    fn add_file(
        &mut self,
        files: &BTreeMap<RelPath, Arc<FileIndex>>,
        index: &FileIndex,
        resolutions: &[Resolution],
    ) {
        for include in &index.includes {
            let Some(target) = &include.target else {
                continue;
            };
            if files.contains_key(target) {
                insert_sorted(
                    self.includes.entry(target.clone()).or_default(),
                    IncludeEdge {
                        file: index.path.clone(),
                        span: include.span,
                        section: include.section.clone(),
                    },
                    |e| (e.file.clone(), e.span),
                );
            }
        }
        for (reference, resolution) in index.references.iter().zip(resolutions) {
            match resolution {
                Resolution::Source { target, id, .. } => insert_sorted(
                    self.links.entry(target.clone()).or_default(),
                    LinkSite {
                        file: index.path.clone(),
                        span: reference.span,
                        id: id.clone(),
                    },
                    |e| (e.file.clone(), e.span),
                ),
                Resolution::Asset { path, .. } => insert_sorted(
                    self.assets.entry(path.clone()).or_default(),
                    AssetSite {
                        file: index.path.clone(),
                        span: reference.span,
                        kind: reference.kind,
                    },
                    |e| (e.file.clone(), e.span),
                ),
                _ => {}
            }
        }
    }

    /// Forgets everything a file contributed (given what it was, before).
    fn remove_file(&mut self, index: &FileIndex, resolutions: &[Resolution]) {
        let path = &index.path;
        for include in &index.includes {
            if let Some(target) = &include.target {
                retain_edges(&mut self.includes, target, |e| &e.file != path);
            }
        }
        for resolution in resolutions {
            match resolution {
                Resolution::Source { target, .. } => {
                    retain_edges(&mut self.links, target, |e| &e.file != path);
                }
                Resolution::Asset { path: asset, .. } => {
                    retain_edges(&mut self.assets, asset, |e| &e.file != path);
                }
                _ => {}
            }
        }
    }
}

fn insert_sorted<T>(list: &mut Vec<T>, item: T, key: impl Fn(&T) -> (RelPath, Span)) {
    let at = key(&item);
    let position = list.partition_point(|e| key(e) <= at);
    list.insert(position, item);
}

fn retain_edges<T>(map: &mut BTreeMap<RelPath, Vec<T>>, key: &RelPath, keep: impl Fn(&T) -> bool) {
    if let Some(list) = map.get_mut(key) {
        list.retain(keep);
        if list.is_empty() {
            map.remove(key);
        }
    }
}

/// An asset a page uses, with where the reference is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageAsset {
    /// The asset's source path: its content path, starting with `..` when
    /// it's outside the content root.
    pub path: RelPath,
    /// Whether it's an image source or a link target.
    pub kind: RefKind,
    /// The part after `#`, kept after the rewritten reference.
    pub fragment: Option<String>,
    /// The file the reference is written in: the fragment, for included
    /// content. The reference resolves from here.
    pub written_in: RelPath,
    /// The reference there.
    pub location: Location,
    /// The includes it came through, outermost first (in the page's own
    /// file); empty for the page's own content.
    pub via: Arc<[IncludeSite]>,
}

/// The folders below the content root that [`FileSystem::sources`] found
/// holding an `ascribe.toml`: other projects', and the project's own.
pub(crate) struct Folders {
    nested: Vec<RelPath>,
    own: Option<RelPath>,
}

/// The source index of a project: independent of any build.
///
/// It holds one [`FileIndex`] per source file, what every link and image
/// names, the edges between files in both directions, and the includes'
/// expansion on demand ([`Project::expand`]).
#[derive(Debug)]
pub struct Project {
    pub(crate) model: Arc<ContentModel>,
    pub(crate) layout: Layout,
    pub(crate) files: BTreeMap<RelPath, Arc<FileIndex>>,
    by_id: HashMap<FileId, RelPath>,
    resolutions: BTreeMap<RelPath, Arc<Vec<Resolution>>>,
    edges: ReverseEdges,
    unreadable: Vec<Unreadable>,
    nested: Vec<RelPath>,
    own_folder: Option<RelPath>,
    /// Expansions worked out so far, by page. `None` records a path that
    /// isn't a file. Filled on demand; the incremental update drops the
    /// entries a change can affect (`crate::incremental`).
    expansions: Mutex<HashMap<RelPath, Option<Arc<ExpandedPage>>>>,
}

impl Clone for Project {
    fn clone(&self) -> Project {
        Project {
            model: self.model.clone(),
            layout: self.layout.clone(),
            files: self.files.clone(),
            by_id: self.by_id.clone(),
            resolutions: self.resolutions.clone(),
            edges: self.edges.clone(),
            unreadable: self.unreadable.clone(),
            nested: self.nested.clone(),
            own_folder: self.own_folder.clone(),
            expansions: Mutex::new(self.expansion_cache().clone()),
        }
    }
}

impl Project {
    /// Indexes every `.md` file under the content root.
    ///
    /// Each file gets a [`FileId`], in path order, from 1 (id 0 is
    /// `ascribe.toml`). A file that can't be read is left out and listed in
    /// [`Project::unreadable`].
    pub fn load(model: Arc<ContentModel>, layout: Layout, fs: &dyn FileSystem) -> Project {
        Project::load_with_ids(model, layout, fs, &mut FileIds::default())
    }

    /// Like [`Project::load`], with ids from `ids`: a file whose path the
    /// table knows keeps its id, and any other gets the next free one, in path
    /// order. The table is updated. An [`IncrementalProject`](crate::IncrementalProject)
    /// numbers files this way, so a from-scratch load with its table is what
    /// its updates must equal.
    pub fn load_with_ids(
        model: Arc<ContentModel>,
        layout: Layout,
        fs: &dyn FileSystem,
        ids: &mut FileIds,
    ) -> Project {
        let slugger: Box<dyn Slugger> =
            slugger_by_name(&model.consumer.slugger).unwrap_or_else(default_slugger);
        let found = fs.sources();
        let mut paths = found.paths;
        paths.sort();
        paths.dedup();

        let mut files = BTreeMap::new();
        let mut by_id = HashMap::new();
        let mut unreadable = found.unreadable;
        let mut nested = found.nested;
        nested.sort();
        nested.dedup();
        for path in paths {
            match fs.read(&path) {
                Ok(text) => {
                    let id = ids.assign(&path);
                    let index = index_file(id, &path, &text, &model, slugger.as_ref());
                    by_id.insert(id, path.clone());
                    files.insert(path, Arc::new(index));
                }
                Err(err) => unreadable.push(Unreadable {
                    path,
                    reason: err.to_string(),
                }),
            }
        }
        let folders = Folders {
            nested,
            own: found.own_folder,
        };
        Project::assemble(model, layout, files, by_id, unreadable, folders, fs)
    }

    /// A project from indexed files: resolves every reference and builds the
    /// edges.
    pub(crate) fn assemble(
        model: Arc<ContentModel>,
        layout: Layout,
        files: BTreeMap<RelPath, Arc<FileIndex>>,
        by_id: HashMap<FileId, RelPath>,
        unreadable: Vec<Unreadable>,
        folders: Folders,
        fs: &dyn FileSystem,
    ) -> Project {
        let mut project = Project {
            model,
            layout,
            files,
            by_id,
            resolutions: BTreeMap::new(),
            edges: ReverseEdges::default(),
            unreadable,
            nested: folders.nested,
            own_folder: folders.own,
            expansions: Mutex::new(HashMap::new()),
        };
        project.resolutions = project
            .files
            .values()
            .map(|index| {
                (
                    index.path.clone(),
                    Arc::new(project.resolve_file(index, fs)),
                )
            })
            .collect();
        project.edges = project.build_edges();
        project
    }

    /// The content model the project was indexed with.
    pub fn model(&self) -> &ContentModel {
        &self.model
    }

    /// Where the content root and the output directory are.
    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    /// Every source file's index, in path order.
    pub fn files(&self) -> impl Iterator<Item = &FileIndex> {
        self.files.values().map(|f| &**f)
    }

    /// The pages: every source file that isn't a fragment, in path order.
    pub fn pages(&self) -> impl Iterator<Item = &FileIndex> {
        self.files().filter(|f| f.kind == FileKind::Page)
    }

    /// The fragments, in path order.
    pub fn fragments(&self) -> impl Iterator<Item = &FileIndex> {
        self.files().filter(|f| f.kind == FileKind::Fragment)
    }

    /// One file's index, by content path.
    pub fn file(&self, path: &RelPath) -> Option<&FileIndex> {
        self.files.get(path).map(|f| &**f)
    }

    /// The path of a file, by id.
    pub fn path_of(&self, file: FileId) -> Option<&RelPath> {
        self.by_id.get(&file)
    }

    /// The index of a file, by id.
    pub fn file_by_id(&self, file: FileId) -> Option<&FileIndex> {
        self.path_of(file)
            .and_then(|p| self.files.get(p))
            .map(|f| &**f)
    }

    /// Source files and directories that couldn't be read. They're never
    /// skipped silently.
    pub fn unreadable(&self) -> &[Unreadable] {
        &self.unreadable
    }

    /// The folders of other projects nested in the content root: the
    /// directories below it that hold an `ascribe.toml`, as content paths, in
    /// path order ([`Sources::nested`](crate::Sources::nested)). Nothing in
    /// them is a source of this project.
    pub fn nested_projects(&self) -> &[RelPath] {
        &self.nested
    }

    /// The project's own folder as a content path, when it's below the content
    /// root ([`Sources::own_folder`](crate::Sources::own_folder)): its
    /// `ascribe.toml` is the project's own, not a nested project's.
    pub fn own_folder(&self) -> Option<&RelPath> {
        self.own_folder.as_ref()
    }

    /// Whether a content path names a source file of this project, whether or
    /// not a file is there: [`is_source_path`](crate::is_source_path), and
    /// not inside a nested project's folder ([`Project::nested_projects`]).
    pub fn is_source(&self, path: &RelPath) -> bool {
        crate::is_source_path(path) && !crate::in_nested_project(path, &self.nested)
    }

    /// The title of a file (its frontmatter `title`), or, with an id, of the
    /// heading with that source id: the text a link with no text shows
    /// (SPEC §5.2).
    pub fn title_for(&self, path: &RelPath, id: Option<&str>) -> Option<&str> {
        let file = self.files.get(path)?;
        match id {
            Some(id) => file.heading_by_id(id).map(|h| h.text.as_str()),
            None => file.title.as_deref(),
        }
    }

    /// The heading with this source id in a file.
    pub fn heading(&self, path: &RelPath, id: &str) -> Option<&Heading> {
        self.files.get(path)?.heading_by_id(id)
    }

    /// What every link and image in a file names, in the order of
    /// [`FileIndex::references`].
    pub fn resolutions(&self, path: &RelPath) -> &[Resolution] {
        self.resolutions.get(path).map_or(&[], |r| r.as_slice())
    }

    /// The resolution of one reference of a file, by its span.
    pub fn resolution_at(&self, path: &RelPath, span: Span) -> Option<&Resolution> {
        let file = self.files.get(path)?;
        let at = file.references.iter().position(|r| r.span == span)?;
        self.resolutions.get(path)?.get(at)
    }

    /// The files that include this one, and where.
    pub fn includers(&self, target: &RelPath) -> &[IncludeEdge] {
        self.edges.includes.get(target).map_or(&[], Vec::as_slice)
    }

    /// The pages that include this file, directly or through other files
    /// that include it. An include of a section counts as including the file.
    pub fn including_pages(&self, target: &RelPath) -> Vec<RelPath> {
        let mut seen: BTreeSet<RelPath> = BTreeSet::new();
        let mut queue = vec![target.clone()];
        while let Some(next) = queue.pop() {
            for edge in self.includers(&next) {
                if seen.insert(edge.file.clone()) {
                    queue.push(edge.file.clone());
                }
            }
        }
        seen.into_iter()
            .filter(|p| self.files.get(p).is_some_and(|f| f.kind == FileKind::Page))
            .collect()
    }

    /// The links to a source file, from every file.
    pub fn links_to(&self, target: &RelPath) -> &[LinkSite] {
        self.edges.links.get(target).map_or(&[], Vec::as_slice)
    }

    /// The links that name this source id in a file.
    pub fn links_to_id<'a>(
        &'a self,
        target: &RelPath,
        id: &'a str,
    ) -> impl Iterator<Item = &'a LinkSite> {
        self.links_to(target)
            .iter()
            .filter(move |l| l.id.as_deref() == Some(id))
    }

    /// The files that use an asset, by its source path.
    pub fn asset_users(&self, asset: &RelPath) -> &[AssetSite] {
        self.edges.assets.get(asset).map_or(&[], Vec::as_slice)
    }

    /// Expands a file's includes. `None` if the file isn't in the project.
    ///
    /// The expansion is kept: asking again for the same file costs a copy,
    /// and an incremental update ([`crate::IncrementalProject`]) keeps the
    /// expansions of the pages a change can't reach. [`Project::expansion`]
    /// shares it instead of copying.
    pub fn expand(&self, path: &RelPath) -> Option<ExpandedPage> {
        self.expansion(path).map(|page| (*page).clone())
    }

    /// The expansion of a file, shared with the project's cache: the same
    /// value as [`Project::expand`] without copying it.
    pub fn expansion(&self, path: &RelPath) -> Option<Arc<ExpandedPage>> {
        if let Some(hit) = self.expansion_cache().get(path) {
            return hit.clone();
        }
        // Expanding doesn't hold the lock: it may ask for other expansions.
        let page = Expander::new(self).expand(path).map(Arc::new);
        self.expansion_cache().insert(path.clone(), page.clone());
        page
    }

    fn expansion_cache(
        &self,
    ) -> std::sync::MutexGuard<'_, HashMap<RelPath, Option<Arc<ExpandedPage>>>> {
        // A poisoned lock only means another thread panicked while holding
        // it; the map is still a valid cache.
        self.expansions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// The assets a page uses: every image, and every link to a file that
    /// isn't a source file, in the page after its includes are expanded,
    /// resolved from the file each reference is written in, in document
    /// order. Build modes aren't applied: a build keeps only the references
    /// that survive it.
    ///
    /// A reference to a file that doesn't exist is a problem, not an asset.
    pub fn assets(&self, page: &RelPath) -> Vec<PageAsset> {
        self.expansion(page)
            .map(|expanded| self.page_assets(&expanded))
            .unwrap_or_default()
    }

    /// The assets of an already expanded page.
    pub fn page_assets(&self, page: &ExpandedPage) -> Vec<PageAsset> {
        let mut out = Vec::new();
        page.visit(&mut |block| {
            let Some(index) = self.file_by_id(block.file) else {
                return;
            };
            for range in block.own_ranges() {
                for (at, reference) in index.references.iter().enumerate() {
                    if !range.contains_span(reference.span) {
                        continue;
                    }
                    let resolution = self.resolutions.get(&index.path).and_then(|r| r.get(at));
                    if let Some(Resolution::Asset { path, fragment }) = resolution {
                        out.push(PageAsset {
                            path: path.clone(),
                            kind: reference.kind,
                            fragment: fragment.clone(),
                            written_in: index.path.clone(),
                            location: Location::new(index.file, reference.span),
                            via: block.via.clone(),
                        });
                    }
                }
            }
        });
        out
    }

    /// The problems in one file that follow from the source alone, in source
    /// order: includes of files that don't exist, links and images to files
    /// that don't exist or can't be copied, links to fragments, links that
    /// look like routes, and links to ids the target page doesn't have.
    ///
    /// Include cycles and includes of ids that don't exist are found while
    /// expanding ([`ExpandedPage::problems`]). Each issue is located in the
    /// file where it's written, and none is reported here at an include site
    /// of a page: that is for the page-level checks.
    pub fn problems(&self, path: &RelPath) -> Vec<Issue> {
        let Some(index) = self.files.get(path) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for include in &index.includes {
            if let Some(issue) = self.include_problem(index, include) {
                out.push(issue);
            }
        }
        for (reference, resolution) in index.references.iter().zip(self.resolutions(path)) {
            out.extend(self.reference_problem(index, reference, resolution));
        }
        out.sort_by_key(|i| (i.location.span, i.slug));
        out
    }

    /// Headings with no `@id` whose slug is empty (SPEC §5.5): a heading made
    /// only of punctuation or emoji. Nothing can link to it usefully.
    /// `ascribe check` reports them as `heading-empty-slug` (SPEC §5.5).
    pub fn empty_slug_headings(&self) -> Vec<(RelPath, &Heading)> {
        self.files()
            .flat_map(|f| {
                f.headings
                    .iter()
                    .filter(|h| h.empty_slug && h.explicit_id.is_none())
                    .map(|h| (f.path.clone(), h))
            })
            .collect()
    }

    // -- Resolution ---------------------------------------------------------

    pub(crate) fn resolve_file(&self, index: &FileIndex, fs: &dyn FileSystem) -> Vec<Resolution> {
        index
            .references
            .iter()
            .map(|r| {
                resolve_reference(
                    r.kind,
                    &r.target,
                    &index.path,
                    &self.model,
                    self,
                    fs,
                    &self.layout,
                )
            })
            .collect()
    }

    fn build_edges(&self) -> ReverseEdges {
        let mut edges = ReverseEdges::default();
        for index in self.files.values() {
            edges.add_file(&self.files, index, self.resolutions(&index.path));
        }
        edges
    }

    // -- Updates (crate::incremental) --------------------------------------

    /// Swaps the content model. Nothing is re-indexed or re-resolved here.
    pub(crate) fn set_model(&mut self, model: Arc<ContentModel>) {
        self.model = model;
    }

    pub(crate) fn set_unreadable(&mut self, unreadable: Vec<Unreadable>) {
        self.unreadable = unreadable;
    }

    /// Forgets what a file contributed to the edges, and its resolutions.
    pub(crate) fn detach_file(&mut self, path: &RelPath) {
        if let Some(index) = self.files.get(path).cloned() {
            let resolutions = self.resolutions.get(path).cloned().unwrap_or_default();
            self.edges.remove_file(&index, &resolutions);
        }
    }

    /// Puts a file's index in place (replacing any before it; call
    /// [`Project::detach_file`] first) without touching its edges.
    pub(crate) fn put_index(&mut self, index: Arc<FileIndex>) {
        self.by_id.insert(index.file, index.path.clone());
        self.files.insert(index.path.clone(), index);
    }

    /// Removes a file: its index, its resolutions, and its id's entry.
    pub(crate) fn drop_index(&mut self, path: &RelPath) {
        if let Some(index) = self.files.remove(path) {
            self.by_id.remove(&index.file);
        }
        self.resolutions.remove(path);
    }

    pub(crate) fn put_resolutions(&mut self, path: &RelPath, resolutions: Vec<Resolution>) {
        self.resolutions.insert(path.clone(), Arc::new(resolutions));
    }

    /// Adds what a file contributes to the edges, from its current index and
    /// resolutions.
    pub(crate) fn attach_file(&mut self, path: &RelPath) {
        if let Some(index) = self.files.get(path).cloned() {
            let resolutions = self.resolutions.get(path).cloned().unwrap_or_default();
            self.edges.add_file(&self.files, &index, &resolutions);
        }
    }

    /// Recomputes the include edges that point at `target`, after a file
    /// appeared or disappeared there. `includers` are the files that write an
    /// `@include` naming it.
    pub(crate) fn refresh_include_edges(&mut self, target: &RelPath, includers: &[RelPath]) {
        self.edges.includes.remove(target);
        if !self.files.contains_key(target) {
            return;
        }
        let mut list = Vec::new();
        for file in includers {
            let Some(index) = self.files.get(file) else {
                continue;
            };
            for include in index
                .includes
                .iter()
                .filter(|i| i.target.as_ref() == Some(target))
            {
                insert_sorted(
                    &mut list,
                    IncludeEdge {
                        file: file.clone(),
                        span: include.span,
                        section: include.section.clone(),
                    },
                    |e| (e.file.clone(), e.span),
                );
            }
        }
        if !list.is_empty() {
            self.edges.includes.insert(target.clone(), list);
        }
    }

    /// Drops the cached expansions of these files, or of all files.
    pub(crate) fn forget_expansions(&self, paths: Option<&BTreeSet<RelPath>>) {
        let mut cache = self.expansion_cache();
        match paths {
            Some(paths) => cache.retain(|path, _| !paths.contains(path)),
            None => cache.clear(),
        }
    }

    pub(crate) fn resolution_list(&self, path: &RelPath) -> Arc<Vec<Resolution>> {
        self.resolutions.get(path).cloned().unwrap_or_default()
    }

    // -- Problems -----------------------------------------------------------

    fn include_problem(&self, index: &FileIndex, include: &Include) -> Option<Issue> {
        let at = Location::new(index.file, include.primary.unwrap_or(include.span));
        include_issue(&include.written, include.target.as_ref(), self, at)
    }

    fn reference_problem(
        &self,
        index: &FileIndex,
        reference: &Reference,
        resolution: &Resolution,
    ) -> Vec<Issue> {
        let at = Location::new(index.file, reference.span);
        let mut out: Vec<Issue> = reference_issue(
            reference.kind,
            &reference.target,
            &reference.destination,
            resolution,
            at,
            reference.destination_span,
        )
        .into_iter()
        .collect();
        // A link's `#id` names a heading of its target page, or, for a `#id`
        // alone in a fragment, of the fragment itself (SPEC §5.2, resolved
        let names_itself =
            matches!(&reference.target, crate::index::Target::Local(l) if l.written.is_empty());
        if let Resolution::Source {
            target,
            id: Some(id),
            fragment,
        } = resolution
            && (!fragment || names_itself)
        {
            out.extend(self.link_id_problem(reference, at, target, id));
        }
        out
    }

    /// A link's `#id` must be a source id of the target page itself, not of a
    /// fragment it includes (SPEC §4.2, §5.2).
    fn link_id_problem(
        &self,
        _reference: &Reference,
        at: Location,
        target: &RelPath,
        id: &str,
    ) -> Vec<Issue> {
        let Some(file) = self.files.get(target) else {
            return Vec::new();
        };
        if file.heading_by_id(id).is_some() {
            return Vec::new();
        }
        let in_fragment = self.expansion(target).and_then(|page| {
            page.headings(self)
                .into_iter()
                .find(|(file_path, heading)| file_path != target && heading.source_id == id)
                .map(|(file_path, _)| file_path)
        });
        let issue = match in_fragment {
            Some(fragment) => Issue::new(diagnostics::LINK_ID_IN_FRAGMENT, at)
                .with_arg("id", id)
                .with_arg("fragment", fragment.to_string())
                .with_arg("path", target.to_string()),
            None => Issue::new(diagnostics::LINK_ID_MISSING, at)
                .with_arg("path", target.to_string())
                .with_arg("id", id),
        };
        vec![issue]
    }
}

impl SourceSet for Project {
    fn contains(&self, path: &RelPath) -> bool {
        self.files.contains_key(path)
    }

    fn case_twin(&self, path: &RelPath) -> Option<RelPath> {
        let folded = path.as_str().to_lowercase();
        self.files
            .keys()
            .find(|p| p.as_str().to_lowercase() == folded)
            .cloned()
    }

    fn pages(&self) -> Vec<RelPath> {
        let mut pages: Vec<RelPath> = self.files.keys().cloned().collect();
        pages.sort();
        pages
    }
}
