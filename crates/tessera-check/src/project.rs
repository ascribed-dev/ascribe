//! The project the checks run on: the content model and the source files.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use tessera_core::{FileId, LineIndex, RelPath};
use tessera_model::ContentModel;

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

/// The result of looking for a file, with names compared exactly (SPEC §9.4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Lookup {
    /// A regular file with exactly this name exists.
    Found,
    /// No file has this name, but one differs only in letter case. Holds the
    /// path as it's really spelled, relative to the project root.
    CaseMismatch(String),
    /// There's no such file, or it isn't a regular file.
    Missing,
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
/// path order, so an id is stable for one loaded project. Source texts are
/// held in memory: the language server builds a `Project` from its open
/// buffers, and the command line from the files on disk. Files that aren't
/// sources (images, other downloads) are looked up on disk when a reference
/// needs them; a source file counts as existing even when it's only in
/// memory.
#[derive(Clone, Debug)]
pub struct Project {
    root: PathBuf,
    content_root: RelPath,
    model: ContentModel,
    model_text: String,
    sources: Vec<SourceFile>,
    model_warnings: Vec<Diagnostic>,
    /// Every source file's path relative to the project root, exactly and
    /// lowercased, so a lookup doesn't scan the sources.
    source_paths: HashMap<String, String>,
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
    /// every `.md` file under its content root.
    ///
    /// Directories and files whose names begin with `.` are skipped, and
    /// symbolic links to directories aren't followed. A source file that
    /// isn't valid UTF-8 is a [`LoadError::Read`].
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

    /// Reads every `.md` file under `root/content_root`, in path order, with
    /// ids from 1. See [`Project::load`] for what's skipped.
    pub fn read_sources(root: &Path, content_root: &RelPath) -> Result<Vec<SourceFile>, LoadError> {
        // SPEC-QUESTION(Q52): which files are sources: exactly `.md`, no
        // dot-names, UTF-8, and an unreadable one stops the command.
        let base = join(root, content_root);
        let mut paths = Vec::new();
        walk(&base, &RelPath::root(), &mut paths)?;
        paths.sort();
        let mut sources = Vec::with_capacity(paths.len());
        for (i, path) in paths.into_iter().enumerate() {
            let full = join(&base, &path);
            let text = fs::read(&full)
                .map_err(|e| read_error(&full, e))
                .and_then(|bytes| {
                    String::from_utf8(bytes).map_err(|_| LoadError::Read {
                        path: full.display().to_string(),
                        message: "the file isn't valid UTF-8".into(),
                    })
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
            .filter_map(|s| content_root.join(s.path.as_str()).ok())
            .map(|p| (p.as_str().to_lowercase(), p.to_string()))
            .collect();
        Project {
            root,
            content_root,
            model,
            model_text,
            sources,
            model_warnings,
            source_paths,
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
        &self.content_root
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

    /// Looks for a file by its path relative to the **project root**, comparing
    /// every name exactly, on every platform (SPEC §9.4). Source files held in
    /// memory count as existing; anything else is looked for on disk, one
    /// directory at a time, so `Logo.png` never finds `logo.png` even on a
    /// case-insensitive file system.
    ///
    /// A path that leaves the project root is [`Lookup::Missing`].
    pub fn lookup(&self, path: &RelPath) -> Lookup {
        if !path.is_inside() {
            return Lookup::Missing;
        }
        if let Some(found) = self.lookup_source(path) {
            return found;
        }
        let mut dir = self.root.clone();
        let mut actual: Vec<String> = Vec::new();
        let mut mismatched = false;
        let segments: Vec<&str> = path.segments().collect();
        for (i, seg) in segments.iter().enumerate() {
            let last = i + 1 == segments.len();
            let Ok(entries) = fs::read_dir(&dir) else {
                return Lookup::Missing;
            };
            let names: Vec<String> = entries
                .filter_map(Result::ok)
                .filter_map(|e| e.file_name().into_string().ok())
                .collect();
            let name = if names.iter().any(|n| n == seg) {
                (*seg).to_owned()
            } else if let Some(n) = names
                .iter()
                .find(|n| n.to_lowercase() == seg.to_lowercase())
            {
                mismatched = true;
                n.clone()
            } else {
                return Lookup::Missing;
            };
            dir.push(&name);
            actual.push(name);
            if last {
                if !dir.is_file() {
                    return Lookup::Missing;
                }
            } else if !dir.is_dir() {
                return Lookup::Missing;
            }
        }
        if segments.is_empty() {
            return Lookup::Missing;
        }
        if mismatched {
            Lookup::CaseMismatch(actual.join("/"))
        } else {
            Lookup::Found
        }
    }

    /// The in-memory source files' answer to [`Project::lookup`], if they
    /// have one.
    fn lookup_source(&self, path: &RelPath) -> Option<Lookup> {
        let actual = self.source_paths.get(&path.as_str().to_lowercase())?;
        Some(if actual == path.as_str() {
            Lookup::Found
        } else {
            Lookup::CaseMismatch(actual.clone())
        })
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

fn walk(base: &Path, dir: &RelPath, out: &mut Vec<RelPath>) -> Result<(), LoadError> {
    let full = join(base, dir);
    let entries = fs::read_dir(&full).map_err(|e| read_error(&full, e))?;
    for entry in entries {
        let entry = entry.map_err(|e| read_error(&full, e))?;
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }
        let Ok(rel) = dir.join(&name) else { continue };
        let file_type = entry
            .file_type()
            .map_err(|e| read_error(&entry.path(), e))?;
        if file_type.is_dir() {
            walk(base, &rel, out)?;
        } else if (file_type.is_file() || entry.path().is_file()) && rel.extension() == Some("md") {
            out.push(rel);
        }
    }
    Ok(())
}
