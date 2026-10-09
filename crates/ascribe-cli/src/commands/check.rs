//! `ascribe check`: report every file-level and page-level problem, without
//! building; with paths, only what counts for them.

use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::rc::Rc;

use ascribe_check::prompt::{self, Builds};
use ascribe_check::{
    Diagnosed, Diagnostic, LoadError, Project, Reported, Scope, ScopeError, diagnose,
    diagnose_editor_build_in, locate_for,
};
use ascribe_core::Coded;
use ascribe_core::path::relative_path;
use clap::{Args as ClapArgs, ValueEnum};

use crate::answer::{FromDisk, Loaded, Projects};
use crate::cli::Global;
use crate::context::{stdout_is_terminal, use_color};
use crate::exit;
use crate::report::json::About;
use crate::report::{Counts, FileTable, concise, json, tally, text};
use crate::shell::quote;

/// Arguments of `ascribe check`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Files and directories to report on, relative to the current
    /// directory.
    ///
    /// The whole project is still checked, since links, includes, and ids
    /// need it, and only the diagnostics that count for these paths are
    /// shown: those in a file under one of them, or with a related place in
    /// one (a problem a fragment causes on the page that includes it). Such a
    /// problem in a fragment is shown once, at its first include. Without
    /// `--config`, the project is the nearest `ascribe.toml` at or above the
    /// first path, and every path must be in it. By default, the whole
    /// project.
    #[arg(conflicts_with = "stdin")]
    pub paths: Vec<PathBuf>,

    /// Check standard input as the text of the file `--path` names.
    ///
    /// The text is laid over the project on disk, which isn't changed; the
    /// file doesn't have to exist. Only the diagnostics that count for it are
    /// shown.
    #[arg(long, requires = "path")]
    pub stdin: bool,

    /// The file standard input is checked as, relative to the current
    /// directory. Only with `--stdin`.
    #[arg(long, requires = "stdin", value_name = "PATH")]
    pub path: Option<PathBuf>,

    /// Check only this build.
    ///
    /// Repeat it for several. By default, every build in `ascribe.toml`. An
    /// unknown build name is a usage error, and the message lists the builds.
    #[arg(long, value_name = "NAME")]
    pub build: Vec<String>,

    /// Run the page-level checks of the editor's build only, as the editor
    /// does as you type.
    ///
    /// The editor's build is `[editor] build` in `ascribe.toml`, or the
    /// first build. With paths that name files, only those files and the
    /// pages that include them are checked, which is quick: for a check
    /// after every edit. A full `ascribe check` still covers every build.
    #[arg(long, conflicts_with = "build")]
    pub editor_build: bool,

    /// Show how many diagnostics each code and each file has, most first,
    /// instead of listing them.
    #[arg(long)]
    pub summary: bool,

    /// How to show the results.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,

    /// Make warnings fail the command too (exit code 1), for CI.
    #[arg(long)]
    pub deny_warnings: bool,
}

/// The output format.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Format {
    /// Diagnostics with source snippets, for people.
    Text,
    /// One line per diagnostic, `file:line: [code] message`, grouped by file:
    /// for an agent. At most 50, then how many more and the command that
    /// narrows the check.
    Concise,
    /// One JSON document, for tools.
    Json,
    /// A prompt for an agent that fixes the problems: about the file, when
    /// the paths name one file, else about the project or the paths. Nothing
    /// when there are no problems.
    Prompt,
}

/// Why the paths, or standard input, can't be checked.
#[derive(Debug, thiserror::Error)]
pub enum CheckError {
    /// A path can't be checked.
    #[error(transparent)]
    Scope(#[from] ScopeError),
    /// Standard input can't be read as text.
    #[error("can't read standard input: {0}")]
    Stdin(io::Error),
}

impl Coded for CheckError {
    fn code(&self) -> &'static str {
        match self {
            CheckError::Scope(e) => e.code(),
            CheckError::Stdin(_) => "stdin_unreadable",
        }
    }
}

/// Runs the command. Exit codes: 0 with no errors, 1 with errors (or with
/// warnings under `--deny-warnings`), 2 when the project can't be checked.
pub fn run(global: &Global, args: Args) -> ExitCode {
    // Buffered: a report is many small writes, and a locked stdout flushes
    // at every line.
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let stderr = io::stderr();
    let mut err = stderr.lock();
    let code = check(global, &args, &mut io::stdin(), &mut out, &mut err);
    let _ = out.flush();
    exit::code(code)
}

