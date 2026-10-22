//! `ascribe templates`: the page templates Ascribe carries. `list` shows the
//! set, `eject` writes templates into the project, as its own files, and
//! declares their types (`ascribe_templates::eject`), and `status` says
//! what's changed in the set since the project's copy
//! (`ascribe_templates::status`).

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ascribe_check::{LoadError, LocateError, Project};
use ascribe_core::Coded;
use ascribe_core::RelPath;
use ascribe_core::path::{normalize, relative_path};
use ascribe_resolve::{DiskFs, Layout};
use ascribe_templates::{
    Change, EjectAnswer, FileStatus, Record, SectionChangeKind, StatusAnswer, TEMPLATES_DIR,
    TemplateStatus, Upstream, UpstreamChange,
};
use clap::{Args as ClapArgs, Subcommand};

use crate::answer::{self, Format, from_here};
use crate::cli::Global;
use crate::context::{Failure, locate};
use crate::exit;

/// Arguments of `ascribe templates`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

/// The subcommands of `ascribe templates`.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Show the templates Ascribe carries, each with what it's for. Reads
    /// nothing in the project.
    #[command(after_help = docs_link!("cli-templates-list"))]
    List(ListArgs),
    /// Write templates into the project, with their guides, as its own
    /// files, and declare each one's type in ascribe.toml.
    ///
    /// Writes nothing when a type is declared already, unless --keep-type
    /// keeps it, or a file is where one would be written.
    #[command(after_help = docs_link!("cli-templates-eject"))]
    Eject(EjectArgs),
    /// Say what's changed in the templates since the project's copy: for
    /// each template it took, whether the set has changed it and whether
    /// the project has, and each template it doesn't have.
    ///
    /// Compares the project's files and templates.lock with the set this
    /// ascribe carries, and reads nothing else. Changes no file.
    #[command(after_help = docs_link!("cli-templates-status"))]
    Status(StatusArgs),
}

/// Arguments of `ascribe templates list`.
#[derive(Debug, ClapArgs)]
pub struct ListArgs {}

/// Arguments of `ascribe templates eject`.
#[derive(Debug, ClapArgs)]
pub struct EjectArgs {
    /// The templates to write, by name, as `ascribe templates list` shows
    /// them. Without a name, every one.
    #[arg(value_name = "NAME")]
    pub names: Vec<String>,

    /// The folder to write them in, from the current folder: inside the
    /// folder ascribe.toml is in, and outside the content root. By default,
    /// templates/ beside ascribe.toml.
    #[arg(long, value_name = "DIR")]
    pub to: Option<PathBuf>,

    /// When ascribe.toml declares a template's type already, keep the type
    /// and add only its template and guide to it, leaving its frontmatter,
    /// other keys, and comments as they are. Refuses a type
    /// whose frontmatter doesn't declare the template's fields as strings,
    /// or that names a template or guide already.
    #[arg(long)]
    pub keep_type: bool,

    /// How to show what was written.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
}

/// Arguments of `ascribe templates status`.
#[derive(Debug, ClapArgs)]
pub struct StatusArgs {
    /// How to show what's changed.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,

    /// Exit with 1 when the set has changed a template the project took
    /// since the copy, or no longer has one.
    #[arg(long)]
    pub exit_code: bool,

    /// Print the change to one template, by name, as a unified diff from
    /// the project's files to the set's, instead of the status. It applies
    /// nothing.
    #[arg(long, value_name = "NAME", conflicts_with = "format")]
    pub diff: Option<String>,
}

/// Why `ascribe templates eject` couldn't do its work, before the library's
/// own errors.
#[derive(Debug, thiserror::Error)]
pub enum TemplatesError {
    /// There's no `ascribe.toml` to add to.
    #[error(
        "{0}\neject adds templates to a project, and this folder isn't in one: to start a project here with the templates, run `ascribe init --templates`"
    )]
    NoProject(LocateError),
    /// `--to` names a folder outside the project's.
    #[error("--to names `{}`, which isn't inside the folder ascribe.toml is in, `{}`", to.display(), root.display())]
    ToOutside {
        /// The folder, as given.
        to: PathBuf,
        /// The project's folder.
        root: PathBuf,
    },
    /// `--diff` names a template neither the set nor the project has.
    #[error("there's no template named `{name}`; the set has {known}")]
    Unknown {
        /// The name given.
        name: String,
        /// The set's names.
        known: String,
    },
}

