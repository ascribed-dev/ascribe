//! `ascribe check`: report every file-level and page-level problem, without
//! building; with paths, only what counts for them.

use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;
use std::rc::Rc;

use ascribe_check::baseline;
use ascribe_check::fix;
use ascribe_check::prompt::{self, Builds};
use ascribe_check::prose;
use ascribe_check::{
    Acknowledged, BaselineError, Diagnosed, Diagnostic, LoadError, Project, Reported, Scope,
    ScopeError, Severity, diagnose, diagnose_editor_build_in, locate_for,
};
use ascribe_core::Coded;
use clap::{Args as ClapArgs, ValueEnum};

use crate::answer::{Found, FromDisk, Loaded, Projects};
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
    /// need it, and only the problems that count for these paths are
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
    /// file doesn't have to exist. Only the problems that count for it are
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

    /// Show only the problems with this code (`ASC036`) or name
    /// (`link-target-missing`), as `ascribe explain` takes it.
    ///
    /// Repeat it for several. The whole project is still checked, and the
    /// rest is left out of the report, as with paths: the counts, the exit
    /// code, a prompt, and the fixes `--fix` applies are about what's left.
    /// An unknown code is a usage error, and the message names the closest.
    #[arg(long, value_name = "CODE")]
    pub code: Vec<String>,

    /// Show how many problems each code and each file has, most first,
    /// instead of listing them, then the command that lists the first code's
    /// problems.
    #[arg(long)]
    pub summary: bool,

    /// How to show the results.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,

    /// List at most this many problems with `--format concise`, or `all` of
    /// them.
    ///
    /// By default, 50. When some are left out, the list ends with how many,
    /// and the command that lists them all. Only with `--format concise`, and
    /// not with `--summary`, which lists none.
    #[arg(long, value_name = "N", conflicts_with = "summary")]
    pub limit: Option<concise::Limit>,

    /// Make warnings fail the command too (exit code 1), for CI.
    #[arg(long)]
    pub deny_warnings: bool,

    /// The revision whose addresses the check compares the project's with,
    /// when it has `ascribe.addresses`: anything git accepts (a branch, a
    /// tag, a commit).
    ///
    /// An address the base published that no build publishes now, and that
    /// no entry of `ascribe.addresses` accounts for, is `address-lost`. By
    /// default, the repository's default branch, the first of `origin/HEAD`,
    /// `origin/main`, `origin/master`, `main`, and `master` that exists,
    /// from its merge base with `HEAD`, as `ascribe diff` chooses it. A base
    /// that can't be read is `addresses-base-unreadable`, and the rest of the
    /// check is the same. Only when the check covers the whole project:
    /// with paths, `--stdin`, or `--editor-build`, the base isn't read.
    #[arg(long, value_name = "REV")]
    pub base: Option<String>,

    /// Compare the addresses with the base revision itself instead of its
    /// merge base with `HEAD`: in CI on a push, the commit before it.
    #[arg(long)]
    pub base_exact: bool,

    /// Whether the addresses are compared with the base when the check
    /// covers the whole project: by `ascribe check` and the MCP server's
    /// tool, and never by the agents' hook (decision #905).
    #[arg(skip)]
    pub addresses: Addresses,

    /// Apply the fixes marked safe, then report what's left.
    ///
    /// A fix is safe when applying it can't change what the page says and
    /// leaves nothing to decide, such as linking to a page's file instead of
    /// its route. Only the fixes of the diagnostics reported are applied, so
    /// with paths, those that count for them. Unsafe fixes are never applied.
    /// The report, and the exit code, are the check after the fixes.
    #[arg(long, conflicts_with = "stdin")]
    pub fix: bool,

    /// Check the prose with Vale too, as `[checks.vale]` in `ascribe.toml`
    /// sets it up.
    ///
    /// Vale runs once, on the files reported on. Its alerts are `prose`
    /// diagnostics, at their place in the source. A Vale that can't be run
    /// is one `prose-not-checked` advice, and the rest of the check is the
    /// same. `[checks.vale] in-check = true` checks the prose without this
    /// option. Without `[checks.vale]`, it's a usage error.
    #[arg(long)]
    pub vale: bool,

    /// Report the advice `ascribe.baseline` holds back too, without reading
    /// the file.
    ///
    /// By default, a problem the baseline lists isn't reported while it's
    /// advice, and is counted instead, and an entry that matches nothing is
    /// reported, as `baseline-entry-unmatched`. A baseline that can't be read
    /// stops the check, unless this is given.
    #[arg(long)]
    pub no_baseline: bool,

    /// How long Vale may take, when it runs: the hooks give it less than
    /// they wait. `None` is the time a check of the whole project has.
    #[arg(skip)]
    pub vale_timeout: Option<std::time::Duration>,

    /// Which Vale the check may start, when it checks the prose: any, for
    /// `ascribe check`; less for the agents, which no person approves
    /// (decision 76).
    #[arg(skip)]
    pub vale_runs: ValeRuns,
}

