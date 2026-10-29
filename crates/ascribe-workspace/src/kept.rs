//! A project kept loaded between calls and brought up to date before each
//! from a [`Listing`] of its files, as the MCP server keeps one.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use ascribe_core::RelPath;
use ascribe_core::path::normalize;
use ascribe_model::{LOCK_FILE, SourceBound};
use ascribe_resolve::{DiskFs, Layout};

use crate::listing::{CodeStamps, Listing, code_change};
use crate::load::{LoadError, Loader, ModelFile};
use crate::workspace::{Changed, SharedFs, Workspace, WorkspaceSnapshot};

/// Makes the file system a project at a root, with a layout, is read
/// through.
pub type FsFor = Box<dyn Fn(&Path, &Layout) -> SharedFs>;

/// What a kept project's files are read through.
enum Files {
    /// The disk, keeping what it lists and reads until the listing shows a
    /// change: then it's all read again.
    Disk(Option<DiskFs>),
    /// What a function makes.
    Given(FsFor),
}

impl Files {
    fn make(&mut self, root: &Path, layout: &Layout) -> SharedFs {
        match self {
            Files::Disk(disk) => {
                let made = DiskFs::with_listing_cache(root, layout);
                *disk = Some(made.clone());
                Arc::new(made)
            }
            Files::Given(fs_for) => fs_for(root, layout),
        }
    }

    /// Forgets what the disk said, which has changed.
    fn forget(&self) {
        if let Files::Disk(Some(disk)) = self {
            disk.forget();
        }
    }
}

/// A [`Workspace`] kept between calls, with the listing of its files it was
/// loaded from. Before each call, [`Kept::refresh`] lists the files again
/// and applies the difference, reading only the files that differ; a
/// difference the `Workspace` can't take in place loads the project again.
pub struct Kept {
    loader: Loader,
    config: PathBuf,
    root: PathBuf,
    files: Files,
    workspace: Workspace,
    listing: Listing,
    /// The code files snippets have read, each as it was when it was first
    /// seen read.
    code: CodeStamps,
}

impl Kept {
    /// Loads the project of `config`, an `ascribe.toml`, with `loader`, its
    /// files read from the disk as they are at each call.
    ///
    /// # Errors
    ///
    /// As [`Workspace::load`].
    pub fn load(loader: Loader, config: &Path, bound: SourceBound<'_>) -> Result<Kept, LoadError> {
        Kept::load_from(loader, config, bound, Files::Disk(None))
    }

    /// [`Kept::load`], with each project's files read through what
    /// `fs_for` makes for its root and layout.
    ///
    /// # Errors
    ///
    /// As [`Workspace::load`].
    pub fn load_through(
        loader: Loader,
        config: &Path,
        bound: SourceBound<'_>,
        fs_for: FsFor,
    ) -> Result<Kept, LoadError> {
        Kept::load_from(loader, config, bound, Files::Given(fs_for))
    }

    fn load_from(
        loader: Loader,
        config: &Path,
        bound: SourceBound<'_>,
        mut files: Files,
    ) -> Result<Kept, LoadError> {
        let config = normalize(&std::path::absolute(config).unwrap_or_else(|_| config.to_owned()));
        let model = loader.load_model_bounded(&config, bound)?;
        let root = model.root.clone();
        let (workspace, listing) = Kept::read(loader, &config, model, &mut files)?;
        let mut kept = Kept {
            loader,
            config,
            root,
            files,
            workspace,
            listing,
            code: CodeStamps::default(),
        };
        kept.stamp_snippets();
        Ok(kept)
    }

    /// The project around `model`, and the listing of its files, made
    /// before it's read, so a file that changes while it is is read again.
    fn read(
        loader: Loader,
        config: &Path,
        model: ModelFile,
        files: &mut Files,
    ) -> Result<(Workspace, Listing), LoadError> {
        let content_root =
            RelPath::parse(&model.model.project.content_root).map_err(|e| LoadError::Read {
                path: config.display().to_string(),
                message: e.to_string(),
            })?;
        let layout = Layout::from_model(&model.model);
        let listing = Listing::of(&model.root, &layout, &model.model, loader.side_files);
        let fs = files.make(&model.root, &layout);
        let workspace = Workspace::load_with(loader.side_files, model, content_root, fs);
        Ok((workspace, listing))
    }

    /// The project as it is.
    pub fn snapshot(&self) -> WorkspaceSnapshot {
        self.workspace.snapshot()
    }

    /// The `Workspace` kept.
    pub fn workspace(&self) -> &Workspace {
        &self.workspace
    }

    /// Brings the project up to date with its files: lists them again, and
    /// applies what differs, reading only the files that do. When
    /// `ascribe.toml` or `ascribe.lock` changed, the model is loaded again
    /// and applied. A difference the `Workspace` refuses, or can't be told
    /// as changes, loads the project again.
    ///
    /// # Errors
    ///
    /// The content model doesn't load now: the project is as it was, and
    /// should be dropped.
    pub fn refresh(&mut self, bound: SourceBound<'_>) -> Result<(), LoadError> {
        let snapshot = self.workspace.snapshot();
        let now = Listing::of(
            &self.root,
            snapshot.layout(),
            snapshot.model(),
            self.loader.side_files,
        );
        let differences = self.listing.differences(&now);
        let code = self.code_changes(&snapshot);
        if differences.is_empty() && code.is_empty() {
            return Ok(());
        }
        self.files.forget();
        let model_changed = differences.touches(&self.root, ascribe_model::MODEL_FILE)
            || differences.touches(&self.root, LOCK_FILE);
        let model = if model_changed {
            let model = self.loader.load_model_bounded(&self.config, bound)?;
            // Another source changes what's listed, and where snippets read.
            let sources = |m: &ascribe_model::ContentModel| {
                m.sources.iter().map(|s| s.path.clone()).collect::<Vec<_>>()
            };
            if sources(&model.model) != sources(snapshot.model()) {
                return self.reload(model);
            }
            Some(model)
        } else {
            None
        };
        let Some(mut changes) = differences.changes(&self.root, &snapshot) else {
            return self.reload_bounded(bound);
        };
        changes.extend(code);
        changes.extend(model.map(|m| Changed::Model(Box::new(m))));
        if self.workspace.apply(changes).is_err() {
            return self.reload_bounded(bound);
        }
        self.listing = now;
        Ok(())
    }

    /// Stamps the code files the loaded project's snippets name, as they
    /// are now, which is when they're read: one changed before the next
    /// refresh is then a change.
    fn stamp_snippets(&mut self) {
        let snapshot = self.workspace.snapshot();
        self.code.changed(&self.root, &snapshot);
    }

    /// The code files snippets have read whose stamps differ from when each
    /// was first seen read, as changes; those seen for the first time are
    /// stamped.
    fn code_changes(&mut self, snapshot: &WorkspaceSnapshot) -> Vec<Changed> {
        self.code
            .changed(&self.root, snapshot)
            .into_iter()
            .map(|(path, present)| code_change(path, present))
            .collect()
    }

    fn reload_bounded(&mut self, bound: SourceBound<'_>) -> Result<(), LoadError> {
        let model = self.loader.load_model_bounded(&self.config, bound)?;
        self.reload(model)
    }

    fn reload(&mut self, model: ModelFile) -> Result<(), LoadError> {
        let (workspace, listing) = Kept::read(self.loader, &self.config, model, &mut self.files)?;
        self.workspace = workspace;
        self.listing = listing;
        self.code.clear();
        self.stamp_snippets();
        Ok(())
    }
}