impl Coded for TemplatesError {
    fn code(&self) -> &'static str {
        match self {
            TemplatesError::NoProject(e) => e.code(),
            TemplatesError::ToOutside { .. } => "templates_folder_refused",
            TemplatesError::Unknown { .. } => "unknown_template",
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
        Command::List(_) => run_list(&mut out, &mut err),
        Command::Eject(args) => run_eject(global, &args, &mut out, &mut err),
        Command::Status(args) => run_status(global, &args, &mut out, &mut err),
    };
    let _ = out.flush();
    exit::code(code)
}

/// Exit codes: 0, or 2 when the list can't be written.
fn run_list(out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let set = ascribe_templates::set();
    let width = set
        .templates
        .iter()
        .map(|t| t.name.len())
        .max()
        .unwrap_or(0);
    let result = (|| {
        for template in &set.templates {
            writeln!(out, "{:width$}  {}", template.name, template.summary)?;
        }
        writeln!(
            out,
            "From The Good Docs Project's templates, {}, at {}, under the {}.",
            set.repository, set.tag, set.license
        )
    })();
    answer::written(result, err).unwrap_or(exit::OK)
}

/// Exit codes: 0 when every template named was written and its type added;
/// 2 when anything was refused, and nothing was written, or it couldn't run.
fn run_eject(global: &Global, args: &EjectArgs, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let config = match locate(global) {
        Ok(config) => config,
        Err(e @ LocateError::NotFound { .. }) => {
            return answer::fail(err, &TemplatesError::NoProject(e));
        }
        Err(e) => return answer::fail(err, &Failure::Config(e)),
    };
    let file = match Project::load_model(&config) {
        Ok(file) => file,
        Err(e) => return answer::report_failure(err, &Failure::Load(e)),
    };
    let to = match &args.to {
        None => RelPath::parse(TEMPLATES_DIR).unwrap_or_default(),
        Some(to) => match folder_in_project(&file.root, to) {
            Some(rel) => rel,
            None => {
                let error = TemplatesError::ToOutside {
                    to: to.clone(),
                    root: file.root.clone(),
                };
                return answer::fail(err, &error);
            }
        },
    };
    let fs = DiskFs::new(&file.root, &Layout::from_model(&file.model));
    let request = ascribe_templates::Eject {
        root: &file.root,
        model: &file.model,
        model_text: &file.text,
        names: &args.names,
        to: &to,
        ascribe_version: env!("ASCRIBE_VERSION"),
        keep_type: args.keep_type,
    };
    let ejected = match ascribe_templates::eject(&request, &fs) {
        Ok(ejected) => ejected,
        Err(e) => return answer::fail(err, &e),
    };
    let result = match args.format {
        Format::Json => answer::write_json(out, &ejected),
        Format::Text => write_eject(out, err, &file.root, &ejected),
    };
    if let Some(code) = answer::written(result, err) {
        return code;
    }
    if ejected.refused.is_empty() {
        exit::OK
    } else {
        exit::FAILURE
    }
}

/// `to`, a folder from the current folder, as a path from the project's
/// folder, when it's inside it.
fn folder_in_project(root: &Path, to: &Path) -> Option<RelPath> {
    let cwd = std::env::current_dir().ok()?;
    let root = normalize(&std::path::absolute(root).ok()?);
    relative_path(&root, &normalize(&cwd.join(to))).filter(RelPath::is_inside)
}

fn write_eject(
    out: &mut dyn Write,
    err: &mut dyn Write,
    root: &Path,
    ejected: &EjectAnswer,
) -> io::Result<()> {
    if !ejected.refused.is_empty() {
        return write_refused(err, ejected);
    }
    for path in &ejected.written {
        writeln!(out, "wrote {}", from_here(root, path))?;
    }
    write_added(out, root, ejected, true)
}

