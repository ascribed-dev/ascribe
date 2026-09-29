//! The [`Emitter`] trait, the context an emitter works in, and the shared
//! walk that turns a resolved build into files.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use tessera_core::{AssetUse, FileId, LineIndex, RelPath, Span, WideEncoding};
use tessera_model::{Build, ContentModel};
use tessera_resolve::{Project, RefKind, ResolvedBuild, ResolvedPage};

use crate::assets::{Placement, mirrored_path, relative_reference};
use crate::error::EmitError;
use crate::store::{Contents, EmittedFile, FileKind};

/// What an emitter needs to know about the project and the build it's
/// emitting, besides the resolved pages themselves.
pub struct EmitContext<'a> {
    /// The content model.
    pub model: &'a ContentModel,
    /// The build being emitted.
    pub build: &'a Build,
    content_dir: PathBuf,
    files: HashMap<FileId, SourceFile>,
}

struct SourceFile {
    path: RelPath,
    lines: LineIndex,
}

/// A position in a source file, counted from 1, with columns in Unicode
/// scalar values (as `ascribe check` reports them).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    /// The line.
    pub line: u32,
    /// The column.
    pub column: u32,
}

impl<'a> EmitContext<'a> {
    /// A context for emitting `build` of `project`, whose `ascribe.toml` is
    /// in `project_root`. Assets are copied from the content root under it.
    pub fn new(project: &'a Project, project_root: &Path, build: &'a Build) -> EmitContext<'a> {
        let files = project
            .files()
            .map(|f| {
                (
                    f.file,
                    SourceFile {
                        path: f.path.clone(),
                        lines: LineIndex::new(&f.source),
                    },
                )
            })
            .collect();
        EmitContext {
            model: project.model(),
            build,
            content_dir: project_root.join(project.layout().content_root.as_str()),
            files,
        }
    }

    /// The content path of a source file.
    pub fn file_path(&self, file: FileId) -> Option<&RelPath> {
        self.files.get(&file).map(|f| &f.path)
    }

    /// The position of a byte offset in a source file.
    pub fn position(&self, file: FileId, offset: usize) -> Option<Position> {
        let at = self
            .files
            .get(&file)?
            .lines
            .wide_line_col(WideEncoding::Utf32, offset)?;
        Some(Position {
            line: at.line + 1,
            column: at.col + 1,
        })
    }

    /// The start and end positions of a span.
    pub fn range(&self, file: FileId, span: Span) -> Option<(Position, Position)> {
        Some((
            self.position(file, span.start())?,
            self.position(file, span.end())?,
        ))
    }

    /// The published site's origin, when the content model has one.
    pub fn site_origin(&self) -> Option<&str> {
        self.model.consumer.site.as_deref()
    }

    /// A root-relative URL made absolute with the site's origin, when there is
    /// one (SPEC §9.4: plain-markdown links are absolute). Without an origin
    /// it stays root-relative, and the build warns
    /// ([`Emitter::warnings`]).
    pub fn absolute_url(&self, url: &str) -> String {
        match self.site_origin() {
            Some(origin) if url.starts_with('/') && !url.starts_with("//") => {
                format!("{}{url}", origin.trim_end_matches('/'))
            }
            _ => url.to_owned(),
        }
    }

    /// Where an asset is read from.
    pub fn asset_source(&self, asset: &RelPath) -> PathBuf {
        self.content_dir.join(asset.as_str())
    }
}

/// What an emitter sees while it renders one page.
pub struct PageContext<'a> {
    /// The project and build.
    pub emit: &'a EmitContext<'a>,
    /// Where the page is written, relative to the emitter root.
    pub output: RelPath,
    emitter: &'a dyn Emitter,
}

impl PageContext<'_> {
    /// Where this emitter puts an asset a reference on this page uses, and how
    /// the page refers to it.
    pub fn asset(&self, asset: &RelPath, usage: AssetUse) -> Placement {
        self.emitter.place_asset(&self.output, asset, usage)
    }
}