/// The project, and the scope of the paths named, if any.
struct Checked {
    loaded: Rc<Loaded>,
    scope: Option<Scope>,
}

/// What a check found, ready to write in any format.
pub struct Outcome {
    loaded: Rc<Loaded>,
    scope: Option<Scope>,
    reported: Vec<Reported>,
    builds_checked: Vec<String>,
    files: FileTable,
    /// How many errors and warnings are reported.
    pub counts: Counts,
}

impl Outcome {
    /// What's reported, the files it's in, and the first build checked.
    pub fn parts(&self) -> (&[Reported], &FileTable, Option<&str>) {
        (
            &self.reported,
            &self.files,
            self.builds_checked.first().map(String::as_str),
        )
    }
}

/// Why a check couldn't be made.
pub enum Stopped {
    /// The project couldn't be found or loaded, or the paths can't be
    /// checked.
    Failure(Failure),
    /// A build named isn't the content model's.
    Build(ascribe_check::UnknownBuild),
}

fn check(
    global: &Global,
    args: &Args,
    stdin: &mut dyn Read,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    let color = use_color(global, stdout_is_terminal());
    let outcome = match run_check(&FromDisk, global, args, stdin) {
        Ok(outcome) => outcome,
        Err(stopped) => return report_stopped(stopped, args, color, out, err),
    };
    if let Err(e) = write(&outcome, global, args, color, out) {
        // A closed pipe (`| head`) isn't a problem with the project.
        if e.kind() != io::ErrorKind::BrokenPipe {
            let _ = writeln!(err, "error: can't write the report: {e}");
            return exit::FAILURE;
        }
    }
    let counts = outcome.counts;
    if counts.errors > 0 || (args.deny_warnings && counts.warnings > 0) {
        exit::PROBLEMS
    } else {
        exit::OK
    }
}

/// Checks the project, and keeps what counts for the paths named: what
/// `ascribe check` reports, before it's written. `stdin` is read with
/// `--stdin`.
///
/// # Errors
///
/// The project can't be found or loaded, the paths can't be checked, or a
/// build named isn't the content model's.
pub fn run_check(
    projects: &dyn Projects,
    global: &Global,
    args: &Args,
    stdin: &mut dyn Read,
) -> Result<Outcome, Stopped> {
    let Checked { loaded, scope } =
        load(projects, global, args, stdin).map_err(Stopped::Failure)?;
    let project = &loaded.project;
    let names = |d: Diagnosed<'_>| {
        let builds = d.builds.iter().map(|b| b.name.clone()).collect();
        (d.diagnostics, builds)
    };
    let (diagnostics, builds_checked) = if args.editor_build {
        let files = scope.as_ref().and_then(|s| s.named_sources(project));
        names(diagnose_editor_build_in(
            project,
            Some(loaded.page_index()),
            files.as_deref(),
        ))
    } else {
        loaded
            .diagnosed(&args.build, |project| {
                diagnose(project, &args.build).map(names)
            })
            .map_err(Stopped::Build)?
    };
    let reported = match &scope {
        Some(scope) => scope.report(project, diagnostics),
        None => Reported::all(diagnostics),
    };
    let files = FileTable::of_project(project);
    let counts = Counts::of_reported(&reported);
    Ok(Outcome {
        loaded: Rc::clone(&loaded),
        scope,
        reported,
        builds_checked,
        files,
        counts,
    })
}

/// Writes what a check found in `args.format`, as `ascribe check` does.
///
/// # Errors
///
/// Writing to `out` failed.
pub fn write(
    outcome: &Outcome,
    global: &Global,
    args: &Args,
    color: bool,
    out: &mut dyn Write,
) -> io::Result<()> {
    let Outcome {
        loaded,
        scope,
        reported,
        builds_checked,
        files,
        counts,
    } = outcome;
    let project = &loaded.project;
    let checked = project.sources().len();
    let files_reported = scope.as_ref().map(|s| s.sources(project).len());
    let summary = text::check_summary(
        *counts,
        checked,
        files_reported,
        args.editor_build
            .then(|| builds_checked.first().map(String::as_str))
            .flatten(),
    );
    let command = Command::new(global, args);
    match (args.format, args.summary) {
        (Format::Json, summary_only) => json::write(
            out,
            files,
            reported,
            &About {
                error: None,
                files_checked: checked,
                files_reported: files_reported.unwrap_or(checked),
                builds_checked: builds_checked.clone(),
                summary_only: summary_only.then(|| command.listing()),
            },
        ),
        (Format::Prompt, _) => write_prompt(out, project, args, scope.as_ref(), reported),
        (Format::Text | Format::Concise, true) => {
            let (codes, by_file) = tally(files, reported);
            text::write_tally(out, &codes, &by_file).and_then(|()| writeln!(out, "{summary}"))
        }
        (Format::Text, false) => text::write_reported(out, files, reported, &summary, color),
        (Format::Concise, false) => {
            let more = |next: &Reported| {
                command.narrowed(project, &files.path(next.diagnostic.location.file), scope)
            };
            concise::write(out, files, reported, &summary, &more)
        }
    }
}

