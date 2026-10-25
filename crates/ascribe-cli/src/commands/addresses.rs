//! `ascribe addresses`: the project's record of where the addresses it has
//! published went, `ascribe.addresses` (SPEC §7.10). `init` makes the file,
//! `move` records a move already made, `retire` an address removed on
//! purpose, and `list` shows the redirects and retired addresses each build
//! writes, and the entry each comes from.

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ascribe_check::Project;
use ascribe_core::{Coded, RelPath};
use ascribe_emit::site::redirects::{self, Redirects};
use ascribe_model::addresses::{Key, RETIRED_PLACEHOLDER, split_place};
use ascribe_model::{ADDRESSES_FILE, AddressFile};
use clap::{Args as ClapArgs, Subcommand};
use serde::Serialize;

use crate::answer::{self, Format};
use crate::cli::Global;
use crate::context::load_project;
use crate::exit;

/// The version of the commands' JSON. It changes only when a field is
/// removed or changes meaning.
const SCHEMA_VERSION: u32 = 1;

/// Arguments of `ascribe addresses`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

/// The subcommands of `ascribe addresses`.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Make ascribe.addresses, so moves are recorded and `ascribe check`
    /// reports an address the docs published that's lost. A file already
    /// there is left alone.
    #[command(after_help = docs_link!("cli-addresses-init"))]
    Init(InitArgs),
    /// Record that a page, or a heading's id, moved, after a move made
    /// without Ascribe, as with `git mv`: the build forwards its old
    /// addresses to the new ones.
    #[command(after_help = docs_link!("cli-addresses-move"))]
    Move(MoveArgs),
    /// Record that a page, or a heading's id, was removed on purpose, and
    /// why.
    #[command(after_help = docs_link!("cli-addresses-retire"))]
    Retire(RetireArgs),
    /// Show the redirects and retired addresses each build writes, and the
    /// entry of ascribe.addresses each comes from.
    #[command(after_help = docs_link!("cli-addresses-list"))]
    List(ListArgs),
}

/// Arguments of `ascribe addresses init`.
#[derive(Debug, ClapArgs)]
pub struct InitArgs {
    /// How to show what was done.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
}

/// Arguments of `ascribe addresses move`.
#[derive(Debug, ClapArgs)]
pub struct MoveArgs {
    /// Where it was: a page's path, from the current folder or from the
    /// content root; a file's path, `#`, and a heading's id; or an address
    /// from the host's root, starting with `/`, from before the project used
    /// Ascribe.
    #[arg(value_name = "OLD")]
    pub old: String,

    /// Where it is now: a page's path, from the current folder or from the
    /// content root, and optionally `#` and a heading's id.
    #[arg(value_name = "NEW")]
    pub new: String,

    /// How to show what was recorded.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
}

/// Arguments of `ascribe addresses retire`.
#[derive(Debug, ClapArgs)]
pub struct RetireArgs {
    /// What was removed: a page's path, from the current folder or from the
    /// content root; a file's path, `#`, and a heading's id; or an address
    /// from the host's root, starting with `/`.
    #[arg(value_name = "OLD")]
    pub old: String,

    /// Why it was removed.
    #[arg(long, value_name = "TEXT")]
    pub reason: String,

    /// How to show what was recorded.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
}

/// Arguments of `ascribe addresses list`.
#[derive(Debug, ClapArgs)]
pub struct ListArgs {
    /// Show only this build. By default, every build in `ascribe.toml`.
    #[arg(long, value_name = "NAME")]
    pub build: Option<String>,

    /// How to show the redirects.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
}

/// Why an addresses command couldn't do its work.
#[derive(Debug, thiserror::Error)]
pub enum AddressesError {
    /// The project has no `ascribe.addresses` to record in.
    #[error("there's no {ADDRESSES_FILE}; run `ascribe addresses init` to make one")]
    NoFile,
    /// `ascribe.addresses` can't be read.
    #[error("{ADDRESSES_FILE} can't be read: {0}; run `ascribe check` to see where")]
    Invalid(String),
    /// What was given isn't something the file records.
    #[error(
        "`{0}` isn't a page's path, a file's path with `#` and a heading's id, or an address starting with `/`"
    )]
    NotAKey(String),
    /// Where it moved isn't a page or a heading of the project.
    #[error(
        "`{0}` isn't a page of the project: give the page it's on now, as a path from here or from the content root"
    )]
    NotAPage(String),
    /// Where it moved has no heading with that id.
    #[error("`{path}` has no heading with the id `{id}`")]
    NoHeading {
        /// The page.
        path: String,
        /// The id.
        id: String,
    },
    /// What was given is still a file of the project.
    #[error(
        "`{0}` is still a file of the project: move it first, or move it with `ascribe rename`, which records the move"
    )]
    StillThere(String),
    /// The reason given to `retire` is empty, or the placeholder.
    #[error("give --reason a sentence that says why it was removed")]
    NoReason,
    /// A build named isn't one of the content model's.
    #[error(transparent)]
    UnknownBuild(#[from] ascribe_check::UnknownBuild),
    /// `ascribe.addresses` can't be written.
    #[error("can't write {ADDRESSES_FILE}: {0}")]
    Unwritable(String),
}

