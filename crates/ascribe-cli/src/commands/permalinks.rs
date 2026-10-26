//! `ascribe permalinks`: the project's list of permalinks,
//! `ascribe.permalinks` (SPEC §7.5), and retiring them (§7.6). `sync`
//! brings the list up to date, `list` shows every permalink and whether the
//! list has it, `verify` checks names against a list alone, as a product's
//! CI does, and `retire` ends permalinks on purpose, with a reason, or
//! moves them to a page, in `ascribe.addresses`.

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ascribe_core::Coded;
use ascribe_model::{
    ADDRESSES_FILE, AddressFile, PERMALINKS_FILE, is_retired_placeholder, permalink_key,
};
use ascribe_query::permalinks::{ListState, ListedAs};
use ascribe_query::{PermalinksAnswer, Retire, RetireAnswer, SyncAnswer, VerifyAnswer};
use clap::{Args as ClapArgs, Subcommand};

use crate::answer::{self, Format};
use crate::cli::Global;
use crate::context::load_project;
use crate::exit;

/// Arguments of `ascribe permalinks`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

/// The subcommands of `ascribe permalinks`.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Bring ascribe.permalinks up to date with the permalinks the content
    /// declares, making it when there's none. A name that's gone from the
    /// content stays.
    #[command(after_help = docs_link!("cli-permalinks-sync"))]
    Sync(SyncArgs),
    /// Show every permalink the content declares, where, and whether
    /// ascribe.permalinks has it.
    #[command(after_help = docs_link!("cli-permalinks-list"))]
    List(ListArgs),
    /// Check that names are permalinks in ascribe.permalinks. Reads only the
    /// list, so a product's repository can check its links against a copy.
    #[command(after_help = docs_link!("cli-permalinks-verify"))]
    Verify(VerifyArgs),
    /// Retire permalinks in ascribe.addresses, as `go:<name>`: names gone
    /// from the content, every permalink on a page, or every one under a
    /// folder. Their addresses show --reason, or lead to --to. With
    /// ascribe.permalinks, records them there too.
    #[command(after_help = docs_link!("cli-permalinks-retire"))]
    Retire(RetireArgs),
}

/// Arguments of `ascribe permalinks sync`.
#[derive(Debug, ClapArgs)]
pub struct SyncArgs {
    /// Write nothing, and exit with 1 when the list would change: for CI.
    #[arg(long)]
    pub check: bool,

    /// How to show what changed.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
}

/// Arguments of `ascribe permalinks list`.
#[derive(Debug, ClapArgs)]
pub struct ListArgs {
    /// How to show the permalinks.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
}

/// Arguments of `ascribe permalinks verify`.
#[derive(Debug, ClapArgs)]
pub struct VerifyArgs {
    /// The names to check.
    #[arg(value_name = "NAME", required_unless_present = "from")]
    pub names: Vec<String>,

    /// Also check the names in this file, one on each line. Blank lines and
    /// lines that start with `#` are skipped.
    #[arg(long, value_name = "FILE")]
    pub from: Option<PathBuf>,

    /// The list to check against. By default, the ascribe.permalinks beside
    /// the project's ascribe.toml.
    #[arg(long, value_name = "PATH")]
    pub against: Option<PathBuf>,

    /// How to show the answer.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
}

/// Arguments of `ascribe permalinks retire`.
#[derive(Debug, ClapArgs)]
pub struct RetireArgs {
    /// Names to retire: permalinks the content no longer declares, which
    /// ascribe.permalinks has.
    #[arg(value_name = "NAME", required_unless_present_any = ["page", "under"])]
    pub names: Vec<String>,

    /// Why they were retired, for a reader whose link leads there.
    #[arg(
        long,
        value_name = "TEXT",
        required_unless_present = "to",
        conflicts_with = "to"
    )]
    pub reason: Option<String>,

    /// Where their addresses lead now, in place of a reason: a page, by its
    /// path from the current folder or from the content root, and `#` and a
    /// heading's id; or another permalink, `go:<name>`.
    #[arg(long, value_name = "PAGE")]
    pub to: Option<String>,

    /// Retire every permalink this page declares, or that ascribe.permalinks
    /// has on it: a path from the current folder, or from the content root.
    #[arg(long, value_name = "PAGE", conflicts_with = "under")]
    pub page: Option<String>,

    /// Retire every permalink declared in a file under this folder, or that
    /// ascribe.permalinks has there: a path from the current folder, or from
    /// the content root.
    #[arg(long, value_name = "FOLDER")]
    pub under: Option<String>,

    /// How to show what was retired.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
}

