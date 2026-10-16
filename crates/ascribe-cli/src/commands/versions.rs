//! `ascribe versions`: a project's release lines. `list` shows them,
//! `changes` says what differs between two from the marks alone,
//! `release` makes a line live, rewriting the marks that named it through
//! a feature, and `archive` renders a line into a folder of HTML that needs
//! nothing else.

use std::collections::BTreeMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ascribe_core::availability::parse_version;
use ascribe_core::{Coded, RelPath};
use ascribe_diff::{DiffError, Repository};
use ascribe_emit::{ArchiveOptions, OutputDir};
use ascribe_lsp::{Released, Releasing};
use ascribe_model::LineStatus;
use ascribe_query::versions::{LineChange, VersionChanges, VersionsList};
use clap::{Args as ClapArgs, Subcommand, ValueEnum};
use serde::Serialize;

use crate::answer::{self, Format};
use crate::cli::Global;
use crate::context::load_project;
use crate::exit;

/// Arguments of `ascribe versions`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

/// The subcommands of `ascribe versions`.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Show each target's release lines, newest first: each one's status,
    /// the output and pages of each live line, when its support ends, and
    /// where an archived line can be read.
    #[command(after_help = docs_link!("cli-versions-list"))]
    List(ListArgs),
    /// Say what differs between two lines of a target, from the marks
    /// alone: the pages, sections, blocks, and rows one line has and the
    /// other doesn't, and those whose state differs.
    #[command(after_help = docs_link!("cli-versions-changes"))]
    Changes(ChangesArgs),
    /// Release a line: rewrite every mark that names a feature pointing at
    /// the line to the line's spec, point the feature at the next planned
    /// line, and move the line from planned to live. Where no build is
    /// versioned by the target, the line that was current moves to
    /// archived.
    ///
    /// Writes only files git can put back: each must be tracked and
    /// unchanged, unless --force.
    #[command(after_help = docs_link!("cli-versions-release"))]
    Release(ReleaseArgs),
    /// Render a line into a folder of HTML that needs nothing else: every
    /// page, with a statement of what it is, a sidebar, a contents page,
    /// and a print stylesheet, marked so crawlers leave it alone, working
    /// from a disk with no server.
    ///
    /// Writes nothing into the project: the text says what to add to
    /// [releases] once the folder is hosted.
    #[command(after_help = docs_link!("cli-versions-archive"))]
    Archive(ArchiveArgs),
}

/// Arguments of `ascribe versions list`.
#[derive(Debug, ClapArgs)]
pub struct ListArgs {
    /// How to show the lines.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
}

/// Arguments of `ascribe versions changes`.
#[derive(Debug, ClapArgs)]
pub struct ChangesArgs {
    /// The line compared from, named as a mark names it:
    /// "self-managed 3.4".
    #[arg(value_name = "FROM")]
    pub from: String,

    /// The line compared to, of the same target: 3.5.
    #[arg(value_name = "TO")]
    pub to: String,

    /// How to show what differs.
    #[arg(long, value_enum, default_value_t = ChangesFormat::Text, value_name = "FORMAT")]
    pub format: ChangesFormat,
}

/// The formats of `ascribe versions changes`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum ChangesFormat {
    /// What was added, removed, and changed, for people.
    Text,
    /// One JSON document, for tools.
    Json,
    /// A first draft of release notes, with a heading for each page.
    Markdown,
}

/// Arguments of `ascribe versions release`.
#[derive(Debug, ClapArgs)]
pub struct ReleaseArgs {
    /// The line to release, named as a mark names it: "ascribe 0.4.0".
    #[arg(value_name = "LINE")]
    pub line: String,

    /// The line the features that named the released line move to, added
    /// to planned. Needed when a feature names the line and no planned
    /// line follows it.
    #[arg(long, value_name = "LINE")]
    pub next: Option<String>,

    /// Say what the release would do, and write nothing.
    #[arg(long)]
    pub dry_run: bool,

    /// Write the files even when git can't put them back: untracked or
    /// changed files, or no repository at all.
    #[arg(long)]
    pub force: bool,

    /// How to show what the release did.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
}

/// Arguments of `ascribe versions archive`.
#[derive(Debug, ClapArgs)]
pub struct ArchiveArgs {
    /// The line to archive, named as a mark names it: "self-managed 3.4".
    /// It's live or planned.
    #[arg(value_name = "LINE")]
    pub line: String,