/// One output (SPEC §9.4): the same resolved tree in another form.
///
/// An emitter renders what the resolved tree holds and nothing else: it never
/// works out what a build mode would keep (SPEC §9.2).
pub trait Emitter {
    /// The emitter's name: its directory under the build's output, and the
    /// name `--emit` takes (`plain`, `json`, `site`).
    fn name(&self) -> &'static str;

    /// Where a page is written, relative to the emitter root. Pages mirror
    /// their source paths (output-layout contract, §1.1).
    fn page_path(&self, page: &RelPath) -> RelPath;

    /// Checks that the resolved build can be written in this output's form,
    /// before any page is rendered. The site output refuses two pages with
    /// the same route.
    ///
    /// # Errors
    ///
    /// When it can't.
    fn prepare(&self, _cx: &EmitContext<'_>, _build: &ResolvedBuild) -> Result<(), EmitError> {
        Ok(())
    }

    /// Renders a page.
    ///
    /// # Errors
    ///
    /// When the page can't be rendered in this output's form.
    fn render_page(&self, cx: &PageContext<'_>, page: &ResolvedPage) -> Result<String, EmitError>;

    /// Where a copy of `asset` goes, and how the page written at `page_output`
    /// refers to it (asset contract, §3.2). The default is the mirrored path
    /// and a relative reference, which is what the plain-markdown and JSON
    /// outputs do.
    fn place_asset(&self, page_output: &RelPath, asset: &RelPath, _usage: AssetUse) -> Placement {
        let copy_to = mirrored_path(asset);
        Placement {
            reference: relative_reference(page_output, &copy_to),
            copy_to,
            url: None,
        }
    }

    /// Generated files the emitter adds to the root besides pages and assets,
    /// under `_tessera/` (output-layout contract, §1.1).
    ///
    /// # Errors
    ///
    /// When a file can't be generated.
    fn generated(
        &self,
        _cx: &EmitContext<'_>,
        _build: &ResolvedBuild,
    ) -> Result<Vec<EmittedFile>, EmitError> {
        Ok(Vec::new())
    }

    /// Warnings about this build in this output that aren't diagnostics of
    /// the source: things the content model leaves out that make the output
    /// less useful.
    fn warnings(&self, _cx: &EmitContext<'_>) -> Vec<String> {
        Vec::new()
    }
}

/// The files one emitter writes for one build.
#[derive(Debug)]
pub struct Emission {
    /// Every file, sorted by path.
    pub files: Vec<EmittedFile>,
}

/// Renders every page of a resolved build with `emitter`, and lists the
/// assets those pages use, each once (asset contract, §5).
///
/// # Errors
///
/// When a page can't be rendered.
pub fn emit(
    emitter: &dyn Emitter,
    cx: &EmitContext<'_>,
    build: &ResolvedBuild,
) -> Result<Emission, EmitError> {
    emitter.prepare(cx, build)?;
    let mut files = Vec::new();
    let mut copies: BTreeSet<(RelPath, RelPath)> = BTreeSet::new();
    let mut urls: BTreeMap<RelPath, String> = BTreeMap::new();
    for page in &build.pages {
        let output = emitter.page_path(&page.path);
        let page_cx = PageContext {
            emit: cx,
            output: output.clone(),
            emitter,
        };
        let text = emitter.render_page(&page_cx, page)?;
        files.push(EmittedFile {
            path: output.clone(),
            kind: FileKind::Page,
            source: Some(page.path.clone()),
            url: None,
            contents: Contents::Text(text),
        });
        for asset in &page.assets {
            let usage = match asset.kind {
                RefKind::Image => AssetUse::Image,
                RefKind::Link => AssetUse::Link,
            };
            let placement = emitter.place_asset(&output, &asset.path, usage);
            if let Some(url) = placement.url {
                urls.insert(placement.copy_to.clone(), url);
            }
            copies.insert((placement.copy_to, asset.path.clone()));
        }
    }
    for (copy_to, source) in copies {
        files.push(EmittedFile {
            url: urls.get(&copy_to).cloned(),
            contents: Contents::Copy(cx.asset_source(&source)),
            path: copy_to,
            kind: FileKind::Asset,
            source: Some(source),
        });
    }
    files.extend(emitter.generated(cx, build)?);
    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(Emission { files })
}