impl Coded for AddressesError {
    fn code(&self) -> &'static str {
        match self {
            AddressesError::NoFile => "addresses_file_missing",
            AddressesError::Invalid(_) => "addresses_file_invalid",
            AddressesError::NotAKey(_) => "addresses_key_invalid",
            AddressesError::NotAPage(_) => "addresses_target_not_a_page",
            AddressesError::NoHeading { .. } => "addresses_target_no_heading",
            AddressesError::StillThere(_) => "addresses_source_still_there",
            AddressesError::NoReason => "retire_reason_missing",
            AddressesError::UnknownBuild(e) => e.code(),
            AddressesError::Unwritable(_) => "addresses_file_unwritable",
        }
    }
}

/// What `ascribe addresses init --format json` writes.
#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub(crate) struct InitJson {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    schema_version: u32,
    /// The version of Ascribe that wrote it.
    ascribe_version: &'static str,
    /// The file, from the project's folder: `ascribe.addresses`.
    file: &'static str,
    /// Whether the project had no file, so this made one. A file already
    /// there is left as it is.
    created: bool,
}

/// What `ascribe addresses move` and `retire` write with `--format json`.
#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub(crate) struct RecordJson {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    schema_version: u32,
    /// The version of Ascribe that wrote it.
    ascribe_version: &'static str,
    /// The file, from the project's folder: `ascribe.addresses`.
    file: &'static str,
    /// The table the entry is in: `moved` or `retired`.
    table: &'static str,
    /// The entry's key: a page's path from the content root, a file's path
    /// with `#` and an id, or an address starting with `/`.
    key: String,
    /// The entry's value: where it moved, or why it was retired.
    value: String,
    /// Whether the file changed: an entry already the same changes nothing.
    changed: bool,
}

/// What `ascribe addresses list --format json` writes.
#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub(crate) struct ListJson {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    schema_version: u32,
    /// The version of Ascribe that wrote it.
    ascribe_version: &'static str,
    /// Whether the project has `ascribe.addresses`. Without it, every
    /// build's lists are empty.
    has_file: bool,
    /// Each build's redirects and retired addresses, as its
    /// `_ascribe/redirects.json` has them, in the content model's order.
    builds: Vec<Redirects>,
}

/// Runs the command.
pub fn run(global: &Global, args: Args) -> ExitCode {
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let stderr = io::stderr();
    let mut err = stderr.lock();
    let code = match args.command {
        Command::Init(args) => run_init(global, &args, &mut out, &mut err),
        Command::Move(args) => run_move(global, &args, &mut out, &mut err),
        Command::Retire(args) => run_retire(global, &args, &mut out, &mut err),
        Command::List(args) => run_list(global, &args, &mut out, &mut err),
    };
    let _ = out.flush();
    exit::code(code)
}

/// Exit codes: 0 when the file is there, made now or before; 2 when the
/// project doesn't load or the file can't be written.
fn run_init(global: &Global, args: &InitArgs, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let project = match load_project(global) {
        Ok(project) => project,
        Err(failure) => return answer::report_failure(err, &failure),
    };
    let created = project.addresses_text().is_none();
    if created && let Err(e) = write_file(project.root(), &AddressFile::default().to_toml()) {
        return answer::fail(err, &e);
    }
    let result = match args.format {
        Format::Json => answer::write_json(
            out,
            &InitJson {
                schema_version: SCHEMA_VERSION,
                ascribe_version: env!("CARGO_PKG_VERSION"),
                file: ADDRESSES_FILE,
                created,
            },
        ),
        Format::Text if created => writeln!(
            out,
            "Wrote {ADDRESSES_FILE}. Moves are recorded from now on."
        ),
        Format::Text => writeln!(
            out,
            "{ADDRESSES_FILE} is already here, so it's left as it is."
        ),
    };
    answer::written(result, err).unwrap_or(exit::OK)
}