    /// The folder to write the archive in, from the current folder: an
    /// empty one, or one an archive was written in, whose files it
    /// replaces.
    #[arg(long, value_name = "DIR")]
    pub to: PathBuf,

    /// The versioned build to archive the line of, when two builds are
    /// versioned by the line's target.
    #[arg(long, value_name = "NAME")]
    pub build: Option<String>,

    /// The address of the current documentation, which the statement at
    /// the top of every page links to.
    #[arg(long, value_name = "URL")]
    pub current: Option<String>,

    /// How to show what was written.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
}

/// Why `ascribe versions release` wrote nothing, apart from the line.
#[derive(Debug, thiserror::Error)]
pub enum VersionsError {
    /// The line isn't named as a mark names one.
    #[error(
        "`{0}` isn't a line: name one as a mark does, with its target, such as \"self-managed 3.4\""
    )]
    BadLine(String),
    /// `--next` isn't a version.
    #[error("--next {0} isn't a version, such as 3.6")]
    BadNext(String),
    /// git can't say whether the files can be put back.
    #[error(
        "the release rewrites files that only git could put back, and {why}; run it in a repository, or pass --force"
    )]
    NoGit {
        /// Why git can't.
        why: String,
    },
    /// Files the release would write aren't tracked, or have changes.
    #[error(
        "the release would write files git can't put back as they are: {}; commit or stash them, or pass --force",
        .files.join(", ")
    )]
    NotRestorable {
        /// The files, from the current folder.
        files: Vec<String>,
    },
    /// No build is versioned by the line's target.
    #[error(
        "no build is versioned by {target}, so it has no line to archive: set `versions = \"{target}\"` on a build"
    )]
    ArchiveNoBuild {
        /// The target.
        target: String,
    },
    /// Several builds are versioned by the line's target, and none was
    /// named.
    #[error("several builds are versioned by {target}, so name one with --build: {}", .builds.join(", "))]
    ArchiveBuildRequired {
        /// The target.
        target: String,
        /// The builds.
        builds: Vec<String>,
    },
    /// `--build` names no build versioned by the line's target.
    #[error("--build {build} isn't a build versioned by {target}; those are {}", .builds.join(", "))]
    ArchiveUnknownBuild {
        /// The build, as given.
        build: String,
        /// The target.
        target: String,
        /// The builds versioned by it.
        builds: Vec<String>,
    },
    /// The line is archived already, so it has no output to render.
    #[error("{target} {line} is archived; only a live or planned line can be archived")]
    ArchiveLineArchived {
        /// The target.
        target: String,
        /// The line.
        line: String,
    },
    /// The checks found errors in the build.
    #[error(
        "the checks found {count} error{} in build `{build}`, so nothing was written; {listed}",
        if *.count == 1 { "" } else { "s" }
    )]
    ArchiveErrors {
        /// How many.
        count: usize,
        /// The build checked.
        build: String,
        /// What lists them.
        listed: String,
    },
    /// A file can't be written.
    #[error("can't write {}: {message}", path.display())]
    Unwritable {
        /// The file.
        path: PathBuf,
        /// What went wrong.
        message: String,
    },
}

impl Coded for VersionsError {
    fn code(&self) -> &'static str {
        match self {
            VersionsError::BadLine(_) => "release_bad_line",
            VersionsError::BadNext(_) => "release_next_bad",
            VersionsError::NoGit { .. } => "release_without_git",
            VersionsError::NotRestorable { .. } => "release_files_unrestorable",
            VersionsError::Unwritable { .. } => "release_unwritable",
            VersionsError::ArchiveNoBuild { .. } => "archive_no_build",
            VersionsError::ArchiveBuildRequired { .. } => "build_required",
            VersionsError::ArchiveUnknownBuild { .. } => "unknown_build",
            VersionsError::ArchiveLineArchived { .. } => "archive_line_archived",
            VersionsError::ArchiveErrors { .. } => "archive_has_errors",
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
        Command::List(args) => run_list(global, &args, &mut out, &mut err),
        Command::Changes(args) => run_changes(global, &args, &mut out, &mut err),
        Command::Release(args) => run_release(global, &args, &mut out, &mut err),
        Command::Archive(args) => run_archive(global, &args, &mut out, &mut err),
    };
    let _ = out.flush();
    exit::code(code)
}

