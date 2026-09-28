//! The project: every source file's index, what the references in them name,
//! the edges between files, and the problems found on the way.

use std::collections::{BTreeMap, BTreeSet};

use std::sync::Arc;
use tessera_core::{FileId, Issue, Location, RelPath, Slugger, Span, diagnostics};
use tessera_model::ContentModel;

use crate::expand::{ExpandedPage, Expander, IncludeSite};
use crate::fs::{FileSystem, Probe};
use crate::index::{FileIndex, FileKind, Heading, Include, RefKind, Reference, Target, index_file};
use crate::layout::Layout;
use crate::slug::{default_slugger, slugger_by_name};

/// What a link or image names, once the project's files are known.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Resolution {
    /// A URL with a scheme. Nothing to check or copy.
    External,
    /// A reference form whose definition holds a declared phrase, which can't
    /// be applied yet ([`Target::Deferred`]). Nothing is said about it.
    Deferred,
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
    /// the boundary (asset contract, §2).
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
    /// names no file (SPEC §5.2, Q22).
    Route {
        /// The page it probably means, as a content path.
        page: String,
        /// The link to write instead, as a file path.
        suggestion: String,
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
    /// is treated as not existing (SPEC §9.4, Q10).
    Outside,
}

/// A source file the project couldn't read.
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

/// The source index of a project: independent of any build.
///
/// It holds one [`FileIndex`] per source file, what every link and image
/// names, the edges between files in both directions, and the includes'
/// expansion on demand ([`Project::expand`]).
#[derive(Debug)]
pub struct Project {
    pub(crate) model: Arc<ContentModel>,
    pub(crate) layout: Layout,
    pub(crate) files: BTreeMap<RelPath, FileIndex>,
    order: Vec<RelPath>,
    resolutions: BTreeMap<RelPath, Vec<Resolution>>,
    edges: ReverseEdges,
    unreadable: Vec<Unreadable>,
}

impl Project {
    /// Indexes every `.md` file under the content root.
    ///
    /// Each file gets a [`FileId`], in path order. A file that can't be read
    /// is left out and listed in [`Project::unreadable`].
    pub fn load(model: Arc<ContentModel>, layout: Layout, fs: &dyn FileSystem) -> Project {
        let slugger: Box<dyn Slugger> =
            slugger_by_name(&model.consumer.slugger).unwrap_or_else(default_slugger);
        let mut paths = fs.sources();
        paths.sort();
        paths.dedup();

        let mut files = BTreeMap::new();
        let mut order = Vec::new();
        let mut unreadable = Vec::new();
        for path in paths {
            match fs.read(&path) {
                Ok(text) => {
                    let id = file_id(order.len());
                    let index = index_file(id, &path, &text, &model, slugger.as_ref());
                    order.push(path.clone());
                    files.insert(path, index);
                }
                Err(err) => unreadable.push(Unreadable {
                    path,
                    reason: err.to_string(),
                }),
            }
        }

        let mut project = Project {
            model,
            layout,
            files,
            order,
            resolutions: BTreeMap::new(),
            edges: ReverseEdges::default(),
            unreadable,
        };
        project.resolutions = project
            .files
            .values()
            .map(|index| (index.path.clone(), project.resolve_file(index, fs)))
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
        self.files.values()
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
        self.files.get(path)
    }

    /// The path of a file, by id.
    pub fn path_of(&self, file: FileId) -> Option<&RelPath> {
        self.order.get(file_index(file))
    }

    /// The index of a file, by id.
    pub fn file_by_id(&self, file: FileId) -> Option<&FileIndex> {
        self.path_of(file).and_then(|p| self.files.get(p))
    }