/// Why a permalinks command couldn't do its work.
#[derive(Debug, thiserror::Error)]
pub enum PermalinksError {
    /// The list to check against isn't there.
    #[error("{} doesn't exist; pass the list with --against, or run `ascribe permalinks sync` in the docs to make one", .0.display())]
    ListNotFound(PathBuf),
    /// A file can't be read: the list, or the names.
    #[error("can't read {}: {message}", path.display())]
    Unreadable {
        /// The file.
        path: PathBuf,
        /// What went wrong.
        message: String,
    },
    /// The list can't be written.
    #[error("can't write {}: {message}", path.display())]
    Unwritable {
        /// The file.
        path: PathBuf,
        /// What went wrong.
        message: String,
    },
    /// The reason given to `retire` is empty, or the placeholder.
    #[error(
        "give --reason a sentence that says why they were retired: a reader whose link leads there sees it"
    )]
    NoReason,
    /// `ascribe.addresses` can't be read or written.
    #[error(transparent)]
    Addresses(#[from] crate::commands::addresses::AddressesError),
}

impl Coded for PermalinksError {
    fn code(&self) -> &'static str {
        match self {
            PermalinksError::ListNotFound(_) => "permalinks_list_not_found",
            PermalinksError::Unreadable { .. } => "permalinks_file_unreadable",
            PermalinksError::Unwritable { .. } => "permalinks_list_unwritable",
            PermalinksError::NoReason => "retire_reason_missing",
            PermalinksError::Addresses(e) => e.code(),
        }
    }
}

/// Runs the command.
pub fn run(global: &Global, args: Args) -> ExitCode {
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let stderr = io::stderr();
    let mut err = stderr.lock();
    let code = match args.command {
        Command::Sync(args) => run_sync(global, &args, &mut out, &mut err),
        Command::List(args) => run_list(global, &args, &mut out, &mut err),
        Command::Verify(args) => run_verify(global, &args, &mut out, &mut err),
        Command::Retire(args) => run_retire(global, &args, &mut out, &mut err),
    };
    let _ = out.flush();
    exit::code(code)
}

/// Exit codes: 0 when the list is up to date, whether or not it changed; 1
/// with `--check` when it would change; 2 when the project doesn't load, or
/// the list can't be read or written.
fn run_sync(global: &Global, args: &SyncArgs, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let project = match load_project(global) {
        Ok(project) => project,
        Err(failure) => return answer::report_failure(err, &failure),
    };
    let synced = match ascribe_query::permalinks::sync(
        &project.index(),
        project.permalinks(),
        project.permalinks_text(),
    ) {
        Ok(synced) => synced,
        Err(e) => return answer::fail(err, &e),
    };
    if synced.changed
        && !args.check
        && let Err(error) = write_permalinks(project.root(), &synced.text)
    {
        return answer::fail(err, &error);
    }
    let result = match args.format {
        Format::Json => answer::write_json(out, &synced),
        Format::Text if args.check => write_sync_check(out, err, &synced),
        Format::Text => write_sync(out, err, &synced),
    };
    if let Some(code) = answer::written(result, err) {
        return code;
    }
    if args.check && synced.changed {
        exit::PROBLEMS
    } else {
        exit::OK
    }
}

fn write_sync(out: &mut dyn Write, err: &mut dyn Write, s: &SyncAnswer) -> io::Result<()> {
    if s.created {
        let count = s.added.len();
        let permalinks = if count == 1 {
            "permalink"
        } else {
            "permalinks"
        };
        return writeln!(
            out,
            "wrote {PERMALINKS_FILE}, with {count} {permalinks}; commit it with the docs"
        );
    }
    for name in &s.added {
        writeln!(out, "added {name}")?;
    }
    for moved in &s.moved {
        writeln!(out, "moved {}: {} to {}", moved.name, moved.from, moved.to)?;
    }
    for name in &s.aliased {
        writeln!(out, "recorded the alias {name}")?;
    }
    for name in &s.retired {
        writeln!(out, "recorded {name} as retired")?;
    }
    if s.changed {
        writeln!(out, "wrote {PERMALINKS_FILE}")?;
    } else {
        writeln!(out, "{PERMALINKS_FILE} is up to date.")?;
    }
    for name in &s.removed {
        writeln!(
            err,
            "warning: kept {name}, which the project no longer has; `ascribe check` reports it until it's back or retired"
        )?;
    }
    Ok(())
}