/// Whether a check compares the project's addresses with the base's.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Addresses {
    /// When the project has `ascribe.addresses` and the check covers the
    /// whole project: `ascribe check`, and the MCP server's `ascribe_check`.
    #[default]
    Compared,
    /// Never: the agents' hook and prompts, and `ascribe baseline`.
    NotCompared,
}

/// Which Vale a check may start.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ValeRuns {
    /// Whatever `[checks.vale] command` names: `ascribe check`, which a
    /// person runs.
    #[default]
    Any,
    /// Only a bare name found on the `PATH`, never a program in the
    /// project: the agents' hook, which can't ask anyone.
    OnPath,
    /// None: the MCP server's tools, which say they only read, and the
    /// agents' prompts.
    Never,
}

/// The output format.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Format {
    /// Problems with source snippets, for people.
    Text,
    /// One line per problem, `file:line: [code] message`, grouped by file:
    /// for an agent. At most 50, or as many as `--limit` says, then how many
    /// more and the command that lists them all.
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
    /// `--vale` was given to a project that doesn't set Vale up.
    #[error("--vale checks the prose as `[checks.vale]` sets it up, and ascribe.toml has none")]
    NoVale,
    /// `--fix` couldn't write a file it fixed.
    #[error(transparent)]
    Fix(#[from] fix::FixError),
    /// `--code` names no diagnostic.
    #[error("--code: {0}")]
    Code(ascribe_query::QueryError),
    /// `ascribe.baseline` can't be read.
    #[error("{0}; fix the line, or pass --no-baseline to check without the baseline")]
    Baseline(BaselineError),
    /// `--limit` was given to a format that lists every problem.
    #[error("--limit sets how many problems `--format concise` lists; add `--format concise`")]
    LimitNotConcise,
}

impl Coded for CheckError {
    fn code(&self) -> &'static str {
        match self {
            CheckError::Scope(e) => e.code(),
            CheckError::Stdin(_) => "stdin_unreadable",
            CheckError::NoVale => "vale_not_set_up",
            CheckError::Fix(e) => e.code(),
            CheckError::Code(e) => e.code(),
            CheckError::Baseline(e) => e.code(),
            CheckError::LimitNotConcise => "limit_not_concise",
        }
    }
}

/// Runs the command. Exit codes: 0 with no errors, 1 with errors (or with
/// warnings under `--deny-warnings`), 2 when the project can't be checked.
/// Advice never changes the exit code.
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

/// The project, its content model's file, and the scope of the paths named,
/// if any.
struct Checked {
    loaded: Rc<Loaded>,
    config: PathBuf,
    scope: Option<Scope>,
}

/// What a check found, ready to write in any format.
pub struct Outcome {
    loaded: Rc<Loaded>,
    /// The content model's file.
    config: PathBuf,
    scope: Option<Scope>,
    reported: Vec<Reported>,
    builds_checked: Vec<String>,
    /// The problems acknowledged as intended in the files reported on.
    acknowledged: Vec<Acknowledged>,
    /// How many problems `ascribe.baseline` holds back in the files
    /// reported on, of the codes named.
    baselined: usize,
    files: FileTable,
    /// The codes `--code` names, as the registry writes them, in the order
    /// given; empty for every code.
    codes: Vec<String>,
    /// With `--fix`, the safe fixes applied before this check.
    fixed: Vec<json::FixedEntry>,
    /// How many errors, warnings, and advice are reported.
    pub counts: Counts,
}

