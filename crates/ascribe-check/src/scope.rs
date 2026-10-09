//! Reporting on part of a project: what `ascribe check <paths>` shows.
//!
//! The project is still checked whole, since links, includes, and ids need
//! it; a [`Scope`] chooses what's reported. **A diagnostic counts for a path
//! when its file, or the file of one of its related places, is that path or
//! under it.** A problem in included content is reported at the include, with
//! its place in the fragment as related information, so:
//!
//! - a page's scope shows the problems its fragments cause on it;
//! - a fragment's scope shows the problems in it. The same problem appears at
//!   every include, so these are collapsed: diagnostics with the same code
//!   and the same related places in the scope are shown once, at the first
//!   include in the list's order, and [`Reported::repeats`] says at how many
//!   other includes it also appears.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use ascribe_core::path::relative_path;
use ascribe_core::{FileId, Location, RelPath};

use crate::{Diagnostic, LocateError, MODEL_FILE, Project, SourceFile};

/// Why paths named on the command line can't be checked.
#[derive(Debug, thiserror::Error)]
pub enum ScopeError {
    /// No content model was found or named.
    #[error(transparent)]
    Locate(#[from] LocateError),
    /// A path that should exist doesn't.
    #[error("{} doesn't exist", path.display())]
    Missing {
        /// The path, as given.
        path: PathBuf,
    },
    /// No `ascribe.toml` is at or above a path.
    #[error(
        "{} isn't in an Ascribe project: there's no {MODEL_FILE} in its folder or any parent; pass --config to name one",
        path.display()
    )]
    NoProject {
        /// The path, as given.
        path: PathBuf,
    },
    /// A path is in another project than the one being checked.
    #[error(
        "{} is in the project of {}, not of {}; check each project on its own",
        path.display(),
        its.display(),
        config.display()
    )]
    OtherProject {
        /// The path, as given.
        path: PathBuf,
        /// The content model nearest the path.
        its: PathBuf,
        /// The content model being checked.
        config: PathBuf,
    },
    /// A path is neither in the project's folder nor in its content root.
    #[error("{} isn't in the project of {}", path.display(), config.display())]
    Outside {
        /// The path, as given.
        path: PathBuf,
        /// The content model being checked.
        config: PathBuf,
    },
    /// Text to check as a file was given a path that isn't a source file's.
    #[error(
        "{} can't be a source file of the project: a source file is a `.md` file in the content root, {}",
        path.display(),
        content_root
    )]
    NotASource {
        /// The path, as given.
        path: PathBuf,
        /// The content root, relative to the project's folder.
        content_root: String,
    },
}

impl ascribe_core::Coded for ScopeError {
    fn code(&self) -> &'static str {
        match self {
            ScopeError::Locate(e) => e.code(),
            ScopeError::Missing { .. } => "path_missing",
            ScopeError::NoProject { .. } => "path_not_in_a_project",
            ScopeError::OtherProject { .. } => "paths_in_two_projects",
            ScopeError::Outside { .. } => "path_outside_project",
            ScopeError::NotASource { .. } => "path_not_a_source",
        }
    }
}

/// The content model to check `paths` (relative to the current directory)
/// with: `config` when given ([`Project::locate`]), or else the nearest
/// `ascribe.toml` at or above the first path, or else at or above the
/// current directory.
///
/// # Errors
///
/// What [`Project::locate`] returns, and [`ScopeError::NoProject`] when the
/// first path has no `ascribe.toml` at or above it.
pub fn locate_for(config: Option<&Path>, paths: &[PathBuf]) -> Result<PathBuf, ScopeError> {
    match (config, paths.first()) {
        (None, Some(first)) => {
            // Made absolute, not canonical: the project's paths are joined to
            // its root, and a canonical path on Windows takes no `/`.
            let first_absolute = std::path::absolute(first).unwrap_or_else(|_| first.clone());
            Project::find_config(&first_absolute).ok_or_else(|| ScopeError::NoProject {
                path: first.clone(),
            })
        }
        _ => Ok(Project::locate(config)?),
    }
}

/// `path` made absolute and, as far as it exists, with its symbolic links and
/// `.` and `..` resolved, so two spellings of one place compare equal. What
/// doesn't exist yet is kept as written, after the part that does. For
/// comparing only: on Windows it's a `\\?\` path.
fn real(path: &Path) -> PathBuf {
    let absolute = std::path::absolute(path).unwrap_or_else(|_| path.to_owned());
    let mut existing = absolute.as_path();
    let mut rest = Vec::new();
    loop {
        // Outside FileSystem: finding where a path named on the command line
        // is, before there's a project to read it through.
        if let Ok(found) = existing.canonicalize() {
            let mut out = found;
            out.extend(rest.iter().rev());
            return out;
        }
        match (existing.parent(), existing.file_name()) {
            (Some(parent), Some(name)) => {
                rest.push(name.to_owned());
                existing = parent;
            }
            _ => return absolute,
        }
    }
}

/// The files a report covers: paths relative to the project root, each a
/// file or a directory.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Scope {
    paths: Vec<RelPath>,
}

/// A diagnostic as a scoped report shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reported {
    /// The diagnostic.
    pub diagnostic: Diagnostic,
    /// For a problem in included content that counts for the scope only
    /// through its related places: at how many other includes the same
    /// problem is reported too, collapsed into this one. `0` otherwise.
    pub repeats: usize,
}