/// What `sync --check` says: what `sync` would write, without writing it.
fn write_sync_check(out: &mut dyn Write, err: &mut dyn Write, s: &SyncAnswer) -> io::Result<()> {
    if s.created {
        return writeln!(
            err,
            "error: there's no {PERMALINKS_FILE}; run `ascribe permalinks sync` to make one"
        );
    }
    for name in &s.added {
        writeln!(out, "would add {name}")?;
    }
    for moved in &s.moved {
        writeln!(
            out,
            "would move {}: {} to {}",
            moved.name, moved.from, moved.to
        )?;
    }
    for name in &s.aliased {
        writeln!(out, "would record the alias {name}")?;
    }
    for name in &s.retired {
        writeln!(out, "would record {name} as retired")?;
    }
    for name in &s.removed {
        writeln!(
            err,
            "warning: {PERMALINKS_FILE} has {name}, which the project no longer has; `ascribe check` reports it until it's back or retired"
        )?;
    }
    if s.changed {
        writeln!(
            err,
            "error: {PERMALINKS_FILE} is out of date; run `ascribe permalinks sync` to update it"
        )
    } else {
        writeln!(out, "{PERMALINKS_FILE} is up to date.")
    }
}

/// Exit codes: 0 whatever the list says; 2 when the project doesn't load,
/// or its list can't be read.
fn run_list(global: &Global, args: &ListArgs, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let project = match load_project(global) {
        Ok(project) => project,
        Err(failure) => return answer::report_failure(err, &failure),
    };
    let listed = match ascribe_query::permalinks::list(&project.index(), project.permalinks()) {
        Ok(listed) => listed,
        Err(e) => return answer::fail(err, &e),
    };
    let result = match args.format {
        Format::Json => answer::write_json(out, &listed),
        Format::Text => write_list(out, &listed),
    };
    answer::written(result, err).unwrap_or(exit::OK)
}

fn write_list(out: &mut dyn Write, a: &PermalinksAnswer) -> io::Result<()> {
    if a.permalinks.is_empty()
        && a.aliases.is_empty()
        && a.moved.is_empty()
        && a.retired.is_empty()
        && a.removed.is_empty()
    {
        return writeln!(out, "The content declares no permalinks.");
    }
    for p in &a.permalinks {
        let state = match (p.state, a.has_list) {
            (ListState::Listed, _) => String::new(),
            (ListState::NotListed, true) => format!("  (not in {PERMALINKS_FILE} yet)"),
            (ListState::NotListed, false) => String::new(),
            (ListState::Moved, _) => format!(
                "  ({PERMALINKS_FILE} has it in {})",
                p.listed_at.as_deref().unwrap_or_default()
            ),
        };
        writeln!(out, "{}:{}: {}{state}", p.file, p.line, p.name)?;
    }
    let behind = |state: ListState| match (state, a.has_list) {
        (ListState::NotListed, true) => format!("  (not in {PERMALINKS_FILE} yet)"),
        _ => String::new(),
    };
    for alias in &a.aliases {
        let state = behind(alias.state);
        writeln!(
            out,
            "{ADDRESSES_FILE}: {} moved to the permalink {}{state}",
            alias.name, alias.target
        )?;
    }
    for moved in &a.moved {
        let state = behind(moved.state);
        writeln!(
            out,
            "{ADDRESSES_FILE}: {} moved to {}{state}",
            moved.name, moved.to
        )?;
    }
    for retired in &a.retired {
        let state = behind(retired.state);
        writeln!(
            out,
            "{ADDRESSES_FILE}: {} is retired: {}{state}",
            retired.name, retired.reason
        )?;
    }
    for r in &a.removed {
        let was = match r.was {
            ListedAs::Permalink => format!("was in {}, and nothing declares it now", r.place),
            ListedAs::Alias => format!(
                "was moved to the permalink {}, and {ADDRESSES_FILE} doesn't record it now",
                r.place
            ),
            ListedAs::Moved => format!(
                "was moved to {}, and {ADDRESSES_FILE} doesn't record it now",
                r.place
            ),
            ListedAs::Retired => {
                format!("was retired, and {ADDRESSES_FILE} doesn't record it now")
            }
        };
        writeln!(out, "{PERMALINKS_FILE}: {} {was}", r.name)?;
    }
    if !a.has_list {
        writeln!(out)?;
        writeln!(
            out,
            "No {PERMALINKS_FILE}: `ascribe permalinks sync` makes one, and then `ascribe check` reports a permalink that's lost."
        )?;
    }
    Ok(())
}

