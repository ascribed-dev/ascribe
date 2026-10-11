//! `ascribe rename`: what the editor's rename does, from the command line.
//! A phrase, a dimension's value, a heading's id, or a page or fragment
//! that moves, renamed everywhere it's used, through
//! [`ascribe_lsp::rename`], which makes the editor's own edits.

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ascribe_check::Project;
use ascribe_core::{Coded, RelPath};
use ascribe_lsp::{RenameError, Renamed, Renaming};
use clap::Args as ClapArgs;

use crate::answer::{self, Projects};
use crate::cli::Global;
use crate::exit;

/// Arguments of `ascribe rename`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    /// What to rename: `phrase:<key>`, `value:<value>` (a dimension's
    /// value), a heading's id (`keys.md#rotate-keys`), or a page or fragment
    /// (`keys.md`), which moves.
    ///
    /// A path is from the current directory, or from the content root.
    #[arg(value_name = "TARGET")]
    pub target: String,

    /// The new name: a phrase's key, a value, or an id. For a page or
    /// fragment, where it moves to, read as TARGET is: from the current
    /// directory, or from the content root.
    #[arg(value_name = "NEW")]
    pub new: String,
}

/// Why `ascribe rename` couldn't do its work.
#[derive(Debug, thiserror::Error)]
pub enum RenameCommandError {
    /// The target isn't one the command can read.
    #[error("can't rename `{given}`: {reason}")]
    BadTarget {
        /// The target as given.
        given: String,
        /// Why.
        reason: String,
    },
    /// A file can't be written, or the one that moves can't be moved.
    #[error("can't write {}: {message}", path.display())]
    Unwritable {
        /// The file.
        path: PathBuf,
        /// What went wrong.
        message: String,
    },
}

impl Coded for RenameCommandError {
    fn code(&self) -> &'static str {
        match self {
            RenameCommandError::BadTarget { .. } => "rename_bad_target",
            RenameCommandError::Unwritable { .. } => "rename_unwritable",
        }
    }
}

/// Runs the command. Exit codes: 0 when it renamed, 1 when what's named
/// doesn't exist, 2 when it couldn't rename: the new name can't be its name,
/// or a file can't be read or written.
pub fn run(global: &Global, args: Args) -> ExitCode {
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let stderr = io::stderr();
    let mut err = stderr.lock();
    let code = rename(global, &args, &mut out, &mut err);
    let _ = out.flush();
    exit::code(code)
}

fn rename(global: &Global, args: &Args, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let entry = entry(&args.target, args.new.trim());
    let entry_given = entry.is_some();
    // A path target finds its project from the path; an entry from the
    // current directory. `--config` says otherwise for either.
    let near = match entry {
        Some(_) => None,
        None => Some(Path::new(
            args.target
                .split_once('#')
                .map_or(args.target.as_str(), |(path, _)| path),
        )),
    };
    let config = match answer::locate(global, near) {
        Ok(config) => config,
        Err(e) => return answer::fail(err, &e),
    };
    // Something of the project's that isn't there is an answer to act on,
    // not a failure to do the work: exit code 1. A path is named as it was
    // given.
    let missing = |err: &mut dyn Write, message: &str| {
        let _ = writeln!(err, "error: {message}");
        exit::PROBLEMS
    };
    let given = format!("`{}` doesn't exist.", args.target);
    let renaming = match entry {
        Some(renaming) => renaming,
        None => match answer::FromDisk.load(&config) {
            Ok(loaded) => match path_renaming(&loaded.project, &args.target, &args.new) {
                Ok(Some(renaming)) => Ok(renaming),
                Ok(None) => return missing(err, &given),
                Err(e) => Err(e),
            },
            Err(e) => return answer::report_failure(err, &crate::context::Failure::Load(e)),
        },
    };
    let renaming = match renaming {
        Ok(renaming) => renaming,
        Err(e) => return answer::fail(err, &e),
    };
    let renamed = match ascribe_lsp::rename(&config, &renaming) {
        Ok(renamed) => renamed,
        Err(RenameError::Missing(message)) => {
            return missing(err, if entry_given { &message } else { &given });
        }
        Err(e) => return answer::fail(err, &e),
    };
    if let Err(e) = write(&renamed) {
        return answer::fail(err, &e);
    }
    answer::written(write_text(out, &renamed), err).unwrap_or(exit::OK)
}