impl Outcome {
    /// The project checked.
    pub fn project(&self) -> &Project {
        &self.loaded.project
    }

    /// How many problems `ascribe.baseline` holds back in the files
    /// reported on, which the report leaves out.
    pub fn baselined(&self) -> usize {
        self.baselined
    }

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
    let mut outcome = match run_check(&FromDisk, global, args, stdin) {
        Ok(outcome) => outcome,
        Err(stopped) => return report_stopped(stopped, args, color, out, err),
    };
    if args.fix {
        let files = fix::safe_fixes(
            outcome.project(),
            &outcome.config,
            outcome.reported.iter().map(|r| &r.diagnostic),
        );
        if !files.is_empty() {
            if let Err(e) = fix::write(&files) {
                let stopped = Stopped::Failure(Failure::Check(CheckError::Fix(e)));
                return report_stopped(stopped, args, color, out, err);
            }
            // What's left is the check of the files as they are now.
            outcome = match run_check(&FromDisk, global, args, &mut io::empty()) {
                Ok(outcome) => outcome,
                Err(stopped) => return report_stopped(stopped, args, color, out, err),
            };
            outcome.fixed = json::FixedEntry::of(&files);
        }
    }
    if let Err(e) = write(&outcome, global, args, color, out) {
        // A closed pipe (`| head`) isn't a problem with the project.
        if e.kind() != io::ErrorKind::BrokenPipe {
            let _ = writeln!(err, "error: can't write the report: {e}");
            return exit::FAILURE;
        }
    }
    exit_code(outcome.counts, args.deny_warnings)
}

/// The exit code for what a check reports: 1 with errors, or with warnings
/// under `--deny-warnings`; 0 otherwise. Advice never fails the check.
fn exit_code(counts: Counts, deny_warnings: bool) -> u8 {
    if counts.errors > 0 || (deny_warnings && counts.warnings > 0) {
        exit::PROBLEMS
    } else {
        exit::OK
    }
}