/// Exit codes: 0 whatever the lists say; 2 when the project doesn't load.
fn run_list(global: &Global, args: &ListArgs, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let project = match load_project(global) {
        Ok(project) => project,
        Err(failure) => return answer::report_failure(err, &failure),
    };
    let listed = ascribe_query::versions::list(&project.index());
    let result = match args.format {
        Format::Json => answer::write_json(out, &listed),
        Format::Text => write_list(out, &listed),
    };
    answer::written(result, err).unwrap_or(exit::OK)
}

fn write_list(out: &mut dyn Write, listed: &VersionsList) -> io::Result<()> {
    if listed.targets.is_empty() {
        return writeln!(
            out,
            "The project lists no release lines: [releases.<target>] in ascribe.toml declares them."
        );
    }
    for (i, target) in listed.targets.iter().enumerate() {
        if i > 0 {
            writeln!(out)?;
        }
        match target.builds.as_slice() {
            [] => writeln!(out, "{}", target.target)?,
            builds => writeln!(
                out,
                "{}, versioning {}",
                target.target,
                builds
                    .iter()
                    .map(|b| format!("`{b}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )?,
        }
        let width = target.lines.iter().map(|l| l.line.len()).max().unwrap_or(0);
        for line in &target.lines {
            let mut about: Vec<String> = line
                .outputs
                .iter()
                .map(|o| {
                    let pages = if o.pages == 1 { "page" } else { "pages" };
                    format!("{} at {}, {} {pages}", o.build, o.base, o.pages)
                })
                .collect();
            if let Some(ends) = &line.ends {
                about.push(format!("support ends {ends}"));
            }
            if let Some(ended) = &line.ended {
                about.push(format!("support ended {ended}"));
            }
            if line.status == "archived" {
                about.push(match &line.url {
                    Some(url) => format!("at {url}"),
                    None => "readable nowhere".to_owned(),
                });
            }
            let about = if about.is_empty() {
                String::new()
            } else {
                format!("  {}", about.join("; "))
            };
            let row = format!("  {:width$}  {:8}{about}", line.line, line.status);
            writeln!(out, "{}", row.trim_end())?;
        }
    }
    Ok(())
}

/// Exit codes: 0 whatever differs; 2 when the project doesn't load, or a
/// line isn't declared.
fn run_changes(
    global: &Global,
    args: &ChangesArgs,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    let project = match load_project(global) {
        Ok(project) => project,
        Err(failure) => return answer::report_failure(err, &failure),
    };
    let changes = match ascribe_query::versions::changes(&project.index(), &args.from, &args.to) {
        Ok(changes) => changes,
        Err(e) => return answer::fail(err, &e),
    };
    let result = match args.format {
        ChangesFormat::Json => answer::write_json(out, &changes),
        ChangesFormat::Text => write_changes(out, &changes, project.root()),
        ChangesFormat::Markdown => write_notes(out, &changes, project.root()),
    };
    answer::written(result, err).unwrap_or(exit::OK)
}

/// The text of `changes`, each path from the folder it was run in; `root`
/// is the project's folder, which the report's paths are from.
fn write_changes(out: &mut dyn Write, c: &VersionChanges, root: &Path) -> io::Result<()> {
    let (target, from, to) = (&c.target, &c.from, &c.to);
    if c.added.is_empty() && c.removed.is_empty() && c.changed.is_empty() {
        return writeln!(
            out,
            "Nothing differs between {target} {from} and {to}, by the marks."
        );
    }
    let groups = [
        (format!("In {to} and not {from}:"), &c.added),
        (format!("In {from} and not {to}:"), &c.removed),
        ("In both, in another state:".to_owned(), &c.changed),
    ];
    let mut first = true;
    for (title, entries) in groups {
        if entries.is_empty() {
            continue;
        }
        if !first {
            writeln!(out)?;
        }
        first = false;
        writeln!(out, "{title}")?;
        for e in entries {
            let state = match (&e.from_state, &e.to_state) {
                (Some(a), Some(b)) => format!(" ({a} in {from}, {b} in {to})"),
                _ => String::new(),
            };
            let place = match &e.in_place_of {
                Some(old) => format!(
                    ", in place of \"{}\" at {}:{}",
                    old.page,
                    answer::path_from_here(&root.join(&old.file)),
                    old.line
                ),
                None => String::new(),
            };
            writeln!(
                out,
                "  {}:{}: {}{state}{place}",
                answer::path_from_here(&root.join(&e.file)),
                e.line,
                described(e)
            )?;
        }
    }
    Ok(())
}

/// A change as a phrase: `the page "Webhooks"`, `a section "Rotate keys"`.
fn described(e: &LineChange) -> String {
    match e.kind {
        "page" => format!("the page \"{}\"", e.page),
        "section" => format!("the section \"{}\"", e.text),
        "row" => format!("a row: {}", e.text),
        _ => match &e.heading {
            Some(heading) => format!("a block under \"{heading}\": {}", e.text),
            None => format!("a block: {}", e.text),
        },
    }
}

/// The Markdown draft of release notes: a heading for each page, and a
/// line for each change on it.
fn write_notes<'a>(out: &mut dyn Write, c: &'a VersionChanges, root: &Path) -> io::Result<()> {
    let (target, from, to) = (&c.target, &c.from, &c.to);
    writeln!(out, "## {target} {to}, since {from}")?;
    let mut pages: BTreeMap<(&str, &str), Vec<String>> = BTreeMap::new();
    let mut note = |e: &'a LineChange, line: String| {
        pages.entry((&e.file, &e.page)).or_default().push(line);
    };
    for e in &c.added {
        let place = match &e.in_place_of {
            Some(old) => format!(
                ", in place of \"{}\" in `{}`",
                old.page,
                answer::path_from_here(&root.join(&old.file))
            ),
            None => String::new(),
        };
        note(e, format!("- New: {}{place}", noted(e)));
    }
    for e in &c.removed {
        note(e, format!("- Removed: {}", noted(e)));
    }
    for e in &c.changed {
        let state = match (&e.from_state, &e.to_state) {
            (Some(a), Some(b)) => format!(", now {b}, was {a}"),
            _ => String::new(),
        };
        note(e, format!("- Changed: {}{state}", noted(e)));
    }
    if pages.is_empty() {
        writeln!(out)?;
        return writeln!(out, "Nothing differs, by the marks.");
    }
    for ((file, page), lines) in pages {
        writeln!(out)?;
        writeln!(out, "### {page}")?;
        writeln!(out)?;
        writeln!(out, "In `{}`.", answer::path_from_here(&root.join(file)))?;
        writeln!(out)?;
        for line in lines {
            writeln!(out, "{line}")?;
        }
    }
    Ok(())
}

fn noted(e: &LineChange) -> String {
    match e.kind {
        "page" => "the page".to_owned(),
        "section" => format!("the section \"{}\"", e.text),
        "row" => format!("a row, `{}`", e.text),
        _ => match &e.heading {
            Some(heading) => format!("under \"{heading}\", {}", e.text),
            None => e.text.clone(),
        },
    }
}

/// What `ascribe versions release --format json` writes: the line released,
/// the files and marks rewritten, the features moved, and what the release
/// changes in the outputs.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub(crate) struct ReleaseJson {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    schema_version: u32,
    /// The version of Ascribe that wrote it.
    ascribe_version: &'static str,
    /// Whether the files were written: false with `--dry-run`.
    written: bool,
    /// The target.
    target: String,
    /// The line released, as given.
    line: String,
    /// Whether the line was in `planned`; a line that wasn't is a release
    /// nobody planned, such as a patch.
    was_planned: bool,
    /// The line the features that named the released one move to, when
    /// one does.
    next: Option<String>,
    /// Whether `next` was added to `planned`, from `--next`.
    next_added: bool,
    /// The line that was current, moved to `archived` with no address,
    /// because no build is versioned by the target.
    archived: Option<String>,
    /// Each feature that named the line, pointed at `next`.
    features: Vec<FeatureJson>,
    /// Each source file whose marks are rewritten, from the project root.
    files: Vec<FileJson>,
    /// What the release changes in the builds' outputs, by build and
    /// output.
    outputs: Vec<OutputJson>,
}