/// The renaming `target` names when it's a content model entry,
/// `phrase:<key>` or `value:<value>`, to `new`; `None` when it isn't one.
fn entry(target: &str, new: &str) -> Option<Result<Renaming, RenameCommandError>> {
    let (prefix, name) = target.split_once(':')?;
    let name = name.trim();
    let bad = |reason: String| {
        Some(Err(RenameCommandError::BadTarget {
            given: target.to_owned(),
            reason,
        }))
    };
    match prefix {
        // `phrase:{product}` names `phrase:product`, and `{new}` is `new`.
        "phrase" if unbraced(name).is_empty() => bad("give a key after `phrase:`".to_owned()),
        "phrase" => Some(Ok(Renaming::Phrase {
            key: unbraced(name).to_owned(),
            new: unbraced(new).to_owned(),
        })),
        "value" if name.is_empty() => bad("give a dimension's value after `value:`".to_owned()),
        "value" => Some(Ok(Renaming::Value {
            value: name.to_owned(),
            new: new.to_owned(),
        })),
        _ if ascribe_query::refs::PREFIXES
            .iter()
            .any(|(p, _)| *p == prefix) =>
        {
            bad(format!(
                "a `{prefix}:` can't be renamed from the command line; rename takes `phrase:<key>`, `value:<value>`, a heading (`page.md#id`), or a page"
            ))
        }
        _ => None,
    }
}

fn unbraced(name: &str) -> &str {
    name.strip_prefix('{')
        .and_then(|n| n.strip_suffix('}'))
        .unwrap_or(name)
}

/// The renaming a path target names: a heading's id, `page.md#id`, or a
/// page or fragment that moves to `new`, read the way the target was read.
/// `None` when it names no page or fragment.
fn path_renaming(
    project: &Project,
    target: &str,
    new: &str,
) -> Result<Option<Renaming>, RenameCommandError> {
    let (path, id) = match target.split_once('#') {
        Some((path, id)) => (path, Some(id)),
        None => (target, None),
    };
    // Each way the path can be read: from the current directory, then from
    // the content root. The first that names a source wins, and the new
    // path is read the same way.
    let readings = answer::content_path_readings(project, Path::new(path));
    let Some((at, file)) = readings.into_iter().enumerate().find_map(|(at, reading)| {
        reading
            .filter(|p| project.source_at(p).is_some())
            .map(|p| (at, p))
    }) else {
        return Ok(None);
    };
    match id {
        Some(id) => Ok(Some(Renaming::Id {
            file,
            id: id.to_owned(),
            new: new.trim().trim_start_matches('#').to_owned(),
        })),
        None => {
            let to = answer::content_path_readings(project, Path::new(new))
                .into_iter()
                .nth(at)
                .flatten()
                .filter(RelPath::is_inside)
                .ok_or_else(|| RenameCommandError::BadTarget {
                    given: target.to_owned(),
                    reason: format!("`{new}` isn't a path in the content root"),
                })?;
            Ok(Some(Renaming::File { from: file, to }))
        }
    }
}

/// Writes each file the rename changes, then moves the one that moves.
fn write(renamed: &Renamed) -> Result<(), RenameCommandError> {
    let unwritable = |path: &Path, e: io::Error| RenameCommandError::Unwritable {
        path: answer::path_from_here(path).into(),
        message: e.to_string(),
    };
    for file in &renamed.files {
        std::fs::write(&file.path, &file.text).map_err(|e| unwritable(&file.path, e))?;
    }
    if let Some((from, to)) = &renamed.moved {
        if let Some(folder) = to.parent() {
            std::fs::create_dir_all(folder).map_err(|e| unwritable(folder, e))?;
        }
        std::fs::rename(from, to).map_err(|e| unwritable(to, e))?;
    }
    Ok(())
}

/// What the rename changed, a line for each file, as the command writes it.
fn write_text(out: &mut dyn Write, renamed: &Renamed) -> io::Result<()> {
    for file in &renamed.files {
        let places = if file.edits == 1 { "place" } else { "places" };
        writeln!(
            out,
            "changed {} ({} {places})",
            answer::path_from_here(&file.path),
            file.edits
        )?;
    }
    if let Some((from, to)) = &renamed.moved {
        writeln!(
            out,
            "moved {} to {}",
            answer::path_from_here(from),
            answer::path_from_here(to)
        )?;
    }
    Ok(())
}
