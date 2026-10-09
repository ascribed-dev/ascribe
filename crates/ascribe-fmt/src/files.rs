//! Formatting the files of a project on disk: what `ascribe fmt` does.

use std::path::{Path, PathBuf};

use ascribe_core::{SourceBoundary, TextEdit, apply_edits};
use ascribe_model::{ContentModel, MODEL_FILE};

use crate::{format, options_from_model};

/// Why [`format_files`] stopped.
#[derive(Debug, thiserror::Error)]
pub enum FormatFilesError {
    /// A path couldn't be read, or a file couldn't be written.
    #[error("{}: {source}", path.display())]
    Io {
        /// The path.
        path: PathBuf,
        /// What went wrong.
        source: std::io::Error,
    },
    /// A file isn't valid UTF-8.
    #[error("{}: not valid UTF-8", path.display())]
    NotUtf8 {
        /// The file.
        path: PathBuf,
    },
    /// The formatter's edits to a file couldn't be applied. A bug in the
    /// formatter, never in the file.
    #[error("{}: the formatter made bad edits: {message}", path.display())]
    BadEdits {
        /// The file.
        path: PathBuf,
        /// Why the edits couldn't be applied.
        message: String,
    },
}

impl ascribe_core::Coded for FormatFilesError {
    fn code(&self) -> &'static str {
        match self {
            FormatFilesError::Io { .. } => "format_io",
            FormatFilesError::NotUtf8 { .. } => "not_utf8",
            FormatFilesError::BadEdits { .. } => "format_bad_edits",
        }
    }
}

/// What [`format_files`] did.
#[derive(Debug, Default)]
pub struct Formatted {
    /// The files that changed, or with `check` would have.
    pub changed: Vec<PathBuf>,
    /// The files left alone because a symbolic link on the way to them leads
    /// out of the content root, in path order.
    pub refused: Vec<Refused>,
}

/// A file [`format_files`] didn't read, because a symbolic link on the way to
/// it leads to a file that isn't a source file of the content root.
#[derive(Debug)]
pub struct Refused {
    /// The file, as it was found.
    pub path: PathBuf,
    /// Why, as `check` words it ([`SourceBoundary::check_link`]).
    pub reason: String,
}

/// Formats every `.md` file under `paths` (each a file, or a directory
/// searched recursively) or, with no path, under the content root of the
/// project whose content model, `model`, is at `config`. Each file that
/// changes is passed to `on_changed` as it's done, with its text before and
/// the edits that format it, and all of them are returned.
///
/// With `check`, nothing is written: the files returned are the ones that
/// would change.
///
/// Directories whose names start with `.`, `node_modules`, and directories
/// inside the searched ones that hold an `ascribe.toml` other than the
/// project's own (another project, formatted under its own model) are
/// skipped. A file in the content root reached through a symbolic link that
/// leads to a file that isn't a source file of the content root, as
/// `boundary` decides, is neither read nor written, and is returned in
/// [`Formatted::refused`]. The files are formatted in path order, each once.
///
/// # Errors
///
/// A path or a file can't be read, a file isn't UTF-8, or a file can't be
/// written. The files formatted before it stay formatted.
pub fn format_files(
    config: &Path,
    model: &ContentModel,
    paths: &[PathBuf],
    check: bool,
    boundary: &dyn SourceBoundary,
    on_changed: &mut dyn FnMut(&Path, &str, &[TextEdit]),
) -> Result<Formatted, FormatFilesError> {
    let project = config
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_owned);
    let parse_options = options_from_model(model);

    // Outside FileSystem: `fmt` formats the files and folders it's given, as
    // given, and this crate can't use ascribe-resolve. `boundary` keeps a link
    // in the content root from leading out of it, as FileSystem does.
    let roots: Vec<PathBuf> = if paths.is_empty() {
        vec![project.join(&model.project.content_root)]
    } else {
        paths.to_vec()
    };
    let own = std::fs::canonicalize(&project).ok();
    let mut files = Vec::new();
    for root in &roots {
        collect(root, own.as_deref(), &mut files)?;
    }
    files.sort();
    files.dedup();

    let mut done = Formatted::default();
    for file in files {
        if let Err(reason) = boundary.check_link(&file) {
            done.refused.push(Refused { path: file, reason });
            continue;
        }
        // Outside FileSystem: as above.
        let source = std::fs::read(&file).map_err(|source| FormatFilesError::Io {
            path: file.clone(),
            source,
        })?;
        let source = String::from_utf8(source)
            .map_err(|_| FormatFilesError::NotUtf8 { path: file.clone() })?;
        let edits = format(&source, &parse_options, model);
        if edits.is_empty() {
            continue;
        }
        let formatted = apply_edits(&source, &edits).map_err(|e| FormatFilesError::BadEdits {
            path: file.clone(),
            message: e.to_string(),
        })?;
        if !check {
            std::fs::write(&file, formatted).map_err(|source| FormatFilesError::Io {
                path: file.clone(),
                source,
            })?;
        }
        on_changed(&file, &source, &edits);
        done.changed.push(file);
    }
    Ok(done)
}

/// Whether a directory holds a file named exactly `ascribe.toml` and isn't
/// `own`, this project's folder: it's another project's folder, whose files
/// that project's model formats.
fn holds_model(dir: &Path, own: Option<&Path>) -> bool {
    // Outside FileSystem: as in `format_files`.
    if own.is_some() && std::fs::canonicalize(dir).ok().as_deref() == own {
        return false;
    }
    std::fs::read_dir(dir).is_ok_and(|entries| {
        entries
            .flatten()
            .any(|e| e.file_name() == MODEL_FILE && e.path().is_file())
    })
}

/// Adds every `.md` file at or under `path` to `files`. `own` is the project's
/// folder, canonical.
fn collect(
    path: &Path,
    own: Option<&Path>,
    files: &mut Vec<PathBuf>,
) -> Result<(), FormatFilesError> {
    // Outside FileSystem: as in `format_files`.
    let io = |source| FormatFilesError::Io {
        path: path.to_owned(),
        source,
    };
    let meta = std::fs::metadata(path).map_err(io)?;
    if meta.is_file() {
        files.push(path.to_owned());
        return Ok(());
    }
    let entries = std::fs::read_dir(path).map_err(io)?;
    for entry in entries {
        let entry = entry.map_err(io)?;
        let child = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if child.is_dir() {
            if !name.starts_with('.') && name != "node_modules" && !holds_model(&child, own) {
                collect(&child, own, files)?;
            }
        } else if child.extension().is_some_and(|e| e == "md") {
            files.push(child);
        }
    }
    Ok(())
}