/// A feature moved by the release.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct FeatureJson {
    /// Its key.
    key: String,
    /// Its spec before.
    from: String,
    /// Its spec after.
    to: String,
    /// The other targets its spec names, left as they were.
    kept: Vec<String>,
}

/// A source file the release rewrites.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct FileJson {
    /// The file, from the project root.
    file: String,
    /// Each mark rewritten.
    marks: Vec<MarkJson>,
}

/// A mark the release rewrites.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct MarkJson {
    /// Its line, from 1.
    line: u32,
    /// The feature it named.
    from: String,
    /// The spec it names now.
    to: String,
}

/// One output the release changes.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct OutputJson {
    /// The build: `site`, or an older line's, `site@3.4`. Empty for the
    /// file the lines share.
    build: String,
    /// The output: `site`, `plain`, or `json`, or `_ascribe` for the file
    /// the lines share.
    output: String,
    /// The files it gains, by their path in the output.
    added: Vec<String>,
    /// The files it loses.
    removed: Vec<String>,
    /// The files that change.
    changed: Vec<String>,
}

/// The version of the release's JSON.
const RELEASE_SCHEMA_VERSION: u32 = 1;

impl ReleaseJson {
    fn new(r: &Released, written: bool, layout: &ascribe_resolve::Layout) -> ReleaseJson {
        ReleaseJson {
            schema_version: RELEASE_SCHEMA_VERSION,
            ascribe_version: env!("CARGO_PKG_VERSION"),
            written,
            target: r.target.clone(),
            line: r.line.clone(),
            was_planned: r.was_planned,
            next: r.next.clone(),
            next_added: r.next_added,
            archived: r.archived.clone(),
            features: r
                .features
                .iter()
                .map(|f| FeatureJson {
                    key: f.key.clone(),
                    from: f.from.clone(),
                    to: f.to.clone(),
                    kept: f.kept.clone(),
                })
                .collect(),
            files: r
                .files
                .iter()
                .map(|f| FileJson {
                    file: layout.project_path(&f.path).to_string(),
                    marks: f
                        .marks
                        .iter()
                        .map(|m| MarkJson {
                            line: m.line,
                            from: m.from.clone(),
                            to: m.to.clone(),
                        })
                        .collect(),
                })
                .collect(),
            outputs: r
                .outputs
                .iter()
                .map(|o| OutputJson {
                    build: o.build.clone(),
                    output: o.output.clone(),
                    added: o.added.clone(),
                    removed: o.removed.clone(),
                    changed: o.changed.clone(),
                })
                .collect(),
        }
    }
}