/// Exit codes: 0 when the list has every name, 1 when it doesn't have one
/// or has it as retired, 2 when the list or the names can't be read.
fn run_verify(global: &Global, args: &VerifyArgs, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let list = match &args.against {
        Some(path) => path.clone(),
        None => match crate::context::locate(global) {
            Ok(config) => config
                .parent()
                .unwrap_or(Path::new("."))
                .join(PERMALINKS_FILE),
            Err(e) => return answer::fail(err, &e),
        },
    };
    let text = match read(&list) {
        Ok(text) => text,
        Err((_, io::ErrorKind::NotFound)) => {
            return answer::fail(err, &PermalinksError::ListNotFound(list));
        }
        Err((e, _)) => return answer::fail(err, &e),
    };
    let mut names = args.names.clone();
    if let Some(from) = &args.from {
        match read(from) {
            Ok(text) => names.extend(
                text.lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty() && !line.starts_with('#'))
                    .map(str::to_owned),
            ),
            Err((e, _)) => return answer::fail(err, &e),
        }
    }
    let shown = list.display().to_string();
    let verified = match ascribe_query::permalinks::verify(&text, &shown, &names) {
        Ok(verified) => verified,
        Err(e) => return answer::fail(err, &e),
    };
    let result = match args.format {
        Format::Json => answer::write_json(out, &verified),
        Format::Text => write_verify(out, err, &verified),
    };
    if let Some(code) = answer::written(result, err) {
        return code;
    }
    if verified.missing.is_empty() && verified.retired.is_empty() {
        exit::OK
    } else {
        exit::PROBLEMS
    }
}

fn write_verify(out: &mut dyn Write, err: &mut dyn Write, v: &VerifyAnswer) -> io::Result<()> {
    for retired in &v.retired {
        writeln!(
            err,
            "error: {} was retired from the docs: {}",
            retired.name, retired.reason
        )?;
    }
    for name in &v.missing {
        writeln!(err, "error: {name} isn't a permalink in {}", v.list)?;
    }
    if v.missing.is_empty() && v.retired.is_empty() {
        let count = v.found.len();
        let names = if count == 1 { "name is" } else { "names are" };
        writeln!(out, "{count} {names} permalinks in {}.", v.list)?;
    }
    Ok(())
}

/// Exit codes: 0 when the names are retired; 2 when the project doesn't
/// load, a name can't be retired, or a file can't be read or written.
fn run_retire(global: &Global, args: &RetireArgs, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let reason = args.reason.as_deref().map(str::trim);
    if reason.is_some_and(|r| r.is_empty() || is_retired_placeholder(r)) {
        return answer::fail(err, &PermalinksError::NoReason);
    }
    let project = match load_project(global) {
        Ok(project) => project,
        Err(failure) => return answer::report_failure(err, &failure),
    };
    let index = project.index();
    // A page or folder is a path from the current folder, or from the
    // content root; the first reading that names something is the one meant.
    let given = args.page.as_deref().or(args.under.as_deref());
    let places: Vec<Option<String>> = match given {
        Some(given) => content_paths(&project, given)
            .into_iter()
            .map(Some)
            .collect(),
        None => vec![None],
    };
    let mut found = None;
    for place in &places {
        let retire = Retire {
            names: &args.names,
            page: place.as_deref().filter(|_| args.page.is_some()),
            under: place.as_deref().filter(|_| args.under.is_some()),
            named: given,
        };
        match ascribe_query::permalinks::retiring(&index, project.permalinks(), &retire) {
            Ok(names) => {
                found = Some(Ok(names));
                break;
            }
            Err(e) => {
                found.get_or_insert(Err(e));
            }
        }
    }
    let names = match found {
        Some(Ok(names)) => names,
        Some(Err(e)) => return answer::fail(err, &e),
        None => Vec::new(),
    };
    // What's written into ascribe.addresses is from the content root, as
    // its other entries are.
    let to = args.to.as_deref().map(|to| target_in_content(&project, to));
    if let (Some(to), Some(given)) = (&to, &args.to)
        && ascribe_model::addresses::split_place(to).is_none()
        && ascribe_model::permalink_name(to).is_none()
    {
        return answer::fail(
            err,
            &PermalinksError::from(crate::commands::addresses::AddressesError::NotAPage(
                given.clone(),
            )),
        );
    }
    let (mut file, created) = match project.addresses() {
        None => (AddressFile::default(), true),
        Some(Ok(file)) => (file.clone(), false),
        Some(Err(issues)) => {
            let detail = issues
                .first()
                .and_then(|i| i.arg("detail"))
                .unwrap_or("it doesn't read")
                .to_owned();
            return answer::fail(
                err,
                &PermalinksError::from(crate::commands::addresses::AddressesError::Invalid(detail)),
            );
        }
    };
    if let Some(to) = to
        .as_deref()
        .filter(|to| ascribe_model::permalink_name(to).is_some())
        && let Err(e) = crate::commands::addresses::known_permalink(&project, &file, to)
    {
        return answer::fail(err, &PermalinksError::from(e));
    }
    for name in &names {
        let key = permalink_key(name);
        match (&to, reason) {
            (Some(to), _) => file.record_move(&key, to),
            (None, Some(reason)) => file.record_retired(&key, reason),
            (None, None) => {}
        }
    }
    if let Err(e) = crate::commands::addresses::write_file(project.root(), &file.to_toml()) {
        return answer::fail(err, &PermalinksError::from(e));
    }
    let list = match project.permalinks() {
        Some(Ok(list)) => ascribe_query::permalinks::record_retired(
            &index,
            list,
            &names,
            reason.unwrap_or_default(),
            to.as_deref(),
        ),
        _ => None,
    };
    if let Some(text) = &list
        && let Err(error) = write_permalinks(project.root(), text)
    {
        return answer::fail(err, &error);
    }
    let done = RetireAnswer::new(names, reason, to.as_deref(), created, list.is_some());
    let result = match args.format {
        Format::Json => answer::write_json(out, &done),
        Format::Text => write_retire(out, &done, project.permalinks().is_some()),
    };
    answer::written(result, err).unwrap_or(exit::OK)
}

