//! `ascribe rename`: what the editor's rename does, from the command line.
//! A phrase, a dimension's value, a heading's id, or a page or fragment
//! that moves, renamed everywhere it's used, through
//! [`ascribe_edit::rename`], which makes the editor's own edits.

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ascribe_check::Project;
use ascribe_core::path::normalize;
use ascribe_core::{Coded, FileId, RelPath};
use ascribe_edit::{RenameError, Renamed, Renaming};
use ascribe_model::{ADDRESSES_FILE, AddressFile};
use clap::Args as ClapArgs;
use serde::Serialize;

use crate::answer::{self, Format, Projects};
use crate::cli::Global;
use crate::commands::addresses::MoveJson;
use crate::exit;

/// The version of the command's JSON. It changes only when a field is
/// removed or changes meaning.
const SCHEMA_VERSION: u32 = 1;

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

    /// How to show what changed. `json` gives each changed file's whole
    /// text, the file that moves, and the entries recorded in
    /// ascribe.addresses.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
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
    let answer = match answer(&answer::FromDisk, global, args, true) {
        Ok(answer) => answer,
        Err(stopped) => {
            let _ = writeln!(err, "error: {}", stopped.message);
            return stopped.code;
        }
    };
    let result = match args.format {
        Format::Json => answer::write_json(out, &answer.json),
        Format::Text => write_text(out, &answer.renamed),
    };
    answer::written(result, err).unwrap_or(exit::OK)
}

/// A rename made: what changes, and the JSON that says so.
pub(crate) struct Answer {
    /// Each file's text after, and the file that moves.
    renamed: Renamed,
    /// What `--format json` writes.
    pub json: RenameJson,
}

/// Why a rename wasn't made: what the command writes after `error: `, and
/// its exit code.
pub(crate) struct Stopped {
    /// The message.
    pub message: String,
    /// The exit code: 1 when what's named doesn't exist, 2 otherwise.
    pub code: u8,
}

impl Stopped {
    fn failed(error: &dyn Coded) -> Stopped {
        Stopped {
            message: error.to_string(),
            code: exit::of(error),
        }
    }
}

/// Renames `args.target` to `args.new`, in the project `projects` loads,
/// and writes the files and moves the one that moves when `write`: what
/// `ascribe rename` does, and with `write` false what the MCP server's
/// `ascribe_rename` gives.
pub(crate) fn answer(
    projects: &dyn Projects,
    global: &Global,
    args: &Args,
    write: bool,
) -> Result<Answer, Stopped> {
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
        // `--config docs` names it from the current directory; the project
        // and the editor's rename work from where it is.
        Ok(config) => std::path::absolute(&config).map_or(config, |c| normalize(&c)),
        Err(e) => return Err(Stopped::failed(&e)),
    };
    // Something of the project's that isn't there is an answer to act on,
    // not a failure to do the work: exit code 1. A path is named as it was
    // given.
    let missing = |message: String| Stopped {
        message,
        code: exit::PROBLEMS,
    };
    let load = || {
        projects.load(&config).map_err(|e| {
            let failure = crate::context::Failure::Load(e);
            Stopped {
                message: answer::Stop::Load(failure).message(),
                code: exit::FAILURE,
            }
        })
    };
    let renaming = match entry {
        Some(renaming) => renaming,
        None => {
            let loaded = load()?;
            match path_renaming(&loaded.project, &args.target, &args.new) {
                Ok(Some(renaming)) => Ok(renaming),
                Ok(None) => return Err(missing(not_found(&loaded.project, &args.target))),
                Err(e) => Err(e),
            }
        }
    };
    let renaming = renaming.map_err(|e| Stopped::failed(&e))?;
    let renamed = match ascribe_edit::rename(&config, &renaming) {
        Ok(renamed) => renamed,
        Err(RenameError::Missing(message)) => {
            return Err(missing(if entry_given {
                message
            } else {
                format!("`{}` doesn't exist.", args.target)
            }));
        }
        Err(e) => return Err(Stopped::failed(&e)),
    };
    // The entries recorded are those of `ascribe.addresses` before and
    // after that differ, read before anything is written.
    let recorded = match renamed.files.iter().find(|f| is_addresses(&f.path)) {
        Some(file) => recorded(
            load()?.project.addresses_text().unwrap_or_default(),
            &file.text,
        ),
        None => Vec::new(),
    };
    if write {
        self::write(&renamed).map_err(|e| Stopped::failed(&e))?;
    }
    let json = RenameJson {
        schema_version: SCHEMA_VERSION,
        ascribe_version: env!("CARGO_PKG_VERSION"),
        files: renamed
            .files
            .iter()
            .map(|file| FileJson {
                file: answer::path_from_here(&file.path),
                text: file.text.clone(),
                edits: file.edits,
            })
            .collect(),
        moved: renamed.moved.as_ref().map(|(from, to)| MovedJson {
            from: answer::path_from_here(from),
            to: answer::path_from_here(to),
        }),
        recorded,
        written: write,
    };
    Ok(Answer { renamed, json })
}