/// Exit codes: 0 when the line is released, or with `--dry-run` would be;
/// 2 when it can't be: the line isn't one to release, a feature names it
/// and nothing says where the feature goes, a file can't be put back or
/// written, or a step that should change no output would.
fn run_release(
    global: &Global,
    args: &ReleaseArgs,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    let Some((target, line)) = ascribe_model::named_line(&args.line) else {
        return answer::fail(err, &VersionsError::BadLine(args.line.clone()));
    };
    let next = match &args.next {
        Some(next) => match parse_version(next.trim(), 0) {
            Ok(next) => Some(next),
            Err(_) => return answer::fail(err, &VersionsError::BadNext(next.clone())),
        },
        None => None,
    };
    let config = match crate::context::locate(global) {
        Ok(config) => config,
        Err(e) => return answer::fail(err, &e),
    };
    let project = match load_project(global) {
        Ok(project) => project,
        Err(failure) => return answer::report_failure(err, &failure),
    };
    let releasing = Releasing {
        target: &target,
        line: &line,
        next: next.as_ref(),
    };
    let released = match ascribe_lsp::release(&project, &releasing) {
        Ok(released) => released,
        Err(e) => return answer::fail(err, &e),
    };
    let content = project.root().join(project.content_root().as_str());
    let mut writes: Vec<(PathBuf, &str)> = vec![(config.clone(), released.model_text.as_str())];
    writes.extend(
        released
            .files
            .iter()
            .map(|f| (content.join(f.path.as_str()), f.text.as_str())),
    );
    if !args.dry_run {
        if !args.force
            && let Err(e) = restorable(&project, &config, &released)
        {
            return answer::fail(err, &e);
        }
        for (path, text) in &writes {
            if let Err(e) = std::fs::write(path, text) {
                let error = VersionsError::Unwritable {
                    path: answer::path_from_here(path).into(),
                    message: e.to_string(),
                };
                return answer::fail(err, &error);
            }
        }
    }
    let result = match args.format {
        Format::Json => answer::write_json(
            out,
            &ReleaseJson::new(&released, !args.dry_run, project.layout()),
        ),
        Format::Text => write_release(out, &released, args.dry_run, &content),
    };
    answer::written(result, err).unwrap_or(exit::OK)
}

