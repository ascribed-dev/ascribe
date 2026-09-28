//! The project the checks run on: the content model and the source files.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use tessera_core::{FileId, LineIndex, RelPath};
use tessera_model::ContentModel;
use tessera_resolve::{DiskFs, FileSystem, Layout, SourceSet};

use crate::Diagnostic;

/// The content model's file name, at the project root.
pub const MODEL_FILE: &str = "tessera.toml";

/// One source file: a Markdown file under the content root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceFile {
    /// The file's id, used in every location that points into it.
    pub id: FileId,
    /// The path relative to the content root, `/`-separated.
    pub path: RelPath,
    /// The text.
    pub text: String,
}

/// What a file id names, for reporting.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileEntry<'a> {
    /// The id.
    pub id: FileId,
    /// The path relative to the project root, `/`-separated: how a person
    /// would type it. `tessera.toml` for the content model.
    pub display_path: String,
    /// The text.
    pub text: &'a str,
    /// The path relative to the content root, for a source file; `None` for
    /// the content model.
    pub content_path: Option<&'a RelPath>,
}

/// Why a project couldn't be loaded.
#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    /// A file or directory couldn't be read.
    #[error("can't read {path}: {message}")]
    Read {
        /// The path.
        path: String,
        /// What went wrong.
        message: String,
    },
    /// The content model has errors.
    #[error("{} has errors", MODEL_FILE)]
    Model {
        /// The content model's text, for showing snippets.
        text: String,
        /// The problems found, with the warnings found before them, in
        /// source order. Every location is in file [`FileId::new(0)`](FileId).
        diagnostics: Vec<Diagnostic>,
    },
}

/// A documentation set: its content model and source files.
///
/// The content model is file id 0, and the source files have ids 1, 2, … in
/// path order, so an id is stable for one loaded project (the same numbering
/// as `tessera_resolve::Project`). Source texts are held in memory: the
/// language server builds a `Project` from its open buffers, and the command
/// line from the files on disk. Files that aren't sources (images, other
/// downloads) are looked for on disk, through `tessera_resolve`'s
/// [`FileSystem`], when a reference needs them; a source file counts as
/// existing even when it's only in memory.
///
/// Which files are sources, and every rule about what a reference names, are
/// `tessera_resolve`'s, so this crate and the source index can't disagree.
#[derive(Clone, Debug)]
pub struct Project {
    root: PathBuf,
    layout: Layout,
    model: ContentModel,
    model_text: String,
    sources: Vec<SourceFile>,
    model_warnings: Vec<Diagnostic>,
    /// Every source file's content path, lowercased, to its real spelling,
    /// for [`SourceSet`].
    source_paths: HashMap<String, RelPath>,
    disk: DiskFs,
}

impl Project {
    /// The nearest `tessera.toml` in `start` or one of its parents.
    pub fn find_config(start: &Path) -> Option<PathBuf> {
        start
            .ancestors()
            .map(|dir| dir.join(MODEL_FILE))
            .find(|p| p.is_file())
    }

    /// Loads the project whose content model is at `config`: the model, then
    /// every source file under its content root, as
    /// [`tessera_resolve::FileSystem::sources`] finds them.
    ///
    /// A source file or directory that can't be read, or a source file that
    /// isn't valid UTF-8, is a [`LoadError::Read`].
    pub fn load(config: &Path) -> Result<Project, LoadError> {
        let text = fs::read(config)
            .map_err(|e| read_error(config, e))
            .and_then(|bytes| {
                String::from_utf8(bytes).map_err(|_| LoadError::Read {
                    path: config.display().to_string(),
                    message: "the file isn't valid UTF-8".into(),
                })
            })?;
        let root = match config.parent() {
            Some(p) if !p.as_os_str().is_empty() => p.to_owned(),
            _ => PathBuf::from("."),
        };
        let model = match tessera_model::load_str_in(&text, FileId::new(0), &root) {
            Ok(model) => model,
            Err(issues) => {
                return Err(LoadError::Model {
                    text,
                    diagnostics: issues.iter().map(Diagnostic::from_issue).collect(),
                });
            }
        };
        let content_root =
            RelPath::parse(&model.project.content_root).map_err(|e| LoadError::Read {
                path: config.display().to_string(),
                message: e.to_string(),
            })?;
        let sources = Project::read_sources(&root, &content_root)?;
        Ok(Project::from_parts(
            root,
            content_root,
            model,
            text,
            sources,
        ))
    }