/// Checks the project, and keeps what counts for the paths named and has
/// the codes named: what `ascribe check` reports, before it's written.
/// `stdin` is read with `--stdin`.
///
/// # Errors
///
/// A code named isn't a diagnostic's, the project can't be found or loaded,
/// the paths can't be checked, or a build named isn't the content model's.
pub fn run_check(
    projects: &dyn Projects,
    global: &Global,
    args: &Args,
    stdin: &mut dyn Read,
) -> Result<Outcome, Stopped> {
    if args.limit.is_some() && args.format != Format::Concise {
        return Err(Stopped::Failure(Failure::Check(
            CheckError::LimitNotConcise,
        )));
    }
    let codes = codes(&args.code).map_err(Stopped::Failure)?;
    let Checked {
        loaded,
        config,
        scope,
    } = load(projects, global, args, stdin).map_err(Stopped::Failure)?;
    let project = &loaded.project;
    let names = |d: Diagnosed<'_>| Found {
        builds: d.builds.iter().map(|b| b.name.clone()).collect(),
        diagnostics: d.diagnostics,
        acknowledged: d.acknowledged,
    };
    let Found {
        diagnostics,
        builds: builds_checked,
        mut acknowledged,
    } = if args.editor_build {
        let files = scope.as_ref().and_then(|s| s.named_sources(project));
        names(diagnose_editor_build_in(
            project,
            Some(loaded.page_index()),
            files.as_deref(),
        ))
    } else {
        // The base is read beside the check, as `ascribe diff` reads it.
        let compared = args.addresses == Addresses::Compared
            && scope.is_none()
            && !args.stdin
            && matches!(project.addresses(), Some(Ok(_)));
        let (found, base) = std::thread::scope(|threads| {
            let base = compared.then(|| {
                threads.spawn(|| {
                    ascribe_diff::base_addresses(
                        project.root(),
                        args.base.as_deref(),
                        args.base_exact,
                    )
                })
            });
            let found = loaded.diagnosed(&args.build, |project| {
                diagnose(project, &args.build).map(names)
            });
            let base = base.map(|thread| {
                thread
                    .join()
                    .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
            });
            (found, base)
        });
        let mut found = found.map_err(Stopped::Build)?;
        if let Some(base) = base {
            let more = compared_addresses(&loaded, &found.builds, base);
            found
                .diagnostics
                .extend(ascribe_check::apply_levels(&project.model().checks, more));
        }
        found
    };
    let (diagnostics, about_vale) = with_prose(project, scope.as_ref(), args, diagnostics)?;
    let (diagnostics, mut held) = with_baseline(project, scope.as_ref(), args, diagnostics, {
        let in_check = project
            .model()
            .checks
            .vale
            .as_ref()
            .is_some_and(|v| v.in_check);
        about_vale.is_empty() && (args.vale || in_check)
    })?;
    let mut reported = match &scope {
        Some(scope) => scope.report(project, diagnostics),
        None => Reported::all(diagnostics),
    };
    reported.extend(Reported::all(about_vale));
    if let Some(scope) = &scope {
        acknowledged.retain(|a| scope.contains_file(project, a.problem.location.file));
        held.retain(|d| scope.contains_file(project, d.location.file));
    }
    if !codes.is_empty() {
        let named = |code: &str| codes.iter().any(|c| c == code);
        reported.retain(|r| named(r.diagnostic.code));
        acknowledged.retain(|a| named(a.problem.code));
        held.retain(|d| named(d.code));
    }
    // Advice after errors and warnings; stable, so each keeps its order.
    reported.sort_by_key(|r| r.diagnostic.severity == Severity::Advice);
    let files = FileTable::of_project(project);
    let counts = Counts::of_reported(&reported);
    Ok(Outcome {
        loaded: Rc::clone(&loaded),
        config,
        scope,
        reported,
        builds_checked,
        acknowledged,
        baselined: held.len(),
        files,
        codes,
        fixed: Vec::new(),
        counts,
    })
}

/// The codes `given` names, by code or name, as the registry writes them,
/// each once, in the order given.
fn codes(given: &[String]) -> Result<Vec<String>, Failure> {
    let mut codes: Vec<String> = Vec::new();
    for code_or_slug in given {
        let code = ascribe_query::code_of(code_or_slug)
            .map_err(|e| Failure::Check(CheckError::Code(e)))?;
        if !codes.iter().any(|c| c == code) {
            codes.push(code.to_owned());
        }
    }
    Ok(codes)
}

/// The addresses the base published that the project has lost, compared
/// in `builds`, the builds checked; or, when the base couldn't be read, why.
fn compared_addresses(
    loaded: &Loaded,
    builds: &[String],
    base: Result<ascribe_diff::BaseAddresses, ascribe_diff::DiffError>,
) -> Vec<Diagnostic> {
    let Some(Ok(file)) = loaded.project.addresses() else {
        return Vec::new();
    };
    let base = match base {
        Ok(base) => base,
        Err(e) => return vec![ascribe_check::addresses::base_unreadable(&e.to_string())],
    };
    let Some(before) = &base.builds else {
        return vec![ascribe_check::addresses::base_has_no_project(&base.named())];
    };
    // `--build` compares those builds only; a build only the base has is
    // compared, as gone, when every build is checked.
    let every = loaded.project.model().builds.len() == builds.len();
    let before: Vec<_> = before
        .iter()
        .filter(|b| {
            builds.contains(&b.build) || (every && loaded.project.model().build(&b.build).is_none())
        })
        .cloned()
        .collect();
    let index = loaded.index();
    let now: Vec<_> = index
        .model()
        .builds
        .iter()
        .filter(|b| builds.contains(&b.name))
        .map(|b| ascribe_resolve::addresses::build_addresses(index, b))
        .collect();
    ascribe_check::addresses::compare(file, index, &before, &now)
}