/// Whether git can put back every file the release writes: each is in a
/// repository, tracked, and unchanged.
fn restorable(
    project: &ascribe_check::Project,
    config: &Path,
    released: &Released,
) -> Result<(), VersionsError> {
    let repo = Repository::discover(project.root()).map_err(|e| VersionsError::NoGit {
        why: match e {
            DiffError::GitNotFound => "git can't be run".to_owned(),
            DiffError::NotARepository { .. } => format!(
                "{} isn't in a git repository",
                answer::path_from_here(project.root())
            ),
            other => other.to_string(),
        },
    })?;
    let model_file = config
        .file_name()
        .and_then(|n| n.to_str())
        .and_then(|n| RelPath::parse(n).ok());
    let mut paths: Vec<RelPath> = model_file.into_iter().collect();
    paths.extend(
        released
            .files
            .iter()
            .map(|f| project.layout().project_path(&f.path)),
    );
    let found = repo
        .not_restorable(&paths)
        .map_err(|e| VersionsError::NoGit { why: e.to_string() })?;
    if found.is_empty() {
        return Ok(());
    }
    let root = Path::new(&repo.root);
    Err(VersionsError::NotRestorable {
        files: found
            .iter()
            .map(|p| answer::path_from_here(&root.join(p.as_str())))
            .collect(),
    })
}

fn write_release(
    out: &mut dyn Write,
    r: &Released,
    dry_run: bool,
    content: &Path,
) -> io::Result<()> {
    let (target, line) = (&r.target, &r.line);
    let (is, rewrote, names, moved) = if dry_run {
        ("would be", "Would rewrite", "would name", "would move")
    } else {
        ("is", "Rewrote", "now names", "moved")
    };
    writeln!(out, "{target} {line} {is} live, and current.")?;
    let marks: usize = r.files.iter().map(|f| f.marks.len()).sum();
    if marks > 0 {
        let mut by: BTreeMap<(&str, &str), usize> = BTreeMap::new();
        for mark in r.files.iter().flat_map(|f| &f.marks) {
            *by.entry((&mark.from, &mark.to)).or_default() += 1;
        }
        for ((from, to), count) in by {
            let files = r
                .files
                .iter()
                .filter(|f| f.marks.iter().any(|m| m.from == from))
                .count();
            writeln!(
                out,
                "{rewrote} {count} mark{} from `{from}` to `{to}`, in {files} file{}.",
                if count == 1 { "" } else { "s" },
                if files == 1 { "" } else { "s" }
            )?;
        }
    }
    let next = r.next.as_deref().unwrap_or_default();
    for f in &r.features {
        writeln!(
            out,
            "`{}` {names} {next}, which is planned{}.",
            f.key,
            if f.kept.is_empty() {
                String::new()
            } else {
                format!(
                    "; it names {} too, left as {}: look at it",
                    f.kept.join(", "),
                    if f.kept.len() == 1 {
                        "it was"
                    } else {
                        "they were"
                    }
                )
            }
        )?;
    }
    if r.features.is_empty() && r.next_added {
        writeln!(out, "{target} {next} {is} planned.")?;
    }
    if let Some(archived) = &r.archived {
        writeln!(
            out,
            "{target} {archived} {moved} to archived, with no address: no build is versioned by {target}."
        )?;
    }
    if r.outputs.is_empty() {
        writeln!(out, "No output changed.")?;
    } else {
        writeln!(out, "What a reader sees changes in:")?;
        for o in &r.outputs {
            let mut parts = Vec::new();
            for (count, word) in [
                (o.changed.len(), "changed"),
                (o.added.len(), "added"),
                (o.removed.len(), "removed"),
            ] {
                if count > 0 {
                    parts.push(format!(
                        "{count} file{} {word}",
                        if count == 1 { "" } else { "s" }
                    ));
                }
            }
            let name = if o.build.is_empty() {
                "the file the lines share".to_owned()
            } else {
                format!("{}'s {} output", o.build, o.output)
            };
            writeln!(out, "  {name}: {}", parts.join(", "))?;
        }
    }
    if dry_run {
        let files = r.files.len() + 1;
        writeln!(
            out,
            "Dry run: nothing written. It would write ascribe.toml{}.",
            if files > 1 {
                format!(
                    " and {} file{} under {}",
                    files - 1,
                    if files == 2 { "" } else { "s" },
                    answer::path_from_here(content)
                )
            } else {
                String::new()
            }
        )?;
    }
    Ok(())
}