    /// Source files that couldn't be read.
    pub fn unreadable(&self) -> &[Unreadable] {
        &self.unreadable
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
        self.resolutions.get(path).map_or(&[], Vec::as_slice)
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
    pub fn expand(&self, path: &RelPath) -> Option<ExpandedPage> {
        Expander::new(self).expand(path)
    }

    /// The assets a page uses: every image, and every link to a file that
    /// isn't a source file, in the page after its includes are expanded,
    /// resolved from the file each reference is written in, in document
    /// order. Build modes aren't applied: a build keeps only the references
    /// that survive it (phase 12).
    ///
    /// A reference to a file that doesn't exist is a problem, not an asset.
    pub fn assets(&self, page: &RelPath) -> Vec<PageAsset> {
        self.expand(page)
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
    // SPEC-QUESTION(Q61): proposed warning; there is no registry entry yet.
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

    fn resolve_file(&self, index: &FileIndex, fs: &dyn FileSystem) -> Vec<Resolution> {
        index
            .references
            .iter()
            .map(|r| self.resolve_reference(index, r, fs))
            .collect()
    }

    fn resolve_reference(
        &self,
        index: &FileIndex,
        reference: &Reference,
        fs: &dyn FileSystem,
    ) -> Resolution {
        let local = match &reference.target {
            Target::External => return Resolution::External,
            Target::Deferred => return Resolution::Deferred,
            Target::Local(local) => local,
        };
        let Some(path) = &local.path else {
            return Resolution::AssetMissing(Missing::Absent);
        };
        if local.source {
            return match self.files.get(path) {
                Some(target) => Resolution::Source {
                    target: path.clone(),
                    id: local.fragment.clone(),
                    fragment: target.kind == FileKind::Fragment,
                },
                None => Resolution::SourceMissing {
                    actual: self.case_twin(path),
                },
            };
        }
        // A file the build would copy.
        let missing = if !self.layout.is_allowed(path) {
            Missing::Outside
        } else {
            match fs.probe(&self.layout.project_path(path)) {
                Probe::File => {
                    return Resolution::Asset {
                        path: path.clone(),
                        fragment: local.fragment.clone(),
                    };
                }
                Probe::Missing => Missing::Absent,
                Probe::CaseMismatch(actual) => match self.content_path_of(&actual) {
                    Some(actual) => Missing::Case(actual),
                    None => Missing::Absent,
                },
            }
        };
        if local.route_like && !matches!(missing, Missing::Case(_)) {
            return self.route(index, local, path);
        }
        Resolution::AssetMissing(missing)
    }

    /// The page a route-like link probably means, and the file-path link to
    /// write instead.
    fn route(&self, index: &FileIndex, local: &crate::index::Local, path: &RelPath) -> Resolution {
        let as_directory = path.join("index.md").unwrap_or_else(|_| path.clone());
        let as_file = match path.file_name() {
            Some(_) => RelPath::parse(&format!("{path}.md")).unwrap_or_else(|_| path.clone()),
            None => as_directory.clone(),
        };
        // A route `a/b` means `a/b.md`; `a/b/` may mean `a/b/index.md`.
        let page = if local.written.ends_with('/')
            && self.files.contains_key(&as_directory)
            && !self.files.contains_key(&as_file)
        {
            as_directory
        } else {
            as_file
        };
        let suggestion = if local.written.starts_with('/') {
            format!("/{page}")
        } else {
            let from = index.path.parent().unwrap_or_default();
            if page.is_inside() {
                page.relative_from(&from)
                    .unwrap_or_else(|| format!("/{page}"))
            } else {
                // A page above the content root: up out of the file's directory
                // first, then the path's own `..` segments.
                format!("{}{page}", "../".repeat(from.segments().count()))
            }
        };
        let suggestion = match &local.fragment {
            Some(id) => format!("{suggestion}#{id}"),
            None => suggestion,
        };
        Resolution::Route {
            page: page.to_string(),
            suggestion,
        }
    }

    /// A project file whose path differs from `path` only in case.
    fn case_twin(&self, path: &RelPath) -> Option<RelPath> {
        let folded = path.as_str().to_lowercase();
        self.files
            .keys()
            .find(|p| p.as_str().to_lowercase() == folded)
            .cloned()
    }

    /// A path relative to the project root as a content path, when it's
    /// inside the content root or reaches it by `..`.
    fn content_path_of(&self, project_path: &RelPath) -> Option<RelPath> {
        let root: Vec<&str> = self.layout.content_root.segments().collect();
        let mine: Vec<&str> = project_path.segments().collect();
        let common = root.iter().zip(&mine).take_while(|(a, b)| a == b).count();
        let ups = "../".repeat(root.len() - common);
        RelPath::parse(&format!("{ups}{}", mine[common..].join("/"))).ok()
    }

    fn build_edges(&self) -> ReverseEdges {
        let mut edges = ReverseEdges::default();
        for index in self.files.values() {
            for include in &index.includes {
                let Some(target) = &include.target else {
                    continue;
                };
                if self.files.contains_key(target) {
                    edges
                        .includes
                        .entry(target.clone())
                        .or_default()
                        .push(IncludeEdge {
                            file: index.path.clone(),
                            span: include.span,
                            section: include.section.clone(),
                        });
                }
            }
            for (reference, resolution) in
                index.references.iter().zip(self.resolutions(&index.path))
            {
                match resolution {
                    Resolution::Source { target, id, .. } => {
                        edges
                            .links
                            .entry(target.clone())
                            .or_default()
                            .push(LinkSite {
                                file: index.path.clone(),
                                span: reference.span,
                                id: id.clone(),
                            });
                    }
                    Resolution::Asset { path, .. } => {
                        edges
                            .assets
                            .entry(path.clone())
                            .or_default()
                            .push(AssetSite {
                                file: index.path.clone(),
                                span: reference.span,
                                kind: reference.kind,
                            });
                    }
                    _ => {}
                }
            }
        }
        edges
    }

    // -- Problems -----------------------------------------------------------

    fn include_problem(&self, index: &FileIndex, include: &Include) -> Option<Issue> {
        let target = include.target.as_ref()?;
        if self.files.contains_key(target) {
            return None;
        }
        let at = Location::new(index.file, include.primary.unwrap_or(include.span));
        let issue = Issue::new(diagnostics::INCLUDE_TARGET_MISSING, at)
            .with_arg("path", include.written.clone());
        Some(match self.case_twin(target) {
            Some(actual) => issue
                .with_variant("case")
                .with_arg("actual", actual.to_string()),
            None => issue,
        })
    }

    fn reference_problem(
        &self,
        index: &FileIndex,
        reference: &Reference,
        resolution: &Resolution,
    ) -> Vec<Issue> {
        let at = Location::new(index.file, reference.span);
        let written = match &reference.target {
            Target::Local(l) => l.written.clone(),
            _ => reference.destination.clone(),
        };
        let missing_slug = match reference.kind {
            RefKind::Link => diagnostics::LINK_TARGET_MISSING,
            RefKind::Image => diagnostics::IMAGE_SOURCE_MISSING,
        };
        match resolution {
            Resolution::External | Resolution::Deferred | Resolution::Asset { .. } => Vec::new(),
            Resolution::SourceMissing { actual } => {
                let issue = Issue::new(missing_slug, at).with_arg("path", written);
                vec![match actual {
                    Some(actual) => issue
                        .with_variant("case")
                        .with_arg("actual", actual.to_string()),
                    None => issue,
                }]
            }
            Resolution::AssetMissing(why) => {
                let issue = Issue::new(missing_slug, at).with_arg("path", written);
                vec![match why {
                    Missing::Absent => issue,
                    Missing::Case(actual) => issue
                        .with_variant("case")
                        .with_arg("actual", actual.to_string()),
                    Missing::Outside => issue.with_variant("outside"),
                }]
            }
            Resolution::Route { page, suggestion } => vec![
                Issue::new(diagnostics::LINK_ROUTE, at)
                    .with_arg("page", page.clone())
                    .with_arg("suggestion", suggestion.clone()),
            ],
            Resolution::Source {
                target,
                id,
                fragment,
            } => {
                if *fragment {
                    // SPEC-QUESTION(Q64): a link with only `#id` in a fragment
                    // names the fragment itself, so it's reported like any
                    // other link to a fragment.
                    return vec![
                        Issue::new(diagnostics::LINK_TO_FRAGMENT, at).with_arg("path", written),
                    ];
                }
                match id {
                    Some(id) => self.link_id_problem(reference, at, target, id),
                    None => Vec::new(),
                }
            }
        }
    }

    /// A link's `#id` must be a source id of the target page itself, not of a
    /// fragment it includes (SPEC §4.2, §5.2, Q6).
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
        let in_fragment = self.expand(target).and_then(|page| {
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

fn file_id(index: usize) -> FileId {
    // A project never has 2^32 files; saturate rather than wrap.
    FileId::new(u32::try_from(index).unwrap_or(u32::MAX))
}

fn file_index(file: FileId) -> usize {
    file.index() as usize
}