/// Exit codes: 0 when the move is recorded; 2 when the project doesn't
/// load, it has no file, or what's given can't be recorded.
fn run_move(global: &Global, args: &MoveArgs, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let project = match load_project(global) {
        Ok(project) => project,
        Err(failure) => return answer::report_failure(err, &failure),
    };
    let recorded = (|| {
        let file = existing(&project)?;
        let old = old_key(&project, &args.old)?;
        let new = new_place(&project, &args.new)?;
        let mut changed = file.clone();
        changed.record_move(&old, &new);
        Ok::<_, AddressesError>((file, changed, old, new))
    })();
    let (before, after, old, new) = match recorded {
        Ok(recorded) => recorded,
        Err(e) => return answer::fail(err, &e),
    };
    let changed = before.to_toml() != after.to_toml();
    if changed && let Err(e) = write_file(project.root(), &after.to_toml()) {
        return answer::fail(err, &e);
    }
    let result = match args.format {
        Format::Json => answer::write_json(
            out,
            &RecordJson {
                schema_version: SCHEMA_VERSION,
                ascribe_version: env!("CARGO_PKG_VERSION"),
                file: ADDRESSES_FILE,
                table: "moved",
                key: old,
                value: new,
                changed,
            },
        ),
        Format::Text => writeln!(out, "Recorded in {ADDRESSES_FILE}: {old} moved to {new}."),
    };
    answer::written(result, err).unwrap_or(exit::OK)
}

/// Exit codes: 0 when the retirement is recorded; 2 when the project
/// doesn't load, it has no file, the reason is empty, or what's given can't
/// be recorded.
fn run_retire(global: &Global, args: &RetireArgs, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let project = match load_project(global) {
        Ok(project) => project,
        Err(failure) => return answer::report_failure(err, &failure),
    };
    let reason = args.reason.trim();
    let recorded = (|| {
        if reason.is_empty() || reason == RETIRED_PLACEHOLDER {
            return Err(AddressesError::NoReason);
        }
        let file = existing(&project)?;
        let old = old_key(&project, &args.old)?;
        let mut changed = file.clone();
        changed.record_retired(&old, reason);
        Ok((file, changed, old))
    })();
    let (before, after, old) = match recorded {
        Ok(recorded) => recorded,
        Err(e) => return answer::fail(err, &e),
    };
    let changed = before.to_toml() != after.to_toml();
    if changed && let Err(e) = write_file(project.root(), &after.to_toml()) {
        return answer::fail(err, &e);
    }
    let result = match args.format {
        Format::Json => answer::write_json(
            out,
            &RecordJson {
                schema_version: SCHEMA_VERSION,
                ascribe_version: env!("CARGO_PKG_VERSION"),
                file: ADDRESSES_FILE,
                table: "retired",
                key: old,
                value: reason.to_owned(),
                changed,
            },
        ),
        Format::Text => writeln!(out, "Recorded in {ADDRESSES_FILE}: {old} was retired."),
    };
    answer::written(result, err).unwrap_or(exit::OK)
}

/// Exit codes: 0 whatever the builds write; 2 when the project doesn't load,
/// a build named isn't one, or the file can't be read.
fn run_list(global: &Global, args: &ListArgs, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let project = match load_project(global) {
        Ok(project) => project,
        Err(failure) => return answer::report_failure(err, &failure),
    };
    if let Some(Err(issues)) = project.addresses() {
        return answer::fail(err, &AddressesError::Invalid(detail(issues)));
    }
    let named: Vec<String> = args.build.iter().cloned().collect();
    let builds = match ascribe_check::select_builds(&project, &named) {
        Ok(builds) => builds,
        Err(e) => return answer::fail(err, &AddressesError::from(e)),
    };
    let index = project.index();
    let listed: Vec<Redirects> = builds
        .iter()
        .map(|build| redirects::of_build(&index, build))
        .collect();
    let has_file = project.addresses().is_some();
    let result = match args.format {
        Format::Json => answer::write_json(
            out,
            &ListJson {
                schema_version: SCHEMA_VERSION,
                ascribe_version: env!("CARGO_PKG_VERSION"),
                has_file,
                builds: listed,
            },
        ),
        Format::Text => write_list(out, has_file, &listed),
    };
    answer::written(result, err).unwrap_or(exit::OK)
}