/// What eject refused, on standard error.
pub(crate) fn write_refused(err: &mut dyn Write, ejected: &EjectAnswer) -> io::Result<()> {
    for refusal in &ejected.refused {
        writeln!(err, "error: {}: {}", refusal.name, refusal.message)?;
    }
    writeln!(err, "wrote nothing")
}

/// The most pages that name no type the text lists; `--format json` lists
/// them all.
const PAGES_LISTED: usize = 10;

/// The types eject added, and how to start a page of one. With `untyped`,
/// the project's pages that name no type are listed; a project just started
/// has none to list.
pub(crate) fn write_added(
    out: &mut dyn Write,
    root: &Path,
    ejected: &EjectAnswer,
    untyped: bool,
) -> io::Result<()> {
    if ejected.page_declared {
        writeln!(
            out,
            "added [types.page] to {}: it declared no types, so every page was of this one, and a page that isn't of a new type names it, with `type: page`",
            from_here(root, ascribe_check::MODEL_FILE)
        )?;
    }
    for added in &ejected.added {
        write!(
            out,
            "added [types.{}] to {}",
            added.name,
            from_here(root, ascribe_check::MODEL_FILE),
        )?;
        if added.copied.is_empty() {
            writeln!(out)?;
        } else {
            writeln!(
                out,
                ", with {} declared as the project's other types declare {}",
                listed(&added.copied),
                if added.copied.len() == 1 {
                    "it"
                } else {
                    "them"
                }
            )?;
        }
    }
    let pages = &ejected.untyped;
    if untyped && !ejected.added.is_empty() && !pages.is_empty() {
        writeln!(
            out,
            "{} no type. In a project whose {} declares types, each page says what type it is, with `type:` in its frontmatter; `ascribe check` offers a fix for each type:",
            if pages.len() == 1 {
                "1 page names".to_owned()
            } else {
                format!("{} pages name", pages.len())
            },
            from_here(root, ascribe_check::MODEL_FILE)
        )?;
        for page in pages.iter().take(PAGES_LISTED) {
            writeln!(out, "  {}", from_here(root, page))?;
        }
        if pages.len() > PAGES_LISTED {
            writeln!(
                out,
                "  and {} more, which `--format json` lists",
                pages.len() - PAGES_LISTED
            )?;
        }
    }
    // In `templates/`, the lock eject wrote gives each type its files, and
    // a type kept isn't changed.
    let lock = ascribe_check::type_files::lock_path();
    let from_lock = ejected.written.iter().any(|w| w == lock.as_str());
    for kept in ejected.kept.iter().filter(|_| from_lock) {
        writeln!(
            out,
            "[types.{}] in {} takes its template and guide from {}, and is left as it was",
            kept.name,
            from_here(root, ascribe_check::MODEL_FILE),
            from_here(root, lock.as_str()),
        )?;
    }
    for kept in ejected.kept.iter().filter(|_| !from_lock) {
        writeln!(
            out,
            "added `template` and `guide` to [types.{}] in {}, and left {} as {}",
            kept.name,
            from_here(root, ascribe_check::MODEL_FILE),
            listed(&kept.left),
            if kept.left.len() <= 1 {
                "it was"
            } else {
                "they were"
            }
        )?;
    }
    let start = |template: &str, guide: &[String]| {
        format!(
            "To start a page, copy {} to where the page goes, and read {} first. The page names its type in its frontmatter, as the template does.",
            from_here(root, template),
            guide
                .iter()
                .map(|g| from_here(root, g))
                .collect::<Vec<_>>()
                .join(" and ")
        )
    };
    match (ejected.added.as_slice(), ejected.kept.as_slice()) {
        ([], []) => {}
        ([], [one]) => writeln!(out, "{}", start(&one.template, &one.guide))?,
        ([one], []) => writeln!(out, "{}", start(&one.template, &one.guide))?,
        _ => writeln!(
            out,
            "To start a page, copy its type's template to where the page goes, and read its guide and Ascribe's notes on it first. The page names its type in its frontmatter, as the template does."
        )?,
    }
    Ok(())
}