/// Writes the prompt for an agent about what's reported: about one file
/// when the paths name one source file (or with `--stdin`, whose text isn't
/// saved there), else about the paths or the project. Nothing when nothing
/// is reported.
fn write_prompt(
    out: &mut dyn Write,
    project: &Project,
    args: &Args,
    scope: Option<&Scope>,
    reported: &[Reported],
) -> io::Result<()> {
    let builds = if args.editor_build {
        Builds::Editor(project.model().editor_default_build().name.clone())
    } else if args.build.is_empty() {
        Builds::All
    } else {
        Builds::Named(args.build.clone())
    };
    let mut context = prompt::Context::of_project(project.root(), builds);
    let named = scope.and_then(|s| s.named_sources(project));
    let one = named.as_deref().and_then(|files| match files {
        [file] => project.source_at(file),
        _ => None,
    });
    let index = std::cell::OnceCell::new();
    let shown_on = |fragment: &ascribe_core::RelPath| {
        index
            .get_or_init(|| project.held_index())
            .including_pages(fragment)
    };
    let text = match one {
        Some(file) => {
            if args.stdin {
                let path = project.display_path(file.id).unwrap_or_default();
                context = context.with_unsaved(vec![path]);
            }
            prompt::file(project, &context, file.id, reported, &shown_on)
        }
        None => {
            let paths = scope.map(Scope::paths).unwrap_or_default();
            prompt::project(project, &context, paths, reported)
        }
    };
    match text {
        Some(text) => out.write_all(text.as_bytes()),
        None => Ok(()),
    }
}

/// Why nothing could be checked.
pub enum Failure {
    /// The paths, standard input, or finding the content model.
    Check(CheckError),
    /// Loading the project.
    Load(LoadError),
}

/// Finds the content model, loads the project, lays standard input over it
/// with `--stdin`, and works out the scope of the paths named.
fn load(
    projects: &dyn Projects,
    global: &Global,
    args: &Args,
    stdin: &mut dyn Read,
) -> Result<Checked, Failure> {
    let named: Vec<PathBuf> = match &args.path {
        Some(path) if args.stdin => vec![path.clone()],
        _ => args.paths.clone(),
    };
    let check = |e: ScopeError| Failure::Check(CheckError::Scope(e));
    let config = locate_for(global.config.as_deref(), &named).map_err(check)?;
    let loaded = projects.load(&config).map_err(Failure::Load)?;
    let project = &loaded.project;
    match &args.path {
        Some(path) if args.stdin => {
            let content_path = Scope::source_path(project, &config, path).map_err(check)?;
            let mut text = String::new();
            stdin
                .read_to_string(&mut text)
                .map_err(|e| Failure::Check(CheckError::Stdin(e)))?;
            let project = project.with_source(&content_path, text);
            let scope = project
                .content_root()
                .join(content_path.as_str())
                .map(|p| Scope::new([p]))
                .ok();
            Ok(Checked {
                loaded: Rc::new(Loaded::new(project)),
                scope,
            })
        }
        _ if named.is_empty() => Ok(Checked {
            loaded,
            scope: None,
        }),
        _ => {
            let scope = Scope::of_paths(project, &config, &named, true).map_err(check)?;
            Ok(Checked {
                loaded,
                scope: Some(scope),
            })
        }
    }
}

/// How the command was run, to name the command that shows what a report
/// leaves out.
struct Command<'a> {
    global: &'a Global,
    args: &'a Args,
}