fn write_list(out: &mut dyn Write, has_file: bool, builds: &[Redirects]) -> io::Result<()> {
    if !has_file {
        return writeln!(
            out,
            "No {ADDRESSES_FILE}: `ascribe addresses init` makes one, and then moves are recorded and the build writes their redirects."
        );
    }
    if builds
        .iter()
        .all(|b| b.redirects.is_empty() && b.gone.is_empty())
    {
        return writeln!(out, "No build writes a redirect or a retired address.");
    }
    for (i, build) in builds.iter().enumerate() {
        if i > 0 {
            writeln!(out)?;
        }
        writeln!(out, "{}:", build.build)?;
        if build.redirects.is_empty() && build.gone.is_empty() {
            writeln!(out, "  nothing")?;
        }
        for r in &build.redirects {
            writeln!(out, "  {} -> {} ({}, {})", r.from, r.to, r.status, r.entry)?;
        }
        for g in &build.gone {
            writeln!(out, "  {} retired ({}): {}", g.path, g.entry, g.reason)?;
        }
    }
    Ok(())
}

/// The project's `ascribe.addresses`, read.
fn existing(project: &Project) -> Result<AddressFile, AddressesError> {
    match project.addresses() {
        None => Err(AddressesError::NoFile),
        Some(Err(issues)) => Err(AddressesError::Invalid(detail(issues))),
        Some(Ok(file)) => Ok(file.clone()),
    }
}

/// The first problem's detail, for a message.
fn detail(issues: &[ascribe_core::Issue]) -> String {
    issues
        .first()
        .and_then(|i| i.arg("detail"))
        .unwrap_or("it doesn't read")
        .to_owned()
}

/// What was moved or retired, as a key of the file: an address as given,
/// or a path read from the current folder, or else from the content root,
/// with its `#` and id. Refused while the file is still a source.
fn old_key(project: &Project, given: &str) -> Result<String, AddressesError> {
    if given.starts_with('/') {
        return Key::parse(given)
            .map(|_| given.to_owned())
            .ok_or_else(|| AddressesError::NotAKey(given.to_owned()));
    }
    let (path, id) = given
        .split_once('#')
        .map_or((given, None), |(p, id)| (p, Some(id)));
    let [from_cwd, as_content] = answer::content_path_readings(project, Path::new(path));
    let path = from_cwd
        .or(as_content)
        .ok_or_else(|| AddressesError::NotAKey(given.to_owned()))?;
    let key = match id {
        Some(id) => format!("{path}#{id}"),
        None => path.to_string(),
    };
    if Key::parse(&key).is_none() {
        return Err(AddressesError::NotAKey(given.to_owned()));
    }
    if id.is_none() && project.source_at(&path).is_some() {
        return Err(AddressesError::StillThere(path.to_string()));
    }
    Ok(key)
}

/// Where it moved, as a value of the file: a page of the project, read
/// from the current folder or from the content root, and the heading with
/// the id given.
fn new_place(project: &Project, given: &str) -> Result<String, AddressesError> {
    let (path, id) = given
        .split_once('#')
        .map_or((given, None), |(p, id)| (p, Some(id)));
    let page: RelPath = answer::content_paths(project, Path::new(path))
        .into_iter()
        .find(|p| project.source_at(p).is_some())
        .ok_or_else(|| AddressesError::NotAPage(given.to_owned()))?;
    let index = project.index();
    let is_page = index
        .file(&page)
        .is_some_and(|f| f.kind == ascribe_resolve::FileKind::Page);
    if !is_page {
        return Err(AddressesError::NotAPage(given.to_owned()));
    }
    let place = match id {
        Some(id) => {
            if index.page_heading(&page, id).is_none() {
                return Err(AddressesError::NoHeading {
                    path: page.to_string(),
                    id: id.to_owned(),
                });
            }
            format!("{page}#{id}")
        }
        None => page.to_string(),
    };
    if split_place(&place).is_none() {
        return Err(AddressesError::NotAPage(given.to_owned()));
    }
    Ok(place)
}

fn write_file(root: &Path, text: &str) -> Result<(), AddressesError> {
    let path: PathBuf = root.join(ADDRESSES_FILE);
    // Outside FileSystem: writing the project's own file beside
    // ascribe.toml, as `ascribe permalinks sync` writes its list.
    if ascribe_resolve::is_link(&path) {
        return Err(AddressesError::Unwritable(
            "it's a link; replace it with a file, or remove it".to_owned(),
        ));
    }
    std::fs::write(&path, text).map_err(|e| AddressesError::Unwritable(e.to_string()))
}