/// What `ascribe versions archive --format json` writes: the line, the
/// folder, and what's in it.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub(crate) struct ArchiveJson {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    schema_version: u32,
    /// The version of Ascribe that wrote it.
    ascribe_version: &'static str,
    /// The target.
    target: String,
    /// The line archived, as `[releases.<target>]` writes it.
    line: String,
    /// The line's status: `current`, `live`, or `planned`.
    status: &'static str,
    /// The build the line is a line of, as it's declared.
    build: String,
    /// The folder written, from the current folder, with `/`.
    folder: String,
    /// The contents page, from the folder: `index.html`, or
    /// `contents.html` when the line's root page is `index.html`.
    contents: String,
    /// How many pages the archive has.
    pages: usize,
    /// How many copies of images and files they use it has.
    assets: usize,
    /// How many files of an archive written there before it no longer
    /// has, removed.
    removed: usize,
    /// The size of the search index, `search-index.js`, in bytes.
    search_index_bytes: usize,
    /// Whether the search index is large enough that a reader's first
    /// search waits for it to load.
    search_index_large: bool,
    /// Each link written as its text: a link out of the line, or to an
    /// address no page of the line has.
    unlinked: Vec<UnlinkedJson>,
}

/// A link an archive writes as its text.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct UnlinkedJson {
    /// The page it's on, from the project root.
    file: String,
    /// Its text.
    text: String,
    /// Where it led.
    href: String,
    /// For a link out of the line, the line it led to.
    line: Option<String>,
}

/// The version of the archive's JSON.
const ARCHIVE_SCHEMA_VERSION: u32 = 1;

/// Exit codes: 0 when the archive is written; 2 when it can't be: the line
/// isn't declared, live, or planned, no build or more than one is
/// versioned by its target, the checks find errors, or the folder isn't
/// empty and isn't an archive Ascribe wrote.
fn run_archive(
    global: &Global,
    args: &ArchiveArgs,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    let project = match load_project(global) {
        Ok(project) => project,
        Err(failure) => return answer::report_failure(err, &failure),
    };
    let index = project.index();
    let model = index.model();
    let (target, line) = match ascribe_query::versions::declared_line(model, &args.line) {
        Ok(found) => found,
        Err(e) => return answer::fail(err, &e),
    };
    let status = model.releases_of(&target).and_then(|r| r.status_of(&line));
    if status == Some(LineStatus::Archived) {
        let error = VersionsError::ArchiveLineArchived {
            target,
            line: line.text,
        };
        return answer::fail(err, &error);
    }
    let mut declared: Vec<String> = Vec::new();
    for build in &model.builds {
        if build.versions.as_deref() == Some(target.as_str())
            && !declared.iter().any(|d| d == build.declared_name())
        {
            declared.push(build.declared_name().to_owned());
        }
    }
    let declared_build = match (&args.build, declared.as_slice()) {
        (_, []) => return answer::fail(err, &VersionsError::ArchiveNoBuild { target }),
        (Some(name), builds) if !builds.contains(name) => {
            let error = VersionsError::ArchiveUnknownBuild {
                build: name.clone(),
                target,
                builds: declared,
            };
            return answer::fail(err, &error);
        }
        (Some(name), _) => name.clone(),
        (None, [one]) => one.clone(),
        (None, _) => {
            let error = VersionsError::ArchiveBuildRequired {
                target,
                builds: declared,
            };
            return answer::fail(err, &error);
        }
    };
    let Some(build) = model.line_build(&declared_build, &line) else {
        let error = VersionsError::ArchiveLineArchived {
            target,
            line: line.text,
        };
        return answer::fail(err, &error);
    };
    // The checks of the line's own build, as `ascribe build` runs them,
    // before anything is written.
    let errors = ascribe_check::count_errors_in(&project, &build);
    if errors > 0 {
        let listed = if model.builds.iter().any(|b| b.name == build.name) {
            format!("`ascribe check --build {}` lists them", build.name)
        } else {
            // A planned line's build is made for the archive alone.
            "`ascribe check` checks the live lines, so it lists those they share with this one"
                .to_owned()
        };
        let error = VersionsError::ArchiveErrors {
            count: errors,
            build: build.name.clone(),
            listed,
        };
        return answer::fail(err, &error);
    }
    let options = ArchiveOptions {
        current: args.current.clone(),
    };
    let archive = match ascribe_emit::archive(&index, project.root(), &build, &options) {
        Ok(archive) => archive,
        Err(e) => return answer::fail(err, &e),
    };
    let replaced = match OutputDir::lock_archive(&args.to)
        .and_then(|dir| dir.replace_archive(&build.name, &archive.files))
    {
        Ok(replaced) => replaced,
        Err(e) => return answer::fail(err, &e),
    };
    let status = match status {
        Some(LineStatus::Current) => "current",
        Some(LineStatus::Planned) => "planned",
        _ => "live",
    };
    let content = project.content_root();
    let json = ArchiveJson {
        schema_version: ARCHIVE_SCHEMA_VERSION,
        ascribe_version: env!("CARGO_PKG_VERSION"),
        target,
        line: line.text,
        status,
        build: declared_build,
        folder: answer::path_from_here(&args.to),
        contents: archive.contents.to_string(),
        pages: archive.pages,
        assets: archive
            .files
            .iter()
            .filter(|f| f.kind == ascribe_emit::FileKind::Asset)
            .count(),
        removed: replaced.removed,
        search_index_bytes: archive.search_index,
        search_index_large: archive.search_index > ascribe_emit::archive::LARGE_SEARCH_INDEX,
        unlinked: archive
            .unlinked
            .iter()
            .map(|u| UnlinkedJson {
                file: content
                    .join(u.page.as_str())
                    .map_or_else(|_| u.page.to_string(), |p| p.to_string()),
                text: u.text.clone(),
                href: u.href.clone(),
                line: u.line.clone(),
            })
            .collect(),
    };
    let result = match args.format {
        Format::Json => answer::write_json(out, &json),
        Format::Text => write_archive(out, &json, project.root()),
    };
    answer::written(result, err).unwrap_or(exit::OK)
}