    /// Reads every source file under `root/content_root`, in path order, with
    /// ids from 1. See [`Project::load`].
    pub fn read_sources(root: &Path, content_root: &RelPath) -> Result<Vec<SourceFile>, LoadError> {
        // SPEC-QUESTION(Q52): an unreadable source stops the command.
        let layout = Layout {
            content_root: content_root.clone(),
            output_dir: RelPath::root(),
        };
        let disk = DiskFs::new(root, &layout);
        let found = disk.sources();
        if let Some(bad) = found.unreadable.first() {
            return Err(LoadError::Read {
                path: join(&join(root, content_root), &bad.path)
                    .display()
                    .to_string(),
                message: bad.reason.clone(),
            });
        }
        let mut sources = Vec::with_capacity(found.paths.len());
        for (i, path) in found.paths.into_iter().enumerate() {
            let text = disk.read(&path).map_err(|e| {
                let full = join(&join(root, content_root), &path);
                if e.kind() == io::ErrorKind::InvalidData {
                    LoadError::Read {
                        path: full.display().to_string(),
                        message: "the file isn't valid UTF-8".into(),
                    }
                } else {
                    read_error(&full, e)
                }
            })?;
            sources.push(SourceFile {
                id: FileId::new(i as u32 + 1),
                path,
                text,
            });
        }
        Ok(sources)
    }

    /// A project from parts the caller already has: the language server's
    /// buffers, or a test.
    ///
    /// `root` is the project root (the directory of `tessera.toml`),
    /// `content_root` is relative to it, and `model_text` is the content
    /// model's text (for showing snippets). `sources` should have ids from 1;
    /// [`Project::from_sources`] assigns them.
    pub fn from_parts(
        root: PathBuf,
        content_root: RelPath,
        model: ContentModel,
        model_text: String,
        sources: Vec<SourceFile>,
    ) -> Project {
        let model_warnings = model.warnings.iter().map(Diagnostic::from_issue).collect();
        let source_paths = sources
            .iter()
            .map(|s| (s.path.as_str().to_lowercase(), s.path.clone()))
            .collect();
        let layout = Layout {
            content_root,
            output_dir: RelPath::parse(&model.project.output_dir).unwrap_or_default(),
        };
        let disk = DiskFs::new(&root, &layout);
        Project {
            root,
            layout,
            model,
            model_text,
            sources,
            model_warnings,
            source_paths,
            disk,
        }
    }

    /// Source files from `(path, text)` pairs, sorted by path, with ids from 1.
    pub fn from_sources(files: impl IntoIterator<Item = (RelPath, String)>) -> Vec<SourceFile> {
        let mut files: Vec<_> = files.into_iter().collect();
        files.sort_by(|a, b| a.0.cmp(&b.0));
        files
            .into_iter()
            .enumerate()
            .map(|(i, (path, text))| SourceFile {
                id: FileId::new(i as u32 + 1),
                path,
                text,
            })
            .collect()
    }

    /// The project root: the directory containing `tessera.toml`.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The content root, relative to the project root.
    pub fn content_root(&self) -> &RelPath {
        &self.layout.content_root
    }

    /// Where the content root and the output directory are.
    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    /// The files on disk, for what isn't a source file held in memory.
    pub fn file_system(&self) -> &dyn FileSystem {
        &self.disk
    }

    /// The content model.
    pub fn model(&self) -> &ContentModel {
        &self.model
    }

    /// The source files, in path order.
    pub fn sources(&self) -> &[SourceFile] {
        &self.sources
    }

    /// The diagnostics for the content model's warnings.
    pub fn model_warnings(&self) -> &[Diagnostic] {
        &self.model_warnings
    }

    /// A source file by content path.
    pub fn source_at(&self, path: &RelPath) -> Option<&SourceFile> {
        self.sources.iter().find(|s| &s.path == path)
    }

    /// What a file id names.
    pub fn file(&self, id: FileId) -> Option<FileEntry<'_>> {
        if id == FileId::new(0) {
            return Some(FileEntry {
                id,
                display_path: MODEL_FILE.to_owned(),
                text: &self.model_text,
                content_path: None,
            });
        }
        let source = self.sources.iter().find(|s| s.id == id)?;
        Some(FileEntry {
            id,
            display_path: self
                .layout
                .content_root
                .join(source.path.as_str())
                .map_or_else(|_| source.path.to_string(), |p| p.to_string()),
            text: &source.text,
            content_path: Some(&source.path),
        })
    }

    /// A line index for a file's text, for turning spans into lines and columns.
    pub fn line_index(&self, id: FileId) -> Option<LineIndex> {
        self.file(id).map(|f| LineIndex::new(f.text))
    }
}

impl SourceSet for Project {
    fn contains(&self, path: &RelPath) -> bool {
        self.source_paths
            .get(&path.as_str().to_lowercase())
            .is_some_and(|actual| actual == path)
    }

    fn case_twin(&self, path: &RelPath) -> Option<RelPath> {
        self.source_paths
            .get(&path.as_str().to_lowercase())
            .filter(|actual| *actual != path)
            .cloned()
    }
}

fn join(base: &Path, rel: &RelPath) -> PathBuf {
    let mut out = base.to_owned();
    for seg in rel.segments() {
        out.push(seg);
    }
    out
}

fn read_error(path: &Path, e: std::io::Error) -> LoadError {
    LoadError::Read {
        path: path.display().to_string(),
        message: e.to_string(),
    }
}
