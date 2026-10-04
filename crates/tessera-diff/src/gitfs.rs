//! A project as it is at a git revision.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io;
use std::sync::Arc;

use tessera_core::{FileId, RelPath};
use tessera_model::ContentModel;
use tessera_resolve::{
    FileSystem, Layout, Probe, Project, Sources, in_nested_project, is_source_path,
};

use crate::DiffError;
use crate::git::{Repository, Tree};

/// The content model's file name.
pub const MODEL_FILE: &str = "ascribe.toml";

/// A project's files at one revision, read from git. It implements
/// [`FileSystem`] with the same rules as `DiskFs`: the same files are
/// sources, and the same folders are other projects' and skipped.
///
/// Every source file is read when it's made, through one `git cat-file
/// --batch`. Other files (images, downloads) are only listed: a probe needs
/// to know they exist, not what they hold. A symbolic link is listed as a
/// file but never followed, so a source file that's a link isn't a source.
#[derive(Clone, Debug)]
pub struct GitFs {
    tree: Arc<Tree>,
    /// The project's folder, from the repository's root.
    project_dir: RelPath,
    /// The content root, from the repository's root.
    content_dir: RelPath,
    /// The project's own folder below the content root, when the content
    /// root is above it.
    own_folder: Option<RelPath>,
    /// Every source file's contents, by content path.
    contents: BTreeMap<RelPath, Vec<u8>>,
    sources: Sources,
    /// Every file's path from the repository's root, lowercased, to its
    /// real spelling, for the case-mismatch probe.
    folded: HashMap<String, RelPath>,
}

/// A project read at a revision: its content model, as text and loaded, and
/// its files.
#[derive(Clone, Debug)]
pub struct Revision {
    /// The content model's text.
    pub model_text: String,
    /// The content model.
    pub model: Arc<ContentModel>,
    /// The files.
    pub fs: GitFs,
}

impl Revision {
    /// The project in the repository's project folder at `commit`, or `None`
    /// when there's no `ascribe.toml` there at that commit.
    ///
    /// # Errors
    ///
    /// [`DiffError::BaseModel`] or [`DiffError::BaseModelText`] when the
    /// content model doesn't load, and
    /// [`DiffError::Git`] when git fails.
    pub fn read(repo: &Repository, commit: &str) -> Result<Option<Revision>, DiffError> {
        let tree = Arc::new(repo.tree(commit)?);
        let project_dir = repo.project_dir();
        let model_path = project_dir
            .join(MODEL_FILE)
            .map_err(|e| DiffError::Path(e.to_string()))?;
        let Some(entry) = tree.files.get(&model_path).filter(|e| !e.symlink) else {
            return Ok(None);
        };
        let bytes = repo
            .read_blobs(&[entry.object.as_str()])?
            .pop()
            .unwrap_or_default();
        let model_text = String::from_utf8(bytes).map_err(|_| DiffError::BaseModelText {
            commit: commit.to_owned(),
        })?;
        let model = tessera_model::load_str(&model_text, FileId::new(0)).map_err(|issues| {
            DiffError::BaseModel {
                commit: commit.to_owned(),
                text: model_text.clone(),
                issues,
            }
        })?;
        let layout = Layout::from_model(&model);
        let fs = GitFs::read(repo, tree, &layout)?;
        Ok(Some(Revision {
            model_text,
            model: Arc::new(model),
            fs,
        }))
    }

    /// The project's source index.
    pub fn project(&self) -> Project {
        Project::load(
            self.model.clone(),
            Layout::from_model(&self.model),
            &self.fs,
        )
    }
}

impl GitFs {
    /// The project in `repo`'s project folder, with `layout`, from `tree`,
    /// reading every source file.
    ///
    /// # Errors
    ///
    /// [`DiffError::OutsideRepository`] when the content root is above the
    /// repository's root, and [`DiffError::Git`] when git fails.
    pub fn read(repo: &Repository, tree: Arc<Tree>, layout: &Layout) -> Result<GitFs, DiffError> {
        let project_dir = repo.project_dir();
        let content_dir = project_dir
            .join(layout.content_root.as_str())
            .map_err(|e| DiffError::Path(e.to_string()))?;
        if !content_dir.is_inside() {
            return Err(DiffError::OutsideRepository(
                layout.content_root.to_string(),
            ));
        }
        let own_folder = relative_to(&project_dir, &content_dir).filter(|p| !p.is_root());
        let mut fs = GitFs {
            tree,
            project_dir,
            content_dir,
            own_folder,
            contents: BTreeMap::new(),
            sources: Sources::default(),
            folded: HashMap::new(),
        };
        fs.folded = fs
            .tree
            .files
            .keys()
            .map(|p| (p.as_str().to_lowercase(), p.clone()))
            .collect();
        fs.sources = fs.discover();
        // Each path with its object, so a blob can only land on its own path.
        let wanted: Vec<(RelPath, &str)> = fs
            .sources
            .paths
            .iter()
            .filter_map(|p| fs.entry(p).map(|e| (p.clone(), e.object.as_str())))
            .collect();
        let objects: Vec<&str> = wanted.iter().map(|(_, o)| *o).collect();
        let blobs = repo.read_blobs(&objects)?;
        let contents = wanted.into_iter().map(|(p, _)| p).zip(blobs).collect();
        fs.contents = contents;
        Ok(fs)
    }