/// `keys` as a person reads a list of them: "`template`, `purpose`, and
/// `frontmatter`".
fn listed(keys: &[String]) -> String {
    let keys: Vec<String> = keys.iter().map(|k| format!("`{k}`")).collect();
    match keys.as_slice() {
        [] => "the rest".to_owned(),
        [one] => one.clone(),
        [a, b] => format!("{a} and {b}"),
        [rest @ .., last] => format!("{}, and {last}", rest.join(", ")),
    }
}

/// Exit codes: 0 whatever it finds; 1 with `--exit-code` when the set has
/// changed a template the project took, or no longer has one, or when `--diff` names one of the
/// set's the project hasn't taken; 2 when it couldn't run, or `--diff` names
/// a template the set doesn't have.
fn run_status(global: &Global, args: &StatusArgs, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let config = match locate(global) {
        Ok(config) => config,
        Err(e) => return answer::fail(err, &Failure::Config(e)),
    };
    let root = match config.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.to_owned(),
        _ => PathBuf::from("."),
    };
    // A template or guide that's gone is something status reports, so the
    // check of the types' files (`type_files::check`) doesn't stop it; every
    // other rule of the loader, the folders it names included, still does.
    let model = match Project::load_model(&config) {
        Ok(file) => file.model,
        Err(LoadError::Model { text, diagnostics }) => {
            match ascribe_model::load_str_bounded(
                &text,
                ascribe_core::FileId::new(0),
                &root,
                ascribe_model::SourceBound::Anywhere,
            ) {
                Ok(model) => model,
                Err(_) => {
                    let failure = Failure::Load(LoadError::Model { text, diagnostics });
                    return answer::report_failure(err, &failure);
                }
            }
        }
        Err(e) => return answer::report_failure(err, &Failure::Load(e)),
    };
    let fs = DiskFs::new(&root, &Layout::from_model(&model));
    let upstream = Upstream::carried();
    let status = ascribe_templates::status(
        &ascribe_templates::Status {
            model: &model,
            upstream: &upstream,
        },
        &fs,
    );
    if let Some(name) = &args.diff {
        return run_diff(args, &root, &status, name, &upstream, &fs, out, err);
    }
    let result = match args.format {
        Format::Json => answer::write_json(out, &status),
        Format::Text => write_status(out, &root, &status),
    };
    if let Some(code) = answer::written(result, err) {
        return code;
    }
    if args.exit_code && status.upstream_changed {
        exit::PROBLEMS
    } else {
        exit::OK
    }
}

/// `--diff`: the change to one template.
#[allow(clippy::too_many_arguments)]
fn run_diff(
    args: &StatusArgs,
    root: &Path,
    status: &StatusAnswer,
    name: &str,
    upstream: &Upstream,
    fs: &DiskFs,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    let Some(diffs) = ascribe_templates::diff(status, name, upstream, fs) else {
        if status.not_taken.iter().any(|t| t.name == name) {
            let _ = writeln!(
                err,
                "error: the project hasn't taken `{name}`: `ascribe templates eject {name}` writes it"
            );
            return exit::PROBLEMS;
        }
        let known: Vec<String> = upstream
            .set
            .templates
            .iter()
            .map(|t| format!("`{}`", t.name))
            .collect();
        let error = TemplatesError::Unknown {
            name: name.to_owned(),
            known: known.join(", "),
        };
        return answer::fail(err, &error);
    };
    let result = (|| {
        if diffs.is_empty() {
            return writeln!(
                err,
                "{name}: nothing has changed in the set since the copy; this ascribe's set is at {}",
                status.set.tag
            );
        }
        for diff in &diffs {
            let path = from_here(root, &diff.path);
            match diff.project {
                Change::Changed => writeln!(
                    err,
                    "note: {path} isn't as eject wrote it, so the diff undoes the project's changes too"
                )?,
                Change::Unknown => writeln!(
                    err,
                    "note: templates.lock remembers nothing of {path}, so the diff may undo changes the project made"
                )?,
                Change::Unchanged | Change::Deleted => {}
            }
            write!(out, "{}", diff.diff)?;
        }
        Ok(())
    })();
    if let Some(code) = answer::written(result, err) {
        return code;
    }
    let changed = status
        .templates
        .iter()
        .any(|t| t.name == name && t.upstream.is_change());
    if args.exit_code && changed {
        exit::PROBLEMS
    } else {
        exit::OK
    }
}

