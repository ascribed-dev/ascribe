//! A project loaded from the disk for a command's edit, as the language
//! server loads one with no document open: the same content model, a
//! [`Workspace`] over the same file systems, and the same [`Ctx`] for each
//! source file, so the command and the editor change the same places.

use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use ascribe_core::RelPath;
use ascribe_core::path::{normalize, relative_path};
use ascribe_model::{ContentModel, SourceBound};
use ascribe_resolve::{DiskFs, Layout};
use ascribe_workspace::{LoadError, Workspace};

use crate::fsx::LayerFs;
use crate::nav::Ctx;
use crate::position::Encoding;

/// Directories nothing in a documentation set lives in: their files are
/// never a project's sources, and the server ignores their changes.
pub const IGNORED_DIRS: [&str; 3] = [".git", "node_modules", ".hg"];

/// What loading a project from its `ascribe.toml` found.
pub enum Load {
    /// `ascribe.toml` can't be read.
    Unread,
    /// The content model doesn't load, for these problems.
    Problem(Vec<ascribe_check::Diagnostic>),
    /// The project.
    Loaded(Box<Loaded>),
}

/// A project loaded from the disk.
pub struct Loaded {
    /// The project's `ascribe.toml`, absolute.
    pub config: PathBuf,
    /// The folder `ascribe.toml` is in.
    pub root: PathBuf,
    /// Where the project's files are.
    pub layout: Layout,
    /// The project.
    pub workspace: Workspace,
    /// The content model.
    pub model: Arc<ContentModel>,
    /// Its text.
    pub model_text: String,
    /// The disk the project was read through, and its files that aren't
    /// sources are.
    pub fs: Arc<LayerFs>,
}

/// Loads the project of `config`, an `ascribe.toml`, from the disk. A
/// command reads a source wherever the model says, so a source isn't bound
/// to a folder.
pub fn load(config: &Path) -> Load {
    let config = normalize(config);
    let loader = ascribe_check::LOADER;
    let file = match loader.load_model_bounded(&config, SourceBound::Anywhere) {
        Ok(file) => file,
        Err(LoadError::Read { .. }) => return Load::Unread,
        Err(LoadError::Model { issues, .. }) => {
            return Load::Problem(
                issues
                    .iter()
                    .map(ascribe_check::Diagnostic::from_issue)
                    .collect(),
            );
        }
    };
    let root = file.root.clone();
    let model_text = file.text.clone();
    let model = Arc::new(file.model.clone());
    let layout = Layout::from_model(&model);
    let fs = Arc::new(LayerFs::new(DiskFs::new(&root, &layout)));
    let workspace = Workspace::load_with(
        loader.side_files,
        file,
        layout.content_root.clone(),
        fs.clone(),
    );
    Load::Loaded(Box::new(Loaded {
        config,
        root,
        layout,
        workspace,
        model,
        model_text,
        fs,
    }))
}

impl Loaded {
    /// The content root, absolute.
    pub fn content_dir(&self) -> PathBuf {
        normalize(&self.root.join(self.layout.content_root.as_str()))
    }

    /// What an edit of the source file at `abs` works from, in UTF-8. `None`
    /// when it isn't one of the project's source files.
    pub fn ctx(&self, abs: &Path) -> Option<Ctx> {
        let abs = normalize(abs);
        let project_rel = relative_path(&self.root, &abs)?;
        if project_rel.segments().any(|s| IGNORED_DIRS.contains(&s))
            || ascribe_resolve::is_memo_path(&project_rel, &self.layout)
        {
            return None;
        }
        let content_dir = self.content_dir();
        let path = relative_path(&content_dir, &abs).filter(RelPath::is_inside)?;
        let snapshot = self.workspace.index_snapshot();
        if !snapshot.is_source(&path) {
            return None;
        }
        snapshot.file(&path)?;
        Some(Ctx {
            snapshot,
            path,
            model: self.model.clone(),
            model_text: self.model_text.clone(),
            model_problem: false,
            config: self.config.clone(),
            content_dir,
            encoding: Encoding::Utf8,
            fs: self.fs.clone(),
            taken: OnceLock::new(),
        })
    }

    /// What an edit of the whole project works from: the first source
    /// file's [`Loaded::ctx`]. `None` when the project has none.
    pub fn project_ctx(&self) -> Option<Ctx> {
        let snapshot = self.workspace.index_snapshot();
        let file = snapshot.files().next()?;
        self.ctx(&self.content_dir().join(file.path.as_str()))
    }
}
