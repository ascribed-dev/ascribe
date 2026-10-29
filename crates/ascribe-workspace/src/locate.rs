//! Finding the content model a command works on.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use ascribe_core::path::{normalize, relative_path};
use ascribe_model::MODEL_FILE;

/// Why [`locate`] found no content model.
#[derive(Debug, thiserror::Error)]
pub enum LocateError {
    /// The current directory, where the search starts, can't be read.
    #[error("can't read the current directory: {0}")]
    CurrentDir(io::Error),
    /// No `ascribe.toml` is in the current directory or a parent.
    #[error("{}", not_found(dir, below))]
    NotFound {
        /// The directory the search started in.
        dir: PathBuf,
        /// The folders below it that hold an `ascribe.toml`, as
        /// [`find_below`] finds them: the first eleven, of which the message
        /// names ten.
        below: Vec<PathBuf>,
    },
    /// The content model named isn't a file.
    #[error("{} doesn't exist or isn't a file", path.display())]
    NotAFile {
        /// The path named.
        path: PathBuf,
    },
}

/// The most projects below the directory searched that
/// [`LocateError::NotFound`] names.
const NOT_FOUND_NAMED: usize = 10;

impl LocateError {
    /// No `ascribe.toml` in `dir` or a parent; the error names the projects
    /// below `dir`, if there are any.
    pub fn not_found(dir: PathBuf) -> LocateError {
        let below = find_below(&dir, NOT_FOUND_NAMED + 1);
        LocateError::NotFound { dir, below }
    }
}

/// What [`LocateError::NotFound`] says: with the projects below, where to
/// run ascribe instead, each folder from the current directory.
fn not_found(dir: &Path, below: &[PathBuf]) -> String {
    let none = format!(
        "no {MODEL_FILE} found in {} or any parent directory",
        dir.display()
    );
    // Each folder as it's typed where ascribe was run.
    let base = normalize(&std::env::current_dir().unwrap_or_else(|_| dir.to_owned()));
    let named: Vec<String> = below
        .iter()
        .take(NOT_FOUND_NAMED)
        .map(|child| match relative_path(&base, &normalize(child)) {
            Some(rel) => rel.as_str().to_owned(),
            None => child.display().to_string(),
        })
        .collect();
    match named.as_slice() {
        [] => format!("{none}; run ascribe from a project, or pass --config"),
        [one] => format!(
            "{none}; the project below it is in {one}: run ascribe there, or pass --config {one}"
        ),
        several => {
            let more = if below.len() > NOT_FOUND_NAMED {
                ", and more"
            } else {
                ""
            };
            format!(
                "{none}; the projects below it are in {}{more}: run ascribe in one of them, \
                 or pass --config with its folder",
                several.join(", ")
            )
        }
    }
}

impl ascribe_core::Coded for LocateError {
    fn code(&self) -> &'static str {
        match self {
            LocateError::CurrentDir(_) => "current_dir_unreadable",
            LocateError::NotFound { .. } => "model_not_found",
            LocateError::NotAFile { .. } => "model_not_a_file",
        }
    }
}

/// The nearest `ascribe.toml` in `start` or one of its parents.
pub fn find_config(start: &Path) -> Option<PathBuf> {
    // Outside FileSystem: looking for a project, before there is one.
    start
        .ancestors()
        .map(|dir| dir.join(MODEL_FILE))
        .find(|p| p.is_file())
}

/// The folders below `dir` that hold an `ascribe.toml`, in path order, at
/// most `limit`: a project's folder, and those below it too. Folders whose
/// names start with `.`, `node_modules`, and `target` aren't looked in.
pub fn find_below(dir: &Path, limit: usize) -> Vec<PathBuf> {
    fn walk(dir: &Path, limit: usize, found: &mut Vec<PathBuf>) {
        // Outside FileSystem: looking for projects, before there is one.
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        let mut dirs: Vec<PathBuf> = entries
            .flatten()
            .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
            .filter(|e| {
                let name = e.file_name();
                let name = name.to_string_lossy();
                !name.starts_with('.') && name != "node_modules" && name != "target"
            })
            .map(|e| e.path())
            .collect();
        dirs.sort();
        for child in dirs {
            if found.len() >= limit {
                return;
            }
            // Outside FileSystem: as above.
            if child.join(MODEL_FILE).is_file() {
                found.push(child.clone());
            }
            walk(&child, limit, found);
        }
    }
    let mut found = Vec::new();
    walk(dir, limit, &mut found);
    found
}

/// The content model a command works on: `config` when given (a file, or a
/// directory that holds `ascribe.toml`), or else the nearest `ascribe.toml`
/// in the current directory or a parent.
///
/// # Errors
///
/// The current directory can't be read, no `ascribe.toml` is found, or
/// `config` names none.
pub fn locate(config: Option<&Path>) -> Result<PathBuf, LocateError> {
    // Outside FileSystem: the path given with `--config`, or the one found,
    // before there's a project.
    let config = match config {
        Some(path) if path.is_dir() => path.join(MODEL_FILE),
        Some(path) => path.to_owned(),
        None => {
            let cwd = std::env::current_dir().map_err(LocateError::CurrentDir)?;
            find_config(&cwd).ok_or_else(|| LocateError::not_found(cwd))?
        }
    };
    if config.is_file() {
        Ok(config)
    } else {
        Err(LocateError::NotAFile { path: config })
    }
}