/// The status, for people: the set's release, then each template the
/// project took, with each file that has something to say, then the ones it
/// didn't take.
fn write_status(out: &mut dyn Write, root: &Path, status: &StatusAnswer) -> io::Result<()> {
    writeln!(
        out,
        "This ascribe's templates are The Good Docs Project's, {}, at {}.",
        status.set.repository, status.set.tag
    )?;
    for template in &status.templates {
        writeln!(out, "{}: {}", template.name, template_line(root, template))?;
        for file in &template.files {
            let Some(line) = file_line(template.record, file) else {
                continue;
            };
            writeln!(out, "  {}: {line}", from_here(root, &file.path))?;
            for section in file.sections.iter().flatten() {
                let heading = section
                    .heading
                    .as_deref()
                    .map_or_else(|| "the start".to_owned(), |h| format!("\"{h}\""));
                let change = match section.change {
                    SectionChangeKind::Added => "added",
                    SectionChangeKind::Removed => "removed",
                    SectionChangeKind::Changed => "changed",
                };
                writeln!(out, "    {change}: {heading}")?;
            }
        }
    }
    if !status.not_taken.is_empty() {
        let names: Vec<&str> = status.not_taken.iter().map(|t| t.name.as_str()).collect();
        writeln!(
            out,
            "Not taken: {}. `ascribe templates eject <NAME>` writes one.",
            names.join(", ")
        )?;
    }
    if status.upstream_changed {
        writeln!(
            out,
            "To see a change, run `ascribe templates status --diff <NAME>`. Nothing was changed."
        )?;
    }
    Ok(())
}

/// What's said of a template, after its name.
fn template_line(root: &Path, template: &TemplateStatus) -> String {
    let lock = from_here(root, &template.lock);
    let unknown = "so what changed in the set can't be told from what changed here";
    match template.record {
        Record::NoLock => format!("there's no {lock}, {unknown}"),
        Record::NoEntry => format!("{lock} has no entry for it, {unknown}"),
        Record::InvalidLock => format!(
            "{lock} can't be read ({}), {unknown}",
            template
                .lock_problem
                .as_deref()
                .unwrap_or("it isn't one eject wrote")
        ),
        Record::Entry => {
            let from = template
                .copied_from
                .as_deref()
                .map_or_else(String::new, |tag| format!(" from {tag}"));
            let upstream = match template.upstream {
                UpstreamChange::Changed => format!("changed in the set since the copy{from}"),
                UpstreamChange::Removed => "the set no longer has it".to_owned(),
                UpstreamChange::Unchanged | UpstreamChange::Unknown => {
                    format!("unchanged in the set since the copy{from}")
                }
            };
            let project = if template.project == Change::Unchanged {
                "the project's copy is as written"
            } else {
                "the project has changed its copy"
            };
            format!("{upstream}; {project}")
        }
    }
}

/// What's said of a file, when there's anything to say.
fn file_line(record: Record, file: &FileStatus) -> Option<String> {
    if file.project == Change::Deleted {
        return Some("deleted".to_owned());
    }
    if record != Record::Entry {
        return Some(match file.matches_set {
            Some(true) => "the same as the set's".to_owned(),
            Some(false) => "different from the set's".to_owned(),
            None => "not in the set".to_owned(),
        });
    }
    let upstream = file.upstream == UpstreamChange::Changed;
    let here = file.project == Change::Changed;
    match (upstream, here) {
        (true, true) => Some("changed in the set, and here".to_owned()),
        (true, false) => Some("changed in the set".to_owned()),
        (false, true) => Some("changed here".to_owned()),
        (false, false) => None,
    }
}