fn write_retire(out: &mut dyn Write, r: &RetireAnswer, has_list: bool) -> io::Result<()> {
    for name in &r.retired {
        match &r.to {
            Some(to) => writeln!(out, "moved {name} to {to}")?,
            None => writeln!(out, "retired {name}")?,
        }
    }
    writeln!(out, "wrote {ADDRESSES_FILE}")?;
    if r.list_changed {
        writeln!(out, "wrote {PERMALINKS_FILE}")?;
    } else if has_list {
        writeln!(
            out,
            "The content still declares some of them: remove them, then run `ascribe permalinks sync`."
        )?;
    }
    Ok(())
}

/// A page or folder named on the command line, as the content paths it can
/// be: from the current folder, then from the content root. The path as
/// given when it can be neither.
fn content_paths(project: &ascribe_check::Project, given: &str) -> Vec<String> {
    let paths: Vec<String> = answer::content_paths(project, Path::new(given))
        .iter()
        .map(ToString::to_string)
        .collect();
    if paths.is_empty() {
        vec![given.to_owned()]
    } else {
        paths
    }
}

/// `--to`, a page and `#` and a heading's id, with the page as a path from
/// the content root: the first of [`content_paths`] that's a page or
/// fragment of the project, else as given, which `ascribe check` reports.
/// A permalink, `go:<name>`, is as given.
fn target_in_content(project: &ascribe_check::Project, to: &str) -> String {
    if to.starts_with(ascribe_model::PERMALINK_PREFIX) {
        return to.to_owned();
    }
    let (page, id) = match to.split_once('#') {
        Some((page, id)) => (page, Some(id)),
        None => (to, None),
    };
    if page.is_empty() {
        return to.to_owned();
    }
    let Some(page) = answer::content_paths(project, Path::new(page))
        .into_iter()
        .find(|path| project.source_at(path).is_some())
    else {
        return to.to_owned();
    };
    match id {
        Some(id) => format!("{page}#{id}"),
        None => page.to_string(),
    }
}

/// Writes the project's `ascribe.permalinks`, but never through a link,
/// which could lead anywhere a repository chose.
fn write_permalinks(root: &Path, text: &str) -> Result<(), PermalinksError> {
    let path = root.join(PERMALINKS_FILE);
    let written = if ascribe_resolve::is_link(&path) {
        Err("it's a link; replace it with a file, or remove it".to_owned())
    } else {
        std::fs::write(&path, text).map_err(|e| e.to_string())
    };
    written.map_err(|message| PermalinksError::Unwritable {
        path: PathBuf::from(PERMALINKS_FILE),
        message,
    })
}

/// A file named on the command line, or why it can't be read.
fn read(path: &Path) -> Result<String, (PermalinksError, io::ErrorKind)> {
    // Outside FileSystem: a file named on the command line, such as a copy
    // of a list in a product's repository, which `verify` reads with no
    // project at all.
    std::fs::read_to_string(path).map_err(|e| {
        let kind = e.kind();
        let error = PermalinksError::Unreadable {
            path: path.to_path_buf(),
            message: e.to_string(),
        };
        (error, kind)
    })
}