/// `diagnostics` with Vale's, when `--vale` or `[checks.vale] in-check` asks
/// for them: Vale runs once, on the files in the scope, or on every source
/// file. What it says about the prose is with the rest; that it couldn't be
/// run, which is about `ascribe.toml`, comes back apart, to be reported
/// whatever the paths named.
fn with_prose(
    project: &Project,
    scope: Option<&Scope>,
    args: &Args,
    mut diagnostics: Vec<Diagnostic>,
) -> Result<(Vec<Diagnostic>, Vec<Diagnostic>), Stopped> {
    let wanted = match &project.model().checks.vale {
        None if args.vale => {
            return Err(Stopped::Failure(Failure::Check(CheckError::NoVale)));
        }
        None => false,
        Some(settings) => args.vale || settings.in_check,
    };
    if !wanted {
        return Ok((diagnostics, Vec::new()));
    }
    let files: Option<Vec<ascribe_core::FileId>> =
        scope.map(|s| s.sources(project).iter().map(|f| f.id).collect());
    let program = prose::Program;
    let on_path = prose::OnPath {
        inner: &program,
        who: "the agents' hook",
    };
    let never =
        prose::NotStarted("Ascribe's tools and prompts for agents don't start Vale".to_owned());
    let linter: &dyn prose::Linter = match args.vale_runs {
        ValeRuns::Any => &program,
        ValeRuns::OnPath => &on_path,
        ValeRuns::Never => &never,
    };
    let found = prose::lint(
        project,
        files.as_deref(),
        linter,
        args.vale_timeout.unwrap_or(prose::PROJECT_TIMEOUT),
    );
    let (about_vale, found): (Vec<Diagnostic>, Vec<Diagnostic>) = found
        .into_iter()
        .partition(|d| d.location.file == ascribe_core::FileId::new(0));
    diagnostics.extend(found);
    Ok((diagnostics, about_vale))
}

/// `diagnostics`, the whole project's, without the advice `ascribe.baseline`
/// holds back, which comes back apart, unless `--no-baseline` says not to
/// read it. An entry that matches nothing is reported when the check covers
/// every build and the whole project, and so would have found what it
/// lists; an entry of Vale's only when `prose_checked` says Vale ran.
fn with_baseline(
    project: &Project,
    scope: Option<&Scope>,
    args: &Args,
    diagnostics: Vec<Diagnostic>,
    prose_checked: bool,
) -> Result<(Vec<Diagnostic>, Vec<Diagnostic>), Stopped> {
    let baseline = match project.baseline() {
        Some(_) if args.no_baseline => return Ok((diagnostics, Vec::new())),
        None => return Ok((diagnostics, Vec::new())),
        Some(Err(e)) => {
            let e = CheckError::Baseline(e.clone());
            return Err(Stopped::Failure(Failure::Check(e)));
        }
        Some(Ok(baseline)) => baseline,
    };
    let whole = scope.is_none() && args.build.is_empty() && !args.editor_build;
    let not_run: &[&str] = if prose_checked {
        &[]
    } else {
        &[ascribe_core::diagnostics::PROSE.as_str()]
    };
    let held = baseline::apply(project, baseline, diagnostics, whole, not_run);
    Ok((held.diagnostics, held.held))
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
        config: _,
        scope,
        reported,
        builds_checked,
        acknowledged,
        baselined,
        files,
        codes,
        fixed,
        counts,
    } = outcome;
    let project = &loaded.project;
    let checked = project.sources().len();
    let files_reported = scope.as_ref().map(|s| s.sources(project).len());
    let covered = match builds_checked.first() {
        Some(build) if args.editor_build => text::Covered::Editor(build),
        _ if args.build.is_empty() => text::Covered::Every,
        _ => text::Covered::Named(builds_checked),
    };
    let summary = text::check_summary(
        *counts,
        checked,
        &covered,
        files_reported.unwrap_or(checked),
        codes,
    ) + &text::acknowledged(acknowledged.len())
        + &text::baselined(*baselined);
    let command = Command::new(global, args, codes);
    if matches!(args.format, Format::Text | Format::Concise) {
        write_fixed(out, fixed)?;
    }
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
                acknowledged,
                baselined: *baselined,
                summary_only: summary_only.then(|| command.listing()),
                fixed,
            },
        ),
        (Format::Prompt, _) => write_prompt(out, project, args, codes, scope.as_ref(), reported),
        (Format::Text | Format::Concise, true) => {
            let (by_code, by_file) = tally(files, reported);
            let first = by_code.first().map(|c| command.of_code(c.code));
            text::write_tally(out, &by_code, &by_file, &command.tally(), first.as_deref())
                .and_then(|()| writeln!(out, "{summary}"))
        }
        (Format::Text, false) => text::write_reported(out, files, reported, &summary, color),
        (Format::Concise, false) => concise::write(
            out,
            files,
            reported,
            &summary,
            args.limit.unwrap_or_default(),
            &command.concise_all(),
        ),
    }
}