    /// The blob of the file at a content path, when there's one and it
    /// isn't a symbolic link: images and other files a page uses, which
    /// aren't read when the project is.
    pub fn object(&self, content_path: &RelPath) -> Option<&str> {
        self.entry(content_path)
            .filter(|e| !e.symlink)
            .map(|e| e.object.as_str())
    }

    fn entry(&self, content_path: &RelPath) -> Option<&crate::git::TreeEntry> {
        let path = self.content_dir.join(content_path.as_str()).ok()?;
        self.tree.files.get(&path)
    }

    /// The source files, by the same rule as `DiskFs` and `MemoryFs`.
    fn discover(&self) -> Sources {
        let content: BTreeSet<RelPath> = self
            .tree
            .files
            .iter()
            .filter(|(_, e)| !e.symlink)
            .filter_map(|(p, _)| relative_to(p, &self.content_dir))
            .filter(|p| !p.is_root())
            .collect();
        // The directories below the content root that hold an `ascribe.toml`,
        // outermost first, keeping only those a walk would reach: not hidden,
        // and not inside another one. The project's own folder isn't another
        // project's.
        let mut holding: Vec<RelPath> = content
            .iter()
            .filter(|p| p.file_name() == Some(MODEL_FILE))
            .filter_map(RelPath::parent)
            .filter(|dir| !dir.is_root())
            .filter(|dir| Some(dir) != self.own_folder.as_ref())
            .filter(|dir| !dir.segments().any(|s| s.starts_with('.')))
            .collect();
        holding.sort_by_key(|dir| dir.segments().count());
        let mut nested: Vec<RelPath> = Vec::new();
        for dir in holding {
            if !in_nested_project(&dir, &nested) {
                nested.push(dir);
            }
        }
        nested.sort();
        let paths = content
            .into_iter()
            .filter(|p| is_source_path(p) && !in_nested_project(p, &nested))
            .collect();
        Sources {
            paths,
            unreadable: Vec::new(),
            nested,
            own_folder: self.own_folder.clone(),
        }
    }
}

/// `path` relative to `dir`, when it's `dir` or inside it.
fn relative_to(path: &RelPath, dir: &RelPath) -> Option<RelPath> {
    if !path.starts_with(dir) {
        return None;
    }
    let rest: Vec<&str> = path.segments().skip(dir.segments().count()).collect();
    RelPath::parse(&rest.join("/")).ok()
}

impl FileSystem for GitFs {
    fn sources(&self) -> Sources {
        self.sources.clone()
    }

    fn read(&self, path: &RelPath) -> io::Result<String> {
        let bytes = self
            .contents
            .get(path)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no such file"))?;
        String::from_utf8(bytes.clone()).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "stream did not contain valid UTF-8",
            )
        })
    }

    fn probe(&self, project_path: &RelPath) -> Probe {
        let Ok(path) = self.project_dir.join(project_path.as_str()) else {
            return Probe::Missing;
        };
        if !path.is_inside() || path.is_root() {
            return Probe::Missing;
        }
        if self.tree.files.contains_key(&path) {
            return Probe::File;
        }
        let Some(actual) = self.folded.get(&path.as_str().to_lowercase()) else {
            return Probe::Missing;
        };
        // The same path from the project root, with the real spelling of the
        // segments after its leading `..`s.
        let ups = project_path.up_count();
        let rest = project_path.segments().count() - ups;
        let spelled: Vec<&str> = actual.segments().collect();
        let mut out: Vec<&str> = vec![".."; ups];
        out.extend(&spelled[spelled.len().saturating_sub(rest)..]);
        match RelPath::parse(&out.join("/")) {
            Ok(p) => Probe::CaseMismatch(p),
            Err(_) => Probe::Missing,
        }
    }
}