/// A size in bytes, as a person reads it: `12 KB`, `3.4 MB`.
fn size(bytes: usize) -> String {
    let kilobytes = bytes.div_ceil(1_000).max(1);
    if kilobytes < 1_000 {
        format!("{kilobytes} KB")
    } else {
        #[allow(clippy::cast_precision_loss)] // A size shown to one decimal.
        let megabytes = bytes as f64 / 1_000_000.0;
        format!("{megabytes:.1} MB")
    }
}

fn write_archive(out: &mut dyn Write, a: &ArchiveJson, root: &Path) -> io::Result<()> {
    let (target, line) = (&a.target, &a.line);
    match a.status {
        "current" => writeln!(
            out,
            "{target} {line} is the current line: an archive is usually of a line that's leaving."
        )?,
        "planned" => writeln!(out, "{target} {line} is planned, and not released yet.")?,
        _ => {}
    }
    writeln!(
        out,
        "Wrote {} page{} to {}. Open {}/{}.",
        a.pages,
        if a.pages == 1 { "" } else { "s" },
        a.folder,
        a.folder,
        a.contents
    )?;
    if a.search_index_large {
        writeln!(
            out,
            "Its search index is {}, so a reader's first search waits while it loads.",
            size(a.search_index_bytes)
        )?;
    } else {
        writeln!(out, "Its search index is {}.", size(a.search_index_bytes))?;
    }
    if !a.unlinked.is_empty() {
        writeln!(
            out,
            "{} written as text, since the page {} to isn't in the line:",
            if a.unlinked.len() == 1 {
                "A link is".to_owned()
            } else {
                format!("{} links are", a.unlinked.len())
            },
            if a.unlinked.len() == 1 {
                "it leads"
            } else {
                "each leads"
            }
        )?;
        for u in &a.unlinked {
            let to = match &u.line {
                Some(l) => format!("{} in {l}", u.href),
                None => u.href.clone(),
            };
            writeln!(
                out,
                "  {}: \"{}\" led to {to}",
                answer::path_from_here(&root.join(&u.file)),
                u.text
            )?;
        }
    }
    writeln!(out, "Host the folder, then record its address:")?;
    writeln!(out, "  [releases.{target}.archived]")?;
    writeln!(out, "  \"{line}\" = {{ url = \"…\" }}")
}

#[cfg(test)]
mod tests {
    use super::size;

    #[test]
    fn a_size_is_in_kilobytes_rounded_up_then_megabytes() {
        assert_eq!(size(0), "1 KB");
        assert_eq!(size(3_501), "4 KB");
        assert_eq!(size(999_000), "999 KB");
        assert_eq!(size(999_001), "1.0 MB");
        assert_eq!(size(8_250_000), "8.2 MB");
    }
}