/// Writes a line for each file `--fix` changed, with how many problems it
/// fixed there, before what's left is reported.
fn write_fixed(out: &mut dyn Write, fixed: &[json::FixedEntry]) -> io::Result<()> {
    let mut i = 0;
    while let Some(first) = fixed.get(i) {
        let n = fixed[i..]
            .iter()
            .take_while(|f| f.file == first.file)
            .count();
        let problems = if n == 1 { "problem" } else { "problems" };
        writeln!(out, "fixed {n} {problems} in {}", first.file)?;
        i += n;
    }
    Ok(())
}

/// Writes the prompt for an agent about what's reported: about one file
/// when the paths name one source file (or with `--stdin`, whose text isn't
/// saved there), else about the paths or the project. Nothing when nothing
/// is reported.
fn write_prompt(
    out: &mut dyn Write,
    project: &Project,
    args: &Args,
    codes: &[String],
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
    let mut context = prompt::Context::seen_from(
        project.root(),
        std::env::current_dir().ok().as_deref(),
        builds,
    )
    .with_codes(codes.to_vec());
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
                config,
                scope,
            })
        }
        _ if named.is_empty() => Ok(Checked {
            loaded,
            config,
            scope: None,
        }),
        _ => {
            let scope = Scope::of_paths(project, &config, &named, true).map_err(check)?;
            Ok(Checked {
                loaded,
                config,
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
    /// The codes `--code` names, as the registry writes them.
    codes: &'a [String],
}

impl<'a> Command<'a> {
    fn new(global: &'a Global, args: &'a Args, codes: &'a [String]) -> Command<'a> {
        Command {
            global,
            args,
            codes,
        }
    }

    /// The options that choose what's checked and shown: `--config`,
    /// `--build`, `--editor-build`, and `--code`, as given.
    fn options(&self) -> Vec<String> {
        let mut words = self.checked();
        for code in self.codes {
            words.push("--code".to_owned());
            words.push(quote(code));
        }
        words
    }

    /// The options that choose what's checked: `--config`, `--build`, and
    /// `--editor-build`, as given.
    fn checked(&self) -> Vec<String> {
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
        self.with_format(false)
    }

    /// This command with `--summary`, writing JSON: what lists every file a
    /// summary counts.
    fn tally(&self) -> String {
        self.with_format(true)
    }

    /// This command writing JSON, with `--summary` or without.
    fn with_format(&self, summary: bool) -> String {
        let mut words = self.on_paths();
        words.extend(self.options());
        if summary {
            words.push("--summary".to_owned());
        }
        words.extend(["--format".to_owned(), "json".to_owned()]);
        words.join(" ")
    }

    /// `ascribe check` and the paths, or standard input, as given.
    fn on_paths(&self) -> Vec<String> {
        let mut words = vec!["ascribe".to_owned(), "check".to_owned()];
        match &self.args.path {
            Some(path) if self.args.stdin => {
                words.push("--stdin".to_owned());
                words.push("--path".to_owned());
                words.push(quote(&path.to_string_lossy()));
            }
            _ => words.extend(self.args.paths.iter().map(|p| quote(&p.to_string_lossy()))),
        }
        words
    }

    /// This command narrowed to the one code `code` instead of the codes
    /// named, without `--summary`, in the same format: what lists the
    /// problems a summary counts for that code.
    fn of_code(&self, code: &str) -> String {
        let mut words = self.on_paths();
        words.extend(self.checked());
        words.extend(["--code".to_owned(), quote(code)]);
        if self.args.format == Format::Concise {
            words.extend(["--format".to_owned(), "concise".to_owned()]);
        }
        words.join(" ")
    }

    /// This command in concise form with `--limit all`: what lists every
    /// problem a cut concise list leaves out.
    fn concise_all(&self) -> String {
        let mut words = self.on_paths();
        words.extend(self.options());
        words.extend([
            "--format".to_owned(),
            "concise".to_owned(),
            "--limit".to_owned(),
            "all".to_owned(),
        ]);
        words.join(" ")
    }
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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use ascribe_core::{FileId, Issue, Location, diagnostics};

    // No diagnostic is advice yet, so these are made advice here.
    fn reported() -> Vec<Reported> {
        let model = "spec = \"0.1\"\n[x]\n";
        let at = |start: usize| Location::new(FileId::new(0), start..start + 1);
        let issue = |start| {
            Issue::new(diagnostics::MODEL_UNKNOWN_KEY, at(start))
                .with_arg("key", "x")
                .with_arg("table", "ascribe.toml")
                .with_arg("version", "0.4.0")
        };
        let mut advice = Diagnostic::from_issue(&issue(0));
        advice.severity = Severity::Advice;
        let mut warning = Diagnostic::from_issue(&issue(model.len() - 2));
        warning.severity = Severity::Warning;
        Reported::all(vec![advice, warning])
    }

    #[test]
    fn advice_never_fails_the_check() {
        let advice = Counts {
            advice: 3,
            ..Counts::default()
        };
        assert_eq!(exit_code(advice, false), exit::OK);
        assert_eq!(exit_code(advice, true), exit::OK);
        let warning = Counts {
            warnings: 1,
            ..advice
        };
        assert_eq!(exit_code(warning, false), exit::OK);
        assert_eq!(exit_code(warning, true), exit::PROBLEMS);
    }

    #[test]
    fn advice_is_counted_apart_and_listed_last() {
        let files = FileTable::of_model("spec = \"0.1\"\n[x]\n".to_owned());
        let reported = reported();
        let counts = Counts::of_reported(&reported);
        assert_eq!(
            text::check_summary(counts, 1, &text::Covered::Every, 1, &[]),
            "checked 1 file for every build, reported on 1: 0 errors, 1 warning, 1 advice"
        );
        assert_eq!(
            text::check_summary(Counts::default(), 1, &text::Covered::Every, 1, &[]),
            "checked 1 file for every build, reported on 1: 0 errors, 0 warnings, 0 advice",
            "the summary counts advice even when there is none"
        );
        let builds = ["cloud".to_owned(), "site".to_owned(), "v2".to_owned()];
        let named = |n: usize| {
            text::check_summary(
                Counts::default(),
                3,
                &text::Covered::Named(&builds[..n]),
                1,
                &[],
            )
        };
        assert_eq!(
            named(1),
            "checked 3 files for build `cloud`, reported on 1: 0 errors, 0 warnings, 0 advice"
        );
        assert_eq!(
            named(3),
            "checked 3 files for builds `cloud`, `site` and `v2`, reported on 1: 0 errors, 0 warnings, 0 advice"
        );
        assert_eq!(
            text::check_summary(Counts::default(), 3, &text::Covered::Editor("site"), 1, &[]),
            "checked 3 files with the page-level checks of build `site` only, reported on 1: 0 errors, 0 warnings, 0 advice"
        );

        let order = concise::order(&files, &reported);
        assert_eq!(order, [1, 0], "advice comes after the warning");

        let mut out = Vec::new();
        let about = json::About {
            error: None,
            files_checked: 1,
            files_reported: 1,
            builds_checked: Vec::new(),
            acknowledged: &[],
            baselined: 0,
            summary_only: None,
            fixed: &[],
        };
        json::write(&mut out, &files, &reported, &about).unwrap();
        let report: serde_json::Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(report["summary"]["advice"], 1);
        assert_eq!(report["summary"]["warnings"], 1);
        assert_eq!(report["diagnostics"][0]["severity"], "advice");
        assert_eq!(report["diagnostics"][0]["next"], "choose");
    }
}