impl<'a> Command<'a> {
    fn new(global: &'a Global, args: &'a Args) -> Command<'a> {
        Command { global, args }
    }

    /// The options that choose what's checked: `--config`, `--build`, and
    /// `--editor-build`, as given.
    fn options(&self) -> Vec<String> {
        let mut words = Vec::new();
        if let Some(config) = &self.global.config {
            words.push("--config".to_owned());
            words.push(quote(&config.to_string_lossy()));
        }
        for build in &self.args.build {
            words.push("--build".to_owned());
            words.push(quote(build));
        }
        if self.args.editor_build {
            words.push("--editor-build".to_owned());
        }
        words
    }

    /// This command without `--summary`, writing JSON: what lists every
    /// diagnostic a summary counts.
    fn listing(&self) -> String {
        let mut words = vec!["ascribe".to_owned(), "check".to_owned()];
        match &self.args.path {
            Some(path) if self.args.stdin => {
                words.push("--stdin".to_owned());
                words.push("--path".to_owned());
                words.push(quote(&path.to_string_lossy()));
            }
            _ => words.extend(self.args.paths.iter().map(|p| quote(&p.to_string_lossy()))),
        }
        words.extend(self.options());
        words.extend(["--format".to_owned(), "json".to_owned()]);
        words.join(" ")
    }

    /// The command that narrows the check to `file` (relative to the project
    /// root), in concise form; or, when the check is already that file
    /// alone, the one that lists all of it, as JSON.
    fn narrowed(&self, project: &Project, file: &str, scope: &Option<Scope>) -> String {
        let only_this = scope.as_ref().is_some_and(|s| {
            s.paths().len() == 1 && s.paths().first().is_some_and(|p| p.as_str() == file)
        });
        if only_this {
            return self.listing();
        }
        let mut words = vec!["ascribe".to_owned(), "check".to_owned()];
        words.push(quote(&from_here(project.root(), file)));
        words.extend(self.options());
        words.extend(["--format".to_owned(), "concise".to_owned()]);
        words.join(" ")
    }
}

/// `file`, relative to the project root, as a path from the current
/// directory, `/`-separated.
fn from_here(root: &Path, file: &str) -> String {
    let target = root.join(file);
    let here = std::env::current_dir().ok();
    let absolute = std::path::absolute(&target).unwrap_or_else(|_| target.clone());
    here.and_then(|here| relative_path(&here, &absolute))
        .map_or_else(
            || absolute.to_string_lossy().replace('\\', "/"),
            |rel| {
                if rel.is_root() {
                    ".".to_owned()
                } else {
                    rel.to_string()
                }
            },
        )
}

/// Reports a check that couldn't be made, and returns the exit code: as
/// [`report_failure`] does, or for an unknown build its message.
pub fn report_stopped(
    stopped: Stopped,
    args: &Args,
    color: bool,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    match stopped {
        Stopped::Failure(failure) => report_failure(failure, args, color, out, err),
        Stopped::Build(e) => exit::fail(err, &e),
    }
}

/// Reports a project that couldn't be checked, and returns the exit code.
///
/// A content model with errors is a configuration
/// failure (exit code 2), and its diagnostics are shown.
fn report_failure(
    failure: Failure,
    args: &Args,
    color: bool,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    let (message, files, diagnostics): (String, FileTable, Vec<Diagnostic>) = match failure {
        Failure::Check(e) => (
            e.to_string(),
            FileTable::of_model(String::new()),
            Vec::new(),
        ),
        Failure::Load(LoadError::Model { text, diagnostics }) => (
            format!(
                "{} has errors, so nothing can be checked",
                ascribe_check::MODEL_FILE
            ),
            FileTable::of_model(text),
            diagnostics,
        ),
        Failure::Load(e @ LoadError::Read { .. }) => (
            e.to_string(),
            FileTable::of_model(String::new()),
            Vec::new(),
        ),
    };
    let _ = match args.format {
        Format::Prompt => writeln!(err, "error: {message}"),
        Format::Text => {
            let _ = text::write_diagnostics(out, &files, &diagnostics, color);
            writeln!(err, "error: {message}")
        }
        Format::Concise => {
            for r in Reported::all(diagnostics) {
                let _ = concise::write_line(out, &files, &r);
            }
            writeln!(err, "error: {message}")
        }
        Format::Json => json::write(
            out,
            &files,
            &Reported::all(diagnostics),
            &About::failed(&message),
        ),
    };
    exit::FAILURE
}