impl Scope {
    /// A scope of `paths`, relative to the project root (the directory of
    /// `ascribe.toml`). The root itself, the empty path, is the whole
    /// project.
    pub fn new(paths: impl IntoIterator<Item = RelPath>) -> Scope {
        Scope {
            paths: paths.into_iter().collect(),
        }
    }

    /// The scope of `paths`, relative to the current directory, in `project`,
    /// whose content model is `config`. Each path must exist, unless `exist`
    /// is false (for a file checked before it's saved), and be in the
    /// project's folder or its content root, and in no other project's.
    ///
    /// # Errors
    ///
    /// The first path that doesn't exist, isn't in the project, or is in
    /// another project nested in it.
    pub fn of_paths(
        project: &Project,
        config: &Path,
        paths: &[PathBuf],
        exist: bool,
    ) -> Result<Scope, ScopeError> {
        let root = real(project.root());
        let config_real = real(config);
        let mut out = Vec::with_capacity(paths.len());
        for path in paths {
            let at = real(path);
            // Outside FileSystem: a path named on the command line, which may
            // be outside the project.
            if exist && !at.try_exists().unwrap_or(false) {
                return Err(ScopeError::Missing { path: path.clone() });
            }
            if let Some(its) = Project::find_config(&at)
                && real(&its) != config_real
            {
                return Err(ScopeError::OtherProject {
                    path: path.clone(),
                    its,
                    config: config.to_owned(),
                });
            }
            let rel = relative_path(&root, &at)
                .filter(|rel| rel.is_inside() || rel.relative_to(project.content_root()).is_some())
                .ok_or_else(|| ScopeError::Outside {
                    path: path.clone(),
                    config: config.to_owned(),
                })?;
            out.push(rel);
        }
        Ok(Scope { paths: out })
    }

    /// The content path of the source file `path` (relative to the current
    /// directory) would be, for text checked before it's saved there.
    ///
    /// # Errors
    ///
    /// What [`Scope::of_paths`] returns for it, and
    /// [`ScopeError::NotASource`] when a file there wouldn't be a source
    /// file.
    pub fn source_path(
        project: &Project,
        config: &Path,
        path: &Path,
    ) -> Result<RelPath, ScopeError> {
        let scope = Scope::of_paths(project, config, &[path.to_owned()], false)?;
        scope
            .paths
            .first()
            .and_then(|rel| rel.relative_to(project.content_root()))
            .filter(ascribe_resolve::is_source_path)
            .ok_or_else(|| ScopeError::NotASource {
                path: path.to_owned(),
                content_root: if project.content_root().is_root() {
                    "the project's folder".to_owned()
                } else {
                    format!("`{}`", project.content_root())
                },
            })
    }

    /// The paths, relative to the project root.
    pub fn paths(&self) -> &[RelPath] {
        &self.paths
    }

    /// Whether `path`, relative to the project root, is one of the paths or
    /// under one.
    pub fn contains(&self, path: &RelPath) -> bool {
        self.paths.iter().any(|p| path.relative_to(p).is_some())
    }

    /// Whether the file with this id is in the scope.
    fn contains_file(&self, project: &Project, id: FileId) -> bool {
        project
            .display_path(id)
            .and_then(|p| RelPath::parse(&p).ok())
            .is_some_and(|p| self.contains(&p))
    }

    /// The project's source files in the scope, in the project's order.
    pub fn sources<'p>(&self, project: &'p Project) -> Vec<&'p SourceFile> {
        project
            .sources()
            .iter()
            .filter(|s| self.contains_file(project, s.id))
            .collect()
    }

    /// The content paths of the source files the scope names, when each of
    /// its paths names one (and not a directory or another file): then nothing
    /// else, such as `ascribe.toml` or a code file, is in the scope.
    pub fn named_sources(&self, project: &Project) -> Option<Vec<RelPath>> {
        self.paths
            .iter()
            .map(|p| {
                let source = p.relative_to(project.content_root())?;
                project.source_at(&source).map(|_| source)
            })
            .collect()
    }

    /// The diagnostics that count for the scope, in the order given, with the
    /// repeats of a problem in included content collapsed into its first
    /// report (see the [module documentation](self)).
    pub fn report(&self, project: &Project, diagnostics: Vec<Diagnostic>) -> Vec<Reported> {
        let mut out: Vec<Reported> = Vec::new();
        let mut first: HashMap<(&'static str, Vec<Location>), usize> = HashMap::new();
        for d in diagnostics {
            let own = self.contains_file(project, d.location.file);
            let related: Vec<Location> = d
                .related
                .iter()
                .map(|r| r.location)
                .filter(|at| self.contains_file(project, at.file))
                .collect();
            if !own {
                if related.is_empty() {
                    continue;
                }
                match first.get(&(d.code, related.clone())) {
                    Some(&i) => {
                        out[i].repeats += 1;
                        continue;
                    }
                    None => {
                        first.insert((d.code, related), out.len());
                    }
                }
            }
            out.push(Reported {
                diagnostic: d,
                repeats: 0,
            });
        }
        out
    }
}

impl Reported {
    /// Diagnostics reported whole, none collapsed.
    pub fn all(diagnostics: Vec<Diagnostic>) -> Vec<Reported> {
        diagnostics
            .into_iter()
            .map(|diagnostic| Reported {
                diagnostic,
                repeats: 0,
            })
            .collect()
    }
}