/// The entries of `[moved]` that `after`, an `ascribe.addresses`, has and
/// `before` doesn't, or has with another value, in key order.
fn recorded(before: &str, after: &str) -> Vec<MoveJson> {
    let parse = |text: &str| AddressFile::parse(text, FileId::new(0)).ok();
    let (Some(before), Some(after)) = (parse(before), parse(after)) else {
        return Vec::new();
    };
    after
        .moved
        .into_iter()
        .filter(|(key, entry)| before.moved.get(key).is_none_or(|e| e.value != entry.value))
        .map(|(key, entry)| MoveJson {
            key,
            value: entry.value,
        })
        .collect()
}

/// Whether `path` is a project's `ascribe.addresses`.
fn is_addresses(path: &Path) -> bool {
    path.file_name() == Some(std::ffi::OsStr::new(ADDRESSES_FILE))
}

/// What `ascribe rename --format json` writes, and the MCP server's
/// `ascribe_rename` gives.
#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub(crate) struct RenameJson {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    schema_version: u32,
    /// The version of Ascribe that wrote it.
    ascribe_version: &'static str,
    /// Each file the rename changes, in path order, with its whole text
    /// after. The file that moves is under where it was, and
    /// `ascribe.addresses` is one of them when the rename records a move.
    files: Vec<FileJson>,
    /// The page or fragment that moves, when one does.
    #[serde(skip_serializing_if = "Option::is_none")]
    moved: Option<MovedJson>,
    /// The entries of `ascribe.addresses`'s `[moved]` the rename records, in
    /// key order, so the build forwards the old addresses; none when it
    /// records nothing, as for a phrase, a fragment, or a project with no
    /// `ascribe.addresses`.
    recorded: Vec<MoveJson>,
    /// Whether the files were written and the file moved: the MCP server's
    /// `ascribe_rename` writes nothing, and gives the changes to make.
    written: bool,
}

/// A file a rename changes.
#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct FileJson {
    /// The file, from the current directory.
    file: String,
    /// Its whole text after the rename.
    text: String,
    /// How many places in it change.
    edits: usize,
}

/// The file a rename moves.
#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct MovedJson {
    /// Where it is, from the current directory.
    from: String,
    /// Where it goes, from the current directory. Its folder is made when
    /// it isn't there.
    to: String,
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

/// What's said when a path target names no page or fragment: the path, and
/// the places it was looked for. A path that leaves the content root isn't
/// read from it, and from the content root itself the two are one.
fn not_found(project: &Project, target: &str) -> String {
    let path = target.split_once('#').map_or(target, |(path, _)| path);
    let here = answer::path_from_here(Path::new(path));
    let content = project.root().join(project.content_root().as_str());
    let in_content = answer::content_path_readings(project, Path::new(path))[1]
        .as_ref()
        .filter(|rel| rel.is_inside())
        .map(|rel| answer::path_from_here(&content.join(rel.as_str())))
        .filter(|there| *there != here);
    match in_content {
        Some(there) => format!(
            "`{target}` doesn't exist: there's no page or fragment at `{here}` from the current directory, or at `{there}` in the content root."
        ),
        None => format!("`{target}` doesn't exist: there's no page or fragment at `{here}`."),
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
    // `ascribe.addresses` is said last: the move it records.
    let mut recorded = false;
    for file in &renamed.files {
        if is_addresses(&file.path) {
            recorded = true;
            continue;
        }
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
    if recorded {
        writeln!(out, "recorded the move in {}", ADDRESSES_FILE)?;
    }
    Ok(())
}
